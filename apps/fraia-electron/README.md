# Fraia Electron Workbench

The Electron shell is the current product UI. It should stay disciplined: one primary renderer window, one Three.js viewport canvas plus one 2D selection overlay, small React DOM, and model-attached graphics rendered through WebGL rather than DOM overlays.

The main process also owns Fraia's embedded Pi runtime and encrypted credential storage. Fraia 0.0.3 exposes only **Sign in with ChatGPT** and fixes every AI workflow to `openai-codex/gpt-5.6-luna` with low reasoning; provider search, API-key entry, model selection, and reasoning selection remain outside the public UI. It starts a token-protected Pi loopback service before the Rust sidecar. The Rust sidecar independently requires a random per-launch bearer token on every endpoint, including health. Renderer code uses narrow IPC methods and never receives either token, credentials, or direct loopback access.

`npm run test:package` deletes prior package output, builds the native Rust sidecar and one exact native Electron directory, verifies both executable architectures, and runs the packaged Electron persistence and solver-boundary test. Packaged mode ignores development sidecar path overrides and fails closed when the architecture-specific bundled sidecar is absent.

## Release Boundary

Stable tags (`vX.Y.Z`) build `Fraia`; beta tags (`vX.Y.Z-beta.N`) build the side-by-side `Fraia Beta` prerelease. The two channels have separate bundle/application ids, package and executable names, artifact prefixes, user-data directories, and updater feeds. Each release builds only its matching channel on native GitHub-hosted runners for exactly six reviewed targets: macOS ARM64/x64, Linux ARM64/x64, and Windows ARM64/x64. Every package must contain same-architecture Electron, `fraia-appd`, and CalculiX executables. Both Windows packages are installed, launched, solver-tested, and uninstalled, while every Linux format is extracted, launched, and solver-tested. The macOS path additionally imports an encrypted Developer ID P12 into a temporary Keychain, notarizes with an App Store Connect P8 key, verifies every nested signature and entitlement, staples and Gatekeeper-assesses the app and DMG, and runs the packaged persistence/solver test after credentials are gone.

Release builds require reviewed native solver payloads under `runtimes/calculix/<platform>-<arch>/`, named `ccx` on macOS/Linux or `ccx.exe` on Windows. Each directory must also contain `THIRD_PARTY_NOTICES.txt` and `runtime-manifest.json`. The manifest pins the upstream and redistribution source digests, build recipe, licenses, executable, notices, bundled dependencies, and exact native dependency closure. The release assembles `Fraia-CalculiX-Corresponding-Source.tar` deterministically from every pinned upstream source and all four reviewed build recipes, then byte-verifies that asset against every runtime manifest before publication. The release remains blocked until all six reviewed runtimes exist; CI never builds, downloads, or substitutes a solver binary at release time.

The macOS review candidate is built only in an explicitly controlled matching native environment with `npm run build:calculix:mac -- --target darwin-arm64|darwin-x64 --output <new-output-dir>`, never by GitHub-hosted automation. The script verifies pinned upstream and toolchain inputs, builds twice from clean paths, requires byte-identical payloads, enforces a macOS 15.0 compatibility ceiling and loader-relative dependency closure, signs a disposable copy, and runs the official `spring1` solver case. Its output is not a release asset until the payload, notices, corresponding source, build evidence, and runtime manifest have been independently reviewed and placed under `runtimes/calculix/`.

The Windows x64 review candidate is acquired and tested in an explicitly controlled native Windows environment with `npm run build:calculix:win -- -OutputDirectory <new-output-dir>`, never by GitHub-hosted automation. It accepts only the SHA-pinned official `ccx_static.exe`, verifies the PE x64 header and exact Windows/UCRT import allowlist, preserves the upstream executable bytes, and runs the same pinned `spring1` case. The solver executable and initial installer remain unsigned; subsequent updater metadata is authenticated separately through Fraia's reviewed TUF trust root.

