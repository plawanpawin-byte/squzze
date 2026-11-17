# Freeform — feature inventory

Source: Apple user guide and related pages (feature level). Not an exhaustive list: the full guide was not crawled; see `limitations.md`.

- Infinite canvas
- Pencil/pen drawing
- Shapes, sticky notes, text boxes
- Connectors/lines
- Embed images, video, audio, PDF, links, documents
- Real-time collaboration

## Deep-topic coverage
| Topic | Summary | Evidence |
|---|---|---|
| Infinite canvas | Pan/zoom over unbounded plane. | INF |
| Drawing | Ink strokes. | DOC feature level |
| Object manipulation | Select/transform/group. | DOC feature level |
| Connectors | Anchored lines that follow shapes. | DOC feature level |
| Embedded assets | Files, media. | DOC |
| Local persistence | App container storage. | 3P weak / UNK |

## Independent implementation notes
For every feature above the suite's own app should expose equivalent *observable* behaviour (inputs, outputs, interactions) without copying Apple implementation. See `implementation-spec.md`.
