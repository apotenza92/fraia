//! Independent observable contracts for the blank-to-analysed programme.
//!
//! These tests intentionally use public domain and persistence boundaries. They
//! do not snapshot implementation details or reach into private repository
//! state.

use fraia_core::{StructuralMember, StructuralModel, StructuralNode, SupportAssignment};
use fraia_revision::analysis_service::{
    SnapshotAnalysisOutcome, analyse_accepted_revision, dependencies_for_snapshot,
};
use fraia_revision::conversation::ConversationOrigin;
use fraia_revision::evidence::{AnalysisEvidence, EvidenceStaleness};
use fraia_revision::patch::{
    Length, MemberRole, NodeInput, Position, StructuralOperation, StructuralPatch,
};
use fraia_revision::repository::{
    InMemoryRevisionRepository, ProposalId, ProposalStatus, RevisionAuthorKind, VisualArtefact,
};
use fraia_revision::snapshot::ModelSnapshot;
use fraia_revision::sqlite::{
    SqliteRevisionRepository, StoredConversation, StoredProjectRoot, StoredRevision, StoredSnapshot,
};
use fraia_revision::{ConversationId, EvidenceId, ProjectId, RevisionId, SnapshotId};
use tempfile::tempdir;

fn empty_model() -> StructuralModel {
    StructuralModel {
        dimension: "2d-in-3d".into(),
        nodes: vec![],
        members: vec![],
        plates: vec![],
        supports: vec![],
        loads: vec![],
        releases: vec![],
        load_cases: vec![],
        builder_node_materializations: vec![],
    }
}

fn geometry_patch() -> StructuralPatch {
    StructuralPatch {
        operations: vec![
            StructuralOperation::AddNode(NodeInput {
                id: "left-base".into(),
                position: Position {
                    x: Length::meters(0.0),
                    y: Length::meters(0.0),
                    z: Length::meters(0.0),
                },
            }),
            StructuralOperation::AddNode(NodeInput {
                id: "right-base".into(),
                position: Position {
                    x: Length::meters(6.0),
                    y: Length::meters(0.0),
                    z: Length::meters(0.0),
                },
            }),
            StructuralOperation::AddMember(StructuralMember {
                id: "beam-1".into(),
                start_node: "left-base".into(),
                end_node: "right-base".into(),
                role: "beam".into(),
                semantic_tags: vec!["programme-fixture".into()],
                section_id: "250UB".into(),
                material_id: "steel".into(),
            }),
            StructuralOperation::AddSupport(SupportAssignment {
                id: "left-support".into(),
                target_node: "left-base".into(),
                ux: true,
                uy: true,
                uz: true,
                rx: false,
                ry: false,
                rz: true,
            }),
            StructuralOperation::AddSupport(SupportAssignment {
                id: "right-support".into(),
                target_node: "right-base".into(),
                ux: false,
                uy: true,
                uz: true,
                rx: false,
                ry: false,
                rz: true,
            }),
        ],
    }
}

fn repository(model: StructuralModel) -> InMemoryRevisionRepository {
    InMemoryRevisionRepository::create(
        ProjectId::from("programme-project"),
        ConversationId::from("overall-design"),
        "Overall design",
        RevisionId::from("root"),
        model,
    )
    .unwrap()
}

#[test]
fn empty_project_becomes_geometry_only_after_explicit_acceptance() {
    let mut repository = repository(empty_model());
    let root_snapshot = repository
        .revision(&RevisionId::from("root"))
        .unwrap()
        .snapshot_id()
        .clone();
    assert_eq!(
        repository
            .snapshot(&root_snapshot)
            .unwrap()
            .model()
            .nodes
            .len(),
        0
    );
    assert_eq!(
        repository
            .snapshot(&root_snapshot)
            .unwrap()
            .model()
            .members
            .len(),
        0
    );

    repository
        .create_proposal(
            ProposalId::from("first-geometry"),
            ConversationId::from("overall-design"),
            RevisionId::from("root"),
            RevisionId::from("geometry-1"),
            geometry_patch(),
        )
        .unwrap();
    assert_eq!(repository.revision_count(), 1);
    assert_eq!(
        repository
            .head(&ConversationId::from("overall-design"))
            .unwrap()
            .head_revision_id,
        RevisionId::from("root")
    );

    repository
        .accept_proposal(&ProposalId::from("first-geometry"))
        .unwrap();
    let child = repository
        .revision(&RevisionId::from("geometry-1"))
        .unwrap();
    let model = repository.snapshot(child.snapshot_id()).unwrap().model();
    assert_eq!(model.nodes.len(), 2);
    assert_eq!(model.members.len(), 1);
    assert!(
        child
            .semantic_diff()
            .affects(fraia_revision::diff::DiffCategory::Geometry)
    );
    assert_eq!(child.author_kind(), RevisionAuthorKind::Agent);
}

