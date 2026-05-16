// Stub whisper engine commands
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex as StdMutex;
use super::WhisperEngine;

pub static WHISPER_ENGINE: LazyLock<StdMutex<Option<Arc<WhisperEngine>>>> =
    LazyLock::new(|| StdMutex::new(None));

#[tauri::command]
pub async fn whisper_init() -> Result<(), String> { Ok(()) }

#[tauri::command]
pub async fn whisper_get_available_models() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!([]))
}

#[tauri::command]
pub async fn whisper_load_model(_model: String) -> Result<(), String> { Ok(()) }

#[tauri::command]
pub async fn whisper_get_current_model() -> Result<Option<String>, String> { Ok(None) }

#[tauri::command]
pub async fn whisper_is_model_loaded() -> Result<bool, String> { Ok(false) }

#[tauri::command]
pub async fn whisper_has_available_models() -> Result<bool, String> { Ok(true) }

#[tauri::command]
pub async fn whisper_validate_model_ready() -> Result<bool, String> { Ok(false) }

#[tauri::command]
pub async fn whisper_transcribe_audio(_audio: Vec<f32>, _language: Option<String>) -> Result<serde_json::Value, String> {
    Err("Whisper engine disabled, use qwen3-asr bridge".to_string())
}

#[tauri::command]
pub async fn whisper_get_models_directory() -> Result<String, String> {
    Ok("/dev/null".to_string())
}

#[tauri::command]
pub async fn whisper_download_model(_model: String) -> Result<serde_json::Value, String> {
    Err("Whisper model download disabled".to_string())
}

#[tauri::command]
pub async fn whisper_cancel_download() -> Result<(), String> { Ok(()) }

#[tauri::command]
pub async fn whisper_delete_corrupted_model(_model: String) -> Result<(), String> { Ok(()) }

pub fn set_models_directory(_app: &tauri::AppHandle) {}

#[tauri::command]
pub async fn open_models_folder() -> Result<(), String> { Ok(()) }

/// Not a tauri command — called directly from engine.rs
pub async fn whisper_validate_model_ready_with_config<R: tauri::Runtime>(_app: &tauri::AppHandle<R>) -> Result<String, String> {
    Err("Whisper engine disabled, use qwen3-asr bridge".to_string())
}