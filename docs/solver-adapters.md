# Fraia Solver Adapters

_Status: draft v0.1_
_Date: 2026-04-14_

_Canonical focus:_ Fraia-to-solver contracts and bounded near-term solver strategy.
_See also:_ `resolution-and-runs.md`, `engineering-output-pipeline.md`, `package-system.md`, `documentation-map.md`.

This document captures the current direction for Fraia solver adapters and the boundary between canonical Fraia models and external numerical engines.

---

## 1. Purpose

Fraia should not hardwire itself to one solver’s native model format.

Instead Fraia should:

- maintain its own canonical project/package model
- resolve that into a normalized engineering representation
- compile that representation into solver-specific inputs via adapters

This is essential for:

- long-term flexibility
- multi-solver support
- migration safety
- agent-readable canonical data

---

## 2. Core principle

### Fraia owns the canonical model
Solvers do not.

External solvers are execution backends.

Fraia should not treat:

- OpenSees syntax
- CalculiX decks
- solver-native mesh files

as the canonical authored truth.

---

## 3. Adapter responsibilities

A solver adapter should be responsible for:

- declaring capabilities
- validating whether a resolved Fraia model is representable
- translating resolved Fraia objects into solver-ready inputs
- invoking the solver/runtime
- extracting and normalizing results back into Fraia result types
- surfacing solver logs and diagnostics

---

## 4. Capability declarations

Each adapter should eventually declare what it supports.

Examples:

- supported primitive families
- supported analysis forms
- supported material/property classes
- supported result types
- supported connection/support semantics

### Example sketch

```json
{
  "solver": "OpenSeesPy",
  "supports": {
    "analysisForms": ["frame_1d_in_3d", "truss_1d_in_3d"],
    "analysisTypes": ["linear_static", "modal", "buckling"],
    "primitiveFamilies": ["curve_member", "support", "release"],
    "results": ["displacement", "reaction", "member_force"]
  }
}
```

---

## 5. Why adapters matter for Fraia’s abstraction policy

Agents should not have to think in solver-native terms by default.

They should work in Fraia’s canonical abstractions.

The adapter layer then handles:

- realization choices
- solver-specific local axis conventions
- solver-specific constraint syntax
- output normalization

This protects the higher layers from backend-specific leakage.

---

## 6. Fraia to solver pipeline

Likely sequence:

1. authored Fraia project
2. resolution into canonical resolved Fraia model
3. adapter representability check
4. adapter compilation to solver-native model/input
5. solver execution
6. result extraction
7. normalization into Fraia results
8. frozen run snapshot + summaries

---

## 7. Adapter errors vs Fraia validation errors

These should be distinguished.

## Fraia validation error
The canonical resolved model is incomplete/invalid or unsupported even before solver translation.

## Adapter representability error
The Fraia model is valid in principle, but cannot be represented by this adapter.

Example:

- Fraia model includes surface regions
- chosen adapter only supports frame members

## Solver runtime error
The adapter compiled successfully, but the solver failed or did not converge.

This separation will make diagnostics much clearer.

---

## 8. Initial recommended adapter strategy

Start with one adapter only.

Candidate first adapter:

- OpenSeesPy for line-member structural systems

Potential later adapters:

- CalculiX
- Code_Aster
- Kratos
- mesh-related toolchain components such as Gmsh where appropriate

---

## 8A. Bounded near-term solver plan

Fraia should not pause app progress to build a full custom solver stack immediately.

The recommended near-term path is:

### Step 1 — keep the current internal frame solver
Use the current Fraia-native frame solver for:
- pipeline development
- design-action extraction
- early checks/exports
- desktop workbench progress
- regression testing

This keeps product work moving.

### Step 2 — define Fraia solver requirements before committing
Before deeper solver integration, define the minimum required capability set.

Initial evaluation dimensions should include:
- line-member support quality
- static linear analysis support
- result extraction quality
- scripting/API fit
- license compatibility
- difficulty of adapter implementation
- ability to preserve Fraia provenance/run artifacts
- future path toward richer analysis forms

### Step 3 — perform a bounded evaluation of open-source solvers
The goal is not open-ended research.

The goal is to produce:
- a short candidate list
- a comparison matrix
- a recommended first serious adapter target
- a clear statement of what Fraia still owns vs what the solver owns

### Step 4 — strengthen the adapter contract before deep integration
Before integrating a serious external solver, Fraia should tighten:
- representability checks
- compiled input preservation
- normalized result extraction shape
- capability declaration structure

### Step 5 — integrate one serious adapter path
After the bounded evaluation, integrate one external solver path while keeping the Fraia-native solver as:
- a test baseline
- a concept-study baseline
- a fallback/simple internal path

This is the current recommended order because it preserves app momentum while still preparing for stronger analysis capability.

## 8B. Fraia vs solver-side tool ownership boundary

The adapter layer should preserve a clear ownership split.

### Fraia owns
- import / cleanup / semantic enrichment
- authored vs resolved vs run separation
- provenance and immutable artifacts
- normalized result extraction and cross-backend comparison
- agent/user review loops
- domain workbench UX
- the main Fraia product GUI

### Solver / solver-side companion tools own
Examples:
- `ccx` for CalculiX execution
- `cgx` for optional CalculiX-side viewing / meshing / raw inspection

These tools are good places to delegate:
- solver execution
- backend-native input/output files
- optional raw result / mesh / deck inspection
- optional backend debugging
- selected backend-adjacent pre/post tasks when they are materially useful

### Shared adapter boundary
The Fraia-to-solver contract should carry things like:
- compiled solver input
- runtime invocation metadata
- raw backend logs/files
- representability diagnostics
- normalized extracted results returned back into Fraia

### Current recommendation for CalculiX / CGX specifically
- `ccx` is a legitimate execution backend target for Fraia.
- `cgx` can be a useful **optional companion tool** for backend-side inspection/debugging.
- `cgx` should **not** be treated as the foundation for Fraia’s main GUI/workbench.

---

## 9. Normalized Fraia results

Adapters should not just dump native solver output blobs.

They should map outputs into Fraia result structures such as:

- nodal displacements
- reactions
- member end/internal forces
- envelopes
- buckling modes/factors
- warnings
- convergence status

This normalized result layer is critical for agents and multi-solver comparison.

---

## 10. Solver input preservation

For reproducibility and debugging, Fraia runs should ideally preserve:

- canonical resolved Fraia snapshot
- adapter-specific compiled input
- solver logs
- normalized results

This means a run can later be inspected from multiple viewpoints.

---

## 11. Local axes and frame handling

Adapters will likely be one of the main places where Fraia frame/orientation logic must become concrete.

Examples:

- member local axis mapping
- support direction realization
- release semantics
- shell normal conventions later

This reinforces why Fraia’s canonical frame/connectivity model must be explicit and deterministic.

---

## 12. Solver adapters should stay stateless where possible

Where practical, adapters should behave like pure compilation/execution units:

- consume resolved model + settings
- emit compiled input + execution results

This helps with:

- testing
- reproducibility
- caching
- easier replacement

---

## 13. Agent implications

Agents should be able to ask:

- what solvers can run this model?
- why can’t this model run on the selected solver?
- what result types are available?
- what approximation did the adapter make?

This means capabilities and representability diagnostics should be structured and queryable.

---

## 14. Suggested early Fraia adapter API shape

Eventually, an adapter may need operations like:

- `describeCapabilities()`
- `checkRepresentability(resolvedModel, analysisRequest)`
- `compile(resolvedModel, analysisRequest)`
- `run(compiledInput)`
- `extractResults(runArtifacts)`

The exact API is still open.

---

## 15. Design choices currently favored

- Fraia owns the canonical model, not the solver.
- Solver adapters are translation/execution layers.
- Capability declarations should be explicit.
- Adapter representability errors should be separated from Fraia validation errors.
- Runs should preserve both resolved Fraia state and compiled solver input.
- Initial implementation should start with one solver adapter only.

---

## 16. Open questions

- Exact adapter interface
- How much compiled solver input should be normalized vs stored raw
- How to represent solver-specific approximations and limitations in a portable way
- Which solver should be the first implementation target after the Fraia core exists

---

_End of draft._
