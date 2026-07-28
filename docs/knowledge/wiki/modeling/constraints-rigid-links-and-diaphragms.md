---
title: Constraints, Rigid Links, and Diaphragms
status: compiled
trust_level: compiled
domain: modeling
applies_to:
  - resolved analysis topology
  - lateral load-path and diaphragm assumptions
  - Fraia agent guidance
not_applicable_to:
  - diaphragm design checks
  - code-specific seismic force distribution
  - final finite-element diaphragm mesh rules
jurisdiction_or_standard_context: concept guidance from public solver documentation and professional technical guidance; not a code check
last_compiled: 2026-05-07
source_count: 3
citation_policy: required
owner: agent-maintained
---

# Constraints, Rigid Links, and Diaphragms

## Summary

Constraints relate or prescribe degrees of freedom in the analysis model. They are different from member releases and ordinary supports: a support restrains motion relative to ground or another reference, a member release changes member-end force transfer, and a multi-point constraint relates motion between nodes.

For Fraia, rigid links and diaphragm assumptions should be durable resolved-topology objects with provenance, not invisible solver conveniences. They affect stiffness, load distribution, reactions, stability diagnostics, and downstream design actions.

## Scope / non-scope

This page covers concept-level constraints, rigid links, and diaphragm idealisations for Fraia agents.

It does not define seismic diaphragm design, collector/chord capacity checks, code-specific rigid/flexible diaphragm criteria, or a final Fraia constraint schema.

## Key concepts

### Single-point and multi-point constraints are different

Solver documentation commonly distinguishes single-point constraints, which restrain or prescribe degrees of freedom at one node, from multi-point constraints, which impose relationships between degrees of freedom at multiple nodes. [S1]

Fraia should preserve this distinction. A support-like restraint is not the same as tying two nodes together.

### Rigid links tie node motions

A rigid link is an idealisation that relates the motion of one node to another, often making a slave/constrained node follow a retained/master node with rigid-body kinematics. This is an inter-node constraint, not a member end release. [S1]

Fraia agents should use rigid links sparingly and with provenance because they can create hidden stiffness paths or mask disconnected geometry.

### Rigid diaphragms constrain in-plane motion

A rigid diaphragm idealisation constrains nodes in a floor or roof plane to share in-plane rigid-body motion through a retained node or reference motion. Solver documentation represents this as a multi-point constraint under a chosen perpendicular direction. [S2]

Fraia should store the diaphragm plane, retained/control node, constrained nodes, affected DOFs, and coordinate frame.

### Diaphragm behavior is a modeling assumption

Professional diaphragm guidance distinguishes diaphragm behavior such as flexible, rigid, and semi-rigid depending on relative stiffness and system behavior. Diaphragms also transfer inertial/lateral forces to vertical resisting elements through chords, collectors, and connections. [S3]

Fraia should not infer rigid diaphragm behavior from a `Plate` role or roof/floor label alone.

### Constraints change the resolved topology

Constraints can condense, tie, or prescribe relationships among DOFs before solution. They therefore affect stiffness assembly, reactions, internal force distribution, mode shapes, and diagnostics. [S1][S2]

Fraia should make constraints visible in resolved topology and run metadata, especially when explaining unexpected load paths or stability.

### Constraints can fix or hide modeling problems

Rigid constraints can intentionally represent rigid offsets, diaphragms, master/slave joints, or connection zones. They can also accidentally hide duplicate nodes, disconnected members, missing plates, unsupported load paths, or over-flexible lateral systems.

Fraia diagnostics should explain whether a constraint is carrying engineering intent or acting as an opaque patch.

## Engineering guidance for Fraia agents

- Separate `SupportAssignment`, `ReleaseAssignment`, and inter-node constraint concepts.
- For every constraint, record retained/constrained nodes, affected DOFs, coordinate frame, stiffness/rigidity assumption, and provenance.
- Do not silently add rigid links to make disconnected geometry solve.
- Do not infer rigid diaphragm behavior solely from a floor/roof/plate role.
- When reporting reactions or member forces, note whether rigid links or diaphragms influenced the load path.
- Treat diaphragm assumptions as lateral-system assumptions that affect collectors, bracing, frames, walls, supports, and foundations.
- If a model becomes unexpectedly stiff or stable after constraints are added, inspect whether the constraints are physically justified.

## Tradeoffs / cautions

- Rigid constraints simplify models and can represent real rigid behavior, but overuse can create unrealistic force transfer.
- Flexible diaphragm models can be more realistic for some systems, but require mesh/stiffness assumptions and careful load transfer.
- Semi-rigid diaphragm modeling can capture distribution better, but adds modeling complexity and source-specific criteria.
- Rigid links are useful for offsets and rigid zones, but can hide eccentricities if not preserved as explicit assumptions.
- Constraint conflicts can overconstrain the model, create singularities, or produce misleading reactions.

## Source-backed claims

- Single-point constraints affect one node's DOFs; multi-point constraints impose relationships between node DOFs. [S1]
- Rigid diaphragm constraints are represented as multi-point constraints tying constrained nodes to retained-node motion in a chosen plane. [S2]
- Diaphragms transfer lateral/inertial forces to vertical resisting elements and involve chords/collectors. [S3]
- Diaphragm behavior may be idealized as flexible, rigid, or semi-rigid depending on system stiffness and design context. [S3]
- Constraint assumptions affect model topology and analysis behavior. [S1][S2]

## Open questions / weak evidence

- Fraia still needs final schema for multi-point constraints, retained/constrained node naming, constraint provenance, and solver export semantics.
- Code-specific diaphragm rigid/flexible criteria are deferred to later jurisdiction-specific guidance.
- Plate/shell diaphragm finite-element modeling and collector/chord checks need separate pages.

## Related pages

- [Supports, restraints, and releases](supports-restraints-and-releases.md)
- [Member end releases](member-end-releases.md)
- [Finite-element idealisation](finite-element-idealisation.md)
- [Local and global coordinate systems](local-and-global-coordinate-systems.md)
- [Load paths](../analysis/load-paths.md)
- [Second-order effects and stability](../analysis/second-order-effects-and-stability.md)

## Sources

- [S1] OpenSees Documentation, *Constraints Commands*. URL: https://opensees.github.io/OpenSeesDocumentation/user/manual/model/constraint.html. Source type: open-source solver documentation. Retrieved: 2026-05-07. Reliability/limits: useful for SP/MP constraint vocabulary and solver-topology distinction; software-specific syntax is not Fraia behavior.
- [S2] OpenSees Documentation, *rigidDiaphragm command*. URL: https://opensees.github.io/OpenSeesDocumentation/user/manual/model/mp_constraint/rigidDiaphragm.html. Source type: open-source solver documentation. Retrieved: 2026-05-07. Reliability/limits: useful source-scoped evidence for rigid diaphragm multi-point constraints; not a design guide.
- [S3] NEHRP / NIST GCR 11-917-10, *Seismic Design of Cast-in-Place Concrete Diaphragms, Chords, and Collectors: A Guide for Practicing Engineers*. URL: https://nehrp.gov/pdf/nistgcr11-917-10.pdf. Source type: public professional technical brief. Retrieved: 2026-05-07. Reliability/limits: strong diaphragm/chord/collector system guidance; concrete/seismic focus and not a generic Fraia schema source.
