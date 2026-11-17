# Final Cut Pro — application architecture

Evidence classes: **DOC** documented behavior (Apple guides/developer docs, via web search), **3P** third-party reverse-engineering/community material (search snippets only, not opened or verified), **INF** inference, **UNK** unknown, **BLK** blocked (needs binary/runtime unavailable here). No finding in this folder is *verified binary evidence*: no Apple binary was available.

## Findings
- **[DOC]** A Library is a package containing Events and Projects; clips in Events reference media rather than being the media.
- **[DOC]** Generated media (render, optimized, proxy, analysis files) are stored inside the library or in user-chosen external locations and can be deleted and regenerated.
- **[INF]** Object model is likely a database/graph of media references, clips, compound clips and project timelines, with a render cache keyed by effect graph. Not confirmed.
- **[UNK]** Internal engine split (decode, effect graph, GPU compositor, cache) is not publicly documented.

## Framework dependencies (probable)
- AVFoundation / Core Media / VideoToolbox / Metal — **INF** — Typical Apple stack; unverified

## Plugins and extensions
Effects/titles/generators/transitions via Motion templates and FxPlug (DOC from general Apple pro-apps docs; exact mechanism UNK this session).

## Memory management
UNK (BLK). Documented only: large media is referenced, not copied, unless the user chooses.

## CPU/GPU acceleration
UNK (BLK). Feature docs mention GPU/hardware encoding; no verified detail.

## Caching and background processing
DOC: render, optimized, proxy, analysis files; Delete Generated Files removes them.

## What would settle the unknowns
Lawfully obtained binary of Final Cut Pro analysed with REA (macOS/arm64/x86_64 Mach-O, `open_binary`, `inspect_macho`, `analyze_swift_types`, `inspect_artifact`) plus authorised runtime observation on a Mac. Neither is available in this session.
