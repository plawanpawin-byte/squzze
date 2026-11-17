# Compressor — feature inventory

Source: Apple user guide and related pages (feature level). Not an exhaustive list: the full guide was not crawled; see `limitations.md`.

- Add source media and apply settings
- Presets (Apple devices, web, ProRes, HEVC, H.264 etc.)
- Destinations/locations, output naming
- Batch queue with progress, pause, cancel
- Distributed processing across shared computers
- Watch folders (INF from product positioning; verify in guide)
- Frame rate/size/audio conversions, filters, timecode
- Delivery packages and other specialised outputs (UNK: list not harvested; check the guide)

## Deep-topic coverage
| Topic | Summary | Evidence |
|---|---|---|
| Codec handling | Wide codec/container list. | DOC general |
| Presets | Encoder + filters + destination bundles. | DOC |
| Batch processing | Multiple jobs per batch. | DOC |
| Transcoding | Decode/encode. | DOC |
| Job queue/progress | Per-job progress and status. | DOC feature level |
| Hardware acceleration | Apple silicon encoders. | DOC marketing level; detail BLK |

## Independent implementation notes
For every feature above the suite's own app should expose equivalent *observable* behaviour (inputs, outputs, interactions) without copying Apple implementation. See `implementation-spec.md`.