The Windows ARM64 runtime is built twice from the pinned CalculiX, SPOOLES, ARPACK-NG, OpenBLAS, and MSYS2 recipe sources on the native `windows-11-arm` runner in the reviewed CLANGARM64 environment. Promotion requires byte-identical payloads, ARM64 PE headers, the Windows 10/subsystem contract, closed native imports, complete LLVM and dependency notices, and the official `spring1` case. Release packaging then builds and solver-tests matching native Electron and `fraia-appd` payloads on the same runner architecture.

Linux ARM64 and x64 review candidates are built in explicitly controlled matching native environments inside the digest-pinned Ubuntu 22.04 builder with `npm run build:calculix:linux -- --target linux-arm64|linux-x64 --output <new-output-dir>`, never by GitHub-hosted automation. The script builds pinned SPOOLES, OpenBLAS, ARPACK-NG, and CalculiX sources twice, requires identical bytes, statically closes solver and compiler runtimes, enforces the GLIBC 2.35 symbol ceiling, rejects build paths and unresolved dependencies, and runs `spring1`. Passing the script outside the pinned container is not accepted evidence.

Every packaged platform checks only the static feed embedded for its identity. Stable checks `stable/<platform>/<arch>` and rejects prereleases; beta checks `beta/<platform>/<arch>` and accepts the separately packaged beta line. Publications replace only the matching channel path. The native application menu offers Never, On Startup, Hourly, Every 6 Hours, Every 12 Hours, Daily, and Weekly frequencies plus a manual check; Daily is the default and the choice persists in that identity's isolated user-data directory.

macOS updates retain the Developer ID and notarization trust chain. Windows and Linux packages embed the reviewed public TUF root at `build/update-trust/root.json`; their updater accepts only target metadata signed by the protected targets, snapshot, and timestamp roles. The root private key remains offline, while expiring online metadata is refreshed without changing published target bytes. Native ARM64 and x64 tests cover valid same-channel previous-to-current replacement, corrupt metadata, wrong signatures, retained settings/project/AI data, and persisted trust. The first public macOS package in each channel requires that channel environment's one-time `MACOS_UPDATER_BOOTSTRAP_TAG`; subsequent releases require real same-channel N-1 evidence. Windows and Linux initial installers are not code-signed, so the public download surface must disclose that limitation and provide checksums and provenance before the user establishes Fraia's embedded update trust.

There is no Homebrew or store publication yet. An approved Homebrew rollout must use distinct `fraia` and `fraia-beta` cask tokens, with `Fraia.app` and `Fraia Beta.app` able to coexist; it must not project one channel's package into the other cask.

Stable packaging uses the maintained Fraia artwork under `build/`; beta packaging uses the maintained beta artwork under `build/beta/`. Both fail closed rather than publishing Electron's default icon.

Fraia releases from its public source repository because installed updaters and public binaries require unauthenticated access to GitHub Releases, update feeds, and provenance. The tag workflow fails before building if repository visibility is not public. It does not silently add a cross-repository token or publish an unusable private feed.

## Performance Smoke And Benchmarking

Use the hardware-aware smoke test for default app health:

```sh
npm run smoke:perf
```

The smoke test builds the production bundle, launches the desktop app in capture mode, samples idle process metrics, writes `../../output/electron-performance-smoke.json`, and checks the selected hardware tier budget. Override the tier when comparing machines:

```sh
FRAIA_PERF_TIER=compact_laptop npm run smoke:perf
```

Viewport benchmark examples:

```sh
npm run benchmark:viewport -- --mode object --benchmark random --members 10000 --labels off
npm run benchmark:viewport -- --mode batched --benchmark random --members 10000 --labels off
npm run benchmark:viewport -- --mode batched --benchmark multi --members 50000 --labels off
npm run benchmark:perf-gate
```

Metrics reported by the benchmark include selected hardware tier, scene generation time, renderer prep time, GPU upload time, average/p95/max frame time, draw calls, Three.js object counts, renderer working set, JS heap, and hit-test timings.

Budget tiers are resolved in `scripts/perf-budgets.cjs` from RAM, CPU thread count, platform, and architecture. `benchmark:perf-gate` applies the selected tier to the standard large-model matrix and fails batched mode when frame time, draw calls, renderer memory, or typical hit testing exceed budget.
