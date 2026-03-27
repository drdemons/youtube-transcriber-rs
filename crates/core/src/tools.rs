use std::path::PathBuf;

/// Resolve path to a bundled tool (yt-dlp, ffmpeg).
///
/// On macOS: first looks next to the current executable (inside .app bundle),
/// then falls back to PATH.
/// On other platforms: always uses PATH.
pub fn tool_path(name: &str) -> PathBuf {
    #[cfg(target_os = "macos")]
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidate = dir.join(name);
            if candidate.exists() {
                return candidate;
            }
        }
    }

    PathBuf::from(name)
}
