use fraia_core::{
    AssignmentTargetRef, LoadAssignment, LoadKind, LoadVector, StructuralMember, StructuralModel,
    StructuralNode, SupportAssignment,
};
use fraia_revision::analysis_service::{AnalysisSettings, SnapshotAnalysisOutcome};
use fraia_revision::evidence::{AnalysisEvidenceStatus, EvidenceStaleness};
use fraia_revision::operations::{
    OPERATION_CONTRACT_VERSION, Operation, OperationErrorCode, OperationOutcome, OperationRequest,
    OperationResult, execute_operation, execute_sqlite_operation,
};
use fraia_revision::patch::{Length, Position, StructuralOperation, StructuralPatch};
use fraia_revision::repository::{InMemoryRevisionRepository, ProposalId};
use fraia_revision::snapshot::ModelSnapshot;
use fraia_revision::sqlite::{
    SqliteRevisionRepository, StoredConversation, StoredProjectRoot, StoredRevision, StoredSnapshot,
};
use fraia_revision::{ConversationId, EvidenceId, RevisionId, root_fixture};
use tempfile::tempdir;

fn seeded_repository(path: &std::path::Path) -> SqliteRevisionRepository {
    let fixture = root_fixture();
    let snapshot = ModelSnapshot::capture(fixture.model).unwrap();
    let stored_snapshot = StoredSnapshot {
        id: snapshot.id().clone(),
        format_version: snapshot.canonical_format_version().as_str().into(),
        canonical_bytes: snapshot.canonical_bytes().to_vec(),
    };
    let mut repository = SqliteRevisionRepository::open(path).unwrap();
    repository
        .create_project(StoredProjectRoot {
            project_id: fixture.project_id.clone(),
            root_conversation: StoredConversation {
                id: fixture.conversation_id.clone(),
                project_id: fixture.project_id,
                purpose: "overall framing".into(),
                origin_json: "{\"kind\":\"root\"}".into(),
                head_revision_id: fixture.root_revision_id.clone(),
            },
            root_revision: StoredRevision {
                id: fixture.root_revision_id,
                snapshot_id: stored_snapshot.id.clone(),
                parent_revision_id: None,
                conversation_id: fixture.conversation_id,
                metadata_json: "{\"operation\":\"root\"}".into(),
            },
            root_snapshot: stored_snapshot,
        })
        .unwrap();
    repository
}

fn seeded_model_repository(
    path: &std::path::Path,
    model: fraia_core::StructuralModel,
) -> (SqliteRevisionRepository, fraia_revision::SnapshotId) {
    let snapshot = ModelSnapshot::capture(model).unwrap();
    let id = snapshot.id().clone();
    let stored_snapshot = StoredSnapshot {
        id: id.clone(),
        format_version: snapshot.canonical_format_version().as_str().into(),
        canonical_bytes: snapshot.canonical_bytes().to_vec(),
    };
    let mut repository = SqliteRevisionRepository::open(path).unwrap();
    repository
        .create_project(StoredProjectRoot {
            project_id: fraia_revision::ProjectId::from("model-project"),
            root_conversation: StoredConversation {
                id: ConversationId::from("model-conversation"),
                project_id: fraia_revision::ProjectId::from("model-project"),
                purpose: "model analysis".into(),
                origin_json: "{\"kind\":\"root\"}".into(),
                head_revision_id: RevisionId::from("model-root"),
            },
            root_revision: StoredRevision {
                id: RevisionId::from("model-root"),
                snapshot_id: id.clone(),
                parent_revision_id: None,
                conversation_id: ConversationId::from("model-conversation"),
                metadata_json: "{\"operation\":\"root\"}".into(),
            },
            root_snapshot: stored_snapshot,
        })
        .unwrap();
    (repository, id)
}

