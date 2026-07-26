# Fraia Knowledge Wiki Workflow

_Status: active v0.3_
_Date: 2026-05-07_

This workflow governs how agents grow `docs/knowledge/`.

## Required update chain

Every future compiled wiki update should pass through this chain:

```text
source learning packet
→ wiki update proposal
→ lint/reviewer
→ Fraia Knowledge Steward review
→ accept | accept-with-edits | needs-more-source | downgrade-to-draft | veto
```

For small maintainer edits, the packet and proposal can be embedded in a PR or maintenance note instead of committed as standalone files. The information must still be present: original-source references, claim-to-source mapping, proposed wiki edits, validation results, and the Steward decision.

## Update tiers

1. **Opportunistic discovery**
   - Any agent that notices missing or weak reusable knowledge may add a proposal under [`proposals/`](proposals/) with suggested sources.
   - Opportunistic agents must not silently mutate compiled pages as a side effect of ordinary project, scheme, or debugging work.
   - Opportunistic agents must not create permanent raw extraction dumps. Source extraction belongs in explicit ingestion runs and temporary staging.

2. **Explicit wiki-maintenance / adapter run**
   - A maintenance worker may consume source learning packets, wiki update proposals, or optional maintainer-side ingestion outputs described by [`adapter-contract.md`](adapter-contract.md) and [`ingestion.md`](ingestion.md).
   - Temporary extraction belongs outside shipped app/runtime behavior, normally in `/tmp/fraia-knowledge/` or gitignored `docs/knowledge/.staging/` when stable repo-local staging is needed.
   - Source learning packets are intermediate review artifacts, not durable authority. Compiled pages cite original sources, not processor tools or packet summaries.
   - The worker must preserve original source provenance, keep pages scoped, update [`index.md`](index.md), update [`topic-map.md`](topic-map.md), and append [`wiki/log.md`](wiki/log.md).
   - The worker must leave enough review evidence for the Steward gate: what changed, which claims are source-backed, which source limits remain, and why the update belongs in Fraia.

3. **Promotion gate**
   - Promotion to `status: compiled` requires deterministic lint, a separate reviewer pass, and Fraia Knowledge Steward review.
   - Recorded Steward evidence can be checked with `python3 scripts/review-knowledge-steward.py --evidence <proposal-or-log.md> --require-checklist`.
   - Human approval is not required for `compiled` pages, but changes must be visible as diffs and reversible.
   - `trust_level: canonical` is reserved for future governance and must not be used.

## Fraia Knowledge Steward review

The Steward review is a product and architecture gate for wiki changes. It is separate from extraction and ordinary linting.

The Steward checks:

- **Fraia product relevance**: the change helps Fraia agents generate, explain, diagnose, or review engineering schemes.
- **Architecture fit**: the change reinforces the compiled-wiki boundary and does not pull heavy ingestion into app/runtime scope.
- **Authored/resolved/run boundary preservation**: guidance keeps authored structural state, resolved/realization state, and immutable run artifacts distinct.
- **Structural vocabulary correctness**: canonical names such as `Member`, `Plate`, `Node`, `SupportAssignment`, `LoadAssignment`, `ReleaseAssignment`, `role`, and analysis `element` are used correctly.
- **Vendor/software-specific leakage**: software manuals are distilled into generic principles or Fraia workflow inspiration, not rewritten as product manuals.
- **Source/confidence risk**: source count, independence, source type, private-source limits, and weak/conflicting evidence are clearly stated.
- **Decision fit**: the update should be accepted, edited, sourced further, downgraded, or rejected.

Steward decisions:

- `accept`: the update can be promoted or retained as compiled after lint/reviewer checks.
- `accept-with-edits`: the update is directionally acceptable once listed edits are applied; repeat Steward review if the edits change substance.
- `needs-more-source`: do not promote to compiled until missing source coverage, corroboration, or claim scope is resolved.
- `downgrade-to-draft`: useful material may remain as draft/proposal, but it is not trusted compiled guidance.
- `veto`: reject the change from compiled wiki pages; keep only a narrow proposal note if useful for future research.

## Source quality

Preferred sources:

- open textbooks and open educational resources
- public university notes
- professional engineering institutions
- peer-reviewed papers or academic monographs where appropriate
- public design guides
- government/public agency guidance
- well-sourced encyclopedic pages only for orientation and reference discovery

Acceptable with caution:

- private/local textbooks or manuals, cited by title/chapter/page and paraphrased only
- vendor/manufacturer guidance when source-scoped and corroborated
- reputable engineering articles that cite primary or professional sources
- software documentation for modeling conventions

Avoid as primary:

- anonymous blogs
- SEO/content-marketing engineering pages
- calculator/tool marketing pages
- AI-generated summaries
- untraceable PDFs
- copied copyrighted textbook passages
- jurisdiction-specific code snippets without clear scope
- Wikipedia except for discovery, vocabulary, and reference hunting

Compiled pages should use at least two independent academic, textbook, professional, government, or otherwise well-sourced references where feasible. One-source pages must state their limitation. SEO/content-marketing pages should not be used for compiled guidance when academic, textbook, professional, or well-sourced references are available.

## Adapter/ingestion gate

Before reading large sources, use an external/maintainer-side adapter workflow and create a bounded chunk plan using [`templates/chunk-manifest.md`](templates/chunk-manifest.md) or a source learning packet using [`templates/source-learning-packet.md`](templates/source-learning-packet.md). No subagent should receive an entire textbook, large manual, or large website crawl. Chunk-reader outputs are temporary staging artifacts unless explicitly approved as compact source notes.

Packets or proposals that lack original source references are research leads only; they must not be promoted into compiled guidance.

## Page lifecycle

- `proposal`: a requested or discovered topic, normally in `proposals/`.
- `raw`: legacy/exceptional compact source notes; not the normal extraction path.
- `draft`: wiki page structure exists but is not trusted for agent guidance.
- `compiled`: synthesized, cited, indexed, linted, and reviewable agent guidance.
- `reviewed`: future stronger state after recorded reviewer pass and sufficient source coverage.
- `canonical`: reserved; do not use without explicit future governance.

## Maintenance checklist

For each compiled page:

- required front matter is present
- no `trust_level: canonical`
- required sections are present
- `source_count` equals the number of `[S#]` source entries
- local links and image links resolve
- sources include `URL:`, `Path:`, or `Local source:`
- sources include `Retrieved:` or `Consulted:`
- sources include `Source type:` and `Reliability/limits:`
- page is listed in [`index.md`](index.md)
- page path appears in [`topic-map.md`](topic-map.md)
- committed media, if any, is listed in [`media/manifest.md`](media/manifest.md)
- mutation is logged in [`wiki/log.md`](wiki/log.md)
- Fraia Knowledge Steward decision is recorded in the proposal, PR, or maintenance log
- `python3 scripts/review-knowledge-steward.py --evidence <proposal-or-log.md> --require-checklist` passes for compiled-promotion evidence
