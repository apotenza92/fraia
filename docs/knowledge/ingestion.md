# Fraia Knowledge Ingestion Workflow

_Status: active v0.2_
_Date: 2026-05-07_

This is optional maintainer/adapter guidance for reading external sources and updating Fraia's compiled knowledge wiki.

It is **not** shipped app/runtime scope. Fraia should ship the compiled wiki and the contract for updating it; PDF/OCR/web/image ingestion can be handled by maintainer scripts, subagents, or third-party/open-source tools that produce the [`adapter-contract.md`](adapter-contract.md) shapes.

## Durable vs temporary state

Durable repo state:

- compiled wiki pages under [`wiki/`](wiki/)
- per-page `## Sources` entries on compiled pages
- optional bibliographic registry in [`sources.md`](sources.md)
- agent-created/open-license media under [`media/`](media/) with manifest metadata
- proposals under [`proposals/`](proposals/)

Temporary state:

- PDF text extraction
- OCR output
- website readable-text dumps
- screenshots, contact sheets, page thumbnails, and figure crops
- chunk-reader notes and reducer notes
- source-section manifests generated for one ingestion run

Temporary state belongs in `/tmp/fraia-knowledge/` by default. Repo-local staging at `docs/knowledge/.staging/` is allowed only for multi-step local workflows that need stable files during a session. `.staging/`, `.cache/`, private media, and local source inventories are gitignored and must not be committed.

## Source ingestion pipeline

This pipeline is one possible adapter implementation, not a required Fraia-owned toolchain. Any external system may be used if it produces source learning packets or wiki update proposals with original-source provenance.

Use a map-reduce style workflow. Do not give an LLM a whole textbook, large manual, or large website crawl.

1. **Source scout**
   - Identify source sections relevant to one wiki topic.
   - Produce a small chunk manifest, using [`templates/chunk-manifest.md`](templates/chunk-manifest.md) as the shape.
   - Record source id, title, author/organization, URL or logical local path, page/section range, and why the chunk matters.

2. **Temporary extraction**
   - Extract only the selected pages/sections into staging.
   - Preserve page numbers, headings, figure numbers/captions, and extraction quality notes.
   - For visual sources, create staging-only screenshots/contact sheets/crops as needed.

3. **Chunk reader agents**
   - One reader handles one bounded chunk or small group of related chunks.
   - Output compact learnings only: definitions, source-backed claims, cautions, Fraia implications, page/figure references, and target wiki pages.
   - Do not output long excerpts or copied prose.

4. **Source reducer**
   - Merge chunk findings from one source/topic.
   - Remove duplication, flag contradictions, and preserve page/section/figure references.
   - Keep reducer notes in staging unless explicitly approved as compact source notes.

5. **Cross-source synthesizer**
   - Compare source reductions across independent sources.
   - Decide what belongs in compiled wiki pages and what remains weak/open.
   - Preserve source IDs so page `## Sources` can point back to original sources.
   - Treat packets without original source references as research proposals, not accepted knowledge.

6. **Wiki writer**
   - Update or create compiled pages under `wiki/`.
   - Update each page's `## Sources` with original sources, not staging artifacts.
   - Add media only when permitted by the media policy.

7. **Reviewer, lint, and Steward gate**
   - Check scope, source quality, copied-text risk, media licensing, local links, source metadata, and no staging leakage.
   - Run `python3 scripts/lint-knowledge.py`.
   - Run Fraia Knowledge Steward review before treating the update as compiled guidance.

## Chunk sizing guidance

- PDF/textbook chunks: normally 5-15 pages, smaller for dense math or diagram-heavy sections.
- Chapter chunks: one heading/subheading at a time.
- Website chunks: one page section or one short page at a time.
- Image chunks: one figure or small group of related figures at a time.
- Topic batch: one focused topic and a small set of relevant chunks.

## PDF strategy

- Prefer embedded text extraction first (`pdftotext`, PyMuPDF, or equivalent).
- Preserve page numbers and headings.
- Detect poor extraction quality: empty pages, scrambled order, missing formulas, bad tables, or scanned pages.
- Use OCR only for targeted pages/sections when embedded text is inadequate.
- Generate page thumbnails/contact sheets only for scouting and keep them in staging.
- Use multimodal review for diagrams, load-path sketches, FE meshes, formulas, and tables when text extraction misses meaning.
- For private/copyrighted textbooks, do not commit extracted text, screenshots, or OCR output. Compiled pages should paraphrase and cite title/chapter/page.

## Web strategy

- Capture title, URL, retrieval date, organization/author, source type, and reliability/limits.
- Extract readable text only for selected sections.
- Screenshot or analyze images only when diagrams materially affect understanding.
- Do not copy website images into the wiki unless license/permission is clear.
- Prefer source-page links plus agent-created diagrams for durable wiki content.

## Media policy summary

Committed media should normally be Fraia-native diagrams created by agents from synthesized understanding, not copied source artwork.

Allowed by default:

- agent-created SVG/PNG schematics
- open-license/public-domain images with attribution and manifest metadata

Staging-only by default:

- private textbook screenshots
- PDF page crops/contact sheets
- website screenshots with unclear license
- OCR image outputs

Every committed media file must be listed in [`media/manifest.md`](media/manifest.md) with source, license/status, and used-by references.

## Adapter output requirements

External/third-party ingestion outputs should follow [`templates/source-learning-packet.md`](templates/source-learning-packet.md). They must distinguish processor/tool metadata from original source metadata. Processor metadata helps reproduce a run; original sources support compiled wiki claims.

Compiled pages should cite original documents/webpages/manuals/textbooks, not adapter outputs or LLM summaries, whenever original sources are known.

## Source citation requirements

Every compiled page source entry must include:

- `[S#]` id
- author/organization and title
- `URL:`, `Path:`, or `Local source:`
- `Retrieved:` for web/public retrieval or `Consulted:` for local/private references
- `Source type:`
- `Reliability/limits:`

Private local sources should use logical locators such as `OneDrive-Personal/Engineering/Theory/...`, not absolute `/Users/...` paths.

## Relationship to raw notes

[`raw/`](raw/) is legacy/exceptional. Existing raw notes are kept for provenance from early wiki seeding, but future extraction should use temporary staging. A new permanent raw note is allowed only when it is a compact agent-authored source note, not copied source content.
