# Motion — feature inventory

Source: Apple user guide and related pages (feature level). Not an exhaustive list: the full guide was not crawled; see `limitations.md`.

- 2D/3D layers, groups, cameras, lights
- Keyframes and curve editor
- Behaviors (simulation, parameter, text)
- Particle emitters and replicators
- Filters, masks, blend modes
- Text animation
- Templates for Final Cut Pro (titles, effects, transitions, generators)
- Render/export

## Deep-topic coverage
| Topic | Summary | Evidence |
|---|---|---|
| Animation engine | Time-evaluated parameters. | INF |
| Keyframes | Curves with interpolation. | DOC feature level |
| Behaviors | Procedural modifiers. | DOC |
| Particles | Emitter plus cell life cycle. | DOC |
| Effects/compositing | Filters, blend. | DOC feature level |
| Rendering | Frame sequence/movie. | DOC feature level |

## Independent implementation notes
For every feature above the suite's own app should expose equivalent *observable* behaviour (inputs, outputs, interactions) without copying Apple implementation. See `implementation-spec.md`.
