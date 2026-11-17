# Final Cut Pro — feature inventory

Source: Apple user guide and related pages (feature level). Not an exhaustive list: the full guide was not crawled; see `limitations.md`.

- Magnetic Timeline editing (primary storyline, connected clips, auditions, compound clips)
- Media import, organisation by events, keywords, ratings, smart collections
- Trim tools (ripple, roll, slip, slide)
- Effects, transitions, titles, generators
- Colour correction and grading (colour board, wheels, curves, LUTs)
- Proxy and optimized media workflow
- Background rendering
- Share/export (including via Compressor)
- Multicam, Roles, Captions
- Transcript-based search and beat detection (per Creator Studio announcement)

## Deep-topic coverage
| Topic | Summary | Evidence |
|---|---|---|
| Magnetic Timeline | Timeline model where clips are positioned relative to a primary storyline and connected clips attach to anchors so gaps close automatically. | INF: model as a tree (spine + attached children with offset). FCPXML's spine/asset-clip/ref structure is DOC evidence for this shape. |
| Media management | Libraries, events, external media, generated media location. | DOC |
| Editing and trimming | Standard NLE edit types. | DOC (guide) ; algorithms INF |
| Effects and colour grading | Effects stack per clip; colour board. | DOC (feature level) |
| Rendering and proxy | Render, optimized, proxy files are separable cache classes. | DOC |
| Export | Share destinations; encode via system encoders or Compressor. | DOC (feature level) |

## Independent implementation notes
For every feature above the suite's own app should expose equivalent *observable* behaviour (inputs, outputs, interactions) without copying Apple implementation. See `implementation-spec.md`.
