# Knowledge Cards Index

> Generated from typed records. Do not hand-edit this file; run `python3 scripts/generate-knowledge-next-views.py`.

## KC-authored-member-analysis-element-separation - Authored Member And Analysis Element Separation

- Status: `draft`
- Domain: `fraia-product-architecture`
- Concepts: authored member, analysis element, realization, finite element, result provenance
- Summary: Fraia should distinguish user-authored structural members from solver-level analysis elements so results and explanations preserve provenance.
- Sources: SRC-8419c2b9c6, SRC-6e33dce5df, SRC-9be980a1af, SRC-4a41e74bd1
- Media: KA-local-global-member-axes
- Relationships: supports KC-member-end-releases, supports KC-steel-design-action-check-input-separation

## KC-determinacy-restraint-mechanisms - Determinacy, Restraint, and Mechanisms

- Status: `draft`
- Domain: `structural-analysis`
- Concepts: static determinacy, restraint sufficiency, mechanism, truss idealisation, compatibility
- Summary: Fraia should distinguish equilibrium solvability, compatibility, restraint sufficiency, and mechanism risk when diagnosing structural models.
- Sources: SRC-e0a97ba94d, SRC-d9c2d2e337, SRC-25c08b9625
- Relationships: requires KC-free-body-equilibrium, supports KC-instability-diagnostics, see_also KC-member-end-releases

## KC-free-body-equilibrium - Free-Body Diagrams and Equilibrium

- Status: `draft`
- Domain: `structural-analysis`
- Concepts: free-body diagram, static equilibrium, force balance, moment balance, body selection
- Summary: Free-body diagrams and equilibrium equations are Fraia's first sanity-check language for explaining reactions, load paths, internal actions, and diagnostics.
- Sources: SRC-764acec0a5, SRC-f9a8b7e202, SRC-e0a97ba94d
- Media: KA-free-body-diagram-components
- Relationships: supports KC-support-reactions-idealisation, supports KC-reaction-sanity-checks, supports KC-determinacy-restraint-mechanisms

## KC-instability-diagnostics - Instability Diagnostics

- Status: `draft`
- Domain: `diagnostics`
- Concepts: instability, mechanism, singular stiffness, connectivity error, ill-conditioning, result trust
- Summary: Fraia should diagnose solver instability by mapping singularity, mechanism, connectivity, release, support, and stiffness-contrast evidence back to authored and resolved model objects before trusting results.
- Sources: SRC-160832a054, SRC-2dadd09595, SRC-46f4a11354, SRC-eadbb18f41, SRC-2c4eebe57e, SRC-df393de74d
- Relationships: requires KC-determinacy-restraint-mechanisms, see_also KC-member-end-releases, supports KC-reaction-sanity-checks

## KC-lateral-torsional-buckling-concepts - Lateral-Torsional Buckling Concepts

- Status: `draft`
- Domain: `steel-stability`
- Concepts: lateral-torsional buckling, compression flange restraint, elastic critical moment, restraint stiffness, unrestrained beam
- Summary: Lateral-torsional buckling is a restraint-sensitive steel beam stability mode that must remain separate from simple cross-section bending capacity.
- Sources: SRC-de331c3f98, SRC-346ae46879
- Media: KA-member-restraint-unbraced-length
- Relationships: requires KC-member-restraint-and-unbraced-length, see_also KC-steel-bending-members

## KC-load-application-equivalent-loads - Load Application and Equivalent Loads

- Status: `draft`
- Domain: `loads`
- Concepts: distributed load, equivalent resultant, equivalent nodal load, load realization, fixed-end action
- Summary: Fraia should keep authored load assignments distinct from the solver-ready loads and equivalent actions produced during realization.
- Sources: SRC-85e8bab79d, SRC-96f3780e82, SRC-4a41e74bd1, SRC-8419c2b9c6
- Media: KA-distributed-load-equivalent-resultant, KA-local-global-member-axes
- Relationships: requires KC-free-body-equilibrium, supports KC-reaction-sanity-checks

