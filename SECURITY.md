# Security

Squeeze is a local, offline desktop app. It has no server, no account, no telemetry and makes no network
requests of its own. This file describes the threat model, what has been checked, and what has not.

## Reporting a vulnerability
Please open a private security advisory on the GitHub repository (Security → Report a vulnerability).
Do not post exploit details in a public issue.

## Threat model
The realistic attacker is **a malicious or malformed media file / preset file** that a user opens, and
**a hostile file name**. There is no remote attacker surface.

## Design decisions
| Risk | Mitigation | Where |
|---|---|---|
| Shell/argument injection through file names | FFmpeg is started with an argument vector, never through a shell. Input paths follow `-i` as a single argument. Tested with hostile names. | `cs-ffmpeg/src/args.rs` |
| FFmpeg opening URLs or pseudo-protocols (`http:`, `concat:`, `subfile:` …) | Inputs must be existing regular files, passed as absolute paths, and every FFmpeg/ffprobe run uses `-protocol_whitelist file`. | `cs-probe`, `cs-ffmpeg` |
| Running a planted `ffmpeg.exe` from the current directory | Tools are located only in absolute directories: `<install>/ffmpeg`, the install folder, then absolute `PATH` entries. The current directory is never searched. | `cs-probe::locate_tool` |
| Overwriting the source or other files via output names | Output names are sanitised (no separators, reserved Windows names, trailing dots); the output can never equal the source; overwrite policy is explicit (rename by default). | `cs-ffmpeg/src/outpath.rs` |
| Half-written output files | Encode to `*.squeeze-part.*`, rename on success, delete on failure/cancel; leftovers from a crash are removed at next start. | `cs-queue` |
| Orphaned FFmpeg processes after a crash | All children are placed in a Windows Job Object with kill-on-close. | `cs-ffmpeg/src/run.rs` |
| Malicious preset JSON | Strict schema + range validation (quality, bitrate, height, fps, channels, container/codec compatibility, id charset). Import size capped at 64 KB. Custom ids cannot shadow built-ins. | `cs-ffmpeg/src/preset.rs` |
| Compromised web UI calling dangerous commands | Content-Security-Policy `default-src 'self'`, no remote content, no `eval`. Capability set is `core:default` only: **no filesystem, shell, HTTP or dialog permission is granted to the page**. File/folder pickers run in Rust. Commands accept preset *ids*, not command lines. | `app/src-tauri/tauri.conf.json`, `capabilities/` |
| Opening arbitrary paths | `reveal_output` only accepts the id of a finished job and reveals that job's own output. | `app/src-tauri/src/lib.rs` |
| Resource exhaustion | Batch probe ≤ 300 files, queue ≤ 5000 jobs, parallel jobs ≤ 4, ffprobe 30 s timeout, preview 15 s / 4 MB cap. | `app/src-tauri/src/lib.rs`, `cs-probe`, `cs-ffmpeg` |
| Bad in/out points | Must be finite, non-negative, out after in and inside the file; validated again in the queue before FFmpeg starts. | `cs-queue::effective_duration` |
| Output that silently differs from the preset | Every finished file is re-probed; codec, height limit, duration and audio presence are checked and shown to the user. | `cs-ffmpeg/src/verify.rs` |
| Corrupt queue/settings files | Corrupt `queue.json` is set aside as `queue.corrupt.json`; invalid settings fall back to defaults and are clamped. | `cs-queue` |

## Verified in this repository
* `cargo test --workspace`: 44 tests including integration tests against a real FFmpeg (all built-in presets are encoded and
  re-probed; cancel, progress, corrupt input, trimming and a real screen recording).
* `cargo audit` (RustSec database, 1527 advisories, 478 crates): **0 vulnerabilities**. 6 informational warnings
  (unmaintained/unsound crates) remain in transitive Tauri dependencies: `glib`, `zbus_names`, `zvariant`,
  `zvariant_derive` are Linux-only and are not compiled into the Windows build; `chrono` and `proc-macro-error` are
  build-time/indirect. They are tracked and will go away as Tauri updates.
* End-to-end test (`scripts/e2e.mjs`) drives the real installed-style exe through WebView2 DevTools: drop files, probe,
  batch, trim, verification, settings, cancel, crash-kill and restart, resizable panels, light/dark themes, plus hostile inputs (URLs, relative paths, path-like preset ids, oversize
  batches) and web-layer isolation (CSP blocks outside network; `fs` and `shell` plugin commands denied by ACL).
* `cargo clippy --workspace --all-targets`: 0 warnings.
* `npm audit --omit=dev`: 0 vulnerabilities in the dependencies shipped inside the app.
* `npm audit` (dev tooling) currently reports advisories in SvelteKit's *server-side* features (form actions, remote
  functions, SSR cache headers) and `cookie`. The shipped app is a static single-page build with no server runtime,
  so these code paths are not present in the released binary. They will be removed by upgrading SvelteKit.

## Not covered / known limits
* **FFmpeg itself.** Media parsers are a large attack surface. Squeeze does not sandbox FFmpeg; keep FFmpeg updated.
  A future hardening step is running FFmpeg in a low-integrity/AppContainer process.
* The installer is **not code-signed**, so Windows SmartScreen will warn on first run. Verify the SHA-256 published with
  each release.
* No third-party penetration test has been done. Supply-chain review of Rust crates (`cargo audit` / `cargo deny`) runs in CI.
* GPU encoders depend on vendor drivers; failures fall back to software encoding automatically.
