# Motion — performance characteristics

- Measured data: **none** (BLK.).
- Memory: UNK.
- Acceleration: UNK; marketing says GPU-accelerated (not verified).

## Targets for our implementation (design goals, not Apple measurements)
- Cold start under 3 s on the reference machine (Ryzen 7 7445HS, 15 GB RAM, RTX 4050).
- UI never blocks on background work; long operations are cancellable.
- Provide a benchmark harness before claiming parity with any Apple behaviour.
