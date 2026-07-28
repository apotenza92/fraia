# Wiki Update Proposal Template

Use this for proposed edits to compiled Fraia knowledge pages. It can accompany a PR, proposal, or source learning packet.

## Summary

- Proposed change:
- Why it matters to Fraia:
- Trigger/request:

## Target pages

- `docs/knowledge/wiki/...`

## Claim-to-source map

Every non-trivial engineering claim should point to an original source entry.

| Claim/change | Source ref(s) | Scope/limits |
| --- | --- | --- |
|  |  |  |

## Source entries to add/update

Draft page-level source entries. Cite original sources, not ingestion tools or source learning packets.

```md
- [S#] Author/organization, *Title*, page/section if applicable. URL: ... or Local source: ... Retrieved/Consulted: YYYY-MM-DD. Source type: ... Reliability/limits: ...
```

## Proposed page edits

- Add:
- Change:
- Remove:
- Related pages/cross-links:

## Software-source filtering

If any source is software documentation/manual/tutorial:

- Generic principles included:
- Workflow inspiration translated into Fraia vocabulary:
- Software conventions kept source-scoped:
- Software-only details excluded:

## Media

- New media files:
- License/status:
- `media/manifest.md` update required: yes | no
- Confirmation no private/copyrighted screenshots or copied figures are committed:

## Status/trust recommendation

- Recommended `status`: draft | compiled | needs-review
- Recommended `trust_level`: raw | compiled | reviewed
- Reason:

## Lint/reviewer result

- `python3 scripts/lint-knowledge.py`: not run | pass | fail
- `python3 scripts/build-knowledge-viewer.py`: not run | pass | fail
- `python3 scripts/review-knowledge-steward.py --evidence <this-file> --require-checklist`: not run | pass | fail
- Reviewer:
- Reviewer finding summary:
- Required reviewer edits:

## Fraia Knowledge Steward review

The Steward review should happen after lint/reviewer checks and before treating the update as compiled guidance.

### Steward checklist

- [ ] Fraia product relevance is clear.
- [ ] Architecture fit is preserved; heavy ingestion remains maintainer/community/adapter side.
- [ ] Authored structural state, resolved/realization state, and immutable run artifacts remain distinct.
- [ ] Structural vocabulary is correct, including `role` for authored structural objects and `element` for analysis discretisation.
- [ ] Vendor/software-specific material is source-scoped or distilled into generic Fraia principles.
- [ ] Source/confidence risks are stated, including one-source, private-source, weak-source, or conflicting-source limits.
- [ ] Decision state is appropriate.

### Steward decision

- Reviewer/steward:
- Decision: accept | accept-with-edits | needs-more-source | downgrade-to-draft | veto
- Required edits or source requests:
- Re-review required after edits: yes | no
- Notes:

## Validation checklist

- [ ] Page-level `## Sources` updated.
- [ ] `source_count` matches `[S#]` entries.
- [ ] Local links resolve.
- [ ] Page is listed in `docs/knowledge/index.md` if compiled/new.
- [ ] Page appears in `docs/knowledge/topic-map.md` if compiled/new.
- [ ] No `trust_level: canonical`.
- [ ] No copied source prose or raw dumps.
- [ ] `python3 scripts/lint-knowledge.py` passes.
- [ ] `python3 scripts/build-knowledge-viewer.py` run if navigation/viewer docs changed.
- [ ] Fraia Knowledge Steward decision recorded.
- [ ] `python3 scripts/review-knowledge-steward.py --evidence <proposal-or-log.md> --require-checklist` passes for compiled-promotion evidence.
