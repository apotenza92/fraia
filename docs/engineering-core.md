# Fraia Engineering Core

This document defines the durable architecture of Fraia's Rust-first engineering core.

**Canonical downstream docs:** use `builder-graph-architecture.md` for builder graph specifics, `resolution-and-runs.md` for pipeline/run separation, and `engineering-output-pipeline.md` for checks/export/output architecture.

---

## 1. Product direction

Fraia is engineering workbench software. Its core provides a:

- versioned engineering core
- open project/package format
- modular library system
- deterministic validation + analysis pipeline
- agent-friendly abstraction system

The long-term product may become a modern structural engineering application, but the initial core should be usable through:

- CLI
- local services / APIs
- agent tools
- later desktop/web UI

The GUI is a client of the core, not the core itself.

---

## 2. Core design principles

### 2.1 Highest-useful-abstraction first
Agents and users should work at the **highest valid abstraction layer** and descend only when necessary.

Examples:

- prefer `portal_frame.single_bay` over manually creating nodes and members
- prefer `beam.simply_supported` over manually assigning supports/releases
- only descend to low-level primitives when customization, diagnostics, or solver realization requires it

### 2.2 Deterministic underneath, semantic above
Human/agent-facing concepts should be semantic and compact.

Underneath, those concepts must map to deterministic low-level primitives and eventually to solver-realizable data.

### 2.3 Modular and Unix-like
The core should be:

- file-based
- inspectable
- scriptable
- composable
- versioned
- migratable

### 2.4 Projects are not blobs
Projects should mostly contain project-specific instances and references.

Reusable definitions should live in external libraries/packages.

### 2.5 Engineering truth is not in the agent session
The source of truth must remain:

- project data
- libraries/catalogs
- resolved analysis snapshots
- results

Agent sessions are orchestration/context, not engineering state.

---

## 3. Scope framing

### 3.1 Near-term product focus
Architect for a broad simulation future, but ship **structural mechanics first**.

Initial domain focus:

- steel structures first
- line-member structural systems first
- 3D canonical geometry from day 1
- preliminary physics-based rules before full code modules
- one solver adapter first

### 3.2 Long-term vision
The deepest core may evolve into a more general agent-friendly simulation platform, not only structural engineering.

However, structural engineering remains the first domain package and first real product.

---

## 4. Abstraction ladder

The system should be built as a layered concept stack.

## Layer 0 — Math / geometry substrate
Foundational concepts only.

Examples:

- Tensor
- Scalar
- Vector
- Matrix
- Point
- Frame
- Transform

Notes:

- `Tensor` is the deep algebraic substrate.
- `Scalar`, `Vector`, and `Matrix` should still exist as explicit typed wrappers.
- `Point`, `Frame`, and `Transform` should remain first-class geometric types, not reduced to raw tensors in the public model.

## Layer 1 — Physical quantities
Domain-neutral physical meanings built on Layer 0.

Examples:

- displacement vector
- force vector
- moment vector
- stress tensor
- strain tensor
- temperature scalar
- heat flux vector

Important:

- `stress tensor` belongs here, not in the pure math layer.
- This layer should remain broader than structural engineering.

## Layer 2 — Generic simulation / analysis primitives
Generic concepts used by analysis domains.

Examples:

- node / point entity
- curve region
- surface region
- volume region
- material law
- property assignment
- boundary condition
- source/load
- constraint
- analysis step
- result field

## Layer 3 — Structural analysis primitives
Structural-specific analytical objects.

Examples:

- curve member
- surface member
- volume member
- support primitive
- release primitive
- joint primitive
- rigid link / MPC primitive
- structural load case
- structural combination
- structural check profile bindings

## Layer 4 — Structural semantics / archetypes
Human-facing engineering concepts.

Examples:

- beam
- column
- brace
- slab
- wall
- simply supported beam
- cantilever beam
- portal frame
- Pratt truss
- braced bay

These are generally **composite** or **semantic** concepts, not deep atoms.

## Layer 5 — Design / optimization / code layer
Where autonomous design loops, checks, and rules live.

Examples:

