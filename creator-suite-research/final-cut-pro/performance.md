# Final Cut Pro — performance characteristics

- Measured data: **none** (BLK: no measurements. Documented design intent: proxy/optimized media for heavy codecs.).
- Memory: UNK (BLK). Documented only: large media is referenced, not copied, unless the user chooses.
- Acceleration: UNK (BLK). Feature docs mention GPU/hardware encoding; no verified detail.

## Targets for our implementation (design goals, not Apple measurements)
- Cold start under 3 s on the reference machine (Ryzen 7 7445HS, 15 GB RAM, RTX 4050).
- UI never blocks on background work; long operations are cancellable.
- Provide a benchmark harness before claiming parity with any Apple behaviour.
