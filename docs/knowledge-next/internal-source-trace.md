# Internal Source Trace

_Status: audit finding_
_Date: 2026-06-15_

This traces the 16 inventory entries whose `rebuild_action` is `trace_to_original_public_source`.

These entries are internal Fraia docs or compiled wiki pages currently cited by other wiki pages. For the public-only rebuild, they are not eligible source material by themselves. Use them as breadcrumbs only.

## Summary

- Domain knowledge pages usually already have public/open sources on their own page. Rebuild from those public sources, not from the internal cross-reference.
- Product/pipeline pages describe Fraia architecture. Keep them as internal product rationale, not public engineering evidence.
- Public provenance/lineage standards can support general claims about provenance, runs, inputs, outputs, and trust, but they do not externally validate Fraia-specific architecture choices.
- The topic map and knowledge workflow are registry/governance docs, not source evidence.

## Public Provenance / Lineage Sources For Product Pages

Use these only for general provenance and lineage concepts:

- W3C, *PROV-DM: The PROV Data Model*. URL: https://www.w3.org/TR/prov-dm/. Source type: public W3C recommendation. Reliability/limits: strong general provenance model; not structural-engineering-specific and not Fraia architecture.
- W3C, *PROV-Overview*. URL: https://www.w3.org/TR/prov-overview/. Source type: public W3C overview. Reliability/limits: useful orientation to PROV family; non-normative overview.
- OpenLineage, *Object Model*. URL: https://openlineage.io/docs/next/spec/object-model/. Source type: public lineage specification documentation. Reliability/limits: useful run/job/dataset lineage concepts; data-pipeline-specific and not structural-engineering-specific.
- OpenLineage, *Facets & Extensibility*. URL: https://openlineage.io/docs/spec/facets/. Source type: public lineage specification documentation. Reliability/limits: useful extensible metadata model for runs/jobs/datasets; not Fraia artifact schema.
- Research Object community, *Workflow Run RO-Crate / Provenance Run Crate*. URL: https://www.researchobject.org/workflow-run-crate/profiles/0.2/provenance_run_crate.html. Source type: public workflow provenance profile. Reliability/limits: useful workflow-run provenance model; research-workflow-specific.

## The 16 Entries

### SRC-86e2e8a778 — Analysis Result Review Before Design Checks

Disposition: rebuild from public domain sources plus internal Fraia architecture note.

Public source replacements:

- Reaction review: use the public sources listed under `Reaction Sanity Checks`, `Reactions and Support Idealisation`, and `Free-Body Diagrams and Equilibrium`.
- Instability/result trust: use public sources listed under `Instability Mechanisms`, excluding the deferred private/local Strand7 entry.
- Design/check separation: use `SteelConstruction.info, Member design` for steel check-context claims and the public provenance/lineage sources above for general artifact lineage.

Notes: the claim that Fraia gates design-action extraction after reviewed analysis results is a Fraia architecture decision. Keep it as internal rationale unless tied to a public provenance/lineage concept.

### SRC-867d2ce54e — Authored/Resolved/Run Artifact Boundaries

Disposition: keep as internal Fraia product architecture, with optional public provenance support.

Public support:

- W3C PROV-DM / PROV-Overview for entity/activity/agent provenance and trust assessment.
- OpenLineage Object Model / Facets for run/job/dataset metadata lineage.
- Workflow Run RO-Crate for computational workflow-run provenance.

Notes: no public structural-engineering source will define Fraia's authored/resolved/run boundary. The rebuild should not treat this as public engineering knowledge; it should remain a product architecture card or internal schema rationale.

### SRC-e441704512 — Design Actions, Check Inputs, and Check Results

Disposition: split into internal Fraia artifact vocabulary plus public steel/check context.

Public source replacements:

- SteelConstruction.info, *Member design*. URL: https://www.steelconstruction.info/Member_design.
- Public provenance/lineage sources above for run, input, output, and metadata traceability.

Notes: `design_action`, `check_input`, and `check_result` are Fraia terms. Public sources can support the need for design context beyond raw forces, but the artifact split remains internal product design.

### SRC-6884636c8c — Engineering Assumptions and Provenance

Disposition: rebuild general provenance claims from public provenance sources; keep Fraia-specific assumption workflow as internal product policy.

Public source replacements:

- W3C PROV-DM / PROV-Overview.
- OpenLineage Facets & Extensibility.
- Workflow Run RO-Crate / Provenance Run Crate.

Notes: claims about source type, confidence, affected layer, and project approval should become Fraia policy fields, not externally sourced structural-engineering facts.

### SRC-f401f081fd — Engineering Output Pipeline

Disposition: keep as internal Fraia architecture, with public lineage/provenance support for general traceability.

Public support:

