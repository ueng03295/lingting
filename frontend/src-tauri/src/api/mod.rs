// API module — connects frontend Tauri commands to database layer.
// Original Meetily backend API was removed for 翎听, but database operations remain.

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Runtime, State};
use crate::state::AppState;
use crate::database::repositories::meeting::MeetingsRepository;
use crate::database::repositories::setting::SettingsRepository;
use crate::database::repositories::transcript::TranscriptsRepository;

// Nested `api` submodule for backward compatibility with `crate::api::api::` paths
pub mod api {
    pub use super::*;
}

/// Transcript configuration (used by audio pipeline)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

// Helper: default TranscriptConfig for qwen3-asr
fn default_transcript_config() -> TranscriptConfig {
    TranscriptConfig {
        provider: "openaiCompatible".to_string(),
        model: "qwen3-asr-1.7b".to_string(),
        language: Some("zh".to_string()),
        endpoint: Some("http://127.0.0.1:8765".to_string()),
        api_key: None,
        openai_compatible_endpoint: Some("http://127.0.0.1:8765".to_string()),
        openai_compatible_api_key: None,
    }
}

// ── API commands ──────────────────────────────────────────────────────────

#[tauri::command]
pub async fn api_get_transcript_config<R: Runtime>(
    _app: AppHandle<R>,
    state: State<'_, AppState>,
    _id: Option<String>,
) -> Result<Option<TranscriptConfig>, String> {
    let pool = state.db_manager.pool();
    match SettingsRepository::get_transcript_config(pool).await {
        Ok(Some(setting)) => Ok(Some(TranscriptConfig {
            provider: setting.provider.clone(),
            model: setting.model.clone(),
            // TranscriptSetting has no language column — default to zh
            language: Some("zh".to_string()),
            endpoint: setting.openai_compatible_endpoint.clone(),
            api_key: None,
            openai_compatible_endpoint: setting.openai_compatible_endpoint.clone(),
            openai_compatible_api_key: setting.openai_compatible_api_key.clone(),
        })),
        Ok(None) => Ok(Some(default_transcript_config())),
        Err(e) => {
            log::warn!("Failed to load transcript config from DB, using defaults: {}", e);
            Ok(Some(default_transcript_config()))
        }
    }
}

