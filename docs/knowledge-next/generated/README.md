# Knowledge-Next Generated Views

Files under `generated/views/` are reproducible renderings of typed knowledge records. They are reader aids only.

Canonical truth remains:

- `docs/knowledge-next/source-inventory.json`
- `docs/knowledge-next/cards/**/*.json`
- `docs/knowledge-next/assets/**/*.json`
- `docs/knowledge-next/evals/**/*.json`

Regenerate views with:

```sh
python3 scripts/generate-knowledge-next-views.py
```

Do not hand-edit generated view output. Fix the underlying typed record or generator instead.
