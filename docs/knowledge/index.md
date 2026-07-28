# Fraia Knowledge Index

_Status: active v0.3_
_Date: 2026-05-07_

This is the registry for compiled Fraia knowledge pages. Use [`topic-map.md`](topic-map.md) for the broader category tree, roadmap, and missing/draft topics.

## Core files

- [README](README.md) — purpose and trust boundary.
- [Workflow](workflow.md) — self-update, proposal, synthesis, lint/reviewer, Fraia Knowledge Steward, and promotion rules.
- [Adapter contract](adapter-contract.md) — contract for third-party/maintainer source learning packets and wiki update proposals.
- [Contributing](contributing.md) — community and maintainer guide for knowledge requests, sources, corrections, and PRs.
- [Ingestion workflow](ingestion.md) — optional maintainer/adapter source ingestion guidance; not app runtime scope.
- [Topic map](topic-map.md) — nested category tree and seeding roadmap.
- [Schema](schema.md) — required page fields, citation policy, media policy, and agent maintenance rules.
- [Source registry](sources.md) — optional bibliography/source-governance aid; page-level sources remain mandatory.
- [Media policy](media/README.md) — committed wiki media rules and manifest requirement.
- [Chunk manifest template](templates/chunk-manifest.md) — template for bounded temporary ingestion runs.
- [Source learning packet template](templates/source-learning-packet.md) — normalized output for external ingestion/subagent reading.
- [Wiki update proposal template](templates/wiki-update-proposal.md) — proposed compiled-page edit shape.
- [Proposals](proposals/README.md) — draft proposal inbox and template.
- [Legacy raw/source notes](raw/README.md) — exceptional compact source notes; not the default ingestion path.
- [Compiled wiki area](wiki/README.md) — compiled pages and topic namespaces.
- [Operation log](wiki/log.md) — append-only update log.

## Start here

- [Load paths](wiki/analysis/load-paths.md) — how actions travel through structural objects, connections, supports, foundations, and soil.
- [Free-body diagrams and equilibrium](wiki/analysis/free-body-diagrams-and-equilibrium.md) — how to isolate bodies and write force/moment balance before reactions or internal actions are explained.
- [Reactions and support idealisation](wiki/analysis/reactions-and-support-idealisation.md) — how support DOF assumptions create reaction components and affect interpretation.
- [Static determinacy and restraint](wiki/analysis/static-determinacy-and-restraint.md) — how equilibrium, restraint sufficiency, determinacy, indeterminacy, and mechanisms differ.
- [Truss analysis and two-force members](wiki/analysis/truss-analysis-and-two-force-members.md) — how truss assumptions, joint/section equilibrium, zero-force members, and axial force signs should be scoped.
- [Beam shear and moment diagrams](wiki/analysis/beam-shear-and-moment-diagrams.md) — how beam internal shear/moment diagrams should be interpreted, sourced, and passed downstream.
- [Matrix stiffness method](wiki/analysis/matrix-stiffness-method.md) — how resolved nodes, DOFs, elements, loads, restraints, assembly, and recovered results fit Fraia's run pipeline.
- [Second-order effects and stability](wiki/analysis/second-order-effects-and-stability.md) — how deformed-geometry effects and stability sensitivity should be scoped before design checks.
- [Supports, restraints, and releases](wiki/modeling/supports-restraints-and-releases.md) — DOF, coordinate-frame, support, restraint, constraint, and release assumptions.
- [Member end releases](wiki/modeling/member-end-releases.md) — how member-end force/moment transfer assumptions differ from supports and resolved element topology.
- [Constraints, rigid links, and diaphragms](wiki/modeling/constraints-rigid-links-and-diaphragms.md) — how inter-node constraints and diaphragm assumptions differ from supports and releases.
- [Connection fixity and partial restraint modeling](wiki/modeling/connection-fixity-and-partial-restraint.md) — how pinned, rigid, and semi-rigid connection assumptions affect frame behavior.
- [Local and global coordinate systems](wiki/modeling/local-and-global-coordinate-systems.md) — coordinate-frame meaning for loads, releases, reactions, member forces, and check inputs.
- [Finite-element idealisation](wiki/modeling/finite-element-idealisation.md) — authored structural objects vs analysis nodes/elements/meshes.
- [Instability mechanisms](wiki/diagnostics/instability-mechanisms.md) — solver/model instability causes and diagnostic workflow.

## Knowledge areas

### Analysis and modeling