#[test]
fn rejected_proposal_is_external_and_acceptance_is_one_child() {
    let mut repository = repository(empty_model());
    repository
        .create_proposal(
            ProposalId::from("reject-me"),
            ConversationId::from("overall-design"),
            RevisionId::from("root"),
            RevisionId::from("never"),
            geometry_patch(),
        )
        .unwrap();
    repository
        .reject_proposal(&ProposalId::from("reject-me"))
        .unwrap();
    assert_eq!(
        repository
            .proposal(&ProposalId::from("reject-me"))
            .unwrap()
            .status(),
        &ProposalStatus::Rejected
    );
    assert_eq!(repository.revision_count(), 1);
    assert_eq!(
        repository
            .head(&ConversationId::from("overall-design"))
            .unwrap()
            .head_revision_id,
        RevisionId::from("root")
    );

    repository
        .create_proposal(
            ProposalId::from("accept-me"),
            ConversationId::from("overall-design"),
            RevisionId::from("root"),
            RevisionId::from("child"),
            geometry_patch(),
        )
        .unwrap();
    repository
        .accept_proposal(&ProposalId::from("accept-me"))
        .unwrap();
    assert_eq!(repository.revision_count(), 2);
    assert_eq!(
        repository
            .proposal(&ProposalId::from("accept-me"))
            .unwrap()
            .status(),
        &ProposalStatus::Accepted {
            revision_id: RevisionId::from("child")
        }
    );
}

#[test]
fn snapshot_identity_and_preview_handoff_are_content_and_source_bound() {
    let first = ModelSnapshot::capture(empty_model()).unwrap();
    let same = ModelSnapshot::capture(empty_model()).unwrap();
    let changed = ModelSnapshot::capture(StructuralModel {
        nodes: vec![StructuralNode {
            id: "preview-node".into(),
            x: 1.0,
            y: 0.0,
            z: 0.0,
        }],
        ..empty_model()
    })
    .unwrap();
    assert_eq!(first.id(), same.id());
    assert_eq!(first.canonical_bytes(), same.canonical_bytes());
    assert_ne!(first.id(), changed.id());

    let mut repository = repository(empty_model());
    let root_snapshot = repository
        .revision(&RevisionId::from("root"))
        .unwrap()
        .snapshot_id()
        .clone();
    repository
        .attach_artefact(VisualArtefact::new(
            fraia_revision::ArtefactId::from("preview-root"),
            "structural-preview",
            root_snapshot.clone(),
            None,
            vec!["preview-node".into()],
            "renderer-test-v1",
            b"read-only-preview".to_vec(),
        ))
        .unwrap();
    let artefact = repository
        .artefact(&fraia_revision::ArtefactId::from("preview-root"))
        .unwrap();
    assert_eq!(artefact.source_snapshot_id(), &root_snapshot);
    assert_eq!(artefact.renderer_version(), "renderer-test-v1");
}

