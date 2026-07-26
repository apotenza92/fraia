# Fraia Hierarchical Fidelity and Submodeling

_Status: draft v0.1_
_Date: 2026-04-13_

This document captures the current direction for Fraia's staged analysis fidelity, realization policy, and local detailed submodeling.

---

## 1. Core idea

Fraia should not use the highest-fidelity element types everywhere by default.

Instead, Fraia should generally:

1. start with the lowest reasonable analysis complexity
2. analyze the overall structure efficiently
3. identify critical regions or unanswered questions
4. selectively increase fidelity where detail is warranted

This is both:

- computationally sensible
- closer to real engineering practice

---

## 2. Why this matters

For global structural behavior, it is often best to use:

- line members for frames
- plates/shells where surfaces are essential
- simplified support and load models

For local behavior, it may be better to use:

- shell/plate models of members or joints
- solid models of local connection regions
- refined plate or wall models

Trying to model everything at maximum fidelity from the start is usually:

- too expensive
- too slow
- harder to interpret
- unnecessary for many decisions

---

## 3. Fraia should support staged fidelity

A useful default strategy is:

## Stage A — Global low-complexity model
Use the simplest model that captures the system-level behavior reasonably.

Typical examples:

- members as line/frame elements
- slabs/walls as plates only if needed
- no local solid detailing yet

Purpose:

- overall force paths
- drift/deflection
- preliminary sizing
- option comparison

## Stage B — Identify critical regions
Use global results and diagnostics to decide where more detail may be needed.

Examples:

- highly utilized connection zones
- members with thin walls or local instability concerns
- regions with severe stress gradients
- plate/member interaction zones
- supports or load introduction regions

## Stage C — Local fidelity increase / submodeling
Create more detailed local models where appropriate.

Examples:

- member-to-plate interface region as shell model
- connection region as shell or solid model
- plate strip or wall panel refined locally
- selected member realized as shell/solid for local checks

---

## 4. Realization complexity should be chosen intelligently

The structural system and Fraia agent should pick realization complexity based on:

- what is being modeled
- what question is being asked
- required accuracy/fidelity
- interaction complexity
- solver/runtime budget

This means realization choice is not only a user toggle.

It is also an engineering inference problem.

---

## 5. Example policy

### Global building concept study
Use:

- member/frame realizations where possible
- plate realizations only where essential

### Slab-on-beam interaction study
Use:

- member + plate model
- line-to-surface coupling

### Connection/local wall/thin-section investigation
Use:

- shell or solid local submodel
- higher mesh density
- refined boundary transfer from global model

This is the kind of staged fidelity Fraia should aim for.

### Practical note on connection detail
In real workflows, detailed FE connection studies are usually selective.

Fraia should therefore prefer:

- global action extraction first
- simplified/detail-family reasoning next
- shell/solid local submodeling only where warranted

---

## 6. Submodeling as a first-class concept

Fraia should eventually support explicit submodeling workflows.

A submodel may be:

- derived from a larger/global model
- bounded by a region of interest
- supplied with transferred forces/displacements from the parent model
- analyzed at higher fidelity than the parent

This is a natural way to connect:

- fast global analysis
- detailed local analysis

---

## 7. What Fraia should infer from the global model

After a lower-complexity run, Fraia should be able to ask:

- which regions govern?
- which regions are uncertain under the current fidelity?
- which interfaces are worth refining?
- which objects require local shell/solid realization?
- which connection or plate regions deserve a more detailed model?

This suggests a future role for a fidelity-selection or refinement agent.

---

## 8. User control vs Fraia control

### Default behavior
Fraia should choose a low-complexity global model first where reasonable.

### Engine/agent behavior
Fraia should propose or trigger higher-fidelity local studies when justified.

### Advanced user behavior
Users should still be able to:

- force a realization policy
- request shell/solid detail explicitly
- request submodeling on a selected region

So the right model is:

- automatic by default
- inspectable
- overridable

---

## 9. Relationship to authored objects

The authored structural objects remain stable:

- Member
- Plate
- Solid

What changes is the **analysis realization** and **analysis scope**, not the authored intent.

A Member may be analyzed as:

- line/frame globally
- shell locally
- solid in a connection/detail submodel

This is a realization/fidelity decision, not a new authored object type.

---

## 10. Generic engine implications

This is not purely structural-specific.

At the deeper engine level, Fraia likely needs generic support for:

- multiple realizations of one authored object
- mixed-dimensional coupling
- region-of-interest extraction
- parent-to-submodel boundary transfer
- fidelity escalation policies

So yes: much of this belongs in the generic engine, even though the first use case is structural engineering.

---

## 11. Practical first implementation path

### Phase 1
- global line-member analysis only
- plate/solid concepts in schema only

### Phase 2
- add plate/shell realizations
- add member-plate interaction
- add interface-aware diagnostics

### Phase 3
- add local submodel extraction
- allow selected regions/connections to be realized in shell/solid form
- transfer global actions/displacements into submodels

### Phase 4
- smarter automatic refinement suggestions
- fidelity escalation policies driven by diagnostics/results

---

## 12. Design choices currently favored

- Fraia should prefer lower-complexity global models where adequate.
- Higher-fidelity analysis should be targeted and selective.
- Submodeling should become a first-class workflow.
- Realization complexity should be chosen based on what is being studied.
- Authored objects should remain stable while realizations change.
- Much of this capability belongs in the generic engine, not only the structural app.

---

## 13. Open questions

- Exact submodel extraction workflow
- Exact boundary transfer strategy between parent and local models
- How Fraia should decide when a local refinement is justified
- Whether realization policy should be defined per object, per region, per analysis request, or all three

---

_End of draft._