- preliminary rule profiles
- official code packs
- optimization variables
- objectives and constraints
- constructability heuristics
- detailing logic
- connection design logic
- report/drawing logic later

## Layer 6 — User intent / project brief / building semantics
A higher layer above structural semantics where the user describes what they are trying to build.

Examples:

- house
- shed
- warehouse
- portal-frame workshop
- mezzanine fitout
- bridge concept
- tower concept

This layer captures things like:

- building/use type
- scale and layout intent
- occupancy/use assumptions
- architectural constraints
- site/environment context
- preferred materials
- cost/carbon priorities
- stage of design maturity
- unknowns still to be clarified

This layer is essential for agent-led requirement discovery before structural modeling begins.

---

## 5. Primitive philosophy

### 5.1 Deep atoms are not “beam” and “column”
At the structural primitive level, a beam and a column may both just be a more generic `curve_member`.

Their distinction is often semantic and rule-oriented, not primitive.

### 5.2 Beam and column are often semantic roles
Examples of semantic/classification data:

- semantic roles: beam, column, brace, rafter, purlin
- system roles: gravity, lateral, primary, secondary
- context tags: roof, floor, perimeter, level_2

### 5.3 Composite library elements are essential
We should absolutely define higher-level reusable elements like:

- `beam.simply_supported`
- `beam.cantilever`
- `column.fixed_base`
- `brace.pin_ended`
- `frame.portal_single_bay`
- `truss.pratt`

These should be:

- parameterized
- reusable
- expandable into lower-level primitives
- summarized for agent use

---

## 6. 3D canonical world

We should define the canonical authored model in **3D from day 1**.

Implications:

- all geometry is embedded in 3D space
- a “2D” model is just a planar/reduced realization within 3D
- no separate 2D project universe

However, **analysis realizations still differ**.

We should separate:

- 3D canonical geometry/topology
- chosen analysis form / realization

Examples of realization forms:

- frame_1d_in_3d
- truss_1d_in_3d
- shell_2d_in_3d
- solid_3d

---

## 6.1 Canonical unit basis

Fraia's backend data basis should be SI units.

Current direction:

- geometry and displacements are canonicalized to metres in backend model data, with displacement display commonly shown in millimetres
- forces are canonicalized to newtons
- line loads are canonicalized to newtons per metre
- moments are canonicalized to newton-metres
- stresses are canonicalized to pascals
- `kN`, `kN/m`, `kN·m`, `MPa`, and similar engineering units are display/input units, not a separate backend truth

Legacy fields with unit suffixes such as `gravity_load_kn_per_m`, `lateral_load_kn`, and `distributed_load_kn_per_m` may remain at authored/input boundaries until schema migration, but they should be converted to canonical SI before structural primitives, frame models, solver inputs, and run artifacts rely on them.

---

## 7. Frames, placements, orientation, and connectivity

This is foundational and must not be hardcoded around a single global XYZ.

### 7.1 Global coordinates are not enough
The schema must support:

- world/global frame
- assembly/local frame
- primitive frame
- port/interface frame
- load/reference frame

### 7.2 Directions must be frame-qualified
A direction should never exist as raw components without a reference frame.

Instead of:

```json
{ "direction": [0, -1, 0] }
```

prefer:

```json
{
  "direction": {
    "components": [0, -1, 0],
    "frameRef": "global"
  }
}
```

### 7.3 Local axes must be explicit or derivable
For example, a curve member should have a local-axis derivation rule, such as:

- local x from curve tangent
- local z from a reference/up vector
- local y by cross product

### 7.4 Ports/interfaces are first-class
Primitives and archetypes should expose connectable ports with:

- location
- local frame
- accepted connection types
- DOF interface
- compatibility rules

### 7.5 Connectivity is more than coincidence
A connection primitive should encode:

- placement/alignment behavior
- transferred DOFs
- released DOFs
- offset/eccentricity rules
- local axis expectations

---

## 8. Semantic-to-deterministic mapping

This is one of the central ideas of the system.

Semantic concepts should be defined in data and mapped to deterministic low-level primitives.

Examples:

