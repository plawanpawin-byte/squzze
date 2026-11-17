# Pixelmator Pro — file formats, import and export

| Format | Description | Evidence |
|---|---|---|
| PXD | Pixelmator Pro only; package/format internals UNK | DOC (existence) / 3P |
| PXM (Classic) | different, legacy | DOC |
| PSD, PNG, JPEG, TIFF, HEIC etc. | import/export | DOC (general) |

## Import/export capabilities
Feature-level only; see features.md. Interop: PSD import/export (DOC general); Photos/Preview integrations UNK.

## Rules for our implementation
- Define our own open, documented, versioned format; do not claim Apple-format fidelity beyond what public documentation or lawful analysis supports.
- Importers of Apple formats (where lawful and useful) must be separate, optional modules with their own license review.
- Never bundle protobuf schemas or data extracted from Apple binaries without legal review (they may be copyrighted).
