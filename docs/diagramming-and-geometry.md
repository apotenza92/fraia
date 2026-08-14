# Fraia Diagramming and Geometry Layer

_Status: draft v0.1_
_Date: 2026-04-13_

This document captures the current direction for Fraia's diagramming, visual geometry, and user-facing model editing layer.

---

## 1. Why this exists

The current MVP still asks for engineering parameters like span and height too early.

That is acceptable for a narrow prototype, but not for the long-term Fraia workflow.

A real Fraia system should be able to move from:

- vague user intent
- to sketched/diagrammed structure
- to structural/archetype model
- to resolved analytical model

This means Fraia needs a diagramming/geometry layer between pure planning and full analysis realization.

---

## 2. Core principle

### Users should not need to think like the solver first
A user often thinks in terms of:

- rooms
- bays
- clear spaces
- roof shape
- walls/columns/frames
- support locations
- openings
- approximate layout

not in terms of:

- exact analysis nodes
- element end releases
- DOF patterns

Therefore Fraia needs an intermediate visual/diagrammatic model.

---

## 3. Proposed layer placement

A useful conceptual stack is:

1. user brief / planning layer
2. diagram / sketch / geometric intent layer
3. structural semantic/archetype layer
4. structural primitive layer
5. simulation/solver realization layer

The diagram layer helps bridge what users mean and what Fraia can analyze.

---

## 4. What diagram information might contain

Diagram information may include:

- points
- curves
- regions
- labels/annotations
- approximate dimensions
- alignment/grid information
- visual grouping
- semantic tags such as wall, frame line, support line, roof edge

This is not yet the full analytical model.

It is a visual/geometric intent representation.

---

## 5. Geometry should attach to primitives and archetypes

Yes: Fraia primitives and archetypes should eventually carry associated geometric/diagram information.

Examples:

- a point primitive can have a visible marker/handle
- a curve member can have a centerline geometry for display
- a support primitive can have a symbol/anchor glyph
- a portal frame archetype can carry a default diagram shape
- a truss archetype can carry a node/curve diagram representation

This is how Fraia can both:

- reason structurally
- and draw/edit visually

---

## 6. Separate visual geometry from analytical realization

This is very important.

A primitive or archetype may need multiple geometry views:

## 6.1 Diagram geometry
Used for:

- sketching
- user interaction
- layout editing
- simple previews

## 6.2 Physical geometry
Used for:

- richer display
- section/profile visualization
- later detailing and documentation

## 6.3 Analytical geometry
Used for:

- centerlines
- surface midsurfaces
- idealized structural regions
- solver realization

These should not be conflated.

---

## 7. Primitive definitions should carry visual metadata

A future primitive/archetype definition may need fields like:

- diagram primitives
- default symbols/glyphs
- handles/control points
- editable dimensions/parameters
- snap/connection anchors

This would make primitives usable in both:

- agent workflows
- graphical editing workflows

---

## 8. User input should often be graphical or semi-graphical

Instead of asking immediately for only spans and heights, Fraia should eventually support workflows like:

- draw a line for a frame
- place supports at ends
- indicate whether internal supports are acceptable
- sketch bays or roof outline
- choose from generated diagram options

This would let users communicate intent more naturally.

---

## 9. Diagram objects as editable semantic handles

A useful Fraia concept may be a diagram object that carries:

- geometry
- semantic meaning
- editable parameters
- links to underlying primitives/archetypes

Examples:

- a frame line diagram object
- a support marker
- a bay dimension handle
- a roof pitch handle

This could become a very powerful middle layer.

---

## 10. Relationship to ports/connectivity

Ports are not only useful for analysis composition.

They can also support diagrammatic editing by providing:

- snap points
- connection handles
- valid attachment interfaces
- local orientation cues

This means the same connectivity ideas can serve both:

- engineering correctness
- interactive geometry editing

---

## 11. Recommendation for Fraia evolution

The current demo's direct span/height questions should be treated as temporary MVP behavior.

A better future Fraia workflow would be:

