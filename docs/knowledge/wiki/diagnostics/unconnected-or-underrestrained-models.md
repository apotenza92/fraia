---
title: Unconnected or Underrestrained Models
status: compiled
trust_level: compiled
domain: diagnostics
applies_to:
  - pre-solve structural model diagnostics
  - solver instability explanations
  - Fraia agent guidance
not_applicable_to:
  - automatic model repair without user adoption
  - code-compliant design checks
  - software-specific troubleshooting workflows
jurisdiction_or_standard_context: concept guidance from Fraia compiled knowledge, academic/open sources, and source-scoped software documentation; not a code check
last_compiled: 2026-05-07
source_count: 3
citation_policy: required
owner: agent-maintained
---

# Unconnected or Underrestrained Models

## Summary

An unconnected or underrestrained model has topology or boundary conditions that allow one or more bodies, nodes, members, plates, or degrees of freedom to move without sufficient resistance. The symptom may be a singular stiffness matrix, rigid-body mode, local mechanism, zero/near-zero reaction, implausible displacement, or a solver warning.

For Fraia, the diagnostic goal is not to "make the solver run" by adding hidden supports or constraints. The goal is to map the symptom back to authored `Node`, `Member`, `Plate`, `SupportAssignment`, `ReleaseAssignment`, constraints, and resolved topology so the user or agent can make an explicit modeling decision.

## Scope / non-scope

This page covers concept-level diagnostics for unconnected or underrestrained Fraia models.

It does not provide automatic repair algorithms, software UI steps, code design checks, or permission to silently mutate authored project state.

## Key concepts

### Visual contact is not connectivity

Members or plates that visually cross, overlap, or touch may still be disconnected in the resolved model if they do not share a node, constraint, or connectivity relationship. Source-scoped FE documentation treats coincident/unconnected geometry as a common connectivity issue. [S1][S3]

Fraia should make authored geometry and resolved connectivity inspectable.

### Underrestraint is DOF-specific

Underrestraint occurs when the relevant translational or rotational degree of freedom is not restrained by supports, constraints, connected stiffness, or member/plate topology. Stability guidance distinguishes support/reaction counts from actual restraint effectiveness and geometry stability. [S2]

Fraia diagnostics should identify the free DOF and the object or connected component it belongs to.

### Disconnected groups can each need restraint

A model may contain multiple disconnected subgraphs. One part can be properly supported while another is floating or only locally connected. [S1][S3]

Fraia should report connected components or unresolved isolated objects before interpreting global results.

### Ineffective supports can look valid

A support may exist but restrain the wrong direction, wrong coordinate frame, wrong node, wrong component, or wrong stage/load case. It may also be bypassed by disconnected topology. [S1][S2]

Fraia agents should diagnose support direction and resolved attachment, not only support presence.

### Constraints should not hide missing connectivity

Rigid links, diaphragms, or multi-point constraints can intentionally connect DOFs. They can also mask duplicate nodes, missing members, or unsupported load paths if added as opaque stabilizers.

Fraia should preserve whether a constraint represents engineering intent or a temporary diagnostic hypothesis.

## Engineering guidance for Fraia agents

- Start by identifying disconnected components, isolated nodes, free DOFs, and supports not connected to loaded/stiff components.
- Check whether visually intersecting members/plates share resolved nodes or explicit constraints.
- Check support DOFs, coordinate frames, support directions, and whether supports attach to the intended resolved nodes.
- Check whether releases, constraints, rigid links, or diaphragms create or hide free DOFs.
- Do not silently add supports, merge nodes, or rigidly tie objects to make analysis pass.
- If proposing a fix, state whether it changes authored state, resolved topology, or only a diagnostic run hypothesis.
- Preserve the failed run artifact and warning context for comparison.

## Tradeoffs / cautions

- Automatic merging can fix accidental gaps but can also destroy intended releases, expansion joints, offsets, or connection eccentricities.
- Adding a weak stabilizing spring may help locate a mechanism but should downgrade result trust.
- A model can be globally stable but locally disconnected or locally unstable.
- A reaction balance check can pass while an isolated unloaded component remains disconnected.
- Some apparent underrestraints are intentional if the model is a submodel with boundary conditions supplied later.

## Source-backed claims

- Solver instability can indicate unconstrained DOFs, disconnected topology, over-releases, or local mechanisms. [S1]
- Stability depends on effective supports, geometry, and restraint, not only reaction counts. [S2]
- Coincident or crossing geometry can remain disconnected without shared topology or connectivity definitions. [S3]
- Instability diagnostics should map causes back to supports, releases, connectivity, constraints, and resolved topology. [S1]
- Artificial stabilizing stiffness or hidden fixes should be treated as diagnostic metadata rather than valid model repair. [S1]

## Open questions / weak evidence

- Fraia still needs final resolved-connectivity graph reports, component IDs, free-DOF reporting, and visual diagnostic overlays.
- The page intentionally avoids vendor-specific warning IDs and UI workflows.
- Automatic repair policy needs future product decisions.

## Related pages

- [Instability mechanisms](instability-mechanisms.md)
- [Static determinacy and restraint](../analysis/static-determinacy-and-restraint.md)
- [Reactions and support idealisation](../analysis/reactions-and-support-idealisation.md)
- [Supports, restraints, and releases](../modeling/supports-restraints-and-releases.md)
- [Constraints, rigid links, and diaphragms](../modeling/constraints-rigid-links-and-diaphragms.md)
- [Member end releases](../modeling/member-end-releases.md)

## Sources

- [S1] Fraia compiled wiki, *Instability Mechanisms*. Path: `docs/knowledge/wiki/diagnostics/instability-mechanisms.md`. Source type: Fraia compiled diagnostic page. Consulted: 2026-05-07. Reliability/limits: useful Fraia-specific synthesis; includes source-scoped software/manual evidence and should be read with its source limits.
- [S2] Felix Udoeyo / Engineering LibreTexts, *Equilibrium Structures, Support Reactions, Determinacy and Stability of Beams and Frames*. URL: https://eng.libretexts.org/Bookshelves/Civil_Engineering/Structural_Analysis_(Udoeyo)/01:_Chapters/1.03:_Equilibrium_Structures_Support_Reactions_Determinacy_and_Stability_of_Beams_and_Frames. Source type: open educational structural-analysis text. Retrieved: 2026-05-07. Reliability/limits: useful stability/restraint framing; introductory and not a finite-element diagnostic algorithm.
- [S3] LUSAS, *Connectivity FAQ*. URL: https://www.lusas.com/user_area/faqs/connectivity.html. Source type: public software FAQ. Retrieved: 2026-05-07. Reliability/limits: useful source-scoped FE connectivity examples; software-specific workflow details are not Fraia behavior.
