# Creator Suite — proposed ecosystem design

Working name: **Creator Suite** (name not trademark-checked). Ten standalone apps plus an optional launcher. This is an original design, not a model of Apple's internals (which are unknown).

## Principles
1. Offline-first: no mandatory server, account, cloud DB or paid API. Sync/collab are optional later modules.
2. Every app is a separate executable and runs without the launcher.
3. One shared Rust workspace of engines (crates) + thin per-app shells.
4. Open, documented, versioned file formats (SQLite + JSON/Zip containers) with import/export for common interchange formats.
5. Windows first, then Linux and macOS from the same code.

## Layered architecture
```
Apps (10 + launcher)           Tauri shell (Rust + web UI) per app
  |
App core crates                cs-<app>-core (domain logic per app)
  |
Shared engines                 cs-media  cs-audio  cs-render  cs-doc  cs-text
                               cs-queue  cs-store  cs-plugin  cs-undo  cs-ipc  cs-settings  cs-ui-kit
  |
Platform adapters              ffmpeg subprocess/libav, WASAPI/ASIO/CoreAudio/ALSA via cpal,
                               wgpu (D3D12/Vulkan/Metal), OS file dialogs, notify, WebView
```

## Technology decision (recommended)
| Concern | Choice | Why | Alternatives considered |
|---|---|---|---|
| Core language | Rust | Memory safety, MSVC toolchain already installed, great for real-time/audio and concurrency | C++ (more existing media code, higher risk), Go |
| UI shell | Tauri 2 + Svelte/TypeScript | Small binaries, WebView2 already present on this PC, fast UI iteration, MIT/Apache | Qt (LGPL, heavy, best for dense pro UIs and DAW-style), egui/iced (pure Rust, less polished text), Slint (GPL/commercial caveat) |
| Heavy native UI (timeline/DAW later) | Re-evaluate Qt Quick or wgpu custom canvas when starting Final Cut/Logic alternatives | Timeline and mixer UIs strain web views | — |
| Media | FFmpeg (subprocess first, libav later) | Covers decode/encode/filters; user machine has full build with hw encoders | GStreamer, MLT (good for editing/timelines, built on FFmpeg) |
| Timeline/compositing | MLT (editing) or custom wgpu | MLT is LGPL-2.1 and proven (Shotcut/Kdenlive) | OpenTimelineIO for interchange |
| Audio I/O | cpal (WASAPI; ASIO via optional SDK) | MIT/Apache | PortAudio, JUCE (AGPL/commercial) |
| Plugins | CLAP + VST3 (+LV2 on Linux); own WASM plugin API for non-audio | CLAP is MIT; VST3 SDK license changed to MIT in late 2025 (verify) | Audio Units only exist on macOS |
| Storage | SQLite (rusqlite) | Public domain, embeddable | sled/redb |
| Text | HarfBuzz + ICU + cosmic-text | Correct shaping, i18n | Pango |
| Graphics | wgpu + Skia/vello/tiny-skia | D3D12/Vulkan/Metal portability | OpenGL |

## App map and dependencies
| # | App | Primary shared engines |
|---|---|---|
| 1 | Compressor-alt (**first**) | cs-media, cs-queue, cs-store, cs-settings |
| 2 | Pixelmator-alt | cs-render, cs-doc, cs-undo, cs-store |
| 3 | Keynote-alt | cs-doc, cs-text, cs-render, cs-media |
| 4 | Pages-alt | cs-doc, cs-text, cs-render |
| 5 | Numbers-alt | cs-doc, cs-calc, cs-text, cs-render |
| 6 | Freeform-alt | cs-doc, cs-render, cs-undo |
| 7 | Motion-alt | cs-render, cs-media, cs-anim |
| 8 | Final Cut-alt | cs-media, cs-render, cs-timeline, cs-queue |
| 9 | Logic-alt | cs-audio, cs-plugin, cs-timeline, cs-undo |
| 10 | MainStage-alt | cs-audio, cs-plugin |
| + | Launcher (optional) | cs-ipc, cs-settings |

## Interapplication communication (optional, never required)
- Files are the contract (open formats + FCPXML/OTIO/XLSX/PDF etc.).
- Optional local IPC: named pipe (Windows) / Unix socket with a small JSON-RPC protocol; apps discover each other via a per-user registry file. If absent, "Send to Compressor" falls back to spawning `compressor-cli`.

## Feasibility on Windows (checked)
See ../development/verification.md. Toolchain components needed for stages 1–2 are present on this machine.
