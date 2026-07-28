# Fraia Tool Contracts

_Status: draft v0.1_
_Date: 2026-04-13_

_Canonical focus:_ operational/tool/API surface shape.
_See also:_ `resolution-and-runs.md`, `engineering-output-pipeline.md`, `project-layout.md`, `documentation-map.md`.

This document captures the current direction for Fraia tool and service contracts, including agent access.

---

## 1. Purpose

Fraia needs a deterministic operational surface for:

- CLI usage
- local service/API usage
- backend orchestration
- agent tool invocation
- testing and automation

This surface should be stable, structured, and independent from any one UI.

---

## 2. Core principle

### The agent should call Fraia tools, not improvise engineering state
The LLM/agent should operate through well-defined Fraia contracts.

This means:

- project changes should go through Fraia operations
- validation should go through Fraia validators
- analysis should go through Fraia run operations
- results should come back in normalized Fraia forms

This preserves determinism and reproducibility.

---

## 3. One core, many frontends

The same Fraia operations should be callable from:

- CLI
- app backend
- local API/service
- agent-facing tools
- tests

This reduces drift between interaction modes.

---

## 4. Recommended operation families

Likely early tool families:

## 4.1 Project/package operations
- create/open project
- inspect project summary
- migrate project
- resolve package references
- inspect lock/package state

## 4.2 Planning operations
- create/update planning markdown
- summarize brief
- list open questions
- record assumptions/decisions

## 4.3 Library operations
- list/search packages
- list/search primitives
- list/search archetypes
- inspect primitive/archetype cards

## 4.4 Modeling operations
- instantiate archetype
- add/connect primitive
- set parameter
- bind support/release/connection
- classify object/assign semantic roles
- create/edit nodes, members, plates, supports, releases, and loads incrementally
- create very small structural systems directly (e.g. simply supported beam)
- preview how authored loads realize into reduced solver-facing loads where applicable

## 4.5 Resolution/validation operations
- resolve project
- validate authoring state
- validate resolved state
- explain diagnostics

## 4.6 Run operations
- run validation
- run analysis
- run optimization iteration
- inspect run status
- inspect run summary

## 4.7 Results operations
- query displacements
- query forces/reactions
- list governing diagnostics/checks
- summarize results at a high level

---

## 5. Tool outputs should be machine-readable first

Every Fraia tool should ideally return structured output.

Human-readable summaries can be layered on top.

This is especially important for:

- agent workflows
- automation
- UI integration
- testing

---

## 6. Abstraction discipline for tools

Tool contracts should reflect Fraia’s layering philosophy.

Preferred order for agent use:

1. planning/brief tools
2. archetype/semantic tools
3. structural primitive tools
4. resolved model inspection tools
5. math/frame inspection tools only if required

This keeps agents from dropping too quickly into low-level details.

---

## 7. Suggested early tool examples

Examples of Fraia-native tools:

- `project_summary`
- `planning_update`
- `planning_questions`
- `library_search`
- `archetype_inspect`
- `archetype_instantiate`
- `project_resolve`
- `project_validate`
- `analysis_run`
- `run_summary`
- `diagnostic_explain`
- `results_query`
- `beam_create_simple`
- `beam_solve`
- `beam_size`
- `model_add_node`
- `model_add_member`
- `model_add_plate`
- `load_apply`
- `load_realization_preview`
- `selection_parameterize`
- `builder_reconcile_edit`
- `model_reparameterize`
- `subsystem_reparameterize`

These can later be exposed through CLI, service endpoints, and agent-facing tools.

### 7.2 Re-parameterization tools should be proposal-oriented

When an agent asks Fraia to re-parameterize a subsystem or whole model, the operation should normally return a proposal describing things like:
- which existing builder nodes still match the current authored state
- which nodes are diverged/stale
- which manual objects could be attached to a project-local builder
- which objects should remain manual
- what new parameterized structure Fraia recommends

The user can then approve adoption of that proposal into the active builder graph.

### 7.1 Important agent behavior rule

These tools should support iterative Fraia behavior, not only one-shot generation.

A useful agent workflow is often:
1. create or refine a small model
2. validate it
3. solve it
4. inspect the result
5. change one thing
6. repeat

This is especially important for early engineering tasks like simply supported beam sizing and later for larger structural-system loops.

---

## 8. Agent-facing tool contracts

Fraia should expose deterministic, provider-independent operations to any agent runtime.

