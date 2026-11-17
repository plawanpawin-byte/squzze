# Xcode — research dossier

**Domain:** IDE and toolchain for Apple platforms (reference app, not part of Creator Studio)
**Priority in suite:** reference app (outside the 10 Creator Studio apps) (see ../development/master-roadmap.md)
**Binary analysis status:** BLOCKED — no binary available. Research is documentation-level only.
**Version studied:** unknown (no binary; Apple guides are version-agnostic web pages)

Evidence classes: **DOC** documented behavior (Apple guides/developer docs, via web search), **3P** third-party reverse-engineering/community material (search snippets only, not opened or verified), **INF** inference, **UNK** unknown, **BLK** blocked (needs binary/runtime unavailable here). No finding in this folder is *verified binary evidence*: no Apple binary was available.

## Files
features.md · ui-ux.md · architecture.md · file-formats.md · processing.md · performance.md · dependencies.md · evidence.md · limitations.md · implementation-spec.md · tasks.md

## Primary sources
- [Xcode overview (Apple)](https://developer.apple.com/xcode/)
- [What's new in Xcode (Apple)](https://developer.apple.com/xcode/whats-new/)
- [Xcode Releases (unofficial index)](https://xcodereleases.com)
- [What is Xcode (Software Mind blog, 3P, June 2025, partly stale)](https://softwaremind.com/blog/what-is-xcode-features-and-use/)

## One-paragraph summary
Xcode (ide and toolchain for apple platforms (reference app, not part of creator studio)) was investigated from public documentation and third-party material only. The most reliable findings are feature-level (DOC). Internal architecture, algorithms, and exact file formats are mostly INF or UNK. Any claim here that looks like an internal detail is labelled with its evidence class.
