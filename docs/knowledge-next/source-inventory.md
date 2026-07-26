# Fraia Knowledge Source Inventory

_Status: rebuild seed inventory_
_Generated: 2026-06-15_

This inventory is generated from the current `docs/knowledge/` wiki, raw notes, and source registry inputs. It is a rebuild seed, not a new source of engineering truth.

## Summary

- Compiled pages audited: 43
- Raw notes audited: 10
- Registry/knowledge plan files audited: 5
- Unique normalized sources: 126
- Public rebuild-eligible sources: 108
- Deferred/replacement sources: 18
- Pages needing original-source rebuild: 7

## Source Buckets

- `internal_fraia`: 16
- `public_professional`: 80
- `software_manual`: 30

## Rebuild Flags

- `internal_source_not_original`: the page cites another Fraia wiki/doc page and should be traced back to original references during rebuild.
- `missing_locator`, `missing_source_type`, `missing_or_placeholder_date`, `missing_reliability_limits`: source metadata needs cleanup before promotion.
- `replace_or_corroborate`: weak or incomplete source; replace or corroborate with stronger references.
- `private_local_deferred`: private/local source is inventoried only and is not eligible for the public-source rebuild seed.

## Pages Needing Original-Source Rebuild

- `docs/knowledge/wiki/diagnostics/analysis-result-review-before-design-checks.md`
- `docs/knowledge/wiki/diagnostics/overreleased-members-and-all-pin-mechanisms.md`
- `docs/knowledge/wiki/diagnostics/reaction-sanity-checks.md`
- `docs/knowledge/wiki/product/authored-resolved-run-boundaries.md`
- `docs/knowledge/wiki/product/design-actions-check-inputs-and-results.md`
- `docs/knowledge/wiki/product/engineering-assumptions-and-provenance.md`
- `docs/knowledge/wiki/product/scheme-generation-from-knowledge.md`

## Source List

### SRC-86e2e8a778 — Analysis Result Review Before Design Checks

- Bucket: `internal_fraia`
- Locator: docs/knowledge/wiki/diagnostics/analysis-result-review-before-design-checks.md
- Source type: Fraia compiled diagnostics page
- Date: 2026-05-07
- Reliability/limits: useful review-gate guidance; final review status schema remains future work
- Pages used: not yet recorded
- Public rebuild eligible: false
- Rebuild action: `trace_to_original_public_source`
- Flags: internal_source_not_original
- Current wiki pages: `docs/knowledge/wiki/product/design-actions-check-inputs-and-results.md`
- Used by: `docs/knowledge/wiki/product/design-actions-check-inputs-and-results.md`

### SRC-867d2ce54e — Authored/Resolved/Run Artifact Boundaries

- Bucket: `internal_fraia`
- Locator: docs/knowledge/wiki/product/authored-resolved-run-boundaries.md
- Source type: Fraia compiled product page
- Date: 2026-05-07
- Reliability/limits: useful product-boundary synthesis; final schemas remain future work
- Pages used: not yet recorded
- Public rebuild eligible: false
- Rebuild action: `trace_to_original_public_source`
- Flags: internal_source_not_original
- Current wiki pages: `docs/knowledge/wiki/product/design-actions-check-inputs-and-results.md`, `docs/knowledge/wiki/product/scheme-generation-from-knowledge.md`
- Used by: `docs/knowledge/wiki/product/design-actions-check-inputs-and-results.md`, `docs/knowledge/wiki/product/scheme-generation-from-knowledge.md`

### SRC-e441704512 — Design Actions, Check Inputs, and Check Results

- Bucket: `internal_fraia`
- Locator: docs/knowledge/wiki/product/design-actions-check-inputs-and-results.md
- Source type: Fraia compiled product page
- Date: 2026-05-13
- Reliability/limits: downstream workflow guidance; check modules and schemas remain future work
- Pages used: not yet recorded
- Public rebuild eligible: false
- Rebuild action: `trace_to_original_public_source`
- Flags: internal_source_not_original
- Current wiki pages: `docs/knowledge/wiki/product/scheme-generation-from-knowledge.md`
- Used by: `docs/knowledge/wiki/product/scheme-generation-from-knowledge.md`

### SRC-6884636c8c — Engineering Assumptions and Provenance

- Bucket: `internal_fraia`
- Locator: docs/knowledge/wiki/product/engineering-assumptions-and-provenance.md
- Source type: Fraia compiled product page
- Date: 2026-05-13
- Reliability/limits: product guidance; final schemas and approval workflows remain future work
- Pages used: not yet recorded
- Public rebuild eligible: false
- Rebuild action: `trace_to_original_public_source`
- Flags: internal_source_not_original
- Current wiki pages: `docs/knowledge/wiki/product/scheme-generation-from-knowledge.md`
- Used by: `docs/knowledge/wiki/product/scheme-generation-from-knowledge.md`

### SRC-f401f081fd — Engineering Output Pipeline

- Bucket: `internal_fraia`
- Locator: docs/engineering-output-pipeline.md
- Source type: Fraia architecture doc
- Date: 2026-05-07
- Reliability/limits: canonical product architecture direction; draft status and not a steel code source
- Pages used: not yet recorded
- Public rebuild eligible: false
- Rebuild action: `trace_to_original_public_source`
- Flags: internal_source_not_original
- Current wiki pages: `docs/knowledge/wiki/materials/steel/design-action-check-input-separation.md`, `docs/knowledge/wiki/product/authored-resolved-run-boundaries.md`, `docs/knowledge/wiki/product/design-actions-check-inputs-and-results.md`, `docs/knowledge/wiki/product/engineering-assumptions-and-provenance.md`, `docs/knowledge/wiki/product/scheme-generation-from-knowledge.md`
- Used by: `docs/knowledge/wiki/materials/steel/design-action-check-input-separation.md`, `docs/knowledge/wiki/product/authored-resolved-run-boundaries.md`, `docs/knowledge/wiki/product/design-actions-check-inputs-and-results.md`, `docs/knowledge/wiki/product/engineering-assumptions-and-provenance.md`, `docs/knowledge/wiki/product/scheme-generation-from-knowledge.md`

### SRC-24058e8370 — Fraia Knowledge Topic Map

- Bucket: `internal_fraia`
- Locator: docs/knowledge/topic-map.md
- Source type: Fraia knowledge registry and roadmap
- Date: 2026-05-13
- Reliability/limits: coverage map and prioritisation aid; individual compiled pages remain the source-backed guidance
- Pages used: not yet recorded
- Public rebuild eligible: false
- Rebuild action: `trace_to_original_public_source`
- Flags: internal_source_not_original
- Current wiki pages: `docs/knowledge/wiki/product/scheme-generation-from-knowledge.md`
- Used by: `docs/knowledge/wiki/product/scheme-generation-from-knowledge.md`

### SRC-63420f1192 — Free-Body Diagrams and Equilibrium

- Bucket: `internal_fraia`
- Locator: docs/knowledge/wiki/analysis/free-body-diagrams-and-equilibrium.md
- Source type: Fraia compiled analysis page
- Date: 2026-05-07
- Reliability/limits: useful free-body/equilibrium basis; not a reaction report schema
- Pages used: not yet recorded
- Public rebuild eligible: false
- Rebuild action: `trace_to_original_public_source`
- Flags: internal_source_not_original
- Current wiki pages: `docs/knowledge/wiki/diagnostics/reaction-sanity-checks.md`
- Used by: `docs/knowledge/wiki/diagnostics/reaction-sanity-checks.md`

### SRC-410825b32b — Instability Mechanisms

- Bucket: `internal_fraia`
- Locator: docs/knowledge/wiki/diagnostics/instability-mechanisms.md
- Source type: Fraia compiled diagnostics page
- Date: 2026-05-07
- Reliability/limits: useful instability/result-trust guidance; includes source-scoped software/manual evidence
- Pages used: not yet recorded
- Public rebuild eligible: false
- Rebuild action: `trace_to_original_public_source`
- Flags: internal_source_not_original
- Current wiki pages: `docs/knowledge/wiki/diagnostics/analysis-result-review-before-design-checks.md`, `docs/knowledge/wiki/diagnostics/overreleased-members-and-all-pin-mechanisms.md`, `docs/knowledge/wiki/diagnostics/unconnected-or-underrestrained-models.md`
- Used by: `docs/knowledge/wiki/diagnostics/analysis-result-review-before-design-checks.md`, `docs/knowledge/wiki/diagnostics/overreleased-members-and-all-pin-mechanisms.md`, `docs/knowledge/wiki/diagnostics/unconnected-or-underrestrained-models.md`

### SRC-984798d4c9 — Knowledge Wiki Workflow

- Bucket: `internal_fraia`
- Locator: docs/knowledge/workflow.md
- Source type: Fraia knowledge workflow doc
- Date: 2026-05-07
- Reliability/limits: canonical wiki maintenance workflow; product/project provenance schemas remain future work
- Pages used: not yet recorded
- Public rebuild eligible: false
- Rebuild action: `trace_to_original_public_source`
- Flags: internal_source_not_original
- Current wiki pages: `docs/knowledge/wiki/product/engineering-assumptions-and-provenance.md`
- Used by: `docs/knowledge/wiki/product/engineering-assumptions-and-provenance.md`

### SRC-f703d7c88a — Load Application and Equivalent Nodal Loads

- Bucket: `internal_fraia`
- Locator: docs/knowledge/wiki/loads/load-application-and-equivalent-nodal-loads.md
- Source type: Fraia compiled loads page
- Date: 2026-05-07
- Reliability/limits: useful resolved-load provenance context; not a complete solver load-vector specification
- Pages used: not yet recorded
- Public rebuild eligible: false
- Rebuild action: `trace_to_original_public_source`
- Flags: internal_source_not_original
- Current wiki pages: `docs/knowledge/wiki/diagnostics/reaction-sanity-checks.md`
- Used by: `docs/knowledge/wiki/diagnostics/reaction-sanity-checks.md`

### SRC-a1fe01fe83 — Member End Releases

- Bucket: `internal_fraia`
- Locator: docs/knowledge/wiki/modeling/member-end-releases.md
- Source type: Fraia compiled modeling page
- Date: 2026-05-07
- Reliability/limits: useful Fraia-specific release semantics; inherits source limits from its page
- Pages used: not yet recorded
- Public rebuild eligible: false
- Rebuild action: `trace_to_original_public_source`
- Flags: internal_source_not_original
- Current wiki pages: `docs/knowledge/wiki/diagnostics/overreleased-members-and-all-pin-mechanisms.md`
- Used by: `docs/knowledge/wiki/diagnostics/overreleased-members-and-all-pin-mechanisms.md`

