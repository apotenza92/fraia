# Design Option Analysis Plan

_Status: implemented decision workflow; ongoing solver/check refinement_
_Date: 2026-05-18_

## Goal

Implement real analysis for generated design options, including each option's internal variables such as support strategy, load realisation, section-family policy, candidate section sizes, and standardisation/grouping choices.

The result should let Fraia say which candidate sections appear viable under a conservative preliminary check, rather than only showing catalogue weight comparisons.

The delivered user-facing journey is documented in [`../docs/structural-app-object-model.md`](../docs/structural-app-object-model.md) and [`../docs/resolution-and-runs.md`](../docs/resolution-and-runs.md). This plan continues to own solver, candidate-check, comparison-result, and immutable analysis-artefact behaviour.

## Current State

Fraia currently does these things:

- Codex authors `DesignOptionIntent` records.
- Deterministic code realises design-option scenes with supports, loads, section-family policy, and group choices.
- The Results view shows selected catalogue shapes and candidate section-weight alternatives.
- The Analysis view only checks the current Base Model readiness path.

Fraia does not yet do these things for design options:

- solve each realised design option;
- calculate option-specific reactions, member actions, deflections, or drift;
- evaluate each candidate section against force/moment demand;
- compare stress against conservative material limits;
- persist option-specific solver/check artefacts.

## Product Principle

Design options are immutable comparison artefacts. Analysis should run against a frozen realised option snapshot, not mutate the Base Model and not silently rewrite the option.

Changing an option remains a replacement workflow. Analysing an option produces run artefacts attached to that option id.

## Required Data Flow

The intended pipeline is:

```text
Base Model brief
-> DesignOptionIntent
-> realised design option scene/model
-> option analysis candidate set
-> per-candidate analysis realisation
-> solve result
-> design actions
-> preliminary check input
-> preliminary check result
-> option comparison result
```

Each stage should be inspectable and persistable where useful.

## Option Variables to Analyse

Fraia must treat these as explicit analysis variables:

- support strategy: pinned, fixed, pinned/roller, or other realised restraint assumptions;
- loads: self weight, member line loads, point loads, and their exact targets/directions;
- member grouping: which Members share size/family intent;
- section family policy: allowed families per coordination group;
- candidate section sizes: every catalogue candidate considered inside each allowed family;
- release/connection assumptions where present;
- stability/bracing additions where an option introduces them;
- standardisation policy: whether one candidate applies to a group or each Member may diverge.

## Near-Term Analysis Scope

The first real version should be conservative and explicit rather than code-complete.

### Include

- Linear elastic frame analysis for each realised design option that can be represented by the supported frame solver path.
- Per-candidate section substitution for every candidate in each option group.
- Simple governing stress checks using conservative material limits.
- Basic serviceability indicators:
  - maximum node displacement;
  - horizontal drift where meaningful;
  - maximum member deflection if derivable.
- Reaction summaries at support Nodes.
- Member action summaries:
  - axial force;
  - shear;
  - bending moment.

### Exclude For Now

- Full code-based member design.
- Connection design.
- Foundation design.
- Buckling, lateral torsional buckling, effective length, and slenderness design except as warnings or future flags.
- Load combinations beyond the currently realised concept load case, unless a load-combination layer already exists.

## Conservative Preliminary Checks

Fraia should create preliminary check results with clear naming such as `preliminary_conservative_stress_check`.

For each candidate section, compute at least:

- maximum absolute bending stress estimate;
- maximum axial stress estimate where axial force exists;
- simple combined utilisation where both are present;
- governing Member id;
- governing load case/combo id;
- conservative stress limit used;
- pass/fail/margin status.

Initial conservative limits can be simple material defaults, for example:

- steel conservative yield stress value stored in the material/check profile;
- explicit safety or conservatism factor if used;
- warning that this is not a code check.

Every output must distinguish:

- `passes preliminary stress screen`;
- `fails preliminary stress screen`;
- `not checked because analysis did not run`;
- `not checked because required assumptions are missing`.

## Persisted Artefacts

Each analysis run should create immutable artefacts under `runs/`.

Recommended layout:

```text
runs/design-option-analysis-<timestamp>/
  run.json
  option-snapshot.json
  candidate-inputs.json
  solver-results.json
  design-actions.json
  preliminary-checks.json
  comparison.json
  diagnostics.json
  summary.md
```

The run manifest should include:

- design option id;
- DesignOptionIntent id and lifecycle status;
- revision/supersession metadata;
- generated option snapshot hash or stable reference;
- candidate section ids analysed;
- solver adapter and version;
- check profile id;
- units.

