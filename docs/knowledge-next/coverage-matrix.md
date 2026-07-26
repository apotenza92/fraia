# Knowledge-Next Coverage Matrix

_Status: cutover planning artifact_
_Date: 2026-06-15_

This matrix compares the current operational wiki under `docs/knowledge/wiki/` with the typed cards, assets, evals, and generated views in `docs/knowledge-next/`.

No old wiki page is retired by this matrix. `docs/knowledge/` remains operational until a separate user-approved cutover/archive action exists.

## Status Legend

- `covered`: core concept is represented by typed cards and generated views.
- `partial`: related cards exist, but the old page still contains useful coverage or needs more source-backed cards.
- `not covered`: keep the old page operational; create replacement cards before any cutover.
- `index/support`: navigation or log page; keep until generated views replace navigation.

## Generated Views

- Cards: `docs/knowledge-next/generated/views/cards-index.md`
- Assets: `docs/knowledge-next/generated/views/assets-index.md`
- Evals: `docs/knowledge-next/generated/views/evals-index.md`

## Matrix

| Old wiki page | Status | New card/view coverage | Cutover action |
| --- | --- | --- | --- |
| `README.md` | index/support | Generated views exist, but old wiki overview is still operational. | Keep. Replace only after generated navigation is reviewed. |
| `analysis/index.md` | index/support | `generated/views/cards-index.md` covers typed card discovery. | Keep until generated views become the agent-facing index. |
| `analysis/free-body-diagrams-and-equilibrium.md` | covered | `KC-free-body-equilibrium`; `KA-free-body-diagram-components`. | Candidate for archive after card review. |
| `analysis/load-paths.md` | covered | `KC-load-paths`; `KA-portal-frame-load-path`. | Candidate for archive after card review. |
| `analysis/reactions-and-support-idealisation.md` | covered | `KC-support-reactions-idealisation`; `KC-reaction-sanity-checks`; `KA-support-dof-reaction-symbols`. | Candidate for archive after card review. |
| `analysis/static-determinacy-and-restraint.md` | covered | `KC-determinacy-restraint-mechanisms`; `KC-instability-diagnostics`. | Candidate for archive after card review. |
| `analysis/beam-shear-and-moment-diagrams.md` | partial | `KC-steel-bending-members` covers bending/shear concepts, but no dedicated shear/moment diagram card exists. | Keep; add diagram/result-interpretation card. |
| `analysis/matrix-stiffness-method.md` | partial | `KC-load-application-equivalent-loads` and `KC-authored-member-analysis-element-separation` touch stiffness/element realization. | Keep; add direct-stiffness/matrix-solver card if needed. |
| `analysis/second-order-effects-and-stability.md` | partial | `KC-instability-diagnostics`; `KC-member-restraint-and-unbraced-length`; `KC-steel-compression-members`. | Keep; add second-order/P-delta card. |
| `analysis/truss-analysis-and-two-force-members.md` | partial | `KC-determinacy-restraint-mechanisms` includes truss idealisation, but no dedicated truss card exists. | Keep; add truss/two-force member card. |
| `diagnostics/index.md` | index/support | `generated/views/cards-index.md` covers typed diagnostics discovery. | Keep until generated views become the agent-facing index. |
| `diagnostics/reaction-sanity-checks.md` | covered | `KC-reaction-sanity-checks`; `KE-suspicious-reactions`. | Candidate for archive after card/eval review. |
| `diagnostics/instability-mechanisms.md` | covered | `KC-instability-diagnostics`; `KC-determinacy-restraint-mechanisms`. | Candidate for archive after card review. |
| `diagnostics/analysis-result-review-before-design-checks.md` | covered | `KC-steel-design-action-check-input-separation`; `KE-solver-result-vs-design-action-check`. | Candidate for archive after card/eval review. |
| `diagnostics/overreleased-members-and-all-pin-mechanisms.md` | partial | `KC-member-end-releases`; `KC-instability-diagnostics`. | Keep; add overrelease/all-pin mechanism card if needed. |
| `diagnostics/unconnected-or-underrestrained-models.md` | partial | `KC-instability-diagnostics`; `KC-member-restraint-and-unbraced-length`. | Keep; add connectivity/underrestraint diagnostic card if needed. |
| `loads/index.md` | index/support | `generated/views/cards-index.md` covers typed loads discovery. | Keep until generated views become the agent-facing index. |
| `loads/load-application-and-equivalent-nodal-loads.md` | covered | `KC-load-application-equivalent-loads`; `KA-distributed-load-equivalent-resultant`. | Candidate for archive after card review. |
| `loads/area-line-point-and-member-loads.md` | partial | `KC-load-application-equivalent-loads` covers load realization, not all authored load families. | Keep; add authored load taxonomy card. |
| `loads/gravity-and-lateral-loads.md` | partial | `KC-load-paths`; `KC-portal-frame-longitudinal-transverse-stability`. | Keep; add gravity/lateral load taxonomy card. |
| `loads/load-cases-and-combinations.md` | not covered | No dedicated load case/combination card exists. | Keep; add public-source-backed load cases/combinations card. |
| `materials/index.md` | index/support | Generated views cover typed material cards. | Keep until generated views become the agent-facing index. |
| `materials/steel/index.md` | index/support | Generated views cover typed steel cards. | Keep until generated views become the agent-facing index. |
| `materials/steel/material-properties-and-section-families.md` | covered | `KC-steel-material-and-section-families`. | Candidate for archive after card review. |
| `materials/steel/beams-and-bending-members.md` | covered | `KC-steel-bending-members`; `KC-lateral-torsional-buckling-concepts`. | Candidate for archive after card review. |
| `materials/steel/compression-members.md` | covered | `KC-steel-compression-members`; `KC-member-restraint-and-unbraced-length`. | Candidate for archive after card review. |
| `materials/steel/design-action-check-input-separation.md` | covered | `KC-steel-design-action-check-input-separation`; `KA-design-action-check-provenance-flow`. | Candidate for archive after card/eval review. |
| `materials/steel/member-behavior.md` | partial | Covered through several focused steel cards, but no single overview card mirrors the old summary. | Keep; generated views may replace overview after review. |
| `materials/steel/connections-concept-taxonomy.md` | partial | `KC-member-end-releases`; `KC-portal-frame-base-fixity-tradeoffs`; no full connection taxonomy card. | Keep; add connection taxonomy card. |
| `modeling/index.md` | index/support | Generated views cover typed modeling/product cards. | Keep until generated views become the agent-facing index. |
| `modeling/member-end-releases.md` | covered | `KC-member-end-releases`; `KA-member-release-components`. | Candidate for archive after card review. |
| `modeling/finite-element-idealisation.md` | covered | `KC-authored-member-analysis-element-separation`; `KA-local-global-member-axes`. | Candidate for archive after card review. |
| `modeling/supports-restraints-and-releases.md` | partial | `KC-support-reactions-idealisation`; `KC-member-end-releases`; `KC-member-restraint-and-unbraced-length`. | Keep; generated views may replace after review. |
| `modeling/connection-fixity-and-partial-restraint.md` | partial | `KC-member-end-releases`; `KC-portal-frame-base-fixity-tradeoffs`. | Keep; add connection fixity/partial restraint card. |
| `modeling/local-and-global-coordinate-systems.md` | partial | `KA-local-global-member-axes`; no dedicated coordinate-system card. | Keep; add local/global coordinate card if needed. |
| `modeling/constraints-rigid-links-and-diaphragms.md` | not covered | No constraints/rigid links/diaphragms card exists. | Keep; add modeling constraints card. |
| `product/index.md` | index/support | Generated views cover typed product cards. | Keep until generated views become the agent-facing index. |
| `product/design-actions-check-inputs-and-results.md` | covered | `KC-steel-design-action-check-input-separation`; `KE-solver-result-vs-design-action-check`. | Candidate for archive after card/eval review. |
| `product/authored-resolved-run-boundaries.md` | partial | `KC-authored-member-analysis-element-separation`; `KC-steel-design-action-check-input-separation`. | Keep; add broader authored/resolved/run artifact card. |
| `product/engineering-assumptions-and-provenance.md` | partial | `KC-steel-design-action-check-input-separation`; `KA-design-action-check-provenance-flow`. | Keep; add broader assumptions/provenance card. |
| `product/scheme-generation-from-knowledge.md` | partial | `KE-missing-context-before-scheme-generation`; `KC-load-paths`; portal-frame cards. | Keep; add scheme-generation policy card. |
| `product/structural-design-option-intelligence.md` | not covered | No design-option intelligence card exists. | Keep; add product/design-option card from public concept-stage sources and internal policy. |
| `stability/index.md` | index/support | Generated views cover typed stability cards. | Keep until generated views become the agent-facing index. |
| `stability/bracing-principles.md` | covered | `KC-steel-bracing-principles`; `KA-portal-frame-bracing-stability-axes`. | Candidate for archive after card review. |
| `stability/compression-member-buckling-concepts.md` | covered | `KC-steel-compression-members`; `KC-member-restraint-and-unbraced-length`. | Candidate for archive after card review. |
| `stability/lateral-torsional-buckling-concepts.md` | covered | `KC-lateral-torsional-buckling-concepts`; `KA-member-restraint-unbraced-length`. | Candidate for archive after card review. |
| `stability/member-restraint-and-unbraced-length.md` | covered | `KC-member-restraint-and-unbraced-length`; `KA-member-restraint-unbraced-length`. | Candidate for archive after card review. |
| `steel/index.md` | index/support | Generated views cover typed steel system cards. | Keep until generated views become the agent-facing index. |
| `steel/portal-frames/index.md` | index/support | Generated views cover typed portal-frame cards. | Keep until generated views become the agent-facing index. |
| `steel/portal-frames/system-overview.md` | covered | `KC-steel-portal-frame-system-overview`; `KA-portal-frame-load-path`. | Candidate for archive after card review. |
| `steel/portal-frames/base-fixity-tradeoffs.md` | covered | `KC-portal-frame-base-fixity-tradeoffs`; `KA-support-dof-reaction-symbols`. | Candidate for archive after card review. |
| `steel/portal-frames/bracing.md` | covered | `KC-steel-bracing-principles`; `KC-portal-frame-longitudinal-transverse-stability`. | Candidate for archive after card review. |
| `steel/portal-frames/longitudinal-vs-transverse-stability.md` | covered | `KC-portal-frame-longitudinal-transverse-stability`; `KA-portal-frame-bracing-stability-axes`. | Candidate for archive after card review. |
| `steel/portal-frames/purlins-girts-and-restraint.md` | covered | `KC-steel-portal-purlins-and-girts`; `KC-member-restraint-and-unbraced-length`. | Candidate for archive after card review. |
| `log.md` | index/support | Historical wiki log; not replaced by typed cards. | Keep as history unless explicitly archived later. |

## Coverage Summary

- Covered content pages: 24
- Partial content pages: 16
- Not covered content pages: 3
- Index/support pages: 12

## Next Backlog From Gaps

- Load cases and combinations.
- Constraints, rigid links, and diaphragms.
- Structural design-option intelligence.
- Beam shear/moment diagrams.
- Matrix/direct-stiffness method overview.
- Second-order effects and P-delta.
- Truss and two-force member idealisation.
- Authored load taxonomy.
- Gravity/lateral load taxonomy.
- Connection taxonomy and partial restraint.
- Local/global coordinate-system card.
- Broader product cards for authored/resolved/run boundaries, assumptions/provenance, and scheme-generation policy.
