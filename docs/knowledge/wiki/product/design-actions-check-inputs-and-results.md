---
title: Design Actions, Check Inputs, and Check Results
status: compiled
trust_level: compiled
domain: product
applies_to:
  - Fraia design/check workflows
  - agent result explanations
  - report/export provenance
not_applicable_to:
  - final check schemas
  - code-specific formulas
  - check engine implementation
jurisdiction_or_standard_context: Fraia product architecture guidance; not a code check
last_compiled: 2026-05-07
source_count: 3
citation_policy: required
owner: agent-maintained
---

# Design Actions, Check Inputs, and Check Results

## Summary

Fraia should distinguish design actions, check inputs, and check results. These are related, but they are not interchangeable.

- A design action is an extracted/curated action from reviewed analysis results for a design purpose.
- A check input is the full context needed to evaluate a check.
- A check result is the output of check execution/evaluation.

This separation prevents agents from treating raw solver output or partially specified assumptions as finished engineering checks.

## Scope / non-scope

This page covers product-level vocabulary for Fraia design/check workflows.

It does not define final schemas, check engines, code formulas, jurisdiction modules, or report layouts.

## Key concepts

### Design actions are extracted from reviewed results

Fraia's downstream pipeline explicitly places design-action extraction after analysis results. [S1]

Design actions should reference the run artifact, load case/combination, extraction method, coordinate frame/local axis, sign convention, station/location, and review status.

### Check inputs add missing engineering context

Check inputs include design actions plus assumptions and data needed for a specific check: material, section, restraint, effective length, connection fixity, code scope, serviceability criteria, or other limit-state context.

Fraia should not hide check inputs inside check code or reports.

### Check results are generated artifacts

Check results are produced by check execution/evaluation. They should record status, governing mode, utilization/margin where applicable, source/code scope, assumptions, warnings, and provenance.

A check result should not overwrite the design action or authored model.

### Review gates matter

Analysis results should be reviewed before design-action extraction. Solver warnings, reaction sanity, instability, wrong sign conventions, or implausible displacements can make downstream checks untrustworthy. [S3]

Fraia should block, downgrade, or flag check packets when review status is missing or failed.

### Exports are renderers

Fraia's output pipeline says reports, spreadsheets, CSV, and CAD/detail outputs should render structured engineering data rather than become source of truth. [S1]

Exports should reference structured artifacts, not replace them.

## Engineering guidance for Fraia agents

- Use the terms analysis result, design action, check input, check result, and export precisely.
- Do not call a design action a pass/fail result.
- Do not call a check result valid unless required check inputs and review status are present.
- Preserve provenance from authored objects through resolved topology, run artifact, review gate, design action, check input, check result, and export.
- If check inputs are missing, say the check is incomplete rather than guessing.
- Keep code/jurisdiction scope explicit.
- Treat reports/spreadsheets as views of structured data.

## Tradeoffs / cautions

- More explicit artifacts add workflow structure, but reduce hidden assumptions.
- Early concept checks may use approximate inputs, but those assumptions must be visible.
- A check can be valid for one jurisdiction, load case, or limit state and invalid for another.
- Changing authored data or check inputs invalidates downstream check results but does not rewrite historical run artifacts.
- Human review remains necessary for project approval.

## Source-backed claims

- Fraia's downstream pipeline separates solve results, design actions, check inputs, check results, and exports. [S1]
- Authored, resolved, run, design/check, and export artifacts must remain distinct. [S2]
- Analysis result review should gate design-action extraction and check-input generation. [S3]
- Exports should render structured engineering data rather than become source of truth. [S1]
- Missing or unreviewed inputs should not become authoritative check results. [S2][S3]

## Open questions / weak evidence

- Final typed schemas for design actions, check inputs, check results, packets, warnings, and reports remain future work.
- Check module boundaries by material, member type, connection, and jurisdiction need future docs.
- Export provenance and report sign-off workflows need future implementation.

## Related pages

- [Authored/resolved/run artifact boundaries](authored-resolved-run-boundaries.md)
- [Steel design action and check-input separation](../materials/steel/design-action-check-input-separation.md)
- [Analysis result review before design checks](../diagnostics/analysis-result-review-before-design-checks.md)
- [Engineering assumptions and provenance](engineering-assumptions-and-provenance.md)

## Sources

- [S1] Fraia, *Engineering Output Pipeline*. Path: `docs/engineering-output-pipeline.md`. Source type: Fraia architecture doc. Consulted: 2026-05-07. Reliability/limits: canonical downstream pipeline direction; draft status and final artifact contracts remain future work.
- [S2] Fraia compiled wiki, *Authored/Resolved/Run Artifact Boundaries*. Path: `docs/knowledge/wiki/product/authored-resolved-run-boundaries.md`. Source type: Fraia compiled product page. Consulted: 2026-05-07. Reliability/limits: useful product-boundary synthesis; final schemas remain future work.
- [S3] Fraia compiled wiki, *Analysis Result Review Before Design Checks*. Path: `docs/knowledge/wiki/diagnostics/analysis-result-review-before-design-checks.md`. Source type: Fraia compiled diagnostics page. Consulted: 2026-05-07. Reliability/limits: useful review-gate guidance; final review status schema remains future work.
