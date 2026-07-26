# Fraia Structural App Object Model

_Status: draft v0.1_
_Date: 2026-04-13_

This document defines the current preferred object model for the structure-specific Fraia application layer.

---

## 1. Purpose

The Fraia monorepo may contain deep generic math, geometry, and FEA layers.

However, the structural engineering application should expose a practical structure-specific model.

The preferred direct authoring layer is currently:

- Node
- Member
- Plate
- SupportAssignment
- LoadAssignment
- ReleaseAssignment
- connection/detail warnings and demands
- builders/archetypes above these

---

## 2. Core principle

### The structural app should speak structural language

The user-facing object model should not be:

- raw tensors
- raw curves/surfaces only
- raw solver entities only

Instead, it should use structural concepts that map cleanly to analysis while still matching engineer expectations.

## 2.1 Naming and display vocabulary

Fraia should keep authored structural objects, semantic roles, and finite-element discretisation distinct.

Canonical authored object types:
- `Member`: line-based structural object
- `Plate`: surface-based structural object
- `Node`: connectivity/DOF point
- `SupportAssignment`, `LoadAssignment`, `ReleaseAssignment`: explicit assignment objects

Semantic/user-facing terms come from `role`:
- member roles: `beam`, `column`, `rafter`, `brace`, `purlin`, `tie`
- plate roles: `slab`, `wall_panel`, `roof_panel`, `plate_region`

Default display labels should be derived from data:
- `role` + `id` is the default label, formatted in sentence case
- `Member { id: "B1", role: "beam" }` displays as `Beam B1`
- `Plate { id: "P2", role: "wall_panel" }` displays as `Wall panel P2`
- if a future `displayName` exists, it may be shown as the title while still preserving `object type`, `role`, and `id` in the inspector

Role formatting should be generic:
- replace `_` or `-` with spaces
- collapse repeated whitespace
- use sentence case, e.g. `primary_roof_beam` -> `Primary roof beam`
- avoid hardcoded role-label tables except for future acronym overrides if needed

Reserve `element` for finite-element/discretisation terminology:
- `AnalysisElement`, `beam_element`, `frame_element`, `plate_element`, `shell_element`, and `solid_element` are solver/discretisation concepts
- do not call an authored `Member` a line element in the structural-app UI
- do not call an authored `Plate` a plate element in the structural-app UI

When a structural object is split for analysis, the split should remain visible:
- `Beam B1` may be discretised into several analysis elements
- selecting the member should show the object type, role, nodes, and analysis-element count
- selecting or discussing an analysis element should say which role-labelled member it belongs to where that mapping exists
- splits should be explained by connectivity, support/restraint, load discontinuity, release, property change, non-collinearity, or import cleanup

---

## 3. Primary authored object types

## 3.1 Node
A structural point in space used for:

- connectivity
- supports
- nodal loads
- member endpoints
- plate boundary points

A Node is a structure-specific wrapper around positioned geometry plus structural semantics.

### Candidate responsibilities
- position in 3D
- snapping / selection anchor
- participation in connectivity graph
- host for some assignments

---

## 3.2 Member
The primary line-based structural authored object.

Examples of semantic member roles:

- beam
- column
- brace
- rafter
- purlin
- tie

### Important design rule
Beam/column/brace should usually be semantic classifications of Member, not completely separate deep object families.

### Candidate Member properties
- reference line / analytical centerline
- section assignment
- material assignment
- orientation / local-axis rule
- primary semantic role
- semantic context tags
- optional offsets/releases/connection refs

### 3.2.1 Semantic classification is layered

Current implementation treats semantic classification as layered metadata on top of the primitive object, not as a single inheritance hierarchy.

For members and plates:

- object type remains the authored primitive: `Member` or `Plate`
- `role` is the singular primary engineering classification
- `semantic_tags` are multiple context/system tags
- supports, loads, releases, connections, tributary links, and other assignments remain explicit relationship objects

Examples:

- roof beam: `Member { role: "beam", semantic_tags: ["roof", "primary"] }`
- rafter: `Member { role: "rafter", semantic_tags: ["roof"] }`
- floor slab: `Plate { role: "slab", semantic_tags: ["floor"] }`
- roof panel: `Plate { role: "roof_panel", semantic_tags: ["roof"] }`

`roof` and `floor` are context tags, not parent object types.

Unknown or unresolved objects should use a conservative role such as `unclassified` until confirmed. Generic geometry-only inference can suggest roles/tags, but downstream checks should consume the explicit semantic layer once it exists rather than repeatedly inferring meaning from geometry alone.

### 3.2.2 Section family guidance

Section family guidance should come from explicit project intent, retrieved wiki knowledge, catalogue metadata, or reviewed agent-authored planning records. Fraia should not maintain a hidden role-to-family table such as "rafter means UB/PFC" in generic runtime code.

