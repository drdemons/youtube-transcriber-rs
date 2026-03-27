use anyhow::Result;
use std::io::Write;
use std::path::Path;

use crate::OutputFormat;
use crate::transcriber::TranscriptionResult;

/// Save transcription result to file in the requested format.
pub fn save(result: &TranscriptionResult, output_path: &Path, format: &OutputFormat) -> Result<()> {
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = std::fs::File::create(output_path)?;

    match format {
        OutputFormat::Txt => write_txt(&mut file, result),
        OutputFormat::Srt => write_srt(&mut file, result),
        OutputFormat::Vtt => write_vtt(&mut file, result),
    }
}

fn write_txt(file: &mut impl Write, result: &TranscriptionResult) -> Result<()> {
    for segment in &result.segments {
        let timestamp = format_simple(segment.start_ms);
        writeln!(file, "[{timestamp}] {}", segment.text)?;
        writeln!(file)?;
    }
    Ok(())
}

fn write_srt(file: &mut impl Write, result: &TranscriptionResult) -> Result<()> {
    for (i, segment) in result.segments.iter().enumerate() {
        writeln!(file, "{}", i + 1)?;
        writeln!(
            file,
            "{} --> {}",
            format_srt(segment.start_ms),
            format_srt(segment.end_ms)
        )?;
        writeln!(file, "{}", segment.text)?;
        writeln!(file)?;
    }
    Ok(())
}

fn write_vtt(file: &mut impl Write, result: &TranscriptionResult) -> Result<()> {
    writeln!(file, "WEBVTT")?;
    writeln!(file)?;
    for segment in &result.segments {
        writeln!(
            file,
            "{} --> {}",
            format_vtt(segment.start_ms),
            format_vtt(segment.end_ms)
        )?;
        writeln!(file, "{}", segment.text)?;
        writeln!(file)?;
    }
    Ok(())
}

fn format_simple(ms: i64) -> String {
    let total_secs = ms / 1000;
    let m = total_secs / 60;
    let s = total_secs % 60;
    format!("{}:{:02}", m, s)
}

fn format_srt(ms: i64) -> String {
    let ms = ms.max(0) as u64;
    let h = ms / 3_600_000;
    let m = (ms % 3_600_000) / 60_000;
    let s = (ms % 60_000) / 1_000;
    let millis = ms % 1_000;
    format!("{:02}:{:02}:{:02},{:03}", h, m, s, millis)
}

fn format_vtt(ms: i64) -> String {
    let ms = ms.max(0) as u64;
    let h = ms / 3_600_000;
    let m = (ms % 3_600_000) / 60_000;
    let s = (ms % 60_000) / 1_000;
    let millis = ms % 1_000;
    format!("{:02}:{:02}:{:02}.{:03}", h, m, s, millis)
}
