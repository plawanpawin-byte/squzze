# Open motion graphics tool — implementation spec (draft)

**Status:** research stage only; not scheduled for implementation now (user instruction: study first, do not build).

## Goals
- Independent, original, offline-first, cross-platform (Windows first) application covering the documented feature set for Motion.
- Runs standalone; the launcher is optional.

## Approach
Node-less layer tree + keyframes; shader-based effects in WGSL; Lottie/After Effects-like JSON export optional.

## Shared engines used
See ../shared-architecture/shared-engines.md.

## Out of scope
Pixel/behaviour cloning of Apple UI, reuse of Apple assets, bypassing any protection.

## Open questions
- Which file format features of Motion do users need interoperability for?
- Lawful route to harvest more format detail (own sample files created by the user).