- W3C PROV-DM / PROV-Overview.
- OpenLineage Object Model / Facets.
- Workflow Run RO-Crate / Provenance Run Crate.

Notes: exports-as-renderers and typed pipeline stages are Fraia design decisions. Public sources can support provenance and workflow-run traceability, not the exact stage names.

### SRC-24058e8370 — Fraia Knowledge Topic Map

Disposition: do not use as source evidence.

Public source replacements:

- None directly. Trace each topic to its compiled page and then to that page's public sources.

Notes: the topic map is a roadmap and registry. It can guide coverage, but it should not be cited as evidence for engineering claims.

### SRC-63420f1192 — Free-Body Diagrams and Equilibrium

Disposition: replace internal citations with the page's public sources.

Public source replacements:

- Daniel W. Baker and William Haynes / Engineering LibreTexts, *Free Body Diagrams*. URL: https://eng.libretexts.org/Bookshelves/Mechanical_Engineering/Engineering_Statics:_Open_and_Interactive_(Baker_and_Haynes)/05:_Rigid_Body_Equilibrium/5.02:_Free_Body_Diagrams.
- OpenStax / Physics LibreTexts, *Conditions for Static Equilibrium*. URL: https://phys.libretexts.org/Bookshelves/University_Physics/University_Physics_(OpenStax)/Book:_University_Physics_I_-_Mechanics_Sound_Oscillations_and_Waves_(OpenStax)/12:_Static_Equilibrium_and_Elasticity/12.02:_Conditions_for_Static_Equilibrium.
- Felix Udoeyo / Engineering LibreTexts, *Equilibrium Structures, Support Reactions, Determinacy and Stability of Beams and Frames*. URL: https://eng.libretexts.org/Bookshelves/Civil_Engineering/Structural_Analysis_(Udoeyo)/01:_Chapters/1.03:_Equilibrium_Structures_Support_Reactions_Determinacy_and_Stability_of_Beams_and_Frames.

### SRC-410825b32b — Instability Mechanisms

Disposition: replace internal citations with public software/manual sources; defer private Strand7.

Public source replacements:

- PyNite Documentation, *Stability*. URL: https://pynite.readthedocs.io/en/latest/stability.html.
- SCIA, *Warning: stiffness matrix is singular*. URL: https://scia.net/en/support/faq/scia-engineer/analysis/warning-stiffness-matrix-singular-structure-unstable.
- RISA-3D Help, *Stability*. URL: https://help.risa.com/risahelp/risa3d/Content/Stability/Stability.htm.
- LUSAS, *Connectivity FAQ*. URL: https://www.lusas.com/user_area/faqs/connectivity.html.
- Dlubal, *Finding and Fixing Calculation Instabilities*. URL: https://www.dlubal.com/en/support-and-learning/support/faq/005345.
- Oasys GSA, *Model debugging*. URL: https://docs.oasys-software.com/structural/gsa/version/10.2.12/tutorials/model-debugging/.

Notes: do not use the local/private Strand7 entry for the public-only rebuild.

### SRC-984798d4c9 — Knowledge Wiki Workflow

Disposition: keep as internal governance, not source evidence.

Public source replacements:

- None required for structural engineering claims.
- If provenance claims are needed, use W3C PROV / OpenLineage / Workflow Run RO-Crate.

Notes: this defines Fraia maintenance workflow. It should inform process, not seed engineering knowledge cards.

### SRC-f703d7c88a — Load Application and Equivalent Nodal Loads

Disposition: replace internal citations with the page's public sources.

Public source replacements:

- Daniel W. Baker and William Haynes / Engineering LibreTexts, *Distributed Loads*. URL: https://eng.libretexts.org/Bookshelves/Mechanical_Engineering/Engineering_Statics:_Open_and_Interactive_(Baker_and_Haynes)/07:_Centroids_and_Centers_of_Gravity/7.08:_Distributed_Loads.
- Engineering LibreTexts / Aerospace Structures, *Direct stiffness method*. URL: https://eng.libretexts.org/Under_Construction/Aerospace_Structures_(Johnson)/15:_Direct_stiffness_method.
- Engineering LibreTexts / Aerospace Structures, *Applications of the direct stiffness method*. URL: https://eng.libretexts.org/Under_Construction/Aerospace_Structures_(Johnson)/16:_Applications_of_the_direct_stiffness_method.
- Engineering LibreTexts / Aerospace Structures, *Finite element method*. URL: https://eng.libretexts.org/Under_Construction/Aerospace_Structures_(Johnson)/17:_Finite_element_method.

### SRC-a1fe01fe83 — Member End Releases

Disposition: replace internal citations with the page's public sources.

Public source replacements:

- SteelConstruction.info / SCI and BCSA, *Modelling and analysis*. URL: https://steelconstruction.info/Modelling_and_analysis.
- N. S. Trahair and M. A. Bradford, *Member end releases in framed structures*. URL: https://www.sciencedirect.com/science/article/abs/pii/004579499390214X.
- OpenSees Documentation, *Elastic Beam Column Element*. URL: https://opensees.github.io/OpenSeesDocumentation/user/manual/model/elements/elasticBeamColumn.html.

Notes: the ScienceDirect source is abstract-level access only; do not rely on inaccessible details.

### SRC-3f4ddb6043 — Reaction Sanity Checks

Disposition: rebuild from transitive public sources.

Public source replacements:

- Reactions/supports: Udoeyo / Engineering LibreTexts equilibrium/support reactions, TU Delft *Supports*, and OpenStax static equilibrium.
- Free-body/equilibrium: Baker and Haynes / Engineering LibreTexts *Free Body Diagrams*.
- Resolved/equivalent loads: Baker and Haynes *Distributed Loads* plus LibreTexts direct-stiffness/FEM pages.

Notes: reaction sanity checking itself is a Fraia diagnostic workflow. The public sources support the physics/analysis basis, not Fraia's final tolerance/report schema.

### SRC-c240fbba26 — Reactions and Support Idealisation

Disposition: replace internal citations with the page's public sources.

Public source replacements:

- Felix Udoeyo / Engineering LibreTexts, *Equilibrium Structures, Support Reactions, Determinacy and Stability of Beams and Frames*. URL: https://eng.libretexts.org/Bookshelves/Civil_Engineering/Structural_Analysis_(Udoeyo)/01:_Chapters/1.03:_Equilibrium_Structures_Support_Reactions_Determinacy_and_Stability_of_Beams_and_Frames.
- Tom van Woudenberg / Delft University of Technology, *Supports*. URL: https://oit.tudelft.nl/CEG-mechanics-BSc/support_internal_forces/model/supports.html.
- OpenStax, *University Physics Volume 1: 12.1 Conditions for Static Equilibrium*. URL: https://openstax.org/books/university-physics-volume-1/pages/12-1-conditions-for-static-equilibrium.

### SRC-854b1c9dbb — Resolution and Runs

Disposition: keep as internal Fraia architecture, with optional public provenance support.

Public support:

- W3C PROV-DM / PROV-Overview.
- OpenLineage Object Model / Facets.
- Workflow Run RO-Crate / Provenance Run Crate.

Notes: authored/resolved/frozen-run separation is a Fraia architecture choice. Public sources support provenance and run traceability, not the exact Fraia representation split.

### SRC-fe0bc7d06d — Steel Design Action and Check-Input Separation

Disposition: split into public steel behavior source plus internal artifact vocabulary.

Public source replacements:

- SteelConstruction.info / SCI and BCSA, *Member design*. URL: https://www.steelconstruction.info/Member_design.
- Public provenance/lineage sources above for run-to-output traceability.

Notes: public steel sources support why raw actions are insufficient without restraint, buckling, LTB, section, and combined-action context. The exact design-action/check-input/check-result artifact vocabulary is Fraia-specific.

### SRC-6776d0f8d7 — Truss Analysis and Two-Force Members

Disposition: replace internal citations with the page's public sources.

Public source replacements:

- Felix Udoeyo / Engineering LibreTexts, *Internal Forces in Plane Trusses*. URL: https://eng.libretexts.org/Bookshelves/Civil_Engineering/Structural_Analysis_(Udoeyo)/01:_Chapters/1.05:_Internal_Forces_in_Plane_Trusses.
- Daniel W. Baker and William Haynes / Engineering LibreTexts, *Method of Joints*. URL: https://eng.libretexts.org/Bookshelves/Mechanical_Engineering/Engineering_Statics:_Open_and_Interactive_(Baker_and_Haynes)/06:_Equilibrium_of_Structures/6.04:_Method_of_Joints.
- Jacob Moore and contributors / Engineering LibreTexts, *Method of Sections*. URL: https://eng.libretexts.org/Bookshelves/Mechanical_Engineering/Mechanics_Map_(Moore_2nd_Edition)/05:_Engineering_Structures/5.05:_Method_of_Sections.

## Recommended Rebuild Actions

1. For domain pages, replace internal `## Sources` entries with the listed public sources before converting to `KnowledgeCard` records.
2. For product/pipeline pages, split the content into:
   - internal Fraia architecture cards or schema docs
   - public-source-backed provenance/lineage cards only where W3C/OpenLineage/RO-Crate genuinely apply
3. Do not cite `topic-map.md` or `workflow.md` as evidence for engineering claims.
4. Keep all private/local sources deferred until the public-only rebuild phase is complete.