fn analysed_model() -> StructuralModel {
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
        loads: vec![LoadAssignment {
            id: "gravity".into(),
            target: AssignmentTargetRef::Member("beam".into()),
            load_case_id: "dead".into(),
            kind: LoadKind::UniformLine,
            direction: LoadVector {
                x: 0.0,
                y: -1.0,
                z: 0.0,
            },
            magnitude: 10_000.0,
        }],
        releases: vec![],
        load_cases: vec![],
        builder_node_materializations: vec![],
    }
}

fn patch() -> StructuralPatch {
    StructuralPatch {
        operations: vec![StructuralOperation::MoveNode {
            node_id: "left-eave".into(),
            position: Position {
                x: Length::meters(0.0),
                y: Length::meters(6.5),
                z: Length::meters(0.0),
            },
        }],
    }
}

fn propose(request_id: &str) -> OperationRequest {
    OperationRequest {
        contract_version: OPERATION_CONTRACT_VERSION.into(),
        request_id: request_id.into(),
        operation: Operation::ProposeStructuralPatch {
            proposal_id: ProposalId::from("proposal-1"),
            conversation_id: ConversationId::from("overall-framing"),
            expected_head_revision_id: RevisionId::from("fixture-root-revision"),
            proposed_revision_id: RevisionId::from("revision-1"),
            patch: patch(),
            agent_provenance: None,
        },
    }
}

#[test]
fn durable_propose_accept_restart_and_exact_request_replay() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("operations.sqlite");
    let mut repository = seeded_repository(&path);

    let proposed = execute_sqlite_operation(&mut repository, propose("propose-1"));
    assert!(matches!(
        proposed.outcome,
        OperationOutcome::Success { ref result }
            if matches!(**result, OperationResult::StructuralPatchProposed { .. })
    ));
    let replay = execute_sqlite_operation(&mut repository, propose("propose-1"));
    assert_eq!(
        serde_json::to_value(&replay).unwrap(),
        serde_json::to_value(&proposed).unwrap()
    );

    drop(repository);
    let mut repository = SqliteRevisionRepository::open(&path).unwrap();
    let accept = OperationRequest {
        contract_version: OPERATION_CONTRACT_VERSION.into(),
        request_id: "accept-1".into(),
        operation: Operation::AcceptStructuralPatch {
            proposal_id: ProposalId::from("proposal-1"),
            conversation_id: ConversationId::from("overall-framing"),
            expected_head_revision_id: RevisionId::from("fixture-root-revision"),
        },
    };
    let accepted = execute_sqlite_operation(&mut repository, accept.clone());
    assert!(matches!(
        accepted.outcome,
        OperationOutcome::Success { ref result }
            if matches!(**result, OperationResult::StructuralPatchAccepted { .. })
    ));
    let replay = execute_sqlite_operation(&mut repository, accept.clone());
    assert_eq!(
        serde_json::to_value(&replay).unwrap(),
        serde_json::to_value(&accepted).unwrap()
    );
    let mut semantic_replay = accept;
    semantic_replay.request_id = "accept-2".into();
    let semantic_replay = execute_sqlite_operation(&mut repository, semantic_replay);
    assert!(matches!(
        semantic_replay.outcome,
        OperationOutcome::Success { ref result }
            if matches!(**result, OperationResult::StructuralPatchAccepted { .. })
    ));
    assert_eq!(
        repository
            .conversation(&ConversationId::from("overall-framing"))
            .unwrap()
            .head_revision_id,
        RevisionId::from("revision-1")
    );
}

