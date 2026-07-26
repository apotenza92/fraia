# Fraia Frames and Connectivity

_Status: draft v0.1_
_Date: 2026-04-13_

This document defines the current direction for Fraia placement, orientation, ports, and connection semantics.

---

## 1. Why this matters

A serious FEA/structural platform cannot rely on a single global XYZ interpretation.

We need first-class support for:

- local coordinate systems
- relative placements
- member/local axes
- connection alignment
- frame-aware loads and restraints
- port-based composition

This is essential for:

- deterministic solver realization
- reusable primitives/archetypes
- agent understanding
- migration-safe modeling

---

## 2. Core principle

### Coordinates are always frame-relative
Any position, direction, offset, orientation, or load direction must either:

- explicitly reference a frame, or
- inherit a clearly defined frame context

Never assume raw XYZ values are globally meaningful without context.

---

## 3. Frame hierarchy

Fraia should support multiple frame levels.

## 3.1 Global/world frame
Project-wide reference frame.

Typical uses:

- site positioning
- view/navigation
- external export references

## 3.2 Assembly frame
A frame for an assembly or archetype instance.

Typical uses:

- placing a portal frame instance
- placing a truss module
- placing a connection assembly

## 3.3 Primitive frame
A frame local to a primitive instance.

Typical uses:

- local orientation rules
- primitive-local parameter interpretation
- child placement

## 3.4 Port frame
A frame attached to a specific connectable interface.

Typical uses:

- beam end alignment
- support connection faces
- joint attachment rules
- local boundary condition direction meaning

## 3.5 Load/reference frame
A frame in which a load or directional quantity is expressed.

Typical uses:

- gravity in global frame
- pressure normal to a local surface frame
- load along member local axis

---

## 4. Placement model

A primitive or assembly should be placeable relative to another frame.

### Candidate placement object
A placement should define:

- parent frame reference
- origin position
- orientation relative to parent
- optional transform representation

### Example

```json
{
  "type": "placement3",
  "parentFrameRef": "global",
  "origin": {
    "type": "point3",
    "coordinates": [0, 0, 0],
    "frameRef": "global"
  },
  "orientation": {
    "type": "transform3",
    "fromFrameRef": "self.local",
    "toFrameRef": "global"
  }
}
```

---

## 5. Orientation model

### 5.1 Orientation must be explicit or derivable
Every primitive that depends on local axes must define how those axes are obtained.

### 5.2 Example: curve member axis derivation
For a curve member:

- local x: tangent from start to end
- local z: derived from explicit reference/up vector projected perpendicular to x
- local y: cross product of z and x, or x and z depending on chosen convention

### 5.3 Orientation rule object
We will likely need something like an orientation rule definition:

- method name
- required inputs
- fallback behavior
- failure conditions

Possible methods:

- explicit frame
- tangent_plus_reference_vector
- tangent_plus_global_up
- normal_based
- inherited_from_parent

---

## 6. Ports as first-class interfaces

Ports are how instances connect in a reusable and agent-friendly way.

A port should describe:

- where connection happens
- what frame applies there
- what kinds of things can connect
- what DOFs or field variables exist at that interface
- what compatibility/constraint behavior is expected

### Port candidate fields

- `id`
- `kind`
- `location`
- `frameRef`
- `accepts`
- `providesDofs`
- `metadata/tags`

### Example

```json
{
  "id": "end_a",
  "type": "port",
  "kind": "structural-node-port",
  "location": {
    "type": "point3",
    "coordinates": [0, 0, 0],
    "frameRef": "self.local"
  },
  "frameRef": "self.port.end_a",
  "accepts": ["support", "joint", "member_end"],
  "providesDofs": ["UX", "UY", "UZ", "RX", "RY", "RZ"]
}
```

---

## 7. Connectivity model

Connectivity is not just geometric coincidence.

A connection should potentially define:

- which ports are linked
- coincidence/alignment behavior
- relative orientation rules
- translational compatibility
- rotational compatibility
- released DOFs
- offsets/eccentricities
- optional stiffness or partial restraint later

### Candidate connection primitive families

- rigid connection
- pin connection
- roller/support connection
- member-end release
- rigid link / MPC connection
- tied interface
- member-to-plate interface connection
- plate-edge support/attachment connection

## 7.1 Member/plate interaction is important early

Fraia should plan for member/plate interaction from day 1.

Typical examples:

- beam supporting slab edge or slab region
- wall plate attached to frame member line
- stiffened or framed panel regions

This likely requires:

