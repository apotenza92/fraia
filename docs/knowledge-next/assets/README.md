# Knowledge Assets

_Status: rebuild storage_

This directory stores typed `KnowledgeAsset` JSON records for diagrams, figures, image references, and generated-safe visual targets.

Rules:

- One asset per `.json` file.
- Asset ids use `KA-*`.
- Asset records are metadata and generation targets, not copied source media.
- Original source figures should normally use `metadata_only` or `generated_derivative_only` unless an open license is clear.
- Do not commit private PDFs, screenshots, OCR dumps, crops, copied diagrams, or copied source images.
- Generated/open diagrams should be tracked separately from original source locators.

Validate with:

```sh
python3 scripts/validate-knowledge-next.py
```
