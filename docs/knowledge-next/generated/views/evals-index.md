# Retrieval Eval Seeds

> Generated from typed records. Do not hand-edit this file; run `python3 scripts/generate-knowledge-next-views.py`.

## KE-authored-member-vs-analysis-element - Authored Member Vs Analysis Element

- Prompt: A user sees one rafter in the UI but three beam-column elements in the solver artifact. Explain what knowledge the agent should retrieve.
- Expected cards: KC-authored-member-analysis-element-separation, KC-member-end-releases, KC-load-application-equivalent-loads, KC-steel-design-action-check-input-separation
- Expected concepts: authored member, analysis element, realization, member end release, check result
- Unacceptable patterns:
  - call every split analysis element a separate authored beam
  - hide the realization mapping
  - lose release or load provenance across element splits
  - treat element force output as a final design result

## KE-missing-context-before-scheme-generation - Missing Context Before Scheme Generation

- Prompt: The user asks for a portal-frame scheme but only gives building width and height. What should the agent retrieve and ask about before generating members?
- Expected cards: KC-load-paths, KC-support-reactions-idealisation, KC-steel-portal-frame-system-overview, KC-portal-frame-base-fixity-tradeoffs, KC-steel-design-action-check-input-separation
- Expected concepts: load path, support idealisation, portal frame, base fixity, check input
- Unacceptable patterns:
  - generate final member sizes from width and height alone
  - choose a base fixity without exposing the assumption
  - skip load path and support questions
  - present generated scheme text as a completed design check

## KE-portal-frame-bracing-review - Portal-Frame Bracing Review

- Prompt: Review a single-storey steel portal-frame scheme where roof plan bracing is shown but wall bracing and purlin restraint roles are unclear.
- Expected cards: KC-steel-portal-frame-system-overview, KC-steel-bracing-principles, KC-portal-frame-longitudinal-transverse-stability, KC-steel-portal-purlins-and-girts, KC-load-paths
- Expected concepts: portal frame, bracing, longitudinal stability, transverse stability, purlin
- Unacceptable patterns:
  - assume transverse frame action provides longitudinal building stability
  - assume purlins are adequate restraints without checking their role
  - copy portal-frame design formulas into the answer
  - give final bracing member sizes

## KE-solver-result-vs-design-action-check - Solver Result Vs Design Action And Check Result

- Prompt: The solver reports axial force and major-axis moment for a steel column. What should the agent retrieve before saying whether the member passes?
- Expected cards: KC-steel-design-action-check-input-separation, KC-steel-bending-members, KC-steel-compression-members, KC-reaction-sanity-checks, KC-authored-member-analysis-element-separation
- Expected concepts: design action, check input, check result, result provenance
- Unacceptable patterns:
  - say the member passes or fails from raw forces alone
  - skip restraint, section, material, and standard context
  - merge analysis results with check results
  - invent a jurisdiction-specific code rule

## KE-suspicious-reactions - Suspicious Reactions

- Prompt: A frame model has vertical loads but one support reaction is near zero and another support reaction is unexpectedly large. What knowledge should the agent retrieve before diagnosing the model?
- Expected cards: KC-reaction-sanity-checks, KC-free-body-equilibrium, KC-support-reactions-idealisation, KC-load-paths, KC-instability-diagnostics
- Expected concepts: reaction sanity check, free-body diagram, support idealisation, load path, instability
- Unacceptable patterns:
  - declare the structure safe or unsafe without checking load path and support idealisation
  - treat solver reactions as design check results
  - ignore instability or missing restraint possibilities
  - invent a code-specific pass/fail rule