### SRC-3f4ddb6043 — Reaction Sanity Checks

- Bucket: `internal_fraia`
- Locator: docs/knowledge/wiki/diagnostics/reaction-sanity-checks.md
- Source type: Fraia compiled diagnostics page
- Date: 2026-05-07
- Reliability/limits: useful reaction-review guidance; final tolerances/report schemas remain future work
- Pages used: not yet recorded
- Public rebuild eligible: false
- Rebuild action: `trace_to_original_public_source`
- Flags: internal_source_not_original
- Current wiki pages: `docs/knowledge/wiki/diagnostics/analysis-result-review-before-design-checks.md`
- Used by: `docs/knowledge/wiki/diagnostics/analysis-result-review-before-design-checks.md`

### SRC-c240fbba26 — Reactions and Support Idealisation

- Bucket: `internal_fraia`
- Locator: docs/knowledge/wiki/analysis/reactions-and-support-idealisation.md
- Source type: Fraia compiled analysis page
- Date: 2026-05-07
- Reliability/limits: useful Fraia-specific reaction/support synthesis; inherits source limits from its page
- Pages used: not yet recorded
- Public rebuild eligible: false
- Rebuild action: `trace_to_original_public_source`
- Flags: internal_source_not_original
- Current wiki pages: `docs/knowledge/wiki/diagnostics/reaction-sanity-checks.md`
- Used by: `docs/knowledge/wiki/diagnostics/reaction-sanity-checks.md`

### SRC-854b1c9dbb — Resolution and Runs

- Bucket: `internal_fraia`
- Locator: docs/resolution-and-runs.md
- Source type: Fraia architecture doc
- Date: 2026-05-07
- Reliability/limits: canonical authored/resolved/run separation direction; draft status and not a steel code source
- Pages used: not yet recorded
- Public rebuild eligible: false
- Rebuild action: `trace_to_original_public_source`
- Flags: internal_source_not_original
- Current wiki pages: `docs/knowledge/wiki/materials/steel/design-action-check-input-separation.md`, `docs/knowledge/wiki/product/authored-resolved-run-boundaries.md`, `docs/knowledge/wiki/product/engineering-assumptions-and-provenance.md`
- Used by: `docs/knowledge/wiki/materials/steel/design-action-check-input-separation.md`, `docs/knowledge/wiki/product/authored-resolved-run-boundaries.md`, `docs/knowledge/wiki/product/engineering-assumptions-and-provenance.md`

### SRC-fe0bc7d06d — Steel Design Action and Check-Input Separation

- Bucket: `internal_fraia`
- Locator: docs/knowledge/wiki/materials/steel/design-action-check-input-separation.md
- Source type: Fraia compiled steel/product page
- Date: 2026-05-07
- Reliability/limits: useful Fraia pipeline guidance; final schemas remain future work
- Pages used: not yet recorded
- Public rebuild eligible: false
- Rebuild action: `trace_to_original_public_source`
- Flags: internal_source_not_original
- Current wiki pages: `docs/knowledge/wiki/diagnostics/analysis-result-review-before-design-checks.md`, `docs/knowledge/wiki/product/authored-resolved-run-boundaries.md`
- Used by: `docs/knowledge/wiki/diagnostics/analysis-result-review-before-design-checks.md`, `docs/knowledge/wiki/product/authored-resolved-run-boundaries.md`

### SRC-6776d0f8d7 — Truss Analysis and Two-Force Members

- Bucket: `internal_fraia`
- Locator: docs/knowledge/wiki/analysis/truss-analysis-and-two-force-members.md
- Source type: Fraia compiled analysis page
- Date: 2026-05-07
- Reliability/limits: useful truss idealisation context; not a frame mechanism detector
- Pages used: not yet recorded
- Public rebuild eligible: false
- Rebuild action: `trace_to_original_public_source`
- Flags: internal_source_not_original
- Current wiki pages: `docs/knowledge/wiki/diagnostics/overreleased-members-and-all-pin-mechanisms.md`
- Used by: `docs/knowledge/wiki/diagnostics/overreleased-members-and-all-pin-mechanisms.md`

### SRC-07d68bec27 — 2D frame analysis

- Bucket: `public_professional`
- Locator: https://teachbooks.tudelft.nl/computational-modelling/structural_linear/space_frame.html
- Source type: university open course notes
- Date: 2026-05-07
- Reliability/limits: strong 2D frame/local-global stiffness explanation; 3D and nonlinear extensions remain out of scope
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/analysis/matrix-stiffness-method.md`, `docs/knowledge/wiki/modeling/local-and-global-coordinate-systems.md`
- Used by: `docs/knowledge/wiki/analysis/matrix-stiffness-method.md`, `docs/knowledge/wiki/modeling/local-and-global-coordinate-systems.md`

### SRC-65ebca57bd — A Framework for Computer-Aided Conceptual Design of Building Structures

- Bucket: `public_professional`
- Locator: https://www.researchgate.net/publication/244955895_A_Framework_for_Computer-Aided_Conceptual_Design_of_Building_Structures
- Source type: academic conference/chapter paper
- Date: 2026-05-13
- Reliability/limits: useful conceptual-design framework; ResearchGate copy may be author-uploaded and content may be subject to copyright
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/product/structural-design-option-intelligence.md`
- Used by: `docs/knowledge/wiki/product/structural-design-option-intelligence.md`

### SRC-0417b0893b — A Practical P-Delta Analysis Method for Type FR and PR Frames

- Bucket: `public_professional`
- Locator: https://www.aisc.org/A-Practical-P-Delta-Analysis-Method-for-Type-FR-and-PR-Frames
- Source type: professional engineering journal article page
- Date: 2026-05-07
- Reliability/limits: reputable steel-frame stability source; article page/abstract-level evidence only, not a complete design procedure here
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/analysis/second-order-effects-and-stability.md`
- Used by: `docs/knowledge/wiki/analysis/second-order-effects-and-stability.md`

### SRC-ac8741eedd — A parametric analysis of steel and composite portal frames with semi-rigid connections

- Bucket: `public_professional`
- Locator: https://www.sciencedirect.com/science/article/pii/S0141029605003342
- Source type: peer-reviewed article abstract in Thin-Walled Structures
- Date: 2026-05-07
- Reliability/limits: useful source for system-level semi-rigid portal-frame behavior; detailed modeling/calibration is beyond this baseline page
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/modeling/connection-fixity-and-partial-restraint.md`
- Used by: `docs/knowledge/wiki/modeling/connection-fixity-and-partial-restraint.md`

### SRC-e97ede854e — ASCE/SEI 7-22 overview

- Bucket: `public_professional`
- Locator: https://www.asce.org/publications-and-news/codes-and-standards/asce-sei-7-22
- Source type: official public standards overview
- Date: 2026-05-06
- Reliability/limits: overview only
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/loads/gravity-and-lateral-loads.md`, `docs/knowledge/wiki/loads/load-cases-and-combinations.md`
- Used by: `docs/knowledge/raw/loads-gravity-lateral-loads-research.md`, `docs/knowledge/raw/loads-load-cases-combinations-research.md`, `docs/knowledge/wiki/loads/gravity-and-lateral-loads.md`, `docs/knowledge/wiki/loads/load-cases-and-combinations.md`

### SRC-9c9bd7b3a4 — Accounting for moment-rotation behaviour of connections in portal frames

- Bucket: `public_professional`
- Locator: https://scielo.org.za/scielo.php?pid=S1021-20192014000100008&script=sci_arttext
- Source type: peer-reviewed/open academic article
- Date: 2026-05-07
- Reliability/limits: useful evidence for moment-rotation behavior effects; specific modeling approach and not a design code
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/steel/portal-frames/base-fixity-tradeoffs.md`
- Used by: `docs/knowledge/wiki/steel/portal-frames/base-fixity-tradeoffs.md`

### SRC-2889cda0fe — American Wood Council, “Tutorial for Understanding Loads and Using Span Tables”

- Bucket: `public_professional`
- Locator: https://awc.org/codes-standards/spantables/tutorial
- Source type: industry technical tutorial
- Date: 2026-05-06
- Reliability/limits: residential wood joists/rafters and span-table context
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: none
- Used by: `docs/knowledge/raw/analysis-load-paths-research.md`

### SRC-4a41e74bd1 — Applications of the direct stiffness method

- Bucket: `public_professional`
- Locator: https://eng.libretexts.org/Under_Construction/Aerospace_Structures_(Johnson)/16:_Applications_of_the_direct_stiffness_method
- Source type: open educational structural/mechanics text
- Date: 2026-05-07
- Reliability/limits: useful member transformation, global stiffness, and member-force recovery guidance; formula derivations are source-scoped
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/analysis/matrix-stiffness-method.md`, `docs/knowledge/wiki/loads/load-application-and-equivalent-nodal-loads.md`, `docs/knowledge/wiki/modeling/local-and-global-coordinate-systems.md`
- Used by: `docs/knowledge/wiki/analysis/matrix-stiffness-method.md`, `docs/knowledge/wiki/loads/load-application-and-equivalent-nodal-loads.md`, `docs/knowledge/wiki/modeling/local-and-global-coordinate-systems.md`

### SRC-a17357d4ab — Bracing in steel sheds

- Bucket: `public_professional`
- Locator: https://www.steel.org.au/getattachment/6b2b87cd-16fc-4547-8f41-2f535ea3e27f/1_Bracing_in_steel_sheds_bk850_2014.pdf
- Source type: steel industry design guide
- Date: 2026-05-06
- Reliability/limits: Useful concept and design-principle guidance; not a project-specific code check
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/steel/portal-frames/bracing.md`
- Used by: `docs/knowledge/raw/steel-portal-frame-bracing-research.md`, `docs/knowledge/wiki/steel/portal-frames/bracing.md`

### SRC-504746b8d3 — Bracing systems

