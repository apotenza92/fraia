---
title: Steel Compression Members
status: compiled
trust_level: compiled
domain: materials
applies_to:
  - steel columns, braces, and compressed members
  - portal-frame preliminary member explanations
  - Fraia agent guidance
not_applicable_to:
  - final steel code checks
  - built-up member connector design
  - cold-formed steel design
jurisdiction_or_standard_context: concept guidance from professional steel sources; not a code check
last_compiled: 2026-05-07
source_count: 3
citation_policy: required
owner: agent-maintained
---

# Steel Compression Members

## Summary

Steel compression members are members that carry compression design actions, such as columns, braces, struts, posts, and compression zones of frame members. Their behavior is often governed by stability: flexural buckling, torsional buckling, flexural-torsional buckling, local/slender element effects, and second-order frame behavior can control before direct material strength alone.

For Fraia, compression-member design needs design actions plus explicit check inputs. A run axial-force value is not enough.

## Scope / non-scope

This page covers concept-level steel compression member behavior for Fraia agents.

It does not provide code equations, final capacity checks, built-up member connector design, cold-formed steel design, fire design, or project approval guidance.

## Key concepts

### Compression members are behavior roles, not just labels

An authored `Member` with role `column`, `brace`, or `tie` may become a compression member for a particular load case or combination if its design action includes compression. Conversely, a role-labelled brace may see tension in one case and compression in another.

Fraia should classify compression behavior from design actions and load combinations, not role name alone.

### Buckling often governs compression behavior

Professional steel guidance treats buckling resistance as central for members in axial compression. SteelConstruction.info notes that for uniform members in axial compression, design buckling resistance commonly governs. [S2]

Fraia should therefore avoid axial-stress-only explanations for slender steel compression members.

### Multiple global buckling modes can apply

AISC compression provisions and professional guidance consider flexural buckling, torsional buckling, and flexural-torsional buckling as applicable compression member limit states. [S1][S2][S3]

Fraia agents should preserve section family, symmetry, local axes, effective lengths, and restraint assumptions before choosing a mode or check path.

### Local/slender element effects are separate

Compression member strength can also be affected by slender compression elements in the cross-section. AISC Engineering Journal discussion of AISC 360-16 notes flexural, torsional, and flexural-torsional buckling together with slender element provisions. [S3]

Fraia should separate global member buckling from local section slenderness/element effects in check inputs and explanations.

### Frame columns often have combined actions

Portal-frame columns and frame members often carry axial compression plus bending, shear, and second-order effects. Treating them as purely axial struts is only valid when the model and connection/release assumptions support that simplification. [S2]

Fraia should carry compression member design actions as a set, not as a single scalar compression force.

## Engineering guidance for Fraia agents

- Determine compression-member behavior from load case/combination design actions, not member role alone.
- Preserve material grade, section family/designation, local axes, axial force sign, bending moments, effective lengths, restraint, connection fixity, and provenance.
- Do not infer effective length from analysis element length.
- Distinguish flexural, torsional, flexural-torsional, local/slender element, and combined-action questions.
- Review second-order effects and frame sway before trusting compression member check inputs.
- Mark compression checks incomplete when section, restraint, effective length, or connection assumptions are missing.
- Keep run results, design actions, check inputs, and check results as separate artifacts.

## Tradeoffs / cautions

- Axial-only compression assumptions can be useful for truss braces and simple columns, but can be wrong for frames, portal columns, eccentric loads, and partially restrained connections.
- More restraint can increase compression capacity, but it creates demands in the restraining system and connections.
- Conservative effective-length assumptions can drive heavier members; optimistic restraint assumptions can be unsafe.
- Slender section elements can reduce capacity even when global member buckling looks acceptable.
- Code-specific interaction equations are intentionally out of scope here.

## Source-backed claims

- Steel compression member checks consider flexural, torsional, and flexural-torsional buckling as applicable. [S1][S2][S3]
- Uniform steel members in axial compression are commonly governed by buckling resistance rather than axial strength alone. [S2]
- Slender compression elements/local buckling can affect compression member strength. [S1][S3]
- Combined bending and axial compression require separate treatment from pure axial compression. [S2]
- Compression checks require effective length/restraint and section/material context, not just force result data. [S1][S2]

## Open questions / weak evidence

- Fraia still needs final steel compression check-input schema, including effective length axes, section slenderness, built-up member metadata, and combined-action locations.
- Jurisdiction-specific equations and limits are deferred to steel check modules.
- Cold-formed and stainless steel compression members need separate future pages if supported.

## Related pages

- [Steel material properties and section families](material-properties-and-section-families.md)
- [Steel member behavior](member-behavior.md)
- [Compression member buckling concepts](../../stability/compression-member-buckling-concepts.md)
- [Member restraint and unbraced length](../../stability/member-restraint-and-unbraced-length.md)
- [Second-order effects and stability](../../analysis/second-order-effects-and-stability.md)
- [Truss analysis and two-force members](../../analysis/truss-analysis-and-two-force-members.md)

## Sources

- [S1] American Institute of Steel Construction, *Specification for Structural Steel Buildings (ANSI/AISC 360-16)*. URL: https://www.aisc.org/globalassets/aisc/publications/standards/a360-16-spec-and-commentary.pdf. Source type: professional standard/specification. Retrieved: 2026-05-07. Reliability/limits: authoritative steel compression terminology; US code context and formulas are not reproduced here.
- [S2] SteelConstruction.info / Steel Construction Institute and British Constructional Steelwork Association, *Member design*. URL: https://www.steelconstruction.info/Member_design. Source type: professional steel construction guidance. Retrieved: 2026-05-07. Reliability/limits: strong practical steel member guidance; UK/Eurocode context and not Fraia schema guidance.
- [S3] L. F. Geschwindner and Matthew Troemner, *Notes on the AISC 360-16 Provisions for Slender Compression Elements in Compression Members*. URL: https://ej.aisc.org/index.php/engj/article/download/1102/1101. Source type: AISC Engineering Journal paper. Retrieved: 2026-05-07. Reliability/limits: useful professional discussion of compression member limit states and slender elements; not a generic Fraia procedure.
