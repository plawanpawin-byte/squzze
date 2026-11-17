# Squeeze — status against the P0 backlog

Status words follow the backlog's rules: **Implemented** (code exists), **Tested** (a test or the E2E run exercised it
on a real FFmpeg / the real exe), **Partial**, **Not done**. Nothing is called done from a button or a string alone.

Evidence comes from `cargo test --workspace` (44 Rust tests, includes real FFmpeg runs) and `scripts/e2e.mjs`
(drives the real exe through WebView2 DevTools; ~70 checks). Test inputs include a real user screen recording
(HEVC 1496×852, 28 s) when present.

| Package | Status | What exists | What is missing |
|---|---|---|---|
| P0-01 Media Inspector | **Partial, tested** | container, codec, resolution, fps, pixel format, field order, audio codec/channels/rate, bit rate, size, duration; bad/empty files fail visibly | profile/level, bit depth, colour primaries/transfer/matrix, HDR metadata, several audio streams, subtitles, timecode, CFR/VFR flag; the "8 file types vs ffprobe" corpus |
| P0-02 Advanced video settings | **Partial** | custom preset: container, codec, CRF or bitrate, max height, fps, incompatible combinations rejected before the job starts | profile/level, pixel format, bit depth, crop, GOP, two-pass, per-job encoder backend |
| P0-03 Audio inspector & mapping | **Not done** | first audio stream only; codec/bitrate/channels come from the preset | stream picker, channel mapping, passthrough/copy |
| P0-04 Batch queue & multi-output | **Implemented, tested** | one source → several outputs, queued/running/done/failed/cancelled, per-job log and exact command line, cancel (kills FFmpeg, removes temp file), retry, queue survives restart/crash, 1–4 parallel jobs, in/out trim | per-job priority; a per-job "paused" state (pause is queue-wide); an explicit 1 → 3 outputs verification test |
| P0-05 Preset system v2 | **Partial, tested** | validated schema, import/export JSON (64 KB cap), custom edit/delete, ids cannot shadow built-ins | schema version + migration, favourites, duplicate, delivery-target grouping |
| P0-06 Export & delivery | **Partial, tested** | destination folder, name template, overwrite policy (keep both / skip / replace — source never overwritten), Thai/Unicode names, post-export report (size, duration, codec, size change) | estimated output size, metadata policy, job manifest, checksum |
| P0-07 Hardware detection | **Implemented, tested on one PC** | real test encodes find working NVENC/AMF/QSV; UI lists only what works; automatic software fallback | fallback has no forced-failure test; only an NVIDIA + AMD machine was tried |
| P0-08 Progress & diagnostics | **Implemented, tested** | percent, fps, speed, elapsed, ETA, which encoder is used, FFmpeg output tail, copy report | frame counter, error codes, export report to a file |
| P0-09 Source & output preview | **Partial, tested** | frame preview, scrub, frame/second stepping, in/out markers | playback, output preview, A/B compare |
| P0-10 FFmpeg packaging & validation | **Partial** | finds ffmpeg/ffprobe safely, shows version in the footer, friendly message when missing | FFmpeg is **not bundled** (user installs it); no 25+ file corpus; third-party notices are brief (`THIRD_PARTY_NOTICES.md`); not tested on a clean Windows VM |

## Changes in this round (before → after)

| Area | Before (23c8120) | After |
|---|---|---|
| Layout | fixed column widths | draggable splitters for sidebar, inspector and preview height; clamp, double-click reset, keyboard arrows, saved between runs, safe on narrow windows |
| Current | fixed preview height, trim impossible | In/Out points (buttons, **I**/**O** keys or typed times) shown on the timeline, applied when exporting; Start button says why it is disabled |
| Active | plain bars | thumbnail, % / fps / speed / elapsed / ETA, which encoder, overall progress (also in the footer), pause/resume/cancel, details with the exact command and FFmpeg output |
| Completed | "Done" only | ✓ Verified (ffprobe checked codec, size limits, duration, audio), size change with a warning when the output is bigger than the source, copy report, retry |
| Feedback | silent | status messages, drop overlay while dragging files, empty states that explain the next step |
| Settings | none in UI | name template and overwrite policy in the Locations tab; FFmpeg version in the footer |
| Presets | 14 | 16 (adds "smaller file" H.264/HEVC after a real recording grew by 7 % with the H.264 preset) |

## Known limits that matter to users
* Installer is **not code-signed** (SmartScreen warning).
* FFmpeg must be installed separately.
* Tested only on the developer's machine.
* Trimmed jobs are compared by output size only (a percentage would mislead).
