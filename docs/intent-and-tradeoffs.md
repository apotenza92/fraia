# Fraia Intent, Tradeoffs, and Search Bounds

_Status: draft v0.1_
_Date: 2026-04-13_

This document captures the current direction for how Fraia should understand user intent, acceptable compromises, and search boundaries before autonomous iteration begins.

---

## 1. Purpose

Fraia cannot optimize intelligently unless it understands what kinds of changes are acceptable.

A user may care about:

- lowest cost
- fewer internal columns
- easiest fabrication
- lowest carbon
- smallest member depths
- architectural openness
- future flexibility

These goals often conflict.

Therefore Fraia needs an explicit intent/tradeoff model before deeper looping begins.

---

## 2. Core principle

### Intent defines the search boundary
The agent should not assume it is always allowed to:

- add columns
- reduce spans
- change building layout
- add braces
- switch structural system
- change material family

These must be understood from the brief or clarified during planning.

---

## 3. Fraia should distinguish multiple intent categories

A very important part of intent is also the **requested level of detail**.

Examples:

- concept only
- global member sizing only
- include plate behavior
- include local connection checks
- include detailed submodeling where required

If the user does not request connection/detail-level automation, Fraia should still flag where local checks are recommended.


## 3.1 Hard constraints
Non-negotiable conditions.

Examples:

- no interior supports
- maximum overall depth fixed
- must remain steel
- supports can only occur at boundary locations
- architectural envelope fixed

## 3.2 Soft preferences
Desirable, but tradeable.

Examples:

- prefer low cost
- prefer low carbon
- prefer standard sections
- prefer few unique parts
- prefer simple erection

## 3.3 Search permissions
What Fraia is allowed to change during autonomous exploration.

Examples:

- section resizing allowed
- braces may be added
- internal columns may be introduced
- system family may change
- material family may change only with approval

## 3.4 Approval triggers
What requires Fraia to stop and ask the user.

Examples:

- adding internal supports requires approval
- changing from clear-span to multi-column solution requires approval
- changing structural material requires approval
- changing overall geometry requires approval
- escalating from global analysis to detailed connection/submodel analysis requires approval unless pre-authorized

---

## 4. Intent is not just objectives

An objective like “minimize cost” is not enough.

Fraia also needs to know:

- what it is allowed to sacrifice
- what it must preserve
- what alternatives the user still wants to see, even if not preferred

This is why Fraia should behave partly like a consultant rather than a pure optimizer.

---

## 5. Fraia should preserve option diversity

Even if the user has a preferred direction, Fraia should often still present a few alternative solution families.

For example:

- preferred low-cost option
- preferred clear-span option
- preferred low-carbon option
- preferred simplest-fabrication option

This keeps the user informed about the tradeoff space rather than trapping them in one search path.

---

## 6. Candidate comparison dimensions

Likely comparison dimensions include:

- capital cost
- embodied carbon
- structural weight
- member depth/profile sizes
- clear spans / internal supports
- fabrication complexity
- erection complexity
- rule/check margins
- architectural impact

Fraia should likely compare options across multiple axes, not just one scalar score.

## 6.1 Design-option intent records

Design options should be proposed as typed intent records before deterministic realization.

Current direction:

- agents consult the Base Model, confirmed constraints, and retrieved wiki knowledge
- agents propose only design-option intents that are worth exploring
- each intent states its hypothesis, exploration band, objective tags, standardisation strategy, connection/detail strategy, support strategy, section-family policy, coordination-group policy, assumptions, and provenance
- deterministic Fraia code validates and realises accepted intents into design-option views
- once realised, a design option is an immutable comparison artefact
- design-option chats act as a review lens for explanation, critique, comparison, and user questions
- changing an option creates a replacement intent and marks the original option superseded; it does not mutate the Base Model

Fraia should not keep a hidden menu of typical design options in runtime code. Typical concepts belong in compiled wiki guidance, project metadata, or explicit agent-authored `DesignOptionIntent` records.

---

## 7. Consultant-style behavior

Fraia should aim to behave like a strong engineering consultant:

- clarify the real problem first
- identify hard blockers and negotiable tradeoffs
- generate several viable options
- explain why each option exists
- explain what each option gives up to gain something else

This is a better user experience than presenting one opaque “best” result.

---

## 8. Planning mode responsibilities

Planning mode should try to extract intent information such as:

- what is essential to preserve?
- what tradeoffs are acceptable?
- what changes are off-limits?
- what level of creativity is desired?
- should Fraia stay conservative first or also propose more inventive alternatives?

This information should be written into the planning markdown.

---

## 9. Suggested intent model fields

A future Fraia intent model may need fields like:

- project goal summary
- hard constraints
- soft preferences
- ranked priorities
- allowed change classes
- approval-required change classes
- option diversity requested
- design maturity level

---

## 10. Search personas / subagent personalities

A useful future pattern may be to assign different search styles to different subagents.

Examples:

- conservative sizing agent
- practical constructability agent
- low-carbon agent
- clear-span/open-space agent
- inventive alternative-systems agent

The orchestrator can then compare their outputs against the same intent model.

---

## 11. Why this matters for Fraia

Without an explicit intent/tradeoff model, Fraia risks:

- over-optimizing the wrong thing
- violating important but unstated constraints
- being too timid and never proposing better alternatives
- being too creative and proposing unacceptable solutions

Intent modeling is what makes autonomous design exploration useful rather than chaotic.

---

## 12. Suggested early implementation path

Early Fraia versions can start simple by recording in planning markdown:

- hard constraints
- preferred objectives
- allowed search scope
- option count requested

Later this can evolve into a more structured machine-readable intent model.

---

## 13. Design choices currently favored

- Intent and tradeoffs must be established before serious autonomous iteration.
- Fraia should distinguish hard constraints, soft preferences, search permissions, and approval triggers.
- Fraia should generally return multiple strong options, not a single answer.
- Search diversity can be implemented partly through specialized subagents/personas.
- Planning markdown should record intent bounds explicitly.

---

## 14. Open questions

- Exact structured intent schema
- How much option diversity should be automatic vs user-configured
- How to score “interesting but non-dominant” alternatives
- How often Fraia should interrupt to ask approval before crossing into wider search bands

---

_End of draft._
