# Fraia Knowledge Backend

_Status: draft v0.1_
_Date: 2026-04-13_

This document captures the current direction for Fraia's long-term knowledge and memory system, inspired by ideas such as persistent LLM-maintained wikis and autonomous iterative research loops.

Concrete wiki content now lives in [`docs/knowledge/`](knowledge/README.md). This document remains the architecture rationale; `docs/knowledge/` is the agent-maintained compiled wiki layer with per-page sources and temporary ingestion staging.

---

## 1. Purpose

Fraia should not rely only on:

- transient chat history
- one-shot prompts
- raw RAG over arbitrary documents

Instead, Fraia should gradually build a durable, structured knowledge backend that compounds over time.

This backend should help Fraia:

- ask better clarifying questions
- remember recurring design patterns
- accumulate modeling heuristics
- improve diagnostics and explanations
- support autonomous iterative loops
- reduce repeated rediscovery of the same concepts

---

## 2. Core idea

A useful Fraia knowledge backend likely combines two patterns:

## 2.1 Persistent wiki-like semantic memory
A curated, structured, markdown-like or graph-backed knowledge layer that stores:

- concepts
- archetypes
- assumptions
- rules of thumb
- modeling patterns
- diagnostic patterns
- design playbooks
- project/domain glossaries

This is analogous to an “LLM wiki” style backend: compact, curated, cumulative, and semantically organized.

## 2.2 Autonomous iterative loops
A controlled self-looping process that can:

- propose a change
- run deterministic validation/analysis
- inspect outcomes
- keep what improves objectives
- continue iterating within explicit constraints

This is analogous to an “autoresearch”-style loop, but for Fraia engineering tasks.

---

## 3. These are not the same thing

The wiki-like layer and the self-loop layer should remain distinct.

## Wiki-like knowledge layer
Primarily for:

- long-term semantic memory
- better question asking
- concept refinement
- reusable knowledge cards
- accumulated domain understanding

## Self-loop layer
Primarily for:

- trial/evaluate/improve cycles
- optimization
- diagnostics loops
- controlled design iteration

They should reinforce each other, but not collapse into one system.

---

## 4. Fraia should grow smarter over time

A key idea is that Fraia should improve not only by changing the model weights, but by improving its persistent external knowledge.

Examples of things Fraia could accumulate:

- common user intents and missing-information patterns
- question trees for buildings like houses, sheds, warehouses, mezzanines
- archetype-selection heuristics
- common instability causes
- common release/support mistakes
- local-axis/orientation troubleshooting patterns
- optimization playbooks for steel frame systems

This kind of accumulated knowledge can improve agent performance significantly.

---

## 5. Role in planning conversations

This could be especially valuable in planning mode.

For example, Fraia might maintain structured knowledge about:

- what questions to ask for a house vs warehouse vs bridge
- what assumptions are commonly needed before modeling
- what minimum information is required before analysis
- what structural systems are plausible for certain briefs, with evidence, limits, and reasons not to use them

This means the knowledge backend can help the agent ask better follow-up questions before any model exists.

---

## 6. Role in diagnostics

A persistent knowledge backend could help Fraia improve diagnosis by storing patterns such as:

- common mechanism signatures
- common orientation failures
- common connectivity mistakes
- common solver mismatch patterns
- common modeling corrections that worked historically

Important:

This should augment deterministic validation, not replace it.

---

## 7. Role in autonomous design loops

The autoresearch-style idea fits Fraia best in controlled iterative workflows.

A Fraia loop might look like:

1. choose candidate change
2. resolve model
3. validate
4. run analysis
5. evaluate objective + constraints
6. record outcome
7. keep, reject, or branch
8. continue

The knowledge backend could improve this by storing:

- what changes tend to help in specific scenarios
- what failure patterns frequently occur
- what search heuristics worked for similar projects

---

## 8. What Fraia should not do

Avoid these mistakes:

- letting the wiki become an untrusted dump of random chat text
- allowing self-updating memory to silently override deterministic engineering truth
- replacing validation with vibes-based remembered heuristics
- letting autonomous loops modify projects without clear provenance and limits

The knowledge backend should assist reasoning, not become a hidden authority.

---

## 9. Types of Fraia knowledge stores

Fraia will likely need multiple knowledge stores, not one giant memory.

## 9.1 Product/system knowledge
Stable Fraia-internal knowledge about:

- primitives
- archetypes
- package semantics
- tool contracts
- diagnostics

## 9.2 Domain knowledge
Reusable engineering/domain knowledge such as:

- building/system type questionnaires
- structural system selection heuristics
- common design patterns
- material/system comparison notes

## 9.3 Project knowledge
Project-local durable memory such as:

- planning markdown
- assumptions
- decisions
- unresolved questions
- design rationale

## 9.4 Run/iteration knowledge
Operational memory such as:

- what changes were attempted
- what improved or worsened objectives
- what failures recurred

These should likely remain separate but linked.

---

## 10. Candidate Fraia wiki structure

A wiki-like backend might contain pages/cards such as:

- `building.house`
- `building.warehouse`
- `archetype.portal_frame`
- `diagnostic.mechanism.all-pin-frame`
- `diagnostic.orientation.parallel-reference-vector`
- `playbook.optimize.portal-frame-cost`
- `questionnaire.house.preliminary`

These should be concise, structured, and curated.

The first concrete implementation is the markdown LLM wiki scaffold at [`docs/knowledge/`](knowledge/README.md), using:

