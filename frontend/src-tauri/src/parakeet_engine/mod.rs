// Stub module: parakeet_engine (removed for 翎听, replaced by qwen3-asr bridge)
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

pub mod commands;

/// Model status enum (stub)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelStatus {
    Available,
    Missing,
    Downloading { progress: f32 },
    Corrupted { error: String },
    Error(String),
    Loaded(String),
}

/// Model info (used by discover_models)
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub status: ModelStatus,
    pub path: PathBuf,
}

/// Stub ParakeetEngine — no-op
#[derive(Clone)]
pub struct ParakeetEngine;

impl ParakeetEngine {
    pub fn new() -> Self { Self }
    pub async fn get_current_model(&self) -> Option<String> { None }
    pub async fn unload_model(&self) -> bool { true }
    pub async fn is_model_loaded(&self) -> Result<bool, String> { Ok(false) }
    pub async fn load_model(&self, _model: &str) -> Result<bool, String> { Ok(false) }
    pub async fn discover_models(&self) -> Result<Vec<ModelInfo>, String> { Ok(vec![]) }
    pub async fn transcribe_audio(&self, _audio: Vec<f32>) -> Result<String, String> {
        Err("Parakeet engine disabled, use qwen3-asr bridge".to_string())
    }
}