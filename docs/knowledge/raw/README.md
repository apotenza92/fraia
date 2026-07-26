# Legacy Raw / Source Notes

_Status: legacy/exceptional v0.2_
_Date: 2026-05-06_

`raw/` is no longer the default destination for source extraction.

Future PDF text, OCR output, webpage dumps, screenshots, contact sheets, chunk-reader notes, and reducer notes should live temporarily in `/tmp/fraia-knowledge/` or gitignored `docs/knowledge/.staging/` during an explicit ingestion run.

Existing files in this directory are kept as legacy provenance from early wiki seeding. New files should be added here only when they are compact agent-authored source notes, not copied source content.

Rules:

- Do not treat raw notes as compiled Fraia guidance.
- Do not store copied textbook/source prose or large extracts here.
- Preserve source URLs/paths, consulted/retrieved dates, source type, and reliability limits if a compact note is kept.
- Compiled pages must cite original sources in their own `## Sources` sections; links back to raw notes are optional, not required.

See [`../ingestion.md`](../ingestion.md) for the normal temporary ingestion workflow.