- Bucket: `public_professional`
- Locator: https://www.steelconstruction.info/Bracing_systems
- Source type: public steel design guidance
- Date: 2026-05-06
- Reliability/limits: steel/bridge/building examples
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/stability/bracing-principles.md`
- Used by: `docs/knowledge/raw/stability-bracing-principles-research.md`, `docs/knowledge/wiki/stability/bracing-principles.md`

### SRC-cc199baa09 — Buckling of columns and plates

- Bucket: `public_professional`
- Locator: https://eng.libretexts.org/Under_Construction/Aerospace_Structures_(Johnson)/11:_Buckling_of_columns_and_plates
- Source type: open educational mechanics/structures text
- Date: 2026-05-07
- Reliability/limits: useful academic buckling fundamentals; aerospace/plates context and not steel code design
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/stability/compression-member-buckling-concepts.md`
- Used by: `docs/knowledge/wiki/stability/compression-member-buckling-concepts.md`

### SRC-0d5eca393d — Building America Solution Center / PNNL, Minimum Design Loads for Buildings and Other Structures, ASCE/SEI 7-10 library page

- Bucket: `public_professional`
- Locator: https://basc.pnnl.gov/library/minimum-design-loads-buildings-and-other-structures-ascesei-7-10
- Source type: public agency guidance
- Date: 2026-05-06
- Reliability/limits: — public summary confirming ASCE 7 scope and standard lineage
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: none
- Used by: `docs/knowledge/raw/loads-gravity-lateral-loads-research.md`

### SRC-abbf4fedf9 — Building Science Corporation, “BSI-030: Advanced Framing”

- Bucket: `public_professional`
- Locator: https://buildingscience.com/documents/insights/bsi-030-advanced-framing
- Source type: public practice article/commentary
- Date: 2026-05-06
- Reliability/limits: opinionated, residential wood-framing-specific, not a standard
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: none
- Used by: `docs/knowledge/raw/analysis-load-paths-research.md`

### SRC-a020a81cf1 — Building envelopes

- Bucket: `public_professional`
- Locator: https://steelconstruction.info/Building_envelopes
- Source type: professional steel construction guidance
- Date: 2026-05-07
- Reliability/limits: strong purlin/side-rail and envelope load-path guidance; UK/Eurocode context and not Fraia schema guidance
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/steel/portal-frames/purlins-girts-and-restraint.md`, `docs/knowledge/wiki/steel/portal-frames/system-overview.md`
- Used by: `docs/knowledge/wiki/steel/portal-frames/purlins-girts-and-restraint.md`, `docs/knowledge/wiki/steel/portal-frames/system-overview.md`

### SRC-35dcc5d736 — Column Effective Lengths in Unbraced Frames

- Bucket: `public_professional`
- Locator: https://www.aisc.org/Column-Effective-Lengths-in-Unbraced-Frames
- Source type: professional engineering journal article page
- Date: 2026-05-07
- Reliability/limits: useful effective-length/frame-stability framing; article page-level source and not a complete design method here
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/stability/member-restraint-and-unbraced-length.md`
- Used by: `docs/knowledge/wiki/stability/member-restraint-and-unbraced-length.md`

### SRC-bee2191e80 — Concept design

- Bucket: `public_professional`
- Locator: https://www.steelconstruction.info/Concept_design
- Source type: professional steel construction guidance
- Date: 2026-05-07
- Reliability/limits: useful concept-stage bracing/stability guidance; UK context and not a check module
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/steel/portal-frames/longitudinal-vs-transverse-stability.md`
- Used by: `docs/knowledge/wiki/steel/portal-frames/longitudinal-vs-transverse-stability.md`

### SRC-101a542199 — Conceptual design of buildings

- Bucket: `public_professional`
- Locator: https://www.istructe.org/getattachment/4ef4c605-efe3-4a56-9c94-7be1295c8984/attachment.aspx
- Source type: professional structural engineering guidance
- Date: 2026-05-13
- Reliability/limits: strong professional concept-design guidance; copyrighted guide, used only for source-scoped concepts and not copied as design rules
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/product/structural-design-option-intelligence.md`
- Used by: `docs/knowledge/wiki/product/structural-design-option-intelligence.md`

### SRC-d5cc476d2c — Conditions for Static Equilibrium

- Bucket: `public_professional`
- Locator: https://phys.libretexts.org/Bookshelves/University_Physics/University_Physics_(OpenStax)/Book:_University_Physics_I_-_Mechanics_Sound_Oscillations_and_Waves_(OpenStax)/12:_Static_Equilibrium_and_Elasticity/12.02:_Conditions_for_Static_Equilibrium
- Source type: open educational physics text
- Date: 2026-05-07
- Reliability/limits: strong static-equilibrium foundation; not structural-analysis-specific
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/analysis/free-body-diagrams-and-equilibrium.md`
- Used by: `docs/knowledge/wiki/analysis/free-body-diagrams-and-equilibrium.md`

### SRC-afd8c4e149 — Continuous Load Path Provided with Connections from the Roof through the Wall to the Foundation

- Bucket: `public_professional`
- Locator: https://basc.pnnl.gov/resource-guides/continuous-load-path-provided-connections-roof-through-wall-foundation
- Source type: public agency guidance
- Date: 2026-05-06
- Reliability/limits: residential/hazard-focused but useful for continuity concepts
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/analysis/load-paths.md`, `docs/knowledge/wiki/loads/gravity-and-lateral-loads.md`
- Used by: `docs/knowledge/raw/analysis-load-paths-research.md`, `docs/knowledge/raw/loads-gravity-lateral-loads-research.md`, `docs/knowledge/wiki/analysis/load-paths.md`, `docs/knowledge/wiki/loads/gravity-and-lateral-loads.md`

### SRC-ba41b36466 — Degrees of Freedom and Restraint Codes

- Bucket: `public_professional`
- Locator: https://skyciv.com/education/explaining-degrees-of-freedom/
- Source type: public commercial education
- Date: 2026-05-06
- Reliability/limits: simplified/product-oriented
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/modeling/supports-restraints-and-releases.md`
- Used by: `docs/knowledge/raw/modeling-supports-restraints-releases-research.md`, `docs/knowledge/wiki/modeling/supports-restraints-and-releases.md`

### SRC-96f3780e82 — Direct stiffness method

- Bucket: `public_professional`
- Locator: https://eng.libretexts.org/Under_Construction/Aerospace_Structures_(Johnson)/15:_Direct_stiffness_method
- Source type: open educational structural/mechanics text
- Date: 2026-05-07
- Reliability/limits: useful direct-stiffness workflow; page is under construction, so claims are corroborated and kept conceptual
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/analysis/matrix-stiffness-method.md`, `docs/knowledge/wiki/loads/load-application-and-equivalent-nodal-loads.md`
- Used by: `docs/knowledge/wiki/analysis/matrix-stiffness-method.md`, `docs/knowledge/wiki/loads/load-application-and-equivalent-nodal-loads.md`

### SRC-3816bce078 — Distributed Loads

- Bucket: `public_professional`
- Locator: https://eng.libretexts.org/Bookshelves/Mechanical_Engineering/Engineering_Mechanics_-_Statics_(Osgood_Cameron_and_Christensen)/03:_Rigid_Body_Basics/3.03:_Distributed_Loads
- Source type: open educational statics text
- Date: 2026-05-07
- Reliability/limits: useful point/distributed load and intensity concepts; not structural-code guidance
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/loads/area-line-point-and-member-loads.md`
- Used by: `docs/knowledge/wiki/loads/area-line-point-and-member-loads.md`

### SRC-85e8bab79d — Distributed Loads

- Bucket: `public_professional`
- Locator: https://eng.libretexts.org/Bookshelves/Mechanical_Engineering/Engineering_Statics:_Open_and_Interactive_(Baker_and_Haynes)/07:_Centroids_and_Centers_of_Gravity/7.08:_Distributed_Loads
- Source type: open educational statics text
- Date: 2026-05-07
- Reliability/limits: strong equivalent-resultant guidance for statics; not a finite-element load-vector derivation
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/loads/area-line-point-and-member-loads.md`, `docs/knowledge/wiki/loads/load-application-and-equivalent-nodal-loads.md`
- Used by: `docs/knowledge/wiki/loads/area-line-point-and-member-loads.md`, `docs/knowledge/wiki/loads/load-application-and-equivalent-nodal-loads.md`

### SRC-8b07b5b3ff — Engineering LibreTexts, “1.3: Equilibrium Structures, Support Reactions, Determinacy and Stability of Beams and Frames”

- Bucket: `public_professional`
- Locator: https://eng.libretexts.org/Bookshelves/Civil_Engineering/Structural_Analysis_(Udoeyo
- Source type: https://eng.libretexts.org/Bookshelves/Civil_Engineering/Structural_Analysis_(Udoeyo)/01:_Chapters/1.03:_Equilibrium_Structures_Support_Reactions_Determinacy_and_Stability_of_Beams_and_Frames — Retrieved 2026-05-06. Source type/context: open educational resource
- Date: 2026-05-06
- Reliability/limits: introductory/statics framing; images/tables not reused.
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: none
- Used by: `docs/knowledge/raw/modeling-supports-restraints-releases-research.md`

### SRC-3bb2c1a062 — Engineering LibreTexts, “2.2: Load Combinations for Structural Design” (https://eng.libretexts.org/Bookshelves/Mechanical_Engineering/Introduction_to_Aerospace_Structures_and_Materials_(Alderliesten)/01:_Introduction_to_Structural_Analysis_and_Structural_Loads/02:_Structural_Loads_and_Loading_System/2.02:_Load_Combinations_for_Structural_Design)

- Bucket: `public_professional`
- Locator: https://eng.libretexts.org/Bookshelves/Mechanical_Engineering/Introduction_to_Aerospace_Structures_and_Materials_(Alderliesten
- Source type: open educational text
- Date: 2026-05-06
- Reliability/limits: open educational explanation of strength vs serviceability and ASCE 7-16 example combinations
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: none
- Used by: `docs/knowledge/raw/loads-load-cases-combinations-research.md`

### SRC-b31f9afb1f — Engineering students' guide to single storey buildings

