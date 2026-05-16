// Stub module: whisper_engine (removed for 翎听, replaced by qwen3-asr bridge)
// Minimal stubs to satisfy audio pipeline references.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

pub mod commands;
pub mod parallel_commands;

/// Model status enum (stub with all variants needed by audio pipeline)
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

/// Stub WhisperEngine — no-op, all methods return defaults
#[derive(Clone)]
pub struct WhisperEngine;

impl WhisperEngine {
    pub fn new() -> Self { Self }
    pub async fn get_current_model(&self) -> Option<String> { None }
    pub async fn unload_model(&self) -> bool { true }
    pub async fn is_model_loaded(&self) -> Result<bool, String> { Ok(false) }
    pub async fn load_model(&self, _model: &str) -> Result<bool, String> { Ok(false) }
    pub async fn discover_models(&self) -> Result<Vec<ModelInfo>, String> { Ok(vec![]) }
    pub async fn transcribe_audio(&self, _audio: Vec<f32>) -> Result<String, String> {
        Err("Whisper engine disabled, use qwen3-asr bridge".to_string())
    }
    pub async fn transcribe_audio_with_confidence(
        &self,
        _audio: Vec<f32>,
        _language: Option<String>,
    ) -> Result<(String, f32, bool), String> {
        Err("Whisper engine disabled".to_string())
    }
}