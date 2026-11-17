# Final Cut Pro — research dossier

**Domain:** Non-linear video editing
**Priority in suite:** #8 of 10 (see ../development/master-roadmap.md)
**Binary analysis status:** BLOCKED — no binary available. Research is documentation-level only.
**Version studied:** unknown (no binary; Apple guides are version-agnostic web pages)

Evidence classes: **DOC** documented behavior (Apple guides/developer docs, via web search), **3P** third-party reverse-engineering/community material (search snippets only, not opened or verified), **INF** inference, **UNK** unknown, **BLK** blocked (needs binary/runtime unavailable here). No finding in this folder is *verified binary evidence*: no Apple binary was available.

## Files
features.md · ui-ux.md · architecture.md · file-formats.md · processing.md · performance.md · dependencies.md · evidence.md · limitations.md · implementation-spec.md · tasks.md

## Primary sources
- [Final Cut Pro User Guide](https://support.apple.com/guide/final-cut-pro/verfdd5c590e/mac)
- [Media Management (Apple PDF)](https://www.apple.com/kr/final-cut-pro/docs/Media_Management.pdf)
- [FCPXML reference (Apple developer)](https://developer.apple.com/documentation/professional-video-applications/fcpxml-reference.md)
- [FCPXML DTD page](https://developer.apple.com/documentation/professional-video-applications/document-type-definition.md)
- [fcp.cafe FCPXML dev notes](https://fcp.cafe/developers/fcpxml/)

## One-paragraph summary
Final Cut Pro (non-linear video editing) was investigated from public documentation and third-party material only. The most reliable findings are feature-level (DOC). Internal architecture, algorithms, and exact file formats are mostly INF or UNK. Any claim here that looks like an internal detail is labelled with its evidence class.
