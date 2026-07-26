# Fraia Rust Workspace Layout

This document records the maintained repository and crate boundaries.

```text
Cargo.toml
crates/
  fraia-math/
  fraia-geometry/
  fraia-physics/
  fraia-core/
  fraia-app-api/
apps/
  fraia-cli/
  fraia-appd/
  fraia-electron/    # Electron product shell; not a Cargo crate
```

## Responsibilities

### `fraia-math`

Foundational scalar, vector, matrix, tensor, and low-level algebra types.

### `fraia-geometry`

Points, frames, transforms, and geometry semantics built on `fraia-math`.

### `fraia-physics`

Physical quantities and engineering value types that are independent of the product shell.

### `fraia-core`

Fraia project state, authored structural objects, builders, validation, resolution, preliminary analysis, run artefacts, and downstream engineering data.

### `fraia-app-api`

Typed request and response contracts shared across the application boundary.

### `fraia-cli`

Command-line access to deterministic Fraia project and engineering operations.

### `fraia-appd`

Local Rust application service used by the desktop shell. It loads and saves projects, invokes core operations, and persists run artefacts.

### `fraia-electron`

Electron + React product shell, shadcn app chrome, Three.js viewport, and local `fraia-appd` lifecycle. Engineering truth does not live in renderer or chat state.

## Boundary rules

- Keep foundational crates independent of Electron and product UI concerns.
- Keep shell/backend payloads typed in `fraia-app-api`.
- Keep authored state, resolved state, and immutable run artefacts distinct.
- Add a new crate only when a maintained dependency boundary is clearer than a module inside an existing crate.
