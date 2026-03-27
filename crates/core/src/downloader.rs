use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::tools::tool_path;

pub struct VideoInfo {
    pub title: String,
    pub uploader: String,
    pub duration: u64,
}

/// Получить метаданные видео без скачивания
pub fn get_video_info(url: &str, proxy: Option<&str>) -> Result<VideoInfo> {
    let mut cmd = Command::new(tool_path("yt-dlp"));
    cmd.args(["--dump-json", "--no-download", url]);
    if let Some(p) = proxy {
        cmd.args(["--proxy", p]);
    }

    let output = cmd
        .output()
        .context("Failed to run yt-dlp. Make sure it is installed: pip install yt-dlp")?;

    if !output.status.success() {
        bail!(
            "yt-dlp error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).context("Failed to parse yt-dlp JSON output")?;

    Ok(VideoInfo {
        title: json["title"].as_str().unwrap_or("Unknown").to_string(),
        uploader: json["uploader"].as_str().unwrap_or("Unknown").to_string(),
        duration: json["duration"].as_u64().unwrap_or(0),
    })
}

/// Скачать аудио как WAV 16kHz моно (через yt-dlp + ffmpeg)
pub fn download_audio(url: &str, output_dir: &Path, proxy: Option<&str>) -> Result<PathBuf> {
    std::fs::create_dir_all(output_dir)?;

    let template = output_dir.join("%(title)s.%(ext)s");
    let ffmpeg = tool_path("ffmpeg");

    let mut cmd = Command::new(tool_path("yt-dlp"));
    cmd.args([
        "--no-playlist",
        "-x",
        "--audio-format",
        "wav",
        "--ffmpeg-location",
        ffmpeg.to_str().unwrap(),
        "--postprocessor-args",
        // Ресэмплируем до 16кГц моно — именно этого ожидает Whisper
        "ffmpeg:-ar 16000 -ac 1",
        "-o",
        template.to_str().unwrap(),
        "--print",
        "after_move:filepath",
        "--no-progress",
        url,
    ]);

    if let Some(p) = proxy {
        cmd.args(["--proxy", p]);
    }

    let output = cmd.output().context("Failed to run yt-dlp")?;
    if !output.status.success() {
        bail!(
            "yt-dlp download failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let path_str = String::from_utf8(output.stdout)
        .context("Invalid yt-dlp output encoding")?
        .trim()
        .to_string();

    if path_str.is_empty() {
        bail!("yt-dlp did not print the output file path (--print after_move:filepath)");
    }

    Ok(PathBuf::from(path_str))
}
