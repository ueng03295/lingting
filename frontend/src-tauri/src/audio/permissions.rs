// macOS audio permissions handling
use anyhow::Result;
use log::{info, warn, error};

#[cfg(target_os = "macos")]
use std::process::Command;

/// Check if the app has Audio Capture permission (required for Core Audio taps on macOS 14.4+)
///
/// NOTE: There is no public macOS API to check the Audio Capture permission.
/// The private TCC framework (kTCCServiceAudioCapture) was tested but caused runtime
/// issues with hardened runtime / code signing. Instead, we rely on:
///   1. macOS automatically showing the permission dialog when creating a Core Audio tap
///   2. Silence detection in stream.rs to detect denied permission (tap returns all zeros)
///   3. Frontend event (system-audio-silence) to show a user-visible warning
///
/// Audio Capture permission is separate from Screen Recording permission on macOS 14.4+.
/// CoreAudio Process Tap (AudioHardwareCreateProcessTap) requires Audio Capture permission,
/// NOT Screen Recording permission. If denied, the tap creates successfully but returns
/// silence (all zeros).
#[cfg(target_os = "macos")]
pub fn check_screen_recording_permission() -> bool {
    info!("🔐 Audio Capture permission check: relying on silence detection (no public API available)");
    info!("📍 If permission is denied, the tap will return silence and stream.rs will emit a warning");
    // Always return true — the silence detection in the Core Audio stream task
    // will catch the case where permission is actually denied.
    // The macOS permission dialog appears automatically when creating a Core Audio tap.
    true
}

#[cfg(not(target_os = "macos"))]
pub fn check_screen_recording_permission() -> bool {
    true // Not required on other platforms
}

/// Request Audio Capture permission from the user
/// Opens System Settings to the Privacy & Security → Audio Capture page.
/// On macOS 14.4+, the correct section is "Audio Capture" (not Screen Recording).
#[cfg(target_os = "macos")]
pub fn request_screen_recording_permission() -> Result<()> {
    info!("🔐 Opening System Settings for Audio Capture permission...");

    // Try to open System Settings directly to the Audio Capture section
    // macOS 14.4+ has a dedicated "Audio Capture" section under Privacy & Security
    // The URL scheme uses com.apple.preference.security for the main Privacy page
    // Note: There's no guaranteed direct URL for Audio Capture specifically,
    // but we try the privacy URL first, then fall back to the general page
    let result = Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security")
        .spawn();

    match result {
        Ok(_) => {
            info!("✅ Opened System Settings - navigate to Privacy & Security → Audio Capture");
            info!("👉 Enable Audio Capture permission, then restart the app");
            Ok(())
        }
        Err(e) => {
            error!("❌ Failed to open System Settings: {}", e);
            Err(anyhow::anyhow!("Failed to open System Settings: {}", e))
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn request_screen_recording_permission() -> Result<()> {
    Ok(()) // Not required on other platforms
}

/// Check and request Audio Capture permission if not granted
/// Returns true if permission is granted, false otherwise
pub fn ensure_screen_recording_permission() -> bool {
    if check_screen_recording_permission() {
        return true;
    }

    warn!("Audio Capture permission not granted - requesting...");

    if let Err(e) = request_screen_recording_permission() {
        error!("Failed to request Audio Capture permission: {}", e);
        return false;
    }

    false // Permission will be granted after restart
}

/// Tauri command to check Screen Recording permission
#[tauri::command]
pub async fn check_screen_recording_permission_command() -> bool {
    check_screen_recording_permission()
}

/// Tauri command to request Screen Recording permission
#[tauri::command]
pub async fn request_screen_recording_permission_command() -> Result<(), String> {
    request_screen_recording_permission()
        .map_err(|e| e.to_string())
}

/// Trigger system audio permission request and verify it was granted
/// Returns Ok(true) if permission granted (tap created successfully), Ok(false) if denied
#[cfg(target_os = "macos")]
pub fn trigger_system_audio_permission() -> Result<bool> {
    info!("🔐 Triggering Audio Capture permission request...");

    // Try to create a Core Audio capture - this triggers the permission dialog
    // if NSAudioCaptureUsageDescription is present in Info.plist
    // NOTE: We only create the tap, don't start streaming - similar to mic permission approach
    match crate::audio::capture::CoreAudioCapture::new() {
        Ok(_capture) => {
            info!("✅ Core Audio tap created successfully");
            // Sleep briefly to allow permission dialog to appear (if shown)
            // Similar to microphone permission handling in discovery.rs
            std::thread::sleep(std::time::Duration::from_millis(500));
            info!("✅ Audio Capture permission appears to be granted");
            // Note: On macOS, even with permission denied, tap creation may succeed
            // but audio will be silence. For onboarding, we just check tap creation.
            Ok(true)
        }
        Err(e) => {
            let error_msg = e.to_string().to_lowercase();
            if error_msg.contains("permission") || error_msg.contains("denied") {
                info!("🔐 Audio Capture permission denied");
                info!("👉 Please grant Audio Capture permission in System Settings");
                return Ok(false);
            }
            warn!("⚠️ Failed to create Core Audio tap: {}", e);
            // If tap creation fails for other reasons, still return false
            // as we can't verify permission status
            Ok(false)
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn trigger_system_audio_permission() -> Result<bool> {
    // System audio permissions not required on other platforms
    info!("System audio permissions not required on this platform");
    Ok(true)
}

/// Tauri command to trigger system audio permission request
/// Returns true if permission was granted (stream created), false if denied
#[tauri::command]
pub async fn trigger_system_audio_permission_command() -> Result<bool, String> {
    // Run in blocking task to avoid blocking the async runtime
    tokio::task::spawn_blocking(|| {
        trigger_system_audio_permission()
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_permission() {
        let has_permission = check_screen_recording_permission();
        println!("Has Screen Recording permission: {}", has_permission);
    }
}