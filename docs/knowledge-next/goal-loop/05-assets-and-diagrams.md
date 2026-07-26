# 05 - Assets And Diagrams

_Status: complete_
_Purpose: make useful diagrams first-class without copying source media_

## Goal

Create `KnowledgeAsset` records for diagrams, figures, and generated-safe visual targets that help Fraia agents understand structural concepts.

## Candidate Asset Themes

- free-body diagram components
- support DOF/reaction symbols
- local vs global axes
- member release components
- distributed load to equivalent resultant
- load path through portal-frame building
- bracing bay and longitudinal/transverse stability concepts
- member restraint and unbraced length
- design-action/check-input/check-result provenance flow

## Inputs

- Public source figures discovered while writing cards.
- Existing card topics from plans 03 and 04.
- `../schemas/knowledge-asset.schema.json`.

## Tasks

- Create one asset JSON per useful diagram target.
- Record original source locator, copyright status, embed policy, caption, tags, alt text, redraw status, and generated-safe target.
- Prefer `metadata_only` or `generated_derivative_only` for source figures unless open license is clear.
- Add generated-safe diagram specs before generating final images.
- Do not commit copied diagrams, screenshots, crops, or private media.

## Self-Repair Rules

- If licensing is unclear, set `embed_policy` to `metadata_only` or `generated_derivative_only`.
- If a diagram would require copying source expression, rewrite the target as a clean Fraia-native schematic.
- If an asset has no linked card, either link it to a card or keep it draft until the card exists.
- If a generated diagram later exists, keep original-source metadata separate from the generated asset path.

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

- First asset batch exists and parses cleanly.
- Every asset has a source id or generated-original provenance.
- No copied/private media files are committed.
- Generated-safe diagram targets are explicit enough for later image generation or renderer implementation.
- Next plan: `06-retrieval-evals-and-views.md`.

## Completion Note

Completed the first `KnowledgeAsset` batch with generated-safe diagram targets only:

- `assets/KA-free-body-diagram-components.json`
- `assets/KA-support-dof-reaction-symbols.json`
- `assets/KA-local-global-member-axes.json`
- `assets/KA-member-release-components.json`
- `assets/KA-distributed-load-equivalent-resultant.json`
- `assets/KA-portal-frame-load-path.json`
- `assets/KA-portal-frame-bracing-stability-axes.json`
- `assets/KA-member-restraint-unbraced-length.json`
- `assets/KA-design-action-check-provenance-flow.json`

Relevant cards now link to these assets through `media_links`. No copied diagrams, screenshots, source PDFs, crops, photos, or private media were added.

Validation evidence:

- `python3 scripts/audit-knowledge-sources.py --check`
- `python3 scripts/validate-knowledge-next.py`
- `python3 scripts/lint-knowledge.py`
- `python3 -m py_compile scripts/audit-knowledge-sources.py scripts/validate-knowledge-next.py`
- JSON parse over every `docs/knowledge-next/**/*.json`