- [Load paths](wiki/analysis/load-paths.md)
- [Free-body diagrams and equilibrium](wiki/analysis/free-body-diagrams-and-equilibrium.md)
- [Reactions and support idealisation](wiki/analysis/reactions-and-support-idealisation.md)
- [Static determinacy and restraint](wiki/analysis/static-determinacy-and-restraint.md)
- [Truss analysis and two-force members](wiki/analysis/truss-analysis-and-two-force-members.md)
- [Beam shear and moment diagrams](wiki/analysis/beam-shear-and-moment-diagrams.md)
- [Matrix stiffness method](wiki/analysis/matrix-stiffness-method.md)
- [Second-order effects and stability](wiki/analysis/second-order-effects-and-stability.md)
- [Finite-element idealisation](wiki/modeling/finite-element-idealisation.md)
- [Supports, restraints, and releases](wiki/modeling/supports-restraints-and-releases.md)
- [Member end releases](wiki/modeling/member-end-releases.md)
- [Constraints, rigid links, and diaphragms](wiki/modeling/constraints-rigid-links-and-diaphragms.md)
- [Connection fixity and partial restraint modeling](wiki/modeling/connection-fixity-and-partial-restraint.md)
- [Local and global coordinate systems](wiki/modeling/local-and-global-coordinate-systems.md)

### Loads

- [Load cases and load combinations](wiki/loads/load-cases-and-combinations.md)
- [Gravity and lateral loads](wiki/loads/gravity-and-lateral-loads.md)
- [Load application and equivalent nodal loads](wiki/loads/load-application-and-equivalent-nodal-loads.md)
- [Area, line, point, and member loads](wiki/loads/area-line-point-and-member-loads.md)

### Stability and diagnostics

- [Bracing principles](wiki/stability/bracing-principles.md)
- [Member restraint and unbraced length](wiki/stability/member-restraint-and-unbraced-length.md)
- [Lateral-torsional buckling concepts](wiki/stability/lateral-torsional-buckling-concepts.md)
- [Compression member buckling concepts](wiki/stability/compression-member-buckling-concepts.md)
- [Instability mechanisms](wiki/diagnostics/instability-mechanisms.md)
- [Unconnected or underrestrained models](wiki/diagnostics/unconnected-or-underrestrained-models.md)
- [Overreleased members and all-pin mechanisms](wiki/diagnostics/overreleased-members-and-all-pin-mechanisms.md)
- [Reaction sanity checks](wiki/diagnostics/reaction-sanity-checks.md)
- [Analysis result review before design checks](wiki/diagnostics/analysis-result-review-before-design-checks.md)

### Steel

- [Steel material properties and section families](wiki/materials/steel/material-properties-and-section-families.md)
- [Steel member behavior](wiki/materials/steel/member-behavior.md)
- [Steel compression members](wiki/materials/steel/compression-members.md)
- [Steel beams and bending members](wiki/materials/steel/beams-and-bending-members.md)
- [Steel connections concept taxonomy](wiki/materials/steel/connections-concept-taxonomy.md)
- [Steel design action and check-input separation](wiki/materials/steel/design-action-check-input-separation.md)
- [Steel portal-frame bracing](wiki/steel/portal-frames/bracing.md)
- [Steel portal-frame system overview](wiki/steel/portal-frames/system-overview.md)
- [Portal-frame base fixity tradeoffs](wiki/steel/portal-frames/base-fixity-tradeoffs.md)
- [Purlins and girts as restraint/load-transfer members](wiki/steel/portal-frames/purlins-girts-and-restraint.md)
- [Longitudinal vs transverse stability in portal frames](wiki/steel/portal-frames/longitudinal-vs-transverse-stability.md)

### Product and pipeline

- [Authored/resolved/run artifact boundaries](wiki/product/authored-resolved-run-boundaries.md)
- [Design actions, check inputs, and check results](wiki/product/design-actions-check-inputs-and-results.md)
- [Engineering assumptions and provenance](wiki/product/engineering-assumptions-and-provenance.md)
- [Scheme generation from knowledge](wiki/product/scheme-generation-from-knowledge.md)
- [Structural design option intelligence](wiki/product/structural-design-option-intelligence.md)

## Topic namespaces

- [Analysis](wiki/analysis/index.md)
- [Loads](wiki/loads/index.md)
- [Modeling and idealisation](wiki/modeling/index.md)
- [Stability](wiki/stability/index.md)
- [Diagnostics](wiki/diagnostics/index.md)
- [Product and pipeline knowledge](wiki/product/index.md)
- [Materials](wiki/materials/index.md)
  - [Steel material/member behavior](wiki/materials/steel/index.md)
- [Steel systems](wiki/steel/index.md)
  - [Portal frames](wiki/steel/portal-frames/index.md)
  - [Steel portal-frame bracing](wiki/steel/portal-frames/bracing.md)

## Legacy raw/source notes

`raw/` is legacy/exceptional. Future source extraction should use temporary staging via the [ingestion workflow](ingestion.md); compiled pages cite original sources directly in their own `## Sources` sections.

- [Raw/source notes README](raw/README.md)

## Related architecture docs

- [Fraia Knowledge Backend](../knowledge-backend.md)
- [Documentation Map](../documentation-map.md)