1. clarify building/system intent
2. choose or sketch a diagrammatic system
3. expose only relevant parameters from that system
4. generate structural options from the chosen diagram/semantic structure

This would make planning feel much more natural.

---

## 12. Design choices currently favored

- Fraia likely needs a diagram/sketch/geometry-intent layer above the analytical model.
- Primitives and archetypes should eventually carry associated visual geometry metadata.
- Diagram geometry, physical geometry, and analytical geometry should be separated.
- The current demo's direct parameter questioning is a temporary simplification, not the ideal long-term UX.

---

## 13. Open questions

- Exact schema for diagram geometry
- How editable geometry should map to archetype parameters
- Whether Fraia should store sketch objects separately from analytical objects or as linked views
- How much of the first GUI should be form-based versus diagram-based

---

## 14. Project files and design references

Imported material should have two scopes.

### Project files

The project's files include the complete imported package: PDFs, images,
DXF files, IFC models, and later supported CAD or neutral 3D formats. It is the
durable project-wide resource bucket. Importing a source does not create or
change a structural model.

Each source records:

- content hash, safe filename, media type, and import time
- page, layout, layer, entity, object, unit, and coordinate metadata when present
- deterministic derived thumbnails, page renderings, text, and geometry indexes
- source warnings, parser version, and extraction provenance

### Design references

Each design has a small set of design references selected from the project
files. A design reference can be:

- one complete source page or model view
- a rectangular or polygonal crop from a PDF page or image
- a selected set of CAD layers, layouts, blocks, or entities
- a selected set of IFC objects, levels, grids, or storeys
- a saved 3D view or section plane

Design references do not copy project files. They retain the exact source hash
and source-space coordinates. A design conversation receives design references
by default and can request another project file only through an explicit
selection. This keeps model context small and makes every interpretation
traceable.

The UI calls this surface **Design references**. Existing persisted Shelf API
identifiers remain unchanged for compatibility; new domain types should use a
precise name such as `DesignReferenceSet` and `DesignReferenceItem`.

## 15. Import trust levels

Different formats carry different kinds of evidence. Fraia must not flatten
them into one generic image-import path.

### PDF and raster drawings

Preserve native page text, paths, dimensions, and transforms when available.
Render selected pages or crops for visual interpretation. Use optical character
recognition only when native text is absent or unusable. Detected lines,
symbols, and labels become diagram observations with confidence and provenance;
they do not become structural objects automatically.

### DXF and two-dimensional CAD

Preserve units, model space, paper-space layouts, layers, blocks, entity ids,
geometry, text, and dimensions. Ask what each selected view represents, whether
it is plan/elevation/section/detail, and how views relate in 3D. A 2D CAD file
must enter the same conversational calibration and confirmation flow as a PDF.

Fraia indexes ASCII DXF with the bounded Rust parser `fraia.ascii-dxf.bounded`.
The parser is part of Fraia's MIT-licensed source. It adds no third-party parser
licence, native library, downloaded runtime, or platform-specific package. The
same Rust code is compiled into each of the six supported desktop targets. It
works offline and keeps Rust as the owner of source identity and provenance.

The index preserves declared insertion units, model and paper layouts, layer
visibility and freeze state, stable handles, classic and lightweight
polylines, text, dimensions, blocks, inserts, and insert transforms. Nested
block references remain references. Fraia does not expand them into structural
geometry. Entity, pair, vertex, block-depth, byte, and parse-time limits fail
closed. Invalid numeric records, reference cycles, binary DXF, and corrupt input
do not publish a partial index. Unsupported entities and missing block
references remain visible diagnostics.

A DXF selection records the exact project file hash, layout, entity ids, and
entity transforms. The user confirms the view role and its relation to design
coordinates once. The parser then creates traceable, unconfirmed drawing
observations. A line remains unclassified linework. It never becomes a member
without later user confirmation and a structural proposal. Direct DWG remains
unsupported and requires its own licence, fidelity, security, and six-target
package decision.

### IFC and semantic BIM

