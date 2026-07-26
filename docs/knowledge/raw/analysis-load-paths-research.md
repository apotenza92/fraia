# Raw research brief: Structural engineering load paths

Retrieval date for all sources: 2026-05-06. Public/open sources only; no textbook passages copied.

## Scope
- Define load paths/load tracing for structural wiki seed content.
- Cover gravity and lateral load paths, continuous load paths, connections, foundations/soil, temporary conditions, alterations, and simple tributary-area concepts.
- Translate findings into Fraia wiki/data-model implications for authored structural primitives (`Node`, `Member`, `Plate`, supports, loads, releases) and downstream analysis/design artifacts.

## Non-scope
- No jurisdiction-specific design procedure or code-compliance checklist.
- No copyrighted textbook extraction or reproduction of proprietary standards.
- No member sizing tables, connector schedules, or sealed engineering advice.
- No exhaustive treatment of dynamic analysis, nonlinear redistribution, or progressive-collapse design.

## Source-backed claims
1. **A load path is the route by which imposed loads are transferred through a structure to the ground/soil.** TU Delft frames the basic concept as loads being introduced into a structure and needing to be dissipated somewhere; Pressbooks defines a load path as the route loads follow through a structure into the ground, with a typical gravity sequence of roof/snow → sheathing/joists → beams/girders → columns → foundation → soil. [TU Delft OCW](https://ocw.tudelft.nl/course-readings/4-2-1-introduction-to-load-paths/); [Pressbooks structural systems/load tracing](https://saalck.pressbooks.pub/structuralconceptsforarchitectsandconstructionmanagers/chapter/module-4-structural-systems-and-load-tracing/)
2. **Load tracing is a system-level activity, not just checking isolated member strength.** ICE emphasizes that engineers must know the path for every load and verify that every element in the path can carry it; it also warns that global behavior can govern even when individual components appear adequate. [ICE Knowledge Hub](https://knowledgehub.ice.org.uk/cpd/delivery-exc/structural-load-paths/)
3. **Gravity load paths commonly step from surfaces to secondary members to primary members to vertical supports and then foundations/soil.** Open teaching material describes slabs/roofs distributing loads to joists/purlins, then beams/girders/trusses, then columns/foundations; AWC describes houses as structural systems that transfer building loads through the foundation to supporting soil. [Pressbooks structural systems/load tracing](https://saalck.pressbooks.pub/structuralconceptsforarchitectsandconstructionmanagers/chapter/module-4-structural-systems-and-load-tracing/); [American Wood Council span-table tutorial](https://awc.org/codes-standards/spantables/tutorial)
4. **Lateral load paths can differ from gravity load paths and require specific resisting systems.** ICE notes that wind and earthquake load paths often include bracing, shear walls, and/or cores; DOE/PNNL Building America Solution Center describes horizontal loads including out-of-plane wind, in-plane shear, flood, and seismic effects that must be transferred to ground. [ICE Knowledge Hub](https://knowledgehub.ice.org.uk/cpd/delivery-exc/structural-load-paths/); [BASC/PNNL continuous load path guide](https://basc.pnnl.gov/resource-guides/continuous-load-path-provided-connections-roof-through-wall-foundation)
5. **Connections are often the weak links in load paths.** ICE explicitly includes connections as part of every load path and warns that bolts/welds may be adequate while plates/details are missed; BASC similarly states connections between roof, wall, floor, and foundation assemblies tend to be the weakest link. [ICE Knowledge Hub](https://knowledgehub.ice.org.uk/cpd/delivery-exc/structural-load-paths/); [BASC/PNNL continuous load path guide](https://basc.pnnl.gov/resource-guides/continuous-load-path-provided-connections-roof-through-wall-foundation)
6. **Continuous load paths are especially important for wind/seismic uplift and shear.** BASC states buildings need strong continuous paths from roof to foundation to remain intact in hurricanes, high winds, and earthquakes, transferring vertical gravity/uplift and horizontal loads to ground. FEMA-derived public guidance summarized by BASC shows roof-to-wall, wall-to-wall, and wall-to-foundation ties as key links. [BASC/PNNL continuous load path guide](https://basc.pnnl.gov/resource-guides/continuous-load-path-provided-connections-roof-through-wall-foundation)
7. **Openings and discontinuities reroute loads and can create concentrated demands.** BASC notes loads must be routed around windows/doors and that accumulated loads on headers are transferred to studs at the sides of openings; ICE notes missing columns or atria can force loads sideways and increase demands in remaining columns. [BASC/PNNL continuous load path guide](https://basc.pnnl.gov/resource-guides/continuous-load-path-provided-connections-roof-through-wall-foundation); [ICE Knowledge Hub](https://knowledgehub.ice.org.uk/cpd/delivery-exc/structural-load-paths/)
8. **Tributary area is a practical load-tracing abstraction for distributing surface loads to members.** Pressbooks defines tributary area as the portion of floor/roof/wall surface contributing load to a member and uses it to convert surface loads into line loads on beams and point loads on columns. [Pressbooks structural systems/load tracing](https://saalck.pressbooks.pub/structuralconceptsforarchitectsandconstructionmanagers/chapter/module-4-structural-systems-and-load-tracing/)
9. **The load path does not stop at the foundation; soil bearing/settlement is the final link.** AWC states loads are transferred through the foundation to supporting soil, and Pressbooks explicitly treats the load path as ending in soil, with settlement/bearing capacity affecting performance. [American Wood Council span-table tutorial](https://awc.org/codes-standards/spantables/tutorial); [Pressbooks structural systems/load tracing](https://saalck.pressbooks.pub/structuralconceptsforarchitectsandconstructionmanagers/chapter/module-4-structural-systems-and-load-tracing/)
10. **Temporary conditions, construction sequence, alteration, and demolition can create different load paths than the permanent structure.** ICE reports that many collapses occur during construction/alteration/demolition, gives examples where construction support conditions differed from final support conditions, and lists constructability questions designers should ask. [ICE Knowledge Hub](https://knowledgehub.ice.org.uk/cpd/delivery-exc/structural-load-paths/)

## Fraia-specific implications
- Model load paths as derived/inspectable artifacts, not as authored truth: authored primitives remain members/plates/nodes/supports/loads/releases; a resolved/load-tracing view can reference them.
- Wiki seed should introduce two parallel concepts: **gravity tracing** (surface load → plate/member tributaries → supports → foundation/soil) and **lateral tracing** (cladding/diaphragm/plate actions → collectors/bracing/shear walls/frames → anchorage/foundation).
- Connections and releases deserve first-class visibility in load-path explanations; a missing/weak release/connection assumption can invalidate a path even when member capacity looks sufficient.
- Fraia could flag discontinuities: unsupported plate edges, load-bearing member removal, transfer beams, missing collector/diaphragm-to-frame connection, eccentric column loading, and unassigned supports/foundations.
- Tributary areas are a useful educational and preliminary-design layer, but should be clearly labeled approximate and separate from finite-element analysis results.
- Preserve provenance: each load-path segment should link back to structural objects and load cases/combinations, with run IDs for computed paths.
- Wiki examples should use Fraia canonical terms: role-labelled `Member` objects (beam, column, rafter, purlin, brace), `Plate` objects (slab, wall_panel, roof_panel), `SupportAssignment`, `LoadAssignment`, and `ReleaseAssignment`.

## Cautions / contradictions / limits
- Sources agree on the concept, but simplify differently: education sources describe top-down gravity paths; wind/seismic guidance stresses continuous roof-to-foundation ties and lateral/shear paths. Fraia should avoid presenting one path pattern as universal.
- Tributary-area methods assume idealized one-way distribution and regular geometry; FE plates, two-way slabs, diaphragms, arches, shells, transfer structures, and stiffness-dependent distribution require analysis.
- “Continuous load path” in residential wind guidance is connector-focused; in general structural engineering it also includes member stiffness/strength, stability, load combinations, foundations, and temporary states.
- Public sources are not substitutes for ASCE/Eurocode/IBC/IRC primary standard text or professional judgment. BASC quotes/summarizes code provisions but is guidance, not the legal standard.
- Building Science Corp. advanced framing claims are useful for direct/stacked load paths, but the article is opinionated practice commentary and residential wood-framing-specific.

## Suggested cross-links for Fraia wiki
- Structural primitives: `Node`, `Member`, `Plate`, support/load/release assignments.
- Loads and load cases: dead, live, snow, wind, seismic; load combinations.
- Tributary areas and load takedown.
- Diaphragms, collectors, shear walls, braced frames, moment frames.
- Connections, releases, and boundary conditions.
- Foundations, soil bearing, settlement, support assumptions.
- Construction sequencing, temporary works, shoring, alterations/demolition.
- Analysis topology: authored member vs discretized analysis elements.
- Provenance and run artifacts for derived load-path traces.

## Open questions
- What level of load-path automation is intended for MVP: educational diagrams, rule-based warnings, tributary load takedown, or full graph extraction from analysis reactions?
- Should Fraia represent foundations/soil as authored primitives now, or as support metadata until geotechnical features exist?
- How should connection capacity/detail assumptions be represented without turning Fraia into a code-detailing package too early?
- Which jurisdictions/design standards should later wiki pages map to, if any, while keeping seed content standard-neutral?
- Can Fraia derive alternative load paths/redundancy indicators from model topology and releases, or should this remain a manual review aid initially?

## Source list
- TU Delft OpenCourseWare, “4.2.1 Introduction to Load Paths” — public open course reading; source type: university OCW, CC BY-NC-SA context; useful for plain definition that loads introduced into structures must dissipate through a path; limits: very short and aerospace-course context, not building-code guidance. URL: https://ocw.tudelft.nl/course-readings/4-2-1-introduction-to-load-paths/ ; retrieved 2026-05-06.
- Institution of Civil Engineers Knowledge Hub, “Explainer: Structural load paths” — source type: professional engineering explainer; useful for system behavior, weakest-link connections, temporary conditions, alterations, alternative paths, progressive collapse examples; limits: article/explainer, not a standard, some case discussion is qualitative/speculative. URL: https://knowledgehub.ice.org.uk/cpd/delivery-exc/structural-load-paths/ ; retrieved 2026-05-06.
- American Wood Council, “Tutorial for Understanding Loads and Using Span Tables” — source type: industry technical tutorial; useful for load transfer through foundation to soil, dead/live/snow/wind concepts, strength vs stiffness, bearing checks; limits: residential wood joists/rafters and span-table context. URL: https://awc.org/codes-standards/spantables/tutorial ; retrieved 2026-05-06.
- Building America Solution Center / PNNL, “Continuous Load Path Provided with Connections from the Roof through the Wall to the Foundation” — source type: public DOE/PNNL building-science guide using FEMA/IBHS/IRC context; useful for continuous load path, vertical/uplift/lateral loads, connector chain, openings, roof-wall-foundation links; limits: house-focused and hazard/resilience guidance, not a structural design standard. URL: https://basc.pnnl.gov/resource-guides/continuous-load-path-provided-connections-roof-through-wall-foundation ; retrieved 2026-05-06.
- Pressbooks, “Module 4: Structural Systems and Load Tracing – Structural Concepts for Architects and Construction Managers” — source type: open educational resource; useful for structural systems, primary/secondary members, gravity/lateral tracing, tributary areas, soil/foundation link, construction relevance; limits: teaching material with simplified examples and some generated/secondary imagery. URL: https://saalck.pressbooks.pub/structuralconceptsforarchitectsandconstructionmanagers/chapter/module-4-structural-systems-and-load-tracing/ ; retrieved 2026-05-06.
- SUNY/Pressbooks, “Load Tracing – Basic Concepts of Structural Design for Architecture Students” — source type: open educational chapter found in search; useful corroboration for load tracing as following loads to ground; limits: less comprehensive than the kept Pressbooks chapter above. URL: https://structuraldesign.pressbooks.sunycreate.cloud/chapter/chapter-12-load-tracing/ ; retrieved 2026-05-06.
- FEMA P-499, “Home Builder’s Guide to Coastal Construction” — source type: public FEMA PDF referenced via BASC; useful context for coastal load-path fact sheets and wind/uplift connector diagrams; limits: large PDF not fully extracted in this run, residential/coastal focus. URL: https://www.fema.gov/sites/default/files/2020-07/p-499_homebuilders-guide-coastal-construction.pdf ; retrieved 2026-05-06.
- FEMA P-55, “Coastal Construction Manual” — source type: public FEMA PDF found in search; useful for continuous load path, structural connections, coastal hazards; limits: large PDF extraction failed/too large in this run, coastal-specific. URL: https://www.fema.gov/sites/default/files/2020-08/fema55_volii_combined_rev.pdf ; retrieved 2026-05-06.
- Building Science Corporation, “BSI-030: Advanced Framing” — source type: public practice article/commentary; useful for direct/stacked residential framing load paths and connection simplification; limits: opinionated, residential wood-framing-specific, not a standard. URL: https://buildingscience.com/documents/insights/bsi-030-advanced-framing ; retrieved 2026-05-06.

## Dropped / not relied on heavily
- ASCE 7 pages on ASCE Amplify — authoritative standard context, but full standard text is restricted/copyrighted; avoided quoting detailed provisions. URL seen: https://www.asce.org/publications-and-news/codes-and-standards/asce-sei-7-22
- FEMA NEHRP seismic provisions PDFs — authoritative public FEMA documents, but broad/large and not needed for concise seed beyond corroborating continuous seismic load-path concepts.
- SEO/general construction pages — excluded where they repeated definitions without primary/professional context.
