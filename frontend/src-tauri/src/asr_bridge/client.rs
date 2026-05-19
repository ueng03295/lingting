use anyhow::{anyhow, Result};
use serde_json::Value;

/// HTTP client for communicating with the qwen3-asr FastAPI server.
pub struct ASRClient {
    base_url: String,
    client: reqwest::Client,
}

impl ASRClient {
    pub fn new(base_url: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300)) // 5 min for large audio
            .build()
            .expect("Failed to create HTTP client");
        Self { base_url, client }
    }

    /// Get ASR server status.
    pub async fn status(&self) -> Result<Value> {
        let url = format!("{}/v1/asr/status", self.base_url);
        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(anyhow!("ASR status request failed: {}", resp.status()));
        }
        let json: Value = resp.json().await?;
        Ok(json)
    }

    /// Load the ASR model into memory.
    pub async fn load_model(&self) -> Result<Value> {
        let url = format!("{}/v1/asr/load", self.base_url);
        let resp = self.client.post(&url).send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await?;
            let detail = if body.trim().is_empty() {
                format!("HTTP {} (empty response body)", status)
            } else {
                format!("{} (HTTP {})", body.trim(), status)
            };
            return Err(anyhow!("ASR load_model failed: {}", detail));
        }
        let json: Value = resp.json().await?;
        Ok(json)
    }

    /// Unload the ASR model from memory.
    pub async fn unload_model(&self) -> Result<Value> {
        let url = format!("{}/v1/asr/unload", self.base_url);
        let resp = self.client.post(&url).send().await?;
        if !resp.status().is_success() {
            let body = resp.text().await?;
            return Err(anyhow!("ASR unload_model failed: {}", body));
        }
        let json: Value = resp.json().await?;
        Ok(json)
    }

    /// Set the default transcription language.
    pub async fn set_language(&self, language: &str) -> Result<Value> {
        let url = format!("{}/v1/asr/language", self.base_url);
        let body = serde_json::json!({ "language": language });
        let resp = self.client.post(&url).json(&body).send().await?;
        if !resp.status().is_success() {
            let body = resp.text().await?;
            return Err(anyhow!("ASR set_language failed: {}", body));
        }
        let json: Value = resp.json().await?;
        Ok(json)
    }

    /// Transcribe audio bytes (WAV format).
    /// Returns the full transcribed text.
    pub async fn transcribe(&self, wav_bytes: &[u8], language: Option<&str>) -> Result<String> {
        let url = format!("{}/v1/audio/transcriptions", self.base_url);
        let part = reqwest::multipart::Part::bytes(wav_bytes.to_vec())
            .file_name("audio.wav")
            .mime_str("audio/wav")?;

        let mut form = reqwest::multipart::Form::new()
            .part("file", part)
            .text("model", "qwen3-asr-1.7b".to_string());

        if let Some(lang) = language {
            form = form.text("language", lang.to_string());
        }

        let resp = self.client.post(&url).multipart(form).send().await?;
        if !resp.status().is_success() {
            let body = resp.text().await?;
            return Err(anyhow!("ASR transcribe failed: {}", body));
        }
        let json: Value = resp.json().await?;
        json.get("text")
            .and_then(|t| t.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("Missing 'text' field in ASR response"))
    }

    /// List available ASR models.
    pub async fn list_models(&self) -> Result<Value> {
        let url = format!("{}/v1/models", self.base_url);
        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(anyhow!("ASR list_models failed: {}", resp.status()));
        }
        let json: Value = resp.json().await?;
        Ok(json)
    }
}