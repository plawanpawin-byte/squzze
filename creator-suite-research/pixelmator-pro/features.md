# Pixelmator Pro — feature inventory

Source: Apple user guide and related pages (feature level). Not an exhaustive list: the full guide was not crawled; see `limitations.md`.

- Layers (raster, vector shapes, text, groups)
- Selections and masking
- Non-destructive adjustments and effects
- Transformations and warps
- Brushes and retouching
- ML-based enhancements (Apple/Creator Studio marketing)
- Export to common raster formats and PXD
- iPad version

## Deep-topic coverage
| Topic | Summary | Evidence |
|---|---|---|
| Layers | Typed layers with blend modes. | DOC feature level |
| Raster processing | Per-pixel filters. | INF |
| Selections/masking | Selection to mask. | DOC feature level |
| Compositing | Blend and group. | INF |
| Transformations | Affine/warp. | DOC feature level |
| Brushes | Brush engine. | UNK |
| Export | Formats list. | DOC |

## Independent implementation notes
For every feature above the suite's own app should expose equivalent *observable* behaviour (inputs, outputs, interactions) without copying Apple implementation. See `implementation-spec.md`.
