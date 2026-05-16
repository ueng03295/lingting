// Stub parakeet engine commands
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex as StdMutex;
use super::ParakeetEngine;

pub static PARAKEET_ENGINE: LazyLock<StdMutex<Option<Arc<ParakeetEngine>>>> =
    LazyLock::new(|| StdMutex::new(None));

#[tauri::command]
pub async fn parakeet_init() -> Result<(), String> { Ok(()) }

#[tauri::command]
pub async fn parakeet_get_available_models() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!([]))
}

#[tauri::command]
pub async fn parakeet_load_model(_model: String) -> Result<(), String> { Ok(()) }

#[tauri::command]
pub async fn parakeet_get_current_model() -> Result<Option<String>, String> { Ok(None) }

#[tauri::command]
pub async fn parakeet_is_model_loaded() -> Result<bool, String> { Ok(false) }

#[tauri::command]
pub async fn parakeet_has_available_models() -> Result<bool, String> { Ok(false) }

#[tauri::command]
pub async fn parakeet_validate_model_ready() -> Result<bool, String> { Ok(false) }

#[tauri::command]
pub async fn parakeet_transcribe_audio(_audio: Vec<f32>, _language: Option<String>) -> Result<serde_json::Value, String> {
    Err("Parakeet engine disabled, use qwen3-asr bridge".to_string())
}

#[tauri::command]
pub async fn parakeet_get_models_directory() -> Result<String, String> {
    Ok("/dev/null".to_string())
}

#[tauri::command]
pub async fn parakeet_download_model(_model: String) -> Result<serde_json::Value, String> {
    Err("Parakeet model download disabled".to_string())
}

#[tauri::command]
pub async fn parakeet_retry_download(_model: String) -> Result<serde_json::Value, String> {
    Err("Parakeet model download disabled".to_string())
}

#[tauri::command]
pub async fn parakeet_cancel_download() -> Result<(), String> { Ok(()) }

#[tauri::command]
pub async fn parakeet_delete_corrupted_model(_model: String) -> Result<(), String> { Ok(()) }

#[tauri::command]
pub async fn open_parakeet_models_folder() -> Result<(), String> { Ok(()) }

pub fn set_models_directory(_app: &tauri::AppHandle) {}

/// Not a tauri command — called directly from engine.rs
pub async fn parakeet_validate_model_ready_with_config<R: tauri::Runtime>(_app: &tauri::AppHandle<R>) -> Result<String, String> {
    Err("Parakeet engine disabled, use qwen3-asr bridge".to_string())
}