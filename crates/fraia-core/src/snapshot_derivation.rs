//! Snapshot-bound entry points for deterministic engineering consumers.
//!
//! This module deliberately lives in `fraia-core`: the revision repository
//! depends on core, so making core depend on `fraia-revision::ModelSnapshot`
//! would introduce a cycle.  The repository supplies an immutable snapshot id
//! and its immutable model; this adapter carries that identity through every
//! derived result without knowing how snapshots are persisted.

use crate::model_understanding::{ModelUnderstandingReport, understand_structural_model};
use crate::realization::{Frame2DRealization, realize_structural_model_to_frame2d};
use crate::structural_app::StructuralModel;
use crate::validate::{ValidationReport, validate_structural_model};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// An immutable authored model supplied by a repository or other snapshot
/// owner. `fraia-core` intentionally treats the identity as opaque.
#[derive(Debug, Clone, Copy)]
pub struct SnapshotBoundStructuralModel<'a> {
    source_snapshot_id: &'a str,
    model: &'a StructuralModel,
}

impl<'a> SnapshotBoundStructuralModel<'a> {
    pub fn new(source_snapshot_id: &'a str, model: &'a StructuralModel) -> Self {
        Self {
            source_snapshot_id,
            model,
        }
    }

    pub fn source_snapshot_id(&self) -> &str {
        self.source_snapshot_id
    }

    pub fn model(&self) -> &StructuralModel {
        self.model
    }
}

/// A deliberately small initial vocabulary. New deterministic consumers add
/// variants here rather than accepting an untyped workflow/project object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeterministicDerivationRequest {
    Frame2DRealization {
        configuration_version: String,
    },
    /// Reserved until a deterministic 3D realization boundary exists.
    Frame3DRealization {
        configuration_version: String,
    },
}

impl DeterministicDerivationRequest {
    pub fn frame2d() -> Self {
        Self::Frame2DRealization {
            configuration_version: "fraia.frame2d.realization.v1".into(),
        }
    }

    fn kind(&self) -> &'static str {
        match self {
            Self::Frame2DRealization { .. } => "frame2d-realization",
            Self::Frame3DRealization { .. } => "frame3d-realization",
        }
    }

    fn configuration_version(&self) -> &str {
        match self {
            Self::Frame2DRealization {
                configuration_version,
            }
            | Self::Frame3DRealization {
                configuration_version,
            } => configuration_version,
        }
    }
}

/// Typed, inspectable diagnostic for a derivation that cannot be performed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivationDiagnostic {
    pub code: String,
    pub message: String,
}

