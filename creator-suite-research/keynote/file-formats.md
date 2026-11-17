# Keynote — file formats, import and export

| Format | Description | Evidence |
|---|---|---|
| .key | ZIP bundle with IWA (Snappy+protobuf) | 3P |
| PDF/PPTX/images/movie export | export | DOC (general) |

## Import/export capabilities
Feature-level only; see features.md. Interop: PPTX import/export (DOC general); shares IWA format family with Pages and Numbers (3P).

## Rules for our implementation
- Define our own open, documented, versioned format; do not claim Apple-format fidelity beyond what public documentation or lawful analysis supports.
- Importers of Apple formats (where lawful and useful) must be separate, optional modules with their own license review.
- Never bundle protobuf schemas or data extracted from Apple binaries without legal review (they may be copyrighted).
