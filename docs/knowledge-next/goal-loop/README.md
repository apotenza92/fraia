# Knowledge Rebuild Goal Loop

_Status: active loop index_
_Date: 2026-06-15_

This folder breaks the Fraia knowledge rebuild into numbered `/goal`-friendly plans.

Use one numbered file as the active goal at a time. Each file is self-contained enough for Codex to inspect current state, repair drift, perform the next slice, validate it, and either continue looping or move to the next numbered plan.

## How To Use With `/goal`

Start with:

```text
/goal Work through docs/knowledge-next/goal-loop/00-loop-protocol.md, then continue to the next numbered plan only when its done criteria are met.
```

For a focused run:

```text
/goal Complete docs/knowledge-next/goal-loop/02-card-store-and-validation.md exactly as written, including checks and self-repair rules.
```

## Plan Order

1. [`00-loop-protocol.md`](00-loop-protocol.md) - shared operating rules for every loop. Start here.
2. [`01-source-base.md`](01-source-base.md) - keep the public source inventory clean and usable.
3. [`02-card-store-and-validation.md`](02-card-store-and-validation.md) - create the typed card/asset storage and validation path.
4. [`03-analysis-modeling-cards.md`](03-analysis-modeling-cards.md) - seed fundamentals, supports, loads, and diagnostics cards.
5. [`04-steel-system-cards.md`](04-steel-system-cards.md) - seed steel behavior and portal-frame system cards.
6. [`05-assets-and-diagrams.md`](05-assets-and-diagrams.md) - create first-class asset records and generated-safe diagram targets.
7. [`06-retrieval-evals-and-views.md`](06-retrieval-evals-and-views.md) - add retrieval eval seeds and generated markdown views.
8. [`07-cutover-and-archive.md`](07-cutover-and-archive.md) - compare coverage and plan the old wiki cutover/archive.

## Current Truth Sources

- Active high-level plan: [`../rebuild-plan.md`](../rebuild-plan.md)
- Source inventory: [`../source-inventory.md`](../source-inventory.md)
- Internal breadcrumb trace: [`../internal-source-trace.md`](../internal-source-trace.md)
- Existing operational wiki: [`../../knowledge/README.md`](../../knowledge/README.md)

## Loop Discipline

- Do not use private/local sources in the first rebuild pass.
- Do not migrate wiki prose directly.
- Do not delete or archive the existing wiki until plan 07 explicitly allows it.
- If a plan finds stale assumptions, repair that plan or its prerequisite docs before proceeding.
- Always run the checks listed in the current numbered plan before marking it done.