- ports on member ends and along member spans where relevant
- ports on plate edges and regions
- explicit connectivity semantics for line-to-surface interaction
- auto-meshing or assisted meshing rules at interfaces
- clear DOF transfer assumptions between connected entities

## 7.2 Load transfer across dimensions should be explicit

The same interaction layer should eventually support load transfer semantics, not only geometric attachment.

Important examples:

- area load on a plate transferred to a plate/shell realization directly
- area load on a plate distributed to supporting members in a reduced line-member realization
- elevation wind pressure turned into equivalent member line loads using tributary width/area assumptions
- slab/deck load sent into beams/girders using deterministic tributary rules when the current analysis form is 1D-focused

This implies Fraia will likely need explicit concepts for:

- authored load target
- supporting-object relationships
- tributary/distribution policy
- realization-time equivalent load generation

The key rule should be:
- the authored load remains attached to the structural target that the user/agent intended
- any equivalent member/nodal loads created for a chosen solver path are downstream realization artifacts

This is important for both correctness and explainability when an agent says things like:
- "apply wind loads to every elevation"
- "send roof area load into the supporting frame"

---

## 8. Supports, releases, and connections

These should be separated conceptually.

## 8.1 Support primitives
Boundary conditions applied at a location/port/node relative to a frame.

Examples:

- fixed support
- pinned support
- roller support

These generally describe restrained and free DOFs.

## 8.2 Release primitives
Behavior at a member end or interface.

Examples:

- release major-axis bending
- release torsion
- release all moments

These are not identical to supports.

## 8.3 Connection primitives
Compatibility relationships between two or more ports/interfaces.

Examples:

- rigid joint
- pin joint
- semi-rigid connection later
- equal-DOF constraint

---

## 9. Semantic concepts vs deterministic primitives

A semantic concept such as `beam.simply_supported` should not directly be a low-level support object.

Instead:

- `beam.simply_supported` is a semantic composite/archetype
- it resolves into lower-level structural primitives like supports/releases/connections
- those deterministic structural primitives then resolve into solver-ready constraints

This layering is essential.

---

## 10. Agent-friendly connectivity

Agents should not need to reason from raw DOF lists unless necessary.

Instead, each port/connection/support primitive should eventually expose a compact semantic card:

- what it means
- where it attaches
- what it restrains or releases
- common compatibility constraints
- common failure conditions

### Example failure explanations

- incompatible port types
- undefined local axis at connection
- support attached to primitive without required DOFs
- release pattern creates mechanism
- reference vector parallel to member tangent

---

## 11. Local axis ambiguity handling

One of the most common failure modes in structural/FEM systems is ambiguous local orientation.

The schema and validator should catch cases like:

- reference/up vector parallel to member axis
- invalid handedness
- missing shell normal/orientation
- inconsistent child/parent frame alignment

The validator should report these at the highest meaningful layer possible.

---

## 12. Direction values

Directions should use vector objects, not anonymous arrays.

### Good

```json
{
  "type": "vector3",
  "components": [0, 0, -1],
  "frameRef": "global"
}
```

### Bad

```json
[0, 0, -1]
```

without context.

---

## 13. Suggested initial schema concepts

Recommended early concepts:

- `frame3`
- `placement3`
- `transform3`
- `orientationRule`
- `port`
- `connection`
- `supportPrimitive`
- `releasePrimitive`

These should be enough to begin modeling reusable structural primitives cleanly.

---

## 14. Example conceptual stack

### A curve member instance
Could contain:

- geometry curve
- local orientation rule
- start port
- end port
- primitive frame

### A support primitive
Could contain:

- attachable port type requirements
- restrained DOFs relative to a local frame
- visualization/semantic metadata

### A simply supported beam archetype
Could contain:

- one curve member primitive
- two ports at member ends
- support scheme definition using lower-level support primitives
- semantic role metadata
- validation rule: must be stable under selected realization assumptions

---

## 15. Design choices currently favored

- Authored projects should always live in a 3D canonical world.
- Directions and offsets should be frame-qualified.
- Local axes should be defined by explicit orientation rules.
- Connectivity should be port-based.
- Semantic archetypes should resolve into deterministic support/release/connection primitives.
- Global coordinates alone are not sufficient.

---

## 16. Open questions

- Exact representation of frame inheritance/defaulting
- Whether transforms should be stored explicitly or generated from placement/orientation fields
- How to represent partial restraint / semi-rigid behavior later
- How generic port definitions should be across non-structural physics domains
- How solver adapters should declare what port/connection semantics they support

---

_End of draft._
