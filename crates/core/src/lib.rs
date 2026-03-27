pub mod deps;
pub mod downloader;
pub mod output;
pub mod tools;
pub mod transcriber;

pub use deps::check_dependencies;

pub use downloader::VideoInfo;
pub use transcriber::{TranscriptionResult, TranscriptionSegment, WhisperTranscriber};

use clap::ValueEnum;

#[derive(ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    Txt,
    Srt,
    Vtt,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum WhisperModel {
    Tiny,
    Base,
    Small,
    Medium,
    Large,
}

impl WhisperModel {
    /// HuggingFace repo containing GGML model files for whisper.cpp
    pub fn hf_repo_id(&self) -> &'static str {
        "ggerganov/whisper.cpp"
    }

    /// GGML model filename in the repo
    pub fn ggml_filename(&self) -> &'static str {
        match self {
            Self::Tiny   => "ggml-tiny.bin",
            Self::Base   => "ggml-base.bin",
            Self::Small  => "ggml-small.bin",
            Self::Medium => "ggml-medium.bin",
            Self::Large  => "ggml-large-v3.bin",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Tiny   => "tiny (~39 MB)",
            Self::Base   => "base (~74 MB)",
            Self::Small  => "small (~244 MB)",
            Self::Medium => "medium (~769 MB)",
            Self::Large  => "large-v3 (~1.5 GB)",
        }
    }
}
