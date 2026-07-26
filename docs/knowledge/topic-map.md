# Fraia Knowledge Topic Map

_Status: active v0.3_
_Date: 2026-05-07_

This is the category tree and roadmap for the LLM wiki. [`index.md`](index.md) is the compiled-page registry; this file tracks compiled, draft, raw, and missing topics.

The roadmap is a baseline coverage plan, not a commitment to ingest any one textbook. Textbooks such as Hibbeler, Brohn, or similar structural-analysis references may seed topic taxonomy and corroborate claims when cited logically, but compiled pages should prefer academic/open textbooks, university notes, professional institutions, government guidance, private textbooks with page/section locators, and otherwise well-sourced references. Avoid SEO/content-marketing and calculator/tool marketing pages for compiled guidance when stronger sources exist.

Topic status values: `missing`, `stub`, `raw`, `draft`, `compiled`, `reviewed`.
Core/policy file status values: `active`, `deprecated`.
Priority values: `baseline-now`, `baseline-next`, `later`.

## Baseline Coverage Themes

- **Structural analysis substrate**: equilibrium, load paths, restraints, determinacy, internal actions, deflection, stability, and matrix/FEM idealisation.
- **Fraia modeling substrate**: authored structural objects, resolved topology, releases, constraints, loads, analysis elements, diagnostics, and immutable run artifacts.
- **Steel first-material track**: steel member behavior, portal frames, bracing, restraint, connections, preliminary section families, and concept-level design/check inputs.
- **Diagnostics and review**: underrestraint, overrelease, disconnected topology, ill-conditioning, result sanity checks, and explainable failure modes.

## Structural Engineering Fundamentals

- **Load paths** — status: compiled; priority: baseline-now; path: [`wiki/analysis/load-paths.md`](wiki/analysis/load-paths.md); why: basis for scheme generation, reactions, bracing, and diagnostics.
- **Free-body diagrams and equilibrium** — status: compiled; priority: baseline-now; path: [`wiki/analysis/free-body-diagrams-and-equilibrium.md`](wiki/analysis/free-body-diagrams-and-equilibrium.md); why: foundation for reaction calculation, internal action diagrams, and agent explanations.
- **Reactions and support idealisation** — status: compiled; priority: baseline-now; path: [`wiki/analysis/reactions-and-support-idealisation.md`](wiki/analysis/reactions-and-support-idealisation.md); why: connects support DOFs, load paths, determinacy, and reaction interpretation.
- **Internal actions and sign conventions** — status: missing; priority: baseline-next; intended path: `wiki/analysis/internal-actions-and-sign-conventions.md`; why: needed for force diagrams, result interpretation, and check inputs.
- **Serviceability and deflection concepts** — status: missing; priority: baseline-next; intended path: `wiki/analysis/serviceability-and-deflection-concepts.md`; why: concept-level scheme comparison before code-specific limits.

## Structural Analysis Methods

- **Static determinacy and restraint** — status: compiled; priority: baseline-now; path: [`wiki/analysis/static-determinacy-and-restraint.md`](wiki/analysis/static-determinacy-and-restraint.md); why: separates equilibrium solvability, restraint sufficiency, indeterminacy, and mechanism risk for diagnostics.
- **Truss analysis and two-force members** — status: compiled; priority: baseline-now; path: [`wiki/analysis/truss-analysis-and-two-force-members.md`](wiki/analysis/truss-analysis-and-two-force-members.md); why: common primitive system and useful source of zero-force/member-force reasoning.
- **Beam shear and moment diagrams** — status: compiled; priority: baseline-now; path: [`wiki/analysis/beam-shear-and-moment-diagrams.md`](wiki/analysis/beam-shear-and-moment-diagrams.md); why: basic beam reasoning, load/result interpretation, and future beam checks.
- **Frame internal force diagrams** — status: missing; priority: baseline-next; intended path: `wiki/analysis/frame-internal-force-diagrams.md`; why: portal frames and braced frames need axial/shear/moment interpretation.
- **Deflection by virtual work / unit load method** — status: missing; priority: baseline-next; intended path: `wiki/analysis/virtual-work-and-unit-load-deflection.md`; why: supports deflection intuition and approximate serviceability explanations.
- **Influence lines and moving loads** — status: missing; priority: later; intended path: `wiki/analysis/influence-lines-and-moving-loads.md`; why: important for bridges/cranes but not first building/portal-frame wedge.
- **Arches, cables, and funicular action** — status: missing; priority: later; intended path: `wiki/analysis/arches-cables-and-funicular-action.md`; why: useful for broader structural vocabulary but not first steel portal-frame wedge.
- **Force method for indeterminate structures** — status: missing; priority: baseline-next; intended path: `wiki/analysis/force-method-indeterminate-structures.md`; why: explains compatibility/redundants behind indeterminate behavior.
- **Slope-deflection and moment distribution** — status: missing; priority: later; intended path: `wiki/analysis/slope-deflection-and-moment-distribution.md`; why: classical frame-analysis intuition; lower priority than matrix stiffness/Fraia solver model.
- **Matrix stiffness method** — status: compiled; priority: baseline-now; path: [`wiki/analysis/matrix-stiffness-method.md`](wiki/analysis/matrix-stiffness-method.md); why: closest conceptual bridge to Fraia's resolved analysis topology and solver adapters.
- **Second-order effects and stability** — status: compiled; priority: baseline-now; path: [`wiki/analysis/second-order-effects-and-stability.md`](wiki/analysis/second-order-effects-and-stability.md); why: portal frames, compression members, bracing, and diagnostics depend on stability assumptions.