## KC-load-paths - Load Paths

- Status: `draft`
- Domain: `structural-analysis`
- Concepts: load path, continuous load path, gravity load transfer, lateral load transfer, connection chain
- Summary: Fraia should reason about how gravity, uplift, lateral, and stability actions travel through connected structural objects before trusting local member or reaction results.
- Sources: SRC-d83160defb, SRC-afd8c4e149, SRC-833946e3b6
- Media: KA-portal-frame-load-path
- Relationships: supports KC-reaction-sanity-checks, supports KC-instability-diagnostics

## KC-member-end-releases - Member End Releases

- Status: `draft`
- Domain: `modeling`
- Concepts: member end release, connection fixity, local axis, partial restraint, overrelease
- Summary: Fraia should model member end releases as explicit force/moment transfer assumptions tied to member end, local axis, component, and provenance.
- Sources: SRC-9be980a1af, SRC-a4257cc619, SRC-6e33dce5df
- Media: KA-member-release-components, KA-local-global-member-axes
- Relationships: supports KC-determinacy-restraint-mechanisms, supports KC-instability-diagnostics

## KC-member-restraint-and-unbraced-length - Member Restraint And Unbraced Length

- Status: `draft`
- Domain: `steel-stability`
- Concepts: member restraint, unbraced length, buckling length, torsional restraint, lateral restraint
- Summary: Member stability reasoning should preserve what restrains a member, which direction it restrains, and the unbraced length implied by those restraints.
- Sources: SRC-de331c3f98, SRC-37f553a164
- Media: KA-member-restraint-unbraced-length
- Relationships: supports KC-lateral-torsional-buckling-concepts, supports KC-steel-portal-purlins-and-girts

## KC-portal-frame-base-fixity-tradeoffs - Portal Frame Base Fixity Tradeoffs

- Status: `draft`
- Domain: `steel-systems`
- Concepts: portal frame, base fixity, pinned base, rigid base, support idealisation
- Summary: Portal-frame base fixity should be treated as an explicit support idealisation with structural and foundation tradeoffs, not as an invisible default.
- Sources: SRC-37f553a164, SRC-9be980a1af
- Media: KA-support-dof-reaction-symbols
- Relationships: requires KC-support-reactions-idealisation, see_also KC-steel-portal-frame-system-overview

## KC-portal-frame-longitudinal-transverse-stability - Portal Frame Longitudinal And Transverse Stability

- Status: `draft`
- Domain: `steel-systems`
- Concepts: portal frame stability, transverse stability, longitudinal stability, vertical bracing, plan bracing
- Summary: Portal-frame reasoning should distinguish transverse frame action from longitudinal building stability and explicitly connect roof and wall bracing load paths.
- Sources: SRC-37f553a164
- Media: KA-portal-frame-bracing-stability-axes, KA-portal-frame-load-path
- Relationships: requires KC-steel-bracing-principles, requires KC-load-paths, see_also KC-instability-diagnostics

## KC-reaction-sanity-checks - Reaction Sanity Checks

- Status: `draft`
- Domain: `diagnostics`
- Concepts: reaction sanity check, support reaction, load balance, sign convention, resolved load
- Summary: Fraia reaction sanity checks should compare solved reactions to the selected body, support DOFs, sign convention, and resolved run loads before downstream checks trust the result.
- Sources: SRC-764acec0a5, SRC-f9a8b7e202, SRC-e0a97ba94d, SRC-1fa009051f, SRC-85e8bab79d, SRC-96f3780e82
- Relationships: requires KC-free-body-equilibrium, requires KC-support-reactions-idealisation, requires KC-load-application-equivalent-loads

## KC-steel-bending-members - Steel Bending Members

- Status: `draft`
- Domain: `steel-design`
- Concepts: steel beam, bending member, bending action, shear action, member resistance
- Summary: Steel bending-member cards should explain which actions and assumptions matter, while leaving final resistance evaluation to scoped checks.
- Sources: SRC-1775f1230f, SRC-a80147d170
- Relationships: requires KC-steel-material-and-section-families, see_also KC-lateral-torsional-buckling-concepts, requires KC-steel-design-action-check-input-separation