## API Shape

Add an endpoint or extend the analysis endpoint with explicit scope:

```json
{
  "projectDir": "...",
  "scope": {
    "kind": "design_options",
    "optionIds": ["..."]
  },
  "candidatePolicy": "all_candidates",
  "checkProfile": "preliminary_conservative_steel"
}
```

Supported scopes should eventually include:

- current Base Model readiness/analysis;
- one design option;
- all active design options;
- one design option plus all candidate sections;
- selected candidate section only.

Do not overload the current Base Model analysis button to imply per-option solving.

## Core/Data Types To Add

Likely new or expanded types:

- `DesignOptionAnalysisRequest`
- `DesignOptionAnalysisRun`
- `DesignOptionCandidateAnalysisInput`
- `DesignOptionCandidateSolveResult`
- `DesignOptionDesignActions`
- `PreliminaryMemberCheckInput`
- `PreliminaryMemberCheckResult`
- `DesignOptionComparisonResult`
- `DesignOptionAnalysisSummary`

The existing `DesignSchemeAnalysisSummary` should either become a real summary sourced from run artefacts or remain absent. It must not be heuristic/fake.

## UI Requirements

The desktop journey exposes Analysis & Comparison as the third and final stage after the Design Options shortlisting stage:

- generated options begin included for analysis and can be excluded without deletion in Design Options
- `Analyse options` appears only in Analysis & Comparison and runs included revisions with missing or stale evidence
- current per-option results are reused when the shortlist changes; a comparison-only refresh freezes the exact current revision/run set without rerunning solvers
- one shared viewport, read-only comparison list, and contextual inspector keep comparison, recommendation, evidence, option chat, and path work together
- Fraia records an explained recommendation but never silently selects an option
- `Work on this option` requires current successful preliminary evidence and creates or reopens a preserved path inside Analysis & Comparison
- raw solver inputs, outputs, and logs remain available through `Engineering evidence` drill-downs

For each option, show:

- selected support strategy;
- realised loads;
- candidate sections analysed;
- pass/fail/margin for each candidate;
- governing Member/action;
- estimated steel mass;
- stress utilisation;
- drift/deflection indicator if available;
- diagnostics when analysis or checks are incomplete.

## Implementation Stages

### Stage 1: Scope and Artefact Plumbing

- Add explicit analysis scope to API types.
- Add design-option analysis run directory creation.
- Persist option snapshots before analysis.
- Stop using user-facing wording that implies solver results exist before this pipeline runs.

### Stage 2: Candidate Expansion

- Expand each design option group into candidate section analysis inputs.
- Preserve grouping/standardisation rules:
  - one candidate may apply to all Members in a group;
  - future policy may allow per-Member divergence.
- Persist candidate input JSON.

### Stage 3: Solver Realisation Per Candidate

- Convert realised option scenes into solver-ready structural models.
- Substitute candidate sections into grouped Members.
- Run the supported frame analysis path for each candidate where possible.
- Persist raw/normalised solver results.

### Stage 4: Preliminary Stress Screen

- Extract governing actions from solver results.
- Calculate conservative stress utilisation per candidate.
- Persist check inputs and check results.
- Mark candidates as pass/fail/not-run/not-checkable.

### Stage 5: Comparison UI

- Replace section-weight-only comparison with real candidate result tables.
- Show mass versus utilisation tradeoffs.
- Highlight lightest passing candidate per option.
- Highlight best option by selectable objective, such as lowest mass among passing candidates or lowest drift.

### Stage 6: Regression Tests

Add tests for:

- design-option analysis cannot mutate the Base Model;
- each active option produces an immutable run artefact;
- each candidate section gets a result or explicit not-checkable diagnostic;
- failed solver runs do not fabricate pass/fail results;
- Results UI does not label mass-only candidates as successful analysis;
- superseded options remain inspectable but are excluded from active analyse-all by default.

## Open Questions

- Which frame solver path should be the first supported option-analysis backend?
- Should candidate generation be capped per family for performance?
- What conservative steel stress limit should be the first default?
- How should self weight be recalculated when candidate section mass changes?
- Should per-option analysis include fixed-base rotational stiffness as ideal fixed, or later allow partial restraint?
- Should options with bracing or topology changes be blocked until their realised model can become a solver-ready structural model?

## Success Criteria

This plan is complete when Fraia can:

- generate design options;
- analyse every active design option without touching the Base Model;
- evaluate every candidate section against a conservative preliminary stress screen;
- show mass, stress utilisation, and key displacement/reaction/action values;
- persist all run artefacts;
- clearly distinguish preliminary checks from full code-based design checks.
