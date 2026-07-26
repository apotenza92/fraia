---
title: Matrix Stiffness Method
status: compiled
trust_level: compiled
domain: analysis
applies_to:
  - concept-stage structural analysis explanations
  - Fraia resolved analysis topology
  - solver adapter and run artifact guidance
not_applicable_to:
  - formula reference for stiffness matrices
  - nonlinear or dynamic analysis implementation
  - final solver API documentation
jurisdiction_or_standard_context: concept guidance from academic/open sources; not a code check
last_compiled: 2026-05-07
source_count: 4
citation_policy: required
owner: agent-maintained
---

# Matrix Stiffness Method

## Summary

The matrix stiffness method represents a structure as nodes, degrees of freedom, analysis elements, stiffness relationships, loads, restraints, and solved displacements/reactions. It is the conceptual bridge between Fraia's authored structural model and the solver-ready resolved analysis model.

For Fraia agents, this page is not a formula table. It explains why authored `Member`, `Plate`, `SupportAssignment`, `LoadAssignment`, and `ReleaseAssignment` objects must be realized into analysis topology before a run, and why the run artifact must preserve the exact assumptions used.

## Scope / non-scope

This page covers concept-level guidance for the linear matrix/direct stiffness workflow as it matters to Fraia's pipeline.

It does not derive element stiffness matrices, define solver API calls, cover nonlinear/dynamic analysis, or replace validated solver documentation.

## Key concepts

### Nodes and DOFs are the solved unknowns

In stiffness-based structural analysis, nodal displacements and rotations are primary unknowns, and forces are related to those unknowns through stiffness relationships. Frame examples express each node with translational and rotational degrees of freedom, then assemble element behavior into a structural system. [S1][S2]

Fraia should keep this separate from the authored model. A user-authored `Member` is not itself a solved equation row; realization creates analysis nodes, analysis elements, DOFs, restraints, loads, and mappings back to the authored object.

### Element behavior is assembled into a global system

Each analysis element contributes stiffness relationships between its end/node DOFs. Because members can have different orientations, element-local behavior may be transformed into global coordinates before assembly. Shared global DOFs connect the elements into one structural system. [S1][S3]

This makes connectivity and coordinate-frame provenance central. Coincident but unmerged nodes, wrong local axes, or hidden releases are not superficial metadata errors; they change the solved system.

### Boundary conditions define the solvable system

Supports, restraints, releases, and constraints determine which DOFs are free, restrained, prescribed, or related. These choices change the global stiffness system and can create indeterminacy, mechanisms, singular matrices, or overconstrained behavior. [S2][S3]

Fraia agents should map any instability or surprising reaction back to authored `SupportAssignment`, `ReleaseAssignment`, constraints, connectivity, and the resolved analysis topology before proposing changes.

### Loads must be realized into solver-ready actions

Loads may be applied directly at nodes or applied to members/elements and converted into equivalent nodal or fixed-end actions depending on the analysis formulation. [S2][S3]

That means a `LoadAssignment` should keep its authored intent and provenance, while the resolved run artifact records the exact load components, coordinate frames, load cases, combinations, and element/node targets used by the solver.

### Results need recovery and provenance

After solving the assembled system, global displacements and reactions can be reported, and member/element forces are recovered using element behavior and coordinate transformations. [S3][S4]

Fraia should preserve the difference between analysis results, design actions, check inputs, and check results. A bending moment recovered for an analysis element is not automatically a complete steel design check.

## Engineering guidance for Fraia agents

- Explain the matrix stiffness method as a pipeline: authored objects → resolved topology → assembled system → solved run → recovered results → downstream design actions/checks.
- Keep authored structural objects distinct from analysis nodes and elements.
- Use `element` for finite-element/discretisation objects, not as a synonym for authored members.
- Preserve mappings from analysis elements back to authored members, plates, supports, loads, releases, builder nodes, and run ids where applicable.
- When discussing solver failures, check DOFs, restraints, releases, connectivity, coordinate transformations, element properties, and load realization.
- Do not treat a successful solver run as proof that authored assumptions are correct; the run artifact only proves the stated resolved model was solved under recorded settings.
- Do not copy formula tables into the wiki unless a future page explicitly needs a source-scoped derivation.