#[test]
fn request_id_collision_and_stale_head_are_stable_failures_without_mutation() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("conflicts.sqlite");
    let mut repository = seeded_repository(&path);
    let first = execute_sqlite_operation(&mut repository, propose("same-id"));
    assert!(matches!(first.outcome, OperationOutcome::Success { .. }));

    let collision = execute_sqlite_operation(
        &mut repository,
        OperationRequest {
            contract_version: OPERATION_CONTRACT_VERSION.into(),
            request_id: "same-id".into(),
            operation: Operation::Inspect {
                conversation_id: ConversationId::from("overall-framing"),
            },
        },
    );
    assert!(matches!(
        collision.outcome,
        OperationOutcome::Error { ref error }
            if error.code == OperationErrorCode::InvalidRequest
    ));

    let mut stale = propose("stale");
    let Operation::ProposeStructuralPatch {
        expected_head_revision_id,
        proposal_id,
        ..
    } = &mut stale.operation
    else {
        unreachable!()
    };
    *expected_head_revision_id = RevisionId::from("stale-revision");
    *proposal_id = ProposalId::from("stale-proposal");
    let stale = execute_sqlite_operation(&mut repository, stale);
    assert!(matches!(
        stale.outcome,
        OperationOutcome::Error { ref error }
            if error.code == OperationErrorCode::ExpectedHeadMismatch
                && error.head_conflict.is_some()
    ));
    assert!(
        repository
            .proposal(&ProposalId::from("stale-proposal"))
            .is_err()
    );
}

#[test]
fn successful_receipt_reserves_request_id_before_a_different_mutation() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("reserved-request.sqlite");
    let mut repository = seeded_repository(&path);
    let reserved = execute_sqlite_operation(
        &mut repository,
        OperationRequest {
            contract_version: OPERATION_CONTRACT_VERSION.into(),
            request_id: "reserved".into(),
            operation: Operation::Capabilities,
        },
    );
    assert!(matches!(reserved.outcome, OperationOutcome::Success { .. }));

    let collision = execute_sqlite_operation(&mut repository, propose("reserved"));
    assert!(matches!(
        collision.outcome,
        OperationOutcome::Error { ref error }
            if error.code == OperationErrorCode::InvalidRequest
    ));
    assert!(
        repository
            .proposal(&ProposalId::from("proposal-1"))
            .is_err()
    );
    assert_eq!(
        repository
            .conversation(&ConversationId::from("overall-framing"))
            .unwrap()
            .head_revision_id,
        RevisionId::from("fixture-root-revision")
    );
}

#[test]
fn sqlite_and_in_memory_inspection_have_contract_parity() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("parity.sqlite");
    let mut sqlite = seeded_repository(&path);
    let fixture = root_fixture();
    let mut memory = InMemoryRevisionRepository::create(
        fixture.project_id,
        fixture.conversation_id,
        "overall framing",
        fixture.root_revision_id,
        fixture.model,
    )
    .unwrap();
    let request = OperationRequest {
        contract_version: OPERATION_CONTRACT_VERSION.into(),
        request_id: "inspect-parity".into(),
        operation: Operation::Inspect {
            conversation_id: ConversationId::from("overall-framing"),
        },
    };
    let durable = execute_sqlite_operation(&mut sqlite, request.clone());
    let transient = execute_operation(&mut memory, request);
    assert_eq!(
        serde_json::to_value(durable).unwrap(),
        serde_json::to_value(transient).unwrap()
    );
}

