---
title: Free-Body Diagrams and Equilibrium
status: compiled
trust_level: compiled
domain: analysis
applies_to:
  - concept-stage structural analysis explanations
  - reaction and load sanity checks
  - Fraia agent guidance
not_applicable_to:
  - full truss analysis procedure
  - shear and moment diagram procedure
  - dynamic analysis
jurisdiction_or_standard_context: concept guidance from academic/open sources; not a code check
last_compiled: 2026-05-07
source_count: 3
citation_policy: required
owner: agent-maintained
---

# Free-Body Diagrams and Equilibrium

## Summary

A free-body diagram isolates the object being analyzed and shows the external loads, support reactions, moments, dimensions, and coordinate frame needed to write equilibrium equations. For Fraia, it is a reasoning tool for explaining reactions, load paths, internal actions, and model diagnostics. It is not a replacement for the authored structural model or the resolved analysis model.

Fraia agents should use free-body reasoning before claiming that reactions, member forces, or stability explanations make sense.

## Scope / non-scope

This page covers concept-level free-body and equilibrium guidance for Fraia agents.

It does not provide detailed truss analysis, method of sections, shear/moment diagram construction, influence lines, or dynamics. Those belong in separate pages.

## Key concepts

### Isolate the body being analyzed

The core free-body step is to choose the body and draw it separated from supports and surrounding objects, then show the forces and moments that act on it. A correct free-body diagram is the basis for writing correct equilibrium equations. [S1]

Fraia agents should name the body being isolated: whole structure, authored `Member`, joint/`Node`, plate region, support, connection, or a cut portion of a resolved model.

### Include loads, reactions, moments, dimensions, and frame

A useful free-body diagram includes known loads, unknown reactions, applied moments/couples, relevant dimensions, and the coordinate frame used for components. [S1]

In Fraia terms, that means equilibrium explanations should reference `LoadAssignment`, `SupportAssignment`, coordinate frame, load case, and relevant geometry/provenance rather than only drawing arrows.

### Equilibrium means force and moment balance

Static equilibrium requires force balance and moment balance. In planar rigid-body problems, this is normally expressed as two independent force-balance equations and one independent moment-balance equation; spatial problems require the corresponding 3D balances. [S2][S3]

Equilibrium is necessary for reaction and load-path reasoning, but it does not by itself prove that a model is stable, determinate, or correctly idealised.

### Body selection changes what becomes external

A whole-structure free-body diagram usually exposes external support reactions and applied loads. A member, joint, or cut free-body diagram exposes internal actions as external forces/moments on the isolated body. [S1][S3]

This is why Fraia should keep authored structural objects and resolved topology visible. The same physical interaction can be internal to one body selection and external to another.

## Engineering guidance for Fraia agents

- Before writing equilibrium equations, identify the isolated body.
- State the coordinate frame and sign convention used for force/moment components.
- List known loads, unknown reactions, support conditions, dimensions, and relevant load cases.
- Do not mix whole-structure reactions with member-cut internal actions without explaining the body selection.
- Treat free-body reasoning as an explanation and sanity-check tool, not as a replacement for solver runs or design checks.
- Use free-body reasoning to cross-check reactions, load paths, missing supports, missing loads, and unexpected internal force signs.
- If the free-body cannot be drawn because the target, frame, or load path is unclear, ask for more context instead of guessing.

## Tradeoffs / cautions

- Free-body diagrams are simple, but a wrong or incomplete one leads to wrong equations.
- Equilibrium checks are powerful for determinate subsystems and sanity checks, but indeterminate systems need compatibility/stiffness information.
- Whole-model equilibrium can look plausible while local releases, disconnected nodes, or mechanisms remain wrong.
- Free-body diagrams are not just visual aids; they are a boundary definition for what interactions are external to the isolated body.
- In Fraia, a clean diagram is less important than preserving the underlying body selection, loads, reactions, frame, and provenance.

## Source-backed claims

- Free-body diagrams identify the forces and moments acting on an isolated body and are the basis for writing equilibrium equations. [S1]
- Correct equilibrium analysis requires force and moment balance. [S2][S3]
- Planar structural equilibrium commonly uses two force-balance equations and one moment-balance equation. [S3]
- Support reactions and applied loads should be represented on the isolated body before solving equilibrium. [S1][S3]
- Body selection determines whether an interaction is treated as external or internal to the free body. [S1][S3]

## Open questions / weak evidence

- Detailed truss joint/section procedures remain future pages.
- Detailed shear and moment diagram procedures remain future pages.
- Fraia still needs final UI/data representation for explicit free-body explanation artifacts, if any.

## Related pages

- [Static determinacy and restraint](static-determinacy-and-restraint.md)
- [Reactions and support idealisation](reactions-and-support-idealisation.md)
- [Truss analysis and two-force members](truss-analysis-and-two-force-members.md)
- [Beam shear and moment diagrams](beam-shear-and-moment-diagrams.md)
- [Load paths](load-paths.md)
- [Area, line, point, and member loads](../loads/area-line-point-and-member-loads.md)
- [Load application and equivalent nodal loads](../loads/load-application-and-equivalent-nodal-loads.md)

## Sources

- [S1] Daniel W. Baker and William Haynes / Engineering LibreTexts, *Free Body Diagrams*. URL: https://eng.libretexts.org/Bookshelves/Mechanical_Engineering/Engineering_Statics:_Open_and_Interactive_(Baker_and_Haynes)/05:_Rigid_Body_Equilibrium/5.02:_Free_Body_Diagrams. Source type: open educational statics text. Retrieved: 2026-05-07. Reliability/limits: strong free-body diagram and rigid-body statics guidance; not a structural solver guide.
- [S2] OpenStax / Physics LibreTexts, *Conditions for Static Equilibrium*. URL: https://phys.libretexts.org/Bookshelves/University_Physics/University_Physics_(OpenStax)/Book:_University_Physics_I_-_Mechanics_Sound_Oscillations_and_Waves_(OpenStax)/12:_Static_Equilibrium_and_Elasticity/12.02:_Conditions_for_Static_Equilibrium. Source type: open educational physics text. Retrieved: 2026-05-07. Reliability/limits: strong static-equilibrium foundation; not structural-analysis-specific.
- [S3] Felix Udoeyo / Engineering LibreTexts, *Equilibrium Structures, Support Reactions, Determinacy and Stability of Beams and Frames*. URL: https://eng.libretexts.org/Bookshelves/Civil_Engineering/Structural_Analysis_(Udoeyo)/01:_Chapters/1.03:_Equilibrium_Structures_Support_Reactions_Determinacy_and_Stability_of_Beams_and_Frames. Source type: open educational structural-analysis text. Retrieved: 2026-05-07. Reliability/limits: useful structural equilibrium and support-reaction framing; introductory and not a code check.
