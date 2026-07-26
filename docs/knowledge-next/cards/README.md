# Knowledge Cards

_Status: rebuild storage_

This directory stores typed `KnowledgeCard` JSON records.

Rules:

- One card per `.json` file.
- Card ids use `KC-*`.
- Cards are source truth for the rebuilt knowledge base.
- Generated markdown/wiki pages are renderers of cards, not hand-maintained truth.
- First-pass cards must use public rebuild-eligible sources from `../source-inventory.json`.
- Do not copy prose from the old wiki or source material into cards.
- Do not use private/local sources unless the rebuild policy changes explicitly.

Validate with:

```sh
python3 scripts/validate-knowledge-next.py
```
