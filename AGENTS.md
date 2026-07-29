# AGENTS.md

Durable project instructions for coding and documentation agents working in Fraia.

## Current application

- Fraia is Rust-first engineering software with an Electron + React product shell.
- Launch or test the desktop application from `apps/fraia-electron` with `npm start`. Do not substitute a browser preview unless the user explicitly requests one.
- Rust owns project state, structural models, validation, analysis, runs, and the application API. Electron owns app chrome and the Three.js viewport.
- Preserve the primitive-first substrate: `Node`, `Member`, `Plate`, `SupportAssignment`, `LoadAssignment`, and `ReleaseAssignment`.
- Builders and archetypes sit above primitives; they do not replace authored engineering truth.
- Keep authored state, resolved/realisation state, and immutable run artefacts distinct.
- Outputs such as spreadsheets, reports, CAD data, and details are downstream renderers, not sources of truth.
- Keep system-specific engineering knowledge in the compiled knowledge wiki, authored metadata, builders, or explicit adapters rather than generic runtime heuristics.

## Structural language

- Use `role` for authored structural semantics such as `beam`, `column`, `rafter`, `brace`, `purlin`, `tie`, `slab`, `wall_panel`, and `roof_panel`.
- Derive display labels from role plus id, for example `Beam B1`.
- Reserve `element` for finite-element or discretisation objects. A semantic member split for analysis remains one role-labelled member discretised into multiple analysis elements.

## UI and renderer

- Use the official shadcn Base UI components and Rhea style configured by `apps/fraia-electron/components.json` for app chrome and standard controls. Compose domain UI from the existing `src/components/ui` primitives instead of recreating buttons, inputs, checkboxes, dialogs, menus, tabs, or tooltips. Follow Base UI's `render` composition API; do not introduce Radix dependencies, `asChild`, Radix CSS variables, or Radix state selectors.
- Use Lucide icons and existing design tokens.
- Draw model-attached members, nodes, supports, loads, releases, labels, axes, origins, highlights, and handles through Three.js/WebGL, not DOM overlays.
- Keep DOM usage for panels, menus, toolbars, inspectors, dialogs, and other app chrome.

## Architecture and provenance

- Preserve stage boundaries: builder graph, structural model, realisation, analysis results, design actions, check inputs/results, and exports.
- Prefer downstream adapters over changes to the canonical upstream model when possible.
- Derived objects should retain useful provenance to builder nodes, authored objects, realisation objects, governing results, and run ids.
- Major transformations should have typed representations and persisted run artefacts where reproducibility requires them.

## Documentation and repository hygiene

- Read `docs/documentation-map.md` before adding or reorganising documentation.
- Update the canonical document for a topic instead of creating a duplicate. Use cross-references rather than repeating philosophy.
- Do not store changing work state in `MEMORY`, `PLAN`, `NOW`, `WORKLOG`, `BACKLOG`, `ROADMAP`, handoff, progress, context, or subagent files. Use GitHub issues and pull requests; keep disposable working artefacts in ignored temporary directories.
- Do not commit generated Electron bundles, generated documentation viewers, benchmark captures, screenshots, videos, logs, or machine-specific paths.
- Preserve legitimate history in changelogs and Git history. Do not retain obsolete implementations or plans solely as historical documentation.
- Never overwrite unrelated user changes in a dirty worktree.

## Required verification

Run checks appropriate to the changed area and report anything not run:

```sh
python3 scripts/check-repository-hygiene.py
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features
cargo test --workspace --all-features
python3 scripts/lint-knowledge.py
python3 scripts/validate-knowledge-next.py
cd apps/fraia-electron && npm run typecheck && npm run build
```

The `knowledge-next` check applies while that directory is present. Electron runtime checks must launch the desktop app rather than a browser page.

## Desktop release contract

- `.github/workflows/ci.yml` is manual-only. Its deterministic checks always run when dispatched; its six-target native package preflight is an explicit opt-in that consumes already reviewed runtime payloads and never constructs or downloads CalculiX.
- `.github/workflows/release.yml` is the only desktop release workflow. It is tag-only and packages exactly six reviewed native targets: macOS ARM64/x64, Linux ARM64/x64, and Windows ARM64/x64. Every target must use matching native Electron, bundled `fraia-appd`, and reviewed CalculiX executables and must pass its native packaged solver test.
- Fraia publishes one stable application identity and one stable package set per version. The stable release projects the same verified update metadata to both the `stable` and `beta` static feed paths; `beta` is a feed alias, not a separate build, tag, prerelease, bundle identity, package identity, or user-data directory. macOS updates require Developer ID signing and notarization. Windows and Linux updates must be authenticated by the reviewed embedded TUF root, protected online role keys, an offline root key, native N-1 replacement tests, and byte-identical stable/beta projections. Windows and Linux installers remain unsigned for the initial public release, so the download page must disclose that limitation and publish independently verifiable checksums and provenance.
- macOS release jobs use the canonical encrypted P12 plus App Store Connect P8 contract in the `release-signing` environment. Compile the renderer and Rust sidecar before credentials enter the step, use a temporary Keychain, and run the final signature/package launch verifier after credential cleanup.
- `stable-release` always requires final human approval and is the only publication environment.
- `MACOS_UPDATER_BOOTSTRAP_TAG` is a one-time exact tag in the `stable-updater-verification` environment. That environment contains no signing credentials and requires no publication reviewer. Use the variable only when no prior public Fraia package exists, remove it after the bootstrap release, and never advance it to bypass N-1 updater tests.
- Release automation byte-verifies existing draft assets and may upload missing assets. It must reject unexpected or differing assets and must never replace a published asset automatically.
- Release jobs must use maintained Fraia icons at `apps/fraia-electron/build/icon.icns`, `build/icon.ico`, and `build/icons/512x512.png`; never publish a package with Electron's default icon.
- Release jobs must package a reviewed native CalculiX executable plus `THIRD_PARTY_NOTICES.txt` from `apps/fraia-electron/runtimes/calculix/<platform>-<arch>/`. Never fetch an unpinned solver during a release or claim a target whose real packaged solver test has not passed.
- Release jobs must assemble the one deterministic `Fraia-CalculiX-Corresponding-Source.tar` asset from the SHA-pinned upstream sources and reviewed platform build recipes, and must byte-verify its public URL and digest against every runtime manifest before publication. Source assembly must never build or substitute a solver binary.
- Do not add Homebrew or store publication until a first accepted public Fraia artifact exists and there is a concrete maintained consumer.
- Fraia releases from its public source repository so installed clients can reach GitHub Releases, update feeds, and provenance without authentication. The maintained same-repository release workflow must fail before building if repository visibility is not public. Do not weaken this gate or introduce a cross-repository token implicitly.
