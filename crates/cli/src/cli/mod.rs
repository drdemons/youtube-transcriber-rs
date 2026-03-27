use clap::Parser;
use yt_transcriber_core::{OutputFormat, WhisperModel};

#[derive(Parser, Debug)]
#[command(
    name = "yt-transcriber",
    about = "Extract text from YouTube videos using Whisper AI (whisper.cpp)",
    version = "0.1.0"
)]
pub struct Cli {
    /// YouTube video URL
    pub url: String,

    /// Whisper model size
    #[arg(short, long, default_value = "base", value_enum)]
    pub model: WhisperModel,

    /// Audio language code (e.g. en, ru, es, de)
    #[arg(short, long, default_value = "en")]
    pub language: String,

    /// Output format
    #[arg(short, long, default_value = "txt", value_enum)]
    pub format: OutputFormat,

    /// Output directory
    #[arg(short = 'o', long, default_value = "output")]
    pub output_dir: String,

    /// Proxy URL (e.g. http://127.0.0.1:8080 or socks5://user:pass@host:port)
    #[arg(short, long)]
    pub proxy: Option<String>,
}
