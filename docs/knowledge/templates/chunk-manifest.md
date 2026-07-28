# Knowledge Ingestion Chunk Manifest Template

_Status: template v0.1_

Use this template for one source/topic ingestion run. Store filled manifests in `/tmp/fraia-knowledge/` or `docs/knowledge/.staging/`, not in committed docs, unless explicitly approved as process evidence.

## Topic

- Target wiki page(s):
- Question to answer:
- Non-goals:

## Source

- Source id:
- Author/organization:
- Title:
- Edition/version/date:
- URL / Path / Local source:
- Retrieved / Consulted:
- Source type:
- Reliability/limits:
- License/private-source notes:

## Chunks

| Chunk id | Pages/section/figure | Extraction mode | Why this chunk matters | Assigned reader | Status |
| --- | --- | --- | --- | --- | --- |
| C1 | pp. / section | text / OCR / screenshot / multimodal | | | planned |

## Chunk reader output contract

For each chunk, return only compact learnings:

- key concepts or definitions
- source-backed claims with page/section/figure references
- diagrams/images that need multimodal interpretation
- Fraia implications
- cautions, contradictions, weak evidence
- suggested wiki target page/section

Do not return long excerpts or copied source prose.

## Reducer checklist

- [ ] Duplicate findings merged.
- [ ] Contradictions flagged.
- [ ] Page/section/figure references preserved.
- [ ] No copied prose retained.
- [ ] Candidate page source entries drafted from original source metadata.
