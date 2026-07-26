---
title: Instability Mechanisms
status: compiled
trust_level: compiled
domain: diagnostics
applies_to:
  - concept-stage structural modeling
  - Fraia agent guidance
not_applicable_to:
  - code-compliant design checks
  - project-specific engineering approval
jurisdiction_or_standard_context: concept guidance from public/open sources plus local software-manual examples; not a code check
last_compiled: 2026-05-06
source_count: 7
citation_policy: required
owner: agent-maintained
---

# Instability Mechanisms

## Summary

Solver instability is often the visible symptom of unconstrained DOFs, disconnected topology, over-released members, torsional mechanisms, nonlinear element activation, or true second-order/global instability.

## Scope / non-scope

Covers concept-level guidance for Fraia agents. It is not a code-design page, not a project approval, and not a replacement for validated analysis/check modules.

## Key concepts

- Keep authored intent, resolved analysis assumptions, run artifacts, and downstream checks separate.
- Store source/provenance metadata for assumptions.
- Prefer explicit local/global frames, affected objects, and load cases over hidden defaults.
- Map instability causes back to authored `SupportAssignment`, `ReleaseAssignment`, connectivity, material/section assignments, and resolved analysis topology; do not silently fix the authored model without an explicit adoption step.

## Engineering guidance for Fraia agents

- Use this page to ask better questions and avoid shallow defaults.
- When generating schemes or diagnostics, state assumptions and cite source-scoped concepts.
- Do not turn simplified examples into universal rules.

## Tradeoffs / cautions

- Most public sources are conceptual, educational, or software/vendor guidance.
- Code-dependent values and formulas require licensed/current standards and jurisdiction metadata.
- Fraia should surface uncertainty rather than silently choose engineering assumptions.
- The Strand7 source is a local software-manual example; keep derived guidance source-scoped and seek public/solver-neutral corroboration before upgrading trust.

## Source-backed claims

- Singular stiffness commonly indicates one or more DOFs can move without resistance [S1][S2].
- Rigid-body motion and matrix singularity are related diagnostics but should not be treated as identical; insufficient restraint is one shared cause, while property errors, over-releases, nonlinear deactivation, missing alternative load paths, and distorted elements can also produce singularities [S1][S7].
- Global rigid-body motion should be distinguished from local nodal mechanisms caused by releases or disconnected members [S1][S3].
- Connectivity errors, coincident unmerged nodes, and crossing unconnected members are common instability sources [S4][S5].
- Unstable-mode visualization, node/DOF reports, and simple load-case checks help localize mechanisms [S2][S5][S6].
- When diagnostics include both localized object/property/node warnings and broad global-matrix symptoms, investigate the localized evidence first while preserving the full warning set [S7].
- Artificial singularity suppression or added stabilizing stiffness can be useful for troubleshooting, but should be surfaced as run diagnostics / result-quality metadata in the frozen run artifact rather than treated as a valid model fix [S3][S7].
- Large stiffness contrasts can create ill-conditioned systems; over-stiff analysis elements, links, pseudo-rigid modeling assumptions, extreme support stiffness values, tiny support stiffnesses, and incorrect material properties should be checked before trusting results [S6][S7].

## Open questions / weak evidence

- Exact Fraia data schemas and validation algorithms remain future implementation work.
- Jurisdiction-specific code templates require separate review.

## Related pages

- [Knowledge topic map](../../topic-map.md)
- [Unconnected or underrestrained models](unconnected-or-underrestrained-models.md)
- [Overreleased members and all-pin mechanisms](overreleased-members-and-all-pin-mechanisms.md)
- [Static determinacy and restraint](../analysis/static-determinacy-and-restraint.md)
- [Raw research note](../../raw/diagnostics-instability-mechanisms-research.md)

## Sources

- [S1] PyNite Documentation, *Stability*. URL: https://pynite.readthedocs.io/en/latest/stability.html. Source type: open-source software documentation. Retrieved: 2026-05-06. Reliability/limits: software-specific but clear taxonomy.
- [S2] SCIA, *Warning: stiffness matrix is singular*. URL: https://scia.net/en/support/faq/scia-engineer/analysis/warning-stiffness-matrix-singular-structure-unstable. Source type: public software help. Retrieved: 2026-05-06. Reliability/limits: product workflow.
- [S3] RISA-3D Help, *Stability*. URL: https://help.risa.com/risahelp/risa3d/Content/Stability/Stability.htm. Source type: public software help. Retrieved: 2026-05-06. Reliability/limits: product examples.
- [S4] LUSAS, *Connectivity FAQ*. URL: https://www.lusas.com/user_area/faqs/connectivity.html. Source type: public software FAQ. Retrieved: 2026-05-06. Reliability/limits: FE connectivity focused.
- [S5] Dlubal, *Finding and Fixing Calculation Instabilities*. URL: https://www.dlubal.com/en/support-and-learning/support/faq/005345. Source type: public software FAQ. Retrieved: 2026-05-06. Reliability/limits: product examples.
- [S6] Oasys GSA, *Model debugging*. URL: https://docs.oasys-software.com/structural/gsa/version/10.2.12/tutorials/model-debugging/. Source type: public software documentation. Retrieved: 2026-05-06. Reliability/limits: product workflow.
- [S7] Strand7 Pty Limited, *ST7-1.10.10.2 Rigid Body Modes and Singularity Warning in Static Solvers*. Local source: `local-source-alias:strand7/ST7-1.10.10.2-rigid-body-modes-and-singularity-warning-in-static-solvers.pdf`. Source type: software manual/tutorial. Consulted: 2026-05-06. Reliability/limits: practical vendor guidance for static-solver diagnostics; useful for generic instability, singularity, release, property, and ill-conditioning patterns, but warning IDs, UI steps, element-specific handling, and numeric thresholds are Strand7-specific.
