# Fraia Resolution and Runs

_Status: draft v0.1_
_Date: 2026-04-14_

_Canonical focus:_ authored vs resolved vs frozen-run separation and deterministic resolution pipeline.
_See also:_ `project-layout.md`, `engineering-output-pipeline.md`, `builder-graph-architecture.md`, `documentation-map.md`.

This document defines the current direction for how Fraia moves from authored project data to resolved engineering models and immutable run snapshots.

---

## 1. Purpose

Fraia needs a clean separation between:

- authored project data
- resolved runtime data
- immutable analysis/optimization run snapshots

This separation is essential for:

- reproducibility
- migration safety
- package/library reuse
- solver independence
- agent friendliness
- traceable optimization loops

---

## 2. The three core representations

## 2.1 Authoring representation
This is the project-facing representation.

It should be:

- compact
- human-readable
- agent-readable
- library/package-referencing
- suitable for git and long-term editing

Typical contents:

- project brief/planning markdown
- project manifest
- instance/archetype usage
- project-local overrides
- analysis requests
- package references

This representation should avoid embedding every resolved property inline if those values come from shared packages.

---

## 2.2 Resolved runtime representation
This is the fully expanded and normalized engineering model used internally before execution.

It should include:

- package references resolved
- primitive/archetype expansions resolved
- placements and local frames resolved
- effective parameters computed
- defaults applied
- inherited rules resolved
- active analysis requests attached
- units normalized

This representation may live primarily in memory, though it can also be serialized for inspection/debugging.

---

## 2.3 Frozen run snapshot
This is the immutable record of a specific run.

A run may be:

- validation run
- analysis run
- optimization iteration
- migration check run
- comparison run

A frozen run snapshot should include:

- exact resolved model used
- effective rules used
- exact package versions
- exact solver adapter and settings
- exact outputs/results/logs
- provenance metadata

This snapshot must not change retroactively.

---

## 3. Why this separation matters

Without this separation, Fraia would risk:

- non-reproducible results when libraries change
- hard-to-debug agent behavior
- migrations breaking historical runs
- analysis outputs depending on hidden mutable state

The authored model should be allowed to evolve.
The run snapshot should remain frozen.

### Important workflow rule
Generated or optimised options should not silently overwrite the authored project model.
A candidate option should become authored state only through an explicit adoption step.

Design-option analysis follows the same rule. Analysing a generated design option creates immutable run artefacts linked to its stable batch-scoped revision identity and realised option snapshot; it does not mutate the Base Model or rewrite the option. The visible journey is Base Model → Design Options → Analysis & Comparison; Engineering Evidence is a contextual drill-down rather than another stage. The active solver/check plan is [`../plans/design-option-analysis.md`](../plans/design-option-analysis.md).

Comparison and development decisions are project state rather than solver state:

- an option batch records the Base Model fingerprint it was generated from
- inclusion controls which active revisions participate in analysis and comparison
- a comparison run records the exact included revision identities, objective, recommendation, explanation, limitations, and immutable per-revision run ids
- a development path references one analysed revision identity; multiple paths can coexist inside Analysis & Comparison without replacing the Base Model or deleting alternatives
- a Base Model fingerprint change makes the active batch outdated and prevents new analysis or development until regeneration

---

## 4. Resolution pipeline

The Fraia core will likely need a deterministic resolution pipeline.

A likely sequence:

1. load project manifest and instance data
2. load package/library references
3. lock or verify package versions
4. expand semantic archetypes into lower-level primitives
5. resolve placements, frames, ports, and connectivity
6. apply project-local overrides
7. apply active rulesets
8. normalize units and numeric forms
9. validate structural completeness/readiness
10. produce resolved model

This resolved model can then be handed to:

- validator
- solver adapter
- optimizer
- diagnostics engine

---

## 5. Resolution should be explicit, not hidden

Fraia should treat resolution as a first-class operation.

Potential user/agent operations later:

- resolve project
- inspect resolved model
- diff authored vs resolved
- inspect why a parameter/rule resolved the way it did

This improves:

- trust
- debuggability
- agent usefulness
- clean separation between generated candidate options and adopted authored state

---

## 6. What gets resolved

Examples of things that should become explicit in the resolved representation:

- final material properties after catalog lookup
- final section properties after catalog lookup
- instantiated child primitives inside archetypes
- final frame/placement tree
- final local-axis derivation inputs
- final support/release/connection assignments
- final rule/profile values after inheritance and overrides
- final analysis request settings

---

## 7. Authoring data should stay compact

Projects should remain concise by relying on references and semantic structures.

### Good authored project behavior
- reference package archetype ids
- provide parameter values
- provide project-specific constraints and overrides

### Bad authored project behavior
- eagerly denormalize everything into one giant blob
- duplicate full section/material/rule definitions from libraries everywhere
- hide resolved values in implicit app state

---

## 8. Frozen run contents

A run directory will likely need at least:

- run manifest/metadata
- resolved model snapshot
- solver input or normalized analysis payload
- results
- diagnostics/warnings
- logs
- summary

### Possible layout

```text
runs/
  2026-04-13T23-59-00Z/
    run.json
    snapshot.json
    solver-input.json
    results.json
    diagnostics.json
    summary.md
    logs.txt
```

---

## 9. Provenance model

Each run should capture provenance such as:

- run id
- timestamp
- triggering actor (user, agent, optimizer)
- source project version
- source package lock data
- resolved schema version
- ruleset version
- solver adapter/version
- parent run if iterative
- notes/reason for run

This will be especially important for optimization and autonomous design loops.

---

## 10. Optimization iterations as runs

Optimization should not be treated as a special hidden side channel.

Each iteration or checkpoint should be representable as a run/provenance object.

This allows Fraia to track:

- what changed
- why it changed
- whether the objective improved
- which constraints passed/failed
- which parent state it came from

This is essential for agent-led design loops.

---

## 11. Relation to migrations

When authored projects evolve through schema migrations:

- the project files may change shape
- package references may be updated
- lockfiles may be refreshed

But historical runs should still remain intact because they capture frozen resolved state.

This is one of the strongest reasons to keep run snapshots immutable.

---

## 12. Relation to packages and locks

A project may reference external packages.

A resolved model should already have those references expanded.

A run snapshot should additionally pin or copy enough package metadata to remain reproducible even if:

- a registry changes
- a package is updated
- a local library moves

The exact balance between copying package content and pinning versions remains an open design question.

---

## 13. Agent interaction with resolution

Agents should normally work on authored abstractions first.

They should only inspect resolved data when necessary.

Examples of when resolved inspection is useful:

- explain why a rule value was applied
- diagnose instability due to expanded support/release patterns
- inspect effective local axis definitions
- inspect exact solver-facing model state

This aligns with Fraia's abstraction guard principle.

---

## 14. Validation before and after resolution

There will likely be at least two validation stages.

## 14.1 Authoring validation
Checks things like:

- missing package refs
- invalid parameter values
- incompatible ports
- unresolved required fields

## 14.2 Resolved validation
Checks things like:

- ambiguous local axes
- disconnected graphs
- invalid support/release patterns
- analysis readiness
- solver capability mismatches

This split is useful and likely necessary.

---

## 15. Human-readable summaries

Each run should also generate a compact human/agent-readable summary.

This may include:

- what was run
- key assumptions
- package/rules versions
- pass/fail overview
- governing issues
- next recommended actions

This helps agents and users work without reading full raw snapshot data every time.

---

## 16. Suggested early Fraia commands/services

Potential core operations later:

- `fraia resolve`
- `fraia validate`
- `fraia run validate`
- `fraia run analyze`
- `fraia run optimize`
- `fraia run inspect`

This suggests that “resolve” should be a first-class engineering operation, not just an internal hidden step.

---

## 17. Design choices currently favored

- Authoring, resolved, and run-snapshot representations should be distinct.
- Resolution should be deterministic and inspectable.
- Historical runs should be immutable.
- Optimization iterations should be tracked as runs/provenance objects.
- Projects should remain compact and reference external packages where possible.
- Agents should stay at authored abstractions by default and descend only when needed.

---

## 18. Open questions

- Exact resolved model schema
- Exact run manifest schema
- How much package content must be copied into a frozen run
- Whether solver-input snapshots should always be preserved separately
- How much diffing/comparison support to provide between authored, resolved, and run states

---

_End of draft._