- Bucket: `public_professional`
- Locator: https://steelconstruction.info/Engineering_students%27_guide_to_single_storey_buildings
- Source type: professional/open educational steel construction guidance
- Date: 2026-05-07
- Reliability/limits: useful system overview; educational UK context
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/steel/portal-frames/longitudinal-vs-transverse-stability.md`, `docs/knowledge/wiki/steel/portal-frames/purlins-girts-and-restraint.md`, `docs/knowledge/wiki/steel/portal-frames/system-overview.md`
- Used by: `docs/knowledge/wiki/steel/portal-frames/longitudinal-vs-transverse-stability.md`, `docs/knowledge/wiki/steel/portal-frames/purlins-girts-and-restraint.md`, `docs/knowledge/wiki/steel/portal-frames/system-overview.md`

### SRC-e0a97ba94d — Equilibrium Structures, Support Reactions, Determinacy and Stability of Beams and Frames

- Bucket: `public_professional`
- Locator: https://eng.libretexts.org/Bookshelves/Civil_Engineering/Structural_Analysis_(Udoeyo)/01:_Chapters/1.03:_Equilibrium_Structures_Support_Reactions_Determinacy_and_Stability_of_Beams_and_Frames
- Source type: open educational structural-analysis text
- Date: 2026-05-07
- Reliability/limits: useful structural equilibrium and support-reaction framing; introductory and not a code check
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/analysis/free-body-diagrams-and-equilibrium.md`, `docs/knowledge/wiki/analysis/reactions-and-support-idealisation.md`, `docs/knowledge/wiki/analysis/static-determinacy-and-restraint.md`, `docs/knowledge/wiki/diagnostics/unconnected-or-underrestrained-models.md`, `docs/knowledge/wiki/modeling/supports-restraints-and-releases.md`
- Used by: `docs/knowledge/wiki/analysis/free-body-diagrams-and-equilibrium.md`, `docs/knowledge/wiki/analysis/reactions-and-support-idealisation.md`, `docs/knowledge/wiki/analysis/static-determinacy-and-restraint.md`, `docs/knowledge/wiki/diagnostics/unconnected-or-underrestrained-models.md`, `docs/knowledge/wiki/modeling/supports-restraints-and-releases.md`

### SRC-ab4b3c6da1 — Eurocode: Basis of structural design

- Bucket: `public_professional`
- Locator: https://eurocodes.jrc.ec.europa.eu/EN-Eurocodes/eurocode-basis-structural-design
- Source type: official public Eurocode overview
- Date: 2026-05-06
- Reliability/limits: overview, national annexes not included
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/loads/load-cases-and-combinations.md`
- Used by: `docs/knowledge/raw/loads-load-cases-combinations-research.md`, `docs/knowledge/wiki/loads/load-cases-and-combinations.md`

### SRC-ba8b564ec9 — European Commission JRC/Eurocodes, `Guidance on the design for structural robustness` (https://eurocodes.jrc.ec.europa.eu/publications/guidance-design-structural-robustness)

- Bucket: `public_professional`
- Locator: https://eurocodes.jrc.ec.europa.eu/publications/guidance-design-structural-robustness
- Source type: public agency guidance
- Date: 2026-05-06
- Reliability/limits: public authoritative context for robustness, tying, alternate load paths, multi-hazard design, and deterioration
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: none
- Used by: `docs/knowledge/raw/stability-bracing-principles-research.md`

### SRC-6258e391b8 — Evaluation of the P-Delta effect in columns and frames using the two-cycle method based on the solution of the beam-column differential equation

- Bucket: `public_professional`
- Locator: https://www.sciencedirect.com/science/article/pii/S2215016123002455
- Source type: open-access peer-reviewed methods article
- Date: 2026-05-07
- Reliability/limits: useful geometric-nonlinearity and P-Delta framing; method-specific and more advanced than this baseline page
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/analysis/second-order-effects-and-stability.md`
- Used by: `docs/knowledge/wiki/analysis/second-order-effects-and-stability.md`

### SRC-d83160defb — Explainer: Structural load paths

- Bucket: `public_professional`
- Locator: https://knowledgehub.ice.org.uk/cpd/delivery-exc/structural-load-paths/
- Source type: professional engineering explainer
- Date: 2026-05-06
- Reliability/limits: qualitative guidance, not a standard
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/analysis/load-paths.md`
- Used by: `docs/knowledge/raw/analysis-load-paths-research.md`, `docs/knowledge/wiki/analysis/load-paths.md`

### SRC-7250b71ac2 — FEM for an Euler-Bernoulli beam

- Bucket: `public_professional`
- Locator: https://teachbooks.tudelft.nl/computational-modelling/structural_linear/Exercises/Workshop_FEM_dyn_beam.html
- Source type: university open teaching material
- Date: 2026-05-06
- Reliability/limits: worked beam example
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/modeling/finite-element-idealisation.md`
- Used by: `docs/knowledge/raw/modeling-finite-element-idealisation-research.md`, `docs/knowledge/wiki/modeling/finite-element-idealisation.md`

### SRC-faa7a92148 — FEMA P-499, “Home Builder’s Guide to Coastal Construction”

- Bucket: `public_professional`
- Locator: https://www.fema.gov/sites/default/files/2020-07/p-499_homebuilders-guide-coastal-construction.pdf
- Source type: public FEMA PDF referenced via BASC
- Date: 2026-05-06
- Reliability/limits: large PDF not fully extracted in this run, residential/coastal focus
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: none
- Used by: `docs/knowledge/raw/analysis-load-paths-research.md`

### SRC-2185db34cf — FEMA P-749 Earthquake-Resistant Design Concepts

- Bucket: `public_professional`
- Locator: https://www.fema.gov/sites/default/files/2020-07/fema_earthquake-resistant-design-concepts_p-749.pdf
- Source type: public agency guide
- Date: 2026-05-06
- Reliability/limits: concept guide, not project design
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/loads/gravity-and-lateral-loads.md`
- Used by: `docs/knowledge/raw/loads-gravity-lateral-loads-research.md`, `docs/knowledge/wiki/loads/gravity-and-lateral-loads.md`

### SRC-5391fbfede — FHWA, Steel Bridge Design Handbook landing page

- Bucket: `public_professional`
- Locator: https://www.fhwa.dot.gov/bridge/steel/pubs/if12052/
- Source type: public agency guidance
- Date: 2026-05-06
- Reliability/limits: confirms handbook public availability and maintenance context by NSBA/AISC
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: none
- Used by: `docs/knowledge/raw/materials-steel-member-behavior-research.md`

### SRC-314b9cb4fd — Finite Element Method

- Bucket: `public_professional`
- Locator: https://eng.libretexts.org/Bookshelves/Materials_Science/TLP_Library_I/30%3A_Finite_Element_Method
- Source type: open educational resource
- Date: 2026-05-06
- Reliability/limits: introductory FEM concepts
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/modeling/finite-element-idealisation.md`
- Used by: `docs/knowledge/raw/modeling-finite-element-idealisation-research.md`, `docs/knowledge/wiki/modeling/finite-element-idealisation.md`

### SRC-ea0bf9ec48 — First Principle Engineering Knowledge, “Basis of Actions and Load Combinations (EN 1990)” (https://knowledge.fppengineering.com/basis-of-actions-and-load-combinations-en-1990/)

- Bucket: `public_professional`
- Locator: https://knowledge.fppengineering.com/basis-of-actions-and-load-combinations-en-1990/
- Source type: public engineering explainer
- Date: 2026-05-06
- Reliability/limits: detailed public conceptual explanation of EN 1990 actions, ψ factors, limit states, and design situations
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: none
- Used by: `docs/knowledge/raw/loads-load-cases-combinations-research.md`

### SRC-1f6adcf4e8 — How to get meaningful and correct results from your finite element model

- Bucket: `public_professional`
- Locator: https://ar5iv.labs.arxiv.org/html/1811.05753
- Source type: open paper/checklist
- Date: 2026-05-06
- Reliability/limits: general FEA guidance, not Fraia-specific
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/modeling/finite-element-idealisation.md`
- Used by: `docs/knowledge/raw/modeling-finite-element-idealisation-research.md`, `docs/knowledge/wiki/modeling/finite-element-idealisation.md`

### SRC-46993e624e — Internal Forces in Beams and Frames

- Bucket: `public_professional`
- Locator: https://eng.libretexts.org/Bookshelves/Civil_Engineering/Structural_Analysis_(Udoeyo)/01:_Chapters/1.04:_Internal_Forces_in_Beams_and_Frames
- Source type: open educational structural-analysis text
- Date: 2026-05-07
- Reliability/limits: useful beam/frame internal-force definitions and diagram behavior; introductory and not a design-code source
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/analysis/beam-shear-and-moment-diagrams.md`
- Used by: `docs/knowledge/wiki/analysis/beam-shear-and-moment-diagrams.md`

### SRC-25c08b9625 — Internal Forces in Plane Trusses

- Bucket: `public_professional`
- Locator: https://eng.libretexts.org/Bookshelves/Civil_Engineering/Structural_Analysis_(Udoeyo)/01:_Chapters/1.05:_Internal_Forces_in_Plane_Trusses
- Source type: open educational structural-analysis text
- Date: 2026-05-07
- Reliability/limits: strong introductory plane-truss assumptions, method-of-joints/sections, and zero-force member framing; not a design-code source
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/analysis/truss-analysis-and-two-force-members.md`
- Used by: `docs/knowledge/wiki/analysis/truss-analysis-and-two-force-members.md`

### SRC-833946e3b6 — Introduction to Load Paths

- Bucket: `public_professional`
- Locator: https://ocw.tudelft.nl/course-readings/4-2-1-introduction-to-load-paths/
- Source type: university OCW
- Date: 2026-05-06
- Reliability/limits: concise concept source, not building-code guidance
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/analysis/load-paths.md`
- Used by: `docs/knowledge/raw/analysis-load-paths-research.md`, `docs/knowledge/wiki/analysis/load-paths.md`

### SRC-1cf8e94574 — Introduction to Structural Analysis

- Bucket: `public_professional`
- Locator: https://eng.libretexts.org/Bookshelves/Civil_Engineering/Structural_Analysis_(Udoeyo)/01:_Chapters/1.01:_Introduction_to_Structural_Analysis
- Source type: open educational structural-analysis text
- Date: 2026-05-07
- Reliability/limits: introductory framing of external loads and structural response; concept-level use only
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/loads/area-line-point-and-member-loads.md`
- Used by: `docs/knowledge/wiki/loads/area-line-point-and-member-loads.md`

### SRC-17fec70cbe — Lateral Systems

- Bucket: `public_professional`
- Locator: https://www.aisc.org/architecture-center/engineering-basics/lateral-systems/
- Source type: professional organization educational page
- Date: 2026-05-06
- Reliability/limits: high-level overview
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/stability/bracing-principles.md`, `docs/knowledge/wiki/steel/portal-frames/bracing.md`
- Used by: `docs/knowledge/raw/stability-bracing-principles-research.md`, `docs/knowledge/raw/steel-portal-frame-bracing-research.md`, `docs/knowledge/wiki/stability/bracing-principles.md`, `docs/knowledge/wiki/steel/portal-frames/bracing.md`

