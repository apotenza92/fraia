# Knowledge Proposals

_Status: active v0.1_
_Date: 2026-05-06_

This directory is the inbox for agent-discovered missing or weak knowledge.

## Rule

Agents may create proposal files automatically, but ordinary project/scheme agents must not update compiled wiki pages directly or create permanent raw extraction dumps. Compiled-page changes belong in an explicit wiki-maintenance/adapter run using [`../adapter-contract.md`](../adapter-contract.md) and [`../ingestion.md`](../ingestion.md).

Use this directory for lightweight gap proposals. For source-derived learnings use [`../templates/source-learning-packet.md`](../templates/source-learning-packet.md). For concrete page edits use [`../templates/wiki-update-proposal.md`](../templates/wiki-update-proposal.md).

## Proposal template

```md
# Proposal: <topic>

status: proposed
created: YYYY-MM-DD
trigger: <what task exposed this gap>
priority: high | medium | low
intended_page: docs/knowledge/wiki/<namespace>/<slug>.md

## Missing knowledge

## Why it matters to Fraia

## Suggested sources

List source candidates only. Do not paste copied source text here.

## Related pages

## Notes / cautions
```
