# Pixelmator Pro — application architecture

Evidence classes: **DOC** documented behavior (Apple guides/developer docs, via web search), **3P** third-party reverse-engineering/community material (search snippets only, not opened or verified), **INF** inference, **UNK** unknown, **BLK** blocked (needs binary/runtime unavailable here). No finding in this folder is *verified binary evidence*: no Apple binary was available.

## Findings
- **[DOC]** PXD stores non-destructive edits so they can be adjusted after reopening.
- **[3P]** pxdlib reads/writes .pxd with raster, vector and text layers; its author says longer RE documentation exists (not retrieved).
- **[INF]** Layer tree of typed layers with adjustment/effect stacks, rendered by GPU-accelerated compositor.
- **[UNK]** Internal package layout was not found in this session.

## Framework dependencies (probable)
- Core Image / Metal / Core ML — **INF** — Typical Apple stack

## Plugins and extensions
UNK.

## Memory management
UNK (BLK).

## CPU/GPU acceleration
UNK; Apple docs describe GPU-accelerated editing at marketing level only.

## Caching and background processing
DOC: sidecar files; PXD retains edit history data.

## What would settle the unknowns
Lawfully obtained binary of Pixelmator Pro analysed with REA (macOS/arm64/x86_64 Mach-O, `open_binary`, `inspect_macho`, `analyze_swift_types`, `inspect_artifact`) plus authorised runtime observation on a Mac. Neither is available in this session.
