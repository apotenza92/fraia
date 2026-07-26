---
title: Static Determinacy and Restraint
status: compiled
trust_level: compiled
domain: analysis
applies_to:
  - concept-stage structural analysis explanations
  - support and release diagnostics
  - Fraia agent guidance
not_applicable_to:
  - final code-compliant design checks
  - automatic proof of 3D finite-element stability
  - project-specific engineering approval
jurisdiction_or_standard_context: concept guidance from academic/open sources; not a code check
last_compiled: 2026-05-07
source_count: 3
citation_policy: required
owner: agent-maintained
---

# Static Determinacy and Restraint

## Summary

Static determinacy, indeterminacy, and restraint sufficiency are related but separate analysis concepts. A determinate model can be solved from equilibrium equations alone. An indeterminate model needs compatibility, deformation, stiffness, or other additional relationships. A stable model must also have restraints and topology that prevent rigid-body motion or mechanisms under the intended analysis assumptions.

For Fraia agents, determinacy should be treated as a model-classification and explanation tool, not as a complete solver-validity test.

## Scope / non-scope

This page covers concept-level guidance for explaining static determinacy, support reactions, restraint sufficiency, and mechanism risk in Fraia analysis workflows.

It does not define final code checks, member sizing, foundation design, nonlinear stability, second-order buckling analysis, or a complete finite-element mechanism detector.

## Key concepts

### Equilibrium equations are necessary but not always sufficient

Planar equilibrium gives force balance in two directions and moment balance about the out-of-plane axis. Spatial equilibrium adds the corresponding three force and three moment balance equations. These equations are the starting point for classifying simple beams, frames, trusses, and support reactions. [S1]

Equilibrium counts alone should not be treated as proof that a Fraia model is valid. The count depends on the resolved topology, support and release idealisations, internal hinges, member connectivity, and whether the model is 2D or 3D.

### Determinacy is about what equilibrium can solve

A statically determinate structure has enough independent equilibrium equations to solve its unknown reactions or internal forces under the adopted idealisation. A statically indeterminate structure has more unknowns than equilibrium alone can resolve, so compatibility/deformation and stiffness relationships are needed. [S1][S3]

Indeterminacy is not an error by itself. Many real structural systems are intentionally redundant. The error is pretending that equilibrium-only reasoning has solved a model that actually needs stiffness/compatibility analysis.

### Stability is a separate restraint question

Stability asks whether the supports, internal connections, and member arrangement prevent the model from moving as a rigid body or mechanism under the intended analysis assumptions. A model can fail because it is underrestrained, because restraint directions are ineffective, or because releases/connectivity create a mechanism even when a simple reaction count looks plausible. [S1][S2]

For Fraia, this means determinacy classification should feed diagnostics, not replace them.

### Support names must resolve to DOFs

Support names such as pin, roller, link, or fixed support are shorthand for restrained motions and reaction components. Fraia should store and explain the actual restrained degrees of freedom, coordinate frame, support plane/direction, and provenance behind a named `SupportAssignment`. [S1][S2]

### External and internal indeterminacy depend on decomposition

External indeterminacy can be reasoned from support reactions, connection forces, and equilibrium equations for free bodies. Internal indeterminacy requires splitting the structure into joints and members, drawing free bodies, counting support reactions and section forces, and comparing them with available member/joint equilibrium equations. [S3]

Fraia should use this as an explanation pattern, not a universal hardcoded formula. Authored `Member` and `Plate` objects may resolve into analysis nodes/elements, internal releases, constraints, and solver-specific topology before a run.

## Engineering guidance for Fraia agents

- When diagnosing a model, ask three separate questions: are the reaction/force unknowns solvable by equilibrium, is the model stable against mechanisms, and what additional stiffness/compatibility assumptions are required?
- Keep authored `SupportAssignment` and `ReleaseAssignment` objects distinct from resolved analysis restraints, constraints, internal hinges, and solver equations.
- Explain support presets by their restrained DOFs and reaction components. Do not rely on support icon names alone.
- Treat simple determinacy formulas as source-scoped checks for simple idealised structures. For general 3D or finite-element models, use them only as diagnostic hints.
- When a member is split during realization, describe the authored role-labelled member as discretised into analysis elements. Do not call each split element a separate beam or column unless it is authored that way.
- If the resolved model is indeterminate, say which additional information is required: compatibility, stiffness, deformation assumptions, or solver analysis.
- If the resolved model is unstable, map the issue back to authored supports, releases, connectivity, constraints, or missing stabilizing members before suggesting changes.

