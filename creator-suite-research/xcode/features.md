# Xcode — feature inventory

Source: Apple user guide and related pages (feature level). Not an exhaustive list: the full guide was not crawled; see `limitations.md`.

- Source editor with completion, diagnostics, refactoring (3P)
- Interface Builder with Auto Layout (3P)
- Previews: live, interactive, select; Dark Mode, orientation, text-size settings
- Device Hub and simulators for Apple devices
- Debugging: breakpoints, memory-leak inspection, variable watching; LLDB (3P)
- Instruments: CPU/disk/memory/GPU tracks, custom instruments, Swift Concurrency instrument and run comparison (Xcode 27)
- Testing: Swift Testing, XCTest, XCUIAutomation, performance measurement
- Xcode Cloud CI/CD
- Organizer: app lifecycle, anonymized performance data (launch, memory, responsiveness, battery)
- Coding intelligence: models/agents from Anthropic, Google, OpenAI; any Agent Client Protocol agent; extensible via Model Context Protocol; agent-assisted localization with String Catalogs (Xcode 27)
- Asset catalogs, code signing/entitlements, Swift Package Manager, TestFlight/App Store upload (3P)

## Deep-topic coverage
| Topic | Summary | Evidence |
|---|---|---|
| Build and toolchain | Each release bundles a Swift version and SDK set (e.g. Xcode 27.2: Swift 6.4.2, SDK 27.2). | DOC (xcodereleases.com) |
| Previews | Macro-driven live/interactive/select modes. | DOC |
| Debugging/Instruments | Time-based tracks, concurrency instrument. | DOC |
| Testing and CI | Swift Testing + XCTest + Xcode Cloud. | DOC |
| AI agents | Model-agnostic via ACP and MCP. | DOC |
| Distribution | Released as .xip from Apple downloads; links need Apple developer sign-in (INF from URL pattern). | INF |

## Independent implementation notes
For every feature above the suite's own app should expose equivalent *observable* behaviour (inputs, outputs, interactions) without copying Apple implementation. See `implementation-spec.md`.
