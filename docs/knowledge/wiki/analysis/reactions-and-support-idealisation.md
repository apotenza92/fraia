---
title: Reactions and Support Idealisation
status: compiled
trust_level: compiled
domain: analysis
applies_to:
  - concept-stage structural analysis explanations
  - support and reaction interpretation
  - Fraia agent guidance
not_applicable_to:
  - final foundation design
  - code-compliant capacity checks
  - nonlinear contact or uplift-only support modeling
jurisdiction_or_standard_context: concept guidance from academic/open sources; not a code check
last_compiled: 2026-05-07
source_count: 3
citation_policy: required
owner: agent-maintained
---

# Reactions and Support Idealisation

## Summary

Support reactions are force and moment components that arise where a support idealisation restrains motion. They are not arbitrary arrows added after analysis; they are consequences of the chosen support degrees of freedom, coordinate frame, load case, and equilibrium/stiffness model.

For Fraia, a named support such as pin, roller, fixed, or sliding support should be treated as a user-facing shorthand for a precise `SupportAssignment`. The durable modeling content is the restrained or prescribed degrees of freedom, support direction, coordinate frame, stiffness or displacement assumption, and provenance.

## Scope / non-scope

This page covers concept-level reaction and support-idealisation guidance for Fraia agents.

It does not provide foundation design, connection design, code-specific checks, nonlinear contact rules, or detailed hand-calculation examples.

## Key concepts

### Reactions come from restrained motion

A support reaction represents the force or moment needed to enforce a restrained or prescribed motion under the adopted model. In a planar model, common idealisations restrain some combination of translation and rotation; in a spatial model, the support may affect translational and rotational degrees of freedom in three dimensions. [S1][S2]

Fraia should therefore store the actual restrained or prescribed DOFs. A label like "pin" is useful for display, but it is not enough to explain what reaction components can exist.

### Support type is an idealisation, not the physical object

Real bearings, base plates, bolts, footings, slabs, walls, and soil supports do not automatically equal perfect pins, perfect rollers, or perfect fixed supports. A support model is an engineering assumption about which movements are restrained, which are free, and whether restraint is rigid, flexible, prescribed, or conditional. [S2]

Fraia agents should describe the idealisation explicitly and preserve why it was chosen.

### Reaction calculation starts with equilibrium

Reaction reasoning begins by isolating a body, drawing applied loads and unknown reactions, choosing axes/sign convention, and applying force and moment equilibrium. Structural-analysis examples commonly compute reactions by replacing distributed loads with resultants when appropriate and then applying equilibrium equations. [S1][S3]

Equilibrium is necessary for reaction sanity checks, but it is not a complete model-validity proof.

### Moment balance can use a convenient point

Moment equilibrium may be written about any convenient point or axis, and a smart choice can eliminate unknown reaction components from one equation. The result should not depend on the pivot point when the same free body and sign convention are used consistently. [S3]

Fraia explanations should state the reference point or axis when using moment balance to justify a reaction.

### Reaction signs are diagnostic information

Unknown reaction directions are often assumed before solving. A solved negative value usually means the actual reaction acts opposite to the assumed positive direction, not automatically that the model is wrong. [S1]

Negative reactions can still be important diagnostics when they contradict a physical support assumption, such as compression-only bearing, uplift limits, or unilateral contact.

### Global reaction balance is not enough

The sum of applied actions and reactions can look balanced while the model still contains a local mechanism, disconnected node, over-release, wrong support axis, or unrealistic stiffness assumption. Determinate support reaction checks and full solver results answer different questions. [S1][S2]

Fraia should use reaction balance as a fast sanity check, then map suspicious results back to authored supports, loads, releases, constraints, and resolved topology.

## Engineering guidance for Fraia agents

