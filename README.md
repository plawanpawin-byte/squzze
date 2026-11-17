# Squeeze

A free, open-source, offline batch media converter for Windows (Linux and macOS builds are planned).
Part of **Creator Suite** — independent tools for creators. Not affiliated with or endorsed by Apple.

Drag in video or audio files, pick one or more outputs for each file, press **Start batch**. Squeeze does the
conversion with [FFmpeg](https://ffmpeg.org) and uses your GPU when it works.

## Features
- Add files by dialog (Ctrl+O) or drag-and-drop. The inspector shows codec, profile/level, frame rate (constant or variable), bit depth, colour/HDR, timecode, every audio stream and subtitle track.
- 16 built-in presets (H.264, HEVC, AV1, VP9, ProRes, FFV1, AAC/Opus/MP3/FLAC/WAV). Presets the installed FFmpeg cannot run are greyed out with the reason.
- Custom presets with advanced settings (10-bit, encoder effort, keyframe interval, two-pass, H.264 profile/level); import/export JSON, duplicate, favourites; old preset files are migrated.
- Audio per file: pick the stream, copy the original untouched, convert, remove, or downmix.
- Several outputs per file; in/out trim (**I** / **O**); destination, file-name template, overwrite policy, optional metadata removal.
- Queue with 1–4 parallel jobs, progress that never goes backwards, ETA, cancel/retry, survives restarts.
- Every export is re-read with ffprobe: **✓ Verified** or what is wrong, size change, SHA-256, exact FFmpeg command, copyable report, delivery manifest (JSON), and an A/B frame comparison of source vs output.
- GPU encoding (NVENC / AMF / Quick Sync) with automatic software fallback.
- Resizable panels, frame preview, dark and light themes. No account, no telemetry, no network access.

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
