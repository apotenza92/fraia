# 01 - Source Base

_Status: complete_
_Purpose: keep the public source pool reliable before card writing starts_

## Goal

Make the source inventory a trustworthy input for the rebuild loop.

## Inputs

- `../source-inventory.json`
- `../source-inventory.md`
- `../internal-source-trace.md`
- `../../knowledge/wiki/`
- `../../knowledge/raw/`
- `../../knowledge/sources.md`
- `../../../scripts/audit-knowledge-sources.py`

## Tasks

- Confirm the inventory reports public rebuild eligibility and rebuild actions.
- Review all sources with missing metadata flags.
- For public sources with missing metadata, fill source metadata at the original wiki/source-registry input if the source is otherwise useful.
- For private/local or weak sources, keep them deferred unless a public replacement is found.
- Keep the 16 internal Fraia sources as breadcrumbs only; do not promote them as source evidence.
- Add source-readiness notes only under `docs/knowledge-next/`; do not rewrite compiled wiki pages unless a metadata fix is clearly required by the inventory.

## Self-Repair Rules

- If `source-inventory.json` is stale, regenerate it and inspect the summary.
- If the inventory includes a private/local source as public eligible, fix `scripts/audit-knowledge-sources.py`.
- If a public source has no reliable locator, mark it deferred or replace it.
- If old plan files imply private-source ingestion is active, keep the historical/superseded banner and follow `../rebuild-plan.md`.

## Checks

```sh
python3 scripts/audit-knowledge-sources.py --check
python3 scripts/lint-knowledge.py
python3 -m py_compile scripts/audit-knowledge-sources.py
```

## Done Criteria

- Inventory check passes.
- Public rebuild-eligible source count is recorded in `../source-inventory.md`.
- Private/local sources are explicitly deferred.
- No first-batch source has unresolved metadata that blocks card writing.
- Next plan: `02-card-store-and-validation.md`.

## Completion Note

Completed 2026-06-15. The inventory reports 108 public rebuild-eligible sources, private/local sources are deferred, and no public rebuild-eligible source has metadata quality flags.
