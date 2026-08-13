# Changelog

All notable user-facing changes to Fraia are documented here. Fraia follows
[Semantic Versioning](https://semver.org/spec/v2.0.0.html), and the release
workflow uses the matching version section below as the authoritative GitHub
release notes and macOS updater notes.

## [0.0.14] - 2026-08-13

### Fixed

- Wait for the development sidecar health signal before the real Electron conversation test creates a model.
- Allow slow source builds more startup time without weakening packaged-app startup checks.

## [0.0.13] - 2026-08-13

### Changed

- Fraia now opens into one conversation-first structural design workspace
  instead of the previous staged Base Model, Design Options, and Analysis
  interface.
- Blank projects can move from a typed brief to proposed geometry, accepted
  revisions, analysis evidence, alternatives, inspection, and manual edits in
  one continuous workflow.
- The simplified desktop shell removes obsolete staged panels and keeps the
  Three.js structural preview, selection, symbols, and precision editor focused
  on the current design conversation.

### Added

- A Rust revision engine now stores canonical model snapshots, immutable
  revisions, typed patches and semantic diffs, conversation branches, working
  copies, analysis evidence, and SQLite-backed project history.
- The Electron and application-service bridge now supports durable project
  conversations, proposal acceptance and rejection, fork and resume, exact
  snapshot-bound analysis, comparisons, and restart recovery.

### Fixed

- Release tests now follow the conversation-first desktop acceptance path after
  removal of the legacy Base UI migration scenario.
- The production dependency tree now pins the patched `js-yaml` release and
  reports no production audit vulnerabilities.

### Updating

- Stable and Beta installations update to separately packaged `0.0.13`
  application identities on their existing channels.

## [0.0.12] - 2026-08-13

### Fixed

- macOS now renders the stable and Beta adaptive icons at a balanced native
  size instead of shrinking the Fraia glyph inside the system icon tile.

### Updating

- Stable and Beta installations update to separately packaged `0.0.12`
  application identities on their existing channels.

## [0.0.11] - 2026-08-12

### Changed

- Fraia now uses a compact anchored-vault symbol that remains clear across
  macOS, Windows, Linux, GitHub, and the public download page.
- Stable uses the Carbon and Chalk mark. Beta uses the Oxide and Oxide Light
  mark from the same warm Fraia brand palette.
- The public product description and download page now use the new Fraia brand
  system while leaving the application interface theme unchanged.

### Updating

- Stable and Beta installations update to separately packaged `0.0.11`
  application identities on their existing channels.

## [0.0.10] - 2026-08-06

### Changed

- Viewport labels now retain their semantic colours, resolve collisions more
  reliably, expand independently, and keep proposed-support labels anchored to
  their passive green support markers. Compact member labels show `SW` on its
  own line, while expanded labels show `Self weight` without separate arrows.
- Selection now prioritises nodes and members over overlapping load graphics,
  uses restrained amber geometry-only highlights, clears predictably on an
  empty click, and supports containment, crossing, and lasso selection without
  making proposed supports selectable.
- The canvas has a persistent Controls bar with accurate mouse diagrams,
  right- and left-handed help, authentic engineering-app navigation profiles,
  and a separate persistent Custom mouse profile. Fraia continues to default
  to SPACE GASS navigation.
- New files open in an isometric view and remember their camera independently
  after the user moves it.
- New assistant replies remain positioned at the top of the latest response,
  and the Base Model handoff into Design Options is clearer and more compact.
- ChatGPT sign-in now uses browser authorization directly without asking users
  to paste a redirect URL.
- Development launches now use one identifiable, single-owner Fraia Dev
  process with isolated user data and reliable stale-launch recovery.
- The update dialog now presents release readiness and release notes more
  directly with less redundant chrome.

### Updating

- Stable and Beta installations update to separately packaged `0.0.10`
  application identities on their existing channels.

## [0.0.9] - 2026-08-05

### Changed

- Base Model and Design Options conversations now share a cleaner transcript
  with consistent assistant, user, streaming, and progress presentation.
- Fraia now follows the system appearance and clears stale manual theme
  overrides.
- The Fraia menu now exposes the automatic update schedule directly and keeps
  menu items wide enough for their labels.
- Workflow stages are navigated directly from their labels, with unavailable
  stages explaining what is needed to continue.
- Error and destructive-alert text now meets WCAG AA contrast in the light
  appearance, with the release desktop test pinned to that appearance.
- The public download page now provides platform-aware Stable and Beta
  downloads, checksums, provenance, and unsigned-package disclosures.
- Stable and Beta now use a clearer low-poly Ionic column icon with a shorter,
  more legible silhouette and distinct solid channel backgrounds.
- macOS uses matching light and dark artwork, while Windows and Linux retain
  the reviewed light variants at every generated application-icon size.

### Updating

- Stable and Beta installations update to separately packaged `0.0.9`
  application identities on their existing channels.

## [0.0.8] - 2026-08-04

### Release status

- The release completed native packaging and verification but remained behind
  its final approval gate. No release assets or updater feeds were published;
  `0.0.9` supersedes this tag.

## [0.0.7] - 2026-08-04

### Release status

- The release gate stopped before native packaging because light-appearance
  destructive-alert text did not meet WCAG AA contrast. No release assets or
  updater feeds were published; `0.0.8` supersedes this tag.

## [0.0.6] - 2026-08-04

### Changed

- The updater-feed publication stage now creates its sealed staging directory
  before writing and checksumming the feed projection. A deterministic release
  workflow test protects that ordering.

### Updating

- Stable and Beta installations on the `0.0.4` feeds update directly to their
  separately packaged `0.0.6` identities. The public `0.0.5` GitHub release
  remains available for direct download, but its updater-feed projection did
  not publish after the release assets were made public.

## [0.0.5] - 2026-08-04

### Changed

- The desktop shell now uses the Base UI Nova component system and Geist type,
  with document tabs, model tools, menu controls, and panel density aligned to
  the Butter Paper desktop family.
- First launch now uses a 1200 by 800 window with a 900 by 600 minimum. Fraia
  continues to restore a user's later window size, position, and maximized
  state.
- The Base Model panel now signs in to ChatGPT directly from its required-sign-in
  button and changes that control to Sign out after authentication, allowing
  users to switch accounts without opening a separate provider window.
- Member, snap, and label tools now use compact Nova split controls with their
  settings attached directly to the relevant action.
- The Fraia menu now exposes Check for Updates directly and groups persisted
  automatic-check frequencies under Automatic Checks. The developer menu and
  separate Fraia AI menu item have been removed from release builds.
- The Fraia menu now reports checking, up-to-date, downloading, ready,
  installing, and retryable error states instead of leaving manual checks
  silent. Downloads show percentage, transferred and total size, speed, and
  estimated time remaining.
- Automatic update settings now record the last successful check separately
  from failed attempts and retry failed background checks with bounded
  backoff.
- Beta is now a stable-inclusive channel: beta installations receive the
  highest Semantic Version across beta and final releases without changing
  into the stable application.
- A final release that advances Beta now builds and verifies separate `Fraia`
  and `Fraia Beta` packages from the same reviewed source commit. Each retains
  its own application identity, icons, user-data directory, and update feed.
- Release policy rejects downgrades and requires both stable and beta approval
  before a final version can advance both feeds.

### Updating

- The embedded updater remains active when Fraia is installed through a future
  Homebrew cask. Homebrew and Fraia are independent same-channel update paths;
  either may replace the application while preserving its identity, settings,
  projects, and AI authorization.
- Stable `0.0.5` installations use only the stable feed, while separately
  installed `Fraia Beta` packages use only the beta feed. The `0.0.5` release
  assets were published before a workflow error prevented either feed from
  advancing; `0.0.6` supersedes this release for automatic updating.

## [0.0.4] - 2026-07-30

### Changed

- Stable and beta are now separate, side-by-side applications with isolated
  application ids, package names, user-data directories, artifacts, update
  feeds, and release tags.
- CalculiX corresponding source now uses GNU's canonical GCC archive endpoint
  while retaining the reviewed GCC 16.1.0 SHA-256.

### Updating

- Stable `0.0.4` updates only the `Fraia` application and stable feed. The
  separately installed `Fraia Beta` application remains on its beta-only feed.

## [0.0.4-beta.1] - 2026-07-30

### Changed

- Stable and beta are now separate, side-by-side applications with isolated
  application ids, package names, user-data directories, artifacts, update
  feeds, and release tags. Stable uses `vX.Y.Z`; beta uses
  `vX.Y.Z-beta.N` and is published as a GitHub prerelease.
- CalculiX corresponding source now uses GNU's canonical GCC archive endpoint
  while retaining the reviewed GCC 16.1.0 SHA-256.

### Updating

- Each identity accepts updates only from its own channel. Stable releases no
  longer overwrite the beta feed, and beta releases cannot replace stable
  installations or their settings and projects.

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
