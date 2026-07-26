# 06 - Retrieval Evals And Generated Views

_Status: complete_
_Purpose: prove the typed knowledge can be retrieved and read by agents_

## Goal

Add retrieval eval seeds and generated markdown/wiki views after enough typed cards exist.

## Inputs

- Cards from plans 03 and 04.
- Assets from plan 05.
- Existing `docs/knowledge/` wiki for coverage comparison only.

## Tasks

- Add a small eval seed directory under `docs/knowledge-next/evals/`.
- Create eval cases for:
  - suspicious reactions
  - portal-frame bracing review
  - missing context before scheme generation
  - authored member vs analysis element
  - raw solver result vs design action/check result
- Each eval must list expected card ids, relevant source-backed concepts, and unacceptable answer patterns.
- Add a generated-view strategy and implementation only after cards are stable enough.
- Mark generated views as generated renderers, not source truth.

## Self-Repair Rules

- If an eval cannot name expected cards, write the missing card first or mark the eval blocked.
- If an eval expects knowledge not source-backed by public sources, remove or defer that expectation.
- If generated views drift from cards, regenerate views and do not hand-edit them.
- If retrieval fails because card relationships are weak, improve card relationships before prompt hacks.

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

- Eval seeds exist for the listed cases.
- Each eval points to card ids and public-source-backed concepts.
- Generated-view policy exists.
- Any generated view output is visibly generated and reproducible.
- Next plan: `07-cutover-and-archive.md`.

## Completion Note

Completed retrieval eval seeds and generated reader views:

- Eval seeds:
  - `evals/suspicious-reactions.json`
  - `evals/portal-frame-bracing-review.json`
  - `evals/missing-context-before-scheme-generation.json`
  - `evals/authored-member-vs-analysis-element.json`
  - `evals/solver-result-vs-design-action-check.json`
- Generated-view policy:
  - `generated/README.md`
- Generated view implementation:
  - `scripts/generate-knowledge-next-views.py`
  - `generated/views/cards-index.md`
  - `generated/views/assets-index.md`
  - `generated/views/evals-index.md`
- Validation now checks eval IDs, expected card IDs, expected concepts, and unacceptable answer patterns.

Validation evidence:

- `python3 scripts/audit-knowledge-sources.py --check`
- `python3 scripts/validate-knowledge-next.py`
- `python3 scripts/lint-knowledge.py`
- `python3 -m py_compile scripts/audit-knowledge-sources.py scripts/validate-knowledge-next.py scripts/generate-knowledge-next-views.py`
- JSON parse over every `docs/knowledge-next/**/*.json`
