# Fraia Optimization and Iteration Loop

_Status: draft v0.1_
_Date: 2026-04-13_

This document captures the current direction for Fraia's autonomous iteration engine.

---

## 1. Purpose

Fraia should not be limited to section resizing loops.

A useful engineering iteration engine must be able to explore multiple scales of change, including:

- section/property changes
- support condition changes
- brace layout changes
- span/grid changes
- added supports or reduced spans where acceptable
- different structural systems entirely

The goal is not to find one "correct" answer.

The goal is to generate and compare multiple viable options under user intent, constraints, and tradeoffs.

---

## 2. Core principle

### Optimization is bounded by intent
The iteration engine should not operate in a vacuum.

Before looping begins, Fraia needs a sufficiently clear description of:

- what the user is trying to achieve
- what tradeoffs are acceptable
- what is non-negotiable
- what kinds of changes are allowed
- what kinds of changes require explicit approval

This means planning mode is not optional. It defines the bounds of the search.

### Optimization/modeling loops should usually be incremental, not one-shot

For many problems, Fraia should not try to generate one whole structure once and call that done.

A better pattern is:
1. create an initial structural system or primitive model
2. validate the model
3. solve/analyze deterministically
4. inspect governing diagnostics/results
5. revise geometry, loads, topology, or sizing
6. repeat until the design is acceptable or a decision gate is hit

This applies both to very small problems and larger buildings.

Examples:
- simply supported beam sizing loop
- portal frame topology/sizing loop
- building subsystem composition loop

This is the style of agent behavior Fraia should support.

---

## 3. Fraia should explore multiple classes of changes

At minimum, Fraia should distinguish several search neighborhoods.

## 3.1 Parametric sizing changes
Examples:

- section size changes
- thickness changes
- material grade changes
- spacing changes within existing topology

This is the most conservative and lowest-risk search class.

## 3.2 Local topology changes
Examples:

- add/remove brace
- add intermediate support
- change release/support pattern
- adjust member arrangement in one bay

This changes the structure meaningfully without replacing the whole system.

## 3.3 System-level alternatives
Examples:

- moment frame -> braced frame
- portal frame -> truss roof system
- long-span beam solution -> more columns with shorter spans
- one-way framing -> two-way framing

This is where Fraia begins exploring fundamentally different valid structures.

## 3.4 Planning-level alternative concepts
Examples:

- fewer internal columns vs lower cost
- larger clear spans vs lighter frame
- simpler erection vs lower steel tonnage
- lower embodied carbon vs lower fabrication simplicity

This connects optimization back to user intent and tradeoffs.

---

## 4. There is rarely one answer

Fraia should generally assume that engineering design is a tradeoff space, not a single-solution problem.

Therefore, a good Fraia workflow should often return:

- top 3-5 candidate options
- each with objective values
- each with assumptions and tradeoffs
- each with clear explanation of why it is interesting

This is closer to how a strong consultant behaves.

---

## 5. Intent model required before iteration

Before autonomous iteration begins, Fraia should establish:

### Hard blockers / non-negotiables
Examples:

- no internal columns allowed
- max building depth fixed
- must use steel
- roof pitch fixed
- certain supports/foundations unavailable

### Soft preferences
Examples:

- prefer lower cost
- prefer lower carbon
- prefer fewer unique sections
- prefer simpler fabrication
- prefer open floor space

### Search permissions
Examples:

- section resizing allowed
- adding braces allowed
- adding supports allowed
- changing spans allowed within envelope
- changing structural system allowed or not

### Approval thresholds
Examples:

- auto-explore conservative changes freely
- ask before changing building layout
- ask before introducing interior supports
- ask before changing material system

This information defines the search bounds.

---

## 6. Search levels / exploration bands

A useful Fraia engine may operate in explicit exploration bands.

## Band A — conservative sizing loop
Only changes sizes/properties inside an existing system.

## Band B — local structural edits
Allows braces, supports, release changes, local topology edits.

## Band C — system alternatives
Allows materially different framing systems/archetypes.

