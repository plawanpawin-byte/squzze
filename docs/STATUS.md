# Squeeze 0.2.0 — status against the P0 backlog

Status words follow the backlog's rules: **Implemented** (code exists), **Tested** (a test or the E2E run exercised it
on a real FFmpeg / the real exe), **Partial**, **Not done**. Nothing is called done from a button or a string alone.

Evidence: `cargo test --workspace` (Rust unit + integration tests against a real FFmpeg, incl. a 31-file generated corpus
and a real user screen recording when present) and three E2E scripts that drive the real exe through WebView2 DevTools:
`scripts/e2e.mjs` (78 checks), `scripts/e2e-p0.mjs` (24), `scripts/e2e-availability.mjs` (10, debug build).

| Package | Status | What exists | What is still missing |
|---|---|---|---|
| P0-01 Media Inspector | **Implemented, tested** | container, codec, profile/level, frame size, constant/variable frame rate, bit depth, pixel format, field order, colour primaries/transfer/matrix/range, timecode tag, HDR (PQ/HLG) warning, every audio stream (codec, layout, rate, language, title), subtitle tracks, bit rate, size; missing details say "Unknown"; corpus check of 31 generated files incl. HDR10, 10-bit, VFR, 5.1, two audio tracks, subtitles, corrupt/empty files | Dolby Vision / mastering-display metadata; chapters; a corpus of *real* camera files |
| P0-02 Advanced video settings | **Partial, tested** | per preset: bit depth 8/10, encoder effort, keyframe interval, two-pass (bitrate mode, H.264/HEVC/VP9), H.264 profile and level; incompatible combinations rejected before a job starts; two-pass bitrate accuracy and log clean-up verified | crop, scaling algorithm, CBR/VBR variants beyond CRF/average bitrate, explicit pixel format, per-job encoder backend choice |
| P0-03 Audio inspector & mapping | **Implemented, tested** | choose the audio stream, copy the original (container compatibility checked), convert, remove, downmix (never upmix); conflicts block Start with the reason | per-channel mapping, sample-rate change, mixing several tracks |
| P0-04 Batch queue & multi-output | **Implemented, tested** | one source → several outputs (E2E: 3 outputs from one recording), states, per-job log and exact command(s), cancel (process killed, temp removed), retry, restart/crash recovery, 1–4 parallel jobs, in/out trim | per-job priority; per-job paused state (pause is queue-wide) |
| P0-05 Preset system v2 | **Implemented, tested** | schema version 2 with migration of version-1 presets, newer versions refused with a reason, validated import/export, duplicate any preset, favourites, custom edit/delete, ids cannot shadow built-ins | delivery-target tags beyond the built-in groups |
| P0-06 Export & delivery | **Implemented, tested** | destination, name template, overwrite policy, Unicode names, metadata policy (keep / strip), post-export report with size change, **SHA-256** (checked against Windows `Get-FileHash`), delivery **manifest** JSON, estimated size where it can be computed from a target bitrate | estimate for quality-based video (depends on content, deliberately not guessed); manifest import/verify tool |
| P0-07 Hardware detection | **Implemented, tested on one PC** | real test encodes find working NVENC/AMF/QSV; fallback to software is tested by forcing a broken GPU family; advanced settings force software and say so; presets the installed FFmpeg cannot run are disabled with the reason | only an NVIDIA + AMD machine was tried |
| P0-08 Progress & diagnostics | **Implemented, tested** | percent (never goes backwards), fps, speed, elapsed, ETA, "Finalizing", encoder used, FFmpeg output tail, copy report | frame counter, error codes, report file export (copy to clipboard exists) |
| P0-09 Source & output preview | **Partial, tested** | source frame preview with scrub and frame stepping; **A/B compare** of the real source and exported file at the same moment with a wipe divider | video playback; side-by-side before encoding |
| P0-10 FFmpeg packaging & validation | **Partial** | safe discovery, version in the footer, friendly message when missing, encoder availability check, 31-file generated corpus with a "no silent failure" rule, `THIRD_PARTY_NOTICES.md` | FFmpeg is **not bundled**; no clean-Windows-VM test; no SBOM |

## Changes in 0.2.0
* Progress bar bug found by CI (went 97 → 69 → 100 %) fixed; unavailable encoders handled; tests made portable.
* P0-01/02/03/05/06/09 features above; P0-07 fallback and P0-10 corpus tests added.

## Known limits that matter to users
* Installer is **not code-signed** (SmartScreen warning).
* FFmpeg must be installed separately.
* HDR sources are converted without tone mapping (a warning is shown).
* Tested on the developer's machine only.
