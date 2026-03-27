/// Whisper transcription via whisper-rs (whisper.cpp bindings).
///
/// whisper.cpp handles mel spectrograms, chunking, beam search, and threading
/// internally — we only need to load the model and pass raw PCM samples.
use anyhow::{Context, Result};
use hf_hub::api::tokio::ApiBuilder;
use std::path::PathBuf;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::WhisperModel;
use super::audio;

pub struct TranscriptionSegment {
    pub text: String,
    /// Start time in milliseconds
    pub start_ms: i64,
    /// End time in milliseconds
    pub end_ms: i64,
}

pub struct TranscriptionResult {
    pub text: String,
    pub segments: Vec<TranscriptionSegment>,
}

pub struct WhisperTranscriber {
    ctx: WhisperContext,
    language: String,
    n_threads: i32,
}

impl WhisperTranscriber {
    pub async fn new(model_variant: &WhisperModel, language: &str) -> Result<Self> {
        let model_path = download_model(model_variant).await?;

        let ctx = WhisperContext::new_with_params(
            &model_path.to_string_lossy(),
            WhisperContextParameters::default(),
        )
        .map_err(|e| anyhow::anyhow!("Failed to load Whisper model: {:?}", e))?;

        // Use all available logical CPUs for maximum parallelism
        let n_threads = std::thread::available_parallelism()
            .map(|n| n.get() as i32)
            .unwrap_or(4);

        Ok(Self {
            ctx,
            language: language.to_string(),
            n_threads,
        })
    }

    /// Transcribe a 16kHz mono WAV file.
    /// whisper.cpp handles chunking and beam search internally.
    pub fn transcribe(&self, wav_path: &std::path::Path) -> Result<TranscriptionResult> {
        let samples = audio::load_wav(wav_path)?;

        let mut state = self
            .ctx
            .create_state()
            .map_err(|e| anyhow::anyhow!("Failed to create Whisper state: {:?}", e))?;

        // Beam search with 5 beams — same default as openai/whisper Python library
        let mut params = FullParams::new(SamplingStrategy::BeamSearch {
            beam_size: 5,
            patience: 1.0,
        });

        params.set_language(Some(&self.language));
        params.set_n_threads(self.n_threads);
        params.set_translate(false);
        params.set_no_context(false);       // use context between segments
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        state
            .full(params, &samples)
            .map_err(|e| anyhow::anyhow!("Whisper inference failed: {:?}", e))?;

        let n_segments = state
            .full_n_segments()
            .map_err(|e| anyhow::anyhow!("Failed to get segment count: {:?}", e))?;

        let mut all_text = String::new();
        let mut segments = Vec::new();

        for i in 0..n_segments {
            let text = state
                .full_get_segment_text(i)
                .map_err(|e| anyhow::anyhow!("Failed to get segment text: {:?}", e))?
                .trim()
                .to_string();

            if text.is_empty() {
                continue;
            }

            // whisper.cpp timestamps are in centiseconds (1/100 s) → convert to ms
            let start_ms = state
                .full_get_segment_t0(i)
                .map_err(|e| anyhow::anyhow!("Failed to get segment t0: {:?}", e))?
                * 10;
            let end_ms = state
                .full_get_segment_t1(i)
                .map_err(|e| anyhow::anyhow!("Failed to get segment t1: {:?}", e))?
                * 10;

            if !all_text.is_empty() {
                all_text.push(' ');
            }
            all_text.push_str(&text);

            segments.push(TranscriptionSegment { text, start_ms, end_ms });
        }

        Ok(TranscriptionResult { text: all_text, segments })
    }
}

/// Download GGML model file from HuggingFace Hub (cached in ~/.cache/huggingface).
async fn download_model(model: &WhisperModel) -> Result<PathBuf> {
    let api = ApiBuilder::from_env()
        .with_progress(true)
        .build()
        .context("Failed to initialize HuggingFace Hub API")?;

    let repo = api.model(model.hf_repo_id().to_string());

    let path = repo
        .get(model.ggml_filename())
        .await
        .with_context(|| format!("Failed to download {}", model.ggml_filename()))?;

    Ok(path)
}
