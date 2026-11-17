# Pages — application architecture

Evidence classes: **DOC** documented behavior (Apple guides/developer docs, via web search), **3P** third-party reverse-engineering/community material (search snippets only, not opened or verified), **INF** inference, **UNK** unknown, **BLK** blocked (needs binary/runtime unavailable here). No finding in this folder is *verified binary evidence*: no Apple binary was available.

## Findings
- **[3P]** Same IWA container family as Keynote/Numbers.
- **[INF]** Document model: sections, paragraphs/styles, floating and inline objects, tables, with pagination engine.
- **[UNK]** Text layout and pagination internals.

## Framework dependencies (probable)
- Core Text / TextKit — **INF** — Typical Apple stack

## Plugins and extensions
None known.

## Memory management
UNK.

## CPU/GPU acceleration
UNK.

## Caching and background processing
UNK.

## What would settle the unknowns
Lawfully obtained binary of Pages analysed with REA (macOS/arm64/x86_64 Mach-O, `open_binary`, `inspect_macho`, `analyze_swift_types`, `inspect_artifact`) plus authorised runtime observation on a Mac. Neither is available in this session.
