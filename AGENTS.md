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
- Fraia publishes two side-by-side application identities. Stable tags use `vX.Y.Z`, the `Fraia` product and package identity, the `Fraia` user-data directory, and the `stable` feed. Beta tags use `vX.Y.Z-beta.N`, a GitHub prerelease, the `Fraia Beta` product and package identity, the isolated `Fraia Beta` user-data directory, and the `beta` feed. Stable users receive stable packages only. Beta users receive the highest Semantic Version available across beta and final releases: when a final stable version is newer than the current beta feed, the stable tag must also build, verify, and publish that version as a separate `Fraia Beta` identity package to the beta feed. It must never point beta metadata at a `Fraia` stable-identity package or replace a higher beta version with a lower stable version. Beta tags remain Beta-only. macOS updates require Developer ID signing and notarization. Windows and Linux updates must be authenticated by the reviewed embedded TUF root, protected online role keys, an offline root key, and native same-channel N-1 replacement tests. Windows and Linux installers remain unsigned for the initial public release, so the download page must disclose that limitation and publish independently verifiable checksums and provenance.
- macOS release jobs use the canonical encrypted P12 plus App Store Connect P8 contract in the `release-signing` environment. Compile the renderer and Rust sidecar before credentials enter the step, use a temporary Keychain, and run the final signature/package launch verifier after credential cleanup.
- `stable-release` and `beta-release` always require final human approval. A stable tag that advances both feeds requires both approvals after both identity-specific package sets pass. A stable release is never marked prerelease; a beta release is always marked prerelease and must not become GitHub's latest stable release.
- Both public channels are past bootstrap. `stable-updater-verification` and `beta-updater-verification` contain no signing credentials or reusable bootstrap override. Every later release must resolve the exact highest-SemVer package already advertised to that identity and pass native N-1 replacement; never reintroduce or advance a bootstrap variable to bypass that proof.
- Release automation byte-verifies existing draft assets and may upload missing assets. It must reject unexpected or differing assets and must never replace a published asset automatically.
- Stable-identity packages must use maintained Fraia icons at `apps/fraia-electron/build/`; beta-identity packages, including a final version promoted by a stable tag, must use the maintained beta icons at `apps/fraia-electron/build/beta/`. Never publish either identity with Electron's default icon.
- Release jobs must package a reviewed native CalculiX executable plus `THIRD_PARTY_NOTICES.txt` from `apps/fraia-electron/runtimes/calculix/<platform>-<arch>/`. Never fetch an unpinned solver during a release or claim a target whose real packaged solver test has not passed.
- Release jobs must assemble the one deterministic `Fraia-CalculiX-Corresponding-Source.tar` asset from the SHA-pinned upstream sources and reviewed platform build recipes, and must byte-verify its public URL and digest against every runtime manifest before publication. Source assembly must never build or substitute a solver binary.
- Homebrew casks may be added only through an approved release gate, with distinct stable and beta cask tokens, `auto_updates true`, and no cross-channel application or user-data collision. Homebrew is an optional install/update path: a cask must install the same signed identity package and must not disable, redirect, wrap, or become a prerequisite for Fraia's embedded in-app updater. Users may converge on a newer same-channel build through either Fraia or `brew upgrade` without losing settings or user data. Do not add store publication without a separate approved consumer and release contract.
- Fraia releases from its public source repository so installed clients can reach GitHub Releases, update feeds, and provenance without authentication. The maintained same-repository release workflow must fail before building if repository visibility is not public. Do not weaken this gate or introduce a cross-repository token implicitly.
