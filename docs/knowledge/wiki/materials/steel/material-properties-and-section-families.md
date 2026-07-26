---
title: Steel Material Properties and Section Families
status: compiled
trust_level: compiled
domain: materials
applies_to:
  - steel material and section vocabulary
  - preliminary steel scheme explanations
  - Fraia agent guidance
not_applicable_to:
  - final steel design checks
  - procurement or availability guarantees
  - copied section-property tables
jurisdiction_or_standard_context: concept guidance from professional steel sources; section names and properties are source/version scoped
last_compiled: 2026-05-07
source_count: 3
citation_policy: required
owner: agent-maintained
---

# Steel Material Properties and Section Families

## Summary

Steel material properties and section families give Fraia the vocabulary needed to talk about steel members, but they do not by themselves determine member role, structural adequacy, restraint, connection fixity, or final design capacity.

For Fraia, steel should be represented as separate but linked assumptions: material grade/properties, section family/designation, section-property source/version, authored `Member` role, resolved analysis properties, design actions, check inputs, and check results.

## Scope / non-scope

This page covers concept-level structural steel material and section-family vocabulary for Fraia agents.

It does not provide section-property tables, capacity formulas, material procurement advice, code checks, or a final Fraia material/section schema.

## Key concepts

### Material properties are source-scoped

Professional steel guidance describes design-relevant material properties such as strength, toughness, ductility, weldability, and durability, with values derived from product standards and manufacturing route. [S1]

Fraia should therefore record material grade/source metadata instead of treating "steel" as a single universal material.

### Product form and section family are different ideas

Steel is supplied in product forms such as flat products and long products. Structural sections include open rolled shapes such as beams, columns, channels, angles, and tees, as well as hollow sections. [S1][S2]

Fraia should distinguish product/section vocabulary from authored member roles like `beam`, `column`, `rafter`, `brace`, `purlin`, `tie`, or `girt`.

### Section property data needs provenance

Recognized section tables and databases provide dimensions and properties for standard sections. SteelConstruction.info points to professional section tables for open and hollow sections, and AISC publishes a shapes database with dimensions and properties consistent with the Steel Construction Manual. [S2][S3]

Fraia should attach source, edition/version, region/catalog, and units to imported section properties.

### Shape family does not decide behavior alone

A wide-flange or universal beam may be used as a beam, column, rafter, or frame member. An HSS may be used as a column, brace, truss member, or beam. Angles, channels, tees, and plates may be primary or secondary depending on the authored system.

Fraia should keep section family separate from `Member.role` and from analysis/check behavior.

### Standard and custom sections need different handling

Catalog sections can reference a source property database. Built-up, welded, tapered, castellated/cellular, plated, composite, or custom sections need explicit geometry/property derivation and provenance rather than a guessed catalog designation. [S2]

Agents should surface uncertainty when a section designation cannot be resolved to an approved property source.

## Engineering guidance for Fraia agents

- Do not use "steel" without material grade/property provenance when design-relevant claims are made.
- Keep material properties, section family/designation, authored `Member` role, and check inputs distinct.
- Record section-property source, edition/version, units, and region/catalog.
- Do not copy section tables into the wiki; cite original professional sources and use maintained property libraries in implementation.
- Do not infer role or adequacy from shape family alone.
- Treat custom/built-up/tapered sections as explicit section definitions with their own provenance.
- When passing steel design actions downstream, include material, section, local axes, load combination, restraint/check inputs, and source/version metadata.

## Tradeoffs / cautions

- Standard sections make preliminary modeling faster, but catalog names vary by region and edition.
- Custom sections can represent real portal frames and fabricated members better, but require explicit geometry and property derivation.
- Section dimensions/properties are data, not design approval.
- Material strength is only one part of steel behavior; toughness, ductility, weldability, durability, stability, and connections can govern.
- A section library mismatch can silently corrupt analysis stiffness, mass, design actions, and check results.

## Source-backed claims

- Design-relevant steel material properties include strength, toughness, ductility, weldability, and durability. [S1]
- Structural steel properties depend on chemical composition, manufacturing, mechanical working, and heat treatment. [S1]
- Steel sections include open sections such as beams, columns, channels, and angles, and hollow sections. [S2]
- Professional section tables/databases provide dimensions and section properties for standard section families. [S2][S3]
- The AISC Shapes Database is a compilation of structural steel shape dimensions and properties consistent with the Steel Construction Manual. [S3]

## Open questions / weak evidence

- Fraia still needs final material and section schemas, source/version identifiers, and unit handling.
- Australian/AS/NZS, European, UK, and US section-library mapping need separate source registries.
- Built-up/tapered portal-frame sections and cold-formed purlins/girts need future pages.

## Related pages

- [Steel member behavior](member-behavior.md)
- [Member restraint and unbraced length](../../stability/member-restraint-and-unbraced-length.md)
- [Lateral-torsional buckling concepts](../../stability/lateral-torsional-buckling-concepts.md)
- [Compression member buckling concepts](../../stability/compression-member-buckling-concepts.md)
- [Beam shear and moment diagrams](../../analysis/beam-shear-and-moment-diagrams.md)
- [Local and global coordinate systems](../../modeling/local-and-global-coordinate-systems.md)

## Sources

- [S1] SteelConstruction.info / Steel Construction Institute and British Constructional Steelwork Association, *Steel material properties*. URL: https://www.steelconstruction.info/Steel_material_properties. Source type: professional steel construction guidance. Retrieved: 2026-05-07. Reliability/limits: strong steel material overview; UK/Eurocode product-standard context and not Fraia schema guidance.
- [S2] SteelConstruction.info / Steel Construction Institute and British Constructional Steelwork Association, *Steel section sizes*. URL: https://www.steelconstruction.info/Steel_section_sizes. Source type: professional steel construction guidance. Retrieved: 2026-05-07. Reliability/limits: useful section family/property-source overview; UK/European catalog context.
- [S3] American Institute of Steel Construction, *AISC Shapes Database v16.0*. URL: https://www.aisc.org/aisc/publications/steel-construction-manual/aisc-shapes-database-v160/. Source type: professional steel section-property database page. Retrieved: 2026-05-07. Reliability/limits: authoritative US shape database page; database contents are not copied into this wiki.
