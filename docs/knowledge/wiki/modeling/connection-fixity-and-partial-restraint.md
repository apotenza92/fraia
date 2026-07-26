---
title: Connection Fixity and Partial Restraint Modeling
status: compiled
trust_level: compiled
domain: modeling
applies_to:
  - steel frame and portal-frame modeling assumptions
  - member end release and joint stiffness decisions
  - Fraia agent guidance
not_applicable_to:
  - final steel connection design
  - code-specific connection classification thresholds
  - nonlinear moment-rotation curve generation
jurisdiction_or_standard_context: concept guidance from professional and academic sources; not a code check
last_compiled: 2026-05-07
source_count: 3
citation_policy: required
owner: agent-maintained
---

# Connection Fixity and Partial Restraint Modeling

## Summary

Connection fixity is a modeling assumption about how much force, moment, and rotation a connection transfers between connected structural objects. Real steel connections are rarely perfect mathematical pins or perfect rigid joints. Many are best treated as nominally pinned, rigid/continuous, or semi-rigid/partially restrained depending on stiffness, strength, ductility, and the analysis purpose.

For Fraia, connection fixity should be explicit provenance attached to authored members, nodes, releases, constraints, and resolved analysis topology. It should not be inferred from visual intersection alone.

## Scope / non-scope

This page covers concept-level connection fixity and partial restraint modeling for Fraia agents.

It does not provide final steel connection design, code thresholds, end-plate/bolt/weld calculations, nonlinear moment-rotation curve derivation, or capacity checks.

## Key concepts

### Fixity affects force distribution and deformation

Professional steel guidance states that joint behavior affects internal force and moment distribution and overall frame deformation. Modeling a joint as pinned, rigid, or semi-rigid changes how the frame attracts moments, shears, axial forces, reactions, and drift. [S1]

Fraia should therefore treat fixity as an analysis assumption with downstream consequences, not as a label.

### Nominally pinned does not mean physically moment-free

A nominally pinned connection is one that may be assumed not to transmit bending moments for the analysis/design purpose, but professional guidance acknowledges that simple joints can transfer some moment. [S1]

Fraia agents should avoid saying "this connection cannot transfer moment" unless the physical/detailing evidence and analysis assumption support that stronger claim.

### Rigid/continuous needs stiffness, not just strength

A rigid or continuous joint is idealized as stiff enough that its flexibility can be neglected in the frame analysis. It must have suitable stiffness as well as moment resistance. [S1]

For Fraia, a "fixed" display assumption should carry source or design intent, especially at portal-frame eaves/ridges, moment frames, and base connections.

### Partially restrained means stiffness matters

AISC describes partially restrained/flexible moment connections as connections that provide some moment restraint while also rotating. Their behavior can affect connected members and frame system design. [S2]

Fraia should represent partial restraint as explicit rotational/translational stiffness or a source-scoped classification, not as a hidden average between pinned and fixed.

### Semi-rigid frame behavior is system-level

Academic portal-frame research reports that semi-rigid connection behavior can affect internal force distribution, lateral displacement magnitude, and collapse mode. [S3]

This matters for Fraia portal-frame schemes: changing eaves, ridge, base, brace, purlin, or girt connection fixity can change the whole system response.

### Releases and partial restraint are related but not identical

A full member-end release removes a selected transfer component in the idealized model. Partial restraint retains finite stiffness. Both should be modeled through explicit `ReleaseAssignment` or connection-stiffness metadata with local-axis and provenance information.

Fraia should not silently convert partial restraint to a full pin or full fixity unless the approximation is recorded and acceptable for the current use.

## Engineering guidance for Fraia agents

- Do not infer connection fixity from geometry or member role alone.
- State whether a connection assumption is nominally pinned, rigid/continuous, semi-rigid/partially restrained, or unknown.
- Record the affected members/nodes, end, local axis, released component, rotational/translational stiffness if partial, and provenance.
- Keep connection modeling assumptions separate from final connection design/check results.
- For portal frames, explicitly record assumptions at bases, eaves, ridge, haunches, bracing interfaces, purlins, and girts.
- If partial restraint is used, carry it into analysis metadata and downstream design-action context.
- Treat unexpected drift, moment distribution, reactions, or instability as possible evidence of wrong fixity assumptions.

## Tradeoffs / cautions

- Pinned assumptions simplify analysis and connection detailing, but can increase deflection, sway, and demand on bracing or other moment-resisting paths.
- Rigid assumptions can reduce sway and redistribute moments, but require connection stiffness/strength and can increase foundation or column demands.
- Partial restraint can better represent real behavior, but needs stiffness data and can make analysis/design communication more complex.
- A connection that behaves acceptably under gravity loads may not provide intended lateral or stability restraint.
- Code-specific classification rules should not be embedded in generic Fraia wiki guidance.

## Source-backed claims

- Joint behavior affects frame force/moment distribution and deformation. [S1]
- Professional steel guidance distinguishes nominally pinned, rigid/continuous, and semi-continuous/semi-rigid joint behavior for analysis. [S1]
- Partially restrained/flexible moment connections provide some moment restraint while allowing rotation. [S2]
- Semi-rigid connection stiffness can influence internal force distribution, lateral displacement, and collapse mode in portal/frame systems. [S3]
- Rigid joint idealisation requires adequate stiffness, not just moment resistance. [S1]

## Open questions / weak evidence

- Fraia still needs final schema for connection fixity assumptions, moment-rotation curves, rotational springs, and partial stiffness provenance.
- Code-specific stiffness/strength/ductility classification thresholds are deferred to steel design/check pages.
- Detailed base-plate, eaves, ridge, purlin/girt, brace, and splice connection behavior need separate steel-system pages.

## Related pages

- [Member end releases](member-end-releases.md)
- [Supports, restraints, and releases](supports-restraints-and-releases.md)
- [Constraints, rigid links, and diaphragms](constraints-rigid-links-and-diaphragms.md)
- [Second-order effects and stability](../analysis/second-order-effects-and-stability.md)
- [Beam shear and moment diagrams](../analysis/beam-shear-and-moment-diagrams.md)
- [Steel portal-frame bracing](../steel/portal-frames/bracing.md)

## Sources

- [S1] SteelConstruction.info / Steel Construction Institute and British Constructional Steelwork Association, *Modelling and analysis*. URL: https://steelconstruction.info/Modelling_and_analysis. Source type: professional steel construction guidance. Retrieved: 2026-05-07. Reliability/limits: strong practical joint modeling guidance; UK/Eurocode context and not Fraia schema guidance.
- [S2] American Institute of Steel Construction, *Partially Restrained and Flexible Moment Connections*. URL: https://www.aisc.org/education/continuingeducation/education-archives/partially-restrained-and-flexible-moment-connections/. Source type: professional steel continuing-education source. Retrieved: 2026-05-07. Reliability/limits: reputable AISC overview of partially restrained connection behavior; course page-level source, not a complete design guide here.
- [S3] L. Simoes da Silva, A. Santiago, and P. Vila Real, *A parametric analysis of steel and composite portal frames with semi-rigid connections*. URL: https://www.sciencedirect.com/science/article/pii/S0141029605003342. Source type: peer-reviewed article abstract in Thin-Walled Structures. Retrieved: 2026-05-07. Reliability/limits: useful source for system-level semi-rigid portal-frame behavior; detailed modeling/calibration is beyond this baseline page.
