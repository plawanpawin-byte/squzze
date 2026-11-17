# Final Cut Pro — file formats, import and export

| Format | Description | Evidence |
|---|---|---|
| Library bundle (.fcpbundle) | package | DOC (concept) / UNK (internals) |
| FCPXML (.fcpxml / .fcpxmld) | XML interchange, DTD published by Apple; versions through at least 1.10 documented, newer exist | DOC |
| Media: QuickTime/MP4/ProRes/H.264/HEVC etc. | import/export | DOC (general) |

## Import/export capabilities
Feature-level only; see features.md. Interop: FCPXML to/from third-party tools (DOC); sends to Compressor/Motion (DOC general).

## Rules for our implementation
- Define our own open, documented, versioned format; do not claim Apple-format fidelity beyond what public documentation or lawful analysis supports.
- Importers of Apple formats (where lawful and useful) must be separate, optional modules with their own license review.
- Never bundle protobuf schemas or data extracted from Apple binaries without legal review (they may be copyrighted).
