// Stub module: notifications (minimal for 翎听)
// All notification calls are no-ops. Commands live in `commands` submodule.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tauri::AppHandle;

/// Nested notification preferences (matches frontend NotificationSettings.notification_preferences)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub show_recording_started: bool,
    pub show_recording_stopped: bool,
    pub show_recording_paused: bool,
    pub show_recording_resumed: bool,
    pub show_transcription_complete: bool,
    pub show_meeting_reminders: bool,
    pub show_system_errors: bool,
    pub meeting_reminder_minutes: Vec<i32>,
}

/// Notification settings (matches frontend interface exactly)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub recording_notifications: bool,
    pub time_based_reminders: bool,
    pub meeting_reminders: bool,
    pub respect_do_not_disturb: bool,
    pub notification_sound: bool,
    pub system_permission_granted: bool,
    pub consent_given: bool,
    pub manual_dnd_mode: bool,
    pub notification_preferences: NotificationPreferences,
}

fn default_notification_settings() -> NotificationSettings {
    NotificationSettings {
        recording_notifications: true,
        time_based_reminders: true,
        meeting_reminders: true,
        respect_do_not_disturb: true,
        notification_sound: true,
        system_permission_granted: false,
        consent_given: true,
        manual_dnd_mode: false,
        notification_preferences: NotificationPreferences {
            show_recording_started: true,
            show_recording_stopped: true,
            show_recording_paused: true,
            show_recording_resumed: true,
            show_transcription_complete: true,
            show_meeting_reminders: true,
            show_system_errors: true,
            meeting_reminder_minutes: vec![5, 10, 15],
        },
    }
}

/// Stub notification manager (no-op, thread-safe)
#[derive(Clone)]
pub struct NotificationManager;

/// Notification manager state type — no generic param needed for stub
pub type NotificationManagerState = Arc<RwLock<Option<NotificationManager>>>;

// Helper functions used by recording commands
pub async fn show_recording_started_notification<R: tauri::Runtime>(
    _app: &AppHandle<R>,
    _state: &NotificationManagerState,
    _meeting_name: Option<String>,
) -> Result<(), String> {
    Ok(())
}

pub async fn show_recording_stopped_notification<R: tauri::Runtime>(
    _app: &AppHandle<R>,
    _state: &NotificationManagerState,
) -> Result<(), String> {
    Ok(())
}

pub async fn initialize_notification_manager<R: tauri::Runtime>(
    _app: AppHandle<R>,
) -> Result<NotificationManager, String> {
    Ok(NotificationManager)
}

pub mod commands {
    use super::*;

    #[tauri::command]
    pub async fn get_notification_settings() -> Result<NotificationSettings, String> {
        Ok(default_notification_settings())
    }

    #[tauri::command]
    pub async fn set_notification_settings(_settings: NotificationSettings) -> Result<(), String> { Ok(()) }

    #[tauri::command]
    pub async fn request_notification_permission() -> Result<bool, String> { Ok(true) }

    #[tauri::command]
    pub async fn show_notification(_title: String, _body: String) -> Result<(), String> { Ok(()) }

    #[tauri::command]
    pub async fn show_test_notification() -> Result<(), String> { Ok(()) }

    #[tauri::command]
    pub async fn is_dnd_active() -> Result<bool, String> { Ok(false) }

    #[tauri::command]
    pub async fn get_system_dnd_status() -> Result<bool, String> { Ok(false) }

    #[tauri::command]
    pub async fn set_manual_dnd(_enabled: bool) -> Result<(), String> { Ok(()) }

    #[tauri::command]
    pub async fn set_notification_consent(_consent: bool) -> Result<(), String> { Ok(()) }

    #[tauri::command]
    pub async fn clear_notifications() -> Result<(), String> { Ok(()) }

    #[tauri::command]
    pub async fn is_notification_system_ready() -> Result<bool, String> { Ok(true) }

    #[tauri::command]
    pub async fn initialize_notification_manager_manual() -> Result<(), String> { Ok(()) }

    #[tauri::command]
    pub async fn test_notification_with_auto_consent() -> Result<(), String> { Ok(()) }

    #[tauri::command]
    pub async fn get_notification_stats() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({}))
    }
}