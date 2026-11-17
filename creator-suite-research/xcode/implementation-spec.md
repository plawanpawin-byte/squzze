# Not planned as a suite app — implementation spec (draft)

**Status:** research stage only; not scheduled for implementation now (user instruction: study first, do not build).

## Goals
- Independent, original, offline-first, cross-platform (Windows first) application covering the documented feature set for Xcode.
- Runs standalone; the launcher is optional.

## Approach
Xcode builds Apple-platform apps and depends on Apple SDKs and licenses restricting use to Apple hardware; an open clone is out of scope. Useful open building blocks instead: LLDB, clang, SourceKit-LSP, SwiftPM (all open source).

## Shared engines used
See ../shared-architecture/shared-engines.md.

## Out of scope
Pixel/behaviour cloning of Apple UI, reuse of Apple assets, bypassing any protection.

## Open questions
- Which file format features of Xcode do users need interoperability for?
- Lawful route to harvest more format detail (own sample files created by the user).