## Loads and Actions

- **Load cases and combinations** — status: compiled; priority: baseline-now; path: [`wiki/loads/load-cases-and-combinations.md`](wiki/loads/load-cases-and-combinations.md); why: separates authored loads, cases, combinations, and run artifacts.
- **Gravity and lateral loads** — status: compiled; priority: baseline-now; path: [`wiki/loads/gravity-and-lateral-loads.md`](wiki/loads/gravity-and-lateral-loads.md); why: prevents generic/hardcoded load assumptions.
- **Load application and equivalent nodal loads** — status: compiled; priority: baseline-now; path: [`wiki/loads/load-application-and-equivalent-nodal-loads.md`](wiki/loads/load-application-and-equivalent-nodal-loads.md); why: maps authored loads to resolved analysis elements and solver loads.
- **Area, line, point, and member loads** — status: compiled; priority: baseline-now; path: [`wiki/loads/area-line-point-and-member-loads.md`](wiki/loads/area-line-point-and-member-loads.md); why: primitive-first authored load modeling.
- **Load paths for diaphragms and collectors** — status: missing; priority: baseline-next; intended path: `wiki/loads/diaphragms-collectors-and-load-transfer.md`; why: needed for steel bracing and building-level lateral systems.
- **Construction-stage and temporary loads** — status: missing; priority: later; intended path: `wiki/loads/construction-stage-and-temporary-loads.md`; why: important but outside first concept-stage wedge.

## Modeling and Idealisation

- **Finite-element idealisation** — status: compiled; priority: baseline-now; path: [`wiki/modeling/finite-element-idealisation.md`](wiki/modeling/finite-element-idealisation.md); why: keeps authored members/plates distinct from analysis elements.
- **Supports, restraints, and releases** — status: compiled; priority: baseline-now; path: [`wiki/modeling/supports-restraints-and-releases.md`](wiki/modeling/supports-restraints-and-releases.md); why: directly maps to `SupportAssignment` and `ReleaseAssignment`.
- **Member end releases** — status: compiled; priority: baseline-now; path: [`wiki/modeling/member-end-releases.md`](wiki/modeling/member-end-releases.md); why: separates `ReleaseAssignment` member-end semantics from supports and constraints.
- **Local and global coordinate systems** — status: compiled; priority: baseline-now; path: [`wiki/modeling/local-and-global-coordinate-systems.md`](wiki/modeling/local-and-global-coordinate-systems.md); why: critical for member releases, loads, reactions, and result interpretation.
- **Constraints, rigid links, and diaphragms** — status: compiled; priority: baseline-now; path: [`wiki/modeling/constraints-rigid-links-and-diaphragms.md`](wiki/modeling/constraints-rigid-links-and-diaphragms.md); why: differentiates supports from inter-node constraints and downstream solver topology.
- **Mesh density and analysis element subdivision** — status: missing; priority: baseline-next; intended path: `wiki/modeling/mesh-density-and-element-subdivision.md`; why: preserves authored/resolved distinction and result interpretation.
- **Connection fixity and partial restraint modeling** — status: compiled; priority: baseline-now; path: [`wiki/modeling/connection-fixity-and-partial-restraint.md`](wiki/modeling/connection-fixity-and-partial-restraint.md); why: steel portal frames and releases depend on assumed fixity.
- **Plate/shell idealisation basics** — status: missing; priority: later; intended path: `wiki/modeling/plate-shell-idealisation-basics.md`; why: important for slabs/walls/diaphragms after member-first wedge.

