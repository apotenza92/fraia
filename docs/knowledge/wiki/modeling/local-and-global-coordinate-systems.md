---
title: Local and Global Coordinate Systems
status: compiled
trust_level: compiled
domain: modeling
applies_to:
  - concept-stage structural modeling
  - member loads, releases, reactions, and result interpretation
  - Fraia agent guidance
not_applicable_to:
  - final solver API documentation
  - complete 3D section-orientation implementation
  - plate/shell local-axis rules
jurisdiction_or_standard_context: concept guidance from academic/open sources; not a code check
last_compiled: 2026-05-07
source_count: 3
citation_policy: required
owner: agent-maintained
---

# Local and Global Coordinate Systems

## Summary

Structural models need both global and local coordinate systems. Global coordinates let nodes, supports, loads, and assembled degrees of freedom share one reference frame. Local coordinates let each member or analysis element express axial, shear, bending, torsion, releases, section axes, and member-end forces in directions that follow that object.

For Fraia agents, coordinate frames are not display details. They are part of the engineering meaning of authored loads, `SupportAssignment`, `ReleaseAssignment`, resolved analysis elements, solver run artifacts, and downstream check inputs.

## Scope / non-scope

This page covers concept-level guidance for local and global coordinate systems in member/frame modeling and result interpretation.

It does not define Fraia's final DOF ordering, solver adapter API, complete 3D section-axis construction rules, or plate/shell local-axis rules.

## Key concepts

### Coordinate systems are frames of reference

A coordinate system gives a frame of reference for describing points and vectors. In statics, orthogonal coordinate systems are useful because vector components can be treated as independent scalar components along perpendicular axes. Force, displacement, and reaction components only make sense once their coordinate frame is known. [S1]

Fraia should therefore treat coordinates and component labels as engineering data, not just visual labels.

### Global coordinates assemble the resolved model

Frame and finite-element models contain members with different orientations. A shared global coordinate frame is needed so resolved analysis nodes, nodal degrees of freedom, support restraints, and assembled equations refer to compatible directions. [S2]

For Fraia, this belongs in the resolved analysis model and immutable run artifact. The authored model may contain semantic objects such as `Member`, `Plate`, `SupportAssignment`, `LoadAssignment`, and `ReleaseAssignment`; realization resolves those into analysis nodes, analysis elements, element axes, load components, and solver-ready DOFs.

### Local coordinates follow the member or element

Each frame member or line analysis element can define a local coordinate frame aligned with that element. Local element axes are useful because axial deformation, transverse deformation, curvature, axial force, shear force, and bending moment are naturally interpreted relative to the member/element direction. [S2]

This is why member loads, member-end releases, and steel member results often need local-axis metadata. A `ReleaseAssignment` at a member end is not complete if it says only "pinned" without identifying the affected local rotational/force-transfer components.

### Transformations connect local and global quantities

Direct stiffness methods transform displacements, forces, and stiffness between local element coordinates and global assembled coordinates. The compiled page does not need to copy formula tables, but Fraia agents should understand the data-flow implication: local element behavior is transformed into global equations, and global solved displacements/reactions may need transformation back to local member forces and design actions. [S2][S3]

This same idea applies to load handling. A global vertical load, a member-local transverse load, and a projected roof load can all produce different resolved load components if the coordinate frame is not explicit.

### Role is not axis

Fraia's authored structural objects use `role` for semantic meaning: beam, column, rafter, brace, purlin, or tie. Local axes describe analysis and result directions. A vertical authored `Member` with role `column` and an inclined authored `Member` with role `rafter` both need local axes, but those axes do not replace the semantic role.

When an authored member is split during realization, describe it as a role-labelled member discretised into analysis elements. Do not call each split analysis element a separate beam, column, or rafter unless it is actually authored that way.

## Engineering guidance for Fraia agents

