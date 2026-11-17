# Verification report

## 1. Coverage of the 10 applications
| App | Docs written | Binary evidence | Runtime evidence | Status |
|---|---|---|---|---|
| Final Cut Pro | 12/12 | none (blocked) | none (impossible on Windows) | researched from docs |
| Logic Pro | 12/12 | none | none | researched from docs + 3P |
| Pixelmator Pro | 12/12 | none | none | researched from docs + 3P |
| Keynote | 12/12 | none | none | researched from 3P format notes |
| Pages | 12/12 | none | none | same |
| Numbers | 12/12 | none | none | same |
| Freeform | 12/12 | none | none | researched from docs; format unknown |
| Motion | 12/12 | none | none | researched from docs; format unverified |
| Compressor | 12/12 (+ detailed MVP spec) | none | none | researched from docs |
| MainStage | 12/12 | none | none | researched from docs |

Each app folder was checked to contain exactly the 12 required files (count verified by script: 120 files across 10 folders).

## 2. Honesty checks
- No claim of recovered proprietary algorithms.
- Every internal-sounding claim carries an evidence class; DOC/3P come from search-result summaries only.
- Items I could not confirm (flagged): `.motn` being XML; PXD internals; Freeform file format; Compressor watch folders; MainStage mode names.

## 3. Windows feasibility (verified by commands)
| Requirement for Compressor MVP | Result |
|---|---|
| Rust MSVC toolchain | rustc/cargo 1.97.1, `stable-x86_64-pc-windows-msvc` default; MSVC folders exist |
| Node for UI tooling | Node 24.15.0 |
| WebView2 | runtime 157.0.4341.46 present |
| FFmpeg with encoders | 8.1.2 with x264/x265/SVT-AV1/VP9/Opus/ProRes and NVENC/AMF/QSV |
| GPU encode hardware | RTX 4050 (NVENC) + Radeon 740M (AMF) |
| Disk | C: 4.3 GB (insufficient for builds), D: 23 GB — build on D: |
Not yet verified: an actual `cargo tauri` build (Tauri CLI not checked), NVENC encode of a sample, Linux/macOS builds. These are tasks CMP-001/002.

## 4. Licensing
Register in licensing-register.md; flagged as not re-verified online.

## 5. Acceptance criteria
Every engine (shared-engines.md) and Compressor task (compressor/tasks.md) has testable criteria.

## 6. Known gaps
1. No binary analysis at all. 2. Web pages not individually opened. 3. Shortcut/menu inventories incomplete. 4. Dependency licenses from memory. 5. Time/effort estimates are guesses.
