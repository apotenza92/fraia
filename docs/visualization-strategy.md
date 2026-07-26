# Fraia Visualization Strategy

_Status: draft v0.1_
_Date: 2026-04-13_

This document captures the current direction for Fraia's visual modeling and viewing strategy.

---

## 1. Core question

Engineering is highly visual.

Users need to see what Fraia is referring to when it talks about:

- spans
- supports
- bays
- braces
- frames
- clear spans
- internal columns
- alternative systems

However, building a full CAD environment from day 1 would be a very large effort.

So Fraia needs a strategy that provides meaningful visual feedback early without requiring a full traditional CAD system immediately.

---

## 2. Recommended principle

### Fraia should have a real 3D world from day 1, but not a full CAD environment from day 1

This is the current preferred direction.

Meaning:

- use a proper 3D canvas/world model from the beginning
- allow users to orbit, pan, zoom, and inspect geometry
- render primitives/archetypes in a coherent coordinate system
- support picking/highlighting/selection early
- avoid trying to build a full freeform CAD authoring system immediately

---

## 3. Why this is the preferred compromise

### Why not stay text-only?
Because users need visual grounding.

### Why not build full CAD immediately?
Because that is too much scope too early.

### Therefore
Start with a **visualizer/inspector first**, then grow toward editing.

---

## 4. Suggested early visualization stages

## Stage 1 — 3D viewer / inspector
Support:

- 3D world with coordinates
- rendered points/lines/regions
- camera controls
- labels/highlights
- selection and inspection
- display of supports, loads, and candidate options

This is the most important first milestone.

## Stage 2 — constrained visual editing
Support:

- move handles
- adjust spans/heights/bays
- choose supports/columns from known primitives
- drag known diagram controls

This is not full CAD. It is parameter/primitive-driven editing in 3D.

## Stage 3 — richer model authoring
Support:

- more direct placement/composition of primitives
- snapping and ports
- system sketching
- richer geometry workflows

## Stage 4 — advanced CAD-like workflows later
Only much later if needed.

---

## 5. Fraia should render primitives, not just raw solver entities

The user should ideally see meaningful Fraia objects such as:

- portal frame
- support symbol
- brace
- clear span region
- load arrow
- internal column option

rather than only raw nodes and solver elements.

This reinforces the idea that primitives/archetypes should carry visual metadata.

---

## 6. Geometry associated with primitives

Yes: primitives and archetypes should carry graphics/geometry metadata.

A useful separation is:

## 6.1 Diagram geometry
Simple shapes/lines/handles for communication and editing.

## 6.2 Analytical geometry
Centerlines, midsurfaces, idealized analysis geometry.

## 6.3 Physical/render geometry
More realistic display geometry for richer visualization later.

Not every primitive needs all three immediately, but the distinction is important.

---

## 7. Why a proper 3D world still matters early

Even if Fraia starts with very simple visuals, using a proper 3D coordinate world early helps with:

- consistency with the canonical 3D model
- future support for orientation/local axes
- future surface/solid elements
- future support/load direction display
- easier transition to more capable visual tools later

So the renderer should not be fake or purely 2D-form-based if it can be avoided.

---

## 8. Viewport Drawing Primitive Rule

Current implementation direction:

- Draw viewport/model content through the renderer pipeline, not the DOM.
- Treat members, nodes, supports, loads, releases, labels, axes, origins, highlights, and other model-attached glyphs as renderer primitives.
- Use WebGL/Three.js primitives, screen-space line materials, points, billboards, and camera-facing sprites for viewport symbols and labels.
- Keep DOM usage for app chrome, panels, toolbars, menus, inspectors, and non-viewport UI.
- Avoid projecting DOM elements over the canvas for model-attached graphics. That creates timing, layout, and camera-update jitter because the DOM and WebGL scene are not rendered in one coherent pass.

Near-term exception:

- A temporary DOM overlay is acceptable only as a development bridge when the same item is planned to become a renderer primitive. It should not be the default design for production viewport symbols.

---

## 9. What the first Fraia GUI should probably be

Not a full generic CAD app.

