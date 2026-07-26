# Research: Load cases and load combinations for structural engineering concept wiki seed

Retrieval date for all sources: 2026-05-06. Scope: public/open web sources only. This is a concept-level brief, not design-code advice.

## Summary

A **load case** is a named grouping of loads/actions that are treated together, commonly by source or design situation (e.g. dead load, live load, wind from a direction, snow, construction-stage load). A **load combination** combines load cases with factors and/or accompanying-value reductions to test strength, stability, and serviceability under plausible simultaneous actions. Specific factor values, action categories, and required combinations are jurisdiction/code-dependent; Fraia should model the concepts generically and attach code templates as scoped presets, not universal rules.

## Findings / claims

1. **Load cases are containers for individual loads, usually grouped by action source.** SAF documentation says individual loads “must be included in load cases” and that a load case is commonly used to group loads from the same action source; it also distinguishes action type (Permanent, Variable, Accidental), load group, and load subtype such as self weight, wind, snow, fire, moving, or seismic. [SAF StructuralLoadCase](https://www.saf.guide/en/stable/loads/structuralloadcase.html)

2. **Load combinations are factor maps over load cases.** PyNite’s public wiki defines a load combination as load cases combined with different load factors, e.g. a unique combination name plus a dictionary `{D: 1.2, L: 1.6, S: 0.5}` and a type such as `strength` or `service`. [PyNite load cases & combinations](https://github.com/JWock82/Pynite/wiki/5.-Load-Cases-&-Load-Combinations)

3. **Analysis tools often preserve both case results and combination results.** anaStruct groups different loads in a `LoadCase`, then builds a `LoadCombination` by adding load cases with factors; its solve process returns separate results for each load case and for the whole combination. [anaStruct documentation](https://anastruct.readthedocs.io/en/latest/loadcases.html)

4. **The strength/serviceability split is foundational but expressed differently by codes.** LibreTexts summarizes that structures are designed for both strength (life/property safety) and serviceability (occupant comfort/aesthetics), then illustrates ASCE 7-16 LRFD and ASD combinations. Treat the listed equations as an educational ASCE 7-16 example, not a universal set. [LibreTexts load combinations](https://eng.libretexts.org/Bookshelves/Mechanical_Engineering/Introduction_to_Aerospace_Structures_and_Materials_(Alderliesten)/01:_Introduction_to_Structural_Analysis_and_Structural_Loads/02:_Structural_Loads_and_Loading_System/2.02:_Load_Combinations_for_Structural_Design)

5. **US ASCE 7 scope: hazard/load categories and combination loads are part of the standard, but the standard itself is copyrighted/paywalled.** ASCE’s public overview states ASCE 7-22 prescribes design loads for hazards including dead, live, soil, flood, tsunami, snow, rain, atmospheric ice, seismic, wind, and fire, and covers “Combination Loads.” It is adopted by reference in major US model codes. Use public summaries for wiki context, but do not reproduce proprietary tables/equations beyond open/public excerpts. [ASCE 7-22 overview](https://www.asce.org/publications-and-news/codes-and-standards/asce-sei-7-22)

6. **Eurocode scope: EN 1990 is the basis-of-design standard for safety, serviceability, durability, reliability, limit states, and combinations; EN 1991 supplies individual actions.** The European Commission/JRC Eurocodes page states EN 1990 establishes principles and requirements for structural safety/serviceability/durability and is used with EN 1991–EN 1999, including geotechnical, fire, earthquake, execution, and temporary structures. [JRC EN 1990 page](https://eurocodes.jrc.ec.europa.eu/EN-Eurocodes/eurocode-basis-structural-design)

7. **Eurocode uses “actions,” a broader term than direct loads.** A public EN 1990 explainer notes actions include direct loads and indirect actions such as imposed deformations or accelerations from temperature, uneven settlement, earthquakes, etc.; it classifies actions as permanent (G), variable (Q), and accidental (A). [First Principle Engineering, EN 1990 actions](https://knowledge.fppengineering.com/basis-of-actions-and-load-combinations-en-1990/)

8. **Combination logic includes leading and accompanying variable actions.** The EN 1990 explainer describes a leading variable action at characteristic value and accompanying variable actions reduced by ψ factors because independent variable actions are unlikely to reach maximum values simultaneously. Use this as a concept explanation; actual ψ and partial-factor values depend on Eurocode annexes/national annexes and project type. [First Principle Engineering, EN 1990 actions](https://knowledge.fppengineering.com/basis-of-actions-and-load-combinations-en-1990/)

9. **Limit states and design situations drive which combinations matter.** The EN 1990 explainer distinguishes ultimate limit states (e.g. EQU, STR, GEO, FAT, UPL, HYD), serviceability limit states, and persistent/transient/accidental/seismic design situations; combinations are sets of design values for verifying structural reliability for a limit state under simultaneous actions. [First Principle Engineering, EN 1990 combinations](https://knowledge.fppengineering.com/basis-of-actions-and-load-combinations-en-1990/)

10. **Codes/templates should be scoped by jurisdiction, edition, material/design method, occupancy, and limit state.** Public sources show ASCE examples tied to ASCE 7-16 or 7-22 and Eurocode examples tied to EN 1990 plus national annexes. Therefore Fraia should avoid a single built-in “correct” combination list without explicit code basis metadata. [ASCE 7-22 overview](https://www.asce.org/publications-and-news/codes-and-standards/asce-sei-7-22), [JRC EN 1990 page](https://eurocodes.jrc.ec.europa.eu/EN-Eurocodes/eurocode-basis-structural-design)

## Fraia implications

- **Canonical authored concepts:** model `LoadAssignment` objects as authored loads attached to nodes/members/plates/supports/etc.; require each load to reference a named `LoadCase`.
- **Keep load cases primitive-first:** a `LoadCase` should be a small authored grouping with id/name, description, action/category metadata, optional source/provenance, and stage/situation tags.
- **Combinations as downstream design/check inputs:** a `LoadCombination` can be represented as `{load_case_id: factor}` plus metadata: name, limit state (`strength`/`serviceability` or ULS/SLS), design method/code preset, jurisdiction, edition, and whether it is user-authored or generated.
- **Do not hardcode code rules as universal:** provide generic manual combinations first; later add scoped generators/templates such as “ASCE 7-22 / US / building / LRFD” or “EN 1990 + National Annex / building / ULS STR/GEO.”
- **Preserve result provenance:** analysis results should record the governing combination, contributing load cases, factors, source preset/user input, and run id.
- **Support envelopes separately:** practical workflows often need max/min envelopes over many combinations/directions/patterns; represent envelopes as derived result views, not as load cases.
- **Account for nonlinear analysis caution:** for linear analysis, superposition of case results may be valid; for geometric/material/contact/nonlinear effects, combinations may need to be solved as combined loads rather than algebraically combining independent case results.

## Cautions for wiki wording

- Say “loads/actions” when discussing Eurocode concepts; clarify that Eurocode “actions” include direct loads and indirect imposed effects.
- Do not publish proprietary ASCE/Eurocode tables as Fraia documentation unless licensing permits; link to official standards and use public summaries for conceptual orientation.
- Mark code-specific examples with code edition and source, e.g. “ASCE 7-16 educational example,” “ASCE 7-22 public overview,” or “EN 1990 concept with national annex caveat.”
- Avoid implying wind and seismic, snow and live, or accidental actions are combined the same way everywhere; simultaneity rules and exclusions are code-specific.
- Distinguish authored load cases from generated combinations and from immutable run artifacts.

## Suggested Fraia wiki cross-links

- `docs/structural-app-object-model.md` — `LoadAssignment` as canonical authored structural object.
- `docs/engineering-core.md` — primitive-first structural substrate.
- `docs/resolution-and-runs.md` — resolved state and immutable run artifacts for combinations/results.
- `docs/engineering-output-pipeline.md` — downstream design/check/export views and governing-result provenance.
- Potential wiki pages: “Loads and actions,” “Load cases,” “Load combinations,” “Limit states,” “Design situations,” “Analysis result envelopes.”

## Sources

### Kept

- SAF Documentation, “StructuralLoadCase” (https://www.saf.guide/en/stable/loads/structuralloadcase.html) — clear public schema-like definition of load cases, action type, load subtype, and duration. Retrieved 2026-05-06.
- PyNite Wiki, “5. Load Cases & Load Combinations” (https://github.com/JWock82/Pynite/wiki/5.-Load-Cases-&-Load-Combinations) — concise open-source software representation of combinations as factor maps over cases. Retrieved 2026-05-06.
- anaStruct documentation, “Load cases and load combinations” (https://anastruct.readthedocs.io/en/latest/loadcases.html) — practical open-source example preserving load-case and combination result objects. Retrieved 2026-05-06.
- ASCE, “ASCE 7-22” (https://www.asce.org/publications-and-news/codes-and-standards/asce-sei-7-22) — official public scope statement for ASCE 7-22 hazards and combination loads. Retrieved 2026-05-06.
- European Commission/JRC Eurocodes, “Eurocode: Basis of structural design” (https://eurocodes.jrc.ec.europa.eu/EN-Eurocodes/eurocode-basis-structural-design) — official public scope statement for EN 1990. Retrieved 2026-05-06.
- Engineering LibreTexts, “2.2: Load Combinations for Structural Design” (https://eng.libretexts.org/Bookshelves/Mechanical_Engineering/Introduction_to_Aerospace_Structures_and_Materials_(Alderliesten)/01:_Introduction_to_Structural_Analysis_and_Structural_Loads/02:_Structural_Loads_and_Loading_System/2.02:_Load_Combinations_for_Structural_Design) — open educational explanation of strength vs serviceability and ASCE 7-16 example combinations. Retrieved 2026-05-06.
- First Principle Engineering Knowledge, “Basis of Actions and Load Combinations (EN 1990)” (https://knowledge.fppengineering.com/basis-of-actions-and-load-combinations-en-1990/) — detailed public conceptual explanation of EN 1990 actions, ψ factors, limit states, and design situations. Retrieved 2026-05-06.

### Dropped / not relied on

- Generic SEO articles from commercial structural-analysis vendors — useful for examples but less authoritative than official pages and open-source docs.
- ASCE standard purchase/library pages — official but full technical content is not openly accessible; used ASCE public overview instead.
- Random PDF slide decks about ASCE updates — potentially useful but not canonical and may be event-specific.
- EurocodePy documentation — relevant software angle, but not needed after SAF/PyNite/anaStruct and official Eurocode sources.

## Gaps / next steps

- This brief does not determine Fraia’s exact data schema, UI naming, or code-generator algorithms.
- No licensed code text was reviewed; any future ASCE/Eurocode preset must be validated against licensed standards and applicable national/local amendments.
- Future research should separately cover: load paths, load categories/actions, pattern loading, construction-stage loading, envelopes, and governing design checks.
