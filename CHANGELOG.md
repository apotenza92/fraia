# Changelog

All notable user-facing changes to Fraia are documented here. Fraia follows
[Semantic Versioning](https://semver.org/spec/v2.0.0.html), and the release
workflow uses the matching version section below as the authoritative GitHub
release notes and macOS updater notes.

## [0.0.3] - 2026-07-29

### Added

- Native Windows ARM64 packaging backed by a reviewed ARM64 CalculiX runtime,
  alongside the existing macOS, Linux, and Windows x64 targets.
- Authenticated automatic updates for Windows and Linux using an embedded,
  reviewed TUF root and separately protected targets, snapshot, and timestamp
  signing roles.

### Changed

- The release matrix now requires six native solver-backed packages: macOS
  ARM64 and x64, Linux ARM64 and x64, and Windows ARM64 and x64.
- Stable and beta feed aliases resolve to the same stable application identity
  and package bytes rather than separate beta builds.

### Security

- Windows and Linux update metadata is TUF-signed and rejects corrupt,
  expired, or incorrectly signed targets. Their initial installers remain
  unsigned and must be accompanied by a clear disclosure, public checksums,
  and provenance.

### Updating

- Fraia packages check the public stable feed automatically each day by
  default. macOS retains Developer ID and notarisation verification; Windows
  and Linux require the embedded TUF trust root. Users can change the
  frequency from the Fraia menu.
- Windows NSIS and Linux AppImage installations can update in place. Linux DEB
  and RPM installations remain under their system package manager.

## [0.0.2] - 2026-07-28

### Added

- Initial public Fraia desktop release for macOS (Apple silicon and Intel),
  Linux (ARM64 and x64), and Windows (x64).
- Rust-backed structural modelling workspace with native project state,
  validation, analysis runs, and the Three.js engineering viewport.
- Reviewed native CalculiX runtimes for every supported package target, with
  pinned source and toolchain provenance, bundled notices, checksums, and
  official solver-case evidence.
- Fraia AI sign-in with ChatGPT through the local Pi runtime, without shipping
  test credentials or exposing provider secrets to the renderer.
- Native stable, beta, light, and dark application icon variants for macOS,
  Windows, and Linux.

### Changed

- Fraia AI is fixed to the reviewed ChatGPT and GPT-5.6 Luna contract with low
  reasoning, and fails closed instead of silently switching providers or models.
- Application chrome and controls use the configured shadcn Base UI component
  system and Rhea styling.

### Security

- Release packages exclude local AI credentials and login tokens. Provider
  authentication remains in each user's platform-protected application data.
- macOS packages and updates require Developer ID signing, hardened runtime,
  notarisation, stapling, and public checksum and provenance verification.

### Updating

- Signed and notarised macOS packages check the public stable feed
  automatically each day by default. Users can change the frequency from the
  Fraia menu.
- A downloaded update presents these release notes and lets the user restart
  immediately or install safely when Fraia next quits.
- Windows and Linux automatic updating remains disabled until those platforms
  have an equivalent trusted signing and installation boundary.