#[test]
fn analysis_validation_and_evidence_inspection_survive_restart() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("analysis-operations.sqlite");
    let mut repository = seeded_repository(&path);
    let snapshot_id = ModelSnapshot::capture(root_fixture().model)
        .unwrap()
        .id()
        .clone();

    let validation = execute_sqlite_operation(
        &mut repository,
        OperationRequest {
            contract_version: OPERATION_CONTRACT_VERSION.into(),
            request_id: "validate-root".into(),
            operation: Operation::ValidateSnapshot {
                revision_id: RevisionId::from("fixture-root-revision"),
                expected_snapshot_id: snapshot_id.clone(),
            },
        },
    );
    assert!(matches!(
        validation.outcome,
        OperationOutcome::Success { ref result }
            if matches!(**result, OperationResult::SnapshotValidated { .. })
    ));

    let analysis_request = OperationRequest {
        contract_version: OPERATION_CONTRACT_VERSION.into(),
        request_id: "analyse-root".into(),
        operation: Operation::AnalyseSnapshot {
            revision_id: RevisionId::from("fixture-root-revision"),
            expected_snapshot_id: snapshot_id,
            evidence_id: EvidenceId::from("evidence-root"),
            settings: AnalysisSettings::frame3d(),
        },
    };
    let analysed = execute_sqlite_operation(&mut repository, analysis_request.clone());
    let OperationOutcome::Success { result } = &analysed.outcome else {
        panic!("unsupported analysis attempt must still publish evidence");
    };
    let OperationResult::SnapshotAnalysed { run } = &**result else {
        panic!("expected analysis result");
    };
    assert!(matches!(
        run.outcome,
        SnapshotAnalysisOutcome::Unsupported { .. }
    ));
    let manifest = run.evidence.analysis_manifest().unwrap();
    assert_eq!(manifest.status, AnalysisEvidenceStatus::Unsupported);
    assert_eq!(
        manifest.authored_snapshot_hash,
        run.evidence.authored_snapshot_id().as_str()
    );
    assert_eq!(
        manifest.solver_identity,
        settings_solver(&AnalysisSettings::frame3d())
    );
    assert_eq!(
        manifest.runtime_identity,
        settings_runtime(&AnalysisSettings::frame3d())
    );
    assert_eq!(
        manifest.settings_identity,
        AnalysisSettings::frame3d().identity().unwrap()
    );
    assert!(manifest.metrics.is_none());
    assert!(manifest.result_hash.is_none());
    assert!(manifest.attachments.is_empty());
    assert!(run.resolved_snapshot.is_none());
    let replay = execute_sqlite_operation(&mut repository, analysis_request);
    assert_eq!(
        serde_json::to_value(replay).unwrap(),
        serde_json::to_value(&analysed).unwrap()
    );

    drop(repository);
    let mut repository = SqliteRevisionRepository::open(&path).unwrap();
    let semantic_replay = execute_sqlite_operation(
        &mut repository,
        OperationRequest {
            contract_version: OPERATION_CONTRACT_VERSION.into(),
            request_id: "analyse-root-after-restart".into(),
            operation: Operation::AnalyseSnapshot {
                revision_id: RevisionId::from("fixture-root-revision"),
                expected_snapshot_id: run.evidence.authored_snapshot_id().clone(),
                evidence_id: EvidenceId::from("evidence-root"),
                settings: AnalysisSettings::frame3d(),
            },
        },
    );
    assert!(matches!(
        &semantic_replay.outcome,
        OperationOutcome::Success { result }
            if matches!(
                &**result,
                OperationResult::SnapshotAnalysed { run }
                    if matches!(&run.outcome, SnapshotAnalysisOutcome::Unsupported { .. })
            )
    ));
    let inspected = execute_sqlite_operation(
        &mut repository,
        OperationRequest {
            contract_version: OPERATION_CONTRACT_VERSION.into(),
            request_id: "inspect-evidence".into(),
            operation: Operation::InspectAnalysisEvidence {
                evidence_id: EvidenceId::from("evidence-root"),
                against_revision_id: RevisionId::from("fixture-root-revision"),
            },
        },
    );
    assert!(matches!(
        inspected.outcome,
        OperationOutcome::Success { result }
            if matches!(
                *result,
                OperationResult::AnalysisEvidenceInspection {
                    staleness: EvidenceStaleness::Current,
                    ..
                }
            )
    ));

    assert!(matches!(
        execute_sqlite_operation(&mut repository, propose("propose-descendant")).outcome,
        OperationOutcome::Success { .. }
    ));
    assert!(matches!(
        execute_sqlite_operation(
            &mut repository,
            OperationRequest {
                contract_version: OPERATION_CONTRACT_VERSION.into(),
                request_id: "accept-descendant".into(),
                operation: Operation::AcceptStructuralPatch {
                    proposal_id: ProposalId::from("proposal-1"),
                    conversation_id: ConversationId::from("overall-framing"),
                    expected_head_revision_id: RevisionId::from("fixture-root-revision"),
                },
            },
        )
        .outcome,
        OperationOutcome::Success { .. }
    ));
    let stale = execute_sqlite_operation(
        &mut repository,
        OperationRequest {
            contract_version: OPERATION_CONTRACT_VERSION.into(),
            request_id: "inspect-stale-evidence".into(),
            operation: Operation::InspectAnalysisEvidence {
                evidence_id: EvidenceId::from("evidence-root"),
                against_revision_id: RevisionId::from("revision-1"),
            },
        },
    );
    assert!(matches!(
        stale.outcome,
        OperationOutcome::Success { result }
            if matches!(
                *result,
                OperationResult::AnalysisEvidenceInspection {
                    staleness: EvidenceStaleness::Stale { .. },
                    ..
                }
            )
    ));
}

