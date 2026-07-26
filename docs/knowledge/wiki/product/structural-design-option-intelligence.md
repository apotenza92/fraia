---
title: Structural Design Option Intelligence
status: compiled
trust_level: compiled
domain: product
applies_to:
  - Fraia design-option generation
  - concept-stage structural scheme comparison
  - LLM-backed structural recommendations
not_applicable_to:
  - final member sizing
  - code-specific design checks
  - engineer-of-record approval
jurisdiction_or_standard_context: concept-stage structural design literature and Fraia product guidance; not a code check
last_compiled: 2026-05-13
source_count: 6
citation_policy: required
owner: agent-maintained
---

# Structural Design Option Intelligence

## Summary

Intelligent structural design options are not just different member sizes. They are different plausible ways of organizing load path, geometry, material, restraint, robustness, constructability, serviceability intent, cost/carbon direction, and project constraints.

For Fraia, design-option generation should behave like structured concept design: create a small set of coherent alternatives, explain what each option is testing, preserve assumptions, and keep exact sizing downstream until analysis/design/check artifacts exist.

## Scope / non-scope

This page covers concept-stage design-option philosophy for Fraia agents and LLM-backed scheme surfaces.

It does not provide span tables, code formulas, final sizing rules, cost databases, carbon factors, or jurisdiction-specific design requirements.

## Key concepts

### Concept design is exploration, not single-answer sizing

Conceptual structural design starts with incomplete information and must turn project intent, spatial constraints, loads, material possibilities, and construction constraints into candidate structural systems. Professional concept-design guidance emphasizes idea generation, divergent thinking, brief development, feasibility studies, and questions such as material and load alternatives. [S1]

Fraia should therefore generate options as an explicit comparison set, not as one hidden "best" answer.

### Options should test different structural hypotheses

A useful design option should have a hypothesis such as:

- shorter load path and more direct supports
- fewer member families and simpler procurement
- stiffer lateral system or base restraint
- simpler connections and erection sequence
- lower material mass or lower embodied-carbon direction
- improved redundancy, continuity, or robustness
- better fit with architectural/grid/opening constraints

Two options that differ only by arbitrary section IDs are weak concept options.

### Geometry and load path are high-leverage decisions

Early structural geometry can control efficiency and force magnitudes. Load-path theory, graphic statics, topology optimization, and shape finding all point to the importance of geometry before member sizing. [S2]

Fraia agents should explain whether an option changes the load path, the force flow, the support path to ground, or only the later sizing envelope.

### Integrated design means multiple criteria

Design is a decision-making exploration shaped by knowledge and philosophy. Integrated civil/structural design considers safety, durability, serviceability, sustainability, material strengths, structural form, and construction method together. [S3]

Fraia should surface tradeoffs rather than collapse them into a single weight-minimization objective.

### Multi-objective methods are design aids

Heuristic and multi-objective computation can support architecture/structure design across topology, shape, sizing, life-cycle cost, environmental impact, energy, construction cost, and aesthetic or spatial objectives. They should be treated as part of a design methodology, not merely a black-box optimizer. [S4]

Fraia should preserve why an option exists, what objective it explores, and which constraints it respects.

### Conceptual structural systems are hierarchical

Computer-aided conceptual structural design literature treats the central task as synthesis of the structural system, often using hierarchical or top-down refinement and interaction with architectural constraints. Relevant constraints include function, behavior, performance, reliability, material, cost, compatibility, and constructability. [S5]

Fraia should generate options at the correct level: whole-system form, subsystem/load-path strategy, coordination groups, then member/check details.

### Robustness is an option criterion

Robustness is not only a final abnormal-load check. Concept-stage options can differ in continuity, redundancy, alternate load paths, ductility, and connection resilience. Structural steel robustness literature describes alternate-load-path and indirect approaches based on strength, continuity, redundancy, ductility, and bridging damaged regions. [S6]

Fraia should flag options that are efficient but brittle, poorly continuous, or dependent on one unverified support/restraint assumption.

## Engineering guidance for Fraia agents

- Generate a small option set where each option tests a distinct structural hypothesis.
- Prefer option names that reveal intent: direct load path, standardised member set, stiffness-focused frame, simple connections, robust/redundant path, low-carbon/material-efficient direction.
- Keep options coherent: support assumptions, release/fixity assumptions, member families, bracing/restraint, and load paths should fit together.
- Avoid fake diversity: do not create multiple options that only swap exact section sizes or catalogue IDs.
- Explain the tradeoff axis for each option: weight/carbon, stiffness/serviceability, connection simplicity, erection/buildability, robustness, architectural compatibility, or analysis uncertainty.
- Preserve design uncertainty: identify which assumptions are user-confirmed, wiki-guided, inferred, or intentionally varied.
- Use coordination groups as the normal early-stage unit for section-family and size-intent discussion.
- Escalate to downstream analysis/design/check artifacts before claiming final adequacy.

