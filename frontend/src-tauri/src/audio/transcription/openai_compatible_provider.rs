// audio/transcription/openai_compatible_provider.rs
//
// OpenAI-Compatible transcription provider implementation.
// Sends audio to a user-configured OpenAI-compatible server endpoint
// using the /v1/audio/transcriptions API.

use super::provider::{TranscriptionError, TranscriptionProvider, TranscriptResult};
use async_trait::async_trait;
use log::{info, error};
use std::sync::Arc;
use tokio::sync::RwLock;

/// OpenAI-Compatible transcription provider configuration
#[derive(Debug, Clone)]
pub struct OpenAICompatibleConfig {
    /// Base URL of the transcription server (e.g., "http://localhost:8765")
    pub endpoint: String,
    /// Optional API key for authentication
    pub api_key: Option<String>,
    /// Model name to use (e.g., "qwen3-asr-1.7b")
    pub model: String,
}

/// OpenAI-Compatible transcription provider
///
/// Implements the TranscriptionProvider trait to send audio chunks to a
/// user-configured OpenAI-compatible server for transcription.
pub struct OpenAICompatibleProvider {
    config: Arc<RwLock<OpenAICompatibleConfig>>,
    client: reqwest::Client,
}

impl OpenAICompatibleProvider {
    pub fn new(config: OpenAICompatibleConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            config: Arc::new(RwLock::new(config)),
            client,
        }
    }

    /// Update the configuration (e.g., when user changes settings)
    pub async fn update_config(&self, new_config: OpenAICompatibleConfig) {
        let mut config = self.config.write().await;
        *config = new_config;
    }

    /// Convert f32 audio samples to WAV bytes
    fn samples_to_wav(samples: &[f32], sample_rate: u32) -> Result<Vec<u8>, TranscriptionError> {
        // Convert f32 to i16 PCM
        let pcm_data: Vec<i16> = samples
            .iter()
            .map(|&s| {
                let clamped = s.max(-1.0).min(1.0);
                (clamped * 32767.0) as i16
            })
            .collect();

        let data_size = pcm_data.len() * 2; // 2 bytes per i16 sample
        let file_size = 36 + data_size; // WAV header size + data

        let mut wav = Vec::with_capacity(44 + data_size);

        // RIFF header
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(file_size as u32).to_le_bytes());
        wav.extend_from_slice(b"WAVE");

        // fmt chunk
        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&16u32.to_le_bytes()); // chunk size
        wav.extend_from_slice(&1u16.to_le_bytes()); // PCM format
        wav.extend_from_slice(&1u16.to_le_bytes()); // mono
        wav.extend_from_slice(&sample_rate.to_le_bytes());
        let byte_rate = sample_rate * 2; // sample_rate * num_channels * bits_per_sample/8
        wav.extend_from_slice(&byte_rate.to_le_bytes());
        wav.extend_from_slice(&2u16.to_le_bytes()); // block align (num_channels * bits_per_sample/8)
        wav.extend_from_slice(&16u16.to_le_bytes()); // bits per sample

        // data chunk
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&(data_size as u32).to_le_bytes());

        // PCM data
        for sample in pcm_data {
            wav.extend_from_slice(&sample.to_le_bytes());
        }

        Ok(wav)
    }
}

