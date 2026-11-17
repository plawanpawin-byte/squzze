# Keynote — research dossier

**Domain:** Presentation software
**Priority in suite:** #3 of 10 (see ../development/master-roadmap.md)
**Binary analysis status:** BLOCKED — no binary available. Research is documentation-level only.
**Version studied:** unknown (no binary; Apple guides are version-agnostic web pages)

Evidence classes: **DOC** documented behavior (Apple guides/developer docs, via web search), **3P** third-party reverse-engineering/community material (search snippets only, not opened or verified), **INF** inference, **UNK** unknown, **BLK** blocked (needs binary/runtime unavailable here). No finding in this folder is *verified binary evidence*: no Apple binary was available.

## Files
features.md · ui-ux.md · architecture.md · file-formats.md · processing.md · performance.md · dependencies.md · evidence.md · limitations.md · implementation-spec.md · tasks.md

## Primary sources
- [Reverse Engineering iWork](https://andrews.substack.com/p/reverse-engineering-iwork)
- [SheetJS IWA notes](https://git.sheetjs.com/sheetjs/notes/src/commit/630ce0f1168ac1fd675fab6cfdf170d3f76eb43b/iwa)
- [SVG2Keynote-lib (3P)](https://github.com/eth-siplab/SVG2Keynote-lib/)
- [Stingray reader protobuf notes](https://stingrayreader.sourceforge.net/protobuf.html)

## One-paragraph summary
Keynote (presentation software) was investigated from public documentation and third-party material only. The most reliable findings are feature-level (DOC). Internal architecture, algorithms, and exact file formats are mostly INF or UNK. Any claim here that looks like an internal detail is labelled with its evidence class.
