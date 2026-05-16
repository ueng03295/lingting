pub mod client;
pub mod commands;
pub mod process;

use client::ASRClient;
use process::ASRProcess;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Global ASR bridge state shared across Tauri commands.
pub struct ASRState {
    pub process: ASRProcess,
    pub client: ASRClient,
}

/// Shared ASR state wrapped in Arc<RwLock> for thread-safe access.
pub type SharedASRState = Arc<RwLock<ASRState>>;