More like a **structural engineering workbench**:

- planning panel
- option cards
- 3D viewer
- object inspector
- structural drawing/editing around nodes, members, plates, supports, and loads
- maybe simple handles for spans/heights/support positions

This would already be a strong and useful interface.

---

## 10. Relationship to the current demo

The current demo asks for span/height numerically too early because it lacks a visual middle layer.

A better Fraia GUI should let the user:

- see the candidate structure
- compare variants visually
- understand where columns/supports/braces are added
- inspect what Fraia means before committing further

This would improve trust and usability significantly.

---

## 11. Recommended stack direction

Current product direction:

- Electron remains the product workbench shell while it is materially ahead for app UI.
- React/shadcn app chrome, panels, menus, chat, tables, forms, inspectors, and workflow surfaces stay in Electron for now.
- The existing embedded Three.js viewport remains the production renderer for now.
- The immediate product work should improve structural editor behaviour inside the current viewport: selection, hover, inspection, constrained edit commands, snapping, handles, and clear visual feedback.
- Keep authored structural model state in Fraia/appd, not in the renderer.
- Keep model-attached viewport content in Three.js renderer primitives, not Electron DOM overlays.

Near-term implementation:

- Add renderer-native picking and highlight behaviour to the current Three.js viewport.
- Use Electron panels for object inspectors and edit controls because they are app chrome, not model-attached graphics.
- Keep model-attached selection highlights, handles, support glyphs, load glyphs, labels, and future edit handles inside the renderer scene.
- Add constrained structural commands first, such as selecting members, editing endpoints through known nodes, moving supports between nodes, and adjusting span/bay parameters.

Electron performance guardrails:

- Treat Electron as a measurable shell, not an excuse for high memory use.
- Run the production-built shell with app-owned metrics capture, not a browser preview, when checking memory and idle CPU.
- Keep the normal app to one primary renderer window, one primary renderer process, one WebGL canvas, one GPU process, and one Rust `fraia-appd` sidecar.
- Keep large model geometry in GPU buffers and Three.js objects, not React state or DOM nodes.
- Selection, panel resize, hover, and camera inset changes should update existing renderer state rather than rebuilding the whole Three.js scene.
- Use render-on-demand for the idle viewport. Resize, camera movement, theme changes, scene changes, selection, and explicit invalidation can render; idle polling loops and continuous animation loops should not be present.
- Track viewport draw calls, object counts, memory, DOM count, canvas count, JS heap, and hit-test timings.
- Batch member and node primitives before optimising secondary symbols. Members should be submitted as base, proposed, and selected/highlight overlay batches, with selected/highlight changes updating a small overlay buffer rather than rebuilding the base scene.
- Disable labels automatically at and above `10,000` members unless diagnostics explicitly force them on.
- The first serious large-model gate is `50,000` members with labels off. Standard laptops should stay interactive at `<= 33 ms/frame`, with draw calls in the tens rather than thousands.
- Budget tiers live in `apps/fraia-electron/scripts/perf-budgets.cjs` and are selected from hardware facts, with `FRAIA_PERF_TIER=compact_laptop|standard_laptop|workstation` available for repeatable comparisons.
- Local commands are documented in `apps/fraia-electron/README.md`: `npm run smoke:perf`, `npm run benchmark:viewport`, and `npm run benchmark:perf-gate`.

## 12. Design choices currently favored

- Fraia should show a proper 3D world from early on.
- Fraia should not attempt to build a full CAD environment immediately.
- Primitive and archetype definitions should eventually carry visual/diagram metadata.
- The structural-specific Fraia app should likely use nodes, members, plates, supports, and loads as its lowest direct drawing/editing primitives.
- Viewport-attached symbols and labels should be renderer primitives, not DOM overlays.
- The first serious GUI should be a 3D visualizer/inspector with constrained structural editing, not full generic CAD.

---

## 13. Open questions

- Exact rendering schema attached to primitives
- How to represent loads/supports visually in a reusable way
- How much constrained editing should be supported in the first GUI
- When to move from viewer-first to richer direct modeling

---

_End of draft._