## KC-steel-bracing-principles - Steel Bracing Principles

- Status: `draft`
- Domain: `steel-stability`
- Concepts: bracing, global stability, member restraint, load distribution, dimensional control
- Summary: Bracing knowledge should describe the job a brace is doing before it is treated as a restraint, load path, or global stability component.
- Sources: SRC-504746b8d3, SRC-37f553a164, SRC-a17357d4ab
- Media: KA-portal-frame-bracing-stability-axes
- Relationships: supports KC-load-paths, supports KC-portal-frame-longitudinal-transverse-stability

## KC-steel-compression-members - Steel Compression Members

- Status: `draft`
- Domain: `steel-design`
- Concepts: compression member, steel column, buckling length, slenderness, local buckling
- Summary: Compression-member reasoning should preserve restraint, length, axis, and local slenderness assumptions before any design check is run.
- Sources: SRC-de331c3f98, SRC-c4a211ba11
- Relationships: requires KC-member-restraint-and-unbraced-length, requires KC-steel-design-action-check-input-separation

## KC-steel-design-action-check-input-separation - Steel Design Action And Check Input Separation

- Status: `draft`
- Domain: `fraia-product-architecture`
- Concepts: design action, check input, check result, provenance, artifact separation
- Summary: Fraia should preserve design actions, check inputs, and check results as separate artifacts so steel knowledge retrieval supports checks without masquerading as them.
- Sources: SRC-9be980a1af, SRC-aeb2330d27, SRC-1775f1230f
- Media: KA-design-action-check-provenance-flow
- Relationships: supports KC-steel-bending-members, supports KC-steel-compression-members, supports KC-reaction-sanity-checks

## KC-steel-material-and-section-families - Steel Material And Section Families

- Status: `draft`
- Domain: `steel-design`
- Concepts: structural steel, material properties, section family, section properties, product standard
- Summary: Steel material and section-family knowledge should feed modeling and check inputs without pretending to be a final specification or code design.
- Sources: SRC-6d2d387325, SRC-2836969f03, SRC-aeb2330d27
- Relationships: supports KC-steel-bending-members, supports KC-steel-compression-members

## KC-steel-portal-frame-system-overview - Steel Portal Frame System Overview

- Status: `draft`
- Domain: `steel-systems`
- Concepts: portal frame, rafter, column, moment-resisting connection, haunch
- Summary: Portal-frame cards should describe the primary moment-resisting frame, secondary members, and stability assumptions as a system before member checks are generated.
- Sources: SRC-37f553a164, SRC-65fc8dcb5b, SRC-b31f9afb1f
- Media: KA-portal-frame-load-path
- Relationships: requires KC-load-paths, supports KC-portal-frame-base-fixity-tradeoffs, supports KC-portal-frame-longitudinal-transverse-stability

## KC-steel-portal-purlins-and-girts - Steel Portal Purlins And Girts

- Status: `draft`
- Domain: `steel-systems`
- Concepts: purlin, girt, side rail, rafter restraint, secondary member
- Summary: Purlins and girts should be modeled as secondary members whose load-transfer and restraint roles are explicit, not assumed from their names alone.
- Sources: SRC-37f553a164
- Relationships: requires KC-member-restraint-and-unbraced-length, supports KC-portal-frame-longitudinal-transverse-stability

## KC-support-reactions-idealisation - Support Reactions and Idealisation

- Status: `draft`
- Domain: `structural-analysis`
- Concepts: support reaction, support idealisation, degree of freedom, restraint, reaction sign
- Summary: Fraia should represent named supports as explicit restrained or prescribed DOFs so reactions, signs, and physical assumptions remain inspectable.
- Sources: SRC-e0a97ba94d, SRC-1fa009051f, SRC-f9a8b7e202
- Media: KA-support-dof-reaction-symbols
- Relationships: requires KC-free-body-equilibrium, supports KC-reaction-sanity-checks