#[test]
fn snapshot_mismatch_and_reject_are_typed_and_leave_head_unchanged() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("reject.sqlite");
    let mut repository = seeded_repository(&path);
    let mismatch = execute_sqlite_operation(
        &mut repository,
        OperationRequest {
            contract_version: OPERATION_CONTRACT_VERSION.into(),
            request_id: "bad-snapshot".into(),
            operation: Operation::ValidateSnapshot {
                revision_id: RevisionId::from("fixture-root-revision"),
                expected_snapshot_id: fraia_revision::SnapshotId::from("sha256:stale"),
            },
        },
    );
    assert!(matches!(
        mismatch.outcome,
        OperationOutcome::Error { ref error }
            if error.code == OperationErrorCode::ExpectedSnapshotMismatch
                && error.snapshot_conflict.is_some()
    ));

    let proposed = execute_sqlite_operation(&mut repository, propose("propose-reject"));
    assert!(matches!(proposed.outcome, OperationOutcome::Success { .. }));
    let stale_reject = execute_sqlite_operation(
        &mut repository,
        OperationRequest {
            contract_version: OPERATION_CONTRACT_VERSION.into(),
            request_id: "reject-stale".into(),
            operation: Operation::RejectStructuralPatch {
                proposal_id: ProposalId::from("proposal-1"),
                conversation_id: ConversationId::from("overall-framing"),
                expected_head_revision_id: RevisionId::from("stale-head"),
            },
        },
    );
    assert!(matches!(
        stale_reject.outcome,
        OperationOutcome::Error { ref error }
            if error.code == OperationErrorCode::ExpectedHeadMismatch
    ));
    assert_eq!(
        repository
            .proposal(&ProposalId::from("proposal-1"))
            .unwrap()
            .status,
        "pending"
    );
    let reject = OperationRequest {
        contract_version: OPERATION_CONTRACT_VERSION.into(),
        request_id: "reject-1".into(),
        operation: Operation::RejectStructuralPatch {
            proposal_id: ProposalId::from("proposal-1"),
            conversation_id: ConversationId::from("overall-framing"),
            expected_head_revision_id: RevisionId::from("fixture-root-revision"),
        },
    };
    let rejected = execute_sqlite_operation(&mut repository, reject.clone());
    assert!(matches!(
        rejected.outcome,
        OperationOutcome::Success { ref result }
            if matches!(**result, OperationResult::StructuralPatchRejected { .. })
    ));
    assert_eq!(
        repository
            .conversation(&ConversationId::from("overall-framing"))
            .unwrap()
            .head_revision_id,
        RevisionId::from("fixture-root-revision")
    );
    let replay = execute_sqlite_operation(&mut repository, reject);
    assert_eq!(
        serde_json::to_value(replay).unwrap(),
        serde_json::to_value(rejected).unwrap()
    );
}

