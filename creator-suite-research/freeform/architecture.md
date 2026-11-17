# Freeform — application architecture

Evidence classes: **DOC** documented behavior (Apple guides/developer docs, via web search), **3P** third-party reverse-engineering/community material (search snippets only, not opened or verified), **INF** inference, **UNK** unknown, **BLK** blocked (needs binary/runtime unavailable here). No finding in this folder is *verified binary evidence*: no Apple binary was available.

## Findings
- **[DOC]** Boards sync through iCloud when enabled; boards can be exported as PDF.
- **[3P]** Community reports say local-only boards have no visible file location and no built-in backup (forum, unverified).
- **[UNK]** No public file format or schema found.

## Framework dependencies (probable)
- PencilKit / Core Data+CloudKit — **INF** — Guess from platform; unverified

## Plugins and extensions
None.

## Memory management
UNK.

## CPU/GPU acceleration
UNK.

## Caching and background processing
UNK.

## What would settle the unknowns
Lawfully obtained binary of Freeform analysed with REA (macOS/arm64/x86_64 Mach-O, `open_binary`, `inspect_macho`, `analyze_swift_types`, `inspect_artifact`) plus authorised runtime observation on a Mac. Neither is available in this session.