## Tradeoffs / cautions

- A concept option can be structurally intelligent without being final-design adequate.
- A low-material option can be poor if it creates awkward connections, bad serviceability, weak robustness, or difficult construction.
- A highly regular option can be buildable but overconservative or architecturally intrusive.
- Optimization can miss non-numerical constraints unless function, constructability, compatibility, and project intent are represented.
- Robustness and redundancy can increase material or connection demand but reduce brittleness and hidden single-point dependencies.

## Source-backed claims

- Concept design involves idea generation, divergent thinking, brief development, feasibility studies, and asking material/load questions before final design. [S1]
- Early geometry and layout influence efficiency and the magnitude of forces that must be accommodated. [S2]
- Integrated structural design considers safety, durability, serviceability, sustainability, materials, structural form, and construction methodology together. [S3]
- Multi-objective heuristic computation can support structural and architectural design across many competing criteria and act as part of design methodology. [S4]
- Conceptual structural synthesis should consider function, behavior, performance, reliability, material, cost, compatibility, and constructability, and can be organized hierarchically. [S5]
- Robust structural options should consider alternate load paths, continuity, redundancy, ductility, and connection resilience. [S6]

## Open questions / weak evidence

- Fraia still needs source-scoped preliminary sizing heuristics and material/carbon data before options can rank embodied carbon numerically.
- Exact scoring weights for option comparison should remain project/user configurable.
- Robustness guidance here is concept-level; final progressive-collapse, accidental-action, seismic, fire, or abnormal-load checks require code- and project-specific modules.

## Related pages

- [Scheme generation from knowledge](scheme-generation-from-knowledge.md)
- [Engineering assumptions and provenance](engineering-assumptions-and-provenance.md)
- [Authored/resolved/run artifact boundaries](authored-resolved-run-boundaries.md)
- [Load paths](../analysis/load-paths.md)
- [Supports, restraints, and releases](../modeling/supports-restraints-and-releases.md)
- [Connection fixity and partial restraint modeling](../modeling/connection-fixity-and-partial-restraint.md)
- [Steel material properties and section families](../materials/steel/material-properties-and-section-families.md)

## Sources

- [S1] Institution of Structural Engineers, *Conceptual design of buildings*, version 1.1. URL: https://www.istructe.org/getattachment/4ef4c605-efe3-4a56-9c94-7be1295c8984/attachment.aspx. Source type: professional structural engineering guidance. Retrieved: 2026-05-13. Reliability/limits: strong professional concept-design guidance; copyrighted guide, used only for source-scoped concepts and not copied as design rules.
- [S2] Baker W. F., Beghini L. L., Mazurek A., Carrion J., and Beghini A., *Structural Innovation: Combining Classic Theories with New Technologies*, AISC Engineering Journal, 2015. URL: https://www.aisc.org/Structural-Innovation-Combining-Classic-Theories-with-New-Technologies. Source type: professional/academic steel structural engineering article. Retrieved: 2026-05-13. Reliability/limits: useful for geometry, load-path theory, topology optimization, and shape-finding concepts; not a Fraia schema or final sizing guide.
- [S3] Ming X., Huang J. C., and Li Z., *Materials-oriented integrated design and construction of structures in civil engineering: A review*, Frontiers of Structural and Civil Engineering, 2022. URL: https://link.springer.com/article/10.1007/s11709-021-0794-9. Source type: open-access academic review. Retrieved: 2026-05-13. Reliability/limits: broad integrated-design review; not specific to steel portal frames or Fraia workflows.
- [S4] Moreno-De-Luca L. and Begambre Carrillo O. J., *Multi-Objective Heuristic Computation Applied to Architectural and Structural Design: A Review*, International Journal of Architectural Computing, 2013. URL: https://journals.sagepub.com/doi/10.1260/1478-0771.11.4.363. Source type: academic review. Retrieved: 2026-05-13. Reliability/limits: useful survey of multi-objective methods; access-limited and not a deterministic design standard.
- [S5] Mora R., Bedard C., and Rivard H., *A Framework for Computer-Aided Conceptual Design of Building Structures*, 2004. DOI: 10.1007/978-1-4020-2393-4_3. URL: https://www.researchgate.net/publication/244955895_A_Framework_for_Computer-Aided_Conceptual_Design_of_Building_Structures. Source type: academic conference/chapter paper. Retrieved: 2026-05-13. Reliability/limits: useful conceptual-design framework; ResearchGate copy may be author-uploaded and content may be subject to copyright.
- [S6] AISC, *Robustness in Structural Steel Framing Systems*. URL: https://www.aisc.org/globalassets/aisc/research-library/robustness-in-structural-steel-framing-systems.pdf. Source type: professional/academic structural steel research report. Retrieved: 2026-05-13. Reliability/limits: useful robustness and alternate-load-path concepts for steel framing; final robustness design remains code- and project-specific.
