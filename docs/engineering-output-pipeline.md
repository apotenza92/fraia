# Fraia Engineering Output Pipeline

_Status: draft v0.1_
_Date: 2026-04-14_

This document defines the downstream Fraia engineering data pipeline from authored structure through analysis, design-action extraction, checks, and export renderers.

It is intended to be the canonical reference for Fraia's Unix-like, pipe-friendly output architecture.

---

## 1. Why this exists

Fraia should not stop at:
- modeling
- viewing
- analysis

It should eventually support practical engineering outputs such as:
- member schedules
- reaction summaries
- worksheet-style check packets
- CSV exports
- XLSX/Excel exports
- report sections
- detail-family seed outputs
- CAD/detailing parameter exports later

To support that without architectural churn, Fraia needs a stable downstream data pipeline.

---

## 2. Core principle

### Outputs are renderers of structured engineering data

Excel, CSV, reports, and CAD/detail outputs should not become the source of truth.

The source of truth should remain:
- project data
- builder graph data
- authored structural model
- resolved/realized analysis model
- analysis results
- normalized design-action/check data

Outputs should be materializations of that data.

This preserves:
- reproducibility
- provenance
- scriptability
- testability
- compatibility with Fraia's Unix-like philosophy

---

## 3. Pipeline overview

A recommended Fraia downstream pipeline is:

1. intent / planning context
2. builder graph
3. authored structural model
4. realization / solver-ready model
5. analysis results
6. design-action extraction
7. check input generation
8. check execution / evaluation
9. output rendering

Expressed another way:

`builder_graph -> structural_model -> realization -> solve_result -> design_actions -> check_inputs -> check_results -> exports`

Each stage should have:
- explicit inputs
- explicit outputs
- stable typed structures
- persistable artifacts where useful

---

## 4. Canonical Fraia layers in the downstream path

## 4.1 Builder graph
Compact concept/configuration model.

Examples:
- portal frame subsystem
- hall root concept
- braced bay subsystem

Builder graph should remain:
- concept-level
- parametric
- provenance-bearing
- not solver-specific

## 4.2 Authored structural model
Project-facing structural primitives.

Examples:
- nodes
- members
- plates
- supports
- loads
- releases

This remains the main engineering object model Fraia renders, validates, and edits.

## 4.3 Realization
Derived solver-ready representation.

Examples:
- frame2d model
- future shell/solid realizations
- reduced submodels

Realization is:
- derived
- disposable
- replaceable
- explicitly validated

## 4.4 Analysis results
Normalized solver outputs.

Examples:
- displacements
- reactions
- member forces
- combination results
- utilization-like solver-side metrics

These are still not the same thing as practical worksheet inputs.

## 4.5 Design actions
Engineering-meaningful extracted demands.

Examples:
- governing member moment
- governing member shear
- governing axial force
- support reaction envelope
- story drift summary
- connection demand summary
- serviceability envelope

This is the key bridge between raw solver outputs and practical engineering checks.

## 4.6 Check inputs
Structured worksheet-style data packets.

Examples:
- steel beam check input
- steel column check input
- reaction/base check input
- drift check input
- connection-family demand input

These should be explicit and inspectable.

## 4.7 Check results
Structured pass/fail and utilization outputs.

Examples:
- utilization
- governing case
- pass/fail
- assumption notes
- conservative default flags
- future code clause references

## 4.8 Output renderers
Adapters that render Fraia data into external forms.

Examples:
- JSON exports
- CSV schedules
- XLSX workbooks
- markdown summaries
- engineering reports
- detail-family seed exports
- CAD/detail parameter outputs later

Current MVP note:
- Fraia now has shared `fraia-core` markdown/CSV renderer helpers for beam sizing and validation/check exports
- both the CLI and the desktop can use the same renderer layer instead of maintaining divergent output formatting

---

## 5. Unix-like Fraia philosophy applied here

The downstream engineering pipeline should be:
- file-based where practical
- inspectable
- scriptable
- composable
- provenance-bearing
- renderer-agnostic

A Fraia pipeline stage should ideally behave like a good Unix tool:
- clear input
- clear output
- minimal hidden side effects
- reusable in larger workflows

This does not mean every tiny internal function needs its own artifact.
It means major architectural boundaries should have stable representations.

---

## 6. Suggested stable data contracts

The following contracts are worth protecting early.

## 6.1 `StructuralModel`
Canonical authored structural primitives.

## 6.2 `Frame2DRealization` and future realization models
Canonical derived analysis-ready representations.

## 6.3 `SolveResult`
Canonical normalized analysis result set.

## 6.4 `DesignAction`
Canonical engineering-demand abstraction.

Candidate categories:
- member force envelope
- support reaction envelope
- drift envelope
- deflection envelope
- connection demand envelope

## 6.5 `CheckSubject`
The thing Fraia is checking.

Examples:
- member flexure subject
- column compression subject
- drift subject
- support/base subject
- connection demand subject

