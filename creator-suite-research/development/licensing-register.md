# Open-source dependency licensing register

**Caution:** licenses below come from general knowledge and were *not* re-verified online in this session. Verify each at adoption time (cargo-deny / SPDX) and record the version. Not legal advice.

| Dependency | Typical license | Obligations / notes | Used for |
|---|---|---|---|
| Rust toolchain/std | MIT OR Apache-2.0 | Attribution in distributed builds for crates | all |
| Tauri 2 | MIT OR Apache-2.0 | Notice | app shells |
| Svelte / Vite | MIT | Notice | UI |
| Microsoft WebView2 runtime | Microsoft redistributable terms (proprietary, free) | Bootstrapper/evergreen; not open source | Windows UI runtime |
| SQLite / rusqlite | Public domain / MIT | None / notice | storage |
| FFmpeg | LGPL-2.1+ (default); GPL-2+/3 when built with `--enable-gpl` (x264, x265); "nonfree" builds not redistributable | **Run as separate executable** to keep our license independent; ship LGPL-only build or let user supply binary; provide source offer for build we ship. The build on this PC is gyan.dev "full" (GPL) — fine for local testing, not for bundling into a permissive app | media |
| x264 / x265 | GPL-2.0(+) (commercial available) | Only in GPL build variant | H.264/HEVC software encode |
| SVT-AV1 | BSD-3-Clause-Clear + AOM patent license | Notice | AV1 |
| libvpx, libopus, FLAC | BSD-3 | Notice | VP9, Opus, FLAC |
| LAME | LGPL-2 | Dynamic/separate | MP3 |
| fdk-aac | Fraunhofer license (GPL-incompatible, patent caveats) | Avoid; use FFmpeg native AAC | — |
| Patents (H.264, HEVC, AAC) | Patent pools | Open-source code is fine; commercial distribution of encoders may need patent licenses in some regions; hardware encoders via OS drivers shift this to the vendor | Compressor |
| NVENC / AMF / QSV | Vendor drivers (proprietary) | Used through FFmpeg/driver, no redistribution | hw encode |
| MLT | LGPL-2.1+ (some modules GPL) | Dynamic link or subprocess | timeline/editing |
| wgpu, cpal | MIT OR Apache-2.0 | Notice | render, audio |
| ASIO SDK | Steinberg proprietary / GPLv3 dual | Cannot redistribute the SDK freely; optional | Windows low-latency |
| CLAP | MIT | Notice | plug-ins |
| VST3 SDK | MIT since v3.8 (late 2025) — previously GPLv3/proprietary; **verify** | Trademark rules for "VST" | plug-ins |
| LV2 | ISC | Notice | Linux plug-ins |
| JUCE | AGPLv3 / commercial | Strong copyleft; avoid unless suite is AGPL | (rejected default) |
| Qt | LGPLv3 / GPLv3 / commercial | Dynamic linking, relink rights | alternative UI |
| Slint | GPLv3 / royalty-free / commercial | Terms with attribution | (rejected default) |
| HarfBuzz | MIT | Notice | text |
| ICU | Unicode License | Notice | text |
| cosmic-text | MIT OR Apache-2.0 | Notice | text |
| OpenImageIO, OpenColorIO, OpenEXR | BSD-3 / BSD-3 / BSD-3 | Notice | images/color |
| protobuf (if any reader) | BSD-3 | Notice | optional importers |
| Snappy | BSD-3 | Notice | optional importers |
| numbers-parser | MIT (verify) | Any schemas it ships were extracted from Apple software — **do not copy** | reference only |
| pxdlib | unknown (verify) | Reference only | reference only |
| libetonyek | MPL-2.0 | File-level copyleft | optional iWork import (study first) |

## Suite license recommendation
- If FFmpeg is only invoked as a subprocess and no GPL code is linked: Apache-2.0 or MIT is possible for the suite, but GPL-3.0-or-later is simpler for contributors and protects openness. Decision gate SUITE-001.
