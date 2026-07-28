---
title: Load Paths
status: compiled
trust_level: compiled
domain: analysis
applies_to:
  - concept-stage structural modeling
  - Fraia agent guidance
not_applicable_to:
  - code-compliant design checks
  - project-specific engineering approval
jurisdiction_or_standard_context: concept guidance from public/open sources; not a code check
last_compiled: 2026-05-06
source_count: 4
citation_policy: required
owner: agent-maintained
---

# Load Paths

## Summary

A load path is the route by which gravity, lateral, and other actions pass through structural objects, connections, supports, foundations, and soil. Fraia should treat load-path traces as derived, inspectable guidance rather than authored truth.

## Scope / non-scope

Covers concept-level gravity/lateral load tracing and Fraia modeling implications. It does not give code design procedures, capacity checks, or project-specific load takedown approval.

## Key concepts

- Gravity paths often proceed from plates/surfaces to secondary members, primary members, vertical supports, foundations, and soil.
- Lateral paths need diaphragms, collectors, bracing/walls/frames, anchorage, and foundations.
- Connections, releases, discontinuities, and construction stages can control the real path.
- Tributary areas are useful preliminary abstractions but are not a substitute for analysis where stiffness-dependent distribution matters.

## Engineering guidance for Fraia agents

- Ask what path each significant load has to ground before proposing a scheme.
- Keep authored loads on `Node`, `Member`, `Plate`, or support objects separate from derived load-transfer artifacts.
- Surface missing collectors, unsupported plate edges, disconnected members, transfer conditions, and missing foundation/support assumptions.
- Preserve provenance from load case to structural object to run artifact.

## Tradeoffs / cautions

- Do not present one simple top-down gravity path as universal.
- Do not hide lateral load transfer behind decorative geometry.
- Tributary-area estimates should be labeled preliminary and assumption-based.

## Source-backed claims

- A load path is the route loads follow through a structure to ground/soil [S1][S2].
- Load tracing is system-level; individual member adequacy does not prove global adequacy [S2].
- Connections and discontinuities are common weak or rerouting points in load paths [S2][S3].
- Continuous load paths are especially important for wind/seismic uplift and shear [S3].

## Open questions / weak evidence

- How much automated load-path graph extraction should Fraia attempt in MVP remains open.
- Foundation/soil representation may start as support metadata before becoming a richer authored model.

## Related pages

- [Load cases and combinations](../loads/load-cases-and-combinations.md)
- [Gravity and lateral loads](../loads/gravity-and-lateral-loads.md)
- [Bracing principles](../stability/bracing-principles.md)
- [Supports, restraints, and releases](../modeling/supports-restraints-and-releases.md)
- [Raw research note](../../raw/analysis-load-paths-research.md)

## Sources

- [S1] TU Delft OpenCourseWare, *Introduction to Load Paths*. URL: https://ocw.tudelft.nl/course-readings/4-2-1-introduction-to-load-paths/. Source type: university OCW. Retrieved: 2026-05-06. Reliability/limits: concise concept source, not building-code guidance.
- [S2] Institution of Civil Engineers, *Explainer: Structural load paths*. URL: https://knowledgehub.ice.org.uk/cpd/delivery-exc/structural-load-paths/. Source type: professional engineering explainer. Retrieved: 2026-05-06. Reliability/limits: qualitative guidance, not a standard.
- [S3] Building America Solution Center / PNNL, *Continuous Load Path Provided with Connections from the Roof through the Wall to the Foundation*. URL: https://basc.pnnl.gov/resource-guides/continuous-load-path-provided-connections-roof-through-wall-foundation. Source type: public agency guidance. Retrieved: 2026-05-06. Reliability/limits: residential/hazard-focused but useful for continuity concepts.
- [S4] Pressbooks, *Structural Systems and Load Tracing*. URL: https://saalck.pressbooks.pub/structuralconceptsforarchitectsandconstructionmanagers/chapter/module-4-structural-systems-and-load-tracing/. Source type: open educational resource. Retrieved: 2026-05-06. Reliability/limits: simplified teaching examples.
