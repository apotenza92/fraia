//! Immutable design-revision domain boundary.
//!
//! This crate owns canonical snapshots, typed patches and semantic diffs,
//! revision graphs, conversations, immutable evidence, working copies, and
//! SQLite persistence without making `fraia-core` depend on this crate.

use fraia_core::{StructuralMember, StructuralModel, StructuralNode, SupportAssignment};
use serde::{Deserialize, Serialize};
use std::fmt;

pub mod agent_contract;
pub mod analysis_service;
pub mod canonical;
pub mod conversation;
pub mod design_run_adapter;
pub mod diff;
pub mod evidence;
pub mod graph;
pub mod operations;
pub mod patch;
pub mod repository;
pub mod snapshot;
pub mod sqlite;
pub mod working_copy;

macro_rules! typed_id {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self::new(value)
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self::new(value)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }
    };
}

typed_id!(ProjectId, "Stable identity for a Fraia design project.");
typed_id!(
    ConversationId,
    "Stable identity for a design conversation or work item."
);
typed_id!(
    RevisionId,
    "Stable identity for an immutable accepted design revision."
);
typed_id!(
    SnapshotId,
    "Stable identity for an immutable authored model snapshot."
);
typed_id!(
    EvidenceId,
    "Stable identity for immutable analysis evidence."
);
typed_id!(
    ArtefactId,
    "Stable identity for an immutable derived visual artefact."
);

/// Identifies the versioned canonical serialization contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalFormatVersion(String);

impl CanonicalFormatVersion {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Canonical bytes must describe an authored `StructuralModel`, not UI,
/// conversation, timestamp, or process state.
pub trait CanonicalStructuralModelSerializer {
    type Error;

    fn format_version(&self) -> &CanonicalFormatVersion;
    fn serialize(&self, model: &StructuralModel) -> Result<Vec<u8>, Self::Error>;
}

/// Derives an opaque snapshot identity from canonical authored-model bytes.
/// Production uses SHA-256 and never derives identity from timestamps or
/// mutable project state.
pub trait SnapshotIdentityDeriver {
    type Error;

    fn derive_snapshot_id(&self, canonical_bytes: &[u8]) -> Result<SnapshotId, Self::Error>;
}

/// Stable test fixture that uses the primitive-first
/// `fraia-core::StructuralModel` vocabulary without introducing a second model
/// representation.
#[derive(Debug, Clone)]
pub struct RevisionFixture {
    pub project_id: ProjectId,
    pub conversation_id: ConversationId,
    pub root_revision_id: RevisionId,
    pub root_snapshot_id: SnapshotId,
    pub model: StructuralModel,
}

/// Returns a small, semantically meaningful steel portal-frame fixture.
///
/// Fixture ids remain readable labels. Production snapshots use canonical,
/// hash-derived identities.
pub fn root_fixture() -> RevisionFixture {
    RevisionFixture {
        project_id: ProjectId::from("fixture-project"),
        conversation_id: ConversationId::from("overall-framing"),
        root_revision_id: RevisionId::from("fixture-root-revision"),
        root_snapshot_id: SnapshotId::from("fixture-root-snapshot"),
        model: StructuralModel {
            dimension: "2d-in-3d".into(),
            nodes: vec![
                StructuralNode {
                    id: "left-base".into(),
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                StructuralNode {
                    id: "left-eave".into(),
                    x: 0.0,
                    y: 6.0,
                    z: 0.0,
                },
                StructuralNode {
                    id: "right-eave".into(),
                    x: 20.0,
                    y: 6.0,
                    z: 0.0,
                },
                StructuralNode {
                    id: "right-base".into(),
                    x: 20.0,
                    y: 0.0,
                    z: 0.0,
                },
            ],
            members: vec![
                StructuralMember {
                    id: "left-column".into(),
                    start_node: "left-base".into(),
                    end_node: "left-eave".into(),
                    role: "column".into(),
                    semantic_tags: vec!["portal-frame".into()],
                    section_id: "310UC97".into(),
                    material_id: "steel".into(),
                },
                StructuralMember {
                    id: "rafter".into(),
                    start_node: "left-eave".into(),
                    end_node: "right-eave".into(),
                    role: "rafter".into(),
                    semantic_tags: vec!["portal-frame".into()],
                    section_id: "410UB54".into(),
                    material_id: "steel".into(),
                },
                StructuralMember {
                    id: "right-column".into(),
                    start_node: "right-eave".into(),
                    end_node: "right-base".into(),
                    role: "column".into(),
                    semantic_tags: vec!["portal-frame".into()],
                    section_id: "310UC97".into(),
                    material_id: "steel".into(),
                },
            ],
            plates: Vec::new(),
            supports: vec![
                SupportAssignment {
                    id: "left-base-support".into(),
                    target_node: "left-base".into(),
                    ux: true,
                    uy: true,
                    uz: true,
                    rx: false,
                    ry: false,
                    rz: false,
                },
                SupportAssignment {
                    id: "right-base-support".into(),
                    target_node: "right-base".into(),
                    ux: false,
                    uy: true,
                    uz: true,
                    rx: false,
                    ry: false,
                    rz: false,
                },
            ],
            loads: Vec::new(),
            releases: Vec::new(),
            load_cases: Vec::new(),
            builder_node_materializations: Vec::new(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{ConversationId, ProjectId, RevisionId, SnapshotId, root_fixture};

    #[test]
    fn root_fixture_uses_existing_structural_model_vocabulary() {
        let fixture = root_fixture();

        assert_eq!(fixture.model.dimension, "2d-in-3d");
        assert_eq!(fixture.model.nodes.len(), 4);
        assert_eq!(fixture.model.members.len(), 3);
        assert_eq!(fixture.model.members[1].role, "rafter");
        assert_eq!(fixture.model.supports.len(), 2);
        assert_eq!(fixture.model.supports[0].target_node, "left-base");
    }

    #[test]
    fn typed_ids_are_distinct_serializable_domain_values() {
        let project = ProjectId::from("project-a");
        let conversation = ConversationId::from("overall-framing");
        let revision = RevisionId::from("revision-1");
        let snapshot = SnapshotId::from("snapshot-1");

        assert_eq!(project.as_str(), "project-a");
        assert_eq!(conversation.to_string(), "overall-framing");
        assert_eq!(revision.to_string(), "revision-1");
        assert_eq!(snapshot.to_string(), "snapshot-1");
        assert_eq!(serde_json::to_string(&project).unwrap(), "\"project-a\"");
    }
}