#[test]
fn failed_analysis_persists_diagnostics_without_fabricated_metrics() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("failed-analysis.sqlite");
    let mut model = fraia_core::StructuralModel::empty();
    model.dimension = "2d-in-3d".into();
    let (mut repository, snapshot_id) = seeded_model_repository(&path, model);
    let response = execute_sqlite_operation(
        &mut repository,
        OperationRequest {
            contract_version: OPERATION_CONTRACT_VERSION.into(),
            request_id: "analyse-failed".into(),
            operation: Operation::AnalyseSnapshot {
                revision_id: RevisionId::from("model-root"),
                expected_snapshot_id: snapshot_id,
                evidence_id: EvidenceId::from("failed-evidence"),
                settings: AnalysisSettings::frame2d(),
            },
        },
    );
    let OperationOutcome::Success { result } = response.outcome else {
        panic!("failed solver attempt must publish truthful evidence");
    };
    let OperationResult::SnapshotAnalysed { run } = *result else {
        panic!("expected analysis run");
    };
    assert!(matches!(
        run.outcome,
        SnapshotAnalysisOutcome::Failed { .. }
    ));
    let manifest = run.evidence.analysis_manifest().unwrap();
    assert_eq!(manifest.status, AnalysisEvidenceStatus::Failed);
    assert!(manifest.result_hash.is_none());
    assert!(manifest.metrics.is_none());
    assert!(manifest.attachments.is_empty());
    assert!(!manifest.diagnostics.is_empty());
}

#[test]
fn completed_analysis_persists_resolved_and_result_identities() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("completed-analysis.sqlite");
    let (mut repository, snapshot_id) = seeded_model_repository(&path, analysed_model());
    let response = execute_sqlite_operation(
        &mut repository,
        OperationRequest {
            contract_version: OPERATION_CONTRACT_VERSION.into(),
            request_id: "analyse-completed".into(),
            operation: Operation::AnalyseSnapshot {
                revision_id: RevisionId::from("model-root"),
                expected_snapshot_id: snapshot_id,
                evidence_id: EvidenceId::from("completed-evidence"),
                settings: AnalysisSettings::frame2d(),
            },
        },
    );
    let OperationOutcome::Success { result } = response.outcome else {
        panic!("analysis operation must publish an immutable outcome");
    };
    let OperationResult::SnapshotAnalysed { run } = *result else {
        panic!("expected analysis run");
    };
    assert!(matches!(
        run.outcome,
        SnapshotAnalysisOutcome::Completed { .. }
    ));
    let manifest = run.evidence.analysis_manifest().unwrap();
    assert_eq!(manifest.status, AnalysisEvidenceStatus::Completed);
    assert!(manifest.resolved_snapshot_hash.is_some());
    assert!(manifest.input_hash.is_some());
    assert!(manifest.result_hash.is_some());
    assert!(manifest.metrics.is_some());
    assert!(run.resolved_snapshot.is_some());
}

#[test]
fn in_memory_and_sqlite_analysis_have_typed_contract_parity() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("analysis-parity.sqlite");
    let mut sqlite = seeded_repository(&path);
    let fixture = root_fixture();
    let snapshot_id = ModelSnapshot::capture(fixture.model.clone())
        .unwrap()
        .id()
        .clone();
    let mut memory = InMemoryRevisionRepository::create(
        fixture.project_id,
        fixture.conversation_id,
        "overall framing",
        fixture.root_revision_id,
        fixture.model,
    )
    .unwrap();
    let request = OperationRequest {
        contract_version: OPERATION_CONTRACT_VERSION.into(),
        request_id: "analysis-parity".into(),
        operation: Operation::AnalyseSnapshot {
            revision_id: RevisionId::from("fixture-root-revision"),
            expected_snapshot_id: snapshot_id,
            evidence_id: EvidenceId::from("parity-evidence"),
            settings: AnalysisSettings::frame3d(),
        },
    };
    let durable = execute_sqlite_operation(&mut sqlite, request.clone());
    let transient = execute_operation(&mut memory, request);
    assert_eq!(
        serde_json::to_value(durable).unwrap(),
        serde_json::to_value(transient).unwrap()
    );
}

fn settings_solver(settings: &AnalysisSettings) -> String {
    settings.solver_identity().into()
}

fn settings_runtime(settings: &AnalysisSettings) -> String {
    settings.runtime_identity().into()
}