Agent-facing tools should be thin wrappers around Fraia core operations, not places where engineering logic is reinvented.

Examples:

- `fraia_validate` calls the Fraia validation operation
- `fraia_resolve` calls the Fraia resolution operation
- `fraia_analyze` calls the Fraia run operation

This keeps Fraia independent of a particular model provider or orchestration runtime.

### 8.1 Current AI runtime boundary

The Electron main process embeds Pi through the pinned `@earendil-works/pi-ai` and `@earendil-works/pi-agent-core` packages. Fraia registers only Pi's reviewed `openai-codex` provider; Pi owns its ChatGPT authentication, static Luna model definition, reasoning capability, inference transport, and schema-validated tool loop. Fraia owns prompts, project/chat state, response schemas, action filtering, Rust validation, and committed provenance. The broader Pi provider boundary remains internal so that a later Fraia-managed service can be evaluated without coupling engineering state to one inference transport.

Electron starts an authenticated loopback service before `fraia-appd` and passes a random launch-scoped URL and bearer token through the sidecar environment. Rust is the only caller of the turn endpoints. The renderer can manage provider connections through app-scoped IPC but cannot access Pi or credentials directly.

Every turn uses a new low-level in-memory Pi agent with no coding-agent session, extensions, skills, prompt templates, context discovery, project settings, filesystem tools, shell tools, or editing tools. Its only tool is Fraia's terminating `submit_fraia_response`, parameterised by the exact response schema for that workflow. Rust deserialises the returned arguments into its own types and applies the existing action filters before project state can change.

Fraia 0.0.2 exposes one public connection: **Sign in with ChatGPT** through Pi's `openai-codex` OAuth flow. It does not expose provider search, API-key entry, model selection, or reasoning selection. The ChatGPT authorization is stored as an Electron `safeStorage` encrypted blob beneath Electron user data. If operating-system encryption is unavailable, Fraia refuses to persist the connection rather than falling back to plaintext. Projects, chat records, logs, and diagnostics never contain provider credentials. Pi sessions and duplicate Pi transcripts are not persisted.

Every 0.0.2 workflow is locked to `{ providerId: "openai-codex", modelId: "gpt-5.6-luna", reasoningEffort: "low" }`. Existing model-only and per-surface settings still deserialize for compatibility, then project migration replaces them with that reviewed tuple. If Luna is absent or unavailable, the next turn is blocked; Fraia never silently switches model, provider, or reasoning effort. Committed assistant messages and AI-derived design-option batches retain the exact provider, model, reasoning effort, and catalogue timestamp provenance.

---

## 9. Suggested response shape

A tool response will likely need:

- success/failure status
- machine-readable payload
- diagnostics
- optional human summary
- references to affected objects/runs

### Example sketch

```json
{
  "ok": true,
  "summary": "Project resolved successfully.",
  "data": {
    "resolvedModelRef": "runs/tmp-resolve-001/snapshot.json"
  },
  "diagnostics": []
}
```

---

## 10. Long-running operations

Some Fraia operations may be long-running:

- large analyses
- optimization loops
- package migrations on big projects

These should likely support:

- progress events
- run ids
- resumable inspection
- intermediate summaries

This is especially useful for app integration and agent orchestration.

---

## 11. Tool contracts should not expose too much low-level detail by default

For most workflows, agents should see summaries and references first.

Only expose very detailed internal payloads when:

- debugging
- diagnostics
- solver inspection
- advanced control

This is another important abstraction guard.

---

## 12. Early CLI alignment

Likely Fraia CLI directions:

- `fraia project ...`
- `fraia planning ...`
- `fraia library ...`
- `fraia model ...`
- `fraia resolve ...`
- `fraia validate ...`
- `fraia run ...`
- `fraia results ...`

The exact command structure is still open, but these categories map well to the planned tool families.

---

## 13. Design choices currently favored

- Fraia tools should be deterministic and stable.
- The same core operations should back CLI, services, and agent tools.
- Agents should work through Fraia tools rather than mutate engineering state ad hoc.
- Tool outputs should be machine-readable with optional summaries.
- Tools should respect Fraia’s abstraction-guard philosophy.

---

## 14. Open questions

- Exact CLI/service naming and grouping
- Sync vs async run semantics for larger operations
- How granular modeling/edit tools should be in early versions
- How much direct patch/edit capability Fraia should expose versus higher-level commands

---

_End of draft._
