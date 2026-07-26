# Research: Instability mechanisms and underrestrained/unconnected model diagnostics

Retrieval date for all sources: 2026-05-06. Scope: open/public sources only. Intended use: Fraia knowledge wiki seed/raw brief, not final engineering guidance.

## Summary
Structural-analysis instability is usually the solver-visible symptom of an unconstrained degree of freedom, a disconnected load path, an unintended release/hinge chain, or a true geometric/material stability problem. Public documentation from PyNite, RISA, SCIA, Dlubal, LUSAS, and Oasys GSA converges on a practical diagnostic workflow: check supports/restraints in all relevant DOFs, check connectivity/duplicate nodes/intersections, inspect releases and torsion, run simple linear/dead-load cases, and use unstable-mode/eigenvector/deformed-shape visualization to localize the problem.

## Findings

1. **Instability often presents as a singular stiffness matrix because one or more DOFs can move without resistance.** PyNite explains that a stable 3D object needs all six DOFs stabilized and that rigid-body motion makes the stiffness matrix singular; SCIA similarly describes singular stiffness as a problem with degrees of freedom and reports the FE node/direction of the first instability. [PyNite stability](https://pynite.readthedocs.io/en/latest/stability.html); [SCIA singular stiffness FAQ](https://scia.net/en/support/faq/scia-engineer/analysis/warning-stiffness-matrix-singular-structure-unstable)

2. **Separate global rigid-body motion from local nodal instability.** PyNite distinguishes whole-structure rigid body motion from local nodal instability after global restraint is adequate. A common local case is a truss/pinned joint where every attached member is released, leaving a rotational DOF with no stiffness. [PyNite stability](https://pynite.readthedocs.io/en/latest/stability.html)

3. **Over-releasing member ends is a leading modelling cause.** RISA states that overuse of member end releases and/or boundary conditions is the most common instability cause; its examples include released cantilever free ends, pinned supports plus member-end releases, and truss panel points with every member released. Dlubal gives the related rule of avoiding hinge chains: at a node with `n` connected members, only `n-1` hinges with the same DOF should be defined. [RISA stability](https://help.risa.com/risahelp/risa3d/Content/Stability/Stability.htm); [Dlubal instability causes](https://www.dlubal.com/en/support-and-learning/support/faq/005345)

4. **Torsional mechanisms are common in 3D frame/member models.** SCIA’s example shows a beam supported by fully hinged end supports where rotation about the member’s local axis is unrestrained; making one rotational support rigid fixes the instability. Dlubal likewise lists member torsion about its own axis, often caused by torsional releases at both member ends. RISA describes torsional instability where a member or chain of members can spin about its local axis. [SCIA singular stiffness FAQ](https://scia.net/en/support/faq/scia-engineer/analysis/warning-stiffness-matrix-singular-structure-unstable); [Dlubal instability causes](https://www.dlubal.com/en/support-and-learning/support/faq/005345); [RISA stability](https://help.risa.com/risahelp/risa3d/Content/Stability/Stability.htm)

5. **Connectivity errors are a first-class instability source, not just a meshing inconvenience.** LUSAS states that connectivity problems commonly lead to unintended structural behavior and stiffness-matrix condition warnings/errors; compatible FE elements are principally connected by sharing common nodes at boundaries/corners. Oasys GSA identifies poorly restrained or improperly connected elements as one of two main stability-error categories. [LUSAS connectivity FAQ](https://www.lusas.com/user_area/faqs/connectivity.html); [Oasys GSA model debugging](https://docs.oasys-software.com/structural/gsa/version/10.2.12/tutorials/model-debugging/)

6. **Coincident-looking nodes can be disconnected if they are not actually merged/common.** Dlubal flags "no common node" as a frequent CAD-import issue where nodes appear co-located but differ slightly; LUSAS recommends checks for duplicate/overlapping geometry, point distances, hierarchy/selection browsing, and merge operations with appropriate tolerances. [Dlubal instability causes](https://www.dlubal.com/en/support-and-learning/support/faq/005345); [LUSAS connectivity FAQ](https://www.lusas.com/user_area/faqs/connectivity.html)

7. **Crossing members may not be connected unless an intersection node exists.** Dlubal explicitly identifies crossing unconnected members as an instability source and describes model checks that search for members crossing without a common node at the intersection. This is important for primitive member graphs because visual intersection is not equivalent to structural connectivity. [Dlubal instability causes](https://www.dlubal.com/en/support-and-learning/support/faq/005345)

8. **Disconnected substructures can be diagnosed by connectivity coloring, outline plots, deformed-shape inspection, and simple load cases.** LUSAS recommends geometry/mesh connectivity coloring, outline mesh plots, labels, visibility filtering, and a simple linear self-weight/deformed mesh check; disconnected unsupported pieces will move under free-body motion. Dlubal similarly suggests a pure dead-load linear-static run as an early modelling-stability check. [LUSAS connectivity FAQ](https://www.lusas.com/user_area/faqs/connectivity.html); [Dlubal instability causes](https://www.dlubal.com/en/support-and-learning/support/faq/005345)

9. **Solver reports should expose node/element references and instability DOFs.** SCIA reports the first unstable FE node and DOF/direction and provides selection commands and animation to locate it. Oasys GSA tells users to read the analysis report, note referenced elements/nodes, review support conditions, and then rerun or perform stability analysis. [SCIA singular stiffness FAQ](https://scia.net/en/support/faq/scia-engineer/analysis/warning-stiffness-matrix-singular-structure-unstable); [Oasys GSA model debugging](https://docs.oasys-software.com/structural/gsa/version/10.2.12/tutorials/model-debugging/)

10. **Unstable-mode/eigenvector visualization is a powerful explanation tool.** SCIA animates the instability shape after the warning; Dlubal suggests calculating an eigenvector for an unstable model to graphically display the affected component; Oasys GSA recommends model stability analysis when basic checks fail. [SCIA singular stiffness FAQ](https://scia.net/en/support/faq/scia-engineer/analysis/warning-stiffness-matrix-singular-structure-unstable); [Dlubal instability causes](https://www.dlubal.com/en/support-and-learning/support/faq/005345); [Oasys GSA model debugging](https://docs.oasys-software.com/structural/gsa/version/10.2.12/tutorials/model-debugging/)

11. **Instability diagnostics should distinguish modelling errors from true structural instability/second-order effects.** Dlubal notes that calculation aborts can indicate real instability due to overloading or modelling inaccuracies, and that a critical load factor below 1.0 in second-order/stability analysis indicates instability under the applied load. PyNite lists second-order effects as a third instability type and notes P-Delta checks can reveal otherwise hidden model issues. [Dlubal instability causes](https://www.dlubal.com/en/support-and-learning/support/faq/005345); [PyNite stability](https://pynite.readthedocs.io/en/latest/stability.html)

12. **Nonlinear member behavior can remove stabilizing elements and create load-case-specific mechanisms.** Dlubal gives the example of a frame stabilized by tension-only members: under vertical loading, tension members may enter small compression and be removed, leaving the model unstable. This implies stability must be checked per analysis case/combination, not only once globally. [Dlubal instability causes](https://www.dlubal.com/en/support-and-learning/support/faq/005345)

13. **Ill-conditioning is related but distinct from a pure mechanism.** Oasys GSA explains that ill-conditioned problems can produce large result changes from small input changes, especially when stiffnesses differ greatly; GSA debugging also lists elements much stiffer than other elements as a stability-error category. Fraia should flag extreme stiffness ratios separately from zero-stiffness mechanisms. [Oasys GSA ill-conditioning](https://docs.oasys-software.com/structural/gsa/version/10.2.12/references-theory/ill-conditioning/); [Oasys GSA model debugging](https://docs.oasys-software.com/structural/gsa/version/10.2.12/tutorials/model-debugging/)

14. **Automatic DOF locking/weak-spring workarounds are dangerous if hidden.** RISA notes it can lock discovered instabilities and continue, but reactions for those locks are not calculated and ignoring instabilities can be dangerous; it suggests testing by applying a restraint/reaction in the unstable DOF and checking whether the reaction is zero. Fraia should avoid silently stabilizing authored models without explicit diagnostics/provenance. [RISA stability](https://help.risa.com/risahelp/risa3d/Content/Stability/Stability.htm)

## Fraia implications

- **Model validation layer:** add pre-solve diagnostics for graph connectivity, isolated nodes/members/plates, duplicate/coincident nodes within tolerance, crossing members without shared nodes, missing properties, zero/near-zero lengths, and unassigned supports/loads.
- **DOF-level restraint audit:** for each connected component, determine available stiffness/restraint per relevant translational/rotational DOF; report likely rigid-body modes before solver assembly.
- **Release/hinge-chain audit:** at each node, inspect connected member end releases by DOF; warn when all connected members release the same rotational DOF or when supports plus releases leave the node free.
- **Torsion-specific checks:** flag members/chains whose local-axis torsion has no path to support or connected member stiffness, especially for pinned 3D frames and simple beams with both ends torsion-released.
- **Connectivity UX:** make visual intersection distinct from structural connectivity. Wiki examples should stress: crossing lines are not connected unless Fraia creates/resolves a shared node/member split.
- **Analysis topology visibility:** when authored members are split/discretised, diagnostics should reference both analysis node/element ids and authored objects such as `Member B1`, preserving provenance from analysis topology back to structural primitives.
- **Case-aware stability:** run stability/under-restraint checks per load case/combination where nonlinear elements, tension-only bracing, compression-only supports, or staged/conditional supports may change active stiffness.
- **Diagnostics artifacts:** persist a validation/stability report as a run artifact with component ids, suspect DOFs, source authored objects, analysis nodes/elements, severity, suggested fixes, and whether any stabilization was applied.
- **Visualization:** support unstable-mode/deformed-shape previews for diagnostics: arrows/animation for rigid-body motion, local spin/torsion, disconnected islands, and hinge-chain rotations.
- **No silent repair:** merge/split/weak-spring/fixity suggestions should be explicit proposals or downstream analysis settings, not invisible edits to authored structural truth.

## Cautions for wiki wording

- Do not imply every singular matrix is caused by missing supports; local releases, connectivity, properties, stiffness contrasts, nonlinear element activation, and true buckling can also be responsible.
- Do not present solver stabilization as validation. A stabilized solve may produce numbers while still representing a physically invalid or unintended model.
- Do not teach users to "just fix all DOFs"; restraints must match the real structure and intended analytical idealization.
- Distinguish authored structural objects (`Node`, `Member`, `Plate`, `SupportAssignment`, `ReleaseAssignment`, `LoadAssignment`) from finite-element analysis nodes/elements used by the solver.
- CAD tolerance fixes can merge unintended geometry; require preview/diff and provenance when suggesting merges.

## Suggested Fraia wiki cross-links

- Structural object model: `Node`, `Member`, `Plate`, `SupportAssignment`, `ReleaseAssignment`, `LoadAssignment`.
- Analysis topology and discretisation: authored member vs split analysis elements.
- Load paths and support reactions.
- Model validation/checks.
- Releases, hinges, and connection idealization.
- Connectivity and tolerance policy.
- Run artifacts and provenance.
- Second-order/P-Delta and stability analysis.

## Sources

- Kept: PyNite Stability (https://pynite.readthedocs.io/en/latest/stability.html) — open documentation with clear taxonomy: rigid-body motion, nodal instability, second-order effects.
- Kept: LUSAS Connectivity FAQ (https://www.lusas.com/user_area/faqs/connectivity.html) — practical public source on node-sharing connectivity diagnostics and fixes.
- Kept: RISA-3D Stability help (https://help.risa.com/risahelp/risa3d/Content/Stability/Stability.htm) — detailed examples of releases, unconnected elements, torsional mechanisms, and stabilization cautions.
- Kept: Oasys GSA Model debugging (https://docs.oasys-software.com/structural/gsa/version/10.2.12/tutorials/model-debugging/) — public workflow for interpreting errors/warnings and checking supports/connectivity.
- Kept: Oasys GSA Ill-conditioning theory (https://docs.oasys-software.com/structural/gsa/version/10.2.12/references-theory/ill-conditioning/) — public theory note distinguishing numerical conditioning from pure mechanisms.
- Kept: Dlubal FAQ 005345, Finding and Fixing Calculation Instabilities (https://www.dlubal.com/en/support-and-learning/support/faq/005345) — broad public checklist: supports, torsion, missing connections, no common nodes, hinge chains, nonlinear/tension-only effects, eigenvectors.
- Kept: SCIA FAQ, singular stiffness matrix warning (https://scia.net/en/support/faq/scia-engineer/analysis/warning-stiffness-matrix-singular-structure-unstable) — public documentation on DOF-specific warning localization and animation.
- Dropped: EnduraSim Nastran/Femap singularity PDF — useful but vendor/training-style PDF and less directly relevant to Fraia wiki seed than public HTML docs.
- Dropped: MSC Nastran PDF user guide — authoritative but large commercial manual; not needed after cleaner public sources covered diagnostics.
- Dropped: LinkedIn article on unstable structural models — secondary/social-source commentary duplicated Dlubal material.
- Dropped: structures.lv instabilities article — practical commentary but less authoritative and partly anecdotal; retained only as background, not cited.
- Dropped: Abaqus troubleshooting PDF mirror — book/PDF mirror with uncertain redistribution status; avoided for open/public-source cleanliness.

## Gaps

- Public sources rarely give implementable algorithms for detecting every mechanism before matrix assembly; Fraia may need internal prototypes using graph connectivity, DOF stiffness contribution maps, rank/eigen checks, and solver pivot diagnostics.
- Source guidance is mostly software-help oriented, not code/spec level. Next step: design Fraia diagnostic schema and validate against small benchmark models: free rigid body, cantilever with released free end, pinned-base + member release, all-pinned truss joint, crossing-unconnected members, duplicate near-coincident nodes, disconnected plate/frame island, tension-only bracing removed by load case, and extreme stiffness-ratio ill-conditioning.
