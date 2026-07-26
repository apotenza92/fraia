---
title: Gravity and Lateral Loads
status: compiled
trust_level: compiled
domain: loads
applies_to:
  - concept-stage structural modeling
  - Fraia agent guidance
not_applicable_to:
  - code-compliant design checks
  - project-specific engineering approval
jurisdiction_or_standard_context: concept guidance from public/open sources; not a code check
last_compiled: 2026-05-06
source_count: 5
citation_policy: required
owner: agent-maintained
---

# Gravity and Lateral Loads

## Summary

Gravity and lateral loads are different action families with different sources, directions, load paths, and provenance needs. Fraia should avoid treating all loads as generic arrows or design-ready constants.

## Scope / non-scope

Covers concept-level guidance for Fraia agents. It is not a code-design page, not a project approval, and not a replacement for validated analysis/check modules.

## Key concepts

- Keep authored intent, resolved analysis assumptions, run artifacts, and downstream checks separate.
- Store source/provenance metadata for assumptions.
- Prefer explicit local/global frames, affected objects, and load cases over hidden defaults.

## Engineering guidance for Fraia agents

- Use this page to ask better questions and avoid shallow defaults.
- When generating schemes or diagnostics, state assumptions and cite source-scoped concepts.
- Do not turn simplified examples into universal rules.

## Tradeoffs / cautions

- Most public sources are conceptual, educational, or software/vendor guidance.
- Code-dependent values and formulas require licensed/current standards and jurisdiction metadata.
- Fraia should surface uncertainty rather than silently choose engineering assumptions.

## Source-backed claims

- ASCE public scope lists many hazards/actions and load combinations as coordinated criteria [S1].
- Dead/permanent and variable loads differ in persistence and combination behavior [S2].
- Snow, wind, and seismic demands depend strongly on site, geometry, exposure, risk, and code edition [S1][S3][S5].
- Wind and seismic require continuous load paths and lateral-system identity [S4][S5].

## Open questions / weak evidence

- Exact Fraia data schemas and validation algorithms remain future implementation work.
- Jurisdiction-specific code templates require separate review.

## Related pages

- [Knowledge topic map](../../topic-map.md)
- [Raw research note](../../raw/loads-gravity-lateral-loads-research.md)

## Sources

- [S1] ASCE, *ASCE/SEI 7-22 overview*. URL: https://www.asce.org/publications-and-news/codes-and-standards/asce-sei-7-22. Source type: official public standards overview. Retrieved: 2026-05-06. Reliability/limits: overview only.
- [S2] ICC / David A. Fanella, *Structural Load Determination sample*. URL: https://shop.iccsafe.org/media/wysiwyg/material/4034S18-Sample.pdf. Source type: public sample. Retrieved: 2026-05-06. Reliability/limits: ASCE/IBC educational excerpt, not universal.
- [S3] FEMA, *FEMA P-957 Snow Load Safety Guide*. URL: https://www.fema.gov/sites/default/files/documents/fema957_snowload_guide.pdf. Source type: public agency guide. Retrieved: 2026-05-06. Reliability/limits: snow-focused.
- [S4] Building America Solution Center / PNNL, *Continuous Load Path*. URL: https://basc.pnnl.gov/resource-guides/continuous-load-path-provided-connections-roof-through-wall-foundation. Source type: public agency guidance. Retrieved: 2026-05-06. Reliability/limits: residential/hazard focused.
- [S5] FEMA, *FEMA P-749 Earthquake-Resistant Design Concepts*. URL: https://www.fema.gov/sites/default/files/2020-07/fema_earthquake-resistant-design-concepts_p-749.pdf. Source type: public agency guide. Retrieved: 2026-05-06. Reliability/limits: concept guide, not project design.
