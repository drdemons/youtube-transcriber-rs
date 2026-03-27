pub mod cli;

use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use yt_transcriber_core::{check_dependencies, downloader, output, transcriber, OutputFormat};

use cli::Cli;

pub async fn run(cli: Cli) -> Result<()> {
    print_banner();

    if let Err(msg) = check_dependencies() {
        eprintln!("{}", style(msg).red());
        std::process::exit(1);
    }

    // ── Step 1: Video info ──────────────────────────────────────────────────
    let spinner = make_spinner("Fetching video information...");
    let info = downloader::get_video_info(&cli.url, cli.proxy.as_deref())?;
    spinner.finish_with_message("✓ Got video info");

    print_video_info(&info, cli.model.display_name(), &cli.language);

    // ── Step 2: Download audio ──────────────────────────────────────────────
    let spinner = make_spinner("Downloading audio (yt-dlp → 16kHz WAV)...");
    let output_dir = Path::new(&cli.output_dir);
    let audio_file = downloader::download_audio(&cli.url, output_dir, cli.proxy.as_deref())?;
    spinner.finish_with_message(format!("✓ Audio: {}", audio_file.display()));

    // ── Step 3: Load Whisper model from HuggingFace Hub ─────────────────────
    let spinner = make_spinner(format!(
        "Loading Whisper {} from HuggingFace Hub...",
        cli.model.display_name()
    ));
    let transcriber = transcriber::WhisperTranscriber::new(&cli.model, &cli.language).await?;
    spinner.finish_with_message("✓ Model ready");

    // ── Step 4: Transcribe ──────────────────────────────────────────────────
    let spinner = make_spinner("Transcribing audio (this may take a while)...");
    let result = transcriber.transcribe(&audio_file)?;
    spinner.finish_with_message(format!(
        "✓ Transcription complete ({} chunks)",
        result.segments.len()
    ));

    // ── Step 5: Save ────────────────────────────────────────────────────────
    let ext = match cli.format {
        OutputFormat::Txt => "txt",
        OutputFormat::Srt => "srt",
        OutputFormat::Vtt => "vtt",
    };
    let base_name = audio_file
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();
    let output_file = output_dir.join(format!("{}.{}", base_name, ext));

    output::save(&result, &output_file, &cli.format)?;

    // ── Cleanup ─────────────────────────────────────────────────────────────
    if audio_file.exists() {
        let _ = std::fs::remove_file(&audio_file);
    }

    // ── Result ──────────────────────────────────────────────────────────────
    println!();
    println!("{}", style("✨ Done!").green().bold());
    println!("  {} {}", style("📁 Output:").cyan(), output_file.display());
    println!("  {}", style("📝 Preview:").cyan());
    let preview: String = result.text.chars().take(200).collect();
    println!("     {}…", style(preview).dim());
    println!();

    Ok(())
}

fn print_banner() {
    println!();
    println!(
        "{}",
        style("  🎬  YouTube Transcriber RS  ").bold().cyan().on_black()
    );
    println!("{}", style("     Powered by whisper.cpp         ").dim());
    println!();
}

fn print_video_info(info: &downloader::VideoInfo, model: &str, lang: &str) {
    println!("{}", style("── Video Info ─────────────────────────────").dim());
    println!("  {} {}", style("📺 Title:   ").cyan(), info.title);
    println!("  {} {}", style("👤 Uploader:").cyan(), info.uploader);
    println!(
        "  {} {}m {}s",
        style("⏱  Duration:").cyan(),
        info.duration / 60,
        info.duration % 60
    );
    println!("  {} {}", style("🤖 Model:   ").cyan(), model);
    println!("  {} {}", style("🌐 Language:").cyan(), lang);
    println!("{}", style("───────────────────────────────────────────").dim());
    println!();
}

fn make_spinner(msg: impl Into<String>) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message(msg.into());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}
