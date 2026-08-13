//! Isolated manual-edit sessions over immutable model snapshots.
//!
//! A working copy is intentionally ephemeral. It starts from an immutable
//! revision/snapshot, accumulates local typed edits, and can produce exactly
//! one explicit `manual_edit` commit candidate. Persisting the candidate and
//! advancing a conversation head is the responsibility of the later
//! repository service.

use crate::diff::{SemanticDiff, semantic_diff};
use crate::patch::{PatchError, StructuralPatch, apply_patch};
use crate::snapshot::{ModelSnapshot, SnapshotError};
use crate::{RevisionId, SnapshotId};
use fraia_core::StructuralModel;
use std::error::Error;
use std::fmt;

/// An editor-owned mutable value detached from its immutable source snapshot.
#[derive(Debug, Clone)]
pub struct WorkingCopy {
    parent_revision_id: RevisionId,
    source_snapshot_id: SnapshotId,
    source_model: StructuralModel,
    working_model: StructuralModel,
    closed: bool,
}

/// One accepted manual batch, ready for a repository to append as a child.
#[derive(Debug, Clone)]
pub struct ManualEditCommit {
    revision_id: RevisionId,
    parent_revision_id: RevisionId,
    source_snapshot_id: SnapshotId,
    snapshot: ModelSnapshot,
    semantic_diff: SemanticDiff,
}

impl ManualEditCommit {
    /// The child revision identity chosen by the caller/repository.
    pub fn revision_id(&self) -> &RevisionId {
        &self.revision_id
    }

    /// The immutable proposal/revision that was opened in the editor.
    pub fn parent_revision_id(&self) -> &RevisionId {
        &self.parent_revision_id
    }

    pub fn source_snapshot_id(&self) -> &SnapshotId {
        &self.source_snapshot_id
    }

    /// The immutable snapshot captured once when the user explicitly returns
    /// from the editor.
    pub fn snapshot(&self) -> &ModelSnapshot {
        &self.snapshot
    }

    /// The complete diff from the source snapshot, rather than individual UI
    /// gestures or intermediate patch diffs.
    pub fn semantic_diff(&self) -> &SemanticDiff {
        &self.semantic_diff
    }
}

#[derive(Debug)]
pub enum WorkingCopyError {
    Closed,
    Patch(PatchError),
    Snapshot(SnapshotError),
}

impl From<PatchError> for WorkingCopyError {
    fn from(error: PatchError) -> Self {
        Self::Patch(error)
    }
}

impl From<SnapshotError> for WorkingCopyError {
    fn from(error: SnapshotError) -> Self {
        Self::Snapshot(error)
    }
}

impl fmt::Display for WorkingCopyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Closed => formatter.write_str("working copy is already closed"),
            Self::Patch(error) => error.fmt(formatter),
            Self::Snapshot(error) => error.fmt(formatter),
        }
    }
}

impl Error for WorkingCopyError {}

impl WorkingCopy {
    /// Opens a detached working copy. No mutable borrow of the source snapshot
    /// is retained, so editor changes can never alter historical state.
    pub fn open(parent_revision_id: RevisionId, snapshot: &ModelSnapshot) -> Self {
        let source_model = snapshot.to_working_model();
        Self {
            parent_revision_id,
            source_snapshot_id: snapshot.id().clone(),
            working_model: source_model.clone(),
            source_model,
            closed: false,
        }
    }

    pub fn parent_revision_id(&self) -> &RevisionId {
        &self.parent_revision_id
    }

    pub fn source_snapshot_id(&self) -> &SnapshotId {
        &self.source_snapshot_id
    }

    pub fn model(&self) -> &StructuralModel {
        &self.working_model
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }

    /// Applies a validated typed patch locally. It does not create a revision.
    pub fn apply(&mut self, patch: &StructuralPatch) -> Result<SemanticDiff, WorkingCopyError> {
        if self.closed {
            return Err(WorkingCopyError::Closed);
        }
        let applied = apply_patch(&self.working_model, patch)?;
        self.working_model = applied.model;
        Ok(self.working_model_diff())
    }

    /// Closes this editor session and captures its final state once. The
    /// returned record deliberately includes the original parent and complete
    /// final diff; callers append it atomically through the repository later.
    pub fn commit(
        &mut self,
        revision_id: RevisionId,
    ) -> Result<ManualEditCommit, WorkingCopyError> {
        if self.closed {
            return Err(WorkingCopyError::Closed);
        }
        let snapshot = ModelSnapshot::capture(self.working_model.clone())?;
        let semantic_diff = self.working_model_diff();
        self.closed = true;
        Ok(ManualEditCommit {
            revision_id,
            parent_revision_id: self.parent_revision_id.clone(),
            source_snapshot_id: self.source_snapshot_id.clone(),
            snapshot,
            semantic_diff,
        })
    }

    fn working_model_diff(&self) -> SemanticDiff {
        semantic_diff(&self.source_model, &self.working_model)
    }
}

#[cfg(test)]
mod tests {
    use super::{WorkingCopy, WorkingCopyError};
    use crate::conversation::InMemoryDesignProject;
    use crate::diff::DiffCategory;
    use crate::evidence::{AnalysisEvidence, EvidenceDependency, EvidenceStaleness, staleness_for};
    use crate::patch::{Length, Position, StructuralOperation, StructuralPatch};
    use crate::snapshot::ModelSnapshot;
    use crate::{ConversationId, EvidenceId, ProjectId, RevisionId, root_fixture};

