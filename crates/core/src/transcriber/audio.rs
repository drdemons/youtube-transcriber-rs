/// Audio loading — whisper.cpp handles mel spectrograms internally.
use anyhow::Result;

pub const SAMPLE_RATE: u32 = 16_000;

/// Read a 16kHz mono WAV file and return f32 PCM samples in [-1.0, 1.0].
pub fn load_wav(path: &std::path::Path) -> Result<Vec<f32>> {
    let mut reader = hound::WavReader::open(path)?;
    let spec = reader.spec();

    anyhow::ensure!(
        spec.sample_rate == SAMPLE_RATE,
        "Expected 16kHz WAV, got {}Hz. yt-dlp postprocessor-args should have resampled it.",
        spec.sample_rate
    );
    anyhow::ensure!(
        spec.channels == 1,
        "Expected mono WAV, got {} channels.",
        spec.channels
    );

    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .collect::<std::result::Result<_, _>>()?,
        hound::SampleFormat::Int => {
            let max = (1i64 << (spec.bits_per_sample - 1)) as f32;
            reader
                .samples::<i32>()
                .map(|s| s.map(|v| v as f32 / max))
                .collect::<std::result::Result<_, _>>()?
        }
    };

    Ok(samples)
}
