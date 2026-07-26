# Gravity and lateral loads — concept brief

Retrieval date for all sources: 2026-05-06.

## Scope / non-scope

**Scope:** concept-level wiki seed for building structural loads: gravity loads (dead, live, roof live, snow/rain as vertical environmental loads) and lateral loads (wind, seismic, soil/flood where relevant), load paths, combinations, and high-level modeling implications for Fraia.

**Non-scope:** code design procedures, jurisdiction-specific values, detailed ASCE/IBC equations, bridge/offshore/nuclear-specialty loading, final design advice, or copied textbook/code formula content.

## Source-backed claims

1. **Modern building load standards treat loads as a coordinated hazard set, not isolated labels.** ASCE 7-22 covers dead, live, soil, flood, tsunami, snow, rain, atmospheric ice, seismic, wind, fire, and load combinations for general structural design. Fraia should model load *types* and *combinations* as first-class but separate concepts. [ASCE](https://www.asce.org/publications-and-news/codes-and-standards/asce-sei-7-22)

2. **Gravity loads include permanent and variable actions.** Dead load is essentially permanent/self-weight and fixed construction/equipment; live, roof live, snow, rain, wind, and earthquake effects are variable, and combinations may be critical when some variable actions are absent. [ICC sample: Structural Load Determination](https://shop.iccsafe.org/media/wysiwyg/material/4034S18-Sample.pdf)

3. **Load combinations are a design abstraction over plausible simultaneity, not a simple sum of all maxima.** IBC/ASCE combination logic distinguishes strength vs allowable-stress checks, applies different factors, and requires considering variable loads set to zero when that governs. [ICC sample: Structural Load Determination](https://shop.iccsafe.org/media/wysiwyg/material/4034S18-Sample.pdf)

4. **Snow loading is a gravity load but is strongly shape-, exposure-, and operations-dependent.** FEMA notes roof snow load starts from ground snow load but is modified by occupancy/use, wind exposure, roof slope/shape, obstructions, and thermal condition; unbalanced drifting/sliding can be more severe than uniform snow. [FEMA P-957](https://www.fema.gov/sites/default/files/documents/fema957_snowload_guide.pdf)

5. **Wind and seismic design depend on a continuous load path.** Public guidance describes vertical gravity/uplift and horizontal wind/seismic loads being transferred through roof, walls, floors, foundations, and connections to the ground; connection failures can redirect loads and trigger progressive failure. [Building America / PNNL](https://basc.pnnl.gov/resource-guides/continuous-load-path-provided-connections-roof-through-wall-foundation)

6. **Seismic loads are inertial effects tied to mass, ground motion, site conditions, structural system, and occupancy risk.** FEMA explains seismic design categories increase requirements based on shaking intensity and use/occupancy; important seismic performance features include stable foundations, continuous load path, strength/stiffness, regularity, redundancy, ductility/toughness, and rugged nonstructural components. [FEMA P-749](https://www.fema.gov/sites/default/files/2020-07/fema_earthquake-resistant-design-concepts_p-749.pdf)

7. **Lateral-resisting system identity matters.** FEMA categorizes seismic structural systems by how lateral forces are resisted: bearing wall systems, building frame systems, moment-resisting frames, dual systems, cantilever column systems, and systems not specifically detailed for seismic resistance. Fraia should not infer lateral behavior from member geometry alone. [FEMA P-749](https://www.fema.gov/sites/default/files/2020-07/fema_earthquake-resistant-design-concepts_p-749.pdf)

8. **Environmental loads are code-version and locality sensitive.** ASCE 7-22 revised ground snow loads and drift estimation, added tornado provisions, and updates standards on a cycle; FEMA also warns older buildings may predate modern drift/unbalanced snow requirements. Store load-source provenance and code edition metadata. [ASCE](https://www.asce.org/publications-and-news/codes-and-standards/asce-sei-7-22), [FEMA P-957](https://www.fema.gov/sites/default/files/documents/fema957_snowload_guide.pdf)

## Fraia modeling implications

- Represent **load assignments** separately from structural objects: point, line, area, body/self-weight, pressure, imposed displacement/temperature, and derived equivalent nodal/member actions.
- Keep **load nature** explicit: permanent vs variable; gravity vs lateral; environmental vs occupancy; static vs dynamic/equivalent-static.
- Model **load cases** independently from **load combinations** and **analysis/design runs**.
- Attach loads to canonical Fraia primitives: `Node`, `Member`, `Plate`, `SupportAssignment`, with clear local/global direction and coordinate frame.
- Include **tributary/load transfer** as a derived or realization-stage concept, not hidden inside authored loads.
- Store provenance: source standard/manual, code edition, hazard map/version, jurisdiction assumptions, risk/occupancy category, site class/exposure where used.
- Support lateral-system annotations such as braced frame, shear wall, moment frame, diaphragm, collector/chord, and foundation load path without replacing primitive members/plates.
- Treat self-weight as derivable from material/section/geometry but overrideable and traceable.

## Cautions

- Do not present wiki seed values as design-ready loads; actual projects require adopted local code, hazard data, occupancy/risk category, site data, and professional judgment.
- Avoid hardcoding ASCE/IBC formulas in concept pages; cite standards and keep Fraia examples schematic.
- Do not collapse seismic and wind into generic “horizontal load”; their sources, directions, combinations, dynamic behavior, and detailing implications differ.
- Do not assume uniform area load is sufficient for snow, wind pressure, seismic mass distribution, or diaphragm/collector behavior.
- Existing buildings may have design bases that differ from current codes; renovations, rooftop equipment, reroofing, and additions can change load demand and load path.

## Suggested cross-links

- Structural loads overview
- Dead load and self-weight
- Live load / imposed load
- Snow and rain roof loads
- Wind load and uplift
- Seismic load and seismic mass
- Load paths and continuous connections
- Load cases vs load combinations
- Tributary area and load distribution
- Diaphragms, collectors, chords, and lateral-force-resisting systems
- Supports, reactions, and foundations
- Authored loads vs analysis realization artifacts

## Source list

- ASCE, **Minimum Design Loads and Associated Criteria for Buildings and Other Structures, ASCE/SEI 7-22 overview** — https://www.asce.org/publications-and-news/codes-and-standards/asce-sei-7-22 — authoritative public overview of load hazards, combinations, and 2022 updates. Retrieved 2026-05-06.
- ICC / David A. Fanella, **Structural Load Determination: 2018 IBC® and ASCE/SEI 7-16 sample** — https://shop.iccsafe.org/media/wysiwyg/material/4034S18-Sample.pdf — public sample explaining permanent vs variable loads and combination concepts. Retrieved 2026-05-06.
- FEMA, **FEMA P-957 Snow Load Safety Guide** — https://www.fema.gov/sites/default/files/documents/fema957_snowload_guide.pdf — public guidance on snow load variables, roof geometry, drifting/sliding, and existing-building cautions. Retrieved 2026-05-06.
- Building America Solution Center / PNNL, **Continuous Load Path Provided with Connections from the Roof through the Wall to the Foundation** — https://basc.pnnl.gov/resource-guides/continuous-load-path-provided-connections-roof-through-wall-foundation — public DOE/PNNL guidance on vertical/horizontal load transfer and connection importance. Retrieved 2026-05-06.
- FEMA, **FEMA P-749 Earthquake-Resistant Design Concepts** — https://www.fema.gov/sites/default/files/2020-07/fema_earthquake-resistant-design-concepts_p-749.pdf — public seismic concepts source covering risk, SDCs, load path, stiffness/strength, regularity, redundancy, ductility, and system categories. Retrieved 2026-05-06.
- Building America Solution Center / PNNL, **Minimum Design Loads for Buildings and Other Structures, ASCE/SEI 7-10 library page** — https://basc.pnnl.gov/library/minimum-design-loads-buildings-and-other-structures-ascesei-7-10 — public summary confirming ASCE 7 scope and standard lineage. Retrieved 2026-05-06.
