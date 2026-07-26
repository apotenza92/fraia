# Fraia Knowledge Rebuild Plan

_Status: active operational plan_
_Date: 2026-06-15_

This is the active plan for rebuilding Fraia's knowledge base around the source-first, typed, multimodal system staged in `docs/knowledge-next/`.

The current `docs/knowledge/` wiki stays operational during the rebuild. It is a topic map and source lead, not content to migrate directly.

For resumable execution, use the numbered `/goal` loop plans in [`goal-loop/`](goal-loop/README.md). Those files break this plan into self-checking slices that can be repeated until their done criteria are met.

## Current State

- `source-inventory.json` and `source-inventory.md` list the sources found in the current wiki, raw notes, source registry, and knowledge plans.
- The inventory currently has 126 normalized sources:
  - 108 public rebuild-eligible sources
  - 16 internal Fraia/wiki breadcrumbs
  - 2 private/local sources deferred from the public-only rebuild
- `internal-source-trace.md` explains how to treat the 16 internal breadcrumbs.
- Candidate schemas exist for `KnowledgeCard`, `KnowledgeAsset`, and the source inventory.
- The first typed store currently has 20 cards, 9 assets, and 5 retrieval eval seeds.
- Generated markdown reader views exist under `generated/views/`.
- `coverage-matrix.md` and `cutover-proposal.md` define the current conservative cutover posture.
- The old markdown wiki remains available for Fraia agents until typed cards and generated views can replace the needed baseline coverage.

## Rebuild Rules

- Use only sources with `public_rebuild_eligible: true` for the first rebuild pass.
- Do not use private/local textbooks, PDFs, manuals, screenshots, OCR dumps, or copied diagrams in this phase.
- Do not migrate old wiki prose into cards.
- Use old wiki pages only to identify topics, source leads, related concepts, and known gaps.
- Rebuild claims from original public source material, paraphrased into Fraia vocabulary.
- Every durable claim must link to source ids plus page, section, figure, table, URL fragment, or equivalent locator where available.
- Diagrams and images are first-class `KnowledgeAsset` records. If an original source diagram is useful but cannot be embedded directly, record it as `redraw_required` and define a generated-safe target.
- Product/pipeline architecture pages can become internal Fraia rationale cards, but they are not public engineering evidence.

## Phases

### Phase 1 - Stabilize The Source Base

Goal: make the source pool reviewable and boring.

Tasks:

- Keep `scripts/audit-knowledge-sources.py` as the inventory generator.
- Regenerate the source inventory whenever current wiki/source inputs change.
- Review sources with missing metadata flags and either fill metadata or leave them out of the first rebuild batch.
- Treat internal Fraia sources as breadcrumbs only; use `internal-source-trace.md` to find public replacements where needed.
- Keep private/local sources inventory-only until the user explicitly reopens that policy.

Exit criteria:

- `python3 scripts/audit-knowledge-sources.py --check` passes.
- Each first-batch source is public rebuild-eligible.
- No private/source media exists under `docs/knowledge-next/`.

### Phase 2 - Seed Typed Knowledge Cards

Goal: create the first reviewed `KnowledgeCard` batch from public sources.

Initial topic order:

1. Structural-analysis fundamentals: equilibrium, free-body diagrams, reactions, load paths.
2. Supports, restraints, releases, and connection fixity.
3. Load application, load cases, combinations, and equivalent nodal loads.
4. Stability and diagnostics: underrestraint, mechanisms, ill-conditioning, reaction sanity.
5. Steel member behavior: section families, compression, bending, LTB, restraint.
6. Portal frames and bracing: system overview, base fixity, purlins/girts, longitudinal vs transverse stability.
7. Fraia product intelligence: design actions, check inputs, provenance, design-option guidance.

Card-writing rules:

- One card covers one durable concept or tightly related concept family.
- Claims must be small enough to source and test.
- Applicability and limitations are required, especially for jurisdiction, software, or simplified educational sources.
- Relationships should connect cards by engineering meaning, not by old wiki path.
- Product cards must separate Fraia policy from public-source-backed engineering claims.

Exit criteria:

- First card batch validates against `schemas/knowledge-card.schema.json`.
- Each card has at least one public source link.
- High-risk or one-source cards are marked as lower confidence or draft.

### Phase 3 - Add Multimodal Asset Records

Goal: make diagrams/images part of the knowledge model without copying source media.

Tasks:

- For each useful public-source figure or diagram, create a `KnowledgeAsset` record.
- Record source id, original locator, copyright status, caption, concept tags, alt text, redraw status, and generated-safe target.
- Prefer generated/open diagrams for committed assets.
- Do not embed copied textbook/manual/source images unless licensing is explicitly compatible.

Exit criteria:

- Asset records validate against `schemas/knowledge-asset.schema.json`.
- Generated-safe diagrams are tracked separately from original source figures.
- No private or copyrighted source images are committed.

### Phase 4 - Retrieval And Evaluation Seeds

Goal: test that the typed knowledge model helps agents retrieve useful knowledge before generating or reviewing schemes.

Tasks:

- Create small eval prompts for common Fraia tasks:
  - "explain suspicious reactions"
  - "review a portal-frame bracing concept"
  - "ask missing context before scheme generation"
  - "distinguish authored member vs analysis element"
  - "explain why raw solver output is not a check result"
- Each eval should name expected card ids, source-backed concepts, and unacceptable answers.
- Add retrieval eval seeds only after the relevant cards exist.

Exit criteria:

- Evals prove agents can find the right cards and avoid private/internal-only sources.
- Failed retrievals create source/card backlog items, not prompt hacks.

### Phase 5 - Generate Views And Retire Old Wiki Coverage Gradually

Goal: preserve readable wiki ergonomics while moving durable truth into typed records.

Tasks:

- Generate markdown/wiki views from reviewed cards after the typed model stabilizes.
- Keep generated views visibly generated so agents do not hand-edit them as source truth.
- Compare generated view coverage against the old `docs/knowledge/wiki/` baseline.
- Retire or archive old wiki pages only after their concepts are covered by reviewed cards and generated views.

Exit criteria:

- Generated markdown views cover the baseline topic set needed by Fraia agents.
- The old wiki is no longer the only readable knowledge layer.
- Any deletion/archive step has a separate explicit plan and review.

## Acceptance Checks

Run these after changes to the inventory or knowledge-next docs:

```sh
python3 scripts/audit-knowledge-sources.py --check
python3 scripts/validate-knowledge-next.py
python3 scripts/generate-knowledge-next-views.py
python3 scripts/lint-knowledge.py
```

Before promoting typed cards:

- Source inventory is current.
- Card and asset JSON parse cleanly.
- Every claim has an original public source locator.
- No private/local source is used as first-pass evidence.
- No copied prose or copied diagrams are committed.
- Internal Fraia architecture claims are marked as product rationale, not public engineering evidence.

## Obsolete / Historical Plan Handling

The following older plans are preserved as history but are not the active rebuild plan:

- `plans/fraia-llm-wiki-knowledge-base.md`
- `plans/knowledge-wiki-growth-and-seeding.md`
- `plans/knowledge-ingestion-workflow.md`
- `plans/knowledge-ingestion-toolchain.md`
- `plans/knowledge-contribution-adapter-contract.md`

Current truth:

- `docs/knowledge/` is the existing operational compiled wiki.
- `docs/knowledge-next/` is the active rebuild staging area.
- This file is the active operational plan for the public-only source-first typed rebuild.
- `docs/knowledge-next/goal-loop/` is the numbered execution loop for `/goal` runs.
