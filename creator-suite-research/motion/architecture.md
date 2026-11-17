# Motion — application architecture

Evidence classes: **DOC** documented behavior (Apple guides/developer docs, via web search), **3P** third-party reverse-engineering/community material (search snippets only, not opened or verified), **INF** inference, **UNK** unknown, **BLK** blocked (needs binary/runtime unavailable here). No finding in this folder is *verified binary evidence*: no Apple binary was available.

## Findings
- **[DOC]** Replicator builds static copies in a pattern (not a simulation); particle systems comprise an emitter and life cycle.
- **[DOC]** Behaviors (e.g. Scale Over Life) animate objects procedurally.
- **[UNK]** Project file structure was not confirmed by search (.motn XML claim UNVERIFIED).
- **[INF]** Layer/object tree with keyframe curves and behavior evaluators, GPU compositing.

## Framework dependencies (probable)
- Metal / Core Image — **INF** — Typical Apple stack

## Plugins and extensions
FxPlug (INF/general Apple docs).

## Memory management
UNK.

## CPU/GPU acceleration
UNK; marketing says GPU-accelerated (not verified).

## Caching and background processing
UNK.

## What would settle the unknowns
Lawfully obtained binary of Motion analysed with REA (macOS/arm64/x86_64 Mach-O, `open_binary`, `inspect_macho`, `analyze_swift_types`, `inspect_artifact`) plus authorised runtime observation on a Mac. Neither is available in this session.
