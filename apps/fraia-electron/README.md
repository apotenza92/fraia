# Fraia Electron Workbench

The Electron shell is the current product UI. It should stay disciplined: one primary renderer window, one Three.js viewport canvas plus one 2D selection overlay, small React DOM, and model-attached graphics rendered through WebGL rather than DOM overlays.

The main process also owns Fraia's embedded Pi runtime and encrypted credential storage. Fraia 0.0.1 exposes only **Sign in with ChatGPT** and fixes every AI workflow to `openai-codex/gpt-5.6-luna` with low reasoning; provider search, API-key entry, model selection, and reasoning selection remain outside the public UI. It starts a token-protected Pi loopback service before the Rust sidecar. The Rust sidecar independently requires a random per-launch bearer token on every endpoint, including health. Renderer code uses narrow IPC methods and never receives either token, credentials, or direct loopback access.

`npm run test:package` deletes prior package output, builds the native Rust sidecar and one exact native Electron directory, verifies both executable architectures, and runs the packaged Electron persistence and solver-boundary test. Packaged mode ignores development sidecar path overrides and fails closed when the architecture-specific bundled sidecar is absent.

## Release Boundary

Stable tag releases use one `Fraia` identity and native GitHub-hosted runners for exactly five reviewed targets: macOS ARM64/x64, Linux ARM64/x64, and Windows x64. There is no separate beta tag, build, identity, or package set. Every package must contain same-architecture Electron, `fraia-appd`, and CalculiX executables. The Windows x64 package is installed, launched, solver-tested, and uninstalled, while every Linux format is extracted, launched, and solver-tested. The macOS path additionally imports an encrypted Developer ID P12 into a temporary Keychain, notarizes with an App Store Connect P8 key, verifies every nested signature and entitlement, staples and Gatekeeper-assesses the app and DMG, and runs the packaged persistence/solver test after credentials are gone.

Release builds require reviewed native solver payloads under `runtimes/calculix/<platform>-<arch>/`, named `ccx` on macOS/Linux or `ccx.exe` on Windows. Each directory must also contain `THIRD_PARTY_NOTICES.txt` and `runtime-manifest.json`. The manifest pins the upstream and redistribution source digests, build recipe, licenses, executable, notices, bundled dependencies, and exact native dependency closure. The release assembles `Fraia-CalculiX-Corresponding-Source.tar` deterministically from every pinned upstream source and all three reviewed build recipes, then byte-verifies that asset against every runtime manifest before publication. The release remains blocked until all five reviewed runtimes exist; CI never builds, downloads, or substitutes a solver binary at release time. Windows ARM64 is deliberately unsupported and fails closed until its complete native solver-backed package has independent evidence.

The macOS review candidate is built only in an explicitly controlled matching native environment with `npm run build:calculix:mac -- --target darwin-arm64|darwin-x64 --output <new-output-dir>`, never by GitHub-hosted automation. The script verifies pinned upstream and toolchain inputs, builds twice from clean paths, requires byte-identical payloads, enforces a macOS 15.0 compatibility ceiling and loader-relative dependency closure, signs a disposable copy, and runs the official `spring1` solver case. Its output is not a release asset until the payload, notices, corresponding source, build evidence, and runtime manifest have been independently reviewed and placed under `runtimes/calculix/`.

The Windows x64 review candidate is acquired and tested in an explicitly controlled native Windows environment with `npm run build:calculix:win -- -OutputDirectory <new-output-dir>`, never by GitHub-hosted automation. It accepts only the SHA-pinned official `ccx_static.exe`, verifies the PE x64 header and exact Windows/UCRT import allowlist, preserves the upstream executable bytes, and runs the same pinned `spring1` case. It does not sign the executable or enable Windows updates; trusted Windows installation remains a separate release prerequisite.

Linux ARM64 and x64 review candidates are built in explicitly controlled matching native environments inside the digest-pinned Ubuntu 22.04 builder with `npm run build:calculix:linux -- --target linux-arm64|linux-x64 --output <new-output-dir>`, never by GitHub-hosted automation. The script builds pinned SPOOLES, OpenBLAS, ARPACK-NG, and CalculiX sources twice, requires identical bytes, statically closes solver and compiler runtimes, enforces the GLIBC 2.35 symbol ceiling, rejects build paths and unresolved dependencies, and runs `spring1`. Passing the script outside the pinned container is not accepted evidence.

Signed macOS packages check the stable static feed automatically, download in the background, and install on quit. A stable publication writes byte-identical metadata to both the `stable` and `beta` feed paths so both channels resolve to the same accepted stable package. The native application menu offers Never, On Startup, Hourly, Every 6 Hours, Every 12 Hours, Daily, and Weekly frequencies plus a manual check; Daily is the default and the choice persists in the canonical Fraia user-data directory. Release CI tests previous-to-current valid, corrupt, and wrong-signature paths on native ARM64 and Intel runners. The first public release requires the one-time `MACOS_UPDATER_BOOTSTRAP_TAG`; subsequent releases require real N-1 evidence. Windows and Linux updating stays disabled until those platforms have a trusted signing boundary.

There is no Homebrew or store publication yet. Add one only after a first accepted public artifact establishes a real package consumer.

Release packaging uses the maintained Fraia artwork at `build/icon.icns`, `build/icon.ico`, and `build/icons/512x512.png`; it fails closed rather than publishing Electron's default icon.

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
