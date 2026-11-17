# Open timeline editor (later phase) — implementation spec (draft)

**Status:** research stage only; not scheduled for implementation now (user instruction: study first, do not build).

## Goals
- Independent, original, offline-first, cross-platform (Windows first) application covering the documented feature set for Final Cut Pro.
- Runs standalone; the launcher is optional.

## Approach
Model timeline as spine + attached children with explicit offsets; store in SQLite + JSON; render via MLT or custom wgpu compositor; import/export FCPXML as a documented interchange format.

## Shared engines used
See ../shared-architecture/shared-engines.md.

## Out of scope
Pixel/behaviour cloning of Apple UI, reuse of Apple assets, bypassing any protection.

## Open questions
- Which file format features of Final Cut Pro do users need interoperability for?
- Lawful route to harvest more format detail (own sample files created by the user).
