---
title: Supports, Restraints, and Releases
status: compiled
trust_level: compiled
domain: modeling
applies_to:
  - concept-stage structural modeling
  - Fraia agent guidance
not_applicable_to:
  - code-compliant design checks
  - project-specific engineering approval
jurisdiction_or_standard_context: concept guidance from public/open sources; not a code check
last_compiled: 2026-05-06
source_count: 5
citation_policy: required
owner: agent-maintained
---

# Supports, Restraints, and Releases

## Summary

Supports, restraints, constraints, and member releases should be understood as degree-of-freedom assumptions with coordinate frames, stiffness, and provenance, not only as named icons.

## Scope / non-scope

Covers concept-level boundary conditions and release semantics for structural analysis models. It does not define Fraia file schemas or real connection/foundation design rules.

## Key concepts

- Supports restrain or prescribe nodal degrees of freedom relative to ground or another reference.
- Releases modify force/moment transfer at member ends, normally in member-local axes.
- Springs and partial fixity lie between fixed and released assumptions.
- Multi-point constraints and rigid diaphragms relate nodes to other nodes, not just to ground.

## Engineering guidance for Fraia agents

- Store exact restrained/prescribed DOFs, coordinate frame, stiffness or prescribed displacement, and provenance.
- Keep `SupportAssignment` separate from `ReleaseAssignment`.
- Warn when named presets hide local-axis or support-plane dependence.
- Validate common mechanisms caused by over-releases, all-pinned joints, duplicate constraints, and isolated free DOFs.

## Tradeoffs / cautions

- Do not equate “pinned” with identical behavior in every 2D/3D context.
- Do not use arbitrary partial-stiffness values without source or design intent.
- Do not silently convert unstable release patterns into fixed behavior.

## Source-backed claims

- Support and restraint names are shorthand for constrained or prescribed DOFs [S1][S2].
- Support reactions occur in restrained/prescribed directions [S1][S3].
- Member releases are member-end DOF modifiers and are often local-coordinate assumptions [S4].
- Multi-point constraints and rigid diaphragms are analysis constraints between nodes, not ordinary supports [S5].

## Open questions / weak evidence

- Fraia still needs a final canonical DOF ordering and authored/resolved representation for generic constraints.
- Nonlinear supports such as uplift-only or compression-only need future modeling decisions.

## Related pages

- [Finite-element idealisation](finite-element-idealisation.md)
- [Local and global coordinate systems](local-and-global-coordinate-systems.md)
- [Member end releases](member-end-releases.md)
- [Constraints, rigid links, and diaphragms](constraints-rigid-links-and-diaphragms.md)
- [Reactions and support idealisation](../analysis/reactions-and-support-idealisation.md)
- [Static determinacy and restraint](../analysis/static-determinacy-and-restraint.md)
- [Matrix stiffness method](../analysis/matrix-stiffness-method.md)
- [Instability mechanisms](../diagnostics/instability-mechanisms.md)
- [Load paths](../analysis/load-paths.md)
- [Raw research note](../../raw/modeling-supports-restraints-releases-research.md)

## Sources

- [S1] Engineering LibreTexts, *Equilibrium Structures, Support Reactions, Determinacy and Stability*. URL: https://eng.libretexts.org/Bookshelves/Civil_Engineering/Structural_Analysis_(Udoeyo)/01:_Chapters/1.03:_Equilibrium_Structures_Support_Reactions_Determinacy_and_Stability_of_Beams_and_Frames. Source type: open educational resource. Retrieved: 2026-05-06. Reliability/limits: introductory statics framing.
- [S2] TU Delft Open Interactive Textbook, *Supports*. URL: https://oit.tudelft.nl/CT1000/2024/external/mechanics-BSc/book/support_internal_forces/model/supports.html. Source type: university open course notes. Retrieved: 2026-05-06. Reliability/limits: concise course notes.
- [S3] SkyCiv, *Degrees of Freedom and Restraint Codes*. URL: https://skyciv.com/education/explaining-degrees-of-freedom/. Source type: public commercial education. Retrieved: 2026-05-06. Reliability/limits: simplified/product-oriented.
- [S4] Autodesk Inventor Help, *Define a release in a frame structure*. URL: https://help.autodesk.com/cloudhelp/2026/ENU/Inventor-Help/files/GUID-2E87FB0F-06D2-44D7-824B-EB514DD155DD.htm. Source type: public software documentation. Retrieved: 2026-05-06. Reliability/limits: product-specific but useful for release conventions.
- [S5] OpenSees Documentation, *Model Commands / Constraints*. URL: https://opensees.github.io/OpenSeesDocumentation/user/manual/modelCommands.html. Source type: open-source solver documentation. Retrieved: 2026-05-06. Reliability/limits: solver/API-level terminology.
