# Fraia Briefing and Planning Workflow

_Status: draft v0.1_
_Date: 2026-04-13_

This document captures the current thinking about Fraia's planning-first workflow before structural modeling or analysis begins.

---

## 1. Why this exists

Users will often begin at a much higher abstraction level than structural primitives or even structural archetypes.

Typical starting prompts may be closer to:

- "I want to design a house"
- "I need a warehouse"
- "Help me plan a shed"
- "Can you size a portal frame workshop"
- "I want to explore options for a small bridge"

This means Fraia needs a layer above structural semantics:

- project brief
- building/system intent
- design stage
- knowns and unknowns

The agent should not rush immediately into model generation if the brief is incomplete.

---

## 2. Core principle

### Planning before modeling
Fraia should generally begin in a **planning/discovery mode** before it starts making or modifying engineering models.

The system should prefer:

1. understand what the user is trying to build
2. identify missing information
3. record assumptions and decisions
4. only then instantiate structural/archetype models

---

## 3. Higher abstraction layer above structural semantics

A user may describe a:

- house
- shed
- warehouse
- mezzanine
- canopy
- bridge
- tower
- industrial frame

These are not yet structural primitives.

They belong to a higher planning/brief layer that may include:

- use/building type
- size/scale intent
- occupancy or operational use
- layout requirements
- material preferences
- environmental/site assumptions
- design maturity
- cost/carbon priorities
- code/jurisdiction context if known

This should be captured before detailed Fraia structural modeling starts.

---

## 4. Agent behavior expectation

### The agent should ask for missing information
If the user brief is incomplete, the agent should be expected to ask follow-up questions rather than pretending it already knows enough.

Typical questions may cover:

- What are you building?
- What is the approximate size?
- How many levels or bays?
- What material systems are acceptable?
- Is this concept design or detailed engineering?
- Do you already know the jurisdiction/code context?
- Are loads known, or should Fraia assume preliminary defaults?
- Is the goal concept generation, analysis, optimization, or detailing?
- What level of detail should Fraia go to?
- Should Fraia stop at global member design, or also investigate connections/local wall behavior?
- Do you have preferred connection families or fabrication styles?

### The agent should not over-ask unnecessarily
The goal is not to interrogate the user forever.

The goal is to gather enough information to move to the next valid abstraction layer safely.

---

## 5. Fraia should persist the planning state

Planning should not live only in agent chat history.

Fraia should create and maintain a markdown planning file early in the workflow.

This file should capture:

- user brief
- assumptions
- clarified answers
- unresolved questions
- decisions made
- recommended next steps

This gives continuity across:

- sessions
- agents
- humans reviewing the work
- future project evolution

---

## 6. Recommended planning files

A possible early structure:

```text
project/
  planning/
    00-project-brief.md
    01-assumptions.md
    02-open-questions.md
    03-decision-log.md
```

A smaller early alternative:

```text
project/
  planning.md
```

The simpler single-file version is probably enough at first.

---

## 7. Suggested contents of the planning markdown

A good planning file may include sections like:

- Project summary
- User goals
- Current design stage
- Desired analysis/detail level
- Known constraints
- Unknowns / clarifications needed
- Material/system options under consideration
- Connection/detailing preferences
- Modeling assumptions
- Proposed Fraia next actions

### Example headings

```md
# Project Planning

## User brief
## Objectives
## Known information
## Assumptions
## Open questions
## Candidate structural systems
## Next steps
```

---

## 8. Planning mode vs execution mode

Fraia should likely distinguish between:

## Planning mode
- asks clarifying questions
- gathers requirements
- proposes options
- writes/updates planning markdown
- avoids premature model commitment

## Execution mode
- instantiates primitives/archetypes
- validates models
- runs analysis
- performs optimization
- generates results and reports

This distinction could remain implicit at first, but it is a useful architectural concept.

---

## 9. Transition from brief to model

A high-level brief should gradually narrow into structural/archetype choices.

Example:

### User brief
"I want a small warehouse."

### Clarified planning outcome
- single-storey steel portal frame building
- approximately 24 m x 40 m
- 6 m eaves height
- industrial/light storage use
- concept stage only
- optimize for low cost

### Then Fraia can proceed to structural/archetype layer
- choose portal frame archetypes
- set spans, bays, heights
- apply preliminary rules and loads assumptions
- begin validation/analysis

### Important workflow note
The transition should usually be incremental rather than one-shot.

A good Fraia agent should often:
1. clarify the brief
2. instantiate an initial structural system or small primitive model
3. show that geometry/assumptions to the user
4. run deterministic validation/analysis
5. revise the model in loops until the structure is acceptable or a decision is needed

This is a better fit for real engineering work than trying to generate a whole final structure in one pass.

## 9.1 More than one entry path can be valid

After planning, Fraia may move into modeling through several entry modes:

- builder/system first
  - e.g. simply supported beam, portal frame, truss family
- direct primitive authoring
  - e.g. create nodes, members, plates, supports, and loads directly
- sketch/diagram-assisted flow later
  - where geometric intent is turned into builders/primitives

The correct path depends on how specific the user's intent already is and whether a known structural system family fits the problem well.

---

## 10. Relationship to abstraction guards

This planning-first approach supports Fraia's broader abstraction rule:

> start at the highest useful abstraction and descend only when necessary

In many cases, the highest useful abstraction is not even a beam or portal frame.

It is something like:

- house
- shed
- warehouse
- bridge concept

That means Fraia needs a user-intent layer above structural semantics.

---

## 11. Agent responsibilities during planning

During planning, the agent should aim to:

- understand the user's real objective
- identify what information is missing
- avoid pretending unknowns are known
- propose assumptions explicitly when needed
- record those assumptions in markdown
- explain when enough information exists to move into modeling

This is likely one of the most important behaviors for Fraia's usability.

---

## 12. Early implementation recommendation

In the first Fraia implementation:

- always create or update a planning markdown document for significant projects
- make the agent ask clarifying questions before modeling when the brief is incomplete
- treat the planning document as durable project context
- only descend into structural/archetype operations after the brief is sufficiently formed

---

## 13. Design choices currently favored

- Fraia needs a planning/brief layer above structural semantics.
- The agent should stay in planning/discovery mode until it has enough information.
- Planning conversations should be persisted in markdown, not only in chat state.
- The planning file should become part of durable project context.
- Execution/modeling should follow planning, not replace it.

---

## 14. Open questions

- Whether planning mode should be explicit in the UI/backend or just a behavior pattern
- Exact markdown file naming convention
- How structured the planning markdown should become over time
- Whether Fraia should maintain both freeform markdown and machine-readable brief JSON later

---

_End of draft._