### SRC-346ae46879 — Lateral Torsional Buckling of Steel Beams Elastically Restrained at the Support Nodes

- Bucket: `public_professional`
- Locator: https://www.mdpi.com/2076-3417/9/9/1944/htm
- Source type: open-access peer-reviewed article
- Date: 2026-05-07
- Reliability/limits: useful source for restraint-stiffness influence on LTB; support-node restraint focus and not a design-code procedure
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/stability/lateral-torsional-buckling-concepts.md`
- Used by: `docs/knowledge/wiki/stability/lateral-torsional-buckling-concepts.md`

### SRC-067bd39e9f — Materials-oriented integrated design and construction of structures in civil engineering: A review

- Bucket: `public_professional`
- Locator: https://link.springer.com/article/10.1007/s11709-021-0794-9
- Source type: open-access academic review
- Date: 2026-05-13
- Reliability/limits: broad integrated-design review; not specific to steel portal frames or Fraia workflows
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/product/structural-design-option-intelligence.md`
- Used by: `docs/knowledge/wiki/product/structural-design-option-intelligence.md`

### SRC-1775f1230f — Member design

- Bucket: `public_professional`
- Locator: https://www.steelconstruction.info/Member_design
- Source type: professional steel construction guidance
- Date: 2026-05-07
- Reliability/limits: strong practical steel member guidance; UK/Eurocode context and not Fraia schema guidance
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/materials/steel/beams-and-bending-members.md`, `docs/knowledge/wiki/materials/steel/compression-members.md`, `docs/knowledge/wiki/materials/steel/design-action-check-input-separation.md`, `docs/knowledge/wiki/materials/steel/member-behavior.md`
- Used by: `docs/knowledge/raw/materials-steel-member-behavior-research.md`, `docs/knowledge/wiki/materials/steel/beams-and-bending-members.md`, `docs/knowledge/wiki/materials/steel/compression-members.md`, `docs/knowledge/wiki/materials/steel/design-action-check-input-separation.md`, `docs/knowledge/wiki/materials/steel/member-behavior.md`

### SRC-de331c3f98 — Member design

- Bucket: `public_professional`
- Locator: https://steelconstruction.info/Member_design
- Source type: professional steel construction guidance
- Date: 2026-05-07
- Reliability/limits: strong practical steel member stability guidance; UK/Eurocode context and not Fraia schema guidance
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/stability/compression-member-buckling-concepts.md`, `docs/knowledge/wiki/stability/lateral-torsional-buckling-concepts.md`, `docs/knowledge/wiki/stability/member-restraint-and-unbraced-length.md`
- Used by: `docs/knowledge/wiki/stability/compression-member-buckling-concepts.md`, `docs/knowledge/wiki/stability/lateral-torsional-buckling-concepts.md`, `docs/knowledge/wiki/stability/member-restraint-and-unbraced-length.md`

### SRC-a4257cc619 — Member end releases in framed structures

- Bucket: `public_professional`
- Locator: https://www.sciencedirect.com/science/article/abs/pii/004579499390214X
- Source type: peer-reviewed article abstract in Computers & Structures
- Date: 2026-05-07
- Reliability/limits: useful stiffness-method release framing from abstract-level access; detailed algorithm not copied or relied on
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/modeling/member-end-releases.md`
- Used by: `docs/knowledge/wiki/modeling/member-end-releases.md`

### SRC-24aac3b6b5 — Method of Joints

- Bucket: `public_professional`
- Locator: https://eng.libretexts.org/Bookshelves/Mechanical_Engineering/Engineering_Statics:_Open_and_Interactive_(Baker_and_Haynes)/06:_Equilibrium_of_Structures/6.04:_Method_of_Joints
- Source type: open educational statics text
- Date: 2026-05-07
- Reliability/limits: useful joint-equilibrium framing; introductory and not a structural design reference
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/analysis/truss-analysis-and-two-force-members.md`
- Used by: `docs/knowledge/wiki/analysis/truss-analysis-and-two-force-members.md`

### SRC-a2753ade4c — Method of Sections

- Bucket: `public_professional`
- Locator: https://eng.libretexts.org/Bookshelves/Mechanical_Engineering/Mechanics_Map_(Moore_2nd_Edition)/05:_Engineering_Structures/5.05:_Method_of_Sections
- Source type: open educational statics text
- Date: 2026-05-07
- Reliability/limits: useful method-of-sections and tension/compression convention guidance; introductory and not a design-code source
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/analysis/truss-analysis-and-two-force-members.md`
- Used by: `docs/knowledge/wiki/analysis/truss-analysis-and-two-force-members.md`

### SRC-9be980a1af — Modelling and analysis

- Bucket: `public_professional`
- Locator: https://steelconstruction.info/Modelling_and_analysis
- Source type: professional steel construction guidance
- Date: 2026-05-07
- Reliability/limits: strong practical framing for first-order/second-order analysis and steel modeling; UK/Eurocode context and not Fraia implementation guidance
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/analysis/second-order-effects-and-stability.md`, `docs/knowledge/wiki/modeling/connection-fixity-and-partial-restraint.md`, `docs/knowledge/wiki/modeling/member-end-releases.md`
- Used by: `docs/knowledge/wiki/analysis/second-order-effects-and-stability.md`, `docs/knowledge/wiki/modeling/connection-fixity-and-partial-restraint.md`, `docs/knowledge/wiki/modeling/member-end-releases.md`

### SRC-ea31d038c2 — Moment resisting connections

- Bucket: `public_professional`
- Locator: https://www.steelconstruction.info/Moment_resisting_connections
- Source type: professional steel construction guidance
- Date: 2026-05-07
- Reliability/limits: strong practical moment/splice/base connection taxonomy; UK/Eurocode context and not Fraia schema guidance
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/materials/steel/connections-concept-taxonomy.md`, `docs/knowledge/wiki/materials/steel/member-behavior.md`, `docs/knowledge/wiki/steel/portal-frames/base-fixity-tradeoffs.md`
- Used by: `docs/knowledge/raw/materials-steel-member-behavior-research.md`, `docs/knowledge/wiki/materials/steel/connections-concept-taxonomy.md`, `docs/knowledge/wiki/materials/steel/member-behavior.md`, `docs/knowledge/wiki/steel/portal-frames/base-fixity-tradeoffs.md`

### SRC-a8a9f0ef04 — Multi-Objective Heuristic Computation Applied to Architectural and Structural Design: A Review

- Bucket: `public_professional`
- Locator: https://journals.sagepub.com/doi/10.1260/1478-0771.11.4.363
- Source type: academic review
- Date: 2026-05-13
- Reliability/limits: useful survey of multi-objective methods; access-limited and not a deterministic design standard
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/product/structural-design-option-intelligence.md`
- Used by: `docs/knowledge/wiki/product/structural-design-option-intelligence.md`

### SRC-c4a211ba11 — Notes on the AISC 360-16 Provisions for Slender Compression Elements in Compression Members

- Bucket: `public_professional`
- Locator: https://ej.aisc.org/index.php/engj/article/download/1102/1101
- Source type: AISC Engineering Journal paper
- Date: 2026-05-07
- Reliability/limits: useful professional discussion of compression member limit states and slender elements; not a generic Fraia procedure
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/materials/steel/compression-members.md`
- Used by: `docs/knowledge/wiki/materials/steel/compression-members.md`

### SRC-0c0adf8968 — P252 Design of Single-Span Steel Portal Frames to BS 5950-1:2000

- Bucket: `public_professional`
- Locator: https://www.steelconstruction.info/images/4/44/SCI_P252.pdf
- Source type: steel industry design guide
- Date: 2026-05-06
- Reliability/limits: Detailed portal-frame guide; older code basis but useful for system concepts
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/steel/portal-frames/bracing.md`
- Used by: `docs/knowledge/raw/steel-portal-frame-bracing-research.md`, `docs/knowledge/wiki/steel/portal-frames/bracing.md`

### SRC-0789ae2f66 — Partially Restrained and Flexible Moment Connections

- Bucket: `public_professional`
- Locator: https://www.aisc.org/education/continuingeducation/education-archives/partially-restrained-and-flexible-moment-connections/
- Source type: professional steel continuing-education source
- Date: 2026-05-07
- Reliability/limits: reputable AISC overview of partially restrained connection behavior; course page-level source, not a complete design guide here
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/modeling/connection-fixity-and-partial-restraint.md`
- Used by: `docs/knowledge/wiki/modeling/connection-fixity-and-partial-restraint.md`

### SRC-37f553a164 — Portal frames

- Bucket: `public_professional`
- Locator: https://steelconstruction.info/Portal_frames
- Source type: professional steel construction guidance
- Date: 2026-05-07
- Reliability/limits: strong practical portal-frame base fixity guidance; UK/Eurocode context and numerical assumptions are not reproduced here
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/steel/portal-frames/base-fixity-tradeoffs.md`, `docs/knowledge/wiki/steel/portal-frames/longitudinal-vs-transverse-stability.md`, `docs/knowledge/wiki/steel/portal-frames/purlins-girts-and-restraint.md`, `docs/knowledge/wiki/steel/portal-frames/system-overview.md`
- Used by: `docs/knowledge/wiki/steel/portal-frames/base-fixity-tradeoffs.md`, `docs/knowledge/wiki/steel/portal-frames/longitudinal-vs-transverse-stability.md`, `docs/knowledge/wiki/steel/portal-frames/purlins-girts-and-restraint.md`, `docs/knowledge/wiki/steel/portal-frames/system-overview.md`

### SRC-620438d876 — Robustness in Structural Steel Framing Systems