## Tradeoffs / cautions

- The stiffness method is general enough to support many structures, but every useful result depends on modeling assumptions.
- Simple 2D frame examples are excellent for explaining assembly, but they do not settle all 3D, plate/shell, nonlinear, or dynamic behavior.
- Hiding transformations, releases, or equivalent nodal-load conversion makes results hard to audit.
- Overly compact builder graphs should not replace the resolved analysis model; they are configuration layers above the primitive structural model.
- Solver adapters should be downstream of Fraia's canonical authored/resolved data, not the source of engineering truth.

## Source-backed claims

- Matrix/direct stiffness analysis relates nodal displacement/rotation DOFs and nodal forces through element and assembled stiffness relationships. [S1][S2]
- Frame members with different orientations require coordinate transformation between element-local and global assembled coordinates. [S1][S3]
- Element stiffness contributions are assembled into a global structural stiffness system through shared nodal DOFs. [S2][S3]
- Boundary conditions/restraints affect which DOFs are solved, restrained, or prescribed, and therefore affect solvability and reactions. [S2][S3]
- Member forces and reactions are recovered from solved displacements and element relationships after the global system is solved. [S3][S4]

## Open questions / weak evidence

- Fraia still needs final schemas for DOF ordering, element-result channels, equivalent nodal load provenance, and solver adapter metadata.
- Nonlinear analysis, dynamic analysis, and second-order stiffness effects need separate compiled pages.
- The sources are strong for concept-level linear frame/direct stiffness guidance; detailed 3D member, plate/shell, and solver-specific conventions remain future work.

## Related pages

- [Static determinacy and restraint](static-determinacy-and-restraint.md)
- [Local and global coordinate systems](../modeling/local-and-global-coordinate-systems.md)
- [Load application and equivalent nodal loads](../loads/load-application-and-equivalent-nodal-loads.md)
- [Second-order effects and stability](second-order-effects-and-stability.md)
- [Finite-element idealisation](../modeling/finite-element-idealisation.md)
- [Supports, restraints, and releases](../modeling/supports-restraints-and-releases.md)
- [Instability mechanisms](../diagnostics/instability-mechanisms.md)

## Sources

- [S1] TU Delft TeachBooks / Computational Modelling, *2D frame analysis*. URL: https://teachbooks.tudelft.nl/computational-modelling/structural_linear/space_frame.html. Source type: university open course notes. Retrieved: 2026-05-07. Reliability/limits: strong 2D frame/local-global stiffness explanation; 3D and nonlinear extensions remain out of scope.
- [S2] Engineering LibreTexts / Aerospace Structures, *Direct stiffness method*. URL: https://eng.libretexts.org/Under_Construction/Aerospace_Structures_(Johnson)/15:_Direct_stiffness_method. Source type: open educational structural/mechanics text. Retrieved: 2026-05-07. Reliability/limits: useful direct-stiffness workflow; page is under construction, so claims are corroborated and kept conceptual.
- [S3] Engineering LibreTexts / Aerospace Structures, *Applications of the direct stiffness method*. URL: https://eng.libretexts.org/Under_Construction/Aerospace_Structures_(Johnson)/16:_Applications_of_the_direct_stiffness_method. Source type: open educational structural/mechanics text. Retrieved: 2026-05-07. Reliability/limits: useful member transformation, global stiffness, and member-force recovery guidance; formula derivations are source-scoped.
- [S4] Engineering LibreTexts / Aerospace Structures, *Finite element method*. URL: https://eng.libretexts.org/Under_Construction/Aerospace_Structures_(Johnson)/17:_Finite_element_method. Source type: open educational structural/mechanics text. Retrieved: 2026-05-07. Reliability/limits: useful connection between direct stiffness and FEM concepts; not a Fraia solver implementation guide.
