---
title: Load Cases and Load Combinations
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

# Load Cases and Load Combinations

## Summary

A load case groups loads/actions; a load combination applies factors or accompanying-value logic to cases for strength, serviceability, and other checks. Fraia should model authored load cases, generated/manual combinations, and run results separately.

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

- SAF describes load cases as containers for loads from action sources [S1].
- Open-source tools model combinations as factor maps over named cases [S2][S3].
- Strength/serviceability and ULS/SLS concepts are code-scoped, not universal equations [S4][S5].
- ASCE and Eurocode sources show code edition and jurisdiction metadata are essential [S4][S5].

## Open questions / weak evidence

- Exact Fraia data schemas and validation algorithms remain future implementation work.
- Jurisdiction-specific code templates require separate review.

## Related pages

- [Knowledge topic map](../../topic-map.md)
- [Raw research note](../../raw/loads-load-cases-combinations-research.md)

## Sources

- [S1] SAF Documentation, *StructuralLoadCase*. URL: https://www.saf.guide/en/stable/loads/structuralloadcase.html. Source type: public schema documentation. Retrieved: 2026-05-06. Reliability/limits: exchange-format oriented.
- [S2] PyNite Wiki, *Load Cases & Load Combinations*. URL: https://github.com/JWock82/Pynite/wiki/5.-Load-Cases-&-Load-Combinations. Source type: open-source software documentation. Retrieved: 2026-05-06. Reliability/limits: software representation, not design standard.
- [S3] anaStruct Documentation, *Load cases and load combinations*. URL: https://anastruct.readthedocs.io/en/latest/loadcases.html. Source type: open-source software documentation. Retrieved: 2026-05-06. Reliability/limits: linear-analysis tool context.
- [S4] ASCE, *ASCE/SEI 7-22 overview*. URL: https://www.asce.org/publications-and-news/codes-and-standards/asce-sei-7-22. Source type: official public standards overview. Retrieved: 2026-05-06. Reliability/limits: overview only, standard text not reproduced.
- [S5] European Commission/JRC, *Eurocode: Basis of structural design*. URL: https://eurocodes.jrc.ec.europa.eu/EN-Eurocodes/eurocode-basis-structural-design. Source type: official public Eurocode overview. Retrieved: 2026-05-06. Reliability/limits: overview, national annexes not included.
