# 04 - Steel And System Cards

_Status: complete_
_Purpose: seed steel behavior and portal-frame cards from public sources_

## Goal

Create source-backed cards for Fraia's first material/system track: steel members and portal-frame buildings.

## Topic Batch

Create cards for:

- steel material properties and section families
- steel beams and bending members
- steel compression members
- member restraint and unbraced length
- lateral-torsional buckling concepts
- bracing principles
- steel portal-frame system overview
- portal-frame base fixity tradeoffs
- purlins and girts as restraint/load-transfer members
- longitudinal vs transverse stability in portal frames
- steel design action/check-input separation, with Fraia artifact vocabulary separated from public steel claims

## Inputs

- Public rebuild-eligible steel and portal-frame sources from `../source-inventory.json`
- Existing wiki pages under:
  - `../../knowledge/wiki/materials/steel/`
  - `../../knowledge/wiki/stability/`
  - `../../knowledge/wiki/steel/portal-frames/`
- Public provenance/lineage sources from `../internal-source-trace.md` only for general artifact/provenance concepts.

## Tasks

- Read original public sources before writing cards.
- Keep code/jurisdiction context explicit, especially for SteelConstruction.info and Eurocode-oriented guidance.
- Avoid final design formulas unless the public source, scope, and usage are unambiguous.
- Keep preliminary/design-option intelligence concept-level and source-scoped.
- Separate Fraia-specific artifact names from public steel behavior claims.

## Self-Repair Rules

- If a steel card implies code compliance, downgrade or rewrite it as concept guidance.
- If a claim is really Fraia workflow policy, move it to a product/pipeline card.
- If a source is regional, preserve the region/standard context in applicability.
- If portal-frame pages duplicate each other, merge concepts into fewer stronger cards and use relationships.

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

- Steel/system cards cover the topic batch at draft or reviewed status.
- Regional and standard limits are explicit.
- No card claims final code design authority.
- Product vocabulary is separated from public steel source claims.
- Next plan: `05-assets-and-diagrams.md`.

## Completion Note

Completed the steel and portal-frame card batch:

- `cards/steel/KC-steel-material-and-section-families.json`
- `cards/steel/KC-steel-bending-members.json`
- `cards/steel/KC-steel-compression-members.json`
- `cards/steel/KC-member-restraint-and-unbraced-length.json`
- `cards/steel/KC-lateral-torsional-buckling-concepts.json`
- `cards/steel/KC-steel-bracing-principles.json`
- `cards/systems/KC-steel-portal-frame-system-overview.json`
- `cards/systems/KC-portal-frame-base-fixity-tradeoffs.json`
- `cards/systems/KC-steel-portal-purlins-and-girts.json`
- `cards/systems/KC-portal-frame-longitudinal-transverse-stability.json`
- `cards/product/KC-steel-design-action-check-input-separation.json`

Validation evidence:

- `python3 scripts/audit-knowledge-sources.py --check`
- `python3 scripts/validate-knowledge-next.py`
- `python3 scripts/lint-knowledge.py`
- `python3 -m py_compile scripts/audit-knowledge-sources.py scripts/validate-knowledge-next.py`
- JSON parse over every `docs/knowledge-next/**/*.json`
