# 00 - Loop Protocol

_Status: complete_
_Purpose: shared rules for all numbered knowledge rebuild goals_

## Goal

Make every `/goal` run deterministic, self-fixing, and resumable.

## Operating Loop

For each numbered plan:

1. Read the numbered plan, `../rebuild-plan.md`, and `../README.md`.
2. Inspect the current files before editing.
3. Fix prerequisite drift first, even if the drift is in an earlier plan.
4. Make the smallest coherent change that advances the current plan.
5. Run the plan's checks.
6. If checks fail, repair and rerun.
7. Update status notes inside the plan only when the status actually changes.
8. Stop when the plan's done criteria are met, then continue to the next numbered plan.

## Global Invariants

- Public source material only for the first rebuild pass.
- Private/local sources are inventory-only unless the user explicitly changes policy.
- Old wiki pages are topic/source leads, not migration content.
- Typed records become durable truth; generated markdown/wiki views are renderers.
- Product/pipeline knowledge must distinguish Fraia internal policy from public engineering claims.
- No copied source prose, OCR dumps, PDFs, screenshots, or copied diagrams may be committed.

## Standard Checks

Run these whenever source inventory or knowledge-next docs change:

```sh
python3 scripts/audit-knowledge-sources.py --check
python3 scripts/validate-knowledge-next.py
python3 scripts/lint-knowledge.py
```

Run these whenever scripts or JSON schemas/data change:

```sh
python3 -m py_compile scripts/audit-knowledge-sources.py
python3 - <<'PY'
import json
from pathlib import Path
for path in sorted(Path('docs/knowledge-next').glob('**/*.json')):
    json.loads(path.read_text())
    print(path)
PY
```

## Self-Repair Rules

- If the inventory is stale, regenerate it with `python3 scripts/audit-knowledge-sources.py`, inspect the diff, then rerun `--check`.
- If typed records fail validation, repair the card/asset/source link rather than loosening validation by default.
- If a source is not public rebuild-eligible, do not use it for cards; find a public replacement or defer that card.
- If a card needs a claim not covered by current public sources, add a backlog note instead of inventing the claim.
- If generated views drift from cards, regenerate views rather than hand-editing generated files.
- If old plans conflict with `../rebuild-plan.md`, treat the old plans as historical and update their status note if needed.

## Done Criteria

- This protocol is linked from `README.md`.
- Every numbered plan follows this protocol.
- Standard checks pass after any protocol edits.

## Completion Note

Completed 2026-06-15. The protocol is linked from the knowledge-next README and the numbered plans use the shared inspect/repair/check/done pattern.
