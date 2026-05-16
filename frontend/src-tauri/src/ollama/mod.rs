// Stub module: ollama (no-op for 翎听)
use std::time::Duration;

pub mod metadata {
    use std::time::Duration;

    /// Stub ModelMetadataCache — always returns empty results
    pub struct ModelMetadataCache {
        _ttl: Duration,
    }

    /// Model metadata returned by cache
    #[derive(Debug, Clone)]
    pub struct ModelMetadata {
        pub context_size: usize,
    }

    impl ModelMetadataCache {
        pub fn new(_ttl: Duration) -> Self {
            Self { _ttl: _ttl }
        }

        pub fn get_model_info(&self, _model: &str) -> Option<serde_json::Value> { None }
        pub async fn get_or_fetch(&self, _model: &str, _endpoint: Option<&str>) -> Result<ModelMetadata, String> {
            // Return a sensible default
            Ok(ModelMetadata { context_size: 4096 })
        }
        pub async fn refresh(&self, _url: &str) -> Result<(), String> { Ok(()) }
    }
}

#[tauri::command]
pub async fn get_ollama_models() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({"models": []}))
}

#[tauri::command]
pub async fn pull_ollama_model(_name: String) -> Result<(), String> { Ok(()) }

#[tauri::command]
pub async fn delete_ollama_model(_name: String) -> Result<(), String> { Ok(()) }

#[tauri::command]
pub async fn get_ollama_model_context(_name: String) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({}))
}