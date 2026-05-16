use crate::asr_bridge::client::ASRClient;
use crate::asr_bridge::process::ASRProcess;
use crate::asr_bridge::{ASRState, SharedASRState};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Create the shared ASR state for the app.
/// Searches common locations for the qwen3-asr server script and venv.
pub fn create_asr_state() -> SharedASRState {
    let client = ASRClient::new("http://127.0.0.1:8765".to_string());
    let process = ASRProcess::new(
        8765,
        find_server_script(),
        find_venv_python(),
    );
    Arc::new(RwLock::new(ASRState { process, client }))
}

fn find_server_script() -> PathBuf {
    // Bundled inside app resources first, then fallback to dev locations
    let candidates = vec![
        // App resource bundle (production)
        PathBuf::from("asr-server/qwen3_asr_server.py"),
        // Dev locations
        PathBuf::from("/Users/q/Desktop/会议纪要系统/qwen3_asr_server.py"),
        dirs::data_dir()
            .unwrap_or_default()
            .join("com.linglisten.ai/asr-server/qwen3_asr_server.py"),
    ];
    for p in &candidates {
        if p.exists() {
            log::info!("Found ASR server script: {}", p.display());
            return p.clone();
        }
    }
    // Return first candidate as default (will fail with clear error if not found)
    log::warn!(
        "ASR server script not found in any candidate location, using default"
    );
    candidates.into_iter().next().unwrap_or_default()
}

fn find_venv_python() -> PathBuf {
    let candidates = vec![
        // Meeting-minutes venv (current setup)
        PathBuf::from("/Users/q/hermes book vault/Vespera/scripts/meeting-minutes/venv/bin/python3"),
        // App-bundled venv (future)
        dirs::data_dir()
            .unwrap_or_default()
            .join("com.linglisten.ai/asr-server/venv/bin/python3"),
    ];
    for p in &candidates {
        if p.exists() {
            log::info!("Found venv Python: {}", p.display());
            return p.clone();
        }
    }
    log::warn!("venv Python not found, falling back to system python3");
    PathBuf::from("python3")
}

// ── Tauri commands ──────────────────────────────────────────────────────

#[tauri::command]
pub async fn asr_start(state: tauri::State<'_, SharedASRState>) -> Result<(), String> {
    let mut guard = state.write().await;
    guard.process.start().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn asr_stop(state: tauri::State<'_, SharedASRState>) -> Result<(), String> {
    let mut guard = state.write().await;
    guard.process.stop().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn asr_status(state: tauri::State<'_, SharedASRState>) -> Result<serde_json::Value, String> {
    let guard = state.read().await;
    guard.client.status().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn asr_load_model(state: tauri::State<'_, SharedASRState>) -> Result<serde_json::Value, String> {
    let guard = state.read().await;
    guard.client.load_model().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn asr_unload_model(state: tauri::State<'_, SharedASRState>) -> Result<serde_json::Value, String> {
    let guard = state.read().await;
    guard.client.unload_model().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn asr_set_language(
    state: tauri::State<'_, SharedASRState>,
    language: String,
) -> Result<serde_json::Value, String> {
    let guard = state.read().await;
    guard.client.set_language(&language).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn asr_ensure_running(state: tauri::State<'_, SharedASRState>) -> Result<(), String> {
    let mut guard = state.write().await;
    guard.process.ensure_running().await.map_err(|e| e.to_string())
}