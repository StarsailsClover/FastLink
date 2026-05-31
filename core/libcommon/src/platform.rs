//! FastLink Platform Utilities

/// Check if running on Windows
pub fn is_windows() -> bool {
    cfg!(target_os = "windows")
}

/// Check if running on Linux
pub fn is_linux() -> bool {
    cfg!(target_os = "linux")
}

/// Check if running on macOS
pub fn is_macos() -> bool {
    cfg!(target_os = "macos")
}

/// Get current platform name
pub fn platform_name() -> &'static str {
    if is_windows() {
        "windows"
    } else if is_linux() {
        "linux"
    } else if is_macos() {
        "macos"
    } else {
        "unknown"
    }
}
