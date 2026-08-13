//! Immutable authored and derived snapshot records.
//!
//! Snapshot identity is `sha256:<lowercase hex digest>` of canonical bytes.
//! The digest is an identity for content only: timestamps, process order, and
//! mutable repository metadata are intentionally outside it.

use crate::{
    CanonicalFormatVersion, CanonicalStructuralModelSerializer, SnapshotId,
    SnapshotIdentityDeriver,
    canonical::{
        CanonicalJsonStructuralModelSerializer, CanonicalizationError,
        STRUCTURAL_MODEL_CANONICAL_FORMAT,
    },
};
use fraia_core::StructuralModel;
use sha2::{Digest, Sha256};
use std::fmt;

pub const SNAPSHOT_HASH_ALGORITHM: &str = "sha256";
pub const DERIVED_SNAPSHOT_IDENTITY_FORMAT: &str = "fraia.derived-snapshot.identity.v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sha256SnapshotIdentityDeriver;

impl Sha256SnapshotIdentityDeriver {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Sha256SnapshotIdentityDeriver {
    fn default() -> Self {
        Self::new()
    }
}

impl SnapshotIdentityDeriver for Sha256SnapshotIdentityDeriver {
    type Error = std::convert::Infallible;

    fn derive_snapshot_id(&self, canonical_bytes: &[u8]) -> Result<SnapshotId, Self::Error> {
        let digest = Sha256::digest(canonical_bytes);
        Ok(SnapshotId::new(format!(
            "{SNAPSHOT_HASH_ALGORITHM}:{digest:x}"
        )))
    }
}

#[derive(Debug)]
pub enum SnapshotError {
    Canonicalization(CanonicalizationError),
    Deserialization(serde_json::Error),
    Identity(String),
    IdentityMismatch {
        supplied: SnapshotId,
        derived: SnapshotId,
    },
    NonCanonicalBytes,
    UnsupportedFormat {
        expected: &'static str,
        actual: String,
    },
}

impl fmt::Display for SnapshotError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Canonicalization(error) => {
                write!(formatter, "could not create snapshot: {error}")
            }
            Self::Deserialization(error) => {
                write!(
                    formatter,
                    "could not decode canonical snapshot JSON: {error}"
                )
            }
            Self::Identity(error) => {
                write!(formatter, "could not derive snapshot identity: {error}")
            }
            Self::IdentityMismatch { supplied, derived } => write!(
                formatter,
                "canonical snapshot identity `{supplied}` does not match derived identity `{derived}`"
            ),
            Self::NonCanonicalBytes => {
                formatter.write_str("stored snapshot bytes are not canonical")
            }
            Self::UnsupportedFormat { expected, actual } => write!(
                formatter,
                "canonical snapshot format `{actual}` is unsupported; expected `{expected}`"
            ),
        }
    }
}

impl std::error::Error for SnapshotError {}

/// An immutable authored structural state. Its only construction path derives
/// the identifier from canonical bytes; callers receive immutable borrows and
/// cloned values rather than mutable access to stored state.
#[derive(Debug, Clone)]
pub struct ModelSnapshot {
    id: SnapshotId,
    canonical_format_version: CanonicalFormatVersion,
    canonical_bytes: Vec<u8>,
    model: StructuralModel,
}

impl ModelSnapshot {
    pub fn capture(model: StructuralModel) -> Result<Self, SnapshotError> {
        let serializer = CanonicalJsonStructuralModelSerializer::new();
        Self::capture_with(&serializer, &Sha256SnapshotIdentityDeriver, model)
    }

    /// Rehydrates an authored model from durable canonical bytes.
    ///
    /// Decoding is deliberately stricter than ordinary JSON deserialization:
    /// the format version must be known, the bytes must round-trip through the
    /// canonical serializer unchanged, and the supplied content identity must
    /// equal the SHA-256 identity derived from those bytes. This gives a
    /// restart caller a typed snapshot only after its durable record has
    /// passed the same identity boundary used at capture time.
    pub fn from_canonical(
        id: SnapshotId,
        canonical_format_version: CanonicalFormatVersion,
        canonical_bytes: &[u8],
    ) -> Result<Self, SnapshotError> {
        if canonical_format_version.as_str() != STRUCTURAL_MODEL_CANONICAL_FORMAT {
            return Err(SnapshotError::UnsupportedFormat {
                expected: STRUCTURAL_MODEL_CANONICAL_FORMAT,
                actual: canonical_format_version.as_str().to_owned(),
            });
        }

        let model = serde_json::from_slice::<StructuralModel>(canonical_bytes)
            .map_err(SnapshotError::Deserialization)?;
        let captured = Self::capture(model)?;

        if captured.canonical_bytes != canonical_bytes {
            return Err(SnapshotError::NonCanonicalBytes);
        }
        if captured.id != id {
            return Err(SnapshotError::IdentityMismatch {
                supplied: id,
                derived: captured.id,
            });
        }

        Ok(Self {
            id,
            canonical_format_version,
            canonical_bytes: canonical_bytes.to_vec(),
            model: captured.model,
        })
    }

