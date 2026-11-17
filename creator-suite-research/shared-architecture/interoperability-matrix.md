# Interoperability matrix

Rows export to columns. ● = planned native path using an open/standard format. ○ = best-effort importer. — = n/a. Apple-format fidelity is **not** promised (formats are largely undocumented; see per-app file-formats.md).

| From \ To | Compressor | Pixelmator | Keynote | Pages | Numbers | Freeform | Motion | Final Cut | Logic | MainStage |
|---|---|---|---|---|---|---|---|---|---|---|
| Compressor | — | ● frames/images | ● video/media | ● media | — | ● media | ● media | ● media | ● audio | ● audio |
| Pixelmator | ● PNG/EXR/PSD | — | ● images | ● images | ● images | ● images | ● layers (PSD/EXR) | ● stills | — | — |
| Keynote | ● PDF/video | ● PNG | — | ● text | ● tables CSV | ● PDF/images | — | ● video | — | — |
| Pages | ● PDF | ● PNG | ● text | — | ● tables CSV | ● PDF | — | — | — | — |
| Numbers | — | ● PNG charts | ● tables/charts | ● tables | — | ● PNG | — | — | — | — |
| Freeform | ● PDF | ● PNG | ● images | ● images | — | — | — | — | — | — |
| Motion | ● movie/ProRes | ● frames | ● video | — | — | ● video | — | ● templates (own format) | — | — |
| Final Cut | ● send for encode | ● stills | ● video | — | — | — | ● clip round-trip | — | ● audio/AAF/OTIO | — |
| Logic | ● stems | — | — | — | — | — | — | ● audio/OTIO | — | ● patches/plug-in chains |
| MainStage | ● recordings | — | — | — | — | — | — | — | ● patches | — |

## Standard interchange formats used
PNG/JPEG/TIFF/EXR/PSD (images), PDF, CSV/XLSX, SVG, MIDI, WAV/FLAC/AIFF, OpenTimelineIO, FCPXML (documented by Apple; versions through 1.10 found on Apple's developer site), Lottie JSON (optional), CLAP/VST3 plug-in state.

## Rules
- Every app must function with zero other suite apps installed.
- Cross-app features degrade gracefully to a file export.
