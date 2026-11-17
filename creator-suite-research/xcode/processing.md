# Xcode — rendering / processing pipeline

**Pipeline sketch (INF):** Edit -> index/diagnostics -> build (compile Swift/C-family against SDK, link, sign) -> run on simulator/device -> debug/profile. INF from feature list; no build-system detail found.

## Deep-topic notes
### Build and toolchain
Each release bundles a Swift version and SDK set (e.g. Xcode 27.2: Swift 6.4.2, SDK 27.2).  
*Evidence:* DOC (xcodereleases.com)

### Previews
Macro-driven live/interactive/select modes.  
*Evidence:* DOC

### Debugging/Instruments
Time-based tracks, concurrency instrument.  
*Evidence:* DOC

### Testing and CI
Swift Testing + XCTest + Xcode Cloud.  
*Evidence:* DOC

### AI agents
Model-agnostic via ACP and MCP.  
*Evidence:* DOC

### Distribution
Released as .xip from Apple downloads; links need Apple developer sign-in (INF from URL pattern).  
*Evidence:* INF


## Error handling
UNK.

## Settings and preferences
UNK.
