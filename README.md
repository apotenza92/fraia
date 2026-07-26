# Fraia

Fraia is a Rust-first structural engineering workbench for modelling, preliminary analysis, and traceable option development.

The current product combines:

- an Electron + React desktop shell using shadcn UI components;
- a Three.js structural viewport;
- a local Rust application service (`fraia-appd`);
- a Rust engineering core and CLI;
- a linear elastic 2D frame solver for preliminary concept studies.

Fraia is not yet code-compliant design software. Current analysis is preliminary and does not replace project-specific engineering review.

## Architecture

Rust owns project state, structural models, validation, analysis, runs, and the application API. Electron owns app chrome and viewport interaction. Authored state, resolved analysis state, and immutable run artefacts remain distinct.

Start with [`docs/documentation-map.md`](docs/documentation-map.md) for maintained architecture and product documentation.

## Desktop workbench

```sh
cd apps/fraia-electron
npm install
npm start
```

This launches the Fraia Electron application and starts the local Rust service automatically.

For an isolated development launch with empty, disposable application, project, and chat data:

```sh
cd apps/fraia-electron
npm run start:clean
```

## Rust workspace

```sh
cargo build
cargo test --workspace --all-features
```

The workspace contains the engineering core, application API/service, and CLI. See [`docs/rust-workspace-layout.md`](docs/rust-workspace-layout.md) for responsibilities.

## CLI examples

```sh
cargo run -p fraia-cli -- init my-project
cargo run -p fraia-cli -- plan my-project
cargo run -p fraia-cli -- optimize my-project
cargo run -p fraia-cli -- demo demo-project
cargo run -p fraia-cli -- beam-demo beam-project
cargo run -p fraia-cli -- beam-init beam-project 8.0 5.0 12.0 4.0
cargo run -p fraia-cli -- beam-size beam-project
```

Run artefacts are written beneath `<project>/runs/<timestamp>/`.

## Deterministic checks

```sh
python3 scripts/check-repository-hygiene.py
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features
cargo test --workspace --all-features
bash scripts/smoke-test-demo-flow.sh
bash scripts/smoke-test-beam-flow.sh
python3 scripts/lint-knowledge.py
python3 scripts/validate-knowledge-next.py
cd apps/fraia-electron && npm run typecheck && npm run build
```