For the current Australian steel catalogue, available family names are derived from the catalogue, for example `UB`, `UC`, `PFC`, `RHS`, `SHS`, `CHS`, and `EA`. These names are allowed vocabulary for constraints and option intents, not final section assignments.

This is guidance metadata only. The exact `section_id` remains a separate downstream property selected by catalogue lookup, sizing, optimisation, or manual user choice after the relevant assumptions and objectives are explicit.

Near-term behavior:

- agents may propose allowed or excluded section families when the user states a hard preference or when wiki/project evidence justifies an option intent
- deterministic code validates proposed family names against the catalogue-derived family list
- design-option intent can vary section-family policy as an assumption, such as open sections, closed sections, or standardised families
- diagnostics should surface weak grouping, awkward family mixes, or unsupported assumptions instead of silently accepting them

---

## 3.3 Plate
The primary surface-based structural authored object.

Examples of semantic plate roles:

- slab
- wall panel
- plate region
- deck-like panel later

### Candidate Plate properties
- boundary geometry / surface region
- thickness / section-like property
- material assignment
- local orientation / normal rule
- semantic role(s)
- meshing intent later

---

## 3.4 Solid
Not necessarily required for the earliest UI, but should exist in the long-term object model.

Examples:

- local detail region
- continuum block
- thick connection/submodel volume

Solids should likely enter the app later than Members and Plates.

---

## 4. Assignments and conditions

Supports and loads should generally not be modeled only as built-in node properties.

Instead they should be separate assignment objects attached to targets.

This gives better flexibility and scales beyond node-only behavior.

### 4.0.1 Base Model vs design options

In the design-option workspace, the Base Model should be treated as the common starting evidence for option generation:

- authored geometry/topology
- member and plate roles/tags
- section/material hints where known
- load assignments and load cases

The Base Model should not carry a committed support strategy by default.
Support assumptions, connection behaviour, member end releases, bracing/stability choices, and related restraint decisions are part of a design option because they describe how that option stands.

This keeps the workflow clear:
- design options are generated from Base Model evidence, load demand, confirmed constraints, retrieved wiki knowledge, and agent-authored `DesignOptionIntent` records
- each option can propose its own support, connection, release, and stability assumptions as a comparison artefact
- option pages may hide load glyphs by default so the visible comparison focuses on the proposed standing/stability model, while still retaining the load demand as Base Model requirements
- option chats review and revise realised options; changing an option creates a replacement option and marks the original superseded rather than mutating the Base Model
- authored structural primitives remain upstream truth and are not overwritten by design-option chat

The visible desktop journey has three stages:

**Base Model → Design Options → Analysis & Comparison**

`DevelopmentPath` and output artefacts remain internal work and evidence inside Analysis & Comparison; they are not separate top-level destinations. Engineering Evidence is a contextual drill-down that returns to the originating option or path.

The decision sequence is:

1. generate a traceable option batch from a ready Base Model brief
2. inspect, revise, and shortlist option revisions without deleting excluded or earlier work
3. run preliminary analysis for included revisions that do not have current evidence
4. compare the included options and record Fraia's explained recommendation
5. create or reopen one or more preserved option paths and inspect traceable outputs within Analysis & Comparison

`DesignOptionBatch`, `DesignOptionRevision`, `DesignOptionComparisonRun`, and `DevelopmentPath` are persisted project decisions. Every option revision has a batch-scoped stable identity so a regenerated option cannot inherit evidence or a path merely because it reused an authored option id. Solver and check outputs remain immutable run artefacts. Editing the Base Model marks its active option batch outdated; revising an option creates a replacement revision and leaves the earlier revision and its evidence inspectable.

---

## 4.1 SupportAssignment
Represents support/restraint behavior attached to a target.

Possible targets:

- Node
- Member end
- Plate edge
- Plate region
- future Solid face/region

### Candidate SupportAssignment properties
- target reference
- support family/type
- restrained directions/DOFs
- frame reference
- optional spring/stiffness properties later

---

## 4.2 LoadAssignment
Represents load application attached to a target.

Possible targets:

- Node
- Member
- Member end
- Plate
- Plate edge
- region/body later

### Candidate LoadAssignment properties
- target reference
- load family/type
- magnitude/distribution stored in canonical SI units
- direction with frame reference
- load case reference
- optional realization/distribution hints when the global model uses a lower-dimensional idealization

Current canonical load magnitudes:
- point loads: newtons
- uniform line loads: newtons per metre
- moments: newton-metres
- pressures/stresses: pascals when represented as resolved quantities

Engineering input and display may use `kN`, `kN/m`, `kN·m`, or `MPa`, but those are presentation units around the authored/resolved data model.

### 4.2.1 Area loads should remain load assignments, not separate truth objects

An area load should usually be represented as a `LoadAssignment` targeting a Plate or another surface-like region.

