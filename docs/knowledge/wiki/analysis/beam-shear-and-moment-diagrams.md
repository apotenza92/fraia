---
title: Beam Shear and Moment Diagrams
status: compiled
trust_level: compiled
domain: analysis
applies_to:
  - beam and frame result interpretation
  - load-path and reaction sanity checks
  - Fraia agent guidance
not_applicable_to:
  - code-compliant steel or concrete design checks
  - plate or shell stress-resultant diagrams
  - nonlinear second-order analysis
jurisdiction_or_standard_context: concept guidance from academic/open sources; not a code check
last_compiled: 2026-05-07
source_count: 3
citation_policy: required
owner: agent-maintained
---

# Beam Shear and Moment Diagrams

## Summary

Shear force and bending moment diagrams show how internal shear and bending moment vary along a beam or frame member under a particular load case, combination, support idealisation, release pattern, and sign convention. They are interpretation artifacts: useful for checking load paths, locating governing regions, and preparing downstream design actions, but not design checks by themselves.

For Fraia, diagrams should be tied to resolved analysis results or explicitly scoped hand-equilibrium explanations. They should not overwrite authored `Member`, `LoadAssignment`, `SupportAssignment`, or `ReleaseAssignment` intent.

## Scope / non-scope

This page covers concept-level beam shear and moment diagram semantics for Fraia agents.

It does not provide worked hand examples, code capacity checks, nonlinear moment magnification, plate/shell resultants, or a final Fraia UI/storage design.

## Key concepts

### Diagrams represent internal actions along a member

When beams or frames are subjected to transverse loads, internal normal force, shear force, and bending moment can develop at member sections. Shear and bending moment diagrams graph the variation of shear force and bending moment along the length of the member or member segment. [S1][S2]

Fraia agents should treat these diagrams as result views tied to a specific analysis context, not generic properties of a member.

### Cut-section equilibrium exposes internal actions

Internal shear and bending moment are found by cutting the member at a section and enforcing equilibrium of one side of the cut. The cut section receives internal force and moment resultants needed to keep the isolated portion in equilibrium. [S1][S2]

This connects diagrams directly to free-body reasoning: a diagram value only has meaning when the cut location, side of cut, local axis, sign convention, and load case are known.

### Sign convention must be explicit

Shear and bending moment signs depend on the convention used for the cut face, local axes, and diagram display. Different texts and software may draw positive moment above or below a member axis. [S1][S2]

Fraia should store and display diagram values with local axis and sign-convention metadata so downstream explanations and check inputs do not silently flip signs.

### Loads, shear, and moment are related

Distributed load intensity, shear force, and bending moment are locally related: distributed load changes shear, and shear changes bending moment. Equivalently, changes in shear are associated with the area under the load diagram, and changes in moment are associated with the area under the shear diagram, subject to sign convention. [S3]

This relationship is useful for sanity checks. Constant distributed load gives linear shear and curved moment; zero distributed load gives constant shear and linear moment.

### Concentrated actions create discontinuities

Concentrated forces create jumps in the shear diagram equal to the concentrated force magnitude under the chosen convention. Concentrated moments create jumps in the moment diagram. Distributed loads change diagram slope rather than creating a point jump. [S1][S3]

Fraia should use discontinuities as clues to load application, support reactions, point loads, member releases, and modeling discontinuities.

### Maximum moment usually relates to zero shear

For continuous regions without a moment discontinuity, a local maximum or minimum bending moment occurs where shear is zero or changes sign. Endpoints, supports, point moments, and discontinuities must still be checked. [S1][S3]

Fraia should avoid claiming a governing moment from a single diagram rule without checking the actual sampled/enveloped result data.

## Engineering guidance for Fraia agents

