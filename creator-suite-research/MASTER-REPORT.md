# Master report — Apple Creator Studio apps research

Date: 2026-11-17 · Scope: research only (no app built) · Details: 00-environment-and-method.md

## Bottom line
- **No Apple binaries are on this PC and none of the apps run on Windows.** Therefore there is **no verified binary evidence and no runtime evidence** for any of the 10 apps. REA tools are present but had nothing to analyse; Ghidra and Java are not installed.
- All findings come from public documentation and third-party material read via web-search summaries. Treat as medium/low confidence.
- A Windows implementation path was verified for the first app (Compressor-alternative).

## Status per app
| # | App | Strongest evidence | Biggest unknown | File-format knowledge | Blockers |
|---|---|---|---|---|---|
| 1 | Final Cut Pro | Library/Event/generated-media model; FCPXML is Apple-documented | Engine internals, `.fcpbundle` internals | FCPXML: documented (DOC). Library: unknown | binary, Mac |
| 2 | Logic Pro | `.logicx` package layout (3P), save modes (DOC) | `ProjectData` binary format, audio engine | partial (3P) | binary, Mac |
| 3 | Pixelmator Pro | PXD stores non-destructive edits (DOC); pxdlib exists (3P) | PXD structure | partial (3P) | binary |
| 4 | Keynote | IWA = Snappy+protobuf in ZIP (3P) | Schemas, layout/animation engine | good (3P) | binary |
| 5 | Pages | Same IWA family (3P) | Text layout/pagination | good (3P) | binary |
| 6 | Numbers | IWA + formulas stored (3P: numbers-parser) | Formula engine | good (3P) | binary |
| 7 | Freeform | iCloud sync, PDF export (DOC) | Entire file format | none | binary, format |
| 8 | Motion | Replicator/particle concepts (DOC) | `.motn` structure (unverified) | none | binary |
| 9 | Compressor | Batches, settings, shared computers (DOC) | Service architecture, preset format | none | binary |
| 10 | MainStage | Channel strips, AU hosting, buffer settings (DOC) | Concert format, engine | none | binary |

## Verified vs unknown (summary)
- **Verified (this PC):** hardware, OS, toolchain, FFmpeg encoder/hwaccel list, absence of target binaries.
- **Documented:** features, project concepts, generated-media classes, FCPXML existence, MainStage buffer/latency controls, Compressor distributed processing.
- **Third-party (unverified):** iWork IWA format, `.logicx` layout, pxdlib.
- **Unknown/blocked:** everything internal (algorithms, memory, GPU use, schemas) — never claimed.

## Shared architecture (short)
Rust workspace of engines (`cs-store, cs-settings, cs-undo, cs-queue, cs-media, cs-audio, cs-render, cs-text, cs-doc, cs-calc, cs-anim, cs-timeline, cs-plugin, cs-ipc, cs-ui-kit`), Tauri+Svelte shells, FFmpeg subprocess, SQLite, wgpu, cpal, CLAP/VST3. Full design: shared-architecture/ecosystem-design.md.

## Implementation order
Compressor → Pixelmator → Keynote → Pages → Numbers → Freeform → Motion → Final Cut → Logic → MainStage (+ optional launcher). Table and effort guesses: development/master-roadmap.md.

## Compressor first tasks
CMP-001 repo+CI → CMP-002 prove Tauri build on D: → CMP-004/005 fixtures + probe → CMP-006/007/008 presets, argv builder, runner with progress. Full list and 18 tests: compressor/tasks.md, compressor/implementation-spec.md.

## Document map
- Per-app dossiers (12 files each): final-cut-pro/, logic-pro/, pixelmator-pro/, keynote/, pages/, numbers/, freeform/, motion/, compressor/, mainstage/
- shared-architecture/: ecosystem-design.md, interoperability-matrix.md, shared-engines.md
- development/: master-roadmap.md, dependency-graph.md, task-backlog.md, release-plan.md, licensing-register.md, verification.md

## What would raise confidence
1. Lawfully owned Apple app bundles (or a Mac) for REA Mach-O/Swift/ObjC analysis.
2. Full crawl of Apple guides (menus, shortcuts, settings).
3. User-created sample files (.key, .numbers, .pxd, .logicx, .fcpbundle) for format study.
4. Online verification of dependency licenses.

## Addendum 2026-11-17 — Xcode (reference app, not one of the 10)
- Dossier: `xcode/` (12 files). Sources: the four URLs the user supplied. A sub-agent read the Apple pages and the blog in full and the first ~40k characters of xcodereleases.com; the rest of that page was summarised or unread. The agent's report is secondhand, so confidence is medium.
- **No Xcode binary on this PC** (only a user Flutter project containing `.xcodeproj` folders, not analysed). Xcode is macOS-only, so no static analysis with REA and no runtime study here. Verified binary evidence: none.
- Documented facts: newest listed build Xcode 27.3 Beta 2 (16 Nov 2026), latest full release Xcode 27.2 (29 Oct 2026), both requiring macOS 26.6+, bundled Swift 6.4.x and 27.x SDKs; Apple's pages describe previews, Device Hub, Instruments (Swift Concurrency instrument), Swift Testing/XCTest, Xcode Cloud and agent support via Agent Client Protocol and MCP. Download size not found.
- Conflict resolved: the blog says "version 16" and a paid Xcode Cloud; Apple's pages show Xcode 27, so the blog is stale on version.
- Not planned as a suite app: an Xcode clone is out of scope; Apple's SDK/Xcode licence terms (general knowledge, **verify in the actual licence text**) restrict use to Apple hardware and limit reverse engineering. Reusable open pieces: LLDB, clang, SourceKit-LSP, SwiftPM.
