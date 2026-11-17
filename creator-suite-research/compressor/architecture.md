# Compressor — application architecture

Evidence classes: **DOC** documented behavior (Apple guides/developer docs, via web search), **3P** third-party reverse-engineering/community material (search snippets only, not opened or verified), **INF** inference, **UNK** unknown, **BLK** blocked (needs binary/runtime unavailable here). No finding in this folder is *verified binary evidence*: no Apple binary was available.

## Findings
- **[DOC]** Batches contain jobs; settings and destinations define output; distributed processing sends portions of a batch to shared computers.
- **[DOC]** Compressor 4.5 release notes: runs natively on Apple silicon and Intel.
- **[INF]** Frontend UI plus background encoding service(s) and a job queue; Share destinations from Final Cut call into it.
- **[UNK]** Service layout (helper processes, XPC) not documented here.

## Framework dependencies (probable)
- AVFoundation / VideoToolbox — **INF** — Typical Apple stack

## Plugins and extensions
UNK.

## Memory management
UNK.

## CPU/GPU acceleration
DOC marketing-level hardware acceleration; detail BLK.

## Caching and background processing
UNK.

## What would settle the unknowns
Lawfully obtained binary of Compressor analysed with REA (macOS/arm64/x86_64 Mach-O, `open_binary`, `inspect_macho`, `analyze_swift_types`, `inspect_artifact`) plus authorised runtime observation on a Mac. Neither is available in this session.
