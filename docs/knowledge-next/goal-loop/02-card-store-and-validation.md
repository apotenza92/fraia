# 02 - Card Store And Validation

_Status: complete_
_Purpose: create the typed storage and validation path before adding real cards_

## Goal

Create a minimal, inspectable place for `KnowledgeCard` and `KnowledgeAsset` records, plus validation commands that make each future loop self-checking.

## Inputs

- `../schemas/knowledge-card.schema.json`
- `../schemas/knowledge-asset.schema.json`
- `../schemas/source-inventory.schema.json`
- `../source-inventory.json`

## Tasks

- Add a stable card storage layout under `docs/knowledge-next/cards/`.
- Add a stable asset storage layout under `docs/knowledge-next/assets/`.
- Add README files that state records are hand/rebuild-maintained source truth, while generated markdown views are renderers.
- Add or extend validation tooling so JSON records under `cards/` and `assets/` are parsed and checked against required local invariants.
- Keep validation dependency-light; prefer standard-library checks unless a dependency already exists in the repo.
- Add a tiny example draft card and example draft asset only if needed to prove the validation path.

## Required Interface Decisions

- Card files are JSON, one card per file.
- Asset files are JSON, one asset per file.
- IDs must match the existing schemas: `KC-*` for cards and `KA-*` for assets.
- Cards must link to source ids from `source-inventory.json`.
- Assets must link to source ids and must never embed copied source media.

## Self-Repair Rules

- If schemas are too loose for validation, tighten only the minimal fields needed by the first card batch.
- If validation finds a card source id missing from the inventory, either fix the card or add the source through the inventory workflow.
- If a card uses a non-public source, mark it draft-blocked or replace the source.
- If a JSON schema check needs a missing package, implement a small local validator instead of adding a dependency by default.

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

- Card and asset directories exist with clear READMEs.
- Validation command exists and is documented.
- All existing knowledge-next JSON parses cleanly.
- Any example card/asset is marked draft/example and uses public rebuild-eligible sources only.
- Next plan: `03-analysis-modeling-cards.md`.

## Completion Note

Completed 2026-06-15. `cards/` and `assets/` exist with READMEs, and `scripts/validate-knowledge-next.py` validates typed record invariants without external dependencies.
