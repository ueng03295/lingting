// Stub module: anthropic (no-op for 翎听)
#[tauri::command]
pub async fn get_anthropic_models() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({"models": []}))
}