- Always ask whether a component is global, member-local, element-local, support-local, or section-local before explaining it.
- Store coordinate-frame metadata with authored loads and releases when the user intent is frame-dependent.
- Preserve the transformation path from authored object to resolved analysis element to solver run artifact to downstream design action.
- For member loads, distinguish global direction loads from local member direction loads.
- For reactions, state the frame used for reported components.
- For releases, state the local axes and DOFs affected; do not rely on generic names alone.
- For steel checks, keep section/strong/weak-axis assumptions explicit and source-scoped until Fraia has final section-orientation rules.
- If an agent cannot identify a coordinate frame, it should flag uncertainty rather than silently assuming global axes.

## Tradeoffs / cautions

- Global axes are convenient for assembly and project-level coordinates, but they are not always the natural way to describe member behavior.
- Local axes are convenient for member forces, releases, and section checks, but they require a stable definition and provenance.
- Changing local axis orientation can flip signs or swap components without changing the physical member.
- Simple 2D frame sources are strong for the local/global idea but do not settle all 3D section-orientation conventions.
- Different solvers and design packages may report member forces in different local-axis conventions; solver adapter metadata must make this inspectable.

## Source-backed claims

- Coordinate systems provide frames of reference for points and vectors, and force vectors can be represented by scalar components in coordinate directions. [S1]
- Frame models need a global coordinate frame because members can have different orientations while sharing assembled nodal degrees of freedom. [S2]
- Frame elements can define local coordinate frames aligned with the element, and local deformations/section forces are evaluated in that local frame. [S2]
- Direct stiffness formulations transform generalized displacements, forces, and stiffness between local member coordinates and global coordinates. [S3]
- In Fraia terms, coordinate-frame metadata is needed to interpret loads, releases, reactions, member forces, and downstream check inputs. [S1][S2][S3]

## Open questions / weak evidence

- Fraia still needs final canonical DOF ordering and local-axis conventions for 3D members.
- Plate/shell local axes and section-orientation rules need a separate source pass.
- This page avoids solver-specific coordinate-transformation APIs; those belong in solver adapter documentation or source-scoped wiki pages.

## Related pages

- [Finite-element idealisation](finite-element-idealisation.md)
- [Supports, restraints, and releases](supports-restraints-and-releases.md)
- [Static determinacy and restraint](../analysis/static-determinacy-and-restraint.md)
- [Matrix stiffness method](../analysis/matrix-stiffness-method.md)
- [Load application and equivalent nodal loads](../loads/load-application-and-equivalent-nodal-loads.md)
- [Steel member behavior](../materials/steel/member-behavior.md)

## Sources

- [S1] Daniel W. Baker and William Haynes / Engineering LibreTexts, *Two Dimensional Coordinate Systems*. URL: https://eng.libretexts.org/Bookshelves/Mechanical_Engineering/Engineering_Statics:_Open_and_Interactive_(Baker_and_Haynes)/02:_Forces_and_Other_Vectors/2.03:_Two_Dimensional_Coordinate_Systems. Source type: open educational statics text. Retrieved: 2026-05-07. Reliability/limits: strong coordinate/vector foundation; not structural FEM-specific.
- [S2] TU Delft TeachBooks / Computational Modelling, *2D frame analysis*. URL: https://teachbooks.tudelft.nl/computational-modelling/structural_linear/space_frame.html. Source type: university open course notes. Retrieved: 2026-05-07. Reliability/limits: strong 2D frame/local-global coordinate explanation; 3D section orientation remains out of scope.
- [S3] Engineering LibreTexts / Aerospace Structures, *Applications of the direct stiffness method*. URL: https://eng.libretexts.org/Under_Construction/Aerospace_Structures_(Johnson)/16:_Applications_of_the_direct_stiffness_method. Source type: open educational structural/mechanics text. Retrieved: 2026-05-07. Reliability/limits: useful direct-stiffness transformation derivation; page is under construction, so use as source-scoped support rather than sole authority.