- Bucket: `public_professional`
- Locator: https://www.aisc.org/globalassets/aisc/research-library/robustness-in-structural-steel-framing-systems.pdf
- Source type: professional/academic structural steel research report
- Date: 2026-05-13
- Reliability/limits: useful robustness and alternate-load-path concepts for steel framing; final robustness design remains code- and project-specific
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/product/structural-design-option-intelligence.md`
- Used by: `docs/knowledge/wiki/product/structural-design-option-intelligence.md`

### SRC-f00f4275e9 — SUNY/Pressbooks, “Load Tracing – Basic Concepts of Structural Design for Architecture Students”

- Bucket: `public_professional`
- Locator: https://structuraldesign.pressbooks.sunycreate.cloud/chapter/chapter-12-load-tracing/
- Source type: open educational chapter found in search
- Date: 2026-05-06
- Reliability/limits: less comprehensive than the kept Pressbooks chapter above
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: none
- Used by: `docs/knowledge/raw/analysis-load-paths-research.md`

### SRC-0a27cc5847 — Seismic Design Principles

- Bucket: `public_professional`
- Locator: https://www.wbdg.org/resources/seismic-design-principles
- Source type: public federal design resource
- Date: 2026-05-06
- Reliability/limits: seismic-oriented overview
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/stability/bracing-principles.md`
- Used by: `docs/knowledge/raw/stability-bracing-principles-research.md`, `docs/knowledge/wiki/stability/bracing-principles.md`

### SRC-367757f7b9 — Seismic Design of Cast-in-Place Concrete Diaphragms, Chords, and Collectors

- Bucket: `public_professional`
- Locator: https://www.nehrp.gov/pdf/nistgcr10-917-4.pdf
- Source type: public technical brief
- Date: 2026-05-06
- Reliability/limits: seismic diaphragm focus
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/stability/bracing-principles.md`
- Used by: `docs/knowledge/raw/stability-bracing-principles-research.md`, `docs/knowledge/wiki/stability/bracing-principles.md`

### SRC-ec48d66331 — Seismic Design of Cast-in-Place Concrete Diaphragms, Chords, and Collectors: A Guide for Practicing Engineers

- Bucket: `public_professional`
- Locator: https://nehrp.gov/pdf/nistgcr11-917-10.pdf
- Source type: public professional technical brief
- Date: 2026-05-07
- Reliability/limits: strong diaphragm/chord/collector system guidance; concrete/seismic focus and not a generic Fraia schema source
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/modeling/constraints-rigid-links-and-diaphragms.md`
- Used by: `docs/knowledge/wiki/modeling/constraints-rigid-links-and-diaphragms.md`

### SRC-88d628276b — Shear and Bending Moment Diagrams

- Bucket: `public_professional`
- Locator: https://eng.libretexts.org/Bookshelves/Mechanical_Engineering/Mechanics_of_Materials_(Roylance)/04:_Bending/4.01:_Shear_and_Bending_Moment_Diagrams
- Source type: open educational mechanics text derived from MIT materials
- Date: 2026-05-07
- Reliability/limits: strong mechanics-of-materials explanation of cut-section equilibrium; not structural-code guidance
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/analysis/beam-shear-and-moment-diagrams.md`
- Used by: `docs/knowledge/wiki/analysis/beam-shear-and-moment-diagrams.md`

### SRC-9797773856 — Shear/Moment Diagrams

- Bucket: `public_professional`
- Locator: https://eng.libretexts.org/Bookshelves/Mechanical_Engineering/Engineering_Mechanics_-_Statics_(Osgood_Cameron_and_Christensen)/06:_Internal_Forces/6.02:_Shear_Moment_Diagrams
- Source type: open educational statics text
- Date: 2026-05-07
- Reliability/limits: useful load-shear-moment relationship guidance; introductory and partially adapted from Udoeyo
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/analysis/beam-shear-and-moment-diagrams.md`
- Used by: `docs/knowledge/wiki/analysis/beam-shear-and-moment-diagrams.md`

### SRC-75c03f33d8 — Simple connections

- Bucket: `public_professional`
- Locator: https://steelconstruction.info/Simple_connections
- Source type: professional steel construction guidance
- Date: 2026-05-07
- Reliability/limits: strong practical simple/bracing/base connection guidance; UK/Eurocode context and not Fraia schema guidance
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/materials/steel/connections-concept-taxonomy.md`, `docs/knowledge/wiki/materials/steel/member-behavior.md`
- Used by: `docs/knowledge/raw/materials-steel-member-behavior-research.md`, `docs/knowledge/wiki/materials/steel/connections-concept-taxonomy.md`, `docs/knowledge/wiki/materials/steel/member-behavior.md`

### SRC-65fc8dcb5b — Single-storey steel buildings Part 4: Detailed Design of Portal Frames

- Bucket: `public_professional`
- Locator: https://www.steelconstruction.info/images/b/b8/SBE_SS4.pdf
- Source type: steel industry design guide
- Date: 2026-05-06
- Reliability/limits: Corroborating portal-frame guidance
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/steel/portal-frames/bracing.md`
- Used by: `docs/knowledge/raw/steel-portal-frame-bracing-research.md`, `docs/knowledge/wiki/steel/portal-frames/bracing.md`

### SRC-aeb2330d27 — Specification for Structural Steel Buildings (ANSI/AISC 360-16)

- Bucket: `public_professional`
- Locator: https://www.aisc.org/globalassets/aisc/publications/standards/a360-16-spec-and-commentary.pdf
- Source type: professional standard/specification
- Date: 2026-05-07
- Reliability/limits: authoritative steel flexural-member terminology; US code context and formulas are not reproduced here
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/materials/steel/beams-and-bending-members.md`, `docs/knowledge/wiki/materials/steel/compression-members.md`, `docs/knowledge/wiki/stability/compression-member-buckling-concepts.md`, `docs/knowledge/wiki/stability/lateral-torsional-buckling-concepts.md`, `docs/knowledge/wiki/stability/member-restraint-and-unbraced-length.md`
- Used by: `docs/knowledge/wiki/materials/steel/beams-and-bending-members.md`, `docs/knowledge/wiki/materials/steel/compression-members.md`, `docs/knowledge/wiki/stability/compression-member-buckling-concepts.md`, `docs/knowledge/wiki/stability/lateral-torsional-buckling-concepts.md`, `docs/knowledge/wiki/stability/member-restraint-and-unbraced-length.md`

### SRC-d9c2d2e337 — Stability and Determinacy

- Bucket: `public_professional`
- Locator: https://eng.libretexts.org/Bookshelves/Mechanical_Engineering/Engineering_Statics:_Open_and_Interactive_(Baker_and_Haynes)/05:_Rigid_Body_Equilibrium/5.06:_Stability_and_Determinacy
- Source type: open educational statics text
- Date: 2026-05-07
- Reliability/limits: strong rigid-body support/restraint framing; not a full structural-analysis design reference
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/analysis/static-determinacy-and-restraint.md`
- Used by: `docs/knowledge/wiki/analysis/static-determinacy-and-restraint.md`

### SRC-f55817a4e6 — Static Indeterminacy

- Bucket: `public_professional`
- Locator: https://oit.tudelft.nl/CT1000/2025/_git/github.com_TUDelft-books_CEG-mechanics-BSc/EN/book/statically_inderminate/determinancy.html
- Source type: university open course notes
- Date: 2026-05-07
- Reliability/limits: useful free-body/counting procedure for external and internal static indeterminacy; course-scoped examples and not a universal model-validity algorithm
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/analysis/static-determinacy-and-restraint.md`
- Used by: `docs/knowledge/wiki/analysis/static-determinacy-and-restraint.md`

### SRC-a80147d170 — Steel Bridge Design Handbook Chapter 4: Strength Behavior and Design of Steel

- Bucket: `public_professional`
- Locator: https://www.aisc.org/media/hf4jbmik/b904_sbdh_chapter4.pdf
- Source type: public professional design handbook chapter
- Date: 2026-05-07
- Reliability/limits: useful strength behavior source for steel flexure/shear concepts; bridge/AASHTO context and not generic building-code guidance
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/materials/steel/beams-and-bending-members.md`, `docs/knowledge/wiki/materials/steel/member-behavior.md`
- Used by: `docs/knowledge/raw/materials-steel-member-behavior-research.md`, `docs/knowledge/wiki/materials/steel/beams-and-bending-members.md`, `docs/knowledge/wiki/materials/steel/member-behavior.md`

### SRC-6d2d387325 — Steel material properties

- Bucket: `public_professional`
- Locator: https://www.steelconstruction.info/Steel_material_properties
- Source type: professional steel construction guidance
- Date: 2026-05-07
- Reliability/limits: strong steel material overview; UK/Eurocode product-standard context and not Fraia schema guidance
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/materials/steel/material-properties-and-section-families.md`
- Used by: `docs/knowledge/wiki/materials/steel/material-properties-and-section-families.md`

### SRC-2836969f03 — Steel section sizes

- Bucket: `public_professional`
- Locator: https://www.steelconstruction.info/Steel_section_sizes
- Source type: professional steel construction guidance
- Date: 2026-05-07
- Reliability/limits: useful section family/property-source overview; UK/European catalog context
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/materials/steel/material-properties-and-section-families.md`
- Used by: `docs/knowledge/wiki/materials/steel/material-properties-and-section-families.md`

### SRC-71fc32b40e — SteelConstruction.info, `Modelling and analysis` (https://www.steelconstruction.info/Modelling_and_analysis)

- Bucket: `public_professional`
- Locator: https://www.steelconstruction.info/Modelling_and_analysis
- Source type: professional steel construction guidance
- Date: 2026-05-06
- Reliability/limits: open source for model verification, lateral systems, releases, second-order effects, and stability analysis cautions
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: none
- Used by: `docs/knowledge/raw/stability-bracing-principles-research.md`

### SRC-0beb09acb0 — Structural Innovation: Combining Classic Theories with New Technologies

- Bucket: `public_professional`
- Locator: https://www.aisc.org/Structural-Innovation-Combining-Classic-Theories-with-New-Technologies
- Source type: professional/academic steel structural engineering article
- Date: 2026-05-13
- Reliability/limits: useful for geometry, load-path theory, topology optimization, and shape-finding concepts; not a Fraia schema or final sizing guide
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/product/structural-design-option-intelligence.md`
- Used by: `docs/knowledge/wiki/product/structural-design-option-intelligence.md`

### SRC-7087902e41 — Structural Systems and Load Tracing

- Bucket: `public_professional`
- Locator: https://saalck.pressbooks.pub/structuralconceptsforarchitectsandconstructionmanagers/chapter/module-4-structural-systems-and-load-tracing/
- Source type: open educational resource
- Date: 2026-05-06
- Reliability/limits: simplified teaching examples
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/analysis/load-paths.md`
- Used by: `docs/knowledge/raw/analysis-load-paths-research.md`, `docs/knowledge/wiki/analysis/load-paths.md`

