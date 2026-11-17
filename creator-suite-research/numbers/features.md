# Numbers — feature inventory

Source: Apple user guide and related pages (feature level). Not an exhaustive list: the full guide was not crawled; see `limitations.md`.

- Multiple tables per sheet
- Formulas and a function library (size/list not harvested)
- Charts
- Conditional formatting, sorting, filtering
- Import/export CSV, Excel, PDF
- Forms (iCloud)

## Deep-topic coverage
| Topic | Summary | Evidence |
|---|---|---|
| Architecture | Table-centric model. | 3P/INF |
| Formula engine | Cell formulas and function library. | UNK internals; 3P for formula storage |
| Recalculation | Dirty-cell propagation. | INF |
| Charts | Chart objects bound to table ranges. | DOC feature level |
| Sort/filter | Per-table. | DOC feature level |

## Independent implementation notes
For every feature above the suite's own app should expose equivalent *observable* behaviour (inputs, outputs, interactions) without copying Apple implementation. See `implementation-spec.md`.
