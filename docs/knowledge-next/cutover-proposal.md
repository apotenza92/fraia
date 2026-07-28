# Knowledge-Next Cutover Proposal

_Status: proposal only_
_Date: 2026-06-15_

This proposal describes how to eventually cut over from the current operational wiki to `docs/knowledge-next/`. It does not move, delete, or retire any old wiki file.

## Recommendation

Keep `docs/knowledge/wiki/` operational for now.

The typed system is now useful for first-pass retrieval across analysis fundamentals, support idealisation, releases, diagnostics, steel member behavior, portal frames, diagrams, evals, and generated views. It is not yet broad enough to replace the whole old wiki because several pages remain partial or uncovered in `coverage-matrix.md`.

## Cutover Gates

Before any old wiki page is archived:

1. The page must be marked `covered` in `coverage-matrix.md`.
2. Replacement cards must validate with `python3 scripts/validate-knowledge-next.py`.
3. Generated views must be regenerated with `python3 scripts/generate-knowledge-next-views.py`.
4. The generated view must be readable enough for agents and humans to find the replacement cards.
5. A separate explicit user-approved archive plan must list the exact old files to move or retire.

## Proposed Cutover Order

1. Use `docs/knowledge-next/generated/views/cards-index.md` as a secondary retrieval surface while keeping the old wiki live.
2. Fill the high-priority gaps listed in `coverage-matrix.md`.
3. Review cards currently marked as `covered` and decide whether they should remain `draft` or become `reviewed`.
4. Add a small retrieval harness that runs the eval seeds in `docs/knowledge-next/evals/`.
5. Only after those checks pass, prepare a narrow archive plan for fully covered pages.

## No-Deletion Rule

No old wiki content should be deleted in this phase.

The first archive action, if approved later, should be a move-only operation into an archive folder with a manifest. It should preserve paths, source history, and a clear pointer to replacement card IDs.

## Current Operational Truth

- Current readable operational wiki: `docs/knowledge/wiki/`
- Source-first typed records: `docs/knowledge-next/cards/**/*.json`
- Diagram/media metadata: `docs/knowledge-next/assets/**/*.json`
- Retrieval eval seeds: `docs/knowledge-next/evals/**/*.json`
- Generated reader views: `docs/knowledge-next/generated/views/*.md`
- Coverage/cutover planning: `docs/knowledge-next/coverage-matrix.md` and this file
