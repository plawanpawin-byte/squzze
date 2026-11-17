# Pages — research dossier

**Domain:** Word processing and page layout
**Priority in suite:** #4 of 10 (see ../development/master-roadmap.md)
**Binary analysis status:** BLOCKED — no binary available. Research is documentation-level only.
**Version studied:** unknown (no binary; Apple guides are version-agnostic web pages)

Evidence classes: **DOC** documented behavior (Apple guides/developer docs, via web search), **3P** third-party reverse-engineering/community material (search snippets only, not opened or verified), **INF** inference, **UNK** unknown, **BLK** blocked (needs binary/runtime unavailable here). No finding in this folder is *verified binary evidence*: no Apple binary was available.

## Files
features.md · ui-ux.md · architecture.md · file-formats.md · processing.md · performance.md · dependencies.md · evidence.md · limitations.md · implementation-spec.md · tasks.md

## Primary sources
- [Reverse Engineering iWork](https://andrews.substack.com/p/reverse-engineering-iwork)
- [litchi IWA module (3P)](https://docs.rs/crate/litchi/latest/source/src/iwa/mod.rs)
- [kreuzberg iWork extractor (3P)](https://docs.rs/crate/kreuzberg/4.9.8/source/src/extractors/iwork/mod.rs)

## One-paragraph summary
Pages (word processing and page layout) was investigated from public documentation and third-party material only. The most reliable findings are feature-level (DOC). Internal architecture, algorithms, and exact file formats are mostly INF or UNK. Any claim here that looks like an internal detail is labelled with its evidence class.