## Tradeoffs / cautions

- A determinate model can be easier to explain, but it may be less redundant and may not represent the intended real structure.
- An indeterminate model is normal for many structural systems, but it requires stiffness, compatibility, and solver assumptions that must be inspectable.
- Too few effective restraints can create rigid-body modes; too many or poorly represented restraints can hide intended releases, overconstrain the model, or create misleading reaction paths.
- Counting reactions and equations is useful for simple models, but it does not catch every geometric mechanism, local release pattern, disconnected node, 3D torsional freedom, or finite-element modeling error.
- Private textbooks can be useful corroboration for maintainers, but compiled public pages should prefer auditable public/open sources where they are adequate.

## Source-backed claims

- Planar structural equilibrium uses two force-balance equations and one moment-balance equation; spatial equilibrium uses the corresponding six force/moment conditions. [S1]
- Support idealisations control restrained movements and therefore support reaction components. [S1][S2]
- A determinate structure can be solved by equilibrium alone, while an indeterminate structure requires compatibility or deformation/stiffness information in addition to equilibrium. [S1][S3]
- Stability must be considered separately from determinacy because a structure must preserve its geometry and avoid mechanisms under load. [S1][S2]
- External and internal indeterminacy counting can be framed through free-body decomposition, support reactions, connection forces, section forces, and available equilibrium equations. [S3]

## Open questions / weak evidence

- Fraia still needs a concrete algorithm for detecting mechanisms in resolved 3D analysis topology.
- The page intentionally avoids universal formula tables for all trusses/frames/plates because those depend on idealisation and topology.
- Private textbook corroboration was not added to the compiled source list in this run because page/section inspection was not available without adding temporary PDF extraction tooling.

## Related pages

- [Load paths](load-paths.md)
- [Free-body diagrams and equilibrium](free-body-diagrams-and-equilibrium.md)
- [Reactions and support idealisation](reactions-and-support-idealisation.md)
- [Truss analysis and two-force members](truss-analysis-and-two-force-members.md)
- [Second-order effects and stability](second-order-effects-and-stability.md)
- [Finite-element idealisation](../modeling/finite-element-idealisation.md)
- [Supports, restraints, and releases](../modeling/supports-restraints-and-releases.md)
- [Instability mechanisms](../diagnostics/instability-mechanisms.md)

## Sources

- [S1] Felix Udoeyo / Engineering LibreTexts, *Equilibrium Structures, Support Reactions, Determinacy and Stability of Beams and Frames*. URL: https://eng.libretexts.org/Bookshelves/Civil_Engineering/Structural_Analysis_(Udoeyo)/01:_Chapters/1.03:_Equilibrium_Structures_Support_Reactions_Determinacy_and_Stability_of_Beams_and_Frames. Source type: open educational structural-analysis text. Retrieved: 2026-05-07. Reliability/limits: useful for equilibrium, support characteristics, and simple beam/frame determinacy classification; introductory and not a code check.
- [S2] Daniel W. Baker and William Haynes / Engineering LibreTexts, *Stability and Determinacy*. URL: https://eng.libretexts.org/Bookshelves/Mechanical_Engineering/Engineering_Statics:_Open_and_Interactive_(Baker_and_Haynes)/05:_Rigid_Body_Equilibrium/5.06:_Stability_and_Determinacy. Source type: open educational statics text. Retrieved: 2026-05-07. Reliability/limits: strong rigid-body support/restraint framing; not a full structural-analysis design reference.
- [S3] Tom van Woudenberg / Delft University of Technology, *Static Indeterminacy*. URL: https://oit.tudelft.nl/CT1000/2025/_git/github.com_TUDelft-books_CEG-mechanics-BSc/EN/book/statically_inderminate/determinancy.html. Source type: university open course notes. Retrieved: 2026-05-07. Reliability/limits: useful free-body/counting procedure for external and internal static indeterminacy; course-scoped examples and not a universal model-validity algorithm.