    pub fn capture_with<S, D>(
        serializer: &S,
        identity_deriver: &D,
        model: StructuralModel,
    ) -> Result<Self, SnapshotError>
    where
        S: CanonicalStructuralModelSerializer<Error = CanonicalizationError>,
        D: SnapshotIdentityDeriver,
        D::Error: fmt::Display,
    {
        let canonical_bytes = serializer
            .serialize(&model)
            .map_err(SnapshotError::Canonicalization)?;
        let id = identity_deriver
            .derive_snapshot_id(&canonical_bytes)
            .map_err(|error| SnapshotError::Identity(error.to_string()))?;

        Ok(Self {
            id,
            canonical_format_version: serializer.format_version().clone(),
            canonical_bytes,
            model,
        })
    }

    pub fn id(&self) -> &SnapshotId {
        &self.id
    }

    /// Explicit identity accessor for consumers that should not infer whether
    /// `id` is a display label or a content identity.
    pub fn identity(&self) -> &SnapshotId {
        self.id()
    }

    pub fn canonical_format_version(&self) -> &CanonicalFormatVersion {
        &self.canonical_format_version
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    pub fn model(&self) -> &StructuralModel {
        &self.model
    }

    /// Returns a detached working value. Mutating it cannot mutate this
    /// immutable snapshot; a later explicit capture is required for a child.
    pub fn to_working_model(&self) -> StructuralModel {
        self.model.clone()
    }
}

/// Metadata for any immutable resolved/render/analysis derivative. The
/// derivative bytes have their own hash while retaining their authored source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedSnapshotManifest {
    id: SnapshotId,
    source_snapshot_id: SnapshotId,
    kind: String,
    format_version: CanonicalFormatVersion,
    canonical_payload: Vec<u8>,
}

impl DerivedSnapshotManifest {
    pub fn new(
        source_snapshot_id: SnapshotId,
        kind: impl Into<String>,
        format_version: CanonicalFormatVersion,
        canonical_payload: Vec<u8>,
    ) -> Self {
        let id = Sha256SnapshotIdentityDeriver
            .derive_snapshot_id(&canonical_payload)
            .expect("sha256 identity derivation is infallible");
        Self {
            id,
            source_snapshot_id,
            kind: kind.into(),
            format_version,
            canonical_payload,
        }
    }

    pub fn id(&self) -> &SnapshotId {
        &self.id
    }

    pub fn source_snapshot_id(&self) -> &SnapshotId {
        &self.source_snapshot_id
    }

    pub fn kind(&self) -> &str {
        &self.kind
    }

    pub fn format_version(&self) -> &CanonicalFormatVersion {
        &self.format_version
    }

    pub fn canonical_payload(&self) -> &[u8] {
        &self.canonical_payload
    }
}

#[cfg(test)]
mod tests {
    use super::{DerivedSnapshotManifest, ModelSnapshot, SNAPSHOT_HASH_ALGORITHM};
    use crate::{CanonicalFormatVersion, SnapshotId, root_fixture};
    use fraia_core::StructuralPlate;

    #[test]
    fn snapshots_are_content_addressed_and_ignore_collection_insertion_order() {
        let fixture = root_fixture();
        let original = ModelSnapshot::capture(fixture.model.clone()).unwrap();
        let mut reordered = fixture.model.clone();
        reordered.nodes.reverse();
        reordered.members.reverse();
        reordered.supports.reverse();
        let same = ModelSnapshot::capture(reordered).unwrap();

        assert_eq!(original.id(), same.id());
        assert_eq!(original.canonical_bytes(), same.canonical_bytes());
        assert!(
            original
                .id()
                .as_str()
                .starts_with(&format!("{SNAPSHOT_HASH_ALGORITHM}:"))
        );
    }