Prefer IFC as the first semantic 3D building import. Preserve object ids,
classes, placements, storeys, grids, properties, geometry, and units. Imported
architectural objects remain references until the user confirms which objects
or derived centre-lines should inform one structural design.

The first IFC backend uses Fraia's bounded in-tree Rust STEP Part 21 subset
parser, `fraia.ifc-step.bounded`. The parser is covered by Fraia's MIT licence.
It introduces no third-party parser licence, native library, worker, runtime
download, or target-specific package payload. It works offline as ordinary Rust
on all six desktop targets. Rust owns the source hash, parser/version identity,
immutable BIM index, and selection provenance.

This subset preserves GlobalId, IFC class, local placement chains, storey and
grid membership, property-set identity and values, declared length units,
representation ids, and reference transforms. It does not tessellate every IFC
representation and does not claim full schema coverage. Unsupported
representations remain visible diagnostics instead of disappearing. Byte,
record, entity, argument, and parse-time limits fail closed.

Users can select exact objects, storeys, grids, or classes. Fraia resolves every
selector to stable object identities and transforms. Parser-created semantic
hints remain unconfirmed reference observations with no design geometry. An IFC
class such as `IFCBEAM` never creates a Fraia member. Centre-lines, surfaces, and
structural meaning require a later reviewed interpretation and explicit
proposal.

### Neutral solids and meshes

STEP and IGES can preserve precise geometric solids but usually carry less
building semantics than IFC. glTF, OBJ, and STL are primarily reference and
visual geometry. Fraia should not infer authored structural members directly
from a mesh without a reviewed interpretation step.

Fraia Phase 1 indexes glTF 2.0, GLB 2.0, OBJ, and STL with the bounded in-tree
Rust parser `fraia.neutral-mesh.bounded`. This parser is part of Fraia's
MIT-licensed source. It adds no third-party parser licence, native library,
worker, downloaded payload, or platform-specific runtime. The same Rust code
compiles offline for all six desktop targets. Rust owns the immutable source
hash, parser/version identity, stable object and group ids, exact glTF node
matrices, mesh bounds, diagnostics, and saved-view provenance.

glTF uses its defined metre and right-handed Y-up conventions. OBJ and STL do
not define units or a reliable coordinate frame. Fraia therefore requires an
engineer to confirm a positive conversion to metres before it can save an OBJ
or STL view as a design reference. Saved views persist the exact source hash,
selected object ids, camera, transform, orientation, scale, and section planes.
Unsupported topology remains a diagnostic. No mesh entity creates a node,
member, plate, support, load, or release.

The Phase 1 parser does not follow external glTF buffers or OBJ material files.
It does not tessellate STEP or IGES. Fraia must not start STEP or IGES support
until a separate geometry-kernel review approves the licence, offline package,
determinism, memory limits, and all six native targets.

The desktop renderer requests mesh bytes by project and source identity. The
main process returns a bounded binary body only after it verifies package
containment, rejects symbolic links, and rechecks the recorded hash and byte
size. It returns the source id, SHA-256 hash, media type, and byte size in
response headers. It never returns a native path. Mesh indexing uses opaque,
bounded main-process jobs with running, cancelling, completed, cancelled, and
failed states. A cancelled parse publishes no partial derivative.

### Proprietary authoring formats

Direct DWG and RVT ingestion is not an initial release requirement. Prefer DXF
and IFC export paths first. Any later proprietary-format adapter must pass a
separate licence, packaging, fidelity, and six-target runtime review.

## 16. Selected-view interpretation

The first drawing workflow should support selecting one plan crop and one
elevation or section crop from different pages. Fraia should let the user:

1. browse project file thumbnails and extracted drawing titles
2. open a page, layout, model view, or 3D source
3. draw one or more crop or selection regions
4. label each item as plan, elevation, section, detail, schedule, or reference
5. calibrate scale and orientation when source units are insufficient
6. add the items as design references
7. align shared grids, levels, points, or axes between design references
8. review the interpreted diagram overlay before any model proposal exists

Multiple design references may describe the same geometry from different views. The
interpretation layer should retain cross-view correspondence and uncertainty
instead of choosing one silently.