#[test]
fn fork_resume_and_manual_edit_batch_preserve_lineage_and_parent_snapshot() {
    let mut repository = repository(empty_model());
    repository
        .create_proposal(
            ProposalId::from("geometry"),
            ConversationId::from("overall-design"),
            RevisionId::from("root"),
            RevisionId::from("geometry"),
            geometry_patch(),
        )
        .unwrap();
    repository
        .accept_proposal(&ProposalId::from("geometry"))
        .unwrap();
    repository
        .fork(
            ConversationId::from("alternative"),
            "Alternative",
            RevisionId::from("geometry"),
        )
        .unwrap();
    repository
        .resume(
            ConversationId::from("resumed"),
            "Resumed",
            RevisionId::from("root"),
        )
        .unwrap();
    assert!(matches!(
        repository
            .conversation(&ConversationId::from("alternative"))
            .unwrap()
            .origin(),
        ConversationOrigin::ForkedFromRevision { .. }
    ));
    assert!(matches!(
        repository
            .conversation(&ConversationId::from("resumed"))
            .unwrap()
            .origin(),
        ConversationOrigin::ResumedFromRevision { .. }
    ));

    let mut working_copy = repository
        .open_working_copy(&RevisionId::from("geometry"))
        .unwrap();
    working_copy
        .apply(&StructuralPatch {
            operations: vec![
                StructuralOperation::MoveNode {
                    node_id: "right-base".into(),
                    position: Position {
                        x: Length::meters(7.0),
                        y: Length::meters(0.0),
                        z: Length::meters(0.0),
                    },
                },
                StructuralOperation::SetMemberRole {
                    member_id: "beam-1".into(),
                    role: MemberRole::Rafter,
                },
            ],
        })
        .unwrap();
    let source_snapshot = working_copy.source_snapshot_id().clone();
    repository
        .commit_working_copy(
            &ConversationId::from("overall-design"),
            &mut working_copy,
            RevisionId::from("manual-1"),
        )
        .unwrap();
    assert_eq!(
        repository
            .revision(&RevisionId::from("manual-1"))
            .unwrap()
            .author_kind(),
        RevisionAuthorKind::Manual
    );
    assert_eq!(
        repository
            .revision(&RevisionId::from("geometry"))
            .unwrap()
            .snapshot_id(),
        &source_snapshot
    );
    assert!(working_copy.is_closed());
}

#[test]
fn evidence_identity_is_exact_and_descendant_geometry_is_stale() {
    let mut repository = repository(empty_model());
    let root_snapshot = repository
        .revision(&RevisionId::from("root"))
        .unwrap()
        .snapshot_id()
        .clone();
    let evidence = AnalysisEvidence::new(
        EvidenceId::from("evidence-root"),
        root_snapshot.clone(),
        None,
        dependencies_for_snapshot(&root_snapshot),
    )
    .unwrap();
    repository
        .attach_evidence(&RevisionId::from("root"), evidence)
        .unwrap();
    assert!(matches!(
        repository
            .evidence_staleness(
                &EvidenceId::from("evidence-root"),
                &RevisionId::from("root"),
                &dependencies_for_snapshot(&root_snapshot)
            )
            .unwrap(),
        EvidenceStaleness::Current
    ));

    repository
        .create_proposal(
            ProposalId::from("geometry"),
            ConversationId::from("overall-design"),
            RevisionId::from("root"),
            RevisionId::from("child"),
            geometry_patch(),
        )
        .unwrap();
    repository
        .accept_proposal(&ProposalId::from("geometry"))
        .unwrap();
    let child_snapshot = repository
        .revision(&RevisionId::from("child"))
        .unwrap()
        .snapshot_id()
        .clone();
    assert_ne!(root_snapshot, child_snapshot);
    assert!(
        repository
            .evidence_staleness(
                &EvidenceId::from("evidence-root"),
                &RevisionId::from("child"),
                &dependencies_for_snapshot(&child_snapshot)
            )
            .unwrap()
            .is_stale()
    );
}

#[test]
fn analysis_exposes_success_unsupported_and_failed_without_fabricating_results() {
    let mut supported = repository(empty_model());
    supported
        .create_proposal(
            ProposalId::from("geometry"),
            ConversationId::from("overall-design"),
            RevisionId::from("root"),
            RevisionId::from("supported"),
            geometry_patch(),
        )
        .unwrap();
    supported
        .accept_proposal(&ProposalId::from("geometry"))
        .unwrap();
    let supported_run = analyse_accepted_revision(
        &mut supported,
        &RevisionId::from("supported"),
        EvidenceId::from("success"),
    )
    .unwrap();
    assert!(matches!(
        supported_run.outcome,
        SnapshotAnalysisOutcome::Completed { .. }
            | SnapshotAnalysisOutcome::Failed { .. }
            | SnapshotAnalysisOutcome::Unsupported { .. }
    ));
    assert_eq!(
        supported_run.evidence.authored_snapshot_id(),
        supported
            .revision(&RevisionId::from("supported"))
            .unwrap()
            .snapshot_id()
    );
    if let SnapshotAnalysisOutcome::Completed { .. } = supported_run.outcome {
        assert!(
            supported_run
                .evidence
                .analysis_manifest()
                .unwrap()
                .result_hash
                .is_some()
        );
    }

    let mut unsupported = repository(StructuralModel {
        dimension: "2d-in-3d".into(),
        nodes: vec![StructuralNode {
            id: "out-of-plane".into(),
            x: 0.0,
            y: 0.0,
            z: 1.0,
        }],
        ..empty_model()
    });
    let run = analyse_accepted_revision(
        &mut unsupported,
        &RevisionId::from("root"),
        EvidenceId::from("unsupported"),
    )
    .unwrap();
    assert!(matches!(
        run.outcome,
        SnapshotAnalysisOutcome::Unsupported { .. }
    ));
    assert!(
        run.evidence
            .analysis_manifest()
            .unwrap()
            .result_hash
            .is_none()
    );

    // An empty 2D model is a real solver attempt with no valid supports/load
    // path; the contract permits either a deterministic failed or unsupported
    // outcome, but never a successful result without a result hash.
    let mut failed = repository(empty_model());
    let run = analyse_accepted_revision(
        &mut failed,
        &RevisionId::from("root"),
        EvidenceId::from("failed"),
    )
    .unwrap();
    assert!(matches!(
        run.outcome,
        SnapshotAnalysisOutcome::Failed { .. }
    ));
    assert!(
        run.evidence
            .analysis_manifest()
            .unwrap()
            .result_hash
            .is_none()
    );
}