## Stability and Bracing

- **Bracing principles** — status: compiled; priority: baseline-now; path: [`wiki/stability/bracing-principles.md`](wiki/stability/bracing-principles.md); why: general bracing/system guidance.
- **Steel portal-frame bracing** — status: compiled; priority: baseline-now; path: [`wiki/steel/portal-frames/bracing.md`](wiki/steel/portal-frames/bracing.md); why: existing canonical portal-frame-specific guidance.
- **Member restraint and unbraced length** — status: compiled; priority: baseline-now; path: [`wiki/stability/member-restraint-and-unbraced-length.md`](wiki/stability/member-restraint-and-unbraced-length.md); why: ties bracing, LTB, compression buckling, and steel checks together.
- **Lateral-torsional buckling concepts** — status: compiled; priority: baseline-now; path: [`wiki/stability/lateral-torsional-buckling-concepts.md`](wiki/stability/lateral-torsional-buckling-concepts.md); why: key steel beam/rafter behavior and restraint topic.
- **Compression member buckling concepts** — status: compiled; priority: baseline-now; path: [`wiki/stability/compression-member-buckling-concepts.md`](wiki/stability/compression-member-buckling-concepts.md); why: columns, braces, and portal-frame stability depend on it.
- **System stability and sway frames** — status: missing; priority: baseline-next; intended path: `wiki/stability/system-stability-and-sway-frames.md`; why: separates member stability from frame/system stability.
- **Diaphragm and collector action** — status: missing; priority: baseline-next; intended path: `wiki/stability/diaphragm-and-collector-action.md`; why: building-level lateral load path and bracing coordination.

## Steel First-Material Track

- **Steel member behavior** — status: compiled; priority: baseline-now; path: [`wiki/materials/steel/member-behavior.md`](wiki/materials/steel/member-behavior.md); why: separates behavior families from code-specific checks.
- **Steel material properties and section families** — status: compiled; priority: baseline-now; path: [`wiki/materials/steel/material-properties-and-section-families.md`](wiki/materials/steel/material-properties-and-section-families.md); why: Fraia needs section/material vocabulary before preliminary steel schemes.
- **Steel tension members** — status: missing; priority: baseline-next; intended path: `wiki/materials/steel/tension-members.md`; why: ties/braces and net/gross section behavior.
- **Steel compression members** — status: compiled; priority: baseline-now; path: [`wiki/materials/steel/compression-members.md`](wiki/materials/steel/compression-members.md); why: columns and braces are first-order steel design behavior.
- **Steel beams and bending members** — status: compiled; priority: baseline-now; path: [`wiki/materials/steel/beams-and-bending-members.md`](wiki/materials/steel/beams-and-bending-members.md); why: beams/rafters/purlins and LTB/serviceability concepts.
- **Steel combined actions** — status: missing; priority: baseline-next; intended path: `wiki/materials/steel/combined-actions.md`; why: portal-frame members need axial + bending behavior before code checks.
- **Steel connections concept taxonomy** — status: compiled; priority: baseline-now; path: [`wiki/materials/steel/connections-concept-taxonomy.md`](wiki/materials/steel/connections-concept-taxonomy.md); why: simple, moment, base, splice, brace, and purlin connections shape scheme assumptions.
- **Steel connection force transfer** — status: missing; priority: baseline-next; intended path: `wiki/materials/steel/connection-force-transfer.md`; why: downstream detailing/check inputs and load path clarity.
- **Steel preliminary sizing heuristics** — status: missing; priority: baseline-next; intended path: `wiki/materials/steel/preliminary-sizing-heuristics.md`; why: useful for concept generation but must stay source-scoped and non-code-final.
- **Steel design action/check-input separation** — status: compiled; priority: baseline-now; path: [`wiki/materials/steel/design-action-check-input-separation.md`](wiki/materials/steel/design-action-check-input-separation.md); why: preserves analysis results, design actions, check inputs, and check results as distinct stages.

