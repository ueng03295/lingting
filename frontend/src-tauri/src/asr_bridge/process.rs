use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::{Child, Command};

/// Manages the qwen3-asr Python server lifecycle.
pub struct ASRProcess {
    port: u16,
    server_script: PathBuf,
    venv_python: PathBuf,
    child: Option<Child>,
}

impl ASRProcess {
    pub fn new(port: u16, server_script: PathBuf, venv_python: PathBuf) -> Self {
        Self {
            port,
            server_script,
            venv_python,
            child: None,
        }
    }

    /// Start the Python ASR server.
    /// If already running, does nothing.
    pub async fn start(&mut self) -> Result<()> {
        if self.is_running().await {
            log::info!("ASR server already running on port {}", self.port);
            return Ok(());
        }

        log::info!(
            "Starting ASR server: {} {}",
            self.venv_python.display(),
            self.server_script.display()
        );

        if !self.server_script.exists() {
            return Err(anyhow!(
                "ASR server script not found: {}. Please install the ASR server.",
                self.server_script.display()
            ));
        }
        if !self.venv_python.exists() {
            return Err(anyhow!(
                "Python venv not found: {}. Please set up the venv with mlx-audio.",
                self.venv_python.display()
            ));
        }

        let child = Command::new(&self.venv_python)
            .arg(&self.server_script)
            .env("ASR_PORT", self.port.to_string())
            .env("ASR_LANG", "zh")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| anyhow!("Failed to start ASR server: {}", e))?;

        self.child = Some(child);
        log::info!("ASR server process spawned, waiting for health check...");

        // Wait for the server to come up (max 60 seconds)
        self.wait_for_ready(60).await?;
        log::info!("ASR server is ready on port {}", self.port);
        Ok(())
    }

    /// Stop the Python ASR server.
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(ref mut child) = self.child {
            let pid = child.id().unwrap_or(0);
            log::info!("Stopping ASR server (PID: {})", pid);

            // Try graceful shutdown first
            #[cfg(unix)]
            {
                unsafe {
                    libc::kill(pid as i32, libc::SIGTERM);
                }
            }
            #[cfg(not(unix))]
            {
                let _ = child.kill().await;
            }

            // Wait up to 5 seconds for graceful shutdown
            match tokio::time::timeout(std::time::Duration::from_secs(5), child.wait()).await {
                Ok(_) => log::info!("ASR server stopped gracefully"),
                Err(_) => {
                    log::warn!("ASR server did not stop in time, killing...");
                    let _ = child.kill().await;
                    let _ = child.wait().await;
                }
            }
        }
        self.child = None;
        Ok(())
    }

    /// Ensure the server is running, starting it if needed.
    pub async fn ensure_running(&mut self) -> Result<()> {
        if !self.is_running().await {
            self.start().await
        } else {
            Ok(())
        }
    }

    /// Check if the server is responding to HTTP requests.
    pub async fn is_running(&self) -> bool {
        let url = format!("http://127.0.0.1:{}/v1/asr/status", self.port);
        reqwest::Client::new()
            .get(&url)
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await
            .is_ok()
    }

    /// Poll the health endpoint until the server is ready.
    async fn wait_for_ready(&self, max_seconds: u64) -> Result<()> {
        let url = format!("http://127.0.0.1:{}/v1/asr/status", self.port);
        let client = reqwest::Client::new();
        let start = std::time::Instant::now();

        loop {
            if start.elapsed().as_secs() > max_seconds {
                return Err(anyhow!(
                    "ASR server did not become ready within {} seconds",
                    max_seconds
                ));
            }
            match client
                .get(&url)
                .timeout(std::time::Duration::from_secs(2))
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => return Ok(()),
                Ok(_) => {} // Server up but not ready yet
                Err(_) => {}  // Server not up yet
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    }
}

impl Drop for ASRProcess {
    fn drop(&mut self) {
        // Best-effort cleanup: kill the child process if still alive
        if let Some(ref mut child) = self.child {
            let _ = child.start_kill();
        }
    }
}