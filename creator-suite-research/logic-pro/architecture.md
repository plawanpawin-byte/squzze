# Logic Pro — application architecture

Evidence classes: **DOC** documented behavior (Apple guides/developer docs, via web search), **3P** third-party reverse-engineering/community material (search snippets only, not opened or verified), **INF** inference, **UNK** unknown, **BLK** blocked (needs binary/runtime unavailable here). No finding in this folder is *verified binary evidence*: no Apple binary was available.

## Findings
- **[DOC]** A project is a package (.logicx) or folder; package saving duplicates assets, folder saving references them.
- **[3P]** Package contains Alternatives/NNN/ProjectData (undocumented little-endian binary), MetaData.plist, Resources/ProjectInformation.plist; backups under Alternatives/000. Source: community parsers, not verified here.
- **[INF]** Real-time audio graph with per-track channel strips, MIDI event layer, and plug-in hosting through Audio Units.
- **[UNK]** Audio engine threading, buffer scheduling and automation interpolation are undocumented.

## Framework dependencies (probable)
- Core Audio / Core MIDI / Audio Units — **INF** — Typical Apple stack

## Plugins and extensions
Audio Units (DOC general). Windows has no Audio Units; open equivalents are CLAP, VST3, LV2.

## Memory management
UNK (BLK). DOC: freeze files used to offload CPU.

## CPU/GPU acceleration
UNK. Audio engine is CPU-bound by design (INF).

## Caching and background processing
DOC: freeze files, bounces, alternatives backups.

## What would settle the unknowns
Lawfully obtained binary of Logic Pro analysed with REA (macOS/arm64/x86_64 Mach-O, `open_binary`, `inspect_macho`, `analyze_swift_types`, `inspect_artifact`) plus authorised runtime observation on a Mac. Neither is available in this session.
