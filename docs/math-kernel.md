# Fraia Math Kernel

_Status: draft v0.1_
_Date: 2026-04-13_

This document defines the current direction for the deepest Fraia math/geometry substrate.

---

## 1. Purpose

The math kernel exists to provide a stable low-level foundation for:

- geometry
- placement
- orientation
- local/global transformations
- physical quantities
- simulation primitives
- later structural and multiphysics domains

This layer should remain:

- small
- stable
- generic
- domain-neutral

It should know nothing about:

- beams
- columns
- supports
- code checks
- structural semantics

---

## 2. Layer position

The math kernel is **Layer 0** in the current abstraction ladder.

Above it sit:

- Layer 1: physical quantities
- Layer 2: generic simulation primitives
- Layer 3+: structural and other domain semantics

---

## 3. Core design rule

Use a **tensor-backed core**, but expose practical typed wrappers.

Meaning:

- `Tensor` is the deepest algebraic substrate
- `Scalar`, `Vector`, and `Matrix` are explicit types built on top of it
- `Point`, `Frame`, and `Transform` remain first-class geometric types

This avoids leaking abstract tensor language into every everyday engineering object.

---

## 4. Foundational types

## 4.1 Tensor

Deep generic numeric/algebraic substrate.

Candidate properties:

- rank
- shape
- component storage
- symmetry metadata
- optional frame/basis metadata

Examples of usage:

- rank 0 tensor -> scalar
- rank 1 tensor -> vector
- rank 2 tensor -> matrix / second-order tensor
- rank 4 tensor -> constitutive tensor later

### Design note

`Tensor` should exist as a deep type, but most project-facing data should use typed wrappers instead.

---

## 4.2 Scalar

A typed wrapper around a rank-0 tensor.

Candidate concerns:

- numeric value
- units/dimensions handled at a higher or adjacent layer
- optional symbolic/reference values later

Examples:

- length
n- angle
- modulus
- temperature
- density

---

## 4.3 Vector

A typed wrapper around a rank-1 tensor.

Important properties:

- ordered components
- dimensionality
- frame reference
- optional unit-vector invariant

Likely concrete variants:

- `Vector2`
- `Vector3`
- `UnitVector3`

### Important rule

A vector is not meaningful without saying what frame/basis it is expressed in.

---

## 4.4 Matrix

A typed wrapper around a rank-2 tensor.

Likely variants:

- `Matrix2`
- `Matrix3`
- `Matrix4`
- `SymmetricMatrix3`

Typical future uses:

- transforms
- stiffness-like local structures
- constitutive relationships
- orientation operators

---

## 4.5 Point

A point should remain a first-class geometric type.

A point is **not the same thing as a vector**.

Candidate properties:

- coordinates
- frame reference

Typical forms:

- `Point2`
- `Point3`

### Design note

Even if represented internally by 2 or 3 components, a point should not be treated as just another vector in the public model.

---

## 4.6 Frame

A frame is a coordinate system / basis definition.

Candidate properties:

- origin point
- basis vectors / axis vectors
- handedness
- parent frame reference
- normalization / orthogonality rules

Likely canonical form:

- `Frame3`

Frames are central to Fraia because:

- loads need frame-aware directions
- member local axes need deterministic definition
- connectivity ports need local reference systems
- assembly placement needs parent-child transforms

---

## 4.7 Transform

A transform maps geometry/quantities between frames.

Candidate properties:

- source frame
- target frame
- translation
- rotation
- maybe affine transform representation

Likely canonical forms:

- `Transform3`
- rigid-body transforms first

### Design note

Scaling should likely not be part of normal structural placement transforms, though generic support may still exist deeper in the stack.

---

## 5. What belongs outside the math kernel

The following should not appear in the deepest math layer:

- stress tensor
- strain tensor
- force vector
- displacement vector
- material law
- beam
- node
- boundary condition

These are all meaningful only after physical or simulation layers are introduced.

---

## 6. Physical quantity wrappers (next layer, not this layer)

These are expected in Layer 1, built on the math kernel:

- `ForceVector3`
- `MomentVector3`
- `DisplacementVector3`
- `StressTensor3`
- `StrainTensor3`
- `TemperatureScalar`
- `HeatFluxVector3`

This means the math kernel should be capable of supporting them, but should not define their engineering meaning.

---

## 7. Minimal initial Fraia math types

Recommended minimum set for the first implementation:

- `Tensor`
- `Scalar`
- `Vector3`
- `UnitVector3`
- `Matrix3`
- `Point3`
- `Frame3`
- `Transform3`

Optional early additions:

- `Matrix4`
- `Plane3`
- `SymmetricMatrix3`

---

## 8. Invariants and validation

### 8.1 UnitVector3
- non-zero
- normalized within tolerance

### 8.2 Frame3
- basis vectors non-zero
- orthogonal within tolerance
- normalized within tolerance
- handedness explicitly defined or derivable

### 8.3 Transform3
- source and target frames valid
- rotation part valid
- no invalid singular transform where not allowed

### 8.4 Point3 / Vector3
- must specify or inherit frame context cleanly

---

## 9. Public schema vs internal implementation

### Public/project-facing data
Prefer named types like:

- `vector3`
- `point3`
- `frame3`
- `transform3`

### Internal engine
May unify algebra using the generic `Tensor` substrate.

This keeps the system:

- rigorous internally
- readable externally
- agent-friendly at the project level

---

## 10. Example style

### Example: vector

```json
{
  "type": "vector3",
  "components": [0, 0, -1],
  "frameRef": "global"
}
```

### Example: point

```json
{
  "type": "point3",
  "coordinates": [0, 0, 6],
  "frameRef": "frame.portal_a"
}
```

### Example: frame

```json
{
  "id": "frame.portal_a",
  "type": "frame3",
  "origin": {
    "type": "point3",
    "coordinates": [0, 0, 0],
    "frameRef": "global"
  },
  "xAxis": {
    "type": "unitVector3",
    "components": [1, 0, 0],
    "frameRef": "global"
  },
  "yAxis": {
    "type": "unitVector3",
    "components": [0, 1, 0],
    "frameRef": "global"
  },
  "zAxis": {
    "type": "unitVector3",
    "components": [0, 0, 1],
    "frameRef": "global"
  }
}
```

---

## 11. Non-goals for v0.1

Not necessary to fully solve immediately:

- symbolic algebra
- arbitrary tensor calculus interfaces
- general differential geometry machinery
- advanced multiphysics field operators
- infinite-dimensional function-space representation

The initial kernel only needs enough to support clean geometry, frames, transforms, and typed physical quantities later.

---

## 12. Key decisions so far

- The deepest algebraic substrate should include a generic `Tensor` type.
- `Scalar`, `Vector`, and `Matrix` should still be explicit named wrappers.
- `Point`, `Frame`, and `Transform` should remain first-class geometric concepts.
- The public model should prefer readable typed objects over generic raw tensor syntax.
- Local/global frame handling is fundamental from day 1.

---

## 13. Open questions

- Should `Tensor` be serialized directly in public project data, or mostly remain internal/advanced?
- Do we need `Vector2`/`Point2` at all if the canonical authored world is always 3D?
- Should transforms be restricted to rigid transforms in the public authored model?
- How should frame inheritance/defaulting work in nested packages and instantiated archetypes?

---

_End of draft._
