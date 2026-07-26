# Fraia Diagramming and Geometry Layer

_Status: draft v0.1_
_Date: 2026-04-13_

This document captures the current direction for Fraia's diagramming, visual geometry, and user-facing model editing layer.

---

## 1. Why this exists

The current MVP still asks for engineering parameters like span and height too early.

That is acceptable for a narrow prototype, but not for the long-term Fraia workflow.

A real Fraia system should be able to move from:

- vague user intent
- to sketched/diagrammed structure
- to structural/archetype model
- to resolved analytical model

This means Fraia needs a diagramming/geometry layer between pure planning and full analysis realization.

---

## 2. Core principle

### Users should not need to think like the solver first
A user often thinks in terms of:

- rooms
- bays
- clear spaces
- roof shape
- walls/columns/frames
- support locations
- openings
- approximate layout

not in terms of:

- exact analysis nodes
- element end releases
- DOF patterns

Therefore Fraia needs an intermediate visual/diagrammatic model.

---

## 3. Proposed layer placement

A useful conceptual stack is:

1. user brief / planning layer
2. diagram / sketch / geometric intent layer
3. structural semantic/archetype layer
4. structural primitive layer
5. simulation/solver realization layer

The diagram layer helps bridge what users mean and what Fraia can analyze.

---

## 4. What diagram information might contain

Diagram information may include:

- points
- curves
- regions
- labels/annotations
- approximate dimensions
- alignment/grid information
- visual grouping
- semantic tags such as wall, frame line, support line, roof edge

This is not yet the full analytical model.

It is a visual/geometric intent representation.

---

## 5. Geometry should attach to primitives and archetypes

Yes: Fraia primitives and archetypes should eventually carry associated geometric/diagram information.

Examples:

- a point primitive can have a visible marker/handle
- a curve member can have a centerline geometry for display
- a support primitive can have a symbol/anchor glyph
- a portal frame archetype can carry a default diagram shape
- a truss archetype can carry a node/curve diagram representation

This is how Fraia can both:

- reason structurally
- and draw/edit visually

---

## 6. Separate visual geometry from analytical realization

This is very important.

A primitive or archetype may need multiple geometry views:

## 6.1 Diagram geometry
Used for:

- sketching
- user interaction
- layout editing
- simple previews

## 6.2 Physical geometry
Used for:

- richer display
- section/profile visualization
- later detailing and documentation

## 6.3 Analytical geometry
Used for:

- centerlines
- surface midsurfaces
- idealized structural regions
- solver realization

These should not be conflated.

---

## 7. Primitive definitions should carry visual metadata

A future primitive/archetype definition may need fields like:

- diagram primitives
- default symbols/glyphs
- handles/control points
- editable dimensions/parameters
- snap/connection anchors

This would make primitives usable in both:

- agent workflows
- graphical editing workflows

---

## 8. User input should often be graphical or semi-graphical

Instead of asking immediately for only spans and heights, Fraia should eventually support workflows like:

- draw a line for a frame
- place supports at ends
- indicate whether internal supports are acceptable
- sketch bays or roof outline
- choose from generated diagram options

This would let users communicate intent more naturally.

---

## 9. Diagram objects as editable semantic handles

A useful Fraia concept may be a diagram object that carries:

- geometry
- semantic meaning
- editable parameters
- links to underlying primitives/archetypes

Examples:

- a frame line diagram object
- a support marker
- a bay dimension handle
- a roof pitch handle

This could become a very powerful middle layer.

---

## 10. Relationship to ports/connectivity

Ports are not only useful for analysis composition.

They can also support diagrammatic editing by providing:

- snap points
- connection handles
- valid attachment interfaces
- local orientation cues

This means the same connectivity ideas can serve both:

- engineering correctness
- interactive geometry editing

---

## 11. Recommendation for Fraia evolution

The current demo's direct span/height questions should be treated as temporary MVP behavior.

A better future Fraia workflow would be:

1. clarify building/system intent
2. choose or sketch a diagrammatic system
3. expose only relevant parameters from that system
4. generate structural options from the chosen diagram/semantic structure

This would make planning feel much more natural.

---

## 12. Design choices currently favored

- Fraia likely needs a diagram/sketch/geometry-intent layer above the analytical model.
- Primitives and archetypes should eventually carry associated visual geometry metadata.
- Diagram geometry, physical geometry, and analytical geometry should be separated.
- The current demo's direct parameter questioning is a temporary simplification, not the ideal long-term UX.

---

## 13. Open questions

- Exact schema for diagram geometry
- How editable geometry should map to archetype parameters
- Whether Fraia should store sketch objects separately from analytical objects or as linked views
- How much of the first GUI should be form-based versus diagram-based

---

_End of draft._
