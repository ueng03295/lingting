// Stub module: analytics (all no-ops for 翎听)

pub mod commands {
    use tauri::AppHandle;

    #[tauri::command]
    pub async fn init_analytics() -> Result<(), String> { Ok(()) }

    #[tauri::command]
    pub async fn disable_analytics() -> Result<(), String> { Ok(()) }

    #[tauri::command]
    pub async fn track_event(_event_name: String) -> Result<(), String> { Ok(()) }

    #[tauri::command]
    pub async fn identify_user(_user_id: String) -> Result<(), String> { Ok(()) }

    #[tauri::command]
    pub async fn track_meeting_started() -> Result<(), String> { Ok(()) }

    #[tauri::command]
    pub async fn track_recording_started() -> Result<(), String> { Ok(()) }

    #[tauri::command]
    pub async fn track_recording_stopped() -> Result<(), String> { Ok(()) }

    #[tauri::command]
    pub async fn track_meeting_deleted() -> Result<(), String> { Ok(()) }

    #[tauri::command]
    pub async fn track_settings_changed() -> Result<(), String> { Ok(()) }

    #[tauri::command]
    pub async fn track_feature_used(_feature: String) -> Result<(), String> { Ok(()) }

    #[tauri::command]
    pub async fn is_analytics_enabled(_app: AppHandle<tauri::Wry>) -> Result<(), String> { Ok(()) }

    #[tauri::command]
    pub async fn start_analytics_session() -> Result<(), String> { Ok(()) }

    #[tauri::command]
    pub async fn end_analytics_session() -> Result<(), String> { Ok(()) }

    #[tauri::command]
    pub async fn track_daily_active_user() -> Result<(), String> { Ok(()) }

    #[tauri::command]
    pub async fn track_user_first_launch() -> Result<(), String> { Ok(()) }

    #[tauri::command]
    pub async fn is_analytics_session_active() -> Result<(), String> { Ok(()) }

    #[tauri::command]
    pub async fn track_meeting_ended(
        _transcription_provider: String,
        _transcription_model: String,
        _summary_provider: String,
        _summary_model: String,
        _total_duration: f64,
        _active_duration: f64,
        _pause_duration: f64,
        _microphone_device_type: String,
        _system_audio_device_type: String,
        _chunks_processed: u64,
        _transcript_segments_count: u64,
        _had_fatal_error: bool,
    ) -> Result<(), String> { Ok(()) }

    #[tauri::command]
    pub async fn track_analytics_enabled() -> Result<(), String> { Ok(()) }

    #[tauri::command]
    pub async fn track_analytics_disabled() -> Result<(), String> { Ok(()) }

    #[tauri::command]
    pub async fn track_analytics_transparency_viewed() -> Result<(), String> { Ok(()) }

    #[tauri::command]
    pub async fn track_summary_generation_started() -> Result<(), String> { Ok(()) }

    #[tauri::command]
    pub async fn track_summary_generation_completed() -> Result<(), String> { Ok(()) }

    #[tauri::command]
    pub async fn track_summary_regenerated() -> Result<(), String> { Ok(()) }

    #[tauri::command]
    pub async fn track_model_changed(_old_model: String, _new_model: String) -> Result<(), String> { Ok(()) }

    #[tauri::command]
    pub async fn track_custom_prompt_used() -> Result<(), String> { Ok(()) }

    #[tauri::command]
    pub async fn is_analytics_enabled_bool(_app: AppHandle<tauri::Wry>) -> Result<bool, String> { Ok(false) }
}