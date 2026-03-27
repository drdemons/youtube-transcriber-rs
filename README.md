# YouTube Transcriber RS

Extract text from YouTube videos using [Whisper AI](https://github.com/openai/whisper) (via [whisper.cpp](https://github.com/ggerganov/whisper.cpp)).

Available as a **CLI tool** and a **GUI application**.

## Features

- Transcribe any YouTube video to text
- Output formats: TXT, SRT (subtitles), WebVTT
- Multiple Whisper model sizes (tiny → large)
- Language selection (en, ru, es, de, ...)
- Proxy support
- GPU acceleration on macOS (Metal / Apple Neural Engine)
- Multi-threaded CPU inference on Linux (OpenMP)

## Prerequisites

- [Rust](https://rustup.rs/) 1.75+
- [yt-dlp](https://github.com/yt-dlp/yt-dlp) — video download
- [ffmpeg](https://ffmpeg.org/) — audio conversion

```bash
# Ubuntu / Debian
sudo apt install ffmpeg
pip install yt-dlp

# macOS
brew install ffmpeg yt-dlp
```

## Building

### Linux — multi-threaded CPU (OpenMP)

```bash
# CLI
cargo build --release -p yt-transcriber-cli --features yt-transcriber-core/openmp

# GUI
cargo build --release -p yt-transcriber-gui --features yt-transcriber-core/openmp
```

### macOS — Metal GPU acceleration

```bash
# CLI
cargo build --release -p yt-transcriber-cli --features yt-transcriber-core/metal

# GUI
cargo build --release -p yt-transcriber-gui --features yt-transcriber-core/metal
```

### macOS — Apple Neural Engine (CoreML)

```bash
# CLI
cargo build --release -p yt-transcriber-cli --features yt-transcriber-core/coreml

# GUI
cargo build --release -p yt-transcriber-gui --features yt-transcriber-core/coreml
```

### Plain — single-threaded (any platform)

```bash
cargo build --release -p yt-transcriber-cli
cargo build --release -p yt-transcriber-gui
```

Binaries are placed in `target/release/`.

## Usage

### GUI

```bash
./target/release/yt-transcriber-gui
```

### CLI

```bash
./target/release/yt-transcriber <URL> [OPTIONS]

Options:
  -m, --model <MODEL>        Model size [default: base]
                             tiny (~39 MB) | base (~74 MB) | small (~244 MB)
                             medium (~769 MB) | large (~1.5 GB)
  -l, --language <LANGUAGE>  Audio language code [default: en]
  -f, --format <FORMAT>      Output format: txt | srt | vtt [default: txt]
  -o, --output-dir <DIR>     Output directory [default: output]
  -p, --proxy <URL>          Proxy URL (http://... or socks5://...)
```

**Examples:**

```bash
# Basic transcription
./target/release/yt-transcriber "https://youtube.com/watch?v=..."

# Russian video, SRT subtitles, small model
./target/release/yt-transcriber "https://youtube.com/watch?v=..." \
  --model small --language ru --format srt

# Via proxy
./target/release/yt-transcriber "https://youtube.com/watch?v=..." \
  --proxy http://127.0.0.1:8080
```

## Models

Whisper models are downloaded automatically from HuggingFace Hub on first use and cached in `~/.cache/huggingface/`.

| Model  | Size    | Speed  | Accuracy |
|--------|---------|--------|----------|
| tiny   | ~39 MB  | fast   | low      |
| base   | ~74 MB  | fast   | good     |
| small  | ~244 MB | medium | better   |
| medium | ~769 MB | slow   | high     |
| large  | ~1.5 GB | slower | best     |

## Project Structure

```
crates/
├── core/   — transcription logic (whisper, downloader, output formats)
├── cli/    — command-line interface
└── gui/    — graphical interface (Slint)
```

## License

MIT
