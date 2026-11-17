# MainStage — feature inventory

Source: Apple user guide and related pages (feature level). Not an exhaustive list: the full guide was not crawled; see `limitations.md`.

- Concerts, sets, patches
- Software instrument and effect channel strips
- Audio Units hosting
- MIDI controller mapping to screen controls/Smart Controls
- Audio interface selection and routing
- Setlist and patch switching with seamless tails (INF)
- Playback/backing tracks

## Deep-topic coverage
| Topic | Summary | Evidence |
|---|---|---|
| Live audio processing | Low-buffer real-time graph. | DOC |
| MIDI controllers | Mappings. | DOC feature level |
| Instruments/effects | Hosted plug-ins. | DOC |
| Patch management | Switching. | DOC feature level |
| Routing | Buses, aux, I/O. | DOC |
| Low-latency performance | Buffer tuning. | DOC |

## Independent implementation notes
For every feature above the suite's own app should expose equivalent *observable* behaviour (inputs, outputs, interactions) without copying Apple implementation. See `implementation-spec.md`.
