# Fraia Package System

_Status: draft v0.1_
_Date: 2026-04-14_

_Canonical focus:_ package/extensibility model for reusable Fraia definitions and libraries.
_See also:_ `builder-graph-architecture.md`, `project-layout.md`, `engineering-output-pipeline.md`, `documentation-map.md`.

This document captures the current direction for Fraia packages, external libraries, manifests, lock metadata, and migration-friendly project structure.

---

## 1. Goals

The Fraia package system should be:

- modular
- versioned
- open-source friendly
- agent-friendly
- context-window friendly
- migration-friendly
- reproducible

Projects should mostly contain:

- project-specific instances
- project-specific overrides
- project-specific runs/results

Reusable definitions should live in external packages.

---

## 2. High-level principle

### Projects should reference libraries, not embed the world
This keeps projects:

- smaller
- cleaner
- easier for agents to inspect
- easier to migrate
- easier to reuse

It also enables community package ecosystems.

---

## 3. Package categories

Likely Fraia package families:

## 3.1 Core packages
Examples:

- math/geometry primitives
- generic simulation primitives
- core structural primitives

## 3.2 Catalog packages
Examples:

- steel material catalogs
- steel section/profile catalogs
- unit/display presets
- default property libraries

## 3.3 Archetype packages
Examples:

- simply supported beam archetypes
- portal frame archetypes
- truss family libraries
- braced bay templates

## 3.4 Rules packages
Examples:

- preliminary global steel rules
- firm-specific rule overlays
- future official code packs

## 3.5 Validation/diagnostic packages
Examples:

- stability heuristics
- topology diagnostics
- compatibility checks
- common modeling failure rules

## 3.6 Solver adapter packages
Examples:

- OpenSees adapter
- CalculiX adapter
- future Gmsh/mesh integration helpers

---

## 4. Project/package split

### Package responsibility
A package should define reusable things such as:

- primitive definitions
- archetype definitions
- catalogs
- rulesets
- validation knowledge

### Project responsibility
A project should define:

- instantiated objects
- parameter values
- project-local structure composition
- project overrides
- analysis requests
- runs/results

---

## 5. Projects can be partial and abstract

A Fraia project should not need to be a fully analyzed final design.

A project might be:

- a topology sketch
- an archetype instantiation
- a structure with no final sizing
- a structure with no loads yet
- a parametric template variant

This is important for agent workflows.

---

## 6. Suggested project structure

A likely starting project shape:

```text
project/
  fraia.project.json
  fraia.lock.json
  instance.json
  overrides/
    rules.json
    local-catalogs.json
  analysis/
    requests.json
  runs/
    <run-id>/
      snapshot.json
      results.json
      summary.json
      logs.txt
```

This is intentionally compact.

---

## 7. Suggested package concepts

Each Fraia package should probably expose:

- manifest metadata
- package version
- resource types provided
- migration information if needed
- dependency information
- concise agent-readable summaries where relevant

---

## 8. Manifest direction

A package manifest will likely need fields like:

- package id/name
- version
- package type(s)
- description
- dependencies
- exported resources
- compatibility range
- optional migration hooks

### Example sketch

```json
{
  "id": "fraia.structural-archetypes.core",
  "version": "0.1.0",
  "description": "Core structural archetypes for Fraia.",
  "dependsOn": [
    { "id": "fraia.structural-primitives.core", "range": "^0.1.0" }
  ],
  "exports": {
    "archetypes": [
      "beam.simply_supported",
      "beam.cantilever",
      "frame.portal_single_bay"
    ]
  }
}
```

---

## 9. Lock metadata

Because projects reference external packages, reproducibility requires pinned versions.

A project lock file should eventually pin:

- package ids
- exact versions
- maybe content hashes/digests
- compatible resolver information

### Example sketch

```json
{
  "packages": [
    {
      "id": "fraia.structural-primitives.core",
      "version": "0.1.0"
    },
    {
      "id": "fraia.sections.au.steel",
      "version": "0.1.0"
    }
  ]
}
```

---

## 10. Resolved snapshots vs package references

Projects should normally keep compact references to packages.

However, runs must preserve reproducibility.

Therefore each analysis/optimization run should save a frozen snapshot containing:

- fully resolved package references
- effective property values
- effective rules
- effective primitive/archetype expansions
- solver adapter version

This means package ecosystems can evolve while old runs remain reproducible.

---

## 11. Migration strategy

Migration support should exist from day 1.

### 11.1 Version every important format
Examples:

- project schema version
- package manifest version
- archetype definition version
- rules version
- run snapshot version

### 11.2 Explicit migration chain
Prefer:

- old version -> explicit migration step -> new version

Avoid:

- endlessly supporting every legacy format in every parser forever

### 11.3 Immutable runs
Historical runs should remain immutable snapshots.

### 11.4 Project migration vs package migration
We may need to distinguish:

- migrating a project instance file
- migrating a package definition
- re-locking a project to newer package versions

These are related but not identical operations.

---

## 12. Agent friendliness

Packages should be optimized for agent use, not just for machines.

### Good package traits
- concise metadata
- small focused resources
- agent-readable summaries/cards
- clear exported ids
- stable naming

### Bad package traits
- giant opaque blobs
- no summaries
- unclear dependencies
- hidden semantic meaning

---

## 13. Primitive and archetype packaging

A primitive/archetype package should ideally provide:

- definitions
- exposed parameters
- ports/interfaces
- composition rules
- validation notes
- common failure modes
- example usage

This helps both:

- contributors
- agents

---

## 14. Separation from agent tooling

Fraia should have its **own engineering package system**.

Agent skills, prompts, tools, and runtime extensions may call Fraia operations, but they do not define engineering schemas, catalogues, or package resolution. Fraia engineering packages remain separate and domain-native.

This keeps the engineering core independent.

---

## 15. Package identity and naming

Likely needs clear namespace strategy.

Examples:

- `fraia.structural-primitives.core`
- `fraia.archetypes.portal-frames`
- `fraia.sections.au.steel`
- `fraia.rules.preliminary.steel`
- `community.trusses.bridge-pack`

A stable package naming strategy will matter for:

- lockfiles
- migrations
- dependency resolution
- open-source sharing

---

## 16. Suggested first implementation scope

First package-system implementation can stay small.

Likely minimum:

- package manifest
- project manifest
- package references from project
- lock metadata
- package loading/resolution
- snapshot freezing for runs

No need yet for:

- distributed registries
- package signing
- complex semver resolution rules
- remote package indexes

Those can come later.

---

## 17. Design choices currently favored

- Libraries/packages should generally live outside projects.
- Projects should mostly contain instantiated/project-specific data.
- Projects need lock metadata for reproducibility.
- Run snapshots must freeze resolved data, not just references.
- Fraia's engineering package system remains independent of agent and runtime tooling.
- Migration/versioning should be built in from the start.

---

## 18. Open questions

- Exact package manifest schema
- Exact lockfile schema
- Whether packages are filesystem-native first, registry-based later
- How package content hashing should work
- How archetype/primitive expansion should be packaged and versioned
- How package dependencies should be resolved offline vs online

---

_End of draft._
