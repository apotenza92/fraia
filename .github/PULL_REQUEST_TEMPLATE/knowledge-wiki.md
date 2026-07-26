# Knowledge wiki PR

Use this template for PRs that update `docs/knowledge/` wiki content. For normal code PRs, use the repo's ordinary PR process.

## Summary

- Topic/page(s):
- Type: new page | update | correction | source addition | media | other

## Source provenance

- [ ] Page-level `## Sources` cite original sources, not ingestion tools or source learning packets.
- [ ] Claim-to-source mapping is clear in the PR description or accompanying proposal.
- [ ] Public/open sources are used where feasible.
- [ ] Private/local sources are logically identified and scoped, with no raw content committed.

## Software/manual sources

If using software documentation such as Strand7:

- [ ] Generic principles are separated from software conventions.
- [ ] Software-only details, menu paths, click workflows, screenshots, and proprietary examples are not rewritten into the wiki.

## Copyright/media hygiene

- [ ] No copied source prose, OCR output, website dumps, or raw excerpts are committed.
- [ ] No private/copyrighted PDFs or screenshots are committed.
- [ ] Any committed media is generated/owned or clearly licensed and listed in `docs/knowledge/media/manifest.md`.

## Validation

- [ ] `python3 scripts/lint-knowledge.py`
- [ ] `python3 scripts/build-knowledge-viewer.py` if navigation/viewer docs changed
- [ ] `python3 scripts/review-knowledge-steward.py --evidence <proposal-or-pr-body.md> --require-checklist` for compiled wiki promotion

## Fraia Knowledge Steward

- [ ] Steward review completed before treating the update as compiled guidance.
- Decision: accept | accept-with-edits | needs-more-source | downgrade-to-draft | veto
- Required edits/source requests:
