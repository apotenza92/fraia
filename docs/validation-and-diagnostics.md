# Fraia Validation and Diagnostics

_Status: draft v0.1_
_Date: 2026-04-13_

_Canonical focus:_ validation layers, diagnostic semantics, and upward mapping.
_See also:_ `resolution-and-runs.md`, `engineering-output-pipeline.md`, `builder-graph-architecture.md`, `documentation-map.md`.

This document captures the current direction for Fraia validation, readiness checking, diagnostics, and failure explanation.

---

## 1. Purpose

Validation in Fraia is not only about checking whether a solver can run.

It must also support:

- authoring correctness
- package/reference correctness
- frame/orientation correctness
- connectivity correctness
- analysis readiness
- preliminary safety/fitness checks
- agent-friendly explanations

Diagnostics are a first-class feature, not a side effect.

---

## 2. Core principle

### Diagnose at the highest meaningful layer first
If a low-level failure occurs, Fraia should try to map it back upward.

For example, instead of only reporting:

- node 18 invalid DOF pattern

prefer also reporting:

- left base support in portal frame archetype creates a mechanism

This supports both:

- humans
- agents

and aligns with Fraia’s abstraction guard principle.

---

## 3. Validation layers

Fraia will likely need multiple validation layers.

## 3.1 Project/authoring validation
Checks project-facing data before full resolution.

Examples:

- missing package references
- invalid parameter ranges
- missing required archetype parameters
- invalid project manifest structure
- unresolved planning assumptions

## 3.2 Resolution validation
Checks after expansion and normalization.

Examples:

- unresolved child ports
- ambiguous orientation rules
- invalid frame inheritance
- unsupported connection combinations

## 3.3 Analysis readiness validation
Checks whether the resolved model can meaningfully proceed to analysis.

Examples:

- missing supports
- disconnected components
- unassigned material/section properties
- undefined loads when required
- unsupported analysis request

## 3.4 Preliminary engineering fitness checks
Checks post-analysis or analysis-derived constraints.

Examples:

- excessive utilization
- excessive deflection/drift
- low buckling factor
- slenderness issues
- obvious instability indicators

---

## 4. Types of diagnostics

A diagnostic may be one of:

- error
- warning
- info
- note/recommendation

Each diagnostic should ideally include:

- severity
- category
- affected object(s)
- highest meaningful source object
- message
- machine-readable code
- suggested next action if available

---

## 5. Important diagnostic categories

Likely early categories:

- package/reference
- parameters
- frames/orientation
- topology/connectivity
- supports/releases/connections
- analysis capability mismatch
- solver realization
- preliminary strength/serviceability/stability
- migration/version compatibility

---

## 6. Frame and orientation diagnostics

This is likely one of the most important early categories.

Examples:

- reference vector parallel to member tangent
- shell/surface normal undefined or inconsistent
- invalid frame basis
- unresolved frame reference
- ambiguous local axis derivation

These should be expressed clearly in terms agents can act on.

---

## 7. Connectivity diagnostics

Examples:

- disconnected subgraph
- incompatible port types
- unsupported connection between two primitive families
- unconnected required port
- release/support pattern creates mechanism
- duplicate coincident nodes not merged or intentionally separated

These are essential for agent-guided model correction.

---

## 8. Analysis capability diagnostics

The validator should know whether a requested analysis can be supported by the chosen realization and solver adapter.

Examples:

- shell region requested but solver adapter supports only line/frame members
- requested buckling analysis unsupported by chosen adapter
- connection semantics cannot be realized in selected solver

This is important for a multi-solver future.

---

## 9. Planning/brief diagnostics

Because Fraia starts from a planning layer, diagnostics may also exist before formal modeling begins.

Examples:

- building type unclear
- design stage not stated
- dimensions/spans missing
- material system unspecified where needed for next step
- code/jurisdiction unknown for requested compliance task

These are not solver errors, but they are valid planning diagnostics.

---

## 10. Preliminary engineering checks

Before formal code modules are implemented, Fraia should still support conservative preliminary checks.

These might include:

- stress/utilization bounds
- deflection limits
- drift limits
- elastic buckling factor thresholds
- slenderness checks
- equilibrium checks
- convergence/pass-fail indicators

These should be clearly labeled as preliminary/non-code when appropriate.

---

## 11. Suggested diagnostic object shape

An eventual diagnostic object may need fields like:

- `id`
- `severity`
- `category`
- `code`
- `message`
- `details`
- `objectRefs`
- `sourceLayer`
- `mappedFrom`
- `suggestedActions`

### Example sketch

```json
{
  "severity": "error",
  "category": "frames-orientation",
  "code": "curve-member.orientation.parallel-reference-vector",
  "message": "Local axis for member m12 is undefined because the reference vector is parallel to the member axis.",
  "objectRefs": ["member:m12"],
  "suggestedActions": [
    "Provide a non-parallel reference vector.",
    "Use a different orientation rule."
  ]
}
```

---

## 12. Upward mapping

Low-level diagnostics should ideally map to higher-level source objects.

Examples:

- solver node -> structural primitive -> archetype instance -> planning decision
- support primitive -> semantic support scheme -> building system choice

This lets Fraia explain not only what failed, but where that failure came from conceptually.

---

## 13. Agent workflow implications

Agents should be able to ask questions like:

- why won’t this run?
- what is unstable?
- what assumptions are missing?
- what is the minimal change needed to proceed?
- what high-level object caused this low-level issue?

This means diagnostics must be structured, queryable, and not only plain text blobs.

---

## 14. Validator knowledge packages

Validation should likely be extensible through Fraia packages.

Possible package families:

- core topology diagnostics
- structural stability diagnostics
- support/release compatibility rules
- preliminary steel heuristics
- solver capability validators

This fits Fraia’s modular package philosophy.

---

## 15. Suggested early implementation scope

Good early validation scope:

- invalid package references
- invalid parameter values
- unresolved required ports
- invalid frame/orientation definitions
- disconnected structural graph
- missing supports/materials/sections
- simple mechanism/stability heuristics
- solver capability mismatch

This would already provide major value.

---

## 16. Design choices currently favored

- Validation should exist at multiple layers.
- Diagnostics should be structured and machine-readable.
- Fraia should explain failures at the highest meaningful abstraction first.
- Planning-layer missing information is also a valid diagnostic category.
- Preliminary engineering checks should exist before full code modules.
- Validation/diagnostic logic should be extensible through packages.

---

## 17. Open questions

- Exact diagnostic schema
- How rich suggested actions should be
- How much solver-native diagnostic data to preserve vs normalize
- How aggressive Fraia should be in auto-fixing vs only reporting problems

---

_End of draft._