## Band D — planning-level alternatives
Allows different concept directions if still aligned with the user's broad intent.

This gives Fraia a controlled way to widen the search instead of jumping randomly.

---

## 7. Multi-agent/subagent strategy

A multi-agent architecture makes a lot of sense here.

A possible pattern:

## 7.1 Main orchestrator
Responsibilities:

- read user intent
- maintain global constraints/objectives
- decide which exploration bands are active
- compare candidate results
- select/report top options

## 7.2 Conservative optimizer agents
Responsibilities:

- search size/property combinations
- refine known topology
- improve utilization/serviceability efficiently

## 7.3 Topology agents
Responsibilities:

- add/remove braces
- test support changes
- reduce spans or add supports where permitted
- propose local framing improvements

## 7.4 Creative system agents
Responsibilities:

- propose structurally different alternatives
- instantiate different archetypes
- explore radically different but valid systems within intent bounds

## 7.5 Diagnostic agents
Responsibilities:

- explain why candidate systems fail
- identify governing blockers
- suggest next directions

This split may be exposed through an agent runtime, but it remains a Fraia architecture concern first.

---

## 8. Creativity must be bounded

Creative exploration is valuable, but must remain controlled.

The inventive agent should not be free to violate the user's intent silently.

Instead, it should work within:

- hard constraints
- explicit search permissions
- project brief
- architectural envelope
- allowed compromises

This is how Fraia can be both inventive and trustworthy.

---

## 9. Candidate comparison

The orchestrator should compare candidates using both numeric and narrative summaries.

Candidate comparison should eventually include:

- cost estimate
- carbon estimate
- weight/tonnage
- max utilization
- max deflection/drift
- buckling margin
- constructability complexity
- section count / standardization
- need for extra supports/columns/braces
- conceptual impact on architectural intent

This supports a consultant-like output rather than a blind single optimum.

---

## 10. Pareto thinking

Fraia should likely think in Pareto-frontier terms rather than pure single-objective optimization.

Examples of competing objectives:

- lower cost vs fewer columns
- lower carbon vs simpler fabrication
- lower weight vs lower drift
- architectural openness vs structural efficiency

This suggests Fraia should often preserve multiple strong candidates instead of collapsing too early to one answer.

---

## 11. Output style

Fraia should ideally report options in a consultant-like format.

Example output style:

- Option A: lowest cost, more internal supports
- Option B: best clear span, higher steel weight
- Option C: best carbon, more complex fabrication
- Option D: simplest erection, moderate cost increase

Each option should explain:

- what changed
- why it performs well
- what tradeoff it makes
- what assumptions or approvals it depends on

---

## 12. Provenance requirements

Every candidate explored should ideally carry provenance such as:

- parent candidate
- change set
- exploration band
- responsible agent/subagent
- objective values
- constraint pass/fail state
- reason kept or discarded

This is important for trust and debugging.

---

## 13. Suggested first implementation path

A sensible early sequence:

### Phase 1
- conservative size-based loop only
- one system at a time
- return top few sizing variants
- start with very small deterministic problems such as simply supported beams with point loads and distributed loads

### Phase 2
- allow local topology changes
- braces and supports within user-approved limits

### Phase 3
- allow multi-archetype/system alternatives
- compare options consultant-style

### Phase 4
- add explicit orchestrator + specialized subagents
- preserve Pareto sets and richer tradeoff reporting

---

## 14. Design choices currently favored

- Fraia should go well beyond section-size-only optimization.
- Planning mode must establish search permissions and acceptable compromises.
- Multiple exploration bands are needed, from conservative to inventive.
- A multi-agent architecture is a natural fit.
- Fraia should usually return several strong options, not one answer.
- Tradeoff explanation is as important as numeric optimization.

---

## 15. Open questions

- Exact intent schema for hard constraints vs soft preferences vs search permissions
- When Fraia should auto-widen the search versus asking the user first
- How to score candidate diversity versus raw objective quality
- How many options Fraia should retain by default
- How much of the loop should be deterministic search versus agent creativity

---

_End of draft._
