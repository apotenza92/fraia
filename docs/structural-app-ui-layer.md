# Fraia Structural App UI Layer

_Status: draft v0.1_
_Date: 2026-04-13_

This document captures the current direction for how the Fraia structural authoring application should sit on top of the lower generic math/FEA layers.

---

## 1. Core idea

Fraia can have a deep, generic monorepo foundation:

- math primitives
- geometry/frame primitives
- generic simulation / FEA primitives

But the **structural engineering application UI** should not expose those deepest abstractions directly.

The structural app should speak in **structural terms**.

---

## 2. Recommended split

## Lower shared engine layers
These can remain generic and reusable:

- tensors, vectors, frames, transforms
- point / curve / surface / volume foundations
- generic field / boundary / constraint concepts
- solver and resolution kernels

## Structural application layer
This should present structure-specific authoring concepts such as:

- nodes
- members
- plates
- supports
- springs
- releases
- loads
- grids
- structural archetypes/builders

This is likely the right practical abstraction level for a structure-focused Fraia app.

---

## 3. What traditional structural software tends to do

A quick review of products like SPACE GASS and SkyCiv suggests a fairly consistent pattern:

### SPACE GASS
Models are described using:

- nodes
- members
- cables
- plates
- supports/restraints

It also provides higher-level helpers such as:

- structure wizards
- portal frame builder
- graphical draw tools
- datasheet/tabular editing

### SkyCiv Structural 3D
Also centers its modeling workflow around:

- nodes
- members
- supports
- plates/shells
- grids/snapping
- meshing tools for plates

So the structural UI abstraction in established tools is not the deepest FEA math layer.
It is usually the **structural-analysis modeling layer**.

---

## 4. Fraia implication

This suggests Fraia should probably do the same.

### Good lowest direct authoring primitives for the structural app
- Node
- Member (or Beam/Column/Brace as classified member)
- Plate
- maybe Solid later
- Support
- Release
- Load
- Grid / construction plane

These are understandable to structural engineers and still close enough to analysis reality.

---

## 5. Under the hood vs in the UI

The Fraia monorepo can still define deeper generic primitives like:

- point
- curve
- surface
- frame
- transform

But the structural app UI can map these into a structure-specific authoring layer.

For example:

- `Node` wraps a positioned point with structural semantics
- `Member` wraps a structural curve element with section/material/role metadata
- `Plate` wraps a structural surface region with thickness/material/mesh intent

This gives Fraia both:

- deep engine generality
- practical structural authoring UX

---

## 6. Recommended visual authoring levels

The structural app should likely support multiple authoring levels.

## Level A — Builders / archetypes / wizards
Examples:

- portal frame builder
- truss builder
- shed/warehouse concept builder
- floor framing generator
- simply supported beam builder

Current MVP note:
- the Rust desktop app already exposes builder editing for portal frames and simply supported beams
- the first beam workflow can now be seeded from project requirements and sized from the current catalog

## Level B — Direct structural modeling
Examples:

- place/edit nodes
- draw members
- draw plates
- assign supports/releases/loads

## Level C — Deep inspection
Examples:

- local axes
- connectivity
- meshing
- solver realization preview

Most users should spend most of their time in Levels A and B.

### 6.1 The user and the agent should share the same model-editing surface

The structural app should not have one geometry world for the GUI and a different hidden geometry world for the LLM.

Instead:
- user actions in the GUI should edit Fraia-native authored structures
- agent actions should edit those same Fraia-native authored structures through deterministic Fraia operations
- the viewport should show the resulting authored/builder state back to the user

This makes agent-created geometry inspectable and editable instead of mysterious.

### 6.2 Multiple input modes should coexist

A good Fraia workbench should allow at least three practical ways to begin:

1. brief/planning entry
   - the user describes what they want
   - the agent asks clarifying questions
   - Fraia proposes an initial builder/system path

2. builder/system entry
   - the user or agent chooses a beam, frame, truss, or similar system
   - Fraia exposes the relevant parameters

3. direct primitive entry
   - the user draws or edits nodes, members, plates, supports, and loads directly

A later sketch/diagram layer can sit above these, but it should still converge back to the same authored Fraia model.

### 6.3 Mixed manual + generated geometry needs explicit reconciliation tools

A realistic Fraia workbench should assume that many projects will contain a mixture of:
- builder-generated geometry
- agent-added primitives
- user-added primitives
- user edits to previously generated objects

The UI should therefore support explicit actions such as:
- apply edit back to builder
- parameterize selection
- promote selection to project-local builder
- re-parameterize subsystem
- re-parameterize current model

These actions should be explicit review/approval operations, not silent automatic rewriting of builder logic.

### 6.4 Manual additions should stay manual by default

If the user or agent adds new objects directly, Fraia should normally treat them as authored/manual objects first.
They should not be silently merged into an existing archetype/template interpretation unless the user chooses a promotion or re-parameterization action.

---

## 7. Fraia should not start from raw tensors in the UI

Even if the engine is mathematically deep, asking structural users to author directly in terms of generic low-level abstractions would be the wrong UX.

The structural app should feel like structural engineering software, not a math engine frontend.

---

## 8. Geometry and graphics attached to structural primitives

Yes: the structural authoring primitives should carry render/interaction data.

Examples:

### Node
- point position
- snap point
- support/load attachment anchor

### Member
- centerline geometry
- role classification
- local-axis display
- section display style later

### Plate
- boundary polygon or region
- plate symbol/mesh preview
- thickness/material display metadata

### Support / Load
- glyphs/arrows/symbols
- local/global direction cues

---

## 9. Practical Fraia recommendation

For the structural-specific application, the lowest direct drawing primitives should probably be:

- nodes
- members
- plates
- supports
- loads

with archetype/builders above them.

That seems much closer to the expectations set by tools like SPACE GASS and SkyCiv.

## 9.1 Plate-to-member interaction should be a first-class concern

Fraia should architect for member/plate interaction from day 1.

Examples:

- slab supported by beams
- wall panel connected to frame members
- plate edge attached to line member
- member framing into plate-supported regions

This implies the structural app and lower connectivity layers should support:

- plate edge to member connectivity
- explicit attachment semantics
- compatible DOF transfer assumptions
- automatic or assisted meshing at interfaces

### Important scope distinction

It is reasonable to require the **data model and connectivity model** to support this from day 1.

It may still be pragmatic for the **first implemented solver path** to begin narrower and grow into richer member/plate interaction handling.

---

## 10. Design choices currently favored

- Fraia monorepo can contain deeper generic math/FEA layers.
- The structural app GUI should be structure-specific, not math-specific.
- Node/member/plate/support/load is a strong candidate for the lowest direct authoring abstraction.
- Builders/wizards/archetypes should sit above direct structural primitives.
- Plate/member interaction should be part of the architecture from day 1, even if implementation maturity grows in phases.
- Deep solver/math abstractions should remain mostly hidden from normal structural authoring workflows.

---

## 11. Open questions

- Whether “member” should be the canonical structural UI primitive or whether beam/column/brace should be first-class visible labels immediately
- When plates/shells should enter the first production UI
- How much direct node editing should be exposed versus generated from builders
- How to reconcile structural app authoring primitives with future non-structural Fraia domains

---

_End of draft._