    #[test]
    fn meaningful_geometry_role_and_unit_bearing_quantity_changes_have_new_identities() {
        let fixture = root_fixture();
        let baseline = ModelSnapshot::capture(fixture.model.clone()).unwrap();

        let mut moved_node = fixture.model.clone();
        moved_node.nodes[0].x = 0.25;
        let moved_node = ModelSnapshot::capture(moved_node).unwrap();

        let mut changed_role = fixture.model.clone();
        changed_role.members[0].role = "brace".into();
        let changed_role = ModelSnapshot::capture(changed_role).unwrap();

        let mut changed_thickness = fixture.model.clone();
        changed_thickness.plates.push(StructuralPlate {
            id: "mezzanine-slab".into(),
            boundary_nodes: vec![
                "left-base".into(),
                "left-eave".into(),
                "right-eave".into(),
                "right-base".into(),
            ],
            role: "slab".into(),
            semantic_tags: Vec::new(),
            thickness_m: 0.15,
            material_id: "concrete".into(),
            generated_from: "manual".into(),
        });
        let thinner_plate = ModelSnapshot::capture(changed_thickness.clone()).unwrap();
        changed_thickness.plates[0].thickness_m = 0.125;
        let changed_thickness = ModelSnapshot::capture(changed_thickness).unwrap();

        assert_ne!(baseline.id(), moved_node.id());
        assert_ne!(baseline.id(), changed_role.id());
        assert_ne!(thinner_plate.id(), changed_thickness.id());
    }

    #[test]
    fn working_values_cannot_mutate_stored_snapshot_or_derived_manifest() {
        let fixture = root_fixture();
        let snapshot = ModelSnapshot::capture(fixture.model).unwrap();
        let original_id = snapshot.id().clone();
        let original_bytes = snapshot.canonical_bytes().to_vec();

        let mut working = snapshot.to_working_model();
        working.nodes[0].x = 99.0;

        assert_eq!(snapshot.id(), &original_id);
        assert_eq!(snapshot.canonical_bytes(), original_bytes.as_slice());
        assert_ne!(snapshot.model().nodes[0].x, working.nodes[0].x);

        let manifest = DerivedSnapshotManifest::new(
            original_id,
            "resolved-model",
            CanonicalFormatVersion::new("fraia.resolved.v1"),
            br#"{\"resolved\":true}"#.to_vec(),
        );
        assert_eq!(manifest.source_snapshot_id(), snapshot.id());
        assert_eq!(manifest.kind(), "resolved-model");
        assert_eq!(manifest.canonical_payload(), br#"{\"resolved\":true}"#);
    }

    #[test]
    fn canonical_bytes_rehydrate_to_the_same_typed_snapshot() {
        let fixture = root_fixture();
        let snapshot = ModelSnapshot::capture(fixture.model).unwrap();

        let rehydrated = ModelSnapshot::from_canonical(
            snapshot.id().clone(),
            snapshot.canonical_format_version().clone(),
            snapshot.canonical_bytes(),
        )
        .unwrap();

        assert_eq!(rehydrated.id(), snapshot.id());
        assert_eq!(rehydrated.canonical_bytes(), snapshot.canonical_bytes());
        assert_eq!(rehydrated.model().nodes.len(), snapshot.model().nodes.len());
        assert_eq!(
            rehydrated.model().members.len(),
            snapshot.model().members.len()
        );
    }

    #[test]
    fn canonical_rehydration_rejects_noncanonical_or_mismatched_records() {
        let fixture = root_fixture();
        let snapshot = ModelSnapshot::capture(fixture.model).unwrap();
        let mut noncanonical = snapshot.canonical_bytes().to_vec();
        noncanonical.extend_from_slice(b"\n");

        assert!(matches!(
            ModelSnapshot::from_canonical(
                snapshot.id().clone(),
                snapshot.canonical_format_version().clone(),
                &noncanonical,
            ),
            Err(super::SnapshotError::NonCanonicalBytes)
        ));
        assert!(matches!(
            ModelSnapshot::from_canonical(
                SnapshotId::from("sha256:not-the-content"),
                snapshot.canonical_format_version().clone(),
                snapshot.canonical_bytes(),
            ),
            Err(super::SnapshotError::IdentityMismatch { .. })
        ));
    }
}
