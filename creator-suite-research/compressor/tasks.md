# Compressor alternative — implementation tasks (not started)

Order is the recommended build order. Work on D: (C: has ~4 GB free).

| ID | Task | Depends on | Acceptance criteria |
|---|---|---|---|
| CMP-001 | Create repo/workspace, CI (Win/Linux/mac), cargo-deny | — | CI green on empty workspace; license check fails on GPL crate |
| CMP-002 | Verify `cargo tauri` build on this PC (hello window) | 001 | Installer builds on D:, launches, WebView2 OK |
| CMP-003 | `cs-store`: schema + migrations | 001 | Migration tests; kill-during-write test leaves valid DB |
| CMP-004 | Fixture generator via FFmpeg lavfi | 001 | Script produces 10 fixture files deterministically (hash-stable metadata) |
| CMP-005 | `cs-probe` | 004 | T01 passes |
| CMP-006 | `cs-presets` + built-ins | 003 | All presets validate; T12 passes |
| CMP-007 | `cs-ffmpeg` arg builder | 005,006 | Golden-file tests of argv for each preset |
| CMP-008 | Runner + progress parser | 007 | T02, T04 pass |
| CMP-009 | Cancel with process-tree kill (Windows Job Object) | 008 | T05 passes on Windows |
| CMP-010 | `cs-queue` scheduler/workers | 003,008 | T06, T07 pass |
| CMP-011 | HW detection + fallback | 008 | T11 passes on RTX 4050 |
| CMP-012 | `squeeze-cli` | 010 | T13 passes |
| CMP-013 | Settings + FFmpeg discovery | 003 | Discovery order test; missing FFmpeg → friendly error |
| CMP-014 | Tauri commands + event bridge | 010 | UI receives job events within 200 ms |
| CMP-015 | UI: queue table, add files, preset picker | 014 | T17 manual script passes |
| CMP-016 | UI: custom settings inspector, destination/template | 015 | T09, T10 pass |
| CMP-017 | Error handling, logs viewer | 015 | T08, T15 pass |
| CMP-018 | Installer + offline verification | 016 | T14 passes; clean-VM install |
| CMP-019 | Cross-platform CI tests | 012 | T18 passes |
| CMP-020 | Docs, SBOM, release v1.0 | all | Release criteria checklist all ticked |

## First concrete steps (when the user approves building)
1. CMP-001 and CMP-002 (prove the toolchain: Tauri build on D:).
2. CMP-004/005 (fixtures + probe) — gives a testable base within a day.
3. CMP-006/007/008 — first CLI encode working end-to-end.