### SRC-1fa009051f — Supports

- Bucket: `public_professional`
- Locator: https://oit.tudelft.nl/CEG-mechanics-BSc/support_internal_forces/model/supports.html
- Source type: university open course notes
- Date: 2026-05-07
- Reliability/limits: useful DOF/reaction/support notation and prescribed displacement framing; course-scoped and not a full structural design reference
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/analysis/reactions-and-support-idealisation.md`
- Used by: `docs/knowledge/wiki/analysis/reactions-and-support-idealisation.md`

### SRC-9be02b31d1 — Supports

- Bucket: `public_professional`
- Locator: https://oit.tudelft.nl/CT1000/2024/external/mechanics-BSc/book/support_internal_forces/model/supports.html
- Source type: university open course notes
- Date: 2026-05-06
- Reliability/limits: concise course notes
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/modeling/supports-restraints-and-releases.md`
- Used by: `docs/knowledge/raw/modeling-supports-restraints-releases-research.md`, `docs/knowledge/wiki/modeling/supports-restraints-and-releases.md`

### SRC-28666e881a — Technical Guidance Note: Simple connections in steel frames

- Bucket: `public_professional`
- Locator: https://www.istructe.org/journal/volumes/volume-96-(2018)/issue-9/technical-guidance-note-level-2-no-17-steel-frames/
- Source type: professional technical guidance note page
- Date: 2026-05-07
- Reliability/limits: useful professional definition/orientation; page-level access and not full connection design guidance
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/materials/steel/connections-concept-taxonomy.md`
- Used by: `docs/knowledge/wiki/materials/steel/connections-concept-taxonomy.md`

### SRC-708eb4f6a3 — Two Dimensional Coordinate Systems

- Bucket: `public_professional`
- Locator: https://eng.libretexts.org/Bookshelves/Mechanical_Engineering/Engineering_Statics:_Open_and_Interactive_(Baker_and_Haynes)/02:_Forces_and_Other_Vectors/2.03:_Two_Dimensional_Coordinate_Systems
- Source type: open educational statics text
- Date: 2026-05-07
- Reliability/limits: strong coordinate/vector foundation; not structural FEM-specific
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/modeling/local-and-global-coordinate-systems.md`
- Used by: `docs/knowledge/wiki/modeling/local-and-global-coordinate-systems.md`

### SRC-f9a8b7e202 — University Physics Volume 1: 12.1 Conditions for Static Equilibrium

- Bucket: `public_professional`
- Locator: https://openstax.org/books/university-physics-volume-1/pages/12-1-conditions-for-static-equilibrium
- Source type: open educational physics text
- Date: 2026-05-07
- Reliability/limits: strong rigid-body equilibrium foundation; not structural-analysis-specific
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/analysis/reactions-and-support-idealisation.md`
- Used by: `docs/knowledge/wiki/analysis/reactions-and-support-idealisation.md`

### SRC-4dfb303f52 — AISC Shapes Database v16.0

- Bucket: `software_manual`
- Locator: https://www.aisc.org/aisc/publications/steel-construction-manual/aisc-shapes-database-v160/
- Source type: professional steel section-property database page
- Date: 2026-05-07
- Reliability/limits: authoritative US shape database page; database contents are not copied into this wiki
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/materials/steel/material-properties-and-section-families.md`
- Used by: `docs/knowledge/wiki/materials/steel/material-properties-and-section-families.md`

### SRC-1b42650ca2 — Abaqus 2024 Documentation mirror, “Frame Elements”

- Bucket: `software_manual`
- Locator: https://docs.software.vt.edu/abaqusv2024/English/SIMACAEELMRefMap/simaelm-c-frame.htm
- Source type: https://docs.software.vt.edu/abaqusv2024/English/SIMACAEELMRefMap/simaelm-c-frame.htm — Retrieved 2026-05-06. Source type/context: public documentation mirror for commercial FEA
- Date: 2026-05-06
- Reliability/limits: commercial product mirror, not primary Fraia target; used only for broad convention context.
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: none
- Used by: `docs/knowledge/raw/modeling-supports-restraints-releases-research.md`

### SRC-46f4a11354 — Connectivity FAQ

- Bucket: `software_manual`
- Locator: https://www.lusas.com/user_area/faqs/connectivity.html
- Source type: public software FAQ
- Date: 2026-05-06
- Reliability/limits: FE connectivity focused
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/diagnostics/instability-mechanisms.md`, `docs/knowledge/wiki/diagnostics/unconnected-or-underrestrained-models.md`
- Used by: `docs/knowledge/raw/diagnostics-instability-mechanisms-research.md`, `docs/knowledge/wiki/diagnostics/instability-mechanisms.md`, `docs/knowledge/wiki/diagnostics/unconnected-or-underrestrained-models.md`

### SRC-d917ef7f51 — Constraints Commands

- Bucket: `software_manual`
- Locator: https://opensees.github.io/OpenSeesDocumentation/user/manual/model/constraint.html
- Source type: open-source solver documentation
- Date: 2026-05-07
- Reliability/limits: useful for SP/MP constraint vocabulary and solver-topology distinction; software-specific syntax is not Fraia behavior
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/modeling/constraints-rigid-links-and-diaphragms.md`
- Used by: `docs/knowledge/wiki/modeling/constraints-rigid-links-and-diaphragms.md`

### SRC-43ffb698a6 — Define a release in a frame structure

- Bucket: `software_manual`
- Locator: https://help.autodesk.com/cloudhelp/2026/ENU/Inventor-Help/files/GUID-2E87FB0F-06D2-44D7-824B-EB514DD155DD.htm
- Source type: public software documentation
- Date: 2026-05-06
- Reliability/limits: product-specific but useful for release conventions
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/modeling/supports-restraints-and-releases.md`
- Used by: `docs/knowledge/raw/modeling-supports-restraints-releases-research.md`, `docs/knowledge/wiki/modeling/supports-restraints-and-releases.md`

### SRC-87b9542ec6 — EduBeam guide, “Introduction” (https://edubeam.app/guide/introduction.html)

- Bucket: `software_manual`
- Locator: https://edubeam.app/guide/introduction.html
- Source type: public software documentation
- Date: 2026-05-06
- Reliability/limits: public example of structural UI workflow that maps nodes/elements/materials/supports/loads to FEM reactions, displacements, and internal forces
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: none
- Used by: `docs/knowledge/raw/modeling-finite-element-idealisation-research.md`

### SRC-6e33dce5df — Elastic Beam Column Element

- Bucket: `software_manual`
- Locator: https://opensees.github.io/OpenSeesDocumentation/user/manual/model/elements/elasticBeamColumn.html
- Source type: open-source solver documentation
- Date: 2026-05-07
- Reliability/limits: useful source-scoped evidence for end/local-axis release conventions; software-specific syntax is not Fraia behavior
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/modeling/member-end-releases.md`
- Used by: `docs/knowledge/wiki/modeling/member-end-releases.md`

### SRC-c7b2d78a11 — FEM Geometry Preparation and Meshing

- Bucket: `software_manual`
- Locator: https://reqrefusion.github.io/FreeCAD-Documentation-html/wiki/FEM_Geometry_Preparation_and_Meshing.html
- Source type: public software documentation
- Date: 2026-05-06
- Reliability/limits: practical software guidance
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/modeling/finite-element-idealisation.md`
- Used by: `docs/knowledge/raw/modeling-finite-element-idealisation-research.md`, `docs/knowledge/wiki/modeling/finite-element-idealisation.md`

### SRC-a45e036c0d — FEMA P-55, “Coastal Construction Manual”

- Bucket: `software_manual`
- Locator: https://www.fema.gov/sites/default/files/2020-08/fema55_volii_combined_rev.pdf
- Source type: public FEMA PDF found in search
- Date: 2026-05-06
- Reliability/limits: large PDF extraction failed/too large in this run, coastal-specific
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: none
- Used by: `docs/knowledge/raw/analysis-load-paths-research.md`

### SRC-d398a6a7cf — FEMA P-957 Snow Load Safety Guide

- Bucket: `software_manual`
- Locator: https://www.fema.gov/sites/default/files/documents/fema957_snowload_guide.pdf
- Source type: public agency guide
- Date: 2026-05-06
- Reliability/limits: snow-focused
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/loads/gravity-and-lateral-loads.md`
- Used by: `docs/knowledge/raw/loads-gravity-lateral-loads-research.md`, `docs/knowledge/wiki/loads/gravity-and-lateral-loads.md`

### SRC-eadbb18f41 — Finding and Fixing Calculation Instabilities

- Bucket: `software_manual`
- Locator: https://www.dlubal.com/en/support-and-learning/support/faq/005345
- Source type: public software FAQ
- Date: 2026-05-06
- Reliability/limits: product examples
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/diagnostics/instability-mechanisms.md`
- Used by: `docs/knowledge/raw/diagnostics-instability-mechanisms-research.md`, `docs/knowledge/wiki/diagnostics/instability-mechanisms.md`

### SRC-8419c2b9c6 — Finite element method

- Bucket: `software_manual`
- Locator: https://eng.libretexts.org/Under_Construction/Aerospace_Structures_(Johnson)/17:_Finite_element_method
- Source type: open educational structural/mechanics text
- Date: 2026-05-07
- Reliability/limits: useful connection between direct stiffness and FEM concepts; not a Fraia solver implementation guide
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/analysis/matrix-stiffness-method.md`, `docs/knowledge/wiki/loads/load-application-and-equivalent-nodal-loads.md`
- Used by: `docs/knowledge/wiki/analysis/matrix-stiffness-method.md`, `docs/knowledge/wiki/loads/load-application-and-equivalent-nodal-loads.md`

### SRC-764acec0a5 — Free Body Diagrams

