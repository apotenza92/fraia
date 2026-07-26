# Fraia Knowledge Source Registry

_Status: active v0.2_
_Date: 2026-05-07_

This registry is a bibliography and source-governance aid. It is not a replacement for page-level `## Sources`; every compiled page must still list the original sources used for that page.

Ingestion tools, LLM summaries, chunk-reader notes, and source learning packets are processors/intermediate artifacts. They are not original sources when the underlying document/webpage/manual/textbook is known.

## Source quality hierarchy

- **Preferred academic/professional**: open textbooks/OER, public university notes, peer-reviewed papers or academic monographs where appropriate, professional engineering institutions, government/public agency guidance, and public design guides.
- **Strong private reference**: user-provided textbooks/manuals consulted locally. Cite title, edition, chapter/page where feasible. Do not copy or redistribute source text/images.
- **Well-sourced reference/discovery**: Wikipedia or broad encyclopedic pages with useful references. Use for vocabulary, orientation, and reference hunting; prefer the cited primary/professional source for compiled claims.
- **Practical/software reference**: solver/software manuals and tutorials. Useful for modeling workflows and diagnostics, but software-scoped and best corroborated by stronger theory/professional sources.
- **Weak/avoid as primary**: SEO/content-marketing engineering pages, calculator/tool marketing pages, anonymous blogs, AI-generated summaries, untraceable PDFs, copied excerpts, and jurisdiction-specific code snippets without clear scope.

Fraia compiled pages should prefer academic, textbook, professional, government, or otherwise well-sourced references. Avoid SEO/content-marketing pages for durable guidance when better sources are available.

## Public/web source entries

Use page-level sources as the authoritative list. Add global entries here only when a source is repeatedly useful across pages.

Recommended format:

```md
- Source id: SRC-000
  Author/organization: ...
  Title: ...
  URL: ...
  Retrieved: YYYY-MM-DD
  Source type: preferred public | practical/software | discovery only | other
  Reliability/limits: ...
  Used by: wiki/path.md
```

## Private/local source entries

Private/local source entries should use logical local locators, not absolute `/Users/...` paths. Do not commit PDFs, extracted text, OCR output, screenshots, or copied prose.

Example format:

```md
- Source id: LOCAL-STRUCT-001
  Author: David Brohn
  Title: Understanding Structural Analysis
  Local source: OneDrive-Personal/Engineering/Theory/Understanding Structural Analysis By David Brohn .pdf
  Consulted: YYYY-MM-DD
  Source type: strong private reference
  Reliability/limits: local textbook reference; paraphrase only; cite chapter/page in page-level sources
  Used by: pending
```

## Third-party adapters and source laundering

A third-party ingestion system may produce useful summaries, but the source entry on a compiled page should cite the original source. A packet that says "tool X summarized this" without original URL/title/page/section references is a research lead, not accepted evidence.

For upstream/public Fraia wiki contributions, prefer academic/open textbook, university, professional, government, or otherwise well-sourced material where feasible. Private/local sources can support maintainer synthesis, but they should be clearly scoped and generally should not be the only basis for a public compiled page unless limitations are stated.

Fraia Knowledge Steward review should call out source/confidence risk before promotion, especially for one-source updates, private/local sources, software manuals, vendor guidance, weak corroboration, or conflicting sources.

## Software/manual sources

Software manuals are valuable for practical modeling and diagnostics, but page claims must state the software scope when the guidance is not general engineering theory. They should be distilled into generic principles or Fraia workflow inspiration, not rewritten as product manuals.

Example logical locator:

```md
Local source: OneDrive-Personal/Engineering/Strand7/10. Linear/.../Rigid Body Modes and Singularity Warning in Static Solvers.pdf
```

Current local/software sources used:

- Source id: LOCAL-STRAND7-STATIC-RBM
  Organization: Strand7 Pty Limited
  Title: ST7-1.10.10.2 Rigid Body Modes and Singularity Warning in Static Solvers
  Local source: OneDrive-Personal/Engineering/Strand7/10. Linear/ST7-1.10.10.2 Rigid Body Modes and Singularity Warning in Static Solvers.pdf
  Consulted: 2026-05-06
  Source type: practical/software reference
  Reliability/limits: local software manual/tutorial; useful for generic solver diagnostic patterns, but warning IDs, UI steps, element-specific handling, and numeric thresholds are software-specific
  Used by: wiki/diagnostics/instability-mechanisms.md

## Wikipedia policy

Wikipedia may be used to discover vocabulary, related topics, and references. Prefer the cited academic, professional, government, or textbook source for compiled structural engineering claims when available.
