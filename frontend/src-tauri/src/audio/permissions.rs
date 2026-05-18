// macOS audio permissions handling
use anyhow::Result;
use log::{info, warn, error};

#[cfg(target_os = "macos")]
use std::process::Command;

/// Check if the app has Audio Capture permission (required for Core Audio taps on macOS 14.4+)
///
/// Uses the private TCC framework to check kTCCServiceAudioCapture status.
/// Falls back to returning true (optimistic) if TCC framework is unavailable,
/// because the actual permission dialog is triggered automatically when creating a tap.
///
/// Audio Capture permission is separate from Screen Recording permission on macOS 14.4+.
/// CoreAudio Process Tap (AudioHardwareCreateProcessTap) requires Audio Capture permission,
/// NOT Screen Recording permission. If denied, the tap creates successfully but returns
/// silence (all zeros).
#[cfg(target_os = "macos")]
pub fn check_screen_recording_permission() -> bool {
    info!("🔐 Checking Audio Capture permission (kTCCServiceAudioCapture)...");

    // Try to check via TCC framework (private API)
    // This is the same approach used by insidegui/AudioCap
    match check_tcc_audio_capture_permission() {
        Some(granted) => {
            if granted {
                info!("✅ Audio Capture permission: GRANTED");
            } else {
                warn!("⚠️  Audio Capture permission: DENIED");
                warn!("👉 Enable in: System Settings → Privacy & Security → Audio Capture");
            }
            granted
        }
        None => {
            info!("ℹ️  Could not check TCC status (private API unavailable)");
            info!("📍 Permission dialog will appear automatically when recording starts");
            // Optimistic: return true, let the silence detection in stream.rs catch
            // the case where permission is actually denied
            true
        }
    }
}

/// Check Audio Capture permission via TCC private framework.
/// Returns Some(true) if granted, Some(false) if denied, None if check failed.
///
/// Uses the same private TCC SPI approach as insidegui/AudioCap:
///   - dlopen TCC.framework
///   - dlsym TCCAccessPreflight
///   - Call with kTCCServiceAudioCapture
///   - 0 = authorized, 1 = denied
#[cfg(target_os = "macos")]
fn check_tcc_audio_capture_permission() -> Option<bool> {
    use cidre::cf;

    // Load TCC framework
    let tcc_path = std::ffi::CString::new("/System/Library/PrivateFrameworks/TCC.framework/Versions/A/TCC").ok()?;
    let handle = unsafe { libc::dlopen(tcc_path.as_ptr(), libc::RTLD_NOW) };

    if handle.is_null() {
        info!("ℹ️  TCC framework not available");
        return None;
    }

    // Look up TCCAccessPreflight function
    let fn_name = std::ffi::CString::new("TCCAccessPreflight").ok()?;
    let sym = unsafe { libc::dlsym(handle, fn_name.as_ptr()) };

    if sym.is_null() {
        info!("ℹ️  TCCAccessPreflight symbol not found");
        // Don't dlclose — the function pointer may reference it
        return None;
    }

    // TCCAccessPreflight(service: CFString, scope: CFDictionary?) -> Int
    // Returns 0 if authorized, 1 if denied, other values for unknown
    // The service name for Audio Capture is "kTCCServiceAudioCapture"
    type PreflightFunc = unsafe extern "C" fn(*const std::os::raw::c_void, *const std::os::raw::c_void) -> i32;
    let preflight: PreflightFunc = unsafe { std::mem::transmute(sym) };

    // Create CFString for kTCCServiceAudioCapture using cidre's cf module
    let service_name = cf::String::from_str("kTCCServiceAudioCapture");
    // Convert &Type to *const c_void for the private API call
    let service_ref = service_name.as_type_ref() as *const cf::Type as *const std::os::raw::c_void;
    let result = unsafe { preflight(service_ref, std::ptr::null()) };

    // Don't close handle — the function pointer references it
    // (dlclose on macOS with active symbol references is undefined behavior)

    match result {
        0 => Some(true),   // Authorized
        1 => Some(false),  // Denied
        other => {
            info!("ℹ️  TCCAccessPreflight returned unknown result: {}", other);
            None // Unknown status
        }
    }
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