- Bucket: `software_manual`
- Locator: https://eng.libretexts.org/Bookshelves/Mechanical_Engineering/Engineering_Statics:_Open_and_Interactive_(Baker_and_Haynes)/05:_Rigid_Body_Equilibrium/5.02:_Free_Body_Diagrams
- Source type: open educational statics text
- Date: 2026-05-07
- Reliability/limits: strong free-body diagram and rigid-body statics guidance; not a structural solver guide
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/analysis/free-body-diagrams-and-equilibrium.md`
- Used by: `docs/knowledge/wiki/analysis/free-body-diagrams-and-equilibrium.md`

### SRC-df393de74d — Kept: Oasys GSA Ill-conditioning theory (https://docs.oasys-software.com/structural/gsa/version/10.2.12/references-theory/ill-conditioning/)

- Bucket: `software_manual`
- Locator: https://docs.oasys-software.com/structural/gsa/version/10.2.12/references-theory/ill-conditioning/
- Source type: public software documentation
- Date: 2026-05-06
- Reliability/limits: public theory note distinguishing numerical conditioning from pure mechanisms
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: none
- Used by: `docs/knowledge/raw/diagnostics-instability-mechanisms-research.md`

### SRC-8f86f42e18 — Load Cases & Load Combinations

- Bucket: `software_manual`
- Locator: https://github.com/JWock82/Pynite/wiki/5.-Load-Cases-&-Load-Combinations
- Source type: open-source software documentation
- Date: 2026-05-06
- Reliability/limits: software representation, not design standard
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/loads/load-cases-and-combinations.md`
- Used by: `docs/knowledge/raw/loads-load-cases-combinations-research.md`, `docs/knowledge/wiki/loads/load-cases-and-combinations.md`

### SRC-817ad4eb26 — Load cases and load combinations

- Bucket: `software_manual`
- Locator: https://anastruct.readthedocs.io/en/latest/loadcases.html
- Source type: open-source software documentation
- Date: 2026-05-06
- Reliability/limits: linear-analysis tool context
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/loads/load-cases-and-combinations.md`
- Used by: `docs/knowledge/raw/loads-load-cases-combinations-research.md`, `docs/knowledge/wiki/loads/load-cases-and-combinations.md`

### SRC-6109f8d859 — Model Commands / Constraints

- Bucket: `software_manual`
- Locator: https://opensees.github.io/OpenSeesDocumentation/user/manual/modelCommands.html
- Source type: open-source solver documentation
- Date: 2026-05-06
- Reliability/limits: solver/API-level terminology
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/modeling/supports-restraints-and-releases.md`
- Used by: `docs/knowledge/raw/modeling-supports-restraints-releases-research.md`, `docs/knowledge/wiki/modeling/supports-restraints-and-releases.md`

### SRC-2c4eebe57e — Model debugging

- Bucket: `software_manual`
- Locator: https://docs.oasys-software.com/structural/gsa/version/10.2.12/tutorials/model-debugging/
- Source type: public software documentation
- Date: 2026-05-06
- Reliability/limits: product workflow
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/diagnostics/instability-mechanisms.md`
- Used by: `docs/knowledge/raw/diagnostics-instability-mechanisms-research.md`, `docs/knowledge/wiki/diagnostics/instability-mechanisms.md`

### SRC-e00fd82962 — OpenSees Documentation, “EqualDOF Constraints”

- Bucket: `software_manual`
- Locator: https://opensees.github.io/OpenSeesDocumentation/user/manual/model/mp_constraint/equalDOF.html
- Source type: https://opensees.github.io/OpenSeesDocumentation/user/manual/model/mp_constraint/equalDOF.html — Retrieved 2026-05-06. Source type/context: open-source solver documentation
- Date: 2026-05-06
- Reliability/limits: low-level retained/constrained-node terminology.
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: none
- Used by: `docs/knowledge/raw/modeling-supports-restraints-releases-research.md`

### SRC-7ac60e4662 — OpenSees Documentation, “SP_Constraint Commands”

- Bucket: `software_manual`
- Locator: https://opensees.github.io/OpenSeesDocumentation/user/manual/model/spConstraints.html
- Source type: https://opensees.github.io/OpenSeesDocumentation/user/manual/model/spConstraints.html — Retrieved 2026-05-06. Source type/context: open-source solver documentation
- Date: 2026-05-06
- Reliability/limits: command-oriented.
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: none
- Used by: `docs/knowledge/raw/modeling-supports-restraints-releases-research.md`

### SRC-0e5bfea297 — OpenSees Documentation, “Sp Command”

- Bucket: `software_manual`
- Locator: https://opensees.github.io/OpenSeesDocumentation/user/manual/model/pattern/PlainPatternloadcommands/sp.html
- Source type: https://opensees.github.io/OpenSeesDocumentation/user/manual/model/pattern/PlainPatternloadcommands/sp.html — Retrieved 2026-05-06. Source type/context: open-source solver documentation
- Date: 2026-05-06
- Reliability/limits: OpenSees-specific load-factor semantics.
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: none
- Used by: `docs/knowledge/raw/modeling-supports-restraints-releases-research.md`

### SRC-da4a6f9433 — OpenSees Documentation, “ZeroLength Element”

- Bucket: `software_manual`
- Locator: https://opensees.github.io/OpenSeesDocumentation/user/manual/model/elements/zeroLength.html
- Source type: https://opensees.github.io/OpenSeesDocumentation/user/manual/model/elements/zeroLength.html — Retrieved 2026-05-06. Source type/context: open-source solver documentation
- Date: 2026-05-06
- Reliability/limits: implementation-level modeling primitive.
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: none
- Used by: `docs/knowledge/raw/modeling-supports-restraints-releases-research.md`

### SRC-0009e8c2c9 — ST7-1.10.10.2 Rigid Body Modes and Singularity Warning in Static Solvers

- Bucket: `software_manual`
- Locator: local-source-alias:strand7/ST7-1.10.10.2-rigid-body-modes-and-singularity-warning-in-static-solvers.pdf
- Source type: software manual/tutorial
- Date: 2026-05-06
- Reliability/limits: practical vendor guidance for static-solver diagnostics; useful for generic instability, singularity, release, property, and ill-conditioning patterns, but warning IDs, UI steps, element-specific handling, and numeric thresholds are Strand7-specific
- Pages used: not yet recorded
- Public rebuild eligible: false
- Rebuild action: `defer_private_local_source`
- Flags: private_local_deferred
- Current wiki pages: `docs/knowledge/wiki/diagnostics/instability-mechanisms.md`
- Used by: `docs/knowledge/wiki/diagnostics/instability-mechanisms.md`

### SRC-4a5532146b — ST7-1.10.10.2 Rigid Body Modes and Singularity Warning in Static Solvers

- Bucket: `software_manual`
- Locator: OneDrive-Personal/Engineering/Strand7/10. Linear/ST7-1.10.10.2 Rigid Body Modes and Singularity Warning in Static Solvers.pdf
- Source type: practical/software reference
- Date: 2026-05-06
- Reliability/limits: local software manual/tutorial; useful for generic solver diagnostic patterns, but warning IDs, UI steps, element-specific handling, and numeric thresholds are software-specific
- Pages used: not yet recorded
- Public rebuild eligible: false
- Rebuild action: `defer_private_local_source`
- Flags: private_local_deferred
- Current wiki pages: none
- Used by: `docs/knowledge/sources.md`

### SRC-160832a054 — Stability

- Bucket: `software_manual`
- Locator: https://pynite.readthedocs.io/en/latest/stability.html
- Source type: open-source software documentation
- Date: 2026-05-06
- Reliability/limits: software-specific but clear taxonomy
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/diagnostics/instability-mechanisms.md`
- Used by: `docs/knowledge/raw/diagnostics-instability-mechanisms-research.md`, `docs/knowledge/wiki/diagnostics/instability-mechanisms.md`

### SRC-d5dfe23d11 — Stability

- Bucket: `software_manual`
- Locator: https://help.risa.com/risahelp/risa3d/Content/Stability/Stability.htm
- Source type: public software help
- Date: 2026-05-06
- Reliability/limits: product examples
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/diagnostics/instability-mechanisms.md`
- Used by: `docs/knowledge/raw/diagnostics-instability-mechanisms-research.md`, `docs/knowledge/wiki/diagnostics/instability-mechanisms.md`

### SRC-6aeef56975 — Structural Load Determination sample

- Bucket: `software_manual`
- Locator: https://shop.iccsafe.org/media/wysiwyg/material/4034S18-Sample.pdf
- Source type: public sample
- Date: 2026-05-06
- Reliability/limits: ASCE/IBC educational excerpt, not universal
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/loads/gravity-and-lateral-loads.md`
- Used by: `docs/knowledge/raw/loads-gravity-lateral-loads-research.md`, `docs/knowledge/wiki/loads/gravity-and-lateral-loads.md`

### SRC-95ed5742ce — StructuralLoadCase

- Bucket: `software_manual`
- Locator: https://www.saf.guide/en/stable/loads/structuralloadcase.html
- Source type: public schema documentation
- Date: 2026-05-06
- Reliability/limits: exchange-format oriented
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/loads/load-cases-and-combinations.md`
- Used by: `docs/knowledge/raw/loads-load-cases-combinations-research.md`, `docs/knowledge/wiki/loads/load-cases-and-combinations.md`

### SRC-2dadd09595 — Warning: stiffness matrix is singular

- Bucket: `software_manual`
- Locator: https://scia.net/en/support/faq/scia-engineer/analysis/warning-stiffness-matrix-singular-structure-unstable
- Source type: public software help
- Date: 2026-05-06
- Reliability/limits: product workflow
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/diagnostics/instability-mechanisms.md`
- Used by: `docs/knowledge/raw/diagnostics-instability-mechanisms-research.md`, `docs/knowledge/wiki/diagnostics/instability-mechanisms.md`

### SRC-ebc747fb25 — rigidDiaphragm command

- Bucket: `software_manual`
- Locator: https://opensees.github.io/OpenSeesDocumentation/user/manual/model/mp_constraint/rigidDiaphragm.html
- Source type: open-source solver documentation
- Date: 2026-05-07
- Reliability/limits: useful source-scoped evidence for rigid diaphragm multi-point constraints; not a design guide
- Pages used: not yet recorded
- Public rebuild eligible: true
- Rebuild action: `eligible_for_public_rebuild_seed`
- Flags: none
- Current wiki pages: `docs/knowledge/wiki/modeling/constraints-rigid-links-and-diaphragms.md`
- Used by: `docs/knowledge/raw/modeling-supports-restraints-releases-research.md`, `docs/knowledge/wiki/modeling/constraints-rigid-links-and-diaphragms.md`
