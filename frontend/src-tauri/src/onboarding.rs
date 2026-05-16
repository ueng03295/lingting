// Stub module: onboarding (minimal for 翎听)
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingStatus {
    pub version: String,
    pub completed: bool,
    pub current_step: i32,
    pub model_status: serde_json::Value,
    pub last_updated: String,
}

fn default_onboarding_status() -> OnboardingStatus {
    OnboardingStatus {
        version: "1.0".to_string(),
        completed: true,
        current_step: 0,
        model_status: serde_json::json!({"parakeet": "available", "summary": "available"}),
        last_updated: chrono::Utc::now().to_rfc3339(),
    }
}

#[tauri::command]
pub async fn get_onboarding_status(_app: AppHandle<tauri::Wry>) -> Result<OnboardingStatus, String> {
    Ok(default_onboarding_status())
}

#[tauri::command]
pub async fn save_onboarding_status_cmd(_status: OnboardingStatus, _app: AppHandle<tauri::Wry>) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn reset_onboarding_status_cmd(_app: AppHandle<tauri::Wry>) -> Result<(), String> { Ok(()) }

#[tauri::command]
pub async fn complete_onboarding(_app: AppHandle<tauri::Wry>) -> Result<(), String> { Ok(()) }

pub async fn load_onboarding_status<R: tauri::Runtime>(_app: &AppHandle<R>) -> Result<OnboardingStatus, String> {
    Ok(default_onboarding_status())
}