- [`docs/knowledge/wiki/`](knowledge/wiki/README.md) for durable compiled pages
- per-page `## Sources` plus [`docs/knowledge/sources.md`](knowledge/sources.md) for provenance/bibliography
- [`docs/knowledge/adapter-contract.md`](knowledge/adapter-contract.md) for maintainer/community/third-party source-learning and wiki-update inputs
- [`docs/knowledge/contributing.md`](knowledge/contributing.md) for community knowledge requests, source suggestions, corrections, and wiki PRs
- [`docs/knowledge/ingestion.md`](knowledge/ingestion.md) for optional maintainer/adapter temporary source ingestion and chunked reading, not app runtime scope
- [`docs/knowledge/proposals/`](knowledge/proposals/README.md) for agent-discovered draft knowledge gaps
- [`docs/knowledge/raw/`](knowledge/raw/README.md) for legacy/exceptional compact source notes only
- [`docs/knowledge/schema.md`](knowledge/schema.md) for agent maintenance rules
- [`docs/knowledge/workflow.md`](knowledge/workflow.md) for self-update, lint/reviewer, Fraia Knowledge Steward, and promotion policy
- [`docs/knowledge/topic-map.md`](knowledge/topic-map.md) for nested topic roadmap/status
- [`docs/knowledge/index.md`](knowledge/index.md) for compiled-page navigation and registry

The first compiled topic is [steel portal-frame bracing](knowledge/wiki/steel/portal-frames/bracing.md), added to prevent shallow hardcoded bracing schemes from substituting for durable engineering knowledge. The first seed batch adds a small set of general structural engineering pages while preserving the rule that opportunistic agents create proposals, not silent compiled-page mutations. Source extraction now happens temporarily by default in `/tmp/fraia-knowledge/` or gitignored `docs/knowledge/.staging/`, with durable learnings captured in compiled pages and their source lists.

The product/runtime boundary is deliberate: the Fraia app should ship and consult compiled wiki knowledge, while heavy PDF/OCR/web/image ingestion remains maintainer-side, third-party, or community-provided plumbing that feeds Fraia through the adapter contract. Compiled wiki updates should pass through source learning packet, wiki update proposal, lint/reviewer, and Fraia Knowledge Steward review before promotion.

Runtime use should prefer retrieval instructions over page menus or hardcoded design recipes. The app may expose deterministic action schemas and validation gates, but reusable engineering guidance should be retrieved from the compiled wiki and project artefacts. For design options, the agent/wiki layer proposes typed `DesignOptionIntent` records with hypotheses, objective tags, assumptions, and provenance; deterministic Fraia code validates and realises those records into comparable option views. If no justified intents are present, the runtime should ask the planning agent to propose them rather than inventing a canned default set.

---

## 11. Candidate Fraia questionnaires/playbooks

This may be one of the highest-value early uses.

Examples:

- house planning questionnaire
- warehouse planning questionnaire
- portal frame sizing playbook
- instability diagnosis decision tree
- early concept modeling checklist

These can help the agent behave more consistently and ask better questions.

---

## 12. Controlled self-loop architecture

The self-looping agent should not directly improvise unconstrained changes forever.

Instead, a controlled loop should include:

- explicit objective(s)
- explicit constraints
- explicit allowed action space
- deterministic evaluation
- provenance per iteration
- stopping criteria

This keeps the loop engineering-safe and inspectable.

---

## 13. Knowledge extraction sources

Potential future sources for Fraia knowledge growth:

- internal planning files
- validated successful project patterns
- repeated diagnostics patterns
- curated engineering notes
- package documentation
- user-approved design rationales

Fraia should be cautious about what is promoted into durable shared memory.

---

## 14. Governance and trust

Not all remembered knowledge should be equally trusted.

Likely trust categories later:

- canonical Fraia core knowledge
- curated package knowledge
- project-local memory
- experimental learned heuristics

This is important so that self-growing memory does not become a source of silent bad engineering advice.

---

## 15. Relationship to agent runtimes

Fraia's knowledge backend remains Fraia-owned regardless of the agent runtime used to retrieve it.

An agent runtime may help with:

- orchestrating memory updates
- using wiki cards in prompts
- running iterative loops

But Fraia should own:

- the knowledge structure
- trust levels
- update policies
- provenance

The runtime is replaceable; the compiled knowledge, trust policy, and provenance are not runtime state.

---

## 16. Suggested early implementation strategy

Start small.

A sensible first Fraia knowledge backend could be:

- markdown planning files per project
- curated wiki/cards for building types, archetypes, and common diagnostics
- question playbooks for planning mode
- optimization/diagnostic playbooks for a few early structural systems

Only later add:

- optional maintainer-side adapters for automated knowledge extraction
- promotion pipelines
- more autonomous self-improving loops

Automated extraction should remain replaceable plumbing unless a future product decision explicitly brings part of it into runtime. Fraia's stable interface is the compiled wiki plus adapter/contribution contract, not a bundled crawler/OCR stack.

---

## 17. Design choices currently favored

- Fraia should eventually have a persistent structured knowledge backend.
- A wiki-like semantic memory and an autoresearch-style loop are complementary but distinct ideas.
- The knowledge backend should improve planning questions, diagnostics, and optimization guidance.
- Deterministic engineering truth must remain outside the soft memory layer.
- Project-local planning markdown is the first and simplest durable memory.

---

## 18. Open questions

- Exact storage model for wiki/cards
- How much should be markdown vs structured JSON/graph data
- How new knowledge gets promoted from project-local memory into shared Fraia memory
- How much of the self-loop should be heuristic/agent-driven versus deterministic optimizer-driven

---

_End of draft._
