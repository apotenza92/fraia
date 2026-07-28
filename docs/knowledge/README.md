# Fraia Knowledge Wiki

_Status: scaffold v0.2_
_Date: 2026-05-07_

This directory is Fraia's concrete LLM wiki knowledge layer.

It implements the knowledge-backend direction described in [`../knowledge-backend.md`](../knowledge-backend.md): durable, file-based, agent-maintained engineering knowledge that can be read by Fraia agents before they ask questions, generate schemes, explain tradeoffs, or diagnose model issues.

## Purpose

The wiki exists to compound Fraia's reusable engineering knowledge over time. It is not project-specific truth and it is not a substitute for deterministic validation, analysis, code checks, or user-approved scheme artifacts.

Agents should use this wiki to:

- ask better questions
- avoid repeated web research for recurring topics
- explain engineering tradeoffs with provenance
- avoid shallow hardcoded heuristics
- decide when not to generate a scheme
- record durable, cross-linked knowledge after research

## Karpathy-style layers

Fraia's wiki keeps the durable parts small and inspectable:

1. [`wiki/`](wiki/): compiled, cross-linked pages agents can read directly.
2. Per-page `## Sources` plus optional [`sources.md`](sources.md): durable provenance and bibliography.
3. [`proposals/`](proposals/): draft inbox for discovered knowledge gaps.
4. [`adapter-contract.md`](adapter-contract.md) and [`contributing.md`](contributing.md): boundary and contribution rules for maintainer/community/third-party knowledge updates.
5. [`schema.md`](schema.md), [`workflow.md`](workflow.md), [`ingestion.md`](ingestion.md), [`topic-map.md`](topic-map.md), and [`index.md`](index.md): rules, optional adapter/maintainer ingestion guidance, navigation, roadmap, and registry metadata.

Temporary source extraction belongs in `/tmp/fraia-knowledge/` by default, or in gitignored `docs/knowledge/.staging/` for multi-step local workflows. [`raw/`](raw/) is legacy/exceptional and should contain only compact agent-authored source notes, not copied source content.

Fraia deliberately adapts the Karpathy raw → compiled wiki pattern: compiled pages and source lists are durable, but raw extraction is normally temporary for copyright, privacy, and engineering-audit reasons.

## Agent-maintained, human-readable

Default maintenance is agentic. Humans are not expected to hand-maintain pages, but all changes must remain inspectable as file diffs and entries in [`wiki/log.md`](wiki/log.md).

Agents may create/update proposals automatically. Ordinary project/scheme agents must not silently mutate compiled pages or create permanent raw extraction dumps. Pages should only be promoted to `compiled` inside an explicit wiki-maintenance run after schema, citation, link, topic-map, index, deterministic lint, reviewer, and Fraia Knowledge Steward checks pass.

The shipped Fraia app should consult the compiled wiki; heavy PDF/OCR/web/image ingestion belongs to maintainer-side or third-party adapter workflows that produce source learning packets or proposed wiki updates.

## Browser viewer

A self-contained local viewer can be generated and opened in a browser:

```sh
python3 scripts/build-knowledge-viewer.py
open docs/knowledge/viewer.html
```

The viewer is a convenience artifact. Markdown files remain the source of truth.

## Trust boundary

The wiki is general/domain knowledge. Project-specific engineering decisions belong in project, scheme, model, and run artifacts with their own provenance.
