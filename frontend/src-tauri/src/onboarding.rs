// Stub module: onboarding (minimal for 翎听)
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingStatus {
    pub completed: bool,
    pub step: Option<String>,
}

#[tauri::command]
pub async fn get_onboarding_status(_app: AppHandle<tauri::Wry>) -> Result<OnboardingStatus, String> {
    Ok(OnboardingStatus { completed: true, step: None })
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
    Ok(OnboardingStatus { completed: true, step: None })
}