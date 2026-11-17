# Compressor alternative — MVP implementation spec

Working name: **Squeeze** (placeholder, not trademark-checked).
**Status:** specification only. Per user instruction nothing is built yet.

## 1. Product definition
A free, open-source, offline, cross-platform batch media transcoder with presets, a persistent job queue, hardware-encoder support and a GUI plus CLI. Observable behaviour is modelled on the *documented* features of Apple Compressor (batches of jobs, settings/presets, destinations, progress, distributed processing), implemented independently on FFmpeg.

## 2. MVP scope
### In
| ID | Feature | Evidence basis |
|---|---|---|
| F1 | Add files by dialog/drag-drop; probe and show codec, resolution, fps, duration, audio | DOC (Compressor adds source media) |
| F2 | Built-in presets: H.264 MP4, HEVC MP4, AV1 MP4/WebM (SVT-AV1), VP9 WebM, ProRes 422/4444 MOV, AAC/Opus/FLAC/WAV audio-only | DOC (presets), FFmpeg encoders verified on this PC |
| F3 | Custom settings: container, video codec, rate control (CRF/bitrate), resolution, fps, audio codec/bitrate/channels | DOC feature level |
| F4 | Destination folder + filename template (`{name}-{preset}.{ext}`), overwrite policy | DOC (destinations) |
| F5 | Batch queue: N parallel jobs (default 1, max = settings), cancel, retry, remove | DOC (batches) |
| F6 | Progress per job: percent, fps, speed, ETA, log | DOC feature level |
| F7 | Persistent queue across restarts (SQLite) | design choice |
| F8 | Hardware encode auto-detect (NVENC, AMF, QSV) with fallback to software | verified encoders on this PC |
| F9 | Preset import/export (JSON) | design choice |
| F10 | CLI: `squeeze encode`, `squeeze probe`, `squeeze queue` with JSON output | design choice |
| F11 | Offline operation, no telemetry | project requirement |

### Out of MVP (later)
Watch folders (v0.3), distributed encoding across LAN machines (v1.1; Compressor's Shared Computers is DOC), Final Cut/Motion integration, advanced filters (denoise, deinterlace, LUT), subtitles burn-in, DCP/broadcast delivery packages, HDR tone mapping, cloud anything.

## 3. Architecture
```
squeeze/
  crates/
    cs-store      SQLite wrapper + migrations
    cs-settings   typed settings
    cs-probe      ffprobe -print_format json -> typed MediaInfo
    cs-presets    preset schema (serde), validation, built-ins (TOML embedded)
    cs-ffmpeg     EncodePlan -> argv builder, process runner, -progress parser, hw detection
    cs-queue      job table, scheduler, worker pool, events
    squeeze-cli   clap-based CLI
  app/            Tauri 2 shell (src-tauri/ commands; ui/ Svelte+TS)
  tests/          integration tests + fixture media generator (FFmpeg lavfi)
  docs/
```
### Data model (SQLite)
- `jobs(id, source_path, preset_id, dest_path, state, progress, error, created_at, started_at, finished_at, attempt, plan_json)`
- `presets(id, name, json, builtin)`
- `settings(key, value)`
- `job_events(job_id, ts, kind, payload)` for logs.
States: `queued → running → done | failed | cancelled`, `failed → queued` (retry).

### Encode execution
1. `ffprobe` source → `MediaInfo`.
2. Build `EncodePlan` from preset + overrides; validate (e.g. ProRes needs MOV; alpha support).
3. Spawn `ffmpeg -hide_banner -nostdin -y -progress pipe:1 -i … -c:v … out.tmp.ext`; write to temp file, atomic rename on success.
4. Parse key=value progress (`out_time_us`, `fps`, `speed`) and compute percent from probed duration.
5. Cancel: kill process tree (Windows Job Object), delete temp file.
6. HW detection: run `ffmpeg -encoders` once, then a 1-frame test encode per candidate encoder (`-f lavfi -i testsrc … -frames:v 1`) to confirm the GPU actually works; cache result.

### FFmpeg strategy
Subprocess (not linked) so the suite's own license stays independent and users can supply their own build. Distribution ships an LGPL-only build by default; GPL build (x264/x265) as optional separate download. Discovery order: user setting → bundled → PATH.

## 4. UI (Tauri + Svelte)
Main window: left sources/queue table (name, preset, status, progress), right inspector (preset settings, destination), bottom log. Menu: File/Edit/Queue/Help. Keyboard: Ctrl+O add, Ctrl+Enter start, Esc cancel. Dark/light. Full keyboard navigation and ARIA labels.

## 5. Test plan (cases)
| ID | Test | Pass condition |
|---|---|---|
| T01 | Probe 10 fixture files (h264, hevc, prores, vp9, av1, wav, flac, mp3, mkv, mov) | Parsed fields equal values from `ffprobe` reference JSON |
| T02 | Encode H.264 preset on 10 s fixture | Output exists, probes as h264 mp4, duration within ±0.1 s |
| T03 | Each built-in preset on fixture | All succeed; codecs/containers as specified |
| T04 | Progress | Progress events monotonic non-decreasing, final 100 %, ≥ 1 event/s on 60 s encode |
| T05 | Cancel mid-encode | Process gone ≤ 2 s, no temp file left, job = cancelled |
| T06 | Restart persistence | Kill app with 3 queued + 1 running; restart: queued intact, running → failed/queued per policy |
| T07 | Concurrency | With limit 2, never > 2 ffmpeg processes (sampled) |
| T08 | Bad input (corrupt/zero-byte/missing) | Job fails with readable error; queue continues |
| T09 | Overwrite policy | skip/rename/replace behave as chosen |
| T10 | Filename template | Unicode (Thai) and spaces in paths work on Windows |
| T11 | HW detect | On this PC NVENC listed when functional; forced failure falls back to software with warning |
| T12 | Preset JSON round trip | Export → import identical |
| T13 | CLI JSON output | Schema-valid; exit codes 0/1/2 as documented |
| T14 | Offline | With firewall blocking the app, all functions work |
| T15 | Long path / read-only dest | Clear error, no crash |
| T16 | 24 h soak | 500 small jobs, no leak > 50 MB RSS growth |
| T17 | Accessibility | Keyboard-only run through add→encode→open output |
| T18 | Cross-platform CI | T01–T10 pass on Windows, Ubuntu, macOS runners |

## 6. Release criteria for v1.0
1. T01–T18 pass. 2. Windows installer builds and installs cleanly on a fresh VM. 3. No network calls (verified). 4. License register complete, SBOM generated. 5. Docs: user guide, preset schema, CLI reference. 6. Known-issues list published. 7. No data-loss bugs open.

## 7. Risks
Codec patents; FFmpeg build licensing; ProRes encoders in FFmpeg are unofficial reimplementations (output validity vs Apple's ProRes — verify in Final Cut/QuickTime on a Mac); hardware encoders vary by GPU; Windows process-tree kill; huge-file temp disk (note: C: here has 4.3 GB free).