## Steel Systems

- **Steel portal-frame bracing** — status: compiled; priority: baseline-now; path: [`wiki/steel/portal-frames/bracing.md`](wiki/steel/portal-frames/bracing.md); why: canonical portal-frame bracing guidance.
- **Steel portal-frame system overview** — status: compiled; priority: baseline-now; path: [`wiki/steel/portal-frames/system-overview.md`](wiki/steel/portal-frames/system-overview.md); why: first steel building archetype should explain frames, bays, rafters, columns, purlins, girts, bracing, and bases.
- **Portal-frame base fixity tradeoffs** — status: compiled; priority: baseline-now; path: [`wiki/steel/portal-frames/base-fixity-tradeoffs.md`](wiki/steel/portal-frames/base-fixity-tradeoffs.md); why: fixed vs pinned bases affect frame action, foundations, drift, and bracing assumptions.
- **Portal-frame haunches and rafters** — status: missing; priority: baseline-next; intended path: `wiki/steel/portal-frames/haunches-and-rafters.md`; why: common portal-frame behavior and preliminary scheme vocabulary.
- **Purlins and girts as restraint/load-transfer members** — status: compiled; priority: baseline-now; path: [`wiki/steel/portal-frames/purlins-girts-and-restraint.md`](wiki/steel/portal-frames/purlins-girts-and-restraint.md); why: affects load paths, member restraint, and bracing-system assumptions.
- **Longitudinal vs transverse stability in portal frames** — status: compiled; priority: baseline-now; path: [`wiki/steel/portal-frames/longitudinal-vs-transverse-stability.md`](wiki/steel/portal-frames/longitudinal-vs-transverse-stability.md); why: prevents arbitrary bracing and clarifies direction-dependent stability systems.
- **Industrial shed / portal-frame load path** — status: missing; priority: baseline-next; intended path: `wiki/steel/portal-frames/industrial-shed-load-path.md`; why: practical first archetype for Fraia schemes.
- **Steel braced frames** — status: missing; priority: baseline-next; intended path: `wiki/steel/braced-frames.md`; why: general braced steel systems beyond portal frames.
- **Steel moment frames** — status: missing; priority: later; intended path: `wiki/steel/moment-frames.md`; why: useful later, but first wedge is portal-frame/braced systems.

## Diagnostics and Failure Modes

- **Instability mechanisms** — status: compiled; priority: baseline-now; path: [`wiki/diagnostics/instability-mechanisms.md`](wiki/diagnostics/instability-mechanisms.md); why: improves solver failure explanations and pre-solve validation.
- **Unconnected or underrestrained models** — status: compiled; priority: baseline-now; path: [`wiki/diagnostics/unconnected-or-underrestrained-models.md`](wiki/diagnostics/unconnected-or-underrestrained-models.md); why: separates topology/connectivity and ineffective-restraint diagnostics from broader instability mechanisms.
- **Overreleased members and all-pin mechanisms** — status: compiled; priority: baseline-now; path: [`wiki/diagnostics/overreleased-members-and-all-pin-mechanisms.md`](wiki/diagnostics/overreleased-members-and-all-pin-mechanisms.md); why: common model failure and steel frame issue.
- **Duplicate or coincident nodes** — status: missing; priority: baseline-next; intended path: `wiki/diagnostics/duplicate-and-coincident-nodes.md`; why: import/cleanup and solver instability diagnostic.
- **Ill-conditioning and stiffness contrast** — status: missing; priority: baseline-next; intended path: `wiki/diagnostics/ill-conditioning-and-stiffness-contrast.md`; why: needed for solver-result trust and artificial stiffness warnings.
- **Reaction sanity checks** — status: compiled; priority: baseline-now; path: [`wiki/diagnostics/reaction-sanity-checks.md`](wiki/diagnostics/reaction-sanity-checks.md); why: fast validation of load paths, supports, and result plausibility.
- **Analysis result review before design checks** — status: compiled; priority: baseline-now; path: [`wiki/diagnostics/analysis-result-review-before-design-checks.md`](wiki/diagnostics/analysis-result-review-before-design-checks.md); why: separates solver output quality from downstream design automation.

