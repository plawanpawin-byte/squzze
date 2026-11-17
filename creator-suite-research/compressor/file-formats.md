# Compressor — file formats, import and export

| Format | Description | Evidence |
|---|---|---|
| Settings/preset files | UNK format | UNK |
| Output: ProRes, H.264, HEVC, MPEG-2, MXF, audio | varies | DOC general |

## Import/export capabilities
Feature-level only; see features.md. Interop: Final Cut Pro, Motion share/export to Compressor; Apple Devices.

## Rules for our implementation
- Define our own open, documented, versioned format; do not claim Apple-format fidelity beyond what public documentation or lawful analysis supports.
- Importers of Apple formats (where lawful and useful) must be separate, optional modules with their own license review.
- Never bundle protobuf schemas or data extracted from Apple binaries without legal review (they may be copyrighted).
