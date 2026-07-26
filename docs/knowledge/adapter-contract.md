# Fraia Knowledge Adapter Contract

_Status: active v0.2_
_Date: 2026-05-07_

This contract defines how maintainer tools, subagents, community contributors, or third-party ingestion systems may propose knowledge for Fraia's LLM wiki.

Fraia owns the **compiled wiki schema, provenance rules, review gates, and runtime use of knowledge**. It does not need to own every PDF/OCR/web/image ingestion pipeline.

## Product boundary

Fraia runtime/app:

- ships with a curated compiled wiki library
- lets agents consult and cite wiki pages
- may later emit missing-knowledge proposals
- must not run heavy ingestion, crawlers, OCR, or source-compilation jobs as normal app behavior

Maintainer/community side:

- reads external sources using any appropriate tool
- converts source learnings into this contract
- updates compiled wiki pages through visible diffs
- runs lint/reviewer and Fraia Knowledge Steward checks before promotion

## Accepted contribution shapes

Fraia accepts three durable contribution shapes:

1. **Knowledge request** — a topic or correction request with suggested sources.
2. **Source learning packet** — structured learnings extracted from one or more original sources.
3. **Wiki update proposal / PR** — concrete markdown edits to compiled pages plus updated `## Sources`.

Temporary extracts, OCR output, screenshots, chunk notes, and tool summaries are not durable wiki knowledge by themselves.

## Required review chain

Adapter-driven wiki updates must preserve this review chain:

```text
source learning packet
→ wiki update proposal
→ lint/reviewer
→ Fraia Knowledge Steward review
→ accept | accept-with-edits | needs-more-source | downgrade-to-draft | veto
```

For small changes, the source learning packet and wiki update proposal may be represented directly in a PR body or maintenance note. The chain is still required as evidence, not necessarily as committed standalone artifacts.

The Fraia Knowledge Steward is the final product/architecture review before a change is treated as compiled guidance. It checks Fraia relevance, architecture fit, authored/resolved/run boundary preservation, structural vocabulary, vendor/software leakage, source/confidence risk, and the appropriate decision state.

## Anti-source-laundering rule

Do not cite an ingestion tool, LLM summary, chunk note, or source learning packet as the source for an engineering claim when the original source is known.

Every durable claim proposed for a compiled wiki page must trace to an **original source**:

- public URL
- DOI or public bibliographic locator
- logical local/private source locator
- page, chapter, section, figure, or table reference where feasible

Packets without original-source references are treated as **research proposals**, not accepted knowledge.

## Source learning packet contract

A source learning packet should include:

```yaml
packet_id:
topic:
created:
created_by:
status: proposed

processor:
  tool_name:
  tool_version:
  run_id:
  notes:

original_sources:
  - source_ref: S1
    title:
    author_or_organization:
    source_locator: URL | DOI | Local source | bibliographic locator
    source_type: preferred public | strong private reference | practical/software reference | discovery only | other
    consulted_or_retrieved: YYYY-MM-DD
    page_section_figure_range:
    reliability_limits:
    license_or_usage_notes:
```

Processor metadata is useful for debugging, but it is not evidence. Original source metadata is the evidence trail.

## Required learning fields

Each extracted learning should include:

```yaml
- id: L1
  claim_or_principle:
  source_refs:
    - S1 p. 12
    - S2 section 3.4
  confidence: high | medium | low
  applicability:
  limits_cautions:
  suggested_fraia_vocabulary:
  candidate_wiki_targets:
```

A learning without `source_refs` should be marked as an open question or research lead.

## Software-manual distillation taxonomy

Software manuals and tutorials, such as Strand7, OpenSees, RISA, SCIA, Dlubal, LUSAS, Oasys, or solver documentation, are useful practical evidence. They must not be rewritten into Fraia's wiki as product manuals.

Classify each observation as:

- **generic principle** — suitable for compiled guidance after paraphrase, for example insufficient restraint can create rigid-body modes
- **software convention/example** — cite as source-scoped example only, for example a product-specific warning name or result-report convention
- **workflow inspiration for Fraia** — translate into Fraia concepts or UX/diagnostic ideas; do not copy software steps
- **software-only detail to exclude** — menu paths, click steps, proprietary examples, screenshots, or behavior that only matters inside that product

Compiled wiki pages should normally include generic principles and sometimes workflow inspiration translated into Fraia vocabulary. Software-only details stay out unless Fraia is explicitly integrating with that software.

## Public and private source policy

For upstream/public Fraia wiki contributions:

- prefer academic/open textbook, university, professional, government, private textbook, or otherwise well-sourced references where available
- cite public sources for claims that contributors and reviewers should be able to audit
- avoid SEO/content-marketing engineering pages and calculator/tool marketing pages for compiled guidance when stronger sources are available
- do not upload copyrighted PDFs, extracted text, screenshots, or copied figures
- private/local sources may support maintainer synthesis but should be clearly identified and generally should not be the sole basis for public compiled pages unless limitations are stated

For local/private maintainer use:

- cite logical locators such as `OneDrive-Personal/Engineering/Theory/...`
- cite chapter/page/section where feasible
- paraphrase; do not copy source prose or figures into committed pages

## Wiki update proposal contract

A concrete wiki update proposal should include:

- target page(s)
- summary of intended change
- claim-to-source mapping
- exact source entries to add/update
- related pages and cross-links
- trust/status recommendation
- copied-content/media confirmation
- lint/reviewer checklist
- Fraia Knowledge Steward handoff and decision record

Use [`templates/wiki-update-proposal.md`](templates/wiki-update-proposal.md).

## What adapters should not output

Adapters should not produce durable PR content that contains:

- long raw source excerpts
- copied textbook/manual/web prose
- screenshots or copied figures with unclear permission
- claims with no original source reference
- generic claims based only on a tool summary
- software manual click paths or product-specific how-to procedures

## Relationship to compiled pages

Compiled pages under [`wiki/`](wiki/) remain the durable knowledge layer. Source learning packets and wiki update proposals are intermediate review artifacts. A compiled page must still satisfy [`schema.md`](schema.md), page-level source requirements, lint, reviewer checks, and Fraia Knowledge Steward review.
