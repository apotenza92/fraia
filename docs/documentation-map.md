# Fraia Documentation Map

This map identifies the maintained source for each documentation concern. Update an existing canonical document before creating another one.

## Public product surface

- [`index.html`](index.html) — standalone Fraia desktop download page prepared for publication at `apotenza92.github.io/fraia`.

## Core architecture

- [`engineering-core.md`](engineering-core.md) — product and engineering architecture.
- [`builder-graph-architecture.md`](builder-graph-architecture.md) — builders, archetypes, composition, and authored-object provenance.
- [`resolution-and-runs.md`](resolution-and-runs.md) — authored, resolved, and immutable run boundaries.
- [`project-layout.md`](project-layout.md) — Fraia project files and run artefact layout.
- [`package-system.md`](package-system.md) — reusable engineering packages, locking, and migration.
- [`structural-app-object-model.md`](structural-app-object-model.md) — canonical authored structural vocabulary.
- [`engineering-output-pipeline.md`](engineering-output-pipeline.md) — design actions, checks, and downstream outputs.
- [`rust-workspace-layout.md`](rust-workspace-layout.md) — current repository and crate responsibilities.

## Focused supporting documents

- [`primitives-and-archetypes.md`](primitives-and-archetypes.md) — primitive and archetype concepts.
- [`frames-and-connectivity.md`](frames-and-connectivity.md) — frames, placements, ports, connectivity, supports, and releases.
- [`validation-and-diagnostics.md`](validation-and-diagnostics.md) — validation layers and diagnostic semantics.
- [`solver-adapters.md`](solver-adapters.md) — solver handoff and normalised results.
- [`tool-contracts.md`](tool-contracts.md) — deterministic CLI, service, and agent-facing operations.
- [`structural-app-ui-layer.md`](structural-app-ui-layer.md) — authoring and inspection exposure policy.
- [`visualization-strategy.md`](visualization-strategy.md) — Three.js viewport and rendering direction.
- [`diagramming-and-geometry.md`](diagramming-and-geometry.md) — diagram and geometry representations.
- [`connections-and-detailing-strategy.md`](connections-and-detailing-strategy.md) — connection and detailing escalation.
- [`hierarchical-fidelity-and-submodeling.md`](hierarchical-fidelity-and-submodeling.md) — fidelity and submodel boundaries.
- [`briefing-and-planning-workflow.md`](briefing-and-planning-workflow.md) — engineering brief and project decision context.
- [`intent-and-tradeoffs.md`](intent-and-tradeoffs.md) — objectives, constraints, and tradeoffs.
- [`optimization-loop.md`](optimization-loop.md) — deterministic option iteration.
- [`math-kernel.md`](math-kernel.md) — focused mathematical foundation.

## Knowledge systems

- [`knowledge-backend.md`](knowledge-backend.md) — durable knowledge architecture and trust boundaries.
- [`knowledge/README.md`](knowledge/README.md) — current operational compiled wiki.
- [`knowledge-next/README.md`](knowledge-next/README.md) — active typed, source-first knowledge rebuild. The existing wiki remains operational until an explicit cutover.

## Active product decisions

- [`../plans/design-option-analysis.md`](../plans/design-option-analysis.md) — current preliminary option-analysis and comparison behaviour.

Changing work state belongs in GitHub issues and pull requests. Completed implementation plans, migration logs, subagent reports, and temporary handoffs do not belong in the maintained documentation set.
