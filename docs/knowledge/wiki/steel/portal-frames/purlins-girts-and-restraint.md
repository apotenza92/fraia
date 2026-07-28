---
title: Purlins and Girts as Restraint/Load-Transfer Members
status: compiled
trust_level: compiled
domain: structural-steel
applies_to:
  - steel portal-frame secondary steelwork
  - rafter and column restraint assumptions
  - Fraia agent guidance
not_applicable_to:
  - cold-formed purlin/girt design checks
  - cladding diaphragm design
  - manufacturer span-table validation
jurisdiction_or_standard_context: concept guidance from professional steel sources; not a code check
last_compiled: 2026-05-07
source_count: 3
citation_policy: required
owner: agent-maintained
---

# Purlins and Girts as Restraint/Load-Transfer Members

## Summary

Purlins and girts/side rails are secondary steel members, but they are not decorative. They support roof/wall cladding, transfer loads to primary portal frames, and may provide restraint to rafters and columns when the connection, continuity, cladding, stiffness, and bracing load path justify that assumption.

For Fraia, purlins and girts should be authored `Member` objects when they carry load or restraint intent. Their role as load-transfer members should be separated from their possible role as lateral/torsional restraints.

## Scope / non-scope

This page covers concept-level purlin/girt load-transfer and restraint guidance for Fraia agents.

It does not provide cold-formed member design, manufacturer span-table validation, cladding diaphragm design, restraint-force design, or code-specific checks.

## Key concepts

### Purlins and girts support cladding loads

Professional building-envelope guidance describes roof purlins and wall side rails as secondary members that support cladding panels or sheets and transfer cladding self-weight, wind, roof imposed loads, and maintenance loads to the primary steel frame. [S1]

Fraia should represent these load paths explicitly when purlins/girts are in scope.

### Secondary steelwork can restrain primary steelwork

Portal-frame and building-envelope guidance notes that purlins and side rails may restrain rafters and columns and may transfer horizontal loads into bracing systems. [S1][S2][S3]

Fraia agents should say "candidate restraint" until the restraint path is justified.

### Restraint depends on continuity and load path

Restraint requires more than proximity. It depends on member continuity, connection behavior, cladding restraint, bracing stiffness, and a path for restraint forces. [S1][S2]

Fraia should not infer effective restraint from a purlin/girt line crossing a frame member.

### Compression-side changes can invalidate assumptions

Under gravity load, uplift, reversal, or near haunches, the compression side of a rafter or column can change. Professional guidance notes that when the inner flange is in compression, ordinary purlin or side-rail attachment to the outer flange may not directly restrain it. [S1][S2]

Fraia should track load case, compression side, and local axis for restraint assumptions.

### Openings can break restraint assumptions

Side rails interrupted by openings such as doors cannot automatically be relied on as continuous restraints. [S2]

Fraia should keep openings/discontinuities visible in restraint provenance when the building envelope is modeled.

## Engineering guidance for Fraia agents

- Model purlins and girts as authored `Member` objects when they support loads, restrain primary members, or transfer bracing forces.
- Separate load support, lateral restraint, torsional restraint, and longitudinal load transfer in explanations.
- Identify the restrained primary member, flange/side, local axis, load case, purlin/girt continuity, connection, cladding role, and bracing path.
- Do not treat purlin spacing as unbraced length unless the restraint is justified for the relevant mode.
- Flag doors, openings, discontinuities, lap/continuity assumptions, and missing bracing paths.
- Keep manufacturer table assumptions source-scoped and out of generic Fraia claims.
- Preserve purlin/girt assumptions as check inputs for rafters, columns, cladding support, and bracing systems.

## Tradeoffs / cautions

- Using purlins/girts as restraints can improve primary member efficiency, but creates restraint forces and connection/load-path demands.
- Ignoring valid restraint can be conservative but may distort preliminary steel quantities.
- Assuming restraint without continuity, cladding, or bracing evidence can be unsafe.
- Uplift can put the opposite flange in compression and change the effective restraint system.
- Cold-formed purlin/girt behavior and connection details are specialized and need dedicated checks.

## Source-backed claims

- Purlins and side rails support cladding and transfer loads to the primary steel frame. [S1][S3]
- Purlins and side rails may provide restraint to rafters/columns and transfer horizontal loads into bracing systems. [S1][S2]
- Secondary steelwork plays an important role in restraining primary steelwork in portal-frame buildings. [S2][S3]
- Restraint assumptions depend on cladding/sheathing restraint, bracing stiffness/path, and member/flange behavior. [S1][S2]
- Openings or interrupted side rails can invalidate restraint assumptions. [S2]

## Open questions / weak evidence

- Fraia still needs final purlin/girt object schema, continuity modeling, cladding/envelope assumptions, and restraint-force check inputs.
- Cold-formed purlin/girt design, manufacturer table use, stressed-skin/cladding diaphragm action, and connection design are future pages/check modules.
- Load reversal/uplift and haunch-region restraint need more detailed portal-frame pages.

## Related pages

- [Steel portal-frame system overview](system-overview.md)
- [Steel portal-frame bracing](bracing.md)
- [Member restraint and unbraced length](../../stability/member-restraint-and-unbraced-length.md)
- [Lateral-torsional buckling concepts](../../stability/lateral-torsional-buckling-concepts.md)
- [Steel beams and bending members](../../materials/steel/beams-and-bending-members.md)
- [Constraints, rigid links, and diaphragms](../../modeling/constraints-rigid-links-and-diaphragms.md)

## Sources

- [S1] SteelConstruction.info / Steel Construction Institute and British Constructional Steelwork Association, *Building envelopes*. URL: https://steelconstruction.info/Building_envelopes. Source type: professional steel construction guidance. Retrieved: 2026-05-07. Reliability/limits: strong purlin/side-rail and envelope load-path guidance; UK/Eurocode context and not Fraia schema guidance.
- [S2] SteelConstruction.info / Steel Construction Institute and British Constructional Steelwork Association, *Portal frames*. URL: https://steelconstruction.info/Portal_frames. Source type: professional steel construction guidance. Retrieved: 2026-05-07. Reliability/limits: strong portal-frame restraint guidance; UK/Eurocode context and not a purlin design module.
- [S3] SteelConstruction.info / Steel Construction Institute and British Constructional Steelwork Association, *Engineering students' guide to single storey buildings*. URL: https://steelconstruction.info/Engineering_students%27_guide_to_single_storey_buildings. Source type: professional/open educational steel construction guidance. Retrieved: 2026-05-07. Reliability/limits: useful single-storey system overview; educational UK context.
