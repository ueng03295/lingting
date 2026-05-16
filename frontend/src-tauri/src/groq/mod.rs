// Stub module: groq (no-op for 翎听)
#[tauri::command]
pub async fn get_groq_models() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({"models": []}))
}