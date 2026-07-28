# Fraia Primitives and Archetypes

_Status: draft v0.1_
_Date: 2026-04-14_

_Canonical focus:_ primitive vs archetype vocabulary, parameter exposure, ports/interfaces, and expansion semantics.
_See also:_ `builder-graph-architecture.md`, `package-system.md`, `structural-app-object-model.md`, `documentation-map.md`.

This document captures the current direction for Fraia primitive definitions, semantic composites, archetypes, and expansion into lower-level realizations.

---

## 1. Purpose

Fraia needs a reusable modeling system that works for:

- humans
- agents
- open-source contributors
- deterministic engineering pipelines

The core idea is:

- small low-level primitives
- reusable composite structural primitives
- higher-level semantic archetypes
- explicit expansion into lower-level forms

---

## 2. Key principle

### Start high, lower only when necessary
Agents and users should prefer higher-level reusable concepts where available.

Examples:

- use `frame.portal_single_bay` before hand-building members and supports
- use `beam.simply_supported` before manually assigning support/release patterns
- use low-level primitives only when customization or diagnostics require them

---

## 3. Primitive hierarchy

## 3.1 Layer A — generic analysis primitives
Deep analytical building blocks.

Examples:

- point/node entity
- curve region
- surface region
- volume region
- material assignment
- property assignment
- boundary condition
- constraint
- source/load

## 3.2 Layer B — structural primitives
Deterministic structural concepts built on Layer A.

Examples:

- curve member
- surface member
- support primitive
- release primitive
- connection primitive
- joint primitive
- rigid link primitive

## 3.3 Layer C — semantic composites / archetypes
Human-facing reusable patterns.

Examples:

- simply supported beam
- cantilever beam
- fixed-base column
- pin-ended brace
- portal frame
- Pratt truss
- braced bay

---

## 4. Important distinction

A beam or column is often not a deep primitive type.

At the structural primitive level, both may just be a more generic `curve_member`.

The distinction often appears in:

- semantic role
- expected checks
- default usage conventions
- agent interpretation

This means Fraia should avoid hardcoding “beam” and “column” as the deepest object categories.

---

## 5. Semantic roles and classification

Primitive instances should be able to carry semantic classifications such as:

- beam
- column
- brace
- rafter
- purlin
- slab
- wall

and also system/context metadata such as:

- gravity
- lateral
- primary
- secondary
- roof
- floor
- level_2

This gives agents and rule systems a useful bridge between low-level analysis objects and human engineering meaning.

---

## 6. What a primitive definition should contain

A reusable Fraia primitive/archetype definition will likely need:

- id/name
- version
- description
- category/type
- parameters
- local frame assumptions
- ports/interfaces
- composition/expansion rules
- validation requirements
- agent-readable summary/card

---

## 7. Parameters

Primitives and archetypes should expose tweakable parameters from day 1.

Examples:

- span
- height
- pitch
- section family
- material family
- support type
- brace pattern
- bay count
- bay spacing

Each parameter may eventually need:

- type
- units
- default
- range/domain
- optional/required status
- whether agents are allowed to modify it automatically

This is central to agent-driven optimization and reuse.

---

## 8. Ports and interfaces

A primitive should expose named ports for composition.

Examples:

- beam: `start`, `end`
- column: `base`, `top`
- portal frame: `left_base`, `right_base`
- support: `attach`

Ports are essential for:

- deterministic composition
- connection validation
- local frame handling
- agent understanding of how parts connect

---

## 9. Expansion / realization

Every higher-level primitive or archetype should be expandable into lower-level forms.

### Example progression

`frame.portal_single_bay`
-> expands to structural primitives such as:
- two curve members behaving as columns
- one curve member behaving as a rafter
- support primitives at bases
- joint/connection behavior at knees/ridge as appropriate

Then structural primitives resolve into lower-level analysis objects and eventually solver-ready data.

---

## 10. Example: simply supported beam

A simply supported beam should exist as a library element.

It is not a deep atom.

It is a semantic/composite concept built from lower-level pieces.

