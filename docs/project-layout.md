# Fraia Project Layout

_Status: draft v0.1_
_Date: 2026-04-14_

_Canonical focus:_ Fraia project filesystem layout and artifact responsibilities.
_See also:_ `resolution-and-runs.md`, `engineering-output-pipeline.md`, `builder-graph-architecture.md`, `documentation-map.md`.

This document sketches the preferred Fraia project layout for planning files, authored project data, generated outputs, and immutable run artifacts.

---

## 1. Goals

A Fraia project layout should be:

- human-readable
- agent-friendly
- versionable in git
- modular
- easy to migrate
- easy to inspect from the CLI

---

## 2. Preferred early layout

A good early Fraia project layout could look like:

```text
project/
  fraia.project.json
  planning.md
  generated/
  runs/
    <run-id>/
      options.json
      summary.md
      snapshot.json
      results.json
      diagnostics.json
      logs.txt
```

This is intentionally simple.

---

## 3. File responsibilities

## `fraia.project.json`
Structured project state.

Candidate contents:

- schema version
- intent
- requirements
- builder graph metadata when present
- legacy builder-instance migration compatibility when loading older projects
- authored structural model snapshot
- package references later
- analysis defaults
- search permissions

## `planning.md`
Durable human/agent planning context.

Candidate contents:

- user brief
- assumptions
- hard constraints
- soft preferences
- open questions
- next steps

## `generated/`
Optional generated authored/resolved artifacts.

Examples:

- instantiated archetype snapshots
- temporary resolved views
- exported model variants

## `runs/<run-id>/`
Immutable run artifacts.

Examples:

- resolved snapshot
- solver input
- results
- diagnostics
- summary report

---

## 4. Why planning markdown is near the root

Planning is not a side note.

It is part of the durable project context and should be easy to find and review.

This supports:

- agent continuity
- human collaboration
- reduced dependence on transient chat state

---

## 5. Why runs are isolated

Runs should be isolated from authored state because they are:

- immutable
- provenance-bearing
- reproducibility-focused

They should not overwrite the authored project directly.

---

## 6. Future expansion direction

As Fraia grows, the layout may expand into a more modular package-aware structure such as:

```text
project/
  fraia.project.json
  fraia.lock.json
  planning/
    planning.md
    assumptions.md
    decisions.md
  authored/
    instance.json
    overrides.json
  analysis/
    requests.json
  generated/
  runs/
```

But the earliest usable version can remain simpler.

---

## 7. Design choices currently favored

- Keep early project layout small and understandable.
- Put planning markdown near the root.
- Keep immutable runs separate from authored state.
- Allow future growth into a more modular package-aware structure.

---

_End of draft._
