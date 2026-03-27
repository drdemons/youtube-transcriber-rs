use std::path::PathBuf;

/// Returns the default output directory for transcription results.
///
/// macOS: ~/Downloads/YouTube Transcriber
/// Linux/other: output (relative to current directory)
pub fn default_output_dir() -> String {
    #[cfg(target_os = "macos")]
    {
        if let Some(home) = std::env::var_os("HOME") {
            let path = PathBuf::from(home)
                .join("Downloads")
                .join("YouTube Transcriber");
            return path.to_string_lossy().to_string();
        }
    }

    "output".to_string()
}

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
