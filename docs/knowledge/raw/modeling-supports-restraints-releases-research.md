# Raw research brief: supports, restraints, boundary conditions, and releases

Retrieval date: 2026-05-06. Scope limited to open/public web sources and Fraia local project docs. No textbook passages copied; claims are summarized.

## Scope

- Structural-analysis modeling concepts for supports, restraints, boundary conditions, support displacements, springs, multi-point constraints, rigid diaphragm constraints, and member/end releases.
- Practical vocabulary for Fraia wiki seed and implications for Fraia authored primitives: `Node`, `Member`, `Plate`, `SupportAssignment`, `ReleaseAssignment`.
- Line/member/frame modeling first; notes include 3D DOF conventions because Fraia is 3D-canonical.

## Non-scope

- Code-specific design rules for real connections/foundations.
- Detailed derivations, stiffness matrix math, or solver implementation recipes.
- Proprietary/manual-only workflows beyond what is visible in open public documentation.
- Copying textbook tables/figures or closed-source training material.

## Source-backed claims

1. **A support/restraint is best represented as constrained degrees of freedom, not just as a named icon.** Planar supports are commonly described using translational DOFs and rotation; 3D nodes extend this to translations in x/y/z and rotations about x/y/z. TU Delft explicitly describes planar DOFs as `u_x`, `u_z`, and `phi`, and 3D DOFs as x/y/z plus rotations; SkyCiv similarly describes six possible DOFs at a point. Sources: TU Delft supports notes; SkyCiv DOF tutorial.

2. **Classic support names are shorthand for DOF restrictions.** A pin/hinge prevents translation but allows rotation; a roller allows rotation and translation along the supporting surface while restraining normal movement; a fixed support prevents translation and rotation. Sources: LibreTexts Structural Analysis chapter; TU Delft supports notes.

3. **Support reactions exist only in restrained/prescribed directions.** LibreTexts frames supports as external reactions created by constraints; SkyCiv states a support will contain a reaction for each fixed DOF. Source: LibreTexts reactions/supports and SkyCiv restraint codes.

4. **Boundary conditions can be homogeneous or non-homogeneous.** OpenSees separates single-point constraints for individual node DOFs and notes that they may prescribe zero response or non-zero/time-varying response. TU Delft also notes that support displacement is a prescribed non-zero displacement and still creates a reaction in that direction. Sources: OpenSees SP constraints docs; OpenSees `sp` command; TU Delft support displacement.

5. **Named support conditions should not hide coordinate-frame dependence.** Roller/link/bar supports constrain movement relative to a surface, line, or support direction, not necessarily global axes. TU Delft notes supports can be made in any direction, including sloped environments; LibreTexts describes a link reaction along the link axis. Sources: TU Delft supports notes; LibreTexts Structural Analysis chapter.

6. **Member releases are element/member-end DOF modifiers, commonly in the member local coordinate system.** Autodesk Inventor frame analysis says releases are defined by specifying DOFs at beam start/end and that releases are defined in the beam coordinate system. This distinguishes end releases from node support restraints, which belong at a node or support assignment. Source: Autodesk release documentation.

7. **A release is not always binary; partial stiffness and elastic coefficients are common modeling options.** Autodesk exposes elastic stiffness coefficients and partial stiffness coefficients where 0.0 means released and 1.0 means no release. SkyCiv also includes spring restraints as a third state between fixed and released. Sources: Autodesk release documentation; SkyCiv DOF tutorial.

8. **Releases can create modeling singularities or unintended mechanisms if over-applied at a joint.** Autodesk cautions that when two or more adjoining beams meet, one beam should remain fixed without a release at that beam end because the constraint already contains boundary-condition information for the adjoining beam. Source: Autodesk release documentation.

9. **Analysis-model constraints include relationships between nodes, not only node-to-ground supports.** OpenSees documents multi-point constraints such as `equalDOF`, where selected DOFs at a constrained node match a retained node, and `rigidDiaphragm`, where constrained nodes move as if in a rigid plane with a retained node. Sources: OpenSees model commands, equalDOF, rigidDiaphragm.

10. **Springs/restraint stiffness can be modeled as boundary restraint states or as explicit short/zero-length elements.** OpenSees describes a zeroLength element as two coincident nodes connected by uniaxial material relationships, similar to springs in selected DOFs. This suggests Fraia should preserve whether a restraint is an ideal boundary condition, a spring support, or an explicit connection/foundation element. Source: OpenSees zeroLength element docs.