#[tauri::command]
pub async fn api_save_transcript_config(
    config: TranscriptConfig,
    _app: AppHandle<tauri::Wry>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = state.db_manager.pool();
    SettingsRepository::save_transcript_config(pool, &config.provider, &config.model)
        .await
        .map_err(|e| format!("Failed to save transcript config: {}", e))?;
    // Save endpoint and API key
    SettingsRepository::save_openai_compatible_config(
        pool,
        config.openai_compatible_endpoint.as_deref(),
        config.openai_compatible_api_key.as_deref(),
    )
    .await
    .map_err(|e| format!("Failed to save openai compatible config: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn api_get_model_config<R: Runtime>(
    _app: AppHandle<R>,
    state: State<'_, AppState>,
    _id: Option<String>,
) -> Result<Option<ModelConfig>, String> {
    let pool = state.db_manager.pool();
    match SettingsRepository::get_model_config(pool).await {
        Ok(Some(setting)) => Ok(Some(ModelConfig {
            provider: setting.provider,
            model: setting.model,
        })),
        Ok(None) => Ok(Some(ModelConfig {
            provider: "qwen3-asr".to_string(),
            model: "qwen3-asr-1.7b".to_string(),
        })),
        Err(e) => {
            log::warn!("Failed to load model config: {}", e);
            Ok(Some(ModelConfig {
                provider: "qwen3-asr".to_string(),
                model: "qwen3-asr-1.7b".to_string(),
            }))
        }
    }
}

#[tauri::command]
pub async fn api_save_model_config(
    config: ModelConfig,
    _app: AppHandle<tauri::Wry>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = state.db_manager.pool();
    SettingsRepository::save_model_config(pool, &config.provider, &config.model, "qwen3-asr-1.7b", None)
        .await
        .map_err(|e| format!("Failed to save model config: {}", e))
}

#[tauri::command]
pub async fn api_get_meetings(
    _app: AppHandle<tauri::Wry>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let pool = state.db_manager.pool();
    let meetings = MeetingsRepository::get_meetings(pool)
        .await
        .map_err(|e| format!("Failed to get meetings: {}", e))?;
    Ok(serde_json::to_value(meetings).unwrap_or(serde_json::json!([])))
}

#[tauri::command]
pub async fn api_search_transcripts(
    query: String,
    _app: AppHandle<tauri::Wry>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let pool = state.db_manager.pool();
    let results = TranscriptsRepository::search_transcripts(pool, &query)
        .await
        .unwrap_or_default();
    Ok(serde_json::to_value(&results).unwrap_or(serde_json::json!([])))
}

#[tauri::command]
pub async fn api_get_meeting(
    id: String,
    _app: AppHandle<tauri::Wry>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let pool = state.db_manager.pool();
    match MeetingsRepository::get_meeting(pool, &id).await {
        Ok(Some(meeting)) => Ok(serde_json::to_value(meeting).unwrap_or(serde_json::json!({}))),
        Ok(None) => Ok(serde_json::json!({})),
        Err(e) => Err(format!("Failed to get meeting: {}", e)),
    }
}

#[tauri::command]
pub async fn api_get_meeting_metadata(
    id: String,
    _app: AppHandle<tauri::Wry>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let pool = state.db_manager.pool();
    match MeetingsRepository::get_meeting_metadata(pool, &id).await {
        Ok(Some(metadata)) => Ok(serde_json::to_value(metadata).unwrap_or(serde_json::json!({}))),
        Ok(None) => Ok(serde_json::json!({})),
        Err(e) => Err(format!("Failed to get meeting metadata: {}", e)),
    }
}

#[tauri::command]
pub async fn api_get_meeting_transcripts(
    id: String,
    _app: AppHandle<tauri::Wry>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let pool = state.db_manager.pool();
    match MeetingsRepository::get_meeting(pool, &id).await {
        Ok(Some(meeting)) => Ok(serde_json::to_value(meeting).unwrap_or(serde_json::json!({}))),
        Ok(None) => Ok(serde_json::json!({})),
        Err(e) => Err(format!("Failed to get meeting transcripts: {}", e)),
    }
}

#[tauri::command]
pub async fn api_delete_meeting(
    id: String,
    _app: AppHandle<tauri::Wry>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = state.db_manager.pool();
    MeetingsRepository::delete_meeting(pool, &id)
        .await
        .map(|_| ())
        .map_err(|e| format!("Failed to delete meeting: {}", e))
}

#[tauri::command]
pub async fn api_save_meeting_title(
    id: String,
    title: String,
    _app: AppHandle<tauri::Wry>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = state.db_manager.pool();
    MeetingsRepository::update_meeting_title(pool, &id, &title)
        .await
        .map(|_| ())
        .map_err(|e| format!("Failed to save meeting title: {}", e))
}

#[tauri::command]
pub async fn api_save_transcript(
    id: String,
    _transcript: serde_json::Value,
    _app: AppHandle<tauri::Wry>,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    // transcript is expected to be a TranscriptSegment or similar JSON
    // For now, just log it — the audio pipeline saves transcripts directly
    log::info!("Save transcript called for meeting {}", id);
    Ok(())
}

#[tauri::command]
pub async fn api_get_transcript_api_key(
    _app: AppHandle<tauri::Wry>,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    let pool = state.db_manager.pool();
    SettingsRepository::get_transcript_api_key(pool, "openaiCompatible")
        .await
        .map_err(|e| format!("Failed to get API key: {}", e))
}

#[tauri::command]
pub async fn api_get_api_key(
    _app: AppHandle<tauri::Wry>,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    let pool = state.db_manager.pool();
    SettingsRepository::get_api_key(pool, "openai")
        .await
        .map_err(|e| format!("Failed to get API key: {}", e))
}

#[tauri::command]
pub async fn api_get_profile(
    _app: AppHandle<tauri::Wry>,
    _state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({}))
}

#[tauri::command]
pub async fn api_save_profile(
    _profile: serde_json::Value,
    _app: AppHandle<tauri::Wry>,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn api_update_profile(
    _profile: serde_json::Value,
    _app: AppHandle<tauri::Wry>,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn open_meeting_folder(id: String, app: AppHandle<tauri::Wry>) -> Result<(), String> {
    log::info!("Open meeting folder requested for: {}", id);
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
pub async fn open_external_url(url: String) -> Result<(), String> {
    log::info!("Open external URL requested: {}", url);
    Ok(())
}

#[tauri::command]
pub async fn api_save_custom_openai_config(
    endpoint: String,
    api_key: Option<String>,
    model: String,
    max_tokens: Option<i32>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    _app: AppHandle<tauri::Wry>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = state.db_manager.pool();
    let custom_config = crate::summary::CustomOpenAIConfig {
        endpoint,
        api_key,
        model,
        max_tokens,
        temperature,
        top_p,
    };
    SettingsRepository::save_custom_openai_config(pool, &custom_config)
        .await
        .map_err(|e| format!("Failed to save custom OpenAI config: {}", e))
}

#[tauri::command]
pub async fn api_get_custom_openai_config(
    _app: AppHandle<tauri::Wry>,
    state: State<'_, AppState>,
) -> Result<Option<serde_json::Value>, String> {
    let pool = state.db_manager.pool();
    match SettingsRepository::get_custom_openai_config(pool).await {
        Ok(Some(config)) => Ok(Some(serde_json::to_value(&config).unwrap_or_default())),
        Ok(None) => Ok(None),
        Err(e) => Err(format!("Failed to get custom OpenAI config: {}", e)),
    }
}

#[tauri::command]
pub async fn api_test_custom_openai_connection(
    _endpoint: String,
    _api_key: Option<String>,
    _model: Option<String>,
) -> Result<Vec<String>, String> {
    Ok(vec![])
}

#[tauri::command]
pub async fn api_test_openai_compatible_transcription(
    endpoint: String,
    api_key: Option<String>,
) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/v1/models", endpoint.trim_end_matches('/'));

    let mut req = client.get(&url);
    if let Some(key) = api_key.as_deref() {
        req = req.header("Authorization", format!("Bearer {}", key));
    }

    match req.timeout(std::time::Duration::from_secs(5)).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                Ok(serde_json::json!({"success": true, "status": "connected"}))
            } else {
                Ok(serde_json::json!({"success": false, "status": format!("HTTP {}", resp.status())}))
            }
        }
        Err(e) => Ok(serde_json::json!({"success": false, "status": format!("Connection failed: {}", e)})),
    }
}