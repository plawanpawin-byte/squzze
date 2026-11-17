# Final Cut Pro — dependencies

## Apple-side framework dependencies (probable, unverified)
- AVFoundation / Core Media / VideoToolbox / Metal — INF

## Our open-source dependency candidates
See ../shared-architecture/shared-engines.md and ../development/licensing-register.md for versions, licenses and obligations. App-specific notes: Model timeline as spine + attached children with explicit offsets; store in SQLite + JSON; render via MLT or custom wgpu compositor; import/export FCPXML as a documented interchange format.
