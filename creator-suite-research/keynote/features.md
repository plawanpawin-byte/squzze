# Keynote — feature inventory

Source: Apple user guide and related pages (feature level). Not an exhaustive list: the full guide was not crawled; see `limitations.md`.

- Slides, masters, themes
- Text, shapes, tables, charts, media
- Build in/out animations, Magic Move transitions
- Presenter notes, presenter display, rehearse
- Export to PDF/PowerPoint/images/video
- Collaboration via iCloud

## Deep-topic coverage
| Topic | Summary | Evidence |
|---|---|---|
| Slide structure | Slide nodes reference masters and drawables. | 3P/INF |
| Layout engine | Text and shape layout. | UNK |
| Animations/transitions | Builds, Magic Move. | DOC feature level |
| Presenter mode | Notes, next-slide preview. | DOC feature level |
| Rendering | Slide to frames/PDF. | INF |

## Independent implementation notes
For every feature above the suite's own app should expose equivalent *observable* behaviour (inputs, outputs, interactions) without copying Apple implementation. See `implementation-spec.md`.
