# Open live host — implementation spec (draft)

**Status:** research stage only; not scheduled for implementation now (user instruction: study first, do not build).

## Goals
- Independent, original, offline-first, cross-platform (Windows first) application covering the documented feature set for MainStage.
- Runs standalone; the launcher is optional.

## Approach
Plug-in host via CLAP/VST3 (check licenses), mapping layer, WASAPI/ASIO low-latency I/O through cpal/ASIO SDK (ASIO SDK license caution).

## Shared engines used
See ../shared-architecture/shared-engines.md.

## Out of scope
Pixel/behaviour cloning of Apple UI, reuse of Apple assets, bypassing any protection.

## Open questions
- Which file format features of MainStage do users need interoperability for?
- Lawful route to harvest more format detail (own sample files created by the user).
