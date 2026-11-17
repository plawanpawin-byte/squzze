# MainStage — application architecture

Evidence classes: **DOC** documented behavior (Apple guides/developer docs, via web search), **3P** third-party reverse-engineering/community material (search snippets only, not opened or verified), **INF** inference, **UNK** unknown, **BLK** blocked (needs binary/runtime unavailable here). No finding in this folder is *verified binary evidence*: no Apple binary was available.

## Findings
- **[DOC]** Workspace is a concert containing patches built from channel strips with Insert/Sends/I/O.
- **[DOC]** Audio Units plug-ins can host instruments and effects; some add latency and heavy ones can drop out at small buffers.
- **[DOC]** I/O buffer size, driver latency estimate and I/O safety buffer exist in settings.
- **[INF]** Shares the Logic Pro sound/plug-in engine (not verified).

## Framework dependencies (probable)
- Core Audio / Core MIDI / Audio Units — **INF** — Typical Apple stack

## Plugins and extensions
Audio Units.

## Memory management
UNK.

## CPU/GPU acceleration
None expected.

## Caching and background processing
UNK.

## What would settle the unknowns
Lawfully obtained binary of MainStage analysed with REA (macOS/arm64/x86_64 Mach-O, `open_binary`, `inspect_macho`, `analyze_swift_types`, `inspect_artifact`) plus authorised runtime observation on a Mac. Neither is available in this session.