- Tie every diagram or internal action claim to a load case/combination, authored `Member`, resolved analysis topology, local axis, and sign convention.
- Distinguish authored members from analysis elements when a member is discretised.
- Use diagrams to explain load path and governing regions, but do not treat them as final design checks.
- When a diagram looks wrong, first check reactions, load targets, local axes, releases, connectivity, support assumptions, and load combinations.
- Do not compare shear/moment signs between sources, solvers, or UI displays unless their conventions are aligned.
- Preserve provenance from `LoadAssignment` and `SupportAssignment` through resolved loads/reactions to recovered internal actions and downstream design actions.
- For steel checks, pass design actions with context: member role/id, station, local axis, load combination, sign convention, and whether values are enveloped.

## Tradeoffs / cautions

- Diagrams are excellent diagnostic views, but clean-looking diagrams can still come from wrong supports, wrong loads, or wrong local axes.
- Hand-equilibrium diagrams are useful for determinate/simple cases; indeterminate and continuous members generally need stiffness/compatibility analysis.
- Solver diagrams may be sampled or interpolated between analysis nodes, so peak values can be missed if diagram recovery is too coarse.
- Member releases can create moment discontinuities or zeros that are correct under the model but wrong if the release was unintended.
- A maximum absolute diagram value is not automatically the governing design condition without checking combinations, member capacity mode, stability, and serviceability.

## Source-backed claims

- Shear force and bending moment diagrams graph internal shear and bending moment variation along beams or frames. [S1][S2]
- Cut-section equilibrium exposes the internal shear force and bending moment needed to maintain equilibrium of the isolated portion. [S1][S2]
- Distributed load, shear, and moment are related by local derivative/integral relationships. [S3]
- Concentrated loads cause jumps in shear diagrams under the chosen convention. [S1]
- Maximum bending moment in a smooth region occurs where shear is zero, with discontinuities/endpoints requiring separate checks. [S1][S3]

## Open questions / weak evidence

- Fraia still needs final result schemas for diagram stationing, interpolation, envelopes, and local-axis display.
- Plate/shell resultants and frame torsion diagrams need separate coverage.
- Code-specific design-action extraction for steel beams is deferred to steel/check-input pages.

## Related pages

- [Free-body diagrams and equilibrium](free-body-diagrams-and-equilibrium.md)
- [Reactions and support idealisation](reactions-and-support-idealisation.md)
- [Truss analysis and two-force members](truss-analysis-and-two-force-members.md)
- [Second-order effects and stability](second-order-effects-and-stability.md)
- [Load application and equivalent nodal loads](../loads/load-application-and-equivalent-nodal-loads.md)
- [Local and global coordinate systems](../modeling/local-and-global-coordinate-systems.md)
- [Steel member behavior](../materials/steel/member-behavior.md)

## Sources

- [S1] Felix Udoeyo / Engineering LibreTexts, *Internal Forces in Beams and Frames*. URL: https://eng.libretexts.org/Bookshelves/Civil_Engineering/Structural_Analysis_(Udoeyo)/01:_Chapters/1.04:_Internal_Forces_in_Beams_and_Frames. Source type: open educational structural-analysis text. Retrieved: 2026-05-07. Reliability/limits: useful beam/frame internal-force definitions and diagram behavior; introductory and not a design-code source.
- [S2] David Roylance / Engineering LibreTexts, *Shear and Bending Moment Diagrams*. URL: https://eng.libretexts.org/Bookshelves/Mechanical_Engineering/Mechanics_of_Materials_(Roylance)/04:_Bending/4.01:_Shear_and_Bending_Moment_Diagrams. Source type: open educational mechanics text derived from MIT materials. Retrieved: 2026-05-07. Reliability/limits: strong mechanics-of-materials explanation of cut-section equilibrium; not structural-code guidance.
- [S3] Gayla Osgood, Libby Osgood, Jeffery Cameron, and James Christensen / Engineering LibreTexts, *Shear/Moment Diagrams*. URL: https://eng.libretexts.org/Bookshelves/Mechanical_Engineering/Engineering_Mechanics_-_Statics_(Osgood_Cameron_and_Christensen)/06:_Internal_Forces/6.02:_Shear_Moment_Diagrams. Source type: open educational statics text. Retrieved: 2026-05-07. Reliability/limits: useful load-shear-moment relationship guidance; introductory and partially adapted from Udoeyo.
