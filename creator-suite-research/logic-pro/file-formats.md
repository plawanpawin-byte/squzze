# Logic Pro — file formats, import and export

| Format | Description | Evidence |
|---|---|---|
| Project package (.logicx) | package | DOC (concept) / 3P (internals) |
| ProjectData | undocumented binary blob | 3P |
| Bounces | WAV/AIFF/MP3/M4A etc. | DOC (general) |
| MIDI (.mid) | standard | DOC (general) |

## Import/export capabilities
Feature-level only; see features.md. Interop: MIDI, audio files, MainStage shared instruments (INF), Logic project export formats UNK.

## Rules for our implementation
- Define our own open, documented, versioned format; do not claim Apple-format fidelity beyond what public documentation or lawful analysis supports.
- Importers of Apple formats (where lawful and useful) must be separate, optional modules with their own license review.
- Never bundle protobuf schemas or data extracted from Apple binaries without legal review (they may be copyrighted).
