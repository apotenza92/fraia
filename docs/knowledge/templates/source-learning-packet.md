# Source Learning Packet Template

Use this template for learnings produced by a maintainer, subagent, or third-party ingestion system. Do not paste long source excerpts. Preserve original source references.

```yaml
packet_id:
topic:
created: YYYY-MM-DD
created_by:
status: proposed

processor:
  tool_name:
  tool_version:
  run_id:
  notes:
```

Processor/tool metadata is for traceability only. It is not the evidence source for compiled wiki claims.

## Original sources

Each source must be an original document/webpage/reference, not the ingestion tool output.

```yaml
original_sources:
  - source_ref: S1
    title:
    author_or_organization:
    source_locator: URL | DOI | Local source | bibliographic locator
    source_type: preferred public | strong private reference | practical/software reference | discovery only | other
    consulted_or_retrieved: YYYY-MM-DD
    page_section_figure_range:
    reliability_limits:
    license_or_usage_notes:
```

Packets without original-source references are research proposals, not accepted knowledge.

## Scope read

- Source sections/pages/figures actually read:
- Topic focus:
- Non-scope:

## Candidate wiki targets

- `docs/knowledge/wiki/...`

## Extracted learnings

For each learning:

```yaml
- id: L1
  claim_or_principle:
  source_refs:
    - S1 p. 12
  confidence: high | medium | low
  applicability:
  limits_cautions:
  suggested_fraia_vocabulary:
  candidate_wiki_targets:
```

## Software-specific filtering

Required when sources are software manuals/tutorials.

### Generic principles

- Learning id(s):
- Principle:
- Original source refs:

### Software conventions/examples

- Learning id(s):
- Convention/example:
- Why it is software-scoped:

### Workflow inspiration for Fraia

- Learning id(s):
- Possible Fraia diagnostic/UX/modeling implication:
- How to translate into Fraia vocabulary:

### Software-only details to exclude

- Menu paths/click steps/proprietary examples/screenshots/details excluded:

## Suggested wiki edits

- Target page:
- Add/change/remove:
- Claim ids supporting the edit:

## Wiki update proposal handoff

- Wiki update proposal path or PR section:
- Claims not ready for compiled guidance:
- Expected Steward risks:
  - Fraia product relevance:
  - authored/resolved/run boundary:
  - structural vocabulary:
  - vendor/software leakage:
  - source/confidence:

## Suggested source entries

Draft `## Sources` entries for target pages. Cite original sources, not this packet or processor tool.

## Open questions / conflicts

- Weak evidence:
- Conflicting sources:
- Needs reviewer/oracle decision:

## Contributor confirmation

- [ ] I did not include long raw excerpts or copied source prose.
- [ ] I did not include private/copyrighted screenshots or copied figures.
- [ ] Each durable claim has an original source reference or is marked as an open question.
- [ ] Software-specific details are classified and not rewritten as generic Fraia guidance.
- [ ] I identified claims or risks for Fraia Knowledge Steward review.