Examples:
- uniform pressure on a roof plate
- wall pressure on an elevation plate
- floor area load on a slab plate

The load assignment is the engineering truth.
The downstream realization may then choose how to represent it for a specific analysis form.

Examples of downstream realization behavior:
- keep it as a surface/plate pressure in a shell-capable realization
- convert it to equivalent line loads on supporting members in a reduced line-member global model
- distribute it to 1D members using tributary width/tributary area rules when the current realization cannot solve the full plate directly

This means Fraia should distinguish clearly between:
- the authored load assignment
- the derived equivalent loads created for a chosen realization

The equivalent nodal/line/member loads are downstream artifacts, not the original authored truth.

### 4.2.2 Typed load families will matter

Fraia has now started this move in the Rust MVP with a first typed `LoadKind` layer.
Current MVP-authored kinds are:

- `point`
- `uniform_line`
- `area`

Current authored target expectations are:

- `Node` -> `point`
- `Member` -> `uniform_line`
- `Plate` -> `area`

Useful categories will likely continue to expand over time and may include:

- nodal point force
- nodal moment
- member point load
- member uniform/distributed line load
- plate pressure / area load
- plate edge line load
- surface traction later

This makes it easier for both the GUI and the agent to understand what kind of engineering action is being applied and how it may realize downstream.

---

## 4.3 ReleaseAssignment
Represents member-end or interface release behavior.

Typical targets:

- Member end
- interface/connection later

### Candidate ReleaseAssignment properties
- target reference
- released DOFs/behavior
- frame or local-axis context

---

## 4.4 Constraint / InterfaceAssignment
Needed for more advanced coupling.

Examples:

- rigid link
- tie
- member-to-plate interface
- plate-edge attachment
- future member-to-solid or plate-to-solid coupling

This likely becomes increasingly important as Fraia grows into mixed-dimensional interaction and submodeling.

---

## 5. Connection and detailing-related authored concepts

For early Fraia, detailed bolts/welds do not need to be first-class authored FE objects.

But Fraia should still carry concepts such as:

- connection family preference
- connection demand/resultant actions
- actionable detail warning
- local check recommendation

### Candidate objects
- `ConnectionDemand`
- `ActionableEngineeringWarning`
- `DetailLevelRequest`
- `ConnectionFamilyPreference`

These can bridge global modeling and local detailed follow-up.

---

## 6. Realization is downstream, not primary authored identity

A Member is authored as a Member.

It may later realize as:

- line/frame model
- shell model
- solid model

A Plate may later realize as:

- shell/plate model
- solid model

A Solid may realize as:

- continuum/solid model

This means authored object identity remains stable while analysis fidelity changes downstream.

---

## 7. Builders and archetypes sit above authored primitives

The user should often work with higher-level structural builders such as:

- portal frame builder
- truss builder
- floor framing builder
- wall/slab builders later

These builders should generate and edit authored objects like:

- Nodes
- Members
- Plates
- assignments

This keeps Fraia productive while still preserving a practical direct-edit structural model beneath.

---

## 8. Visualization hooks

These authored objects should carry or derive visual metadata.

### Node
- marker
- snap point
- selection handle

### Member
- centerline geometry
- local axis display
- role/section display metadata

### Plate
- boundary surface
- plate symbol/mesh preview cues
- interaction edges

### Assignments
- support glyphs
- load arrows
- release icons
- warning highlights

This is important for the structural workbench UI.

---

## 9. What should likely be hidden from most users

Even if the lower engine contains them, the structural app should mostly hide:

- raw tensor language
- raw frame/transform math
- solver-specific element syntax
- mesh nodes/elements except when explicitly inspecting realizations
- generic coupling internals

These should remain inspectable, but not primary authoring tools.

---

## 10. Recommended first structural app object set

A strong first structural app object model is likely:

- Node
- Member
- Plate
- SupportAssignment
- LoadAssignment
- ReleaseAssignment
- BuilderGraph / BuilderNode
- ActionableEngineeringWarning
- ConnectionDemand
- DetailLevelRequest

This is enough to support a serious structural workbench direction without exposing unnecessary deep-engine complexity.

---

## 11. Design choices currently favored

- The structural app should be structure-specific, not generic-math-specific.
- Member is the main authored line object; beam/column/brace are semantic roles on top.
- Plate is the main authored surface object; slab/wall are semantic roles on top.
- Supports and loads should generally be assignment objects attached to targets, not only node properties.
- Connection/detail concepts should initially be reasoning/orchestration objects more than direct FE detail objects.
- Realization/fidelity should remain downstream from authored identity.

---

## 12. Open questions

- Whether Solid should be a first production authored object or introduced later
- Exact target reference scheme for assignments
- Exact relationship between Builder output and direct structural editing
- When to expose realization overrides in the UI

---

_End of draft._
