---
title: Area, Line, Point, and Member Loads
status: compiled
trust_level: compiled
domain: loads
applies_to:
  - authored load modeling
  - primitive-first structural models
  - Fraia agent guidance
not_applicable_to:
  - code-specific load values
  - load combination rules
  - moving load analysis
jurisdiction_or_standard_context: concept guidance from academic/open sources; not a code check
last_compiled: 2026-05-07
source_count: 3
citation_policy: required
owner: agent-maintained
---

# Area, Line, Point, and Member Loads

## Summary

Fraia should treat loads as authored engineering objects with target, distribution, direction, coordinate frame, magnitude, load case, and provenance. Point, line, area/surface, and member loads are not just UI drawing styles; they describe how the load is applied before realization converts it to solver-ready actions.

For Fraia agents, load type should answer: where does the load act, over what geometric extent, in which direction/frame, in which case/combination, and how will it be resolved for analysis?

## Scope / non-scope

This page covers concept-level authored load taxonomy for point, line, area/surface, distributed, and member loads.

It does not provide code load magnitudes, load combination rules, moving-load workflows, or exact element load-vector derivations.

## Key concepts

### Point loads idealize concentrated force

A point load represents a force acting at a specific location, often a node, support, connection, or idealized application point. Point loads are useful primitives, but many real loads are distributed rather than truly concentrated. [S1]

Fraia should store both the authored target and the resolved target. A point load authored on a member at a station is not the same data as a nodal load authored directly on a node, even if realization later produces nodal force components.

### Line and member loads have intensity along length

Distributed loads spread force over a distance and are commonly represented by load intensity, such as force per unit length. [S1][S2]

In Fraia, a line load may be authored on a geometric line, edge, member, or generated load path. A member load is a line/distributed load whose target is an authored `Member` or a realized analysis element associated with that member. The member-local/global coordinate frame matters.

### Area and surface loads spread over a region

Area or surface loads represent loading over a region rather than at a point or along a single line. In building models, this can include floor loads, roof pressures, cladding pressures, or plate/shell surface loads. The durable authored load should preserve its region, direction, projection rule, tributary assumptions, and source/provenance.

Area loads often need downstream distribution to members, plates, nodes, or analysis elements. That distribution is a realization step, not the original engineering truth.

### Equivalent resultants are useful but not the authored load

Distributed loads can be converted to equivalent resultants for equilibrium reasoning, often using the load distribution to determine magnitude and location. [S1][S2]

Fraia should not treat that equivalent resultant as the only durable representation unless the user actually authored a point load. The original distribution may matter for internal forces, deflections, member checks, and solver-equivalent nodal/fixed-end loads.

### Load taxonomy is separate from cases and combinations

Load type/distribution is separate from load case membership and load combination rules. A point load, area load, member load, or imposed displacement can all belong to a load case. Combinations are downstream grouping/scaling instructions, not replacements for authored load geometry.

## Engineering guidance for Fraia agents

- Use `LoadAssignment` for authored loads and keep it separate from resolved solver loads.
- Record target object: node, member, plate, area, line, support, builder object, or analysis element.
- Record distribution type: point, line, area/surface, volume/body, imposed displacement, temperature, or other typed load.
- Record coordinate frame and direction: global, member-local, element-local, normal-to-surface, projected, or unresolved.
- Record load case separately from load geometry and magnitude.
- Keep load realization auditable: tributary distribution, equivalent nodal loads, fixed-end actions, projected components, and solver targets should remain traceable back to authored loads.
- Do not silently turn area or member loads into point loads without preserving the original distribution and conversion rule.

## Tradeoffs / cautions

- Point loads are simple and inspectable, but overusing them can hide real distribution and tributary assumptions.
- Area loads are natural for floors, roofs, and pressure loads, but they require clear distribution/realization rules.
- Member loads are convenient for beams, rafters, purlins, and braces, but local/global frame assumptions must be explicit.
- Equivalent resultants are useful for hand equilibrium checks, but may not preserve full internal-force or deflection behavior.
- Primitive-first load modeling should not collapse load type, load case, coordinate frame, and solver representation into one field.

## Source-backed claims

- Distributed loads represent force spread over a distance, area, or volume and are often described by intensity rather than one point of application. [S1]
- Distributed loads can be replaced by equivalent point/resultant loads for statics equilibrium reasoning when magnitude and location are determined from the distribution. [S1][S2]
- Structural analysis relates external loads on members/structures to internal responses such as stresses, displacements, and member forces. [S3]
- Fraia should preserve authored load distribution and target metadata because load distribution affects realization, reactions, internal forces, and result interpretation. [S1][S2][S3]

## Open questions / weak evidence

- Fraia still needs final schemas for pressure, projected area, tributary, imposed displacement, settlement, temperature, and body loads.
- Exact element-specific load conversion belongs in solver/analysis implementation docs.
- Code-specific load categories and combinations remain separate from this concept taxonomy.

## Related pages

- [Load application and equivalent nodal loads](load-application-and-equivalent-nodal-loads.md)
- [Load cases and load combinations](load-cases-and-combinations.md)
- [Gravity and lateral loads](gravity-and-lateral-loads.md)
- [Local and global coordinate systems](../modeling/local-and-global-coordinate-systems.md)
- [Matrix stiffness method](../analysis/matrix-stiffness-method.md)

## Sources

- [S1] Engineering Mechanics - Statics / Engineering LibreTexts, *Distributed Loads*. URL: https://eng.libretexts.org/Bookshelves/Mechanical_Engineering/Engineering_Mechanics_-_Statics_(Osgood_Cameron_and_Christensen)/03:_Rigid_Body_Basics/3.03:_Distributed_Loads. Source type: open educational statics text. Retrieved: 2026-05-07. Reliability/limits: useful point/distributed load and intensity concepts; not structural-code guidance.
- [S2] Daniel W. Baker and William Haynes / Engineering LibreTexts, *Distributed Loads*. URL: https://eng.libretexts.org/Bookshelves/Mechanical_Engineering/Engineering_Statics:_Open_and_Interactive_(Baker_and_Haynes)/07:_Centroids_and_Centers_of_Gravity/7.08:_Distributed_Loads. Source type: open educational statics text. Retrieved: 2026-05-07. Reliability/limits: strong equivalent-resultant guidance for statics; not a finite-element load-vector derivation.
- [S3] Felix Udoeyo / Engineering LibreTexts, *Introduction to Structural Analysis*. URL: https://eng.libretexts.org/Bookshelves/Civil_Engineering/Structural_Analysis_(Udoeyo)/01:_Chapters/1.01:_Introduction_to_Structural_Analysis. Source type: open educational structural-analysis text. Retrieved: 2026-05-07. Reliability/limits: introductory framing of external loads and structural response; concept-level use only.
