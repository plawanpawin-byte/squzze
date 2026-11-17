# Xcode — application architecture

Evidence classes: **DOC** documented behavior (Apple guides/developer docs, via web search), **3P** third-party reverse-engineering/community material (search snippets only, not opened or verified), **INF** inference, **UNK** unknown, **BLK** blocked (needs binary/runtime unavailable here). No finding in this folder is *verified binary evidence*: no Apple binary was available.

## Findings
- **[DOC]** Apple's pages describe Xcode as one app bundling editor, SwiftUI/UIKit/AppKit previews, Device Hub, simulators, debugger, Instruments, testing frameworks, Organizer and Xcode Cloud CI/CD.
- **[DOC]** xcodereleases.com lists, per release, the bundled Swift version and SDK versions (macOS, iOS, watchOS, tvOS, visionOS) with build strings.
- **[3P]** Blog says the editor covers Swift, Objective-C, C and C++, and the debugger is LLDB-based; blog is dated and partly contradicted by Apple's pages.
- **[INF]** Likely split: UI app + build system + SourceKit-style language services + LLDB + simulator runtimes + separately installed SDKs; not confirmed.
- **[UNK]** Internal build system, indexing, preview rendering and simulator architecture.

## Framework dependencies (probable)
- AppKit/SwiftUI, LLDB, clang/Swift compilers, Metal tools — **INF** — Not stated on the four pages

## Plugins and extensions
Instruments custom instruments (DOC); agents via ACP, tools via MCP (DOC).

## Memory management
UNK.

## CPU/GPU acceleration
Instruments tracks GPU; renderer for previews UNK.

## Caching and background processing
UNK (DerivedData is known from general experience, not from these pages).

## What would settle the unknowns
Lawfully obtained binary of Xcode analysed with REA (macOS/arm64/x86_64 Mach-O, `open_binary`, `inspect_macho`, `analyze_swift_types`, `inspect_artifact`) plus authorised runtime observation on a Mac. Neither is available in this session.