### Shipped PDF renderer boundary

Fraia renders managed PDF project files with the Apache-2.0 licensed
`pdfjs-dist` 6.2.108 browser build. Vite bundles the renderer and its dedicated
worker for offline use. Electron main resolves a file only from its project
directory and internal source id, rejects paths outside managed file storage, and
verifies the stored byte count and SHA-256 before returning bytes to the
isolated renderer. The renderer never receives a native file path.

The PDF worker is allowed by the desktop content security policy through
`worker-src 'self' blob:`. The production renderer loads PDF.js only when the
page browser opens. The uncompressed split browser chunk is approximately 428
KB and its worker is 1.26 MB. PDF.js is a build dependency, so its optional Node
canvas package is not a packaged runtime dependency. Fraia does not ship or
invoke Poppler. Poppler remains a test-only visual verification tool.

PDF.js draws pages only. Persisted crop geometry uses the exact project file hash,
page boxes, rotation, user unit, coordinate space, and source-to-display
transform returned by the Rust PDF index. A changed hash or inconsistent page
dimensions makes the browser read-only until the project file is re-indexed.

### Spatial drawing-text inference

The Rust PDF index preserves bounded native text runs with their source-space
boxes, font size, extraction method, and parser version. View-role inference
scores text inside the selected crop above text in a bounded surrounding margin.
It returns ranked plan, elevation, section, detail, and schedule suggestions
with exact cited boxes. A high-confidence, non-conflicting suggestion can prefill
the workflow as **Fraia inferred**. It remains an assumption, not a confirmed
fact. Close competing scores are materially conflicting and require one focused
question. A later user correction creates a descendant interpretation revision
and makes bindings to the older inference stale.

Native text only is used. When a scanned page has no spatial native text, the
backend returns an explicit OCR-unavailable diagnostic and no suggestion. It
does not fabricate a title or view role. Page registers, title-block field
grouping, callout graph matching, and richer rotation-aware glyph transforms
remain follow-on extraction work.

The reviewed first-release OCR runtime is Tesseract.js 7.0.0 with
tesseract.js-core 7.0.0. Fraia packages the local worker and all six core
variants selected by that runtime: baseline, SIMD, and relaxed SIMD, each with
LSTM and non-LSTM forms. It uses only the Apache-2.0 English model from the
exact tessdata_fast 4.1.0 commit recorded in the import runtime contract. Fraia
does not use the English data npm package because its declared licence and
upstream licence provenance conflict. No content-delivery-network or runtime
download path is allowed.

OCR runs only when bounded native PDF text is absent or explicitly unusable.
The worker enforces input byte, pixel, word, character, and elapsed-time limits.
Cancellation terminates the worker. Completed text is returned only as typed,
unconfirmed inferred candidates. Each candidate retains the project file id and
hash, page, crop, rotation, raster and source boxes, exact raster-to-source
transform, confidence, engine version, model commit, and model SHA-256. OCR does
not confirm a view role or create structural geometry. A failed, timed-out,
cancelled, unavailable, or over-limit attempt returns one explicit terminal
diagnostic and publishes no partial candidates.

## 17. Drawing interpretation revision boundary

`DrawingInterpretation` is the versioned boundary between selected source
evidence and a later structural proposal. A revision contains typed drawing
observations, exact Shelf and source coordinates, extraction provenance,
confidence and uncertainty, explicit confirmation state, confirmed cross-view
correspondences and transforms, and unresolved conflicts.

Interpretation revisions belong to one design. They are immutable and use a
content identity plus an exact parent identity. A new revision must compare its
expected parent with the current design head. Parser and extraction adapters
can create unconfirmed observations only. They cannot confirm their own output.

Only a confirmed observation with confirmed design-coordinate geometry and no
unresolved conflict can enter the proposal-constraint projection. This
projection carries exact source provenance. It does not create or mutate
structural primitives. Unknown future revision data must be preserved by a
future schema reader or rejected. A current reader must not silently discard it.

---

_End of draft._
