# Numbers — application architecture

Evidence classes: **DOC** documented behavior (Apple guides/developer docs, via web search), **3P** third-party reverse-engineering/community material (search snippets only, not opened or verified), **INF** inference, **UNK** unknown, **BLK** blocked (needs binary/runtime unavailable here). No finding in this folder is *verified binary evidence*: no Apple binary was available.

## Findings
- **[3P]** ZIP of images and Snappy-compressed protobuf IWA files holding metadata, text, cell definitions.
- **[3P]** numbers-parser ships protobuf definitions extracted from a copy of Numbers; newer Numbers can produce unreadable files.
- **[3P]** Formula text extraction is supported optionally; early versions relied on cached values.
- **[UNK]** Formula engine and dependency graph internals.

## Framework dependencies (probable)
- Foundation / Core Graphics — **INF** — Typical Apple stack

## Plugins and extensions
None known.

## Memory management
UNK.

## CPU/GPU acceleration
UNK.

## Caching and background processing
3P: cached cell values stored in file alongside formulas.

## What would settle the unknowns
Lawfully obtained binary of Numbers analysed with REA (macOS/arm64/x86_64 Mach-O, `open_binary`, `inspect_macho`, `analyze_swift_types`, `inspect_artifact`) plus authorised runtime observation on a Mac. Neither is available in this session.
