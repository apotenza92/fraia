# Fraia

Fraia is a Rust-first structural engineering workbench for modelling, preliminary analysis, and traceable option development.

[Download Fraia](https://apotenza92.github.io/fraia/) · [Documentation](docs/documentation-map.md) · [Report an issue](https://github.com/apotenza92/fraia/issues)

> [!IMPORTANT]
> Fraia is early-stage software for preliminary work. It is not code-compliant design software and does not replace project-specific engineering review.

## What it includes

- A desktop modelling workspace with a Three.js structural viewport
- A Rust engineering core, local application service, and CLI
- Preliminary linear elastic 2D frame analysis
- Traceable project state, validation, and immutable run artefacts

## Run locally

```sh
cd apps/fraia-electron
npm install
npm start
```

This launches the Electron desktop app and its local Rust service. Use `npm run start:clean` for an isolated launch with disposable app, project, and chat data.

For the Rust workspace:

```sh
cargo test --workspace --all-features
```

See the [documentation map](docs/documentation-map.md) for architecture, engineering concepts, knowledge systems, and contributor checks.
