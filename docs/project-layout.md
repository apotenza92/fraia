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

## 2. Fraia project folder

New blank models start immediately in Fraia-managed recovery storage. Creating
a model does not open a file or folder picker. When the user first saves the
model, Save asks for a project name and parent location, then creates one
dedicated project folder on macOS, Windows, and Linux. Save As creates another
complete project folder. Fraia refuses to use an existing arbitrary folder.
This prevents Fraia from placing project files beside unrelated user files.

Every design belongs to a project, including a design created directly from the
welcome screen. In that case Fraia silently creates an `Untitled Project` in
managed recovery storage and adds the first blank design. The user can write,
import project files, and curate design references before choosing a filesystem
location. The first Save moves the complete project, including all designs,
sources, shelves, revisions, and runs, into one dedicated project folder.

Do not create an orphan-design storage mode. Do not require a project form or
folder picker before the first conversation. Show the lightweight project name
in the application shell and offer Save without blocking initial work.

Project and design names are mandatory domain fields. Each project and design
also has a stable opaque id that does not change when its name changes. A new
managed workspace uses the valid temporary names `Untitled Project` and
`Design 1` so initial work is not blocked. Before the first filesystem Save,
Fraia requires a non-temporary project name and asks the user to confirm or edit
the first design name. A newly added design requires a non-empty name through a
small inline action.

Design names must be unique within one project. Display names are not filesystem
paths. On first Save, Fraia may derive a safe suggested folder name from the
project display name, but later renaming the project or a design must not move
or rename files silently. Existing projects with missing names receive explicit
migration defaults and remain editable.

Users can inspect, copy, version, and back up every project file with normal
file tools. The app opens either the dedicated folder or its
`fraia.project.json` manifest.

## 3. Project and design scope

A Fraia project is a user-managed folder for related work. It can represent a
building, site, commission, study, or any grouping the user finds useful. Fraia
does not require a project to carry one engineering meaning. It owns information
that can be shared by more than one design:

- source drawings, images, CAD files, and BIM models
- project units, grids, levels, and coordinate systems
- site, jurisdiction, material, and project-wide design facts
- named designs

A design is one unit of engineering work at whatever scale the user chooses. It
may be one beam, the complete steel structure of a house, a warehouse, or a
larger structural system. A design owns one primary conversation, its curated
source shelf, authored model, design options, revisions, and analysis evidence.
Fraia may organise a large design internally into systems, groups, builders,
zones, and analysis submodels without requiring separate user-visible designs.

Use these terms consistently:

- **Project**: a folder and shared-resource context for related work.
- **Design**: one engineering model and conversation at a user-chosen scale.
- **Design option**: one alternative approach within a design.
- **Revision**: one accepted immutable state in a design conversation.

Keep the first implementation to one project level and one design level. Do not
add arbitrary nested design folders.

New designs created while a project is open join that project and can browse
all project files. Each design still has its own design references, conversation,
authored model, revisions, and analysis evidence. Adding a project file as one
design's reference does not add it to another design automatically.

Designs are independent by default. When one design needs another, let the user
add an accepted revision from that design as a read-only reference. Record the
source design, exact revision, coordinate transform, and optional interface
points. Do not silently co-analyse or mutate linked designs. If coupled
structural behaviour must be analysed together, create or use one design that
contains the coupled authored model.

## 4. Package layout

A good early Fraia project layout could look like:

```text
project-name/
  fraia.project.json
  planning.md
  sources/
    source-index.json
    originals/
    derived/
  designs/
    <design-id>/
      design.json
      shelf.json
      interpretations/
        index.json
        <drawing-interpretation-revision-id>.json
      workspace.sqlite
      runs/
        index.json
        design-run-sha256-<content-id>/
          manifest.json
          <checksummed attachments>
  generated/
  runs/
    <legacy-run-id>/
```

This is intentionally simple.

---

The exact migration from the current root-level model remains an implementation
detail. New code should first add typed project and design identities without
moving existing files destructively.

## 5. File responsibilities

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

## `sources/`

Project-wide imported files and deterministic derivatives. Originals are
content-addressed and immutable. Derived page images, text, vector primitives,
thumbnails, and CAD/BIM indexes retain the original content hash and exact page,
layout, layer, object, or model-view reference.

## `designs/<design-id>/shelf.json`

A small curated design-reference set. Design references link to project files;
they do not duplicate originals. A design reference may identify a PDF page
crop, drawing viewport, CAD layer selection, IFC object selection, or saved 3D
view together with scale, orientation, annotations, and confirmation state.

## `designs/<design-id>/interpretations/`

Immutable, design-scoped drawing interpretation revisions. `index.json` records
the exact current head and the known revision identities. Each revision file is
named by its content identity and records its exact parent. Saving a new
revision uses compare-and-swap against the expected parent. Parser adapters may
add unconfirmed observations only. A separate user confirmation must occur
before an observation can constrain a structural proposal.

Each observation retains its exact design reference, source hash, page or source
coordinate space, extraction method, confidence, uncertainty, and any confirmed
transform into design coordinates. Unresolved drawing conflicts remain visible
and prevent the affected observations from becoming proposal constraints.

## `generated/`
Optional generated authored/resolved artifacts.

Examples:

- instantiated archetype snapshots
- temporary resolved views
- exported model variants

## `designs/<design-id>/runs/`
Canonical immutable design-run artefacts. `index.json` lists only complete,
validated `fraia.design-run.v1` manifests. Each content-addressed run directory
contains `manifest.json` and only the checksummed attachments declared by that
manifest. Run publication does not change authored state. The application
service and command-line interface inspect this same index.

Root-level or noncanonical run directories are legacy content. Fraia preserves
and inspects that content read-only. It does not silently rewrite it or treat it
as the current canonical run.

---

## 6. Why planning markdown is near the root

Planning is not a side note.

It is part of the durable project context and should be easy to find and review.

This supports:

- agent continuity
- human collaboration
- reduced dependence on transient chat state

---

## 7. Why runs are isolated

Runs should be isolated from authored state because they are:

- immutable
- provenance-bearing
- reproducibility-focused

They should not overwrite the authored project directly.

---

## 8. Future expansion direction

As Fraia grows, the layout may expand into a more modular package-aware structure such as:

```text
project-name/
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

## 9. Design choices currently favored

- Keep early project layout small and understandable.
- Create one dedicated cross-platform folder and never populate an existing arbitrary folder.
- Put planning markdown near the root.
- Keep immutable runs separate from authored state.
- Allow future growth into a more modular package-aware structure.
- Treat a project as a user-managed folder and shared-source context.
- Let one design be as small or large as the user wants, including the complete
  structure of a building.
- Keep source originals project-wide and keep design shelves as lightweight,
  provenance-bearing references.

---

_End of draft._