## 6.6 `CheckInput`
The worksheet-style input packet to a checking routine.

## 6.7 `CheckResult`
The structured result packet returned by a checking routine.

## 6.8 `ExportArtifact`
A structured description of generated outputs.

Examples:
- workbook file
- CSV file
- markdown report
- CAD/detail seed package

---

## 7. Design actions are the critical bridge

Fraia should not jump directly from solver result arrays to Excel rows.

A translation layer is required.

### Solver-style result examples
- local end forces
- nodal displacements
- element stress values
- combination-by-combination numbers

### Engineer-style worksheet examples
- governing moment
- governing shear
- governing compression/tension
- effective length assumption
- unbraced length
- controlling combination
- span/length used in the check
- section and material references

The design-action layer should perform this translation in a normalized Fraia-native way.

---

## 8. Check system philosophy

Checks should be treated as explicit Fraia operations, not ad hoc spreadsheet formulas hidden in exported workbooks.

### Near-term Fraia check behavior
Initially Fraia can support:
- conservative default checks
- global serviceability checks
- simple member demand summaries
- reaction summaries
- preliminary connection-demand warnings

### Later Fraia check behavior
Later Fraia can support:
- code-specific steel checks
- regional code packs
- firm-specific overlays
- connection-family checks
- submodel/detail checks

This suggests a pluggable check-engine architecture later, but the upstream contracts can be stabilised now.

### 8.1 Design-option analysis plan

The active implementation plan for analysing generated design options is:

- [`../plans/design-option-analysis.md`](../plans/design-option-analysis.md)

That plan should be treated as the operational roadmap for moving from realised design-option assumptions and catalogue section candidates to:

- option-specific solver runs;
- per-candidate member actions and reactions;
- conservative preliminary stress checks;
- mass versus utilisation comparison tables;
- immutable design-option analysis artefacts.

Until that plan is implemented, section-shape comparisons are catalogue/mass comparisons, not proof that a section supports the realised loads.

---

## 9. Excel and spreadsheet outputs

Fraia should support spreadsheet-style outputs because that reflects common engineering practice.

However:
- Fraia should compute the structured inputs and results first
- workbook generation should render those Fraia-native packets
- workbook files should not be the authoritative calculation core

A useful Fraia workbook later might include:
- project summary
- assumptions
- member schedule
- governing member actions
- support reactions
- drift/deflection summaries
- preliminary member checks
- connection-demand summaries
- references back to governing combinations and structural object ids

---

## 10. CAD/detailing outputs later

The same downstream pipeline can eventually support detail-related outputs.

Likely future flow:
- structural model
- design actions / connection demands
- detail family selection
- detail parameter packets
- CAD/detail renderer

That means CAD/detailing should be a downstream consumer of normalized Fraia data, not a separate hidden world.

---

## 11. Provenance requirements

Every downstream artifact should ideally retain references back to its sources.

Examples:
- builder node id
- structural object refs
- realization object refs
- result combination ids
- governing action ids
- check subject ids
- export artifact ids

This supports:
- debugging
- report traceability
- regeneration
- auditability
- agent explainability

---

## 12. Suggested artifact strategy

Not every stage needs to be written to disk every time, but Fraia should support persisted artifacts for major stages when useful.

Examples later:
- `runs/<run-id>/realization.json`
- `runs/<run-id>/results.json`
- `runs/<run-id>/design-actions.json`
- `runs/<run-id>/checks.json`
- `runs/<run-id>/exports/`
  - `member-checks.csv`
  - `reactions.csv`
  - `summary.md`
  - `workbook.xlsx`

This aligns well with Fraia's explicit run-artifact direction.

---

## 13. Open-source and package implications

This pipeline also creates good extension seams.

Possible future package categories:
- archetype packages
- validation packages
- check packages
- report/export packages
- detail-family packages

As long as they consume and produce Fraia-native contracts, they can remain modular and composable.

---

## 14. Immediate implementation direction

Near-term Fraia implementation should move toward:

1. define normalized design-action structs
2. define normalized check-input/check-result structs
3. extract member and support design actions from current frame results
4. generate JSON/CSV-friendly worksheet-style packets
5. later add XLSX export and richer report generation

This gives Fraia a practical path toward engineer-friendly outputs without corrupting the core architecture.

---

## 15. Relationship to other docs

This document complements:
- `engineering-core.md` for top-level philosophy
- `resolution-and-runs.md` for authored/resolved/run separation
- `builder-graph-architecture.md` for concept/configuration modeling
- `connections-and-detailing-strategy.md` for local detailing escalation
- `validation-and-diagnostics.md` for diagnostics semantics
- `tool-contracts.md` for operational/API surface design

---

## 16. Current design decision

Fraia should treat Excel, reports, and future CAD/detail outputs as downstream renderers of structured engineering data, not as alternative sources of engineering truth.

That preserves Fraia's primitive-first, builder-graph-aware, Unix-like pipeline architecture.

---

_End of draft._
