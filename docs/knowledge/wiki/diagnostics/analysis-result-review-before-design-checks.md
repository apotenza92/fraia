---
title: Analysis Result Review Before Design Checks
status: compiled
trust_level: compiled
domain: diagnostics
applies_to:
  - post-analysis review
  - design-action extraction
  - Fraia agent guidance
not_applicable_to:
  - final check engine implementation
  - code-specific design pass/fail criteria
  - solver-specific workflows
jurisdiction_or_standard_context: Fraia diagnostic and pipeline guidance; not a code check
last_compiled: 2026-05-07
source_count: 3
citation_policy: required
owner: agent-maintained
---

# Analysis Result Review Before Design Checks

## Summary

Analysis results should be reviewed before Fraia extracts design actions or builds check inputs. A green-looking force diagram or solved run is not enough: agents should inspect solver status, warnings, stability, reactions, deflections, load cases/combinations, sign conventions, local axes, internal actions, and modeling assumptions first.

For Fraia, this review is a gate between immutable run artifacts and downstream design actions/check inputs.

## Scope / non-scope

This page covers concept-level analysis result review before design checks.

It does not define final review schemas, code-specific thresholds, check engine implementation, or solver UI workflows.

## Key concepts

### Design checks should not consume raw results blindly

Fraia's steel workflow separates analysis results, design actions, check inputs, and check results. [S1]

Design actions should be extracted only after the run result has enough trust for the intended check.

### Solver status and warnings matter

Instability mechanisms, singularities, artificial stabilization, convergence warnings, or diagnostic-only runs can undermine result trust. [S3]

Fraia agents should preserve warnings in the run artifact and block or downgrade checks when warnings affect the result path.

### Reactions are an early trust check

Reaction sanity checks compare applied actions, support reactions, support assumptions, coordinate frames, and load cases. Suspicious reactions can reveal missing loads, wrong supports, disconnected topology, or sign/coordinate issues. [S2]

Fraia should not proceed to foundation or member checks when reaction sanity is unresolved.

### Displacements and shapes should make physical sense

Large, implausible, or wrong-direction displacements can reveal mechanisms, stiffness errors, wrong support assumptions, incorrect units, or incorrect load directions.

Fraia should treat deformed shape review as a diagnostic, especially before second-order or stability-sensitive checks.

### Internal actions need context

Member forces, moments, and diagrams need local axes, sign conventions, stations, load cases/combinations, and release/support assumptions. A maximum value without context is not a design action. [S1]

Fraia agents should preserve extraction metadata before passing actions downstream.

### Review status should be provenance

The review conclusion should be tied to the run artifact: accepted for extraction, accepted with warnings, diagnostic only, needs model repair, or rejected.

Fraia should avoid silent promotion of unreviewed results.

## Engineering guidance for Fraia agents

- Before design-action extraction, check solver status, warnings, convergence, singularity/stabilization flags, and analysis method.
- Review reactions against loads and support assumptions.
- Review displacements/deformed shapes for plausible direction, magnitude, and mechanism signs.
- Review internal force diagrams for sign convention, local axes, discontinuities, releases, and load path plausibility.
- Confirm load cases/combinations, units, coordinate frames, and result stationing/envelopes.
- Map suspicious results back to authored loads, supports, releases, constraints, materials/sections, and resolved topology.
- Mark runs as diagnostic-only or not accepted when warnings or sanity checks are unresolved.
- Record review status/provenance before creating design actions or check inputs.

## Tradeoffs / cautions

- Manual/agent review adds a gate, but prevents false confidence in downstream checks.
- A run can solve numerically while being physically wrong.
- A result can be valid for one check purpose but insufficient for another, such as first-order strength versus second-order stability.
- Automated tolerances are useful but need engineering context.
- Check modules should report missing or untrusted review status rather than filling assumptions silently.

## Source-backed claims

- Fraia separates analysis results, design actions, check inputs, and check results. [S1]
- Reaction sanity checks are needed before trusting load paths and support demands. [S2]
- Instability mechanisms and diagnostic stabilizers can make run results untrustworthy for design checks. [S3]
- Design-action extraction should preserve load case, local axis, sign convention, stationing, and provenance. [S1]
- Suspicious solver warnings or reactions should block or downgrade downstream checks. [S2][S3]

## Open questions / weak evidence

- Fraia still needs final review-status schema, warning severity taxonomy, automated tolerance policy, and review UI.
- Result envelope extraction, station sampling, second-order result trust, and nonlinear convergence policy need future pages/modules.
- Human engineering review remains necessary before project approval.

## Related pages

- [Reaction sanity checks](reaction-sanity-checks.md)
- [Instability mechanisms](instability-mechanisms.md)
- [Unconnected or underrestrained models](unconnected-or-underrestrained-models.md)
- [Steel design action and check-input separation](../materials/steel/design-action-check-input-separation.md)
- [Beam shear and moment diagrams](../analysis/beam-shear-and-moment-diagrams.md)
- [Second-order effects and stability](../analysis/second-order-effects-and-stability.md)

## Sources

- [S1] Fraia compiled wiki, *Steel Design Action and Check-Input Separation*. Path: `docs/knowledge/wiki/materials/steel/design-action-check-input-separation.md`. Source type: Fraia compiled steel/product page. Consulted: 2026-05-07. Reliability/limits: useful Fraia pipeline guidance; final schemas remain future work.
- [S2] Fraia compiled wiki, *Reaction Sanity Checks*. Path: `docs/knowledge/wiki/diagnostics/reaction-sanity-checks.md`. Source type: Fraia compiled diagnostics page. Consulted: 2026-05-07. Reliability/limits: useful reaction-review guidance; final tolerances/report schemas remain future work.
- [S3] Fraia compiled wiki, *Instability Mechanisms*. Path: `docs/knowledge/wiki/diagnostics/instability-mechanisms.md`. Source type: Fraia compiled diagnostics page. Consulted: 2026-05-07. Reliability/limits: useful instability/result-trust guidance; includes source-scoped software/manual evidence.
