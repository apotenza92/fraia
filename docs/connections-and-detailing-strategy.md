# Fraia Connections and Detailing Strategy

_Status: draft v0.1_
_Date: 2026-04-13_

_Canonical focus:_ connection-family reasoning and detailing escalation strategy.
_See also:_ `engineering-output-pipeline.md`, `validation-and-diagnostics.md`, `builder-graph-architecture.md`, `documentation-map.md`.

This document captures the current direction for Fraia connection selection, detailing depth, and local connection-check workflows.

---

## 1. Why this matters

For structural engineering, a global member model is not enough.

Even if Fraia stops at a global line-member analysis, it should still be able to say things like:

- this joint likely requires a moment connection
- this beam-to-column interface needs to transfer these actions
- this thin wall or plate region deserves local checking
- this connection family may be incompatible with the chosen section combination

So Fraia needs a connection layer even before fully automated detailing exists.

---

## 2. Core principle

### Connection reasoning should exist before full connection automation

Fraia should be able to:

- infer likely connection demands
- suggest suitable connection families
- warn about local/detail issues
- tell the user when local wall or connection checks are recommended

before it can fully automate detailed connection submodels.

### Important simplification

In many real workflows, bolts and welds are **not** modeled directly in FE from the start.

A more typical progression is:

1. run global analysis
2. extract interface forces/actions
3. choose likely connection family
4. perform simplified/template/code-style checks
5. only escalate to local shell/solid FE detail when justified

Fraia should follow this reality.

---

## 3. Connection option matrix

Fraia likely needs a structured connection compatibility matrix or rule system.

This should answer questions like:

- if section shape X meets section shape Y, what connection families are plausible?
- what connection families are preferred for pinned vs moment behavior?
- what fabrication styles are compatible?
- what local wall or plate checks are commonly required?
- what detailing level is supported in the current workflow?

Examples of dimensions in the matrix:

- shape family to shape family (I-section to I-section, RHS to plate, beam to wall plate, etc.)
- force transfer intent (shear only, moment, axial, combined)
- fabrication preference (bolted, welded, mixed)
- erection/site constraints
- requested design/detail level

---

## 4. Connection preferences should be part of planning mode

Yes: Fraia should ask about connection/detail preferences during planning if the project is likely to go beyond concept level.

Examples of useful questions:

- Do you want Fraia to go down to connection/detail level?
- Should Fraia stop at global member sizing?
- Do you prefer pinned/simple connections or moment frames where possible?
- Do you prefer bolted or welded solutions?
- Should Fraia generate local submodels automatically when needed, or only flag them?

These answers define the allowed depth of automation.

---

## 5. Detail level should be explicit

A project should probably include an intended detail depth such as:

- planning only
- global structural analysis
- member sizing/preliminary design
- local plate/member interaction review
- connection-level checks
- detailed local submodeling

This is important because not every user wants Fraia to automatically go all the way down to connection detail.

---

## 6. Default behavior if detail is skipped

If the user does **not** request automated connection/detail-level analysis, Fraia should still:

- identify likely connection action demands
- identify likely local wall/plate issues
- identify where global modeling is no longer enough
- generate messages/tasks such as:
  - "this connection should be checked for combined shear and moment"
  - "local wall or flange behavior may govern here"
  - "consider local shell/solid submodeling in this region"

This is a very important middle ground.

The default near-term Fraia behavior should therefore be:

- informative connection reasoning first
- actionable local-detail warnings second
- detailed FE connection/submodel automation only when requested or clearly justified

---

## 7. Actionable messages

Those messages should not just be passive warnings.

Fraia should allow the user to act on them.

Examples of actions:

- create local connection study
- choose connection family
- request shell submodel
- request solid submodel
- defer and mark as unresolved
- approve Fraia automation for this issue

This is likely a very strong workflow for Fraia.

---

## 8. Connection family selection vs detailed realization

Fraia should distinguish at least two steps:

## 8.1 Connection family selection
Examples:

- simple shear tab
- end plate
- gusset plate brace connection
- moment end plate
- welded rigid joint
- base plate family

## 8.2 Detailed local realization/checking
Examples:

- shell model of plate and member walls
- solid model of local region
- bolt/weld group calculations later
- local plate/thin-wall checks

This distinction lets Fraia be useful before full connection automation exists.

---

## 9. Generic engine vs structural-specific logic

Part of this belongs in the structural domain layer:

- connection family concepts
- section-shape compatibility rules
- structural force-transfer semantics

Part may belong in the deeper engine:

- submodel extraction
- shell/solid realization
- mixed-dimensional coupling
- local fidelity escalation

So Fraia should split the problem accordingly.

---

## 10. Suggested near-term Fraia behavior

A good near-term approach:

### Step 1
Global structural model runs first.

### Step 2
Fraia infers likely connection/interface demands from member/plate forces and object interactions.

### Step 3
Fraia suggests likely connection families and emits actionable messages/warnings for locations that deserve local checks.

### Step 4
If allowed by project intent, Fraia can automate a local detailed study or ask the user to choose a connection/detail path.

This is a realistic and valuable early workflow.

---

## 11. Example messages Fraia should eventually produce

- "Beam B14 framing into Column C3 likely requires a moment-capable connection under this option."
- "Plate edge attached to Member M7 shows high demand; local shell refinement is recommended."
- "Thin wall behavior in this section family may need local buckling/detail checks."
- "This support region transfers large reaction forces and should be checked beyond the global member model."

These are exactly the kinds of messages that make Fraia useful even before full detailing automation.

---

## 12. Design choices currently favored

- Fraia needs a structured connection compatibility / option matrix.
- Connection/detail preferences should be part of planning mode.
- Detail depth should be explicit in the project intent.
- If detailed automation is skipped, Fraia should still emit actionable connection and local-detail warnings.
- Near-term Fraia should focus on force extraction, connection family suggestion, and warning/orchestration rather than full bolt/weld FE modeling.
- Users should be able to action those warnings later and trigger more detailed local studies.

---

## 13. Open questions

- Exact schema for connection family matrix/rules
- Exact project-level detail-depth model
- How early Fraia should force the user to choose connection preferences
- How automatic local connection submodel creation should be in early versions

---

_End of draft._