- `support.fixed`
- `support.pinned`
- `support.roller`
- `release.major_axis_free`
- `connection.rigid`
- `connection.pin`

Then composite concepts can be built from those deterministic pieces.

Example:

- `beam.simply_supported` should be a semantic/composite library element
- it resolves into a curve member + boundary/support/release primitives + default classification/check behavior

The exact deterministic realization may vary by analysis form.

---

## 9. Libraries, packages, and projects

### 9.1 Libraries should live outside projects
Projects should reference external libraries/packages rather than embedding all reusable definitions.

Candidate external package families:

- core primitive packages
- structural primitive packages
- archetype libraries
- material catalogs
- section/profile catalogs
- preliminary rules packages
- official code packages later
- validation/diagnostic knowledge packages
- solver adapter packages

### 9.2 Projects can be very small
A project should be allowed to be:

- a partial structure
- an archetype instantiation
- a topology skeleton
- a concept with no final sizing yet
- a structure with no loads assigned yet

This is useful for agent workflows and template composition.

### 9.3 Lockfiles / version pinning
If projects depend on external packages, they will need pinned versions or equivalent lock metadata for reproducibility.

---

## 10. Authoring vs resolved vs run snapshots

We should separate three important representations.

### 10.1 Authoring representation
What users and agents work on.

Properties:

- modular
- human-readable
- references libraries
- compact

### 10.2 Resolved runtime representation
Expanded and normalized in memory before analysis.

Properties:

- references resolved
- defaults applied
- units normalized
- frames resolved
- rule inheritance resolved

### 10.3 Frozen run snapshot
Saved for each analysis/optimization run.

Properties:

- exact resolved model used
- exact rule values used
- exact package versions used
- exact solver adapter/version used
- canonical SI numeric values, with display units applied only by renderers/reports
- results and logs

This split is essential for reproducibility and future migrations.

---

## 11. Migration and versioning

Migration support should exist from day 1.

### 11.1 Version everything explicitly
Examples:

- schema version
- package version
- ruleset version
- solver adapter version
- run snapshot version

### 11.2 Migrations should be explicit and chainable
We should prefer deterministic migrations like:

- 0.1.0 -> 0.2.0
- 0.2.0 -> 0.3.0

rather than permanently supporting many legacy shapes in every reader.

### 11.3 Historical runs must stay immutable
Old analysis snapshots should not be mutated in place just because the authoring schema evolves.

---

## 12. Rules strategy

### 12.1 Early-stage preliminary rules
Before official code modules, use a preliminary physics-based ruleset.

These should focus on things like:

- equilibrium
- stability
- stress/utilization bounds
- deflection
- drift
- buckling margin
- slenderness

### 12.2 Do not confuse preliminary rules with code compliance
We should distinguish:

- preliminary/global conservative rules
- firm/project custom rules
- official code modules

### 12.3 Rules should eventually be scoped
Different members/systems will need different requirements.

Examples:

- floor beam deflection limits
- roof beam limits
- brace slenderness rules
- global drift checks

But the first implementation can remain smaller while still allowing later growth.

---

## 13. Agent-facing principles

### 13.1 Abstraction guards
The system should protect agents from unnecessary low-level detail.

Agents should see only the layer needed for the task.

### 13.2 Progressive lowering
Default workflow:

1. start at user intent / project brief layer
2. use semantic/archetype layer
3. descend to structural primitives if needed
4. descend to simulation primitives if needed
5. descend to math/frame details only when necessary

### 13.3 Compact “cards” for libraries
Every primitive/archetype should eventually expose a concise agent-readable summary including:

- what it is
- what parameters it exposes
- what ports it has
- what it expands to
- what is required before analysis
- common failure modes

### 13.4 Low-level diagnostics should map back upward
If analysis fails at a low level, diagnostics should reference the highest meaningful object if possible.

Example:

- not only “node 18 mechanism”
- but also “left base support in portal frame archetype creates a mechanism”

---

## 14. Structural application abstraction

The Fraia monorepo may contain very deep generic layers, but the structural engineering application should expose a structure-specific modeling abstraction.

Current preferred direct authoring concepts for the structural app:

- nodes
- members
- plates
- supports
- releases
- loads
- structural builders/archetypes above those

This is closer to established structural software workflows and more practical than exposing the deepest math/FEA abstractions directly in the UI.

### 14.1 Shared authored-model contract for users and agents

The Fraia GUI and Fraia agents should operate on the same engineering truth.

That means the user-visible app and the agent backend should both read and write through Fraia-native structures such as:

- planning markdown
- builder graph
- authored structural model
- immutable run artifacts

The agent should not maintain a hidden parallel model of the structure in chat state.
If the agent creates, edits, or loops on a structure, those changes should be reflected in the same authored Fraia model that the GUI renders and edits.

### 14.2 Multiple entry modes are valid

A user should not be forced into only one way of creating a structure.

Fraia should support at least these entry paths:

1. planning/brief first
   - user describes a warehouse, beam, frame, mezzanine, or similar goal
   - agent asks clarifying questions
   - Fraia records assumptions and intent bounds

2. builder/system first
   - user or agent instantiates a structural system such as a simply supported beam or portal frame
   - exposed parameters drive deterministic materialization into authored primitives

3. direct structural primitive authoring
   - users can directly create/edit nodes, members, plates, supports, releases, and loads
   - this remains necessary for structures that do not fit a clean archetype path

4. diagram/sketch-assisted authoring later
   - a future geometry-intent layer can sit above structural primitives/builders
   - but it should still feed the same Fraia-native authored structures underneath

### 14.3 Agent loops should be incremental, not only one-shot

For many structures, the correct agent behavior is not:
- generate one whole structure once
- hope it is right

Instead Fraia should support incremental loops such as:
1. interpret the brief
2. instantiate an initial builder or primitive model
3. show the geometry and load/support assumptions to the user
4. validate and analyze deterministically
5. revise the model or sizing based on diagnostics/results
6. repeat until the structure is acceptable or a decision gate is reached

This is especially important for the earliest real Fraia engineering workflows, such as simply supported beam sizing and other very small structural problems.

---

## 16. Planning-first workflow

The agent should generally begin in a planning/discovery mode before making engineering changes.

### 16.1 The agent should ask questions early
For many projects, the user will initially describe something at a very high level, such as:

- a house
- a shed
- a warehouse
- a tower
- a bridge

That is not enough information to safely instantiate a structural model.

The agent should be expected to ask follow-up questions about:

- intended building/system type
- dimensions and spans
- number of levels
- material preferences
- site/wind/seismic assumptions
- support/foundation assumptions
- intended level of fidelity
- whether loads, sections, and code regime are already known

### 16.2 Fraia should persist planning conversations
A planning markdown document should be created early and updated through discovery.

This file should capture:

- user brief
- assumptions
- unknowns
- decisions
- open questions
- next modeling steps

This supports:

- agent continuity
- human review
- migration of context across sessions
- better traceability than keeping everything only in chat state

---

## 17. Open questions

These remain intentionally unresolved or only partially resolved:

- exact package manifest format
- exact lockfile/version-pinning strategy
- exact solver adapter contract
- exact port/interface schema
- exact local-axis derivation schema
- whether generic tensor type is public everywhere or mostly internal
- how much of the primitive/archetype expansion language should be data-driven vs code-driven
- when to introduce shell/surface/solid primitives relative to line-member support
- when to introduce official code modules vs preliminary rules only

---

## 18. Summary of key decisions

- The engineering core comes before the GUI.
- The system should be modular, versioned, migratable, and agent-friendly.
- We should use a layered abstraction ladder.
- The canonical authored world should be 3D from day 1.
- Global XYZ is insufficient; frames, transforms, and local orientation are fundamental.
- Beam/column are often semantic roles, not deep primitive types.
- Reusable semantic composites like simply supported beam should exist as library elements.
- Libraries/packages should generally live outside projects.
- Projects should be able to be partial, abstract, and template-like.
- Backend engineering values should use SI units; engineering display units are formatting/parsing concerns.
- The system should protect agents from deep abstraction unless needed.
- Agent runtimes are replaceable clients of Fraia operations; engineering truth remains in Fraia.

---

_End of draft._
