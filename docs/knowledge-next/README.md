# Fraia Knowledge Next

_Status: rebuild staging_
_Date: 2026-06-15_

This directory is the staging area for the source-first Fraia knowledge rebuild.

The current `docs/knowledge/` wiki remains operational. Do not delete, rewrite, or replace it until the source inventory and typed schema have been reviewed and a migration path exists.

## Current milestone

The source inventory, first typed card batches, asset records, eval seeds, generated views, and cutover planning artifacts now exist:

- [`source-inventory.json`](source-inventory.json): generated machine-readable inventory.
- [`source-inventory.md`](source-inventory.md): generated human review view.
- [`internal-source-trace.md`](internal-source-trace.md): manual trace for internal Fraia/wiki citations that need public replacements or internal-architecture treatment.
- [`rebuild-plan.md`](rebuild-plan.md): active operational plan for rebuilding the knowledge base from public source material into typed cards/assets.
- [`goal-loop/`](goal-loop/README.md): numbered `/goal`-friendly loop plans for completing the rebuild in self-checking slices; start with [`00-loop-protocol.md`](goal-loop/00-loop-protocol.md).
- [`schemas/knowledge-card.schema.json`](schemas/knowledge-card.schema.json): candidate typed knowledge-card model.
- [`schemas/knowledge-asset.schema.json`](schemas/knowledge-asset.schema.json): candidate diagram/image/media asset model.
- [`schemas/source-inventory.schema.json`](schemas/source-inventory.schema.json): generated inventory shape.
- [`cards/`](cards/README.md): public-source-backed typed knowledge cards.
- [`assets/`](assets/README.md): diagram/image metadata and generated-safe visual targets.
- [`evals/`](evals/README.md): retrieval eval seeds.
- [`generated/`](generated/README.md): reproducible generated reader views.
- [`coverage-matrix.md`](coverage-matrix.md): old wiki to typed-card coverage matrix.
- [`cutover-proposal.md`](cutover-proposal.md): conservative archive/cutover proposal.

Regenerate the inventory with:

```sh
python3 scripts/audit-knowledge-sources.py
python3 scripts/audit-knowledge-sources.py --check
python3 scripts/validate-knowledge-next.py
python3 scripts/generate-knowledge-next-views.py
```

The check verifies that generated files are current, missing source metadata is explicitly flagged, public rebuild eligibility is current, compiled pages without original sources are marked for rebuild, logical local paths are not absolute machine paths, and no private/source media files are committed under `knowledge-next/`.

## Scope

This directory is for:

- source inventory and source audit artifacts
- candidate typed knowledge-card schema
- candidate diagram/image asset schema
- typed `KnowledgeCard` records under [`cards/`](cards/README.md)
- typed `KnowledgeAsset` records under [`assets/`](assets/README.md)
- future migration notes from source-backed cards into generated wiki/markdown views

It is not for:

- copied textbook prose
- copied manual excerpts
- private PDFs
- source screenshots
- copyrighted diagrams
- generated runtime prompts

## Rebuild sequence

1. Audit the current wiki, raw notes, source registry, and knowledge plans.
2. Normalize sources into public/professional, textbook/private reference, software/manual, internal Fraia architecture, discovery-only, and weak/replace buckets.
3. Flag pages that cite Fraia wiki pages instead of original references.
4. Review and fill missing source metadata before promoting anything into the new system.
5. Seed `KnowledgeCard` records from high-value original sources first.
6. Attach `KnowledgeAsset` records for diagrams/images, including redraw requirements and generated-safe targets.
7. Generate markdown/wiki views from the typed records after the card model stabilizes.

## Source policy

For the current rebuild phase, seed new `KnowledgeCard` records from publicly available sources only. Public URL/DOI sources marked `public_rebuild_eligible: true` are eligible for first-pass rebuild work.

Private/local references are inventory-only for now. Keep them as logical locators when they already appear in the audit, but do not use them to seed new cards, do not process them, and do not commit PDFs, screenshots, OCR dumps, copied prose, or copied figures. Replace or defer those entries with public sources before promotion.

Original diagrams and images should be represented as `KnowledgeAsset` records. If an asset cannot be embedded directly, mark it as `redraw_required` and describe the generated-safe diagram target.