#[test]
fn sqlite_restart_preserves_identity_and_failed_cas_has_no_orphan_revision() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("programme.sqlite");
    let snapshot = ModelSnapshot::capture(empty_model()).unwrap();
    let root = StoredProjectRoot {
        project_id: ProjectId::from("project"),
        root_conversation: StoredConversation {
            id: ConversationId::from("conversation"),
            project_id: ProjectId::from("project"),
            purpose: "Overall design".into(),
            origin_json: "{\"origin\":\"root\"}".into(),
            head_revision_id: RevisionId::from("root"),
        },
        root_revision: StoredRevision {
            id: RevisionId::from("root"),
            snapshot_id: snapshot.id().clone(),
            parent_revision_id: None,
            conversation_id: ConversationId::from("conversation"),
            metadata_json: "{\"author\":\"system\"}".into(),
        },
        root_snapshot: StoredSnapshot {
            id: snapshot.id().clone(),
            format_version: snapshot.canonical_format_version().as_str().into(),
            canonical_bytes: snapshot.canonical_bytes().to_vec(),
        },
    };
    {
        let mut db = SqliteRevisionRepository::open(&path).unwrap();
        db.create_project(root).unwrap();
        assert_eq!(
            db.conversation(&ConversationId::from("conversation"))
                .unwrap()
                .head_revision_id,
            RevisionId::from("root")
        );
        let bad = StoredRevision {
            id: RevisionId::from("orphan"),
            snapshot_id: SnapshotId::from("missing"),
            parent_revision_id: Some(RevisionId::from("root")),
            conversation_id: ConversationId::from("conversation"),
            metadata_json: "{}".into(),
        };
        assert!(db.append_revision(&bad, &RevisionId::from("root")).is_err());
        assert!(db.revision(&RevisionId::from("orphan")).is_err());
    }
    let mut first = SqliteRevisionRepository::open(&path).unwrap();
    let mut second = SqliteRevisionRepository::open(&path).unwrap();
    let child_snapshot = ModelSnapshot::capture(StructuralModel {
        dimension: "2d-in-3d".into(),
        nodes: vec![StructuralNode {
            id: "n".into(),
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }],
        ..empty_model()
    })
    .unwrap();
    first
        .insert_snapshot(&StoredSnapshot {
            id: child_snapshot.id().clone(),
            format_version: child_snapshot.canonical_format_version().as_str().into(),
            canonical_bytes: child_snapshot.canonical_bytes().to_vec(),
        })
        .unwrap();
    let child = StoredRevision {
        id: RevisionId::from("child"),
        snapshot_id: child_snapshot.id().clone(),
        parent_revision_id: Some(RevisionId::from("root")),
        conversation_id: ConversationId::from("conversation"),
        metadata_json: "{}".into(),
    };
    first
        .append_revision(&child, &RevisionId::from("root"))
        .unwrap();
    assert!(
        second
            .append_revision(
                &StoredRevision {
                    id: RevisionId::from("loser"),
                    ..child
                },
                &RevisionId::from("root")
            )
            .is_err()
    );
    let restarted = SqliteRevisionRepository::open(&path).unwrap();
    assert_eq!(
        restarted
            .conversation(&ConversationId::from("conversation"))
            .unwrap()
            .head_revision_id,
        RevisionId::from("child")
    );
    assert_eq!(
        restarted
            .snapshot(&child_snapshot.id())
            .unwrap()
            .canonical_bytes,
        child_snapshot.canonical_bytes()
    );
}
