---
title: Reaction Sanity Checks
status: compiled
trust_level: compiled
domain: diagnostics
applies_to:
  - post-analysis reaction review
  - load-path and support diagnostics
  - Fraia agent guidance
not_applicable_to:
  - foundation design
  - support capacity checks
  - final reaction report schema
jurisdiction_or_standard_context: concept guidance from Fraia compiled pages; not a code check
last_compiled: 2026-05-07
source_count: 3
citation_policy: required
owner: agent-maintained
---

# Reaction Sanity Checks

## Summary

Reaction sanity checks compare support reactions against the applied actions, support assumptions, free-body selection, coordinate frame, and load case/combination. They are a fast way to catch missing loads, wrong supports, sign mistakes, disconnected topology, unintended uplift, and implausible load paths before steel or foundation checks are trusted.

For Fraia, reaction sanity checks should be tied to immutable run artifacts and mapped back to authored `SupportAssignment`, `LoadAssignment`, coordinate frames, and resolved topology.

## Scope / non-scope

This page covers concept-level reaction sanity-check guidance for Fraia agents.

It does not provide foundation design, capacity checks, final reporting schemas, or a replacement for deterministic analysis.

## Key concepts

### Define the body and case before comparing

Free-body and equilibrium reasoning require a clearly isolated body, applied loads, unknown reactions, coordinate frame, and sign convention. [S2]

Fraia reaction checks should state whether the body is the whole model, a submodel, a frame slice, a support group, or another isolated system.

### Reactions must match support DOFs

Support reactions arise where restrained or prescribed support DOFs enforce the adopted support idealisation. [S1]

Fraia should not expect a reaction component in an unrestrained direction, and should flag reactions that contradict the physical support assumption.

### Total balance is necessary but not sufficient

For a stable whole-body static model, total applied actions and reactions should satisfy force/moment equilibrium under the chosen convention. [S1][S2]

Passing global balance does not prove local connectivity, release patterns, stiffness, load application, or design adequacy.

### Applied load totals must include realized loads

Loads applied as member, line, area, pressure, or distributed loads may be converted into solver-ready/equivalent nodal loads. Reaction sanity checks must use the resolved load representation for the run being checked. [S3]

Fraia should compare reactions to the load case/combination actually solved.

### Reaction direction is diagnostic

A negative reaction can simply mean the actual direction is opposite the assumed positive direction. It becomes a physical issue when it violates support intent, such as compression-only bearing, uplift limits, or unilateral contact. [S1]

Fraia agents should report sign and physical interpretation separately.

## Engineering guidance for Fraia agents

- Always state load case/combination, coordinate frame, sign convention, and body being checked.
- Compare total applied forces/moments and total reactions for the selected body.
- Include equivalent nodal loads, member loads, area/plate loads, applied moments, imposed displacements, and self-weight where active.
- Check reaction components against support DOFs and physical support assumptions.
- Flag unexpected zero reactions, unexpected uplift, large moment reactions, wrong-direction reactions, and asymmetric reactions when the model/load path should be symmetric.
- Do not treat a balanced reaction summary as proof that local model topology or design checks are valid.
- Link suspicious reactions to authored supports, loads, releases, constraints, and resolved topology.

## Tradeoffs / cautions

- Whole-model reaction checks are fast and valuable, but can miss local disconnected objects and release mechanisms.
- Symmetry checks are useful only when geometry, loads, supports, stiffness, and combinations are actually symmetric.
- Equivalent nodal load totals must be checked in the same coordinate system and units as reactions.
- Support settlements, imposed displacements, springs, and nonlinear supports can make reaction interpretation less intuitive.
- Foundation demand summaries are downstream artifacts, not raw reaction checks.

## Source-backed claims

- Support reactions are tied to restrained or prescribed support DOFs. [S1]
- Equilibrium checks require force and moment balance for the isolated body. [S2]
- Reaction sign should be interpreted against the chosen positive direction and physical support assumption. [S1]
- Resolved/equivalent loads must be part of the action total used for reaction checks. [S3]
- Global reaction balance is useful but not enough to prove stability, connectivity, or design adequacy. [S1][S2]

## Open questions / weak evidence

- Fraia still needs final reaction-summary schema, unit handling, coordinate-frame reporting, tolerance policy, and symmetry/load-path diagnostic rules.
- Nonlinear supports, uplift-only contact, staged construction, and imposed settlements need future pages/check logic.
- Foundation design and support capacity checks are out of scope.

## Related pages

- [Reactions and support idealisation](../analysis/reactions-and-support-idealisation.md)
- [Free-body diagrams and equilibrium](../analysis/free-body-diagrams-and-equilibrium.md)
- [Load application and equivalent nodal loads](../loads/load-application-and-equivalent-nodal-loads.md)
- [Load paths](../analysis/load-paths.md)
- [Unconnected or underrestrained models](unconnected-or-underrestrained-models.md)
- [Analysis result review before design checks](analysis-result-review-before-design-checks.md)

## Sources

- [S1] Fraia compiled wiki, *Reactions and Support Idealisation*. Path: `docs/knowledge/wiki/analysis/reactions-and-support-idealisation.md`. Source type: Fraia compiled analysis page. Consulted: 2026-05-07. Reliability/limits: useful Fraia-specific reaction/support synthesis; inherits source limits from its page.
- [S2] Fraia compiled wiki, *Free-Body Diagrams and Equilibrium*. Path: `docs/knowledge/wiki/analysis/free-body-diagrams-and-equilibrium.md`. Source type: Fraia compiled analysis page. Consulted: 2026-05-07. Reliability/limits: useful free-body/equilibrium basis; not a reaction report schema.
- [S3] Fraia compiled wiki, *Load Application and Equivalent Nodal Loads*. Path: `docs/knowledge/wiki/loads/load-application-and-equivalent-nodal-loads.md`. Source type: Fraia compiled loads page. Consulted: 2026-05-07. Reliability/limits: useful resolved-load provenance context; not a complete solver load-vector specification.
