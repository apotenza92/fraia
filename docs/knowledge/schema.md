# Fraia Knowledge Wiki Schema

_Status: active v0.4_
_Date: 2026-05-07_

This schema is the source of truth for agents maintaining `docs/knowledge/`.

If an agent prompt conflicts with this schema, this schema wins.

## Directory roles

- `wiki/`: compiled knowledge pages. These are synthesized and cross-linked for agent use.
- `proposals/`: draft inbox for agent-discovered missing or weak knowledge.
- `adapter-contract.md`: contract for maintainer/community/third-party source learning packets and proposed wiki updates.
- `contributing.md`: contribution workflow for knowledge requests, source suggestions, and wiki PRs.
- `ingestion.md`: optional maintainer/adapter temporary source-ingestion and chunked reading guidance; not app runtime scope.
- `templates/`: workflow templates such as chunk manifests, source learning packets, and wiki update proposals. Filled extraction artifacts normally belong in staging or PR discussions, not durable wiki pages.
- `sources.md`: optional global bibliography/source registry. Page-level `## Sources` remain mandatory.
- `media/`: committed wiki media, normally agent-created diagrams or open-license media with manifest metadata.
- `raw/`: legacy/exceptional compact source notes only; not default extraction storage.
- `.staging/` and `.cache/`: gitignored temporary local workflow state; never committed.
- `index.md`: registry of compiled pages and primary navigation.
- `topic-map.md`: category tree and roadmap for missing/draft/raw/compiled topics.
- `wiki/log.md`: append-only operation log.

## Required compiled-page front matter

Compiled wiki pages must start with YAML front matter:

```yaml
title: Human readable page title
status: draft | compiled | needs-review | deprecated
trust_level: raw | compiled | reviewed
domain: structural-steel | steel | loads | analysis | modeling | stability | diagnostics | materials | systems | product | other
applies_to:
  - short scope item
not_applicable_to:
  - short non-scope item
jurisdiction_or_standard_context: e.g. concept guidance; AU/UK/US references; not a code check
last_compiled: YYYY-MM-DD
source_count: 1
citation_policy: required | none
owner: agent-maintained
```

`trust_level: canonical` is reserved for future governance and is disallowed.

## Required compiled-page sections

Compiled pages must include these headings:

1. `# <Title>`
2. `## Summary`
3. `## Scope / non-scope`
4. `## Key concepts`
5. `## Engineering guidance for Fraia agents`
6. `## Tradeoffs / cautions`
7. `## Source-backed claims`
8. `## Open questions / weak evidence`
9. `## Related pages`
10. `## Sources`

## Source quality taxonomy

Preferred sources:

- open textbooks and open educational resources
- public university notes
- professional engineering institutions
- peer-reviewed papers or academic monographs where appropriate
- public design guides
- government/public agency guidance
- well-sourced encyclopedic pages only for orientation and reference discovery

Acceptable with caution:

- private/local textbooks and manuals, cited by title/chapter/page and paraphrased only
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
- Wikipedia except for discovery/vocabulary/reference hunting

Compiled pages should rely on academic/open textbook, university, professional, government, private-textbook, or otherwise well-sourced material. Do not use SEO/content-marketing pages for compiled guidance when stronger sources are available.

## Source entry requirements

Every non-trivial engineering claim should be either:

- cited to a source entry,
- marked as an internal Fraia heuristic,
- or marked as unresolved / low-confidence.

Source learning packets, chunk notes, OCR output, ingestion tool summaries, and other processor outputs are not original sources. Compiled pages should cite the original document/webpage/manual/textbook whenever it is known.

Every compiled-page source entry must include:

- `[S#]` id
- title
- organization/author if known
- one locator label: `URL:`, `Path:`, or `Local source:`
- one date label: `Retrieved:` for web/public retrieval or `Consulted:` for local/private references
- `Source type:`
- version/date if available
- region/jurisdiction context if relevant
- `Reliability/limits:`

Compiled pages must have `source_count` equal to the number of `[S#]` source entries.

Private/local sources should use logical paths such as `OneDrive-Personal/Engineering/Theory/...`, not absolute `/Users/...` paths. Do not commit extracted text, screenshots, OCR output, copied prose, or private PDFs.

## Media policy

- Prefer agent-created Fraia-native schematics.
- Private textbook screenshots, PDF crops, OCR images, and unclear-license website screenshots are staging-only by default.
- Every committed file under `media/` must be listed in [`media/manifest.md`](media/manifest.md), except policy/manifest markdown files.
- Local image links must resolve.

## Link policy

- Use relative markdown links for local pages.
- Do not create duplicate canonical pages for the same concept.
- Broader pages may be stubs or indexes if they clearly point to the canonical compiled page.
- Compiled pages must be listed in [`index.md`](index.md).
- Compiled pages must appear in [`topic-map.md`](topic-map.md) once the topic map exists.

## Mutation policy

- Opportunistic agents may create/update proposals automatically.
- Opportunistic agents may suggest sources, but must not create permanent raw extraction dumps.
- Opportunistic agents must not silently update compiled pages during ordinary project/scheme work.
- Explicit wiki-maintenance/adapter runs may synthesize draft/compiled pages from source learning packets, wiki update proposals, or temporary staged chunks.
- Source learning packets without original source references are research proposals only.
- Agents may promote pages to `compiled` only after schema/citation/link checks, deterministic lint, a separate reviewer pass, and Fraia Knowledge Steward review.
- The required wiki-update chain is: source learning packet → wiki update proposal → lint/reviewer → Fraia Knowledge Steward review → `accept`, `accept-with-edits`, `needs-more-source`, `downgrade-to-draft`, or `veto`.
- Steward review decisions should be recorded in the wiki update proposal, PR, or [`wiki/log.md`](wiki/log.md). `accept-with-edits` requires the listed edits to be applied before promotion; `needs-more-source`, `downgrade-to-draft`, and `veto` must not be promoted to compiled guidance.
- `python3 scripts/review-knowledge-steward.py --evidence <proposal-or-log.md> --require-checklist` may be used to verify that Steward evidence and a promotable decision are recorded. It does not perform the review judgment.
- Mutations must be logged in [`wiki/log.md`](wiki/log.md).
- Humans are not expected to manually maintain pages, but all changes must remain auditable and reversible.
