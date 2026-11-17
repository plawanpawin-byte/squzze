# Release plan

## Channels
- `nightly` (CI artifacts), `beta` (tagged pre-release), `stable`.
- Versioning: SemVer per app; shared crates versioned in workspace.

## Packaging
- Windows: MSI/NSIS (Tauri bundler), portable zip. Code signing certificate is a cost; unsigned builds trigger SmartScreen — document this. Free option: SignPath Foundation for OSS (check eligibility).
- Linux: AppImage, .deb, Flatpak.
- macOS: .dmg; notarization needs a paid Apple Developer account — the only unavoidable paid item and it is optional (users can build from source).

## Per-app release gates (every app)
1. All acceptance tests pass in CI on Windows, Linux, macOS.
2. No network access required (verified by running with network disabled).
3. License register current; SBOM produced.
4. Accessibility smoke test (keyboard-only workflow).
5. Documentation: user guide, file format spec, CHANGELOG.
6. Crash-free 24 h soak on reference machine.

## Compressor-alt releases
| Version | Content |
|---|---|
| 0.1 | CLI only: probe, encode with built-in presets, JSON progress |
| 0.2 | Queue persistence + GUI batch window |
| 0.3 | Hardware encoder detection, custom presets, watch folders |
| 1.0 | All MVP release criteria in compressor/implementation-spec.md met |
