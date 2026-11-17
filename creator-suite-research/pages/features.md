# Pages — feature inventory

Source: Apple user guide and related pages (feature level). Not an exhaustive list: the full guide was not crawled; see `limitations.md`.

- Styles (paragraph, character, list)
- Tables, charts, shapes, images
- Page layout documents with free placement
- Footnotes, TOC, headers/footers
- Track changes, comments
- Export to PDF/Word/EPUB/plain text
- Print

## Deep-topic coverage
| Topic | Summary | Evidence |
|---|---|---|
| Document structure | Sections, paragraphs, runs. | INF |
| Text layout/pagination | Shaping, line breaking. | UNK |
| Styles | Named styles inheriting. | DOC feature level |
| Tables | Cell grid with formulas shared with Numbers (INF) | INF |
| Printing | System print. | DOC general |
| Compatibility | DOCX conversion. | DOC general |

## Independent implementation notes
For every feature above the suite's own app should expose equivalent *observable* behaviour (inputs, outputs, interactions) without copying Apple implementation. See `implementation-spec.md`.
