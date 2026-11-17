# Xcode — file formats, import and export

| Format | Description | Evidence |
|---|---|---|
| .xcodeproj / .xcworkspace | project bundles (a Flutter project of the user's contains examples; not analysed) | UNK internals |
| .xip | signed archive Apple distributes Xcode in (per xcodereleases.com downloads) | DOC |
| String Catalogs, asset catalogs, .xctest bundles | resource/test formats | DOC names only |

## Import/export capabilities
Feature-level only; see features.md. Interop: Swift Package Manager, Git (3P), TestFlight/App Store, Xcode Cloud.

## Rules for our implementation
- Define our own open, documented, versioned format; do not claim Apple-format fidelity beyond what public documentation or lawful analysis supports.
- Importers of Apple formats (where lawful and useful) must be separate, optional modules with their own license review.
- Never bundle protobuf schemas or data extracted from Apple binaries without legal review (they may be copyrighted).
