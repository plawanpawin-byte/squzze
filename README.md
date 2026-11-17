# Squeeze

A free, open-source, offline batch media converter for Windows (Linux and macOS builds are planned).
Part of **Creator Suite** — independent tools for creators. Not affiliated with or endorsed by Apple.

Drag in video or audio files, pick one or more outputs for each file, press **Start batch**. Squeeze does the
conversion with [FFmpeg](https://ffmpeg.org) and uses your GPU when it works.

## Features
- Add files by dialog (Ctrl+O) or drag-and-drop; shows codec, size, frame rate, audio and duration.
- 16 built-in presets: H.264, HEVC (also "smaller file" variants), AV1, VP9, ProRes 422/4444, FFV1, and AAC / Opus / MP3 / FLAC / WAV audio.
- Several outputs per file; destination folder or next to the source; file-name template and overwrite policy (keep both / skip / replace).
- In/Out trim: set points with **I** / **O**, buttons or typed times; only that range is exported.
- Custom presets (container, codec, quality or bitrate, max height, frame rate, audio) with JSON import/export.
- Queue with 1–4 jobs at a time; per job: progress, fps, speed, elapsed, time left, which encoder is used, cancel, retry, remove; pause/resume the queue; the queue survives restarts.
- After each export Squeeze re-reads the file with ffprobe and shows **✓ Verified** or what is wrong, plus the size change (and warns when the output is bigger than the source). Each job keeps its exact FFmpeg command and output; "Copy report" puts it on the clipboard.
- Automatic GPU encoding (NVIDIA NVENC, AMD AMF, Intel Quick Sync) with automatic software fallback.
- Resizable panels (drag the dividers; double-click resets; saved), frame preview with a scrubber, dark and light themes. Keyboard: Ctrl+O add, Ctrl+Enter start, Delete remove, I / O in / out.
- No account, no telemetry, no network access.

See [docs/STATUS.md](docs/STATUS.md) for what is done, tested and still missing.

## Install
1. Install FFmpeg: `winget install Gyan.FFmpeg` (or put `ffmpeg.exe` and `ffprobe.exe` in a folder named `ffmpeg` next to Squeeze).
2. Run the Squeeze installer from the Releases page.

The installer is not code-signed yet, so Windows SmartScreen may warn. Compare the SHA-256 shown on the release page.

## Build from source
Requirements: Rust (MSVC toolchain), Node.js 22+, WebView2 (included in Windows 11), FFmpeg for the tests.

```bash
cargo test -p cs-probe -p cs-ffmpeg -p cs-queue   # engine tests (uses real FFmpeg)
cd app && npm install && npm run tauri build       # produces the installer under target/release/bundle
```

## Layout
| Path | What |
|---|---|
| `crates/cs-probe` | ffprobe wrapper, safe tool discovery, input validation |
| `crates/cs-ffmpeg` | presets, FFmpeg argument builder, runner, hardware detection |
| `crates/cs-queue` | persistent queue, scheduler, FFmpeg encoder with fallback |
| `app/` | Tauri 2 + Svelte desktop app |
| `creator-suite-research/` | research notes and specs behind the suite (documentation-level, no Apple binaries analysed) |

## Security
See [SECURITY.md](SECURITY.md) for the threat model, what was verified and what was not.

## License
GPL-3.0-or-later. FFmpeg is a separate program that Squeeze starts as a subprocess; it is not bundled. Check the
license of the FFmpeg build you install (the common builds that include x264/x265 are GPL).
