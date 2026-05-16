// Stub module: console_utils (no-op for 翎听)
#[tauri::command]
pub async fn show_console() -> Result<(), String> { Ok(()) }

#[tauri::command]
pub async fn hide_console() -> Result<(), String> { Ok(()) }

#[tauri::command]
pub async fn toggle_console() -> Result<(), String> { Ok(()) }