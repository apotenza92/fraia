---
title: Authored/Resolved/Run Artifact Boundaries
status: compiled
trust_level: compiled
domain: product
applies_to:
  - Fraia agent guidance
  - model diagnostics and repair suggestions
  - design-action/check workflows
not_applicable_to:
  - final schema implementation
  - UI design details
  - jurisdiction-specific engineering checks
jurisdiction_or_standard_context: Fraia product architecture guidance; not a code check
last_compiled: 2026-05-07
source_count: 3
citation_policy: required
owner: agent-maintained
---

# Authored/Resolved/Run Artifact Boundaries

## Summary

Fraia separates authored project data, resolved runtime data, and immutable run artifacts. This is a core product boundary, not an implementation detail. Agents should preserve it when explaining models, proposing repairs, extracting design actions, or interpreting analysis/check results.

The authored model can evolve. A run artifact records what actually happened for one specific resolved model and must remain immutable.

## Scope / non-scope

This page covers product-level guidance for Fraia agents using the compiled wiki.

It does not define final file schemas, API contracts, UI behavior, migration tooling, or jurisdiction-specific engineering checks.

## Key concepts

### Authored data is the project-facing truth

Fraia's authoring representation is compact, human-readable, agent-readable, package-referencing, and suitable for long-term editing. It contains the project-facing objects and assumptions users can review and change. [S1]

Fraia agents should make proposed model changes at the authored layer unless explicitly inspecting downstream artifacts.

### Resolved data is derived and normalized

Resolved runtime data expands package references, archetypes, placements, frames, effective parameters, defaults, inherited rules, units, and active analysis requests. [S1]

Resolved data is useful for inspection/debugging and solver preparation, but it should not become the only engineering truth.

### Run artifacts are immutable evidence

Frozen run snapshots should include the exact resolved model, rules, package versions, solver adapter/settings, outputs/results/logs, and provenance metadata for that run. [S1]

Fraia should preserve failed and diagnostic runs rather than overwriting them with later fixes.

### Downstream outputs are renderers

Fraia's output pipeline treats reports, CSV, XLSX, and CAD/detail outputs as renderers of structured engineering data, not sources of truth. [S2]

Agents should not use an exported report or spreadsheet as the canonical project state when the structured artifacts exist.

### Design/check artifacts sit downstream

Fraia separates analysis results, design actions, check inputs, check results, and exports. [S2][S3]

This means a steel check result should reference the run and check inputs that produced it, not rewrite the authored member or material assumptions silently.

## Engineering guidance for Fraia agents

- Work at authored abstractions first: `Node`, `Member`, `Plate`, `SupportAssignment`, `LoadAssignment`, and `ReleaseAssignment`.
- Inspect resolved topology or run artifacts when diagnosing solver behavior, checking provenance, or explaining results.
- Do not silently overwrite authored project state from generated options or diagnostics.
- If proposing a repair, state whether it changes authored data, resolution rules, solver settings, or only a diagnostic run.
- Treat run artifacts as immutable evidence tied to exact resolved inputs.
- Keep design actions, check inputs, check results, reports, and exports downstream from authored/resolved/run artifacts.
- When citing wiki guidance, map it to the appropriate layer rather than collapsing layers.

## Tradeoffs / cautions

- Layer separation adds structure, but prevents hidden state and non-reproducible results.
- Agents may need resolved inspection for diagnostics, but should return proposed changes to authored objects.
- A check result can be invalidated by authored model changes without changing the historical run artifact.
- Exports are useful for engineers but should not become the source of truth.
- Generated options should be adoptable, inspectable candidates, not silent mutations.

## Source-backed claims

- Fraia needs clean separation between authored project data, resolved runtime data, and immutable run snapshots. [S1]
- Frozen run snapshots should record exact resolved model, rules, versions, solver settings, outputs/results/logs, and provenance. [S1]
- Fraia's downstream path separates builder graph, structural model, realization, solve result, design actions, check inputs, check results, and exports. [S2]
- Outputs such as reports, CSV, XLSX, and CAD/detail artifacts should render structured engineering data rather than become source of truth. [S2]
- Steel workflows should separate analysis results, design actions, check inputs, and check results. [S3]

## Open questions / weak evidence

- Final JSON schemas and persisted artifact names remain future work.
- Exact diff/inspection UX between authored, resolved, and run states is unresolved.
- Check packet and export schemas need future implementation docs.

## Related pages

- [Design actions, check inputs, and check results](design-actions-check-inputs-and-results.md)
- [Engineering assumptions and provenance](engineering-assumptions-and-provenance.md)
- [Steel design action and check-input separation](../materials/steel/design-action-check-input-separation.md)
- [Analysis result review before design checks](../diagnostics/analysis-result-review-before-design-checks.md)
- [Finite-element idealisation](../modeling/finite-element-idealisation.md)
- [Matrix stiffness method](../analysis/matrix-stiffness-method.md)
- [Unconnected or underrestrained models](../diagnostics/unconnected-or-underrestrained-models.md)

## Sources

- [S1] Fraia, *Resolution and Runs*. Path: `docs/resolution-and-runs.md`. Source type: Fraia architecture doc. Consulted: 2026-05-07. Reliability/limits: canonical authored/resolved/run architecture direction; draft status and final schemas remain future work.
- [S2] Fraia, *Engineering Output Pipeline*. Path: `docs/engineering-output-pipeline.md`. Source type: Fraia architecture doc. Consulted: 2026-05-07. Reliability/limits: canonical downstream pipeline direction; draft status and final artifact contracts remain future work.
- [S3] Fraia compiled wiki, *Steel Design Action and Check-Input Separation*. Path: `docs/knowledge/wiki/materials/steel/design-action-check-input-separation.md`. Source type: Fraia compiled page. Consulted: 2026-05-07. Reliability/limits: useful steel workflow synthesis; exact schemas remain future work.