## Fraia-specific implications

- Keep `SupportAssignment` as a first-class authored object on nodes/regions, storing restrained/prescribed DOFs explicitly rather than only `pin`, `roller`, `fixed` names.
- Treat support archetypes (`fixed`, `pinned`, `roller`, `guided`, `spring`, `settlement`) as UI/library presets that resolve to explicit DOF constraints, coordinate frame, stiffness/prescribed displacement values, and provenance.
- Keep `ReleaseAssignment` separate from `SupportAssignment`; releases attach to `Member` ends or realized analysis elements and should include local-axis basis, end (`start`/`end`), DOF mask, optional stiffness, and whether the release is authored or generated.
- Store coordinate frame for every constraint/release: global, member-local, support-plane-local, or explicitly defined frame. Avoid implying all rollers act in global vertical.
- Preserve authored vs resolved separation: authored high-level support names should resolve into immutable run artifacts with exact solver DOFs, constraint equations, spring elements, and warnings.
- Add validation checks for common mechanisms: under-restraint, all reactions parallel/concurrent, released all adjoining member ends, isolated free DOFs, duplicate constraints, and inconsistent multi-point constraints.
- Consider a `ConstraintAssignment` or internal resolved layer that covers non-ground constraints (`equalDOF`, rigid diaphragm, rigid links), while keeping the user-facing `SupportAssignment` for support-to-ground/bearing conditions.
- For plates/diaphragms, model rigid/semi-rigid diaphragm behavior as a constraint/idealization choice, not as a plate object replacement.

## Cautions / contradictions / terminology traps

- **Pin/hinge ambiguity:** educational statics often says a pin allows rotation about any axis, but software examples may define a 3D pin as a particular 6-character DOF code. Fraia should show exact DOFs rather than rely on ambiguous icons.
- **2D vs 3D mismatch:** planar beam formulas use three equilibrium equations and three nodal DOFs; Fraia’s 3D canonical model needs six DOFs where applicable.
- **Support vs release confusion:** a support restrains a node relative to ground or another reference; a release changes force/moment transfer at a member/element end. UI labels should make this distinction explicit.
- **Local axes matter:** releases and some support directions are local-frame definitions. Incorrect local axes can invert or move the intended released/restrained direction.
- **Partial fixity is modeling judgment:** spring stiffness/partial release values need engineering basis; arbitrary coefficients can give false precision.
- **Commercial docs are practical but product-specific:** Autodesk/SkyCiv examples are useful for common software conventions, but Fraia should not clone their exact UI codes or semantics without review.

## Suggested wiki cross-links

- `Node` / nodal degrees of freedom
- `Member` local axes and orientation
- `SupportAssignment`
- `ReleaseAssignment`
- `LoadAssignment` and imposed displacement/load patterns
- Boundary conditions vs constraints
- Springs and partial fixity
- Rigid diaphragm / multi-point constraints
- Stability, determinacy, and mechanisms
- Authored model vs resolved analysis model vs run artifacts
- Solver adapter mapping and provenance

## Open questions

1. Should Fraia expose a generic `ConstraintAssignment` authored object for rigid links/diaphragms/equalDOF, or keep these as resolved artifacts from higher-level concepts initially?
2. What canonical DOF order should Fraia use in files and UI: `[ux, uy, uz, rx, ry, rz]` appears conventional, but it should be fixed early.
3. How should Fraia represent tension-only/compression-only/uplift-only supports: as nonlinear support laws, springs, contact/gap elements, or library archetypes that resolve later?
4. What minimum validation should block solving vs warn only for mechanisms/duplicates/over-releases?
5. How should plate edge supports and area supports be authored: node-expanded assignments, geometric-region assignments, or both?

## Source list

