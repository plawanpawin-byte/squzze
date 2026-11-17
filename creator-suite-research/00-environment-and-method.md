# Environment discovery and method

Date: 2026-11-17. Scope instruction from the user: **research only, do not build any application.**

## Machine (verified by commands run in this session)
| Item | Value |
|---|---|
| OS | Windows 11 Home Single Language 10.0.26200, 64-bit |
| CPU | AMD Ryzen 7 7445HS, 6 cores / 12 threads |
| GPUs | AMD Radeon 740M (iGPU); NVIDIA GeForce RTX 4050 Laptop, driver 610.74 |
| RAM | 15.2 GB |
| Disk | C: 4.3 GB free (tight), D: 23.1 GB free — build on D: |
| Dev tools | Node 24.15.0, Git 2.53, Python 3.14, Rust 1.97.1 (stable MSVC default; also GNU target and wasm32), Cargo 1.97.1, CMake and GCC (mingw), clang (Swift toolchain), .NET, WSL2 Ubuntu (stopped), Visual Studio folders present, WebView2 runtime 157.0.4341.46 |
| FFmpeg | 8.1.2 "full build" (gyan.dev) with libx264, libx265, libsvtav1, libvpx, libopus, ProRes (prores, prores_ks, prores_aw), and hardware encoders h264/hevc/av1 for NVENC, AMF, QSV; hwaccels: cuda, vaapi, dxva2, qsv, d3d11va, opencl, vulkan, d3d12va, amf |

## Analysis tooling
| Tool | Status |
|---|---|
| REA MCP tools (`mcp__rea__*`) | Available in this session (tool names listed by the harness). Not exercised on any target because no target exists. |
| `rea` CLI / `rea-agents` via npx | Not installed. Documented route: `npx -y rea-agents@6.7.0 doctor --json`; setup writes client config and needs approval. Not run. |
| Ghidra / `analyzeHeadless` | Not found. Needs Java (not installed). |
| Java | Not installed. |
| Hopper / IDA / JADX | Not found. |

## Target availability (searched Downloads, D:, user profile)
No `.app`, `.dmg`, Apple `.pkg`, `.fcpbundle`, `.logicx`, `.pxd`, `.motn`, `.key`, `.pages` or `.numbers` file for any of the 10 target apps was found. The `.ipa` files in Downloads belong to unrelated apps and were not examined. The `.pkg` hits are from the user's own build outputs, not Apple apps.

Apple's apps are macOS/iPadOS software; they cannot run on this Windows machine, so **no runtime observation is possible**.

## Method actually used
1. Local commands to inventory machine, toolchain and FFmpeg capabilities (verified).
2. Web searches (standard mode) on Apple documentation, Apple developer pages and third-party reverse-engineering projects. Only search-result summaries were read; individual pages were not opened. Treat all web findings as medium/low confidence.
3. Document generation by script from a reviewed data set, then manual review of the high-risk claims.

## What this means
- **Verified binary evidence: none for any of the 10 apps.**
- Findings are documented behavior (DOC), third-party (3P), inference (INF), unknown (UNK) or blocked (BLK).
- To get real binary evidence the user must supply lawfully owned macOS app bundles (or an Apple Silicon Mac) and Java for Ghidra, or use REA's Mach-O tools.

## Legal and ethical boundaries followed
No subscription, licensing, DRM or encryption bypass; no Apple code, assets or schemas copied; independent implementation only.
