// Stub module: api
// The original Meetily backend API module has been removed for 翎听.
// We keep minimal type definitions needed by the audio pipeline.

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime, State};
use crate::state::AppState;

// Nested `api` submodule for backward compatibility with `crate::api::api::` paths
pub mod api {
    pub use super::*;
}

/// Transcript configuration (used by audio pipeline)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptConfig {
    pub provider: String,
    pub model: String,
    pub language: Option<String>,
    pub endpoint: Option<String>,
    pub api_key: Option<String>,
    pub openai_compatible_endpoint: Option<String>,
    pub openai_compatible_api_key: Option<String>,
}

/// Model configuration (used by audio pipeline)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub provider: String,
    pub model: String,
}

/// Transcript segment (re-exported for compatibility)
pub use crate::audio::recording_saver::TranscriptSegment;

/// Meeting transcript entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingTranscript {
    pub id: String,
    pub text: String,
    pub timestamp: String,
    pub audio_start_time: f64,
    pub audio_end_time: f64,
    pub duration: f64,
}

/// Meeting details with transcripts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingDetails {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub transcripts: Vec<MeetingTranscript>,
}

/// Transcript search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptSearchResult {
    pub id: String,
    pub title: String,
    pub match_context: String,
    pub timestamp: String,
}

// ── Stub API commands (return defaults) ──────────────────────────────────

#[tauri::command]
pub async fn api_get_transcript_config<R: Runtime>(
    _app: AppHandle<R>,
    _state: State<'_, AppState>,
    _id: Option<String>,
) -> Result<Option<TranscriptConfig>, String> {
    Ok(Some(TranscriptConfig {
        provider: "openai_compatible".to_string(),
        model: "qwen3-asr-1.7b".to_string(),
        language: Some("zh".to_string()),
        endpoint: Some("http://127.0.0.1:8765".to_string()),
        api_key: None,
        openai_compatible_endpoint: Some("http://127.0.0.1:8765".to_string()),
        openai_compatible_api_key: None,
    }))
}

#[tauri::command]
pub async fn api_get_model_config<R: Runtime>(
    _app: AppHandle<R>,
    _state: State<'_, AppState>,
    _id: Option<String>,
) -> Result<Option<ModelConfig>, String> {
    Ok(Some(ModelConfig {
        provider: "qwen3-asr".to_string(),
        model: "qwen3-asr-1.7b".to_string(),
    }))
}

#[tauri::command]
pub async fn api_get_meetings(_app: AppHandle<tauri::Wry>, _state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({"meetings": []}))
}

#[tauri::command]
pub async fn api_search_transcripts(_query: String, _app: AppHandle<tauri::Wry>, _state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({"results": []}))
}

#[tauri::command]
pub async fn api_get_profile(_app: AppHandle<tauri::Wry>, _state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({}))
}

#[tauri::command]
pub async fn api_save_profile(_profile: serde_json::Value, _app: AppHandle<tauri::Wry>, _state: State<'_, AppState>) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn api_update_profile(_profile: serde_json::Value, _app: AppHandle<tauri::Wry>, _state: State<'_, AppState>) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn api_save_model_config(_config: ModelConfig, _app: AppHandle<tauri::Wry>, _state: State<'_, AppState>) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn api_get_api_key(_app: AppHandle<tauri::Wry>, _state: State<'_, AppState>) -> Result<Option<String>, String> {
    Ok(None)
}

#[tauri::command]
pub async fn api_save_transcript_config(_config: TranscriptConfig, _app: AppHandle<tauri::Wry>, _state: State<'_, AppState>) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn api_get_transcript_api_key(_app: AppHandle<tauri::Wry>, _state: State<'_, AppState>) -> Result<Option<String>, String> {
    Ok(None)
}

#[tauri::command]
pub async fn api_delete_meeting(_id: String, _app: AppHandle<tauri::Wry>, _state: State<'_, AppState>) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn api_get_meeting(_id: String, _app: AppHandle<tauri::Wry>, _state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({}))
}

#[tauri::command]
pub async fn api_get_meeting_metadata(_id: String, _app: AppHandle<tauri::Wry>, _state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({}))
}

#[tauri::command]
pub async fn api_get_meeting_transcripts(_id: String, _app: AppHandle<tauri::Wry>, _state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({}))
}

#[tauri::command]
pub async fn api_save_meeting_title(_id: String, _title: String, _app: AppHandle<tauri::Wry>, _state: State<'_, AppState>) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn api_save_transcript(_id: String, _transcript: serde_json::Value, _app: AppHandle<tauri::Wry>, _state: State<'_, AppState>) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn open_meeting_folder(_id: String, _app: AppHandle<tauri::Wry>) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn test_backend_connection(_url: String, _key: Option<String>) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({"connected": false}))
}

#[tauri::command]
pub async fn debug_backend_connection(_url: String, _key: Option<String>) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({"connected": false}))
}

#[tauri::command]
pub async fn open_external_url(_url: String) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn api_save_custom_openai_config(_config: serde_json::Value, _app: AppHandle<tauri::Wry>, _state: State<'_, AppState>) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn api_get_custom_openai_config(_app: AppHandle<tauri::Wry>, _state: State<'_, AppState>) -> Result<Option<serde_json::Value>, String> {
    Ok(None)
}

#[tauri::command]
pub async fn api_test_custom_openai_connection(_config: serde_json::Value) -> Result<Vec<String>, String> {
    Ok(vec![])
}

#[tauri::command]
pub async fn api_test_openai_compatible_transcription(_config: serde_json::Value) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({"success": false}))
}