#[async_trait]
impl TranscriptionProvider for OpenAICompatibleProvider {
    async fn transcribe(
        &self,
        audio: Vec<f32>,
        language: Option<String>,
    ) -> std::result::Result<TranscriptResult, TranscriptionError> {
        let config = self.config.read().await;
        let endpoint = config.endpoint.trim_end_matches('/').to_string();
        let api_key = config.api_key.clone();
        let model = config.model.clone();
        drop(config); // Release lock before network call

        let url = format!("{}/v1/audio/transcriptions", endpoint);

        info!(
            "OpenAI-Compatible: Sending transcription request to {} with model {}",
            url, model
        );

        // Convert audio samples to WAV format
        let wav_data = Self::samples_to_wav(&audio, 16000)
            .map_err(|e| TranscriptionError::EngineFailed(format!("Failed to create WAV: {}", e)))?;

        // Build multipart form data
        let file_part = reqwest::multipart::Part::bytes(wav_data)
            .file_name("audio.wav")
            .mime_str("audio/wav")
            .expect("audio/wav is a valid MIME type");

        let mut form = reqwest::multipart::Form::new()
            .part("file", file_part)
            .text("model", model.clone());

        if let Some(ref lang) = language {
            form = form.text("language", lang.clone());
        }

        // Build request
        let mut request = self.client.post(&url).multipart(form);

        if let Some(ref key) = api_key {
            if !key.is_empty() {
                request = request.bearer_auth(key);
            }
        }

        // Send request
        let response = request.send().await.map_err(|e| {
            let msg = if e.is_timeout() {
                "OpenAI-Compatible transcription request timed out".to_string()
            } else if e.is_connect() {
                format!("Cannot connect to transcription server at {}. Is it running?", endpoint)
            } else {
                format!("OpenAI-Compatible transcription request failed: {}", e)
            };
            TranscriptionError::EngineFailed(msg)
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!(
                "OpenAI-Compatible transcription failed: {} {}",
                status, error_text
            );
            return Err(TranscriptionError::EngineFailed(format!(
                "Transcription server returned {}: {}",
                status, error_text
            )));
        }

        // Parse response
        let response_json: serde_json::Value = response.json().await.map_err(|e| {
            TranscriptionError::EngineFailed(format!("Failed to parse transcription response: {}", e))
        })?;

        let text = response_json["text"]
            .as_str()
            .unwrap_or("")
            .trim()
            .to_string();

        // Extract is_partial from response (streaming ASR returns partial results)
        let is_partial = response_json["is_partial"]
            .as_bool()
            .unwrap_or(false);

        // is_partial already set from top-level response field (ASR server computes it correctly)

        if text.is_empty() {
            info!("OpenAI-Compatible: Transcription returned empty text");
            return Ok(TranscriptResult {
                text: String::new(),
                confidence: None,
                is_partial,
            });
        }

        info!(
            "OpenAI-Compatible: Transcription result: '{}' (model: {}, partial: {})",
            text, model, is_partial
        );

        Ok(TranscriptResult {
            text,
            confidence: None,
            is_partial,
        })
    }

    async fn is_model_loaded(&self) -> bool {
        // For remote providers, we check connectivity
        let config = self.config.read().await;
        !config.endpoint.is_empty()
    }

    async fn get_current_model(&self) -> Option<String> {
        let config = self.config.read().await;
        Some(config.model.clone())
    }

    fn provider_name(&self) -> &'static str {
        "OpenAI-Compatible"
    }
}

/// Test connection to an OpenAI-Compatible transcription server
pub async fn test_openai_compatible_connection(
    endpoint: &str,
    api_key: Option<&str>,
) -> Result<Vec<String>, String> {
    let url = format!("{}/v1/models", endpoint.trim_end_matches('/'));

    // CRITICAL: Disable system proxy for localhost connections.
    // Corporate proxies (e.g., Clash, Surge, Charles) intercept localhost
    // requests and return 502 Bad Gateway when the target is not routable.
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .no_proxy()
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let mut request = client.get(&url);
    if let Some(key) = api_key {
        request = request.bearer_auth(key);
    }

    let response = request.send().await.map_err(|e| {
        if e.is_timeout() {
            "Connection timed out. Please check the endpoint URL.".to_string()
        } else if e.is_connect() {
            "Could not connect to the server. Please verify the URL and that the server is running.".to_string()
        } else {
            format!("Connection failed: {}", e)
        }
    })?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("Server returned {}: {}", status, error_text));
    }

    let body: serde_json::Value = response.json().await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let models: Vec<String> = body["data"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m["id"].as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    Ok(models)
}