## Product and Pipeline Knowledge

- **Scheme generation from knowledge** — status: compiled; priority: baseline-now; path: [`wiki/product/scheme-generation-from-knowledge.md`](wiki/product/scheme-generation-from-knowledge.md); why: defines how compiled knowledge should guide agents without replacing deterministic artifacts.
- **Structural design option intelligence** — status: compiled; priority: baseline-now; path: [`wiki/product/structural-design-option-intelligence.md`](wiki/product/structural-design-option-intelligence.md); why: gives LLM-backed option generation a concept-design philosophy: distinct hypotheses, load-path intelligence, multi-criteria tradeoffs, constructability, robustness, and provenance.
- **Authored/resolved/run artifact boundaries** — status: compiled; priority: baseline-now; path: [`wiki/product/authored-resolved-run-boundaries.md`](wiki/product/authored-resolved-run-boundaries.md); why: core Fraia principle and recurrent Steward check.
- **Design actions, check inputs, and check results** — status: compiled; priority: baseline-now; path: [`wiki/product/design-actions-check-inputs-and-results.md`](wiki/product/design-actions-check-inputs-and-results.md); why: downstream steel checks need clean pipeline boundaries.
- **Engineering assumptions and provenance** — status: compiled; priority: baseline-now; path: [`wiki/product/engineering-assumptions-and-provenance.md`](wiki/product/engineering-assumptions-and-provenance.md); why: every generated scheme and diagnostic should preserve why assumptions exist.
- **Question prompts for missing structural context** — status: missing; priority: baseline-next; intended path: `wiki/product/question-prompts-for-missing-structural-context.md`; why: helps agents ask good questions instead of guessing.

## Later Materials

- **Concrete member behavior** — status: missing; priority: later; intended path: `wiki/materials/concrete/member-behavior.md`; why: important future material but steel is first material track.
- **Reinforced concrete slabs and walls** — status: missing; priority: later; intended path: `wiki/materials/concrete/slabs-and-walls.md`; why: later plate/shell and RC design direction.
- **Timber member behavior** — status: missing; priority: later; intended path: `wiki/materials/timber/member-behavior.md`; why: future material extension.
- **Foundations and soil-support assumptions** — status: missing; priority: later; intended path: `wiki/foundations/soil-support-assumptions.md`; why: necessary eventually, but first wedge can treat supports conceptually.

## Fraia-Specific Agent Guidance

- **Wiki maintenance workflow** — status: active; priority: baseline-now; path: [`workflow.md`](workflow.md); why: controls self-updating behavior, including the lint/reviewer and Fraia Knowledge Steward gates. This is a core policy file, not a compiled engineering wiki page.
- **Knowledge adapter contract** — status: active; priority: baseline-now; path: [`adapter-contract.md`](adapter-contract.md); why: lets third-party/maintainer ingestion systems feed Fraia without shipping ingestion plumbing.
- **Knowledge contributing guide** — status: active; priority: baseline-now; path: [`contributing.md`](contributing.md); why: documents community knowledge requests, source suggestions, corrections, and PR expectations.
- **Knowledge ingestion workflow** — status: active; priority: baseline-next; path: [`ingestion.md`](ingestion.md); why: optional maintainer/adapter guidance for temporary source reading, chunking, and synthesis; not app runtime scope.
- **Source registry** — status: active; priority: baseline-next; path: [`sources.md`](sources.md); why: bibliographic/source-governance aid; page-level sources remain mandatory.
- **Media policy** — status: active; priority: baseline-next; path: [`media/README.md`](media/README.md); why: governs committed diagrams/images and private-source screenshot handling.
- **Chunk manifest template** — status: active; priority: baseline-next; path: [`templates/chunk-manifest.md`](templates/chunk-manifest.md); why: keeps large-source reading bounded and context-safe.
- **Source learning packet template** — status: active; priority: baseline-now; path: [`templates/source-learning-packet.md`](templates/source-learning-packet.md); why: normalizes third-party/subagent learnings with original-source references.
- **Wiki update proposal template** — status: active; priority: baseline-now; path: [`templates/wiki-update-proposal.md`](templates/wiki-update-proposal.md); why: normalizes proposed compiled-page edits and source updates.
