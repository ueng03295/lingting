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
    pub folder_path: Option<String>,
    pub transcripts: Vec<MeetingTranscript>,
}

/// Transcript search result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    meeting_id: String,
    _app: AppHandle<tauri::Wry>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let pool = state.db_manager.pool();
    match MeetingsRepository::get_meeting(pool, &meeting_id).await {
        Ok(Some(meeting)) => Ok(serde_json::to_value(meeting).unwrap_or(serde_json::json!({}))),
        Ok(None) => Ok(serde_json::json!({})),
        Err(e) => Err(format!("Failed to get meeting: {}", e)),
    }
}

#[tauri::command]
pub async fn api_get_meeting_metadata(
    meeting_id: String,
    _app: AppHandle<tauri::Wry>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let pool = state.db_manager.pool();
    match MeetingsRepository::get_meeting_metadata(pool, &meeting_id).await {
        Ok(Some(metadata)) => Ok(serde_json::to_value(metadata).unwrap_or(serde_json::json!({}))),
        Ok(None) => Ok(serde_json::json!({})),
        Err(e) => Err(format!("Failed to get meeting metadata: {}", e)),
    }
}

#[tauri::command]
pub async fn api_get_meeting_transcripts(
    meeting_id: String,
    _app: AppHandle<tauri::Wry>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let pool = state.db_manager.pool();
    match MeetingsRepository::get_meeting(pool, &meeting_id).await {
        Ok(Some(meeting)) => Ok(serde_json::to_value(meeting).unwrap_or(serde_json::json!({}))),
        Ok(None) => Ok(serde_json::json!({})),
        Err(e) => Err(format!("Failed to get meeting transcripts: {}", e)),
    }
}

#[tauri::command]
pub async fn api_delete_meeting(
    meeting_id: String,
    _app: AppHandle<tauri::Wry>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = state.db_manager.pool();
    MeetingsRepository::delete_meeting(pool, &meeting_id)
        .await
        .map(|_| ())
        .map_err(|e| format!("Failed to delete meeting: {}", e))
}

#[tauri::command]
pub async fn api_save_meeting_title(
    meeting_id: String,
    title: String,
    _app: AppHandle<tauri::Wry>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = state.db_manager.pool();
    MeetingsRepository::update_meeting_title(pool, &meeting_id, &title)
        .await
        .map(|_| ())
        .map_err(|e| format!("Failed to save meeting title: {}", e))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMeetingResponse {
    pub meeting_id: String,
}

#[tauri::command]
pub async fn api_save_transcript(
    meeting_title: String,
    transcripts: Vec<TranscriptSegment>,
    folder_path: Option<String>,
    _app: AppHandle<tauri::Wry>,
    state: State<'_, AppState>,
) -> Result<SaveMeetingResponse, String> {
    let pool = state.db_manager.pool();
    log::info!(
        "Saving meeting '{}' with {} transcript segments, folder_path: {:?}",
        meeting_title,
        transcripts.len(),
        folder_path
    );

    let meeting_id = TranscriptsRepository::save_transcript(
        pool,
        &meeting_title,
        &transcripts,
        folder_path,
    )
    .await
    .map_err(|e| format!("Failed to save meeting: {}", e))?;

    log::info!("Meeting saved successfully with id: {}", meeting_id);
    Ok(SaveMeetingResponse { meeting_id })
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
pub async fn open_meeting_folder(
    meeting_id: String,
    app: AppHandle<tauri::Wry>,
) -> Result<(), String> {
    log::info!("Open meeting folder requested for: {}", meeting_id);

    // Look up the meeting's folder_path from database
    let state = app.state::<AppState>();
    let pool = state.db_manager.pool();
    match MeetingsRepository::get_meeting(pool, &meeting_id).await {
        Ok(Some(meeting)) => {
            if let Some(folder_path) = meeting.folder_path {
                log::info!("Opening meeting folder: {}", folder_path);
                #[cfg(target_os = "macos")]
                { let _ = std::process::Command::new("open").arg(&folder_path).spawn(); }
                #[cfg(target_os = "windows")]
                { let _ = std::process::Command::new("explorer").arg(&folder_path).spawn(); }
                #[cfg(target_os = "linux")]
                { let _ = std::process::Command::new("xdg-open").arg(&folder_path).spawn(); }
            } else {
                log::warn!("Meeting {} has no folder_path", meeting_id);
            }
        }
        Ok(None) => {
            log::warn!("Meeting {} not found", meeting_id);
        }
        Err(e) => {
            log::error!("Failed to look up meeting {}: {}", meeting_id, e);
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn open_external_url(url: String) -> Result<(), String> {
    log::info!("Open external URL requested: {}", url);
    #[cfg(target_os = "macos")]
    { let _ = std::process::Command::new("open").arg(&url).spawn(); }
    #[cfg(target_os = "windows")]
    { let _ = std::process::Command::new("explorer").arg(&url).spawn(); }
    #[cfg(target_os = "linux")]
    { let _ = std::process::Command::new("xdg-open").arg(&url).spawn(); }
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
    endpoint: String,
    api_key: Option<String>,
    model: Option<String>,
) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/v1/models", endpoint.trim_end_matches('/'));

    let mut req = client.get(&url);
    if let Some(key) = api_key.as_deref() {
        req = req.header("Authorization", format!("Bearer {}", key));
    }

    match req.timeout(std::time::Duration::from_secs(10)).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                let body: serde_json::Value = resp.json().await.map_err(|e| format!("Failed to parse response: {}", e))?;
                // Extract model IDs from OpenAI-compatible /v1/models response
                let models = body.get("data")
                    .and_then(|d| d.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|m| m.get("id").and_then(|id| id.as_str()).map(String::from))
                            .collect::<Vec<String>>()
                    })
                    .unwrap_or_default();

                let model_available = model.as_ref().map_or(true, |m| m.is_empty() || models.iter().any(|id| id == m));

                Ok(serde_json::json!({
                    "status": if model_available { "connected" } else { "model_not_found" },
                    "message": if model_available {
                        format!("Connection successful! {} model(s) available.", models.len())
                    } else {
                        format!("Connected, but model '{}' not found. Available: {}", model.as_ref().unwrap(), models.join(", "))
                    },
                    "models": models,
                }))
            } else {
                Ok(serde_json::json!({
                    "status": "error",
                    "message": format!("HTTP {}", resp.status()),
                }))
            }
        }
        Err(e) => Ok(serde_json::json!({
            "status": "error",
            "message": format!("Connection failed: {}", e),
        })),
    }
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