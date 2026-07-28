---
title: Scheme Generation from Knowledge
status: compiled
trust_level: compiled
domain: product
applies_to:
  - Fraia LLM-backed scheme generation
  - Base Model Guide planning
  - design-option chat and review replies
not_applicable_to:
  - final design approval
  - code-specific checks
  - deterministic solver or check implementation
jurisdiction_or_standard_context: Fraia product and compiled-wiki guidance; not a code check
last_compiled: 2026-05-13
source_count: 5
citation_policy: required
owner: agent-maintained
---

# Scheme Generation from Knowledge

## Summary

Every LLM-backed Fraia flow that plans, generates, explains, or reviews structural schemes should consult relevant compiled wiki context before making structural recommendations.

Compiled wiki knowledge provides a reusable structural-engineering baseline. It does not approve a project, replace user intent, replace authored project artifacts, or stand in for analysis runs, design actions, check inputs, check results, or engineer review.

## Scope / non-scope

This page covers product-level rules for using compiled wiki excerpts in LLM-backed Fraia surfaces: Base Model Guide, design-option chat, review replies, pre-solve planning, and future LLM-backed recommendations.

It does not define final prompt schemas, retrieval rankings, UI design, deterministic validators, solver settings, final sizing heuristics, or jurisdiction-specific design checks.

## Key concepts

### Wiki context is required for LLM structural advice

LLM-backed structural recommendations should start from compiled wiki excerpts relevant to the current surface and task. Required context should include load paths, supports/restraints/releases, stability and bracing, authored/resolved/run boundaries, design-action/check-input separation, and assumptions/provenance. [S1][S2][S3][S4]

This rule applies to broad planning conversations and narrow review replies. A short GUI reply about loads, section families, or connection fixity still benefits from the same expert baseline.

### Project truth stays in project artifacts

Compiled wiki excerpts are guidance sources. Project-specific truth comes from authored project data, user confirmations, adopted design options, resolved/run artifacts, reviewed analysis results, design actions, check inputs, and check results. [S1][S2][S5]

An LLM may cite wiki-informed reasoning, but it should not describe that reasoning as project approval or code compliance.

### Design options should be group-first

For early steel and frame options, Fraia should normally describe section-family and size intent through coordination groups before discussing individual member sizing. This keeps fabrication, connection review, symmetry, repeated-role assumptions, and provenance visible before downstream analysis/design. [S1][S3][S5]

Exact member sizes should stay out of option chat unless a downstream solve/design/check artifact already produced them.

### Expert review criteria should be consistent

When generating, explaining, or reviewing design options, LLM-backed surfaces should check the same expert criteria:

- load path and target/action clarity
- stability system and bracing/restraint assumptions
- support/base fixity and release assumptions
- section-family tradeoffs and awkward family mixes
- connection buildability at key interfaces
- serviceability intent where relevant
- provenance and adoption state
- separation between authored state, resolved/run artifacts, design actions, check inputs, and check results

The LLM should ask for missing intent when these criteria materially affect the scheme.

### Deterministic logic remains deterministic

Retrieval-augmented prompts should improve explanations and recommendations, not hide deterministic rules inside prose. Validation, realization, solver preparation, analysis execution, design-action extraction, check-input construction, check evaluation, and exports should remain typed and inspectable. [S2][S3][S5]

## Engineering guidance for Fraia agents

- Always include compiled wiki excerpts in LLM prompts for planning, scheme, review, and future recommendation surfaces.
- Use core excerpts for every LLM surface: assumptions/provenance, authored/resolved/run boundaries, supports/restraints/releases, gravity/lateral loads, load paths, bracing/stability, and design-action/check-input separation.
- Add task-aware excerpts for design-option and review surfaces, especially load application, section families, connection fixity, coordination groups, and system-specific pages such as portal frames.
- Prefer coordination-group language for early options; avoid unnecessary unique member sizing.
- Flag weak grouping, unclear load paths, missing restraint/stability assumptions, unsupported support/base fixity, awkward family mixes, and missing provenance.
- Keep compiled wiki guidance distinct from adopted project assumptions and final checks.

## Tradeoffs / cautions

- Wiki-grounded prompts can make LLM advice more consistent, but retrieval should stay compact and inspectable.
- A relevant excerpt can guide questions and tradeoffs without being sufficient for final design.
- Over-specific heuristics should live in compiled wiki pages, archetype/builder modules, or downstream adapters, not generic runtime conditionals.
- Concept-stage options may compare plausible assumptions, but adoption into authored state should remain explicit.

## Source-backed claims

- Fraia agents should keep assumptions explicit, source-scoped, and distinct from project approval. [S1]
- Fraia separates authored project data, resolved runtime data, immutable run artifacts, design actions, check inputs, check results, and exports. [S2][S3]
- Supports, restraints, releases, loads, stability, and load paths are recurring baseline context for scheme generation and diagnostics. [S4]
- Generated or optimized options should remain candidate assumptions until explicitly adopted. [S1][S2]
- Coordination-oriented early steel options help keep section-family and buildability assumptions visible before final sizing. [S3][S5]

## Open questions / weak evidence

- Final prompt payload schemas, retrieval scoring, citation rendering, and UI affordances remain implementation work.
- Source-scoped preliminary sizing heuristics are still missing and should not be invented in generic app prompts.
- Human project approval and engineer-of-record workflows remain out of scope.

## Related pages

- [Structural design option intelligence](structural-design-option-intelligence.md)
- [Engineering assumptions and provenance](engineering-assumptions-and-provenance.md)
- [Authored/resolved/run artifact boundaries](authored-resolved-run-boundaries.md)
- [Design actions, check inputs, and check results](design-actions-check-inputs-and-results.md)
- [Load paths](../analysis/load-paths.md)
- [Supports, restraints, and releases](../modeling/supports-restraints-and-releases.md)
- [Gravity and lateral loads](../loads/gravity-and-lateral-loads.md)
- [Bracing principles](../stability/bracing-principles.md)
- [Steel material properties and section families](../materials/steel/material-properties-and-section-families.md)

## Sources

- [S1] Fraia compiled wiki, *Engineering Assumptions and Provenance*. Path: `docs/knowledge/wiki/product/engineering-assumptions-and-provenance.md`. Source type: Fraia compiled product page. Consulted: 2026-05-13. Reliability/limits: product guidance; final schemas and approval workflows remain future work.
- [S2] Fraia compiled wiki, *Authored/Resolved/Run Artifact Boundaries*. Path: `docs/knowledge/wiki/product/authored-resolved-run-boundaries.md`. Source type: Fraia compiled product page. Consulted: 2026-05-13. Reliability/limits: product-boundary synthesis; final artifact schemas remain future work.
- [S3] Fraia compiled wiki, *Design Actions, Check Inputs, and Check Results*. Path: `docs/knowledge/wiki/product/design-actions-check-inputs-and-results.md`. Source type: Fraia compiled product page. Consulted: 2026-05-13. Reliability/limits: downstream workflow guidance; check modules and schemas remain future work.
- [S4] Fraia Knowledge Topic Map. Path: `docs/knowledge/topic-map.md`. Source type: Fraia knowledge registry and roadmap. Consulted: 2026-05-13. Reliability/limits: coverage map and prioritisation aid; individual compiled pages remain the source-backed guidance.
- [S5] Fraia, *Engineering Output Pipeline*. Path: `docs/engineering-output-pipeline.md`. Source type: Fraia architecture doc. Consulted: 2026-05-13. Reliability/limits: canonical downstream pipeline direction; draft status and final artifact contracts remain future work.
