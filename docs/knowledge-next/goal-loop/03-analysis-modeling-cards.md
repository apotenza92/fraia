# 03 - Analysis And Modeling Cards

_Status: complete_
_Purpose: seed the first durable engineering cards from public sources_

## Goal

Create the first useful `KnowledgeCard` batch for structural-analysis and modeling fundamentals.

## Topic Batch

Create cards for:

- free-body diagrams and equilibrium
- reactions and support idealisation
- load paths
- supports, restraints, and releases
- member end releases
- load application and equivalent nodal loads
- static determinacy, restraint sufficiency, and mechanisms
- reaction sanity checks
- instability mechanisms, excluding private/local Strand7 evidence

## Inputs

- Public rebuild-eligible sources from `../source-inventory.json`
- Public replacements listed in `../internal-source-trace.md`
- Existing wiki pages only as topic/source maps:
  - `../../knowledge/wiki/analysis/`
  - `../../knowledge/wiki/modeling/`
  - `../../knowledge/wiki/loads/`
  - `../../knowledge/wiki/diagnostics/`

## Tasks

- Read original public source material for each card before writing claims.
- Write cards in `docs/knowledge-next/cards/analysis/`, `modeling/`, `loads/`, and `diagnostics/`.
- Keep claims short, source-linked, and explicitly scoped.
- Record applicability and limitations for educational, software, or jurisdiction-scoped sources.
- Link related cards by `card_id`, not by old wiki paths.
- Add backlog notes for missing claims rather than inventing unsupported content.

## Self-Repair Rules

- If a card starts copying old wiki prose, rewrite it from source notes and public citations.
- If a card depends on an internal Fraia source, trace to public sources or mark the claim as Fraia product policy and move it out of the domain card.
- If two cards overlap heavily, split by engineering role: concept, modeling assumption, diagnostic, or product policy.
- If source coverage is weak, keep the card `draft` and add a source gap note.

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

- The topic batch has draft or reviewed cards.
- Every non-product claim cites original public sources.
- No private/local sources are used.
- Each card has relationships to adjacent cards where useful.
- Next plan: `04-steel-system-cards.md`.

## Completion Note

Completed the first structural-analysis and modeling card batch:

- `cards/analysis/KC-free-body-equilibrium.json`
- `cards/analysis/KC-support-reactions-idealisation.json`
- `cards/analysis/KC-load-paths.json`
- `cards/modeling/KC-member-end-releases.json`
- `cards/loads/KC-load-application-equivalent-loads.json`
- `cards/analysis/KC-determinacy-restraint-mechanisms.json`
- `cards/diagnostics/KC-reaction-sanity-checks.json`
- `cards/diagnostics/KC-instability-diagnostics.json`

Validation evidence:

- `python3 scripts/audit-knowledge-sources.py --check`
- `python3 scripts/validate-knowledge-next.py`
- `python3 scripts/lint-knowledge.py`
- `python3 -m py_compile scripts/audit-knowledge-sources.py scripts/validate-knowledge-next.py`
- JSON parse over every `docs/knowledge-next/**/*.json`