/// Immutable provenance for a deterministic derivative. The id is a SHA-256
/// identity over the source snapshot id, request configuration, and normalized
/// derived payload; it is not a timestamp or mutable project identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivedManifest {
    pub derived_id: String,
    pub source_snapshot_id: String,
    pub kind: String,
    pub configuration_version: String,
    pub format_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frame2DDerivation {
    pub manifest: DerivedManifest,
    pub understanding: ModelUnderstandingReport,
    pub validation: ValidationReport,
    pub realization: Frame2DRealization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeterministicDerivation {
    Frame2D(Frame2DDerivation),
}

impl DeterministicDerivation {
    pub fn manifest(&self) -> &DerivedManifest {
        match self {
            Self::Frame2D(derivation) => &derivation.manifest,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeterministicDerivationOutcome {
    Derived(DeterministicDerivation),
    Unsupported {
        source_snapshot_id: String,
        request: DeterministicDerivationRequest,
        diagnostic: DerivationDiagnostic,
    },
}

/// Derives deterministic engineering data from a particular immutable model
/// snapshot. This boundary intentionally has no `ProjectFile` parameter.
pub fn derive_from_snapshot(
    snapshot: SnapshotBoundStructuralModel<'_>,
    request: DeterministicDerivationRequest,
) -> DeterministicDerivationOutcome {
    match request {
        request @ DeterministicDerivationRequest::Frame3DRealization { .. } => {
            DeterministicDerivationOutcome::Unsupported {
                source_snapshot_id: snapshot.source_snapshot_id().into(),
                request,
                diagnostic: DerivationDiagnostic {
                    code: "derivation.frame3d-not-supported".into(),
                    message: "Fraia does not yet provide a deterministic frame3d realization."
                        .into(),
                },
            }
        }
        request @ DeterministicDerivationRequest::Frame2DRealization { .. } => {
            let understanding = understand_structural_model(snapshot.model());
            let validation = validate_structural_model(snapshot.model());
            match realize_structural_model_to_frame2d(snapshot.model()) {
                Ok(realization) => {
                    let manifest = manifest_for(
                        snapshot.source_snapshot_id(),
                        &request,
                        &understanding,
                        &validation,
                        &realization,
                    );
                    DeterministicDerivationOutcome::Derived(DeterministicDerivation::Frame2D(
                        Frame2DDerivation {
                            manifest,
                            understanding,
                            validation,
                            realization,
                        },
                    ))
                }
                Err(error) => DeterministicDerivationOutcome::Unsupported {
                    source_snapshot_id: snapshot.source_snapshot_id().into(),
                    request,
                    diagnostic: DerivationDiagnostic {
                        code: "derivation.frame2d-unrealizable".into(),
                        message: error.to_string(),
                    },
                },
            }
        }
    }
}

fn manifest_for(
    source_snapshot_id: &str,
    request: &DeterministicDerivationRequest,
    understanding: &ModelUnderstandingReport,
    validation: &ValidationReport,
    realization: &Frame2DRealization,
) -> DerivedManifest {
    // These are plain structs/vectors with deterministic producer ordering.
    // If a later consumer adds maps, it must canonicalize them before this
    // boundary can claim a stable identity.
    let payload = serde_json::json!({
        "format_version": "fraia.derived.frame2d.v1",
        "source_snapshot_id": source_snapshot_id,
        "request": request,
        "understanding": understanding,
        "validation": validation,
        "realization": realization,
    });
    let bytes = serde_json::to_vec(&payload)
        .expect("snapshot derivation payload consists only of serializable core types");
    let derived_id = format!("sha256:{:x}", Sha256::digest(bytes));

    DerivedManifest {
        derived_id,
        source_snapshot_id: source_snapshot_id.into(),
        kind: request.kind().into(),
        configuration_version: request.configuration_version().into(),
        format_version: "fraia.derived.frame2d.v1".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DeterministicDerivation, DeterministicDerivationOutcome, DeterministicDerivationRequest,
        SnapshotBoundStructuralModel, derive_from_snapshot,
    };
    use crate::structural_app::{
        StructuralMember, StructuralModel, StructuralNode, SupportAssignment,
    };

    fn frame_model() -> StructuralModel {
        StructuralModel {
            dimension: "2d-in-3d".into(),
            nodes: vec![
                StructuralNode {
                    id: "left".into(),
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                StructuralNode {
                    id: "right".into(),
                    x: 6.0,
                    y: 0.0,
                    z: 0.0,
                },
            ],
            members: vec![StructuralMember {
                id: "beam".into(),
                start_node: "left".into(),
                end_node: "right".into(),
                role: "beam".into(),
                semantic_tags: vec![],
                section_id: "250UB".into(),
                material_id: "steel".into(),
            }],
            plates: vec![],
            supports: vec![
                SupportAssignment {
                    id: "left-support".into(),
                    target_node: "left".into(),
                    ux: true,
                    uy: true,
                    uz: true,
                    rx: false,
                    ry: false,
                    rz: true,
                },
                SupportAssignment {
                    id: "right-support".into(),
                    target_node: "right".into(),
                    ux: false,
                    uy: true,
                    uz: true,
                    rx: false,
                    ry: false,
                    rz: true,
                },
            ],
            loads: vec![],
            releases: vec![],
            load_cases: vec![],
            builder_node_materializations: vec![],
        }
    }

    #[test]
    fn same_snapshot_and_configuration_have_the_same_derived_identity_and_output() {
        let model = frame_model();
        let request = DeterministicDerivationRequest::frame2d();
        let first = derive_from_snapshot(
            SnapshotBoundStructuralModel::new("sha256:authored", &model),
            request.clone(),
        );
        let second = derive_from_snapshot(
            SnapshotBoundStructuralModel::new("sha256:authored", &model),
            request,
        );

        let (
            DeterministicDerivationOutcome::Derived(DeterministicDerivation::Frame2D(first)),
            DeterministicDerivationOutcome::Derived(DeterministicDerivation::Frame2D(second)),
        ) = (first, second)
        else {
            panic!("fixture should realize to frame2d");
        };
        assert_eq!(first.manifest.derived_id, second.manifest.derived_id);
        assert_eq!(
            serde_json::to_vec(&first.realization).unwrap(),
            serde_json::to_vec(&second.realization).unwrap()
        );
        assert_eq!(first.manifest.source_snapshot_id, "sha256:authored");
    }

    #[test]
    fn unsupported_derivation_returns_typed_diagnostic_with_source_identity() {
        let model = frame_model();
        let outcome = derive_from_snapshot(
            SnapshotBoundStructuralModel::new("sha256:source", &model),
            DeterministicDerivationRequest::Frame3DRealization {
                configuration_version: "v1".into(),
            },
        );
        let DeterministicDerivationOutcome::Unsupported {
            source_snapshot_id,
            diagnostic,
            ..
        } = outcome
        else {
            panic!("frame3d is not implemented");
        };
        assert_eq!(source_snapshot_id, "sha256:source");
        assert_eq!(diagnostic.code, "derivation.frame3d-not-supported");
    }

    #[test]
    fn unrealizable_frame2d_is_reported_without_losing_snapshot_identity() {
        let mut model = frame_model();
        model.nodes[0].z = 1.0;
        let outcome = derive_from_snapshot(
            SnapshotBoundStructuralModel::new("sha256:out-of-plane", &model),
            DeterministicDerivationRequest::frame2d(),
        );
        let DeterministicDerivationOutcome::Unsupported {
            source_snapshot_id,
            diagnostic,
            ..
        } = outcome
        else {
            panic!("out-of-plane model cannot realize to frame2d");
        };
        assert_eq!(source_snapshot_id, "sha256:out-of-plane");
        assert_eq!(diagnostic.code, "derivation.frame2d-unrealizable");
    }
}
