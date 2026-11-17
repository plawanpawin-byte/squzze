# Third-party notices

Squeeze is GPL-3.0-or-later. It runs and links to the following. **Licence names below are from the projects' public
documentation as known when this file was written; they have not been machine-verified here. Check each project's own
licence text before redistributing.**

| Component | Use | Licence (to verify) |
|---|---|---|
| FFmpeg / ffprobe | **Not bundled.** Started as a separate program from the user's installation. Builds that include x264/x265 are GPL. | LGPL-2.1+ or GPL-2+/3 depending on build |
| Tauri 2 and its Rust plugins | desktop shell | MIT or Apache-2.0 |
| WebView2 | web view on Windows (Microsoft runtime, not bundled) | Microsoft licence |
| Svelte, SvelteKit, Vite | user interface (build time) | MIT |
| Rust crates (serde, serde_json, thiserror, base64, windows-sys …) | engines | MIT / Apache-2.0 |

`cargo audit` (RustSec) and `npm audit` results are recorded in SECURITY.md. A full SBOM is not generated yet.
