---
title: Member End Releases
status: compiled
trust_level: compiled
domain: modeling
applies_to:
  - frame and truss modeling assumptions
  - authored member to resolved analysis topology
  - Fraia agent guidance
not_applicable_to:
  - final connection design
  - code-specific joint classification
  - nonlinear plastic hinge modeling
jurisdiction_or_standard_context: concept guidance from professional/open, academic, and solver-scoped sources; not a code check
last_compiled: 2026-05-07
source_count: 3
citation_policy: required
owner: agent-maintained
---

# Member End Releases

## Summary

A member end release is an analysis idealisation that removes or modifies transfer of selected force or moment components between a member end and the connected node/joint. It is not the same thing as a support restraint, and it is not just a drawing symbol.

For Fraia, `ReleaseAssignment` should preserve the authored `Member`, member end, released component, coordinate frame/local axis, partial stiffness if any, load-path intent, and provenance. During realization, releases must be mapped carefully onto analysis elements without losing the distinction between authored members and split solver elements.

## Scope / non-scope

This page covers concept-level member end release semantics for Fraia agents.

It does not define steel connection design, code-specific joint classification, plastic hinge modeling, nonlinear release behavior, or final Fraia schema enums.

## Key concepts

### Releases modify member-to-node transfer

Supports restrain nodes or bodies relative to ground or another reference. Member end releases modify what a member end can transfer to its connected node/joint. A moment release, for example, idealises that the member end does not transfer the corresponding bending moment component. [S2][S3]

Fraia should keep `ReleaseAssignment` separate from `SupportAssignment` because they act at different modeling boundaries.

### Release components are axis-dependent

Frame/member releases are commonly specified at member ends and in member or element local axes. Solver documentation commonly distinguishes releases by end and local bending axis. [S3]

Fraia agents should never say "release the moment" without saying which end, which axis/component, and which coordinate frame.

### Releases affect stiffness and equivalent loads

In stiffness-method frame analysis, releases are not a post-processing display choice. Academic treatment of member end releases describes modifying member stiffness and equivalent joint/load vectors to account for released end forces. [S2]

Fraia should therefore treat releases as resolved-model inputs that affect analysis results and run artifacts.

### Connection fixity is rarely binary in reality

Professional steel guidance commonly classifies joint behavior as nominally pinned, rigid/continuous, or semi-continuous/semi-rigid, with joint stiffness affecting internal force distribution and deformation. [S1]

Fraia can use pinned/fixed display labels, but agents should preserve whether the assumption is perfect release, rigid transfer, partial stiffness, or source-scoped engineering judgement.

### Releases can create mechanisms

A release removes stiffness/force-transfer paths. Releasing too many components, releasing both ends of critical members, or combining releases with weak supports can create a mechanism or make a model underrestrained. [S1][S2]

Fraia should diagnose over-release by mapping instability back to authored members, intended joints, supports, constraints, and load path.

### Authored member ends are not always analysis element ends

An authored `Member` may be discretised into multiple analysis elements. A release intended at the physical member end should not be accidentally applied at every internal analysis-element boundary unless an internal hinge or joint was explicitly authored.

This is central to Fraia's authored/resolved/run boundary.

## Engineering guidance for Fraia agents

- Use `ReleaseAssignment` for member-end force/moment transfer assumptions; use `SupportAssignment` for support restraints.
- Identify the target authored `Member`, end, local/global frame, released component, and whether the release is full or partial.
- Preserve why the release exists: intended simple connection, truss idealisation, brace/tie behavior, construction assumption, or imported model provenance.
- Do not blanket-release rotations on every visually diagonal or secondary member.
- When a member is split, apply physical-end releases only at the intended authored member ends unless an internal hinge is explicitly authored.
- Treat partial fixity as a stiffness assumption requiring provenance, not a convenient arbitrary number.
- After adding releases, check stability, deflected shape, reactions, moment diagrams, axial force paths, and zero/near-zero stiffness warnings.
- Report run results with release metadata so downstream design actions know which force components were suppressed or reduced by modeling assumptions.

## Tradeoffs / cautions

- Full releases make simple connection assumptions explicit, but can remove needed frame stability.
- Rigid end assumptions can make models stiff and economical in analysis, but require physical connection stiffness and strength.
- Partial fixity can better represent real behavior, but introduces stiffness values that need source or design intent.
- A release that is valid for gravity load may be wrong for lateral load, uplift, reversal, construction stage, or stability checks.
- Software defaults for releases/fixity are not engineering intent.

## Source-backed claims

- Joint behavior affects internal force/moment distribution and overall deformation. [S1]
- Nominally pinned, rigid/continuous, and semi-continuous/semi-rigid joint idealisations are common professional categories. [S1]
- Member end releases can be handled in stiffness-method analysis by modifying member stiffness and equivalent load vectors for released end forces. [S2]
- Solver documentation commonly represents beam-column moment releases by member end and local bending axis. [S3]
- Release assumptions should be checked because default/support/release inputs strongly affect model verification. [S1]

## Open questions / weak evidence

- Fraia still needs final release-component names, local-axis conventions, and partial-stiffness schema.
- Code-specific connection classification and stiffness calculation are deferred to steel connection pages.
- Nonlinear plastic hinges and rotational capacity are out of scope for this baseline page.

## Related pages

- [Supports, restraints, and releases](supports-restraints-and-releases.md)
- [Constraints, rigid links, and diaphragms](constraints-rigid-links-and-diaphragms.md)
- [Connection fixity and partial restraint modeling](connection-fixity-and-partial-restraint.md)
- [Local and global coordinate systems](local-and-global-coordinate-systems.md)
- [Finite-element idealisation](finite-element-idealisation.md)
- [Truss analysis and two-force members](../analysis/truss-analysis-and-two-force-members.md)
- [Matrix stiffness method](../analysis/matrix-stiffness-method.md)
- [Instability mechanisms](../diagnostics/instability-mechanisms.md)

## Sources

- [S1] SteelConstruction.info / Steel Construction Institute and British Constructional Steelwork Association, *Modelling and analysis*. URL: https://steelconstruction.info/Modelling_and_analysis. Source type: professional steel construction guidance. Retrieved: 2026-05-07. Reliability/limits: strong practical modeling/joint behavior guidance; UK/Eurocode context and not Fraia schema guidance.
- [S2] N. S. Trahair and M. A. Bradford, *Member end releases in framed structures*. URL: https://www.sciencedirect.com/science/article/abs/pii/004579499390214X. Source type: peer-reviewed article abstract in Computers & Structures. Retrieved: 2026-05-07. Reliability/limits: useful stiffness-method release framing from abstract-level access; detailed algorithm not copied or relied on.
- [S3] OpenSees Documentation, *Elastic Beam Column Element*. URL: https://opensees.github.io/OpenSeesDocumentation/user/manual/model/elements/elasticBeamColumn.html. Source type: open-source solver documentation. Retrieved: 2026-05-07. Reliability/limits: useful source-scoped evidence for end/local-axis release conventions; software-specific syntax is not Fraia behavior.
