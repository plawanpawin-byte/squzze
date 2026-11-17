# Keynote — application architecture

Evidence classes: **DOC** documented behavior (Apple guides/developer docs, via web search), **3P** third-party reverse-engineering/community material (search snippets only, not opened or verified), **INF** inference, **UNK** unknown, **BLK** blocked (needs binary/runtime unavailable here). No finding in this folder is *verified binary evidence*: no Apple binary was available.

## Findings
- **[3P]** iWork bundles are ZIP archives; documents are Snappy-compressed protobuf streams (.iwa) with Apple's non-standard framing (no CRC); /Index/Document.iwa is the main entry.
- **[3P]** Protobuf schemas are embedded in app binaries; community extracted them by scanning executables and runtime inspection; type ID -> message mapping differs per app.
- **[INF]** Slide tree with masters/layouts, drawables with transforms, build (animation) timelines.
- **[BLK]** Schema extraction from the actual Keynote binary was not performed here.

## Framework dependencies (probable)
- Core Animation / Core Text / Metal — **INF** — Typical Apple stack

## Plugins and extensions
None known.

## Memory management
UNK.

## CPU/GPU acceleration
UNK; transitions likely GPU (INF).

## Caching and background processing
UNK.

## What would settle the unknowns
Lawfully obtained binary of Keynote analysed with REA (macOS/arm64/x86_64 Mach-O, `open_binary`, `inspect_macho`, `analyze_swift_types`, `inspect_artifact`) plus authorised runtime observation on a Mac. Neither is available in this session.