- Explain every named support by its restrained or prescribed DOFs, coordinate frame, and reaction components.
- Keep authored `SupportAssignment` intent separate from resolved analysis restraints and immutable run reactions.
- Do not infer a fixed base, pinned base, roller, spring, or prescribed settlement from geometry alone.
- When reporting a reaction, include the load case or combination, component direction, sign convention, support/node reference, and whether the value is from a run artifact or a hand equilibrium explanation.
- Treat a negative reaction as a sign/direction result first; flag it as a physical issue only when it violates the support assumption.
- Do not use reaction equilibrium to hide under-restraint, over-release, or disconnected topology.
- If the intended physical support could be partially restrained, flexible, settlement-prone, uplift-only, or direction-dependent, record the assumption and avoid overconfident language.

## Tradeoffs / cautions

- Simple support idealisations make models explainable, but they can overstate or understate real restraint.
- More fixity can reduce some displacements but can attract moments/reactions and change load paths.
- Fewer restraints can make load paths clearer, but may create mechanisms or unrealistic movement.
- A support direction that is correct in local coordinates can be wrong if silently interpreted in global coordinates.
- Support reactions are not design capacities; they are analysis outputs or equilibrium unknowns that downstream checks must interpret with context.

## Source-backed claims

- Support reactions are associated with restrained degrees of freedom and support idealisations. [S1][S2]
- Planar static equilibrium uses force and moment balance; spatial equilibrium requires the corresponding 3D balances. [S3]
- Structural reaction examples compute unknown support reactions from free-body diagrams and equilibrium equations. [S1]
- Moment equilibrium may be taken about a convenient point without changing the physical result when the same body and convention are used consistently. [S3]
- Support displacement or prescribed nonzero movement can still generate support reactions in the restrained direction. [S2]

## Open questions / weak evidence

- Fraia still needs final schema details for support stiffness, prescribed settlement, unilateral support behavior, and partial restraint.
- Foundation/soil interaction is out of scope for this page and needs separate treatment.
- Detailed 3D support presets and UI display names should be checked against Fraia's final DOF ordering.

## Related pages

- [Free-body diagrams and equilibrium](free-body-diagrams-and-equilibrium.md)
- [Truss analysis and two-force members](truss-analysis-and-two-force-members.md)
- [Beam shear and moment diagrams](beam-shear-and-moment-diagrams.md)
- [Static determinacy and restraint](static-determinacy-and-restraint.md)
- [Load paths](load-paths.md)
- [Reaction sanity checks](../diagnostics/reaction-sanity-checks.md)
- [Supports, restraints, and releases](../modeling/supports-restraints-and-releases.md)
- [Local and global coordinate systems](../modeling/local-and-global-coordinate-systems.md)
- [Instability mechanisms](../diagnostics/instability-mechanisms.md)

## Sources

- [S1] Felix Udoeyo / Engineering LibreTexts, *Equilibrium Structures, Support Reactions, Determinacy and Stability of Beams and Frames*. URL: https://eng.libretexts.org/Bookshelves/Civil_Engineering/Structural_Analysis_(Udoeyo)/01:_Chapters/1.03:_Equilibrium_Structures_Support_Reactions_Determinacy_and_Stability_of_Beams_and_Frames. Source type: open educational structural-analysis text. Retrieved: 2026-05-07. Reliability/limits: useful support-reaction and equilibrium examples for beams/frames; introductory and not a code check.
- [S2] Tom van Woudenberg / Delft University of Technology, *Supports*. URL: https://oit.tudelft.nl/CEG-mechanics-BSc/support_internal_forces/model/supports.html. Source type: university open course notes. Retrieved: 2026-05-07. Reliability/limits: useful DOF/reaction/support notation and prescribed displacement framing; course-scoped and not a full structural design reference.
- [S3] OpenStax, *University Physics Volume 1: 12.1 Conditions for Static Equilibrium*. URL: https://openstax.org/books/university-physics-volume-1/pages/12-1-conditions-for-static-equilibrium. Source type: open educational physics text. Retrieved: 2026-05-07. Reliability/limits: strong rigid-body equilibrium foundation; not structural-analysis-specific.
