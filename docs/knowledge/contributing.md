# Contributing to Fraia Knowledge

_Status: active v0.2_
_Date: 2026-05-07_

Fraia's knowledge wiki is compiled, source-backed engineering knowledge. Contributions should improve the compiled wiki without turning the repo into a raw document dump.

## Ways to contribute

- **Knowledge request**: ask Fraia to learn a topic or improve a weak page.
- **Source suggestion**: recommend public sources, textbooks, manuals, or standards commentary.
- **Source learning packet**: submit structured learnings from sources using [`templates/source-learning-packet.md`](templates/source-learning-packet.md).
- **Wiki update proposal**: propose concrete compiled-page edits using [`templates/wiki-update-proposal.md`](templates/wiki-update-proposal.md).
- **Compiled page PR**: edit `wiki/` pages directly, including source entries and cross-links.
- **Correction/errata**: identify an incorrect, overbroad, stale, or weak claim.
- **Media contribution**: add generated/open-license diagrams with [`media/manifest.md`](media/manifest.md) metadata.

## Contribution rules

Do:

- cite original sources, not just ingestion tools or summaries
- prefer academic/open textbooks, university notes, professional institutions, government guidance, private textbooks with logical page/section citations, or otherwise well-sourced references
- include page, chapter, section, figure, or URL references where feasible
- paraphrase concepts in Fraia vocabulary
- distinguish source-backed claims from heuristics or open questions
- keep software-manual learnings generic unless Fraia is explicitly integrating that software
- run the knowledge lint before submitting wiki changes

Do not:

- upload copyrighted PDFs, extracted text, OCR output, source screenshots, or copied figures
- paste long source excerpts
- use SEO/content-marketing engineering pages or calculator/tool marketing pages as compiled guidance when stronger sources exist
- rewrite software manuals into Fraia docs
- cite an LLM/ingestion summary when the original source is known
- use `trust_level: canonical`
- create duplicate canonical pages for existing topics

## Public-source preference

For upstream/public Fraia wiki pages, academic/open textbook, university, professional, government, and otherwise well-sourced references are preferred. Private/local sources can support maintainer synthesis, but public contributors and reviewers should be able to audit important engineering claims where possible.

If a private/local reference is used, cite it logically and state the limitation, for example:

```md
Local source: OneDrive-Personal/Engineering/Theory/Understanding Structural Analysis By David Brohn .pdf. Consulted: YYYY-MM-DD. Source type: strong private reference. Reliability/limits: local textbook reference; paraphrased; not redistributed.
```

## Software manuals

When using software documentation such as Strand7, classify observations as described in [`adapter-contract.md`](adapter-contract.md):

- generic principle
- software convention/example
- workflow inspiration for Fraia
- software-only detail to exclude

Compiled pages should normally include only generic principles and carefully translated workflow inspiration.

## Suggested workflow

1. Check [`index.md`](index.md) and [`topic-map.md`](topic-map.md) for existing coverage.
2. If the topic is missing, open a knowledge request or create a proposal under [`proposals/`](proposals/).
3. If you have source-derived learnings, fill [`templates/source-learning-packet.md`](templates/source-learning-packet.md).
4. If you are proposing page edits, fill [`templates/wiki-update-proposal.md`](templates/wiki-update-proposal.md) or include equivalent information in the PR.
5. Update compiled page `## Sources` with original sources.
6. Update cross-links, [`index.md`](index.md), and [`topic-map.md`](topic-map.md) if adding compiled pages.
7. Append [`wiki/log.md`](wiki/log.md) for maintainer/agent maintenance operations.
8. Run validation and reviewer checks.
9. Complete Fraia Knowledge Steward review before treating the change as compiled guidance.

## Validation

Run:

```sh
python3 scripts/lint-knowledge.py
python3 scripts/build-knowledge-viewer.py
python3 scripts/review-knowledge-steward.py --evidence <proposal-or-log.md> --require-checklist
```

The generated `docs/knowledge/viewer.html` is a convenience artifact. Markdown remains the source of truth.

The Steward script checks for recorded review evidence and a promotable decision. It does not replace the reviewer/steward judgment.

## Maintainer and Steward review

External community PRs should receive maintainer review even though internal wiki-maintenance runs may use agent reviewer plus deterministic lint. Every compiled wiki update also needs Fraia Knowledge Steward review.

Maintainer/reviewer checks:

- source quality and original-source traceability
- no copied source prose or media
- claim scope and engineering caution
- no source laundering through ingestion tools
- no software-manual rewriting
- links, metadata, and viewer generation

Fraia Knowledge Steward checks:

- Fraia product relevance
- fit with the compiled-wiki and adapter boundary
- preservation of authored structural state, resolved/realization state, and immutable run artifacts
- correct structural vocabulary, including `role` for authored objects and `element` for analysis discretisation
- vendor/software-specific leakage
- source/confidence risk
- decision: `accept`, `accept-with-edits`, `needs-more-source`, `downgrade-to-draft`, or `veto`
