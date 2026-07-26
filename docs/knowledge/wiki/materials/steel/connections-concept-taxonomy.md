---
title: Steel Connections Concept Taxonomy
status: compiled
trust_level: compiled
domain: materials
applies_to:
  - steel connection vocabulary
  - portal-frame and braced-frame modeling assumptions
  - Fraia agent guidance
not_applicable_to:
  - final steel connection design
  - bolt or weld sizing
  - seismic detailing rules
jurisdiction_or_standard_context: concept guidance from professional steel sources; not a code check
last_compiled: 2026-05-07
source_count: 3
citation_policy: required
owner: agent-maintained
---

# Steel Connections Concept Taxonomy

## Summary

Steel connections should be described by what they are intended to transfer and how they affect stiffness/rotation, not just by what they look like. A useful Fraia taxonomy separates simple/shear connections, moment-resisting connections, bracing/gusset connections, splices, base connections, purlin/girt connections, and partial-restraint assumptions.

This page is vocabulary and modeling guidance. It is not connection design.

## Scope / non-scope

This page covers concept-level steel connection taxonomy for Fraia agents.

It does not provide bolt/weld design, connection capacity formulas, standard detail tables, seismic detailing, fabrication rules, or project approval guidance.

## Key concepts

### Simple connections are not just "small details"

Professional guidance describes simple connections as connections that transfer shear and tying forces while allowing rotation, supporting simple/braced frame assumptions. [S1][S3]

Fraia should link simple connection assumptions to member-end releases, bracing/lateral system assumptions, and downstream load path.

### Moment-resisting connections carry frame action

Moment-resisting connections are used in continuous frames, multi-storey unbraced frames, and single-storey portal frames. SteelConstruction.info lists common moment connection forms such as full-depth end plate, extended end plate, stiffened extended end plate, haunched, welded, splices, apex connections, and column bases. [S2]

Fraia should treat moment connection assumptions as system behavior assumptions, not decorative details.

### Bracing connections transfer axial stability/load-path forces

Bracing members often connect through gusset plates to beams, columns, or beam end connections. [S1]

Fraia agents should preserve whether a brace connection is intended as axial-only, eccentric, pinned, partially restrained, or part of a larger beam-column joint region.

### Base connections shape frame and foundation behavior

Column/base connections may be nominally pinned, moment-resisting, or partially restrained. Moment-resisting connection guidance treats column bases as connection types that can transmit moment and axial force to the substructure. [S2]

Fraia should not infer base fixity from a support icon or base plate existence alone.

### Splices and apex connections are force-transfer assumptions

Beam, column, and portal-frame apex splices may transfer axial force, shear, moment, or combinations depending on design intent and location. [S2]

Fraia should carry splice assumptions into analysis/release metadata and downstream check inputs.

### Taxonomy is not detailing

Connection taxonomy helps Fraia choose modeling assumptions and ask good questions. It does not size bolts, welds, plates, stiffeners, or anchors.

Fraia should keep connection intent, analysis fixity/release behavior, design actions, check inputs, and final check results separate.

## Engineering guidance for Fraia agents

- Classify steel connections by intended force transfer: shear, axial, moment, torsion, tie force, restraint, or combinations.
- Classify connection stiffness behavior separately: nominally pinned/simple, rigid/continuous, semi-rigid/partially restrained, or unknown.
- Do not infer connection taxonomy from geometry, member role, or section family alone.
- For portal frames, explicitly record base, eaves, ridge/apex, haunch, splice, bracing, purlin, and girt connection assumptions.
- Map connection taxonomy to `ReleaseAssignment`, partial stiffness, constraints, load path, design actions, and check inputs.
- Mark connection design as incomplete when force-transfer intent or fixity is unknown.
- Keep software/detail library names source-scoped and out of generic engineering claims.

## Tradeoffs / cautions

- Simple connections can make framing economical and clear, but require a separate stability/lateral system.
- Moment connections can provide frame action and reduce sway, but are generally more complex and can increase fabrication/foundation demands.
- Bracing connections may be nominally axial but still see eccentricity, shear, moment, or local connection-region effects.
- Base fixity assumptions can strongly change portal-frame forces, drift, and foundation reactions.
- Partial restraint can better represent real behavior, but requires stiffness/capacity evidence and clear communication.

## Source-backed claims

- Simple steel connections are intended to transfer shear and tying forces while permitting rotation. [S1][S3]
- Moment-resisting connections are used in continuous and portal-frame systems and transfer bending moments as part of frame action. [S2]
- Moment-resisting connection families include end plate, extended/stiffened end plate, haunched, welded, splice, apex, and column-base forms. [S2]
- Bracing members are commonly connected through gusset plates to beams/columns or associated connection regions. [S1]
- Column bases can be connection types that transfer moment and axial force between steel members and concrete substructures. [S2]

## Open questions / weak evidence

- Fraia still needs final connection-intent schema, force-transfer channels, stiffness classification, and relationship to detailed connection check modules.
- Purlin/girt connection taxonomy, HSS connections, base plates, seismic connections, and portal-frame haunch/apex detailing need future pages.
- Code-specific connection classifications and capacities are deferred to steel connection check modules.

## Related pages

- [Connection fixity and partial restraint modeling](../../modeling/connection-fixity-and-partial-restraint.md)
- [Member end releases](../../modeling/member-end-releases.md)
- [Steel material properties and section families](material-properties-and-section-families.md)
- [Steel compression members](compression-members.md)
- [Steel beams and bending members](beams-and-bending-members.md)
- [Steel portal-frame bracing](../../steel/portal-frames/bracing.md)

## Sources

- [S1] SteelConstruction.info / Steel Construction Institute and British Constructional Steelwork Association, *Simple connections*. URL: https://steelconstruction.info/Simple_connections. Source type: professional steel construction guidance. Retrieved: 2026-05-07. Reliability/limits: strong practical simple/bracing/base connection guidance; UK/Eurocode context and not Fraia schema guidance.
- [S2] SteelConstruction.info / Steel Construction Institute and British Constructional Steelwork Association, *Moment resisting connections*. URL: https://www.steelconstruction.info/Moment_resisting_connections. Source type: professional steel construction guidance. Retrieved: 2026-05-07. Reliability/limits: strong practical moment/splice/base connection taxonomy; UK/Eurocode context and not Fraia schema guidance.
- [S3] Institution of Structural Engineers, *Technical Guidance Note: Simple connections in steel frames*. URL: https://www.istructe.org/journal/volumes/volume-96-(2018)/issue-9/technical-guidance-note-level-2-no-17-steel-frames/. Source type: professional technical guidance note page. Retrieved: 2026-05-07. Reliability/limits: useful professional definition/orientation; page-level access and not full connection design guidance.