    fn move_node(node_id: &str, x: f64, y: f64) -> StructuralPatch {
        StructuralPatch {
            operations: vec![StructuralOperation::MoveNode {
                node_id: node_id.into(),
                position: Position {
                    x: Length::meters(x),
                    y: Length::meters(y),
                    z: Length::meters(0.0),
                },
            }],
        }
    }

    #[test]
    fn batch_edits_commit_once_with_the_parent_and_complete_diff() {
        let fixture = root_fixture();
        let source = ModelSnapshot::capture(fixture.model.clone()).unwrap();
        let source_bytes = source.canonical_bytes().to_vec();
        let mut copy = WorkingCopy::open(RevisionId::from("proposal-r1"), &source);

        copy.apply(&move_node("left-eave", 0.0, 7.0)).unwrap();
        copy.apply(&move_node("right-eave", 20.0, 7.5)).unwrap();
        copy.apply(&move_node("right-base", 21.0, 0.0)).unwrap();

        let commit = copy.commit(RevisionId::from("manual-r2")).unwrap();
        assert_eq!(
            commit.parent_revision_id(),
            &RevisionId::from("proposal-r1")
        );
        assert_eq!(commit.source_snapshot_id(), source.id());
        assert_eq!(source.canonical_bytes(), source_bytes);
        assert_eq!(source.model().nodes[1].y, 6.0);
        assert_eq!(commit.snapshot().model().nodes[1].y, 7.0);
        assert!(
            commit
                .semantic_diff()
                .changes
                .iter()
                .any(|change| { change.object_kind == "node" && change.object_id == "left-eave" })
        );
        assert!(
            commit
                .semantic_diff()
                .changes
                .iter()
                .any(|change| { change.object_kind == "node" && change.object_id == "right-eave" })
        );
        assert!(
            commit
                .semantic_diff()
                .changes
                .iter()
                .any(|change| { change.object_kind == "node" && change.object_id == "right-base" })
        );
        assert!(copy.is_closed());
        assert!(matches!(
            copy.commit(RevisionId::from("manual-r3")),
            Err(WorkingCopyError::Closed)
        ));
    }

    #[test]
    fn manual_batch_commit_is_one_graph_child_and_stales_only_affected_evidence() {
        let fixture = root_fixture();
        let source = ModelSnapshot::capture(fixture.model.clone()).unwrap();
        let root_revision = RevisionId::from("proposal-r1");
        let conversation = ConversationId::from("overall-framing");
        let mut project = InMemoryDesignProject::create(
            ProjectId::from("warehouse"),
            conversation.clone(),
            "Overall framing",
            root_revision.clone(),
            source.id().clone(),
        )
        .unwrap();
        let mut copy = WorkingCopy::open(root_revision.clone(), &source);

        copy.apply(&move_node("left-eave", 0.0, 7.0)).unwrap();
        copy.apply(&move_node("right-eave", 20.0, 7.5)).unwrap();
        let mut changed_support = copy.model().supports[1].clone();
        changed_support.ux = true;
        copy.apply(&StructuralPatch {
            operations: vec![StructuralOperation::UpdateSupport(changed_support)],
        })
        .unwrap();

        let commit = copy.commit(RevisionId::from("manual-r2")).unwrap();
        project
            .append_revision(
                &conversation,
                &root_revision,
                commit.revision_id().clone(),
                commit.snapshot().id().clone(),
            )
            .unwrap();

        assert_eq!(project.graph().revision_count(), 2);
        assert_eq!(
            project
                .graph()
                .revision(commit.revision_id())
                .unwrap()
                .parent_revision_id(),
            Some(&root_revision)
        );
        assert_eq!(source.model().nodes[1].y, 6.0);
        assert!(commit.semantic_diff().affects(DiffCategory::Geometry));
        assert!(commit.semantic_diff().affects(DiffCategory::Support));

        let affected = AnalysisEvidence::new(
            EvidenceId::from("support-run"),
            source.id().clone(),
            None,
            vec![EvidenceDependency::new(
                "support-restraints",
                "supports:a",
                [DiffCategory::Support],
            )],
        )
        .unwrap();
        let unaffected = AnalysisEvidence::new(
            EvidenceId::from("load-run"),
            source.id().clone(),
            None,
            vec![EvidenceDependency::new(
                "gravity-loads",
                "loads:a",
                [DiffCategory::Load],
            )],
        )
        .unwrap();
        let current_dependencies = vec![
            EvidenceDependency::new("support-restraints", "supports:b", [DiffCategory::Support]),
            EvidenceDependency::new("gravity-loads", "loads:a", [DiffCategory::Load]),
        ];
        assert!(
            staleness_for(
                &affected,
                commit.snapshot().id(),
                &current_dependencies,
                commit.semantic_diff(),
            )
            .is_stale()
        );
        assert_eq!(
            staleness_for(
                &unaffected,
                commit.snapshot().id(),
                &current_dependencies,
                commit.semantic_diff(),
            ),
            EvidenceStaleness::Current
        );
    }
}
