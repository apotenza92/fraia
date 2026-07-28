# 07 - Cutover And Archive

_Status: complete_
_Purpose: retire old wiki coverage only after the typed system is ready_

## Goal

Compare the typed card/generated-view system against the existing `docs/knowledge/` wiki and plan any cutover or archive without losing useful coverage.

## Inputs

- Typed cards and assets.
- Generated views from plan 06.
- Existing `docs/knowledge/wiki/`.
- `../source-inventory.json`.

## Tasks

- Build a coverage matrix from old wiki pages to new cards and generated views.
- Identify old wiki pages that are fully covered, partially covered, or not covered.
- Keep uncovered pages operational until replacement cards exist.
- Write an archive/cutover proposal before moving or deleting any old wiki content.
- Preserve source provenance and public-only policy in any generated replacement.

## Self-Repair Rules

- If a generated view lacks old wiki coverage, create or fix cards rather than archiving the old page.
- If a card lacks public source support, keep the old page operational and mark the card draft.
- If an old page contains product architecture rather than public engineering knowledge, move it into internal rationale docs/cards rather than forcing public-source treatment.
- If deletion is proposed, require a separate explicit user-approved plan.

## Checks

```sh
python3 scripts/audit-knowledge-sources.py --check
python3 scripts/validate-knowledge-next.py
python3 scripts/lint-knowledge.py
python3 - <<'PY'
import json
from pathlib import Path
for path in sorted(Path('docs/knowledge-next').glob('**/*.json')):
    json.loads(path.read_text())
    print(path)
PY
```

## Done Criteria

- Coverage matrix exists.
- No old wiki page is retired without replacement coverage.
- Archive/cutover proposal exists and is separate from source/card generation.
- Current wiki remains operational unless the user explicitly approves cutover.

## Completion Note

Completed conservative cutover/archive planning:

- `coverage-matrix.md` maps old `docs/knowledge/wiki/` pages to typed cards and generated views.
- `cutover-proposal.md` keeps the old wiki operational and requires a separate explicit user-approved archive plan before any move/delete action.
- `README.md` and `rebuild-plan.md` now reflect the current typed-store state.
- No old wiki files were moved, deleted, or retired.

Validation evidence:

- `python3 scripts/generate-knowledge-next-views.py`
- `python3 scripts/audit-knowledge-sources.py --check`
- `python3 scripts/validate-knowledge-next.py`
- `python3 scripts/lint-knowledge.py`
- `python3 -m py_compile scripts/audit-knowledge-sources.py scripts/validate-knowledge-next.py scripts/generate-knowledge-next-views.py`
- JSON parse over every `docs/knowledge-next/**/*.json`
