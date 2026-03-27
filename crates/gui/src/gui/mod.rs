use std::path::Path;

use yt_transcriber_core::{check_dependencies, default_output_dir, downloader, output, transcriber, OutputFormat, WhisperModel};

slint::include_modules!();

/// Launch the Slint GUI window.  Blocks until the window is closed.
pub fn run_gui() -> Result<(), slint::PlatformError> {
    let app = AppWindow::new()?;

    app.set_output_dir(default_output_dir().into());

    // ── Dependency check ─────────────────────────────────────────────────────
    if let Err(msg) = check_dependencies() {
        app.set_status_text(msg.into());
        app.set_has_result(true);
        app.set_has_warning(true);
    }

    // ── Start button ──────────────────────────────────────────────────────────
    let weak = app.as_weak();
    app.on_start_clicked(move || {
        let app = match weak.upgrade() {
            Some(a) => a,
            None => return,
        };

        let url = app.get_url().to_string();
        if url.is_empty() {
            return;
        }

        let model = index_to_model(app.get_model_index());
        let language = app.get_language().to_string();
        let format = index_to_format(app.get_format_index());
        let output_dir = app.get_output_dir().to_string();
        let proxy_str = app.get_proxy().to_string();
        let proxy = if app.get_use_proxy() && !proxy_str.is_empty() { Some(proxy_str) } else { None };

        app.set_is_running(true);
        app.set_has_result(false);
        app.set_progress(0.0);
        app.set_status_text("Starting...".into());

        let weak2 = app.as_weak();
        std::thread::spawn(move || {
            let result = do_transcription(url, model, language, format, output_dir, proxy, weak2.clone());

            let _ = slint::invoke_from_event_loop(move || {
                if let Some(app) = weak2.upgrade() {
                    app.set_is_running(false);
                    if let Err(e) = result {
                        app.set_status_text(format!("❌ Error: {e}").into());
                        app.set_progress(0.0);
                    }
                }
            });
        });
    });

    // ── Open folder button ────────────────────────────────────────────────────
    let weak = app.as_weak();
    app.on_open_folder_clicked(move || {
        if let Some(app) = weak.upgrade() {
            let path_str = app.get_output_path().to_string();
            let folder = Path::new(&path_str)
                .parent()
                .and_then(|p| p.canonicalize().ok())
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| ".".to_string());

            #[cfg(target_os = "linux")]
            let _ = std::process::Command::new("xdg-open").arg(&folder).spawn();
            #[cfg(target_os = "macos")]
            let _ = std::process::Command::new("open").arg(&folder).spawn();
            #[cfg(target_os = "windows")]
            let _ = std::process::Command::new("explorer").arg(&folder).spawn();
        }
    });

    app.run()
}

// ── Background transcription ──────────────────────────────────────────────────

fn do_transcription(
    url: String,
    model: WhisperModel,
    language: String,
    format: OutputFormat,
    output_dir: String,
    proxy: Option<String>,
    weak: slint::Weak<AppWindow>,
) -> anyhow::Result<()> {
    let push_status = |msg: String, pct: f32| {
        let w = weak.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(app) = w.upgrade() {
                app.set_status_text(msg.into());
                app.set_progress(pct);
            }
        });
    };

    // 1. Video metadata
    push_status("Fetching video information...".into(), 0.05);
    let info = downloader::get_video_info(&url, proxy.as_deref())?;

    // 2. Download audio
    push_status(format!("Downloading: {}...", info.title), 0.15);
    let output_path = Path::new(&output_dir);
    let audio_file = downloader::download_audio(&url, output_path, proxy.as_deref())?;

    // 3. Load Whisper model (async inside a blocking runtime)
    push_status(format!("Loading {} model...", model.display_name()), 0.35);
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| anyhow::anyhow!("Failed to create async runtime: {e}"))?;
    let transcriber = rt.block_on(transcriber::WhisperTranscriber::new(&model, &language))?;

    // 4. Transcribe (CPU-intensive — whisper.cpp uses all threads)
    push_status("Transcribing audio (this may take a while)...".into(), 0.5);
    let result = transcriber.transcribe(&audio_file)?;

    // Clean up the temporary WAV file
    if audio_file.exists() {
        let _ = std::fs::remove_file(&audio_file);
    }

    // 5. Save output
    push_status("Saving output...".into(), 0.95);
    let ext = match format {
        OutputFormat::Txt => "txt",
        OutputFormat::Srt => "srt",
        OutputFormat::Vtt => "vtt",
    };
    let base_name = audio_file
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let output_file = output_path.join(format!("{base_name}.{ext}"));
    output::save(&result, &output_file, &format)?;

    // Canonicalize so "Open Folder" works from any working directory
    let abs_path = output_file
        .canonicalize()
        .unwrap_or_else(|_| output_file.clone());
    let abs_path_str = abs_path.to_string_lossy().to_string();

    // 6. Push final result to UI
    let n_segs = result.segments.len();
    let preview: String = result.text.chars().take(500).collect();
    let w = weak.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(app) = w.upgrade() {
            app.set_status_text(format!("✓ Done! {n_segs} segments transcribed.").into());
            app.set_progress(1.0);
            app.set_result_preview(preview.into());
            app.set_output_path(abs_path_str.into());
            app.set_has_result(true);
        }
    });

    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn index_to_model(index: i32) -> WhisperModel {
    match index {
        0 => WhisperModel::Tiny,
        1 => WhisperModel::Base,
        2 => WhisperModel::Small,
        3 => WhisperModel::Medium,
        _ => WhisperModel::Large,
    }
}

fn index_to_format(index: i32) -> OutputFormat {
    match index {
        1 => OutputFormat::Srt,
        2 => OutputFormat::Vtt,
        _ => OutputFormat::Txt,
    }
}
