# Research: Finite element idealisation/discretisation for structural modelling

Retrieval date for all web sources: 2026-05-06. Scope: open/public sources only.

## Summary
Finite element analysis (FEA) is an idealised numerical model, not a direct copy of authored engineering intent: continuous structures are represented by nodes, degrees of freedom, elements, shape functions, element matrices, assembled global equations, and explicit boundary/load definitions. For Fraia, the key wiki seed point is to keep authored structural objects such as `Member`, `Plate`, `SupportAssignment`, `LoadAssignment`, and `ReleaseAssignment` separate from downstream analysis elements/meshes, with durable provenance linking each discretised element back to the authored object and modelling assumptions.

## Source-backed claims

1. **FEA starts by discretising a continuous domain into elements and nodes.** LibreTexts/DoITPoMS lists FEM learning objectives including nodes, elements, discretisation, stiffness matrix assembly, boundary conditions, shape functions, linear/nonlinear static problems, and non-convergence. The TU Delft Euler-Bernoulli beam example explicitly begins with “Step 1: discretize the domain,” choosing 5 elements, 6 nodes, and 2 DOFs per node, then builds element DOF connectivity, element mass/stiffness matrices, global assembly, and boundary-condition reduction. [LibreTexts](https://eng.libretexts.org/Bookshelves/Materials_Science/TLP_Library_I/30%3A_Finite_Element_Method), [TU Delft TeachBooks](https://teachbooks.tudelft.nl/computational-modelling/structural_linear/Exercises/Workshop_FEM_dyn_beam.html)

2. **A single authored beam/member may be represented by multiple analysis elements.** In the TU Delft beam tutorial, one physical Euler-Bernoulli beam of length 3 m is discretised into `ne = 5` finite elements and `nn = ne + 1` nodes; each element contributes local matrices that are assembled into global matrices. This supports Fraia’s terminology rule: do not call each split analysis element a separate beam/column unless it is truly a separate semantic member. [TU Delft TeachBooks](https://teachbooks.tudelft.nl/computational-modelling/structural_linear/Exercises/Workshop_FEM_dyn_beam.html)

3. **Element choice is a modelling decision, not a default UI detail.** Bäker warns that element type is often a crucial step and defaults should not be accepted uncritically. He highlights element order, integration points, shear locking, volumetric locking, hourglassing, and differences between shell/membrane and beam formulations. FreeCAD documentation similarly states that lines/wires are used for beam elements, surfaces for shell/2D elements, and solids for solid elements, and recommends idealising slender or thin-walled structures as beams/shells where valid. [Bäker 2018](https://ar5iv.labs.arxiv.org/html/1811.05753), [FreeCAD FEM geometry/meshing](https://reqrefusion.github.io/FreeCAD-Documentation-html/wiki/FEM_Geometry_Preparation_and_Meshing.html)

4. **Beam/shell/plate idealisations are efficient but have applicability limits.** FreeCAD recommends beam elements for slender, regular-cross-section, beam-like parts and notes a common rule of thumb that cross-section dimensions should be less than about 1/10 of part length. It recommends shell elements for thin-walled parts and notes a similar thickness-to-global-dimension rule of thumb. It also distinguishes truss/membrane idealisations as no-bending models. These are practical heuristics, not universal rules. [FreeCAD FEM geometry/meshing](https://reqrefusion.github.io/FreeCAD-Documentation-html/wiki/FEM_Geometry_Preparation_and_Meshing.html)

5. **Boundary conditions are part of the idealisation and can dominate model correctness.** LibreTexts names boundary conditions as a core FEM concept. TU Delft’s cantilever beam applies a clamp at `x = 0` by fixing displacement and rotation DOFs, then reduces the global matrices to free DOFs. Bäker advises checking after calculation that nodes were constrained as intended, ensuring static/quasi-static models have no unrestrained rigid-body motion, and avoiding point loads at single nodes because they can cause unrealistic adjacent-element stresses. [LibreTexts](https://eng.libretexts.org/Bookshelves/Materials_Science/TLP_Library_I/30%3A_Finite_Element_Method), [TU Delft TeachBooks](https://teachbooks.tudelft.nl/computational-modelling/structural_linear/Exercises/Workshop_FEM_dyn_beam.html), [Bäker 2018](https://ar5iv.labs.arxiv.org/html/1811.05753)

6. **Loads and supports should often be distributed over realistic regions rather than concentrated at nodes.** Bäker cautions that point loads at single nodes may cause unrealistic stresses, especially with nonlinear material/geometry, and suggests distributing loads over several elements with local refinement if needed. FreeCAD notes stress singularities can arise from concentrated forces on solid/shell models and boundary conditions applied to points, recommending loads/BCs on small areas, fillets at sharp corners, or reading stresses away from singularities when justified. [Bäker 2018](https://ar5iv.labs.arxiv.org/html/1811.05753), [FreeCAD FEM geometry/meshing](https://reqrefusion.github.io/FreeCAD-Documentation-html/wiki/FEM_Geometry_Preparation_and_Meshing.html)

7. **Mesh density must match the output quantity and local gradients.** Bäker states fine mesh is needed where stress/strain gradients are large; displacements are usually computed more accurately than stresses/strains because strains are derivatives of displacements. FreeCAD recommends starting with a coarser mesh, refining globally or locally, and using dense mesh near large stress gradients/concentrations while keeping coarser mesh elsewhere to control solving time. [Bäker 2018](https://ar5iv.labs.arxiv.org/html/1811.05753), [FreeCAD FEM geometry/meshing](https://reqrefusion.github.io/FreeCAD-Documentation-html/wiki/FEM_Geometry_Preparation_and_Meshing.html)

8. **Mesh convergence studies are required for serious/accurate results.** FreeCAD recommends mesh convergence studies for serious projects, repeating solve/refine cycles and plotting result versus mesh density; it gives a practical acceptance example of changes below a few percent, e.g. 5%, while noting singular stresses may not converge. Bäker recommends mesh convergence studies to detect too-stiff/too-soft behaviour and cautions that load-controlled and displacement-controlled models require checking different response quantities. [FreeCAD FEM geometry/meshing](https://reqrefusion.github.io/FreeCAD-Documentation-html/wiki/FEM_Geometry_Preparation_and_Meshing.html), [Bäker 2018](https://ar5iv.labs.arxiv.org/html/1811.05753)

9. **Mesh quality and element order matter, especially for bending.** Bäker recommends second-order elements for linear-elastic problems, warns against fully integrated first-order elements in bending due to shear locking, and advises avoiding first-order triangular/tetrahedral elements because they are too stiff, especially in bending, unless used with very fine mesh and convergence checks. FreeCAD similarly says quadrilateral/hexahedral elements are generally preferable to triangular/tetrahedral elements and that second-order elements are preferred in most cases, especially for triangular/tetrahedral meshes. [Bäker 2018](https://ar5iv.labs.arxiv.org/html/1811.05753), [FreeCAD FEM geometry/meshing](https://reqrefusion.github.io/FreeCAD-Documentation-html/wiki/FEM_Geometry_Preparation_and_Meshing.html)

10. **Geometry preparation/defeaturing is part of analysis idealisation.** FreeCAD stresses that FEM requires properly prepared geometry and mesh, invoking “garbage in, garbage out.” It recommends simplifying CAD geometry by omitting small features that do not significantly affect strength/stiffness, such as small fillets/chamfers, small holes, welds, bolts, threads, and decorative details, but also notes exceptions where fillets may be needed to remove singularities. [FreeCAD FEM geometry/meshing](https://reqrefusion.github.io/FreeCAD-Documentation-html/wiki/FEM_Geometry_Preparation_and_Meshing.html)

11. **Symmetry is valid only when geometry, loads, boundary conditions, and response are symmetric.** FreeCAD says planar symmetry requires symmetry in geometry, loads, boundary conditions, and response, and warns that frequency/buckling analyses using symmetry may miss antisymmetric modes. Bäker likewise says loads and deformations must conform to symmetry assumptions and distinguishes plane stress, plane strain, and generalized plane strain. [FreeCAD FEM geometry/meshing](https://reqrefusion.github.io/FreeCAD-Documentation-html/wiki/FEM_Geometry_Preparation_and_Meshing.html), [Bäker 2018](https://ar5iv.labs.arxiv.org/html/1811.05753)

12. **Assembly connectivity is an idealised topology issue, not just visual contact.** FreeCAD explains that exactly touching parts may create a continuous mesh without constraints, while tiny gaps or intersections can produce disconnected meshes requiring tie/contact constraints; frequency analysis can reveal disconnected parts when modes show separation. This maps directly to Fraia’s need to keep authored topology, resolved/realised connectivity, and solver mesh connectivity inspectable. [FreeCAD FEM geometry/meshing](https://reqrefusion.github.io/FreeCAD-Documentation-html/wiki/FEM_Geometry_Preparation_and_Meshing.html)

13. **Post-processing can mislead if analysis-element quantities are mistaken for authored-object quantities.** Bäker notes stresses/strains are defined at integration points and contour-plot extrema may be extrapolated and inaccurate; he recommends checking plausibility, boundary conditions, mesh density, mesh quality, stress continuity, and free-surface normal stresses. This supports reporting analysis results with provenance, quantity location, and extraction method rather than attaching raw contour maxima directly to authored members. [Bäker 2018](https://ar5iv.labs.arxiv.org/html/1811.05753)

14. **Simple structural-analysis tools expose the authored-structure workflow but still solve FEM internally.** EduBeam’s public guide describes a user workflow of placing nodes, connecting elements, assigning materials/sections, applying supports/loads, and reviewing FEM reactions, displacements, and internal forces. This is useful as a public example of structural UI objects being converted into solver-ready FEM objects, though it is not a rigorous modelling-standard source. [EduBeam guide](https://edubeam.app/guide/introduction.html)

## Fraia implications

- **Canonical separation:** Keep authored primitives (`Node`, `Member`, `Plate`, `SupportAssignment`, `LoadAssignment`, `ReleaseAssignment`) distinct from resolved analysis topology (`AnalysisNode`, `FrameElement`, `ShellElement`, mesh cells, DOFs, constraints). One authored `Member` may map to 1..N analysis elements; one `Plate` may map to many shell/plate elements.
- **Use “role” for authored objects, “element” for solver/discretisation objects:** Display “Beam B1 discretised into 12 frame elements,” not “12 beams.”
- **Persist idealisation metadata:** For every analysis run, record element family, formulation assumptions, mesh size/order, refinement regions, releases/offsets/end constraints, local axes, load distribution method, boundary-condition mapping, and solver warnings.
- **Provenance links:** Every analysis element should retain references back to authored object IDs and realization/run IDs. Results should report both analysis location (element/integration point/node) and authored object aggregation.
- **Boundary-condition mapping should be inspectable:** Supports and releases should resolve into DOF constraints with clear labels, signs, coordinate systems, and affected analysis nodes/elements.
- **Load mapping should be explicit:** Nodal point loads, member distributed loads, plate pressures, self-weight, and equivalent nodal loads should be represented as different resolved load forms, not hidden under a single UI label.
- **Mesh/idealisation quality gates:** Fraia wiki should seed warnings for aspect ratio/element quality, first-order tets/tris, abrupt mesh transitions, singularities, missing rigid-body constraints, unconnected parts, and invalid symmetry assumptions.
- **Results are run artifacts:** Store immutable analysis outputs separately from authored model state; include mesh convergence notes and extraction policy for design actions/check inputs.

## Cautions for wiki language

- Do not imply FEM “is the structure.” It is an approximate model whose validity depends on assumptions, idealisation, element choice, mesh quality, loads, BCs, and material data.
- Avoid presenting 1/10 beam/shell slenderness rules as code limits; describe them as common practical heuristics that require engineering judgement.
- Warn that stress maxima at point loads, point constraints, sharp re-entrant corners, and contact corners may be singular and non-convergent.
- Warn that contour plots may show extrapolated nodal/visual values rather than integration-point values.
- Warn that increasing mesh density does not fix wrong physics: bad boundary conditions, wrong element formulation, bad material data, or invalid symmetry can still produce wrong results.
- Avoid mixing authored-member naming with FE element naming in UI, docs, exports, and result tables.

## Suggested Fraia cross-links

- `docs/structural-app-object-model.md` — canonical authored structural object vocabulary and role labels.
- `docs/resolution-and-runs.md` — authored state vs resolved/realisation state vs immutable run artifacts.
- `docs/engineering-core.md` — primitive-first engineering substrate.
- `docs/engineering-output-pipeline.md` — downstream result/design/export pipeline.
- `docs/builder-graph-architecture.md` — builder graphs as compact configuration layers above primitives.

## Seed wiki outline

1. **Authored structure vs analysis model**
   - Authored members/plates are semantic engineering objects.
   - Analysis elements are numerical discretisation objects.
   - One authored object can produce many analysis elements.
2. **Idealisation choices**
   - Beam/frame/truss, plate/shell/membrane, solid models.
   - Centerline/midsurface abstractions.
   - Local axes, offsets, eccentricities, releases.
3. **Mesh/discretisation**
   - Nodes, DOFs, elements, shape functions, assembly.
   - Mesh size/order/refinement/convergence.
   - Quality checks and common bad elements.
4. **Boundary conditions and loads**
   - Supports/restrains as DOF constraints.
   - Load distribution and equivalent nodal loads.
   - Avoid point singularities when modelling distributed real actions.
5. **Model limits and verification**
   - Assumptions, symmetry, linear/nonlinear geometry, material uncertainty.
   - Plausibility checks and independent hand estimates.
   - Result extraction and post-processing cautions.

## Sources

### Kept
- LibreTexts / DoITPoMS, “30: Finite Element Method” (https://eng.libretexts.org/Bookshelves/Materials_Science/TLP_Library_I/30%3A_Finite_Element_Method) — concise open educational source naming nodes, elements, discretisation, boundary conditions, stiffness assembly, shape functions, nonlinear/non-convergence concepts.
- TU Delft TeachBooks, “FEM for an Euler-Bernoulli beam” (https://teachbooks.tudelft.nl/computational-modelling/structural_linear/Exercises/Workshop_FEM_dyn_beam.html) — explicit worked example showing one beam discretised into multiple elements/nodes/DOFs, element matrices, global assembly, and boundary-condition reduction.
- FreeCAD Documentation mirror, “FEM Geometry Preparation and Meshing” (https://reqrefusion.github.io/FreeCAD-Documentation-html/wiki/FEM_Geometry_Preparation_and_Meshing.html) — practical public guidance on beam/shell/solid idealisation, defeaturing, partitioning, assembly connectivity, mesh size/order, convergence, and singularities.
- Martin Bäker, “How to get meaningful and correct results from your finite element model” (https://ar5iv.labs.arxiv.org/html/1811.05753) — open arXiv-accessible checklist-style paper on modelling goals, BCs, loads, element type, mesh generation, convergence, post-processing, and validation.
- EduBeam guide, “Introduction” (https://edubeam.app/guide/introduction.html) — public example of structural UI workflow that maps nodes/elements/materials/supports/loads to FEM reactions, displacements, and internal forces.

### Dropped
- NAFEMS “How To Use Beam Plate & Shell Elements” — likely authoritative, but not fully open/public content; only abstract/marketing page available.
- Abaqus benchmark/manual pages — useful as commercial solver references, but not preferred for Fraia wiki seed because access/status is not fully open and examples are solver-specific.
- COMSOL Structural Mechanics Module PDF — public PDF, but commercial/manual-specific and very large; not necessary given stronger open educational/practical sources.
- Generic structural modelling blog posts — readable but less authoritative than open course notes, FreeCAD documentation, and the arXiv checklist.
- De Gruyter chapter result — paywalled/limited access; excluded.
- Random mirrored textbook PDF — avoided due to unclear rights/provenance.

## Gaps / next steps

- Open, structural-engineering-specific standards language on member-to-element provenance was not found in this short pass; next step would be targeted research in open solver docs such as OpenSees, Code_Aster, CalculiX, OOFEM, or IFC/SAF-style data exchange references.
- Need Fraia-specific decisions on exact type names for `AnalysisNode`, `FrameElement`, `ShellElement`, `MeshRegion`, `ResolvedSupport`, and result aggregation records.
- Need future research on releases/end offsets/rigid links, plate drilling DOFs, member end force recovery, and design-action extraction from split frame elements.
