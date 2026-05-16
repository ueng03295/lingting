# LingListen (fork of Meetily) — Changes

## v0.3.0-ling.1 — 2026-05-16

### Branding
- Renamed product from "Meetily" to "LingListen (翎听)"
- Bundle identifier: `com.linglisten.ai`
- Cargo package name: `linglisten`
- NPM package name: `linglisten`

### New Feature: OpenAI-Compatible Transcription Provider
- Added `openaiCompatible` provider type alongside existing `localWhisper`, `deepgram`, etc.
- New Rust backend: `openai_compatible_provider.rs` — implements `TranscriptionProvider` trait
  - Sends audio via `/v1/audio/transcriptions` API endpoint
  - Supports optional API key authentication
  - Converts f32 audio samples to WAV format (16kHz mono)
  - Connection test endpoint with UI feedback
- New fields in `TranscriptConfig`: `openai_compatible_endpoint`, `openai_compatible_api_key`
- Database migration: adds `openaiCompatibleEndpoint` and `openaiCompatibleApiKey` columns to `transcript_settings`
- Frontend `TranscriptSettings.tsx`: full settings UI for endpoint, API key, model, and connection test with visual feedback (Loader2/CheckCircle2/XCircle icons)
- Frontend `Sidebar`, `ConfigContext`, `configService.ts`: routing and state management for new provider

### Build Fixes (from upstream)
- Upgraded Tauri dependency from 2.6.2 to 2.11 in Cargo.toml (matching Cargo.lock resolution)
- Added `__tauri_command_name_*` re-exports in `summary/mod.rs` and `summary_engine/mod.rs` for Tauri 2.6+ macro compatibility
- Fixed `sqlx::query_scalar` generic argument error in `setting.rs`
- Installed cmake (brew) for whisper-rs-sys build
- Used `DEVELOPER_DIR` env var to select Xcode.app (bypassing xcode-select)
- Built llama-helper release binary and placed in `binaries/`