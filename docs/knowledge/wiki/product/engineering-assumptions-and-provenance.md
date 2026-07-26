---
title: Engineering Assumptions and Provenance
status: compiled
trust_level: compiled
domain: product
applies_to:
  - Fraia agent guidance
  - scheme generation and diagnostics
  - run/check/export provenance
not_applicable_to:
  - final provenance schema
  - UI design
  - project engineering approval
jurisdiction_or_standard_context: Fraia product and knowledge workflow guidance; not a code check
last_compiled: 2026-05-07
source_count: 3
citation_policy: required
owner: agent-maintained
---

# Engineering Assumptions and Provenance

## Summary

Fraia agents should make engineering assumptions explicit, scoped, and traceable. An assumption should say what it affects, why it exists, where it came from, how confident it is, and which authored/resolved/run/design/check/export artifact uses it.

Provenance is not paperwork. It is how Fraia avoids hidden defaults, non-reproducible runs, and unsupported design claims.

## Scope / non-scope

This page covers product-level guidance for recording engineering assumptions and provenance in Fraia workflows.

It does not define final metadata schemas, UI design, approval workflows, code checks, or legal sign-off.

## Key concepts

### Assumptions belong to a layer

An assumption may affect authored project data, resolved topology, solver/run settings, design-action extraction, check inputs, check results, or exports. Fraia's architecture separates these layers, so provenance should identify the layer affected. [S1][S2]

Agents should not place every assumption in free-form narrative only.

### Source scope matters

An assumption can come from user input, project brief, standards, professional guidance, academic sources, private textbooks, software manuals, compiled wiki pages, or agent inference. Each source type has different confidence and scope.

Fraia should preserve source type, locator, date, and limits.

### Confidence is part of the artifact

Assumptions should carry confidence or review status when they influence engineering outputs. A source-backed assumption, user-confirmed assumption, placeholder/default assumption, and agent inference should not look identical.

Missing or weak assumptions should ask for more context or downgrade downstream checks.

### Provenance follows the pipeline

Fraia's output pipeline keeps structured data through design actions, check inputs, check results, and exports. [S2]

Provenance should flow with these artifacts so a report or spreadsheet can be traced back to the run, check inputs, authored objects, and assumptions used.

### Knowledge provenance is separate from project provenance

The Fraia knowledge wiki has its own workflow: source learning packet, wiki update proposal, lint/reviewer, Steward review, and promotion decision. [S3]

Project assumptions may cite compiled wiki guidance, but compiled wiki guidance should not be treated as project-specific approval.

### Candidate options need adoption

Generated or optimized options should not silently overwrite authored state; they should become authored state only through explicit adoption. [S1]

Agents should label assumptions for candidates separately from accepted project assumptions.

## Engineering guidance for Fraia agents

- Record assumptions with affected layer, affected object(s), source/provenance, confidence, date/agent/user, and scope/limits.
- Distinguish user-provided assumptions, source-backed assumptions, inferred assumptions, defaults, and unknowns.
- Do not silently fill missing structural context when the assumption affects stability, load path, design actions, or checks.
- Preserve provenance from authored objects through resolved topology, run artifacts, design actions, check inputs, check results, and exports.
- Treat compiled wiki pages as guidance sources, not project-specific approval.
- When using private textbooks or software manuals, cite logical locators and limitations; do not copy source text/images.
- Keep candidate/generation assumptions separate until the user adopts them into authored state.

## Tradeoffs / cautions

- More provenance metadata can feel heavy, but it prevents hidden design authority.
- Some assumptions are acceptable at concept stage but not for final design.
- Source-backed does not mean universally valid; jurisdiction, material, member type, and project context matter.
- Agent inference should be labeled as inference.
- Exports without provenance can become misleading even when underlying structured artifacts are sound.

## Source-backed claims

- Fraia requires separation between authored project data, resolved runtime data, and immutable run snapshots with provenance metadata. [S1]
- Fraia's downstream outputs should render structured engineering data and preserve provenance. [S2]
- Generated or optimized options should not silently overwrite authored project state. [S1]
- Fraia knowledge updates require source packet/proposal, lint/reviewer, and Steward review workflow gates. [S3]
- Source/confidence risk should be called out during Steward review. [S3]

## Open questions / weak evidence

- Final provenance schema, confidence vocabulary, assumption UX, and report/export rendering remain future work.
- Project approval, engineer-of-record workflows, and legal sign-off are out of scope.
- Integration between knowledge provenance and project/run provenance needs future implementation.

## Related pages

- [Authored/resolved/run artifact boundaries](authored-resolved-run-boundaries.md)
- [Design actions, check inputs, and check results](design-actions-check-inputs-and-results.md)
- [Analysis result review before design checks](../diagnostics/analysis-result-review-before-design-checks.md)
- [Steel design action and check-input separation](../materials/steel/design-action-check-input-separation.md)
- [Knowledge workflow](../../workflow.md)

## Sources

- [S1] Fraia, *Resolution and Runs*. Path: `docs/resolution-and-runs.md`. Source type: Fraia architecture doc. Consulted: 2026-05-07. Reliability/limits: canonical authored/resolved/run architecture direction; draft status and final schemas remain future work.
- [S2] Fraia, *Engineering Output Pipeline*. Path: `docs/engineering-output-pipeline.md`. Source type: Fraia architecture doc. Consulted: 2026-05-07. Reliability/limits: canonical downstream pipeline direction; draft status and final artifact contracts remain future work.
- [S3] Fraia knowledge docs, *Knowledge Wiki Workflow*. Path: `docs/knowledge/workflow.md`. Source type: Fraia knowledge workflow doc. Consulted: 2026-05-07. Reliability/limits: canonical wiki maintenance workflow; product/project provenance schemas remain future work.
