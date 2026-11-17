# Shared engine specifications (original designs)

Each engine is a Rust crate with a stable public API, unit tests, and no dependency on any app. Versions start at 0.x. Acceptance criteria are testable.

| Crate | Responsibility | Key API (sketch) | Used by | Acceptance criteria |
|---|---|---|---|---|
| `cs-store` | SQLite storage, migrations, document containers | `Store::open`, `migrate`, `tx` | all | Opens/creates DB; migration from v1→v2 preserves data; crash during write leaves a valid DB (kill-test) |
| `cs-settings` | Typed per-app and shared settings (JSON file in OS config dir) | `Settings::load/save/watch` | all | Corrupt file falls back to defaults and is backed up; changes propagate within 1 s |
| `cs-undo` | Command-pattern undo/redo with grouping | `Command`, `History::do/undo/redo` | all editors | 1000-step history; undo then new action clears redo; grouped commands undo atomically |
| `cs-queue` | Persistent job queue, scheduler, worker pool, progress, retry | `Queue::submit/cancel/subscribe` | Compressor, Final Cut exports, Motion renders | Jobs survive app restart; N workers respect concurrency limit; cancel stops the child process within 2 s |
| `cs-media` | Probe, decode, encode, filter graph builder via FFmpeg | `probe(path)`, `EncodePlan`, `run(plan)` | Compressor, FCP, Keynote, Motion | Probe JSON parsed for 20 sample files; encode produces file whose probe matches plan; progress events monotonic |
| `cs-audio` | Real-time audio graph, device I/O, mixing, plug-in hosting glue | `Engine`, `Node`, `Device` | Logic, MainStage | 48 kHz stereo graph, 128-sample buffer, no allocations in callback (checked by test harness), 1 h soak without xruns on reference PC |
| `cs-render` | GPU compositor (wgpu), blend modes, effect shaders, image I/O | `Scene`, `render(frame)` | Pixelmator, Motion, FCP, Keynote, Freeform | Blend mode outputs match reference images within ΔE<1; 4K frame composites under 16 ms on RTX 4050 for 5 layers |
| `cs-text` | Shaping, line breaking, layout, fonts | `Layout::new(text, style, width)` | Pages, Keynote, Numbers, Freeform, Motion | Correct shaping for Latin, Thai, Arabic, CJK samples; pagination deterministic |
| `cs-doc` | Document/object model, styles, serialization (JSON + zip) | `Doc`, `Node`, `Style` | Pages, Keynote, Numbers, Freeform, Pixelmator | Round-trip equality for 50 generated documents; forward-compatible unknown fields preserved |
| `cs-calc` | Spreadsheet formula parser/evaluator, dependency graph | `Sheet::set`, `recalc` | Numbers | Excel-compatible results for 200-function conformance suite; dirty-cell recalculation only touches dependents |
| `cs-anim` | Keyframes, easing curves, behaviors | `Track`, `eval(t)` | Motion, Keynote, FCP | Curve evaluation deterministic across platforms (bit-identical f32 test vectors) |
| `cs-timeline` | Spine/track timeline model, edits, OTIO/FCPXML import-export | `Timeline`, `Edit` | Final Cut, Logic | Ripple/roll/slide edit invariants pass property tests; OTIO round-trip |
| `cs-plugin` | Plug-in discovery and hosting (CLAP/VST3/LV2) + WASM effect API | `PluginHost` | Logic, MainStage, Pixelmator, FCP | Scans and loads 5 reference plug-ins; crash in plug-in does not crash host (sandbox process) |
| `cs-ipc` | Optional local IPC and launcher protocol | JSON-RPC over pipe/socket | all, launcher | App starts without launcher; with launcher, "open file in app" works in <1 s |
| `cs-ui-kit` | Shared UI components (Svelte lib) and design tokens | components | all Tauri apps | Keyboard-navigable, passes axe accessibility checks, dark/light themes |

## Build order
cs-store → cs-settings → cs-queue → cs-media (for Compressor MVP), then cs-undo/cs-doc/cs-text/cs-render for document and image apps, then cs-anim/cs-timeline, last cs-audio/cs-plugin.

## Non-goals
Binary compatibility with any Apple engine, plug-ins or file format.