### It likely contains
- one curve member primitive
- two end ports
- a support scheme defined using lower-level support/release primitives
- semantic role metadata such as `beam`
- default validation expectations

### It should expose parameters like
- span
- section/material references
- support configuration choices
- local orientation info if needed

---

## 11. Example: support and release library elements

Lower-level deterministic structural primitives should also exist as reusable library resources.

Examples:

- `support.fixed`
- `support.pinned`
- `support.roller`
- `release.major_axis_free`
- `release.all_moments_free`
- `connection.rigid`
- `connection.pin`

These are very useful intermediate building blocks.

---

## 12. Archetypes vs templates vs primitives

For now, the distinctions can remain practical rather than over-formal.

### Primitive
A reusable low-level or mid-level building block.

### Archetype
A higher-level reusable structural pattern with exposed parameters.

### Instance
A project-specific use of a primitive/archetype.

### Builder node
A typed project-specific archetype instance stored in the Fraia builder graph.

### Builder graph
A project-specific graph of builder nodes representing compact concept-level structure before full primitive materialization.

### Resolved realization
The expanded lower-level model used for validation/analysis.

---

## 13. Agent-readable summaries

Every primitive/archetype should eventually expose a compact card for agents.

Candidate fields:

- what it is
- what it is for
- what parameters matter
- what ports it has
- what lower-level things it expands to
- what must be true before analysis
- common failure modes

This allows agents to reason at the right abstraction layer without loading excessive internals.

---

## 14. Primitive composition rules

Compositions should be explicit.

A composite definition may need to specify:

- child instances
- parameter mappings to children
- child placements
- port-to-port connections
- exposed parameters propagated upward
- derived parameters

This is likely where Fraia becomes very powerful for modular authoring and agent workflows.

---

## 15A. Builder graph relationship

Archetypes should be treated as catalog definitions, while project-specific usage should be stored as builder nodes inside a builder graph.

This document does not define the full builder-graph architecture.
For that, see:
- `builder-graph-architecture.md`
- `package-system.md`

In this document, the important point is simply:
- primitives are the substrate
- archetypes are reusable parameterized templates above them
- builder nodes are project-specific instances of those templates

---

## 15. Parameter exposure discipline

Not every child parameter needs to be exposed upward.

A composite archetype should expose only the controls that matter at its own abstraction level.

Example:

A portal frame archetype may expose:

- span
- height
- roof pitch
- support type
- section family

without exposing every low-level child placement field directly.

This helps protect agents from unnecessary complexity.

---

## 16. Validation responsibilities

Primitive/archetype definitions should be able to declare at least lightweight validation expectations.

Examples:

- span must be positive
- required ports must be connected
- support pattern must be valid for intended analysis form
- section/material must be assigned before analysis
- reference vector must not be parallel to member axis

Later, richer diagnostic packages can build on this.

---

## 17. Suggested early Fraia primitive families

Good initial families:

### Generic / structural foundations
- `curve_member`
- `support.*`
- `release.*`
- `connection.*`

### Early semantic composites
- `beam.simply_supported`
- `beam.cantilever`
- `column.fixed_base`
- `brace.pin_ended`
- `frame.portal_single_bay`
- `truss.pratt`

This would give Fraia a strong starting standard library.

---

## 18. Design choices currently favored

- Deep atoms should remain relatively generic.
- Beam/column distinctions should often live in semantics/classification, not only deep primitive type.
- Useful engineering patterns like simply supported beam should absolutely exist as reusable library elements.
- Composite primitives/archetypes should be parameterized and expandable.
- Ports/interfaces are central to deterministic composition.
- Agent-facing summaries are a first-class concern, not an afterthought.

---

## 19. Open questions

- Exact schema for primitive/archetype definitions
- How much expansion logic should be declarative vs coded
- Exact parameter inheritance/exposure rules
- How archetype definitions should be versioned and migrated
- How rich primitive cards need to be for agent use in early versions
- How quickly Fraia should move from hardcoded parameter structs toward schema-driven parameter boundaries

---

_End of draft._
