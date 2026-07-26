---
title: Steel Design Action and Check-Input Separation
status: compiled
trust_level: compiled
domain: materials
applies_to:
  - steel design/check workflow explanations
  - Fraia agent guidance
  - downstream check packet design
not_applicable_to:
  - final steel code formulas
  - check engine implementation
  - jurisdiction-specific design modules
jurisdiction_or_standard_context: Fraia architecture guidance plus professional steel source context; not a code check
last_compiled: 2026-05-07
source_count: 3
citation_policy: required
owner: agent-maintained
---

# Steel Design Action and Check-Input Separation

## Summary

Steel checking in Fraia should not treat solver output as a finished design result. The workflow needs distinct artifacts:

- analysis results: normalized solver/run outputs
- design actions: curated force/deformation/action values extracted for a design purpose
- check inputs: design context required to evaluate a limit state
- check results: the output of a check module

This separation protects Fraia's authored/resolved/run architecture and prevents agents from claiming steel adequacy from raw force values alone.

## Scope / non-scope

This page covers concept-level steel design-action/check-input separation for Fraia agents.

It does not define final schemas, code formulas, capacity checks, check engines, or jurisdiction-specific steel design modules.

## Key concepts

### Fraia has a downstream pipeline

The canonical Fraia downstream path is `builder_graph -> structural_model -> realization -> solve_result -> design_actions -> check_inputs -> check_results -> exports`. Each stage should have explicit inputs, outputs, typed structures, and persistable artifacts where useful. [S1]

Steel checks should fit this pipeline rather than bypass it.

### Analysis results are not design results

Analysis results are normalized solver outputs tied to a resolved model and run. A bending moment, axial force, reaction, deflection, or mode shape is evidence for later work, not proof that a steel member or connection passes. [S1][S2]

Fraia agents should say "analysis result" or "run result" when discussing solver output.

### Design actions are curated from run results

A design action is a selected, transformed, enveloped, or otherwise curated value prepared for a design/check purpose. For a steel beam this may include moment/shear/torsion/axial actions at particular stations and combinations. For a support or connection it may include reaction, shear, moment, axial, or tie forces.

Design actions should reference the run artifact, load case/combination, station, local axis, sign convention, and extraction method.

### Check inputs add engineering assumptions

Steel member design requires context beyond force result values. Professional member-design guidance treats buckling, lateral-torsional buckling, restraint, effective length, section behavior, and combined actions as design concerns. [S3]

Fraia check inputs should include material/section source, member role/id, local axes, restraint, unbraced/effective length, connection fixity, section classification or equivalent future metadata, code scope, and provenance where relevant.

### Check results are produced by check execution

A check result is the output of a check module after combining design actions, check inputs, code/design rules, and assumptions. It should report status, utilization or margin where applicable, governing mode, source/code scope, and provenance.

Fraia should not store a design action as a check result.

### Missing check inputs should reduce confidence

If restraint, section, material, connection fixity, or code scope is missing, the check should be incomplete or downgraded. Guessing those fields can make a false pass look authoritative.

Fraia agents should ask for missing context or clearly mark assumptions.

## Engineering guidance for Fraia agents

- Use the terms analysis result, design action, check input, and check result precisely.
- Do not call a steel member adequate from force diagrams or axial/moment values alone.
- Preserve provenance from authored `Member`, `LoadAssignment`, `SupportAssignment`, `ReleaseAssignment`, resolved topology, run artifact, extraction method, check inputs, and check result.
- Keep material/section data source, local axes, stationing, combinations, and sign convention attached to design actions.
- Keep restraint, unbraced/effective length, connection fixity, section behavior, and code scope attached to check inputs.
- Mark checks incomplete when required check inputs are missing.
- Treat exports/reports/spreadsheets as renderers of check data, not sources of truth.

## Tradeoffs / cautions

- A stricter pipeline requires more metadata, but it prevents hidden assumptions and misleading automated checks.
- Early concept design may use rough assumptions, but those assumptions must be visible and downgraded appropriately.
- A green analysis run can still produce invalid check inputs if the model has wrong supports, releases, restraints, or load combinations.
- A check result should remain tied to the specific run and check-input set that produced it.
- Human review remains necessary for code-specific engineering judgment and project approval.

## Source-backed claims

- Fraia's downstream architecture explicitly separates analysis results, design-action extraction, check input generation, check execution/evaluation, and output rendering. [S1]
- Fraia's architecture requires authored project data, resolved runtime data, and immutable run snapshots to remain distinct. [S2]
- Outputs such as Excel, CSV, reports, and CAD/detail exports should render structured engineering data rather than become source of truth. [S1]
- Steel member design requires behavior/check context beyond raw force values, including buckling, LTB, restraint, and combined-action considerations. [S3]
- Missing or hidden assumptions should not be silently converted into authoritative check results. [S1][S2]

## Open questions / weak evidence

- Fraia still needs final typed schemas for design actions, check inputs, check results, check packets, and source/code metadata.
- Jurisdiction-specific steel check modules are future work.
- Connection, purlin/girt, portal-frame, serviceability, and combined-action check packet details need future pages.

## Related pages

- [Steel material properties and section families](material-properties-and-section-families.md)
- [Steel compression members](compression-members.md)
- [Steel beams and bending members](beams-and-bending-members.md)
- [Steel connections concept taxonomy](connections-concept-taxonomy.md)
- [Beam shear and moment diagrams](../../analysis/beam-shear-and-moment-diagrams.md)
- [Member restraint and unbraced length](../../stability/member-restraint-and-unbraced-length.md)

## Sources

- [S1] Fraia, *Engineering Output Pipeline*. Path: `docs/engineering-output-pipeline.md`. Source type: Fraia architecture doc. Consulted: 2026-05-07. Reliability/limits: canonical product architecture direction; draft status and not a steel code source.
- [S2] Fraia, *Resolution and Runs*. Path: `docs/resolution-and-runs.md`. Source type: Fraia architecture doc. Consulted: 2026-05-07. Reliability/limits: canonical authored/resolved/run separation direction; draft status and not a steel code source.
- [S3] SteelConstruction.info / Steel Construction Institute and British Constructional Steelwork Association, *Member design*. URL: https://www.steelconstruction.info/Member_design. Source type: professional steel construction guidance. Retrieved: 2026-05-07. Reliability/limits: useful context for steel member behavior/check inputs; UK/Eurocode context and not Fraia schema guidance.
