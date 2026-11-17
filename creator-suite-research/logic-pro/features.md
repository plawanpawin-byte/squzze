# Logic Pro — feature inventory

Source: Apple user guide and related pages (feature level). Not an exhaustive list: the full guide was not crawled; see `limitations.md`.

- Audio and MIDI tracks, software instruments, drummer
- Recording, comping, Flex Time/Pitch
- Mixer with sends, buses, groups, plug-in inserts
- Automation (track and region)
- Built-in effects and instruments, Audio Units hosting
- Score and notation editing
- Freeze files, bounces
- Synth Player and Chord ID (per Creator Studio announcement)

## Deep-topic coverage
| Topic | Summary | Evidence |
|---|---|---|
| Audio engine | Block-based real-time graph. | INF |
| Tracks and recording | Takes, comping, punch. | DOC (feature level) |
| MIDI | MIDI regions/events, quantise, scripting. | DOC (feature level) |
| Mixing & effects | Channel strips, buses, sends. | DOC |
| Plug-ins | Audio Units; Apple built-ins. | DOC (general) |
| Latency | Buffer size trades latency vs CPU; freeze reduces CPU. | DOC (MainStage guide analogue) / DOC (freeze) |
| Automation | Parameter automation lanes. | DOC (feature level) |

## Independent implementation notes
For every feature above the suite's own app should expose equivalent *observable* behaviour (inputs, outputs, interactions) without copying Apple implementation. See `implementation-spec.md`.