- **Engineering LibreTexts, “1.3: Equilibrium Structures, Support Reactions, Determinacy and Stability of Beams and Frames”** — https://eng.libretexts.org/Bookshelves/Civil_Engineering/Structural_Analysis_(Udoeyo)/01:_Chapters/1.03:_Equilibrium_Structures_Support_Reactions_Determinacy_and_Stability_of_Beams_and_Frames — Retrieved 2026-05-06. Source type/context: open educational resource; useful for support types, reactions, determinacy/stability. Limits: introductory/statics framing; images/tables not reused.
- **TU Delft Open Interactive Textbook, “Supports — Bridging course Structural Mechanics”** — https://oit.tudelft.nl/CT1000/2024/external/mechanics-BSc/book/support_internal_forces/model/supports.html — Retrieved 2026-05-06. Source type/context: university open course notes; useful for DOF notation, support direction, rolling clamped support, support displacement. Limits: concise course notes; not a full design standard.
- **SkyCiv, “Degrees of Freedom and Restraint Codes”** — https://skyciv.com/education/explaining-degrees-of-freedom/ — Retrieved 2026-05-06. Source type/context: public commercial education article; useful for 6-DOF vocabulary and fixed/released/spring restraint codes. Limits: product-oriented and simplified.
- **Autodesk Inventor Help, “Define a release in a frame structure”** — https://help.autodesk.com/cloudhelp/2026/ENU/Inventor-Help/files/GUID-2E87FB0F-06D2-44D7-824B-EB514DD155DD.htm — Retrieved 2026-05-06. Source type/context: public software documentation; useful for member-end releases, local beam coordinate system, partial/elastic stiffness, over-release caution. Limits: Inventor-specific terminology such as uplift symbols.
- **OpenSees Documentation, “Model Commands”** — https://opensees.github.io/OpenSeesDocumentation/user/manual/modelCommands.html — Retrieved 2026-05-06. Source type/context: open-source solver documentation; useful for distinguishing nodes, elements, constraints, loads, SP and MP constraints. Limits: solver/API-level, not user-facing structural semantics.
- **OpenSees Documentation, “SP_Constraint Commands”** — https://opensees.github.io/OpenSeesDocumentation/user/manual/model/spConstraints.html — Retrieved 2026-05-06. Source type/context: open-source solver documentation; useful for homogeneous vs non-homogeneous single-point constraints. Limits: command-oriented.
- **OpenSees Documentation, “Sp Command”** — https://opensees.github.io/OpenSeesDocumentation/user/manual/model/pattern/PlainPatternloadcommands/sp.html — Retrieved 2026-05-06. Source type/context: open-source solver documentation; useful for prescribed nodal DOF value in a load pattern. Limits: OpenSees-specific load-factor semantics.
- **OpenSees Documentation, “EqualDOF Constraints”** — https://opensees.github.io/OpenSeesDocumentation/user/manual/model/mp_constraint/equalDOF.html — Retrieved 2026-05-06. Source type/context: open-source solver documentation; useful for multi-point constraint concept. Limits: low-level retained/constrained-node terminology.
- **OpenSees Documentation, “Rigid Diaphragm”** — https://opensees.github.io/OpenSeesDocumentation/user/manual/model/mp_constraint/rigidDiaphragm.html — Retrieved 2026-05-06. Source type/context: open-source solver documentation; useful for diaphragm-like node constraints. Limits: assumes rigid plane idealization.
- **OpenSees Documentation, “ZeroLength Element”** — https://opensees.github.io/OpenSeesDocumentation/user/manual/model/elements/zeroLength.html — Retrieved 2026-05-06. Source type/context: open-source solver documentation; useful for spring-like force-deformation links at coincident nodes. Limits: implementation-level modeling primitive.
- **Abaqus 2024 Documentation mirror, “Frame Elements”** — https://docs.software.vt.edu/abaqusv2024/English/SIMACAEELMRefMap/simaelm-c-frame.htm — Retrieved 2026-05-06. Source type/context: public documentation mirror for commercial FEA; useful for frame element/end force/plastic-hinge context. Limits: commercial product mirror, not primary Fraia target; used only for broad convention context.
- **Fraia local doc, `docs/structural-app-object-model.md`** — Retrieved 2026-05-06. Source type/context: local project canonical doc; establishes `SupportAssignment` and `ReleaseAssignment` as authored object types. Limits: draft architecture, not external evidence.
- **Fraia local doc, `docs/engineering-core.md`** — Retrieved 2026-05-06. Source type/context: local project canonical doc; supports authored/resolved/run separation and primitive-first modeling. Limits: project intent, not structural authority.

## Dropped / not relied on

- Duplicate LibreTexts support pages from aerospace/statics shelves — redundant with the civil structural analysis page.
- Older Abaqus element-release mirrors — useful but redundant with Autodesk/OpenSees and not needed for concise seed.
- OpenSees Doxygen class-reference pages — too implementation-heavy compared with user documentation pages.
