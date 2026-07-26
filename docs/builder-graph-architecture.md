# Fraia Builder Graph Architecture

_Status: draft v0.1_
_Date: 2026-04-14_

_Canonical focus:_ builder graph and builder-node architecture.
_See also:_ `engineering-core.md`, `resolution-and-runs.md`, `engineering-output-pipeline.md`, `documentation-map.md`.

This document defines the intended long-term Fraia direction for builder/archetype workflows.

---

## 1. Why this layer exists

Fraia needs a layer above authored primitives that can preserve:

- concept-level structural intent
- reusable parametric structural patterns
- adoption provenance from option studies
- future regeneration workflows
- future subsystem composition

That layer should not replace the authored structural model.

It should instead provide a typed, deterministic concept graph that can materialize into authored primitives.

---

## 2. The four persistent layers

### 2.1 Intent / rationale layer
Human + LLM planning context.

Examples:
- planning markdown
- design assumptions
- decision logs
- tradeoff summaries
- brief interpretation notes

This layer is rich, historical, and explanation-focused.

### 2.2 Builder graph layer
Typed, compact project concept model.

Examples:
- portal frame subsystem
- braced bay subsystem
- mezzanine subsystem
- industrial hall root concept

This layer is structured and regeneration-focused.

### 2.3 Authored structural model layer
The actual structural objects Fraia validates, realizes, analyzes, renders, and edits.

Examples:
- nodes
- members
- plates
- supports
- loads
- releases

This remains the current authoritative editable model.

### 2.4 Immutable run layer
Run-time snapshots and provenance.

Examples:
- optimization options
- adopted source run ids
- validation artifacts
- analysis diagnostics
- summaries

---

## 3. Core definitions

## 3.1 Archetype definition
A reusable, versioned structural generator in Fraia's catalog.

An archetype definition should include at least:
- id
- version
- name
- description
- category
- scale
- parameter schema
- generator implementation
- compatibility/validation rules
- composition interface expectations

Examples:
- `frame.portal_2d_steel_concept.v1`
- `lateral.braced_end_bays.v1`
- `building.industrial_hall.v1`

## 3.2 Builder node
A project-specific instance of an archetype.

A builder node should include at least:
- id
- optional label
- archetype id
- typed parameters
- child node ids
- source run id / source option index when applicable
- materialization status

## 3.3 Builder graph
A project-specific graph of builder nodes.

Early Fraia can keep this simple:
- one root node
- one supported archetype family
- no explicit interface graph yet

Longer term it should support:
- multiple roots if needed
- parent/child composition
- subsystem interfaces
- selective regeneration
- partial manual override boundaries

---

## 4. Relationship to authored primitives

The contract is:

- builder graph = compact concept model
- structural model = expanded authored engineering model

The builder graph should be able to materialize a structural model.
The structural model should not be forced to stay perfectly synchronized after manual edits.

Instead Fraia should track builder-node state such as:
- proposed
- materialized
- diverged from materialization

This avoids pretending that manually edited authored geometry is still a pristine archetype expansion.

### 4.1 Manual additions should not silently rewrite builder meaning

If a user or agent manually edits generated geometry, Fraia should not silently guess a new archetype meaning and overwrite the builder graph automatically.

Default behavior should be:
- keep the edited primitives as authored truth
- preserve known builder provenance where it still applies
- mark affected builder nodes as diverged when their current parameters no longer explain the authored state cleanly
- treat brand-new manually added objects as authored/manual objects unless the user explicitly asks Fraia to attach or promote them

This preserves trust and predictable regeneration boundaries.

### 4.2 Explicit re-parameterization should exist

Although Fraia should not silently rewrite builder logic, it should support an explicit workflow to reconsider the parameterized structure after mixed manual and generated editing.

Useful user-visible actions later include:
- apply edit back to existing builder
- parameterize selection
- promote selected primitives to project-local builder
- re-parameterize current subsystem
- re-parameterize current model

The important distinction is:
- default editing preserves authored truth first
- re-parameterization is an explicit proposal/review workflow that can replace or update the compact builder explanation layer

### 4.3 Re-parameterization should treat old builders as hints, not unquestioned truth

When Fraia re-parameterizes a mixed model, it should inspect:
- current authored primitives
- existing builder provenance where available
- manual additions and topology changes
- loads/supports/releases and other engineering intent attached to the current model

Then it should propose a new parameterized structure such as:
- updated existing builder nodes
- new child builder nodes
- some objects left manual and unattached to builders

This should be shown to the user for approval before replacing or restructuring the builder graph.

---

## 5. Immediate implementation shape

The current implementation direction should be:

- store `builder_graph` on `ProjectFile`
- allow one root node in practice for now
- support one initial archetype family:
  - `frame.portal_2d_steel_concept.v1`
- materialize that graph into `StructuralModel`
- persist builder-node-to-authored-object materialization mappings on the structural model
- mark builder nodes as diverged when authored primitives are edited manually
- allow explicit rebuild of the authored model from the builder graph

This gives Fraia the correct long-term trunk without requiring full subsystem composition on day 1.

---

## 6. Near-term roadmap

### Phase A — single-root graph
- one root node
- one portal-frame archetype
- adoption provenance on builder node
- explicit rebuild from graph
- divergence tracking in UI

### Phase B — multiple subsystem nodes
- several builder nodes in one project
- root concept + subsystem children
- independent materialization per subsystem
- richer diagnostics tied to builder nodes

This phase is now partially started in code:
- concept-root nodes can own child builder nodes
- multiple builder nodes can materialize into one structural model
- each builder node gets its own materialization map back to authored objects

### Phase C — explicit interfaces / composition
- ports/interfaces between subsystem builders
- parent-to-child parameter mapping
- cross-node compatibility validation
- selective regeneration of one subsystem without rebuilding the entire project

### Phase D — mixed generated/manual authoring
- manual override boundaries
- detach/reattach flows
- node-level stale/overridden diagnostics
- stronger authored-vs-generated UX in the workbench
- explicit promotion of manual selections into project-local builders
- explicit re-parameterization/re-synthesis of mixed user+agent authored models

---

## 7. Archetype management model

Archetypes should be managed as a catalog of typed generators, not as free-form LLM text.

The LLM should help with:
- interpreting user intent
- choosing archetypes
- proposing parameters
- comparing tradeoffs
- explaining implications

But deterministic generation should live in Fraia code.

This keeps builder materialization:
- auditable
- testable
- reproducible
- engineerable

---

## 8. Initial Fraia rule of thumb

Use the highest structural abstraction that is still explicit and reliable.

Examples:
- use a builder node when the subsystem is still archetype-driven
- use authored primitives when the project needs direct custom control
- keep run artifacts immutable
- keep planning/rationale separate from the builder graph

---

## 9. Recommended future additions

Likely future data-model additions include:
- builder node interfaces/ports
- parameter schemas with units and ranges
- node-local validation diagnostics
- richer materialization mappings from builder nodes to structural object ids and regions
- manual override boundaries
- project-local builder promotion metadata for selected authored primitives
- re-parameterization proposal objects / regeneration summaries
- builder graph diffing and regeneration summaries
- archetype package loading/version pinning

---

## 10. Current design decision

Fraia should move from a single flat builder-instance idea toward a builder-graph architecture now, while keeping the first implementation intentionally small.

That means:
- right architecture trunk now
- low implementation complexity now
- room for whole-building and subsystem composition later

---

_End of draft._
