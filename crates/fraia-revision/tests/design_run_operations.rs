use fraia_core::{
    AssignmentTargetRef, DesignRunActor, DesignRunStaleness, LoadAssignment, LoadKind, LoadVector,
    StructuralMember, StructuralModel, StructuralNode, SupportAssignment, design_package_paths,
    inspect_design_run, list_design_run_statuses, list_design_runs,
};
use fraia_revision::analysis_service::{AnalysisSettings, SnapshotAnalysisOutcome};
use fraia_revision::operations::{
    DesignRunOperationContext, OPERATION_CONTRACT_VERSION, Operation, OperationOutcome,
    OperationRequest, OperationResult, execute_sqlite_operation_with_design_runs,
};
use fraia_revision::snapshot::ModelSnapshot;
use fraia_revision::sqlite::{
    SqliteRevisionRepository, StoredConversation, StoredProjectRoot, StoredRevision, StoredSnapshot,
};
use fraia_revision::{ConversationId, EvidenceId, ProjectId, RevisionId};
use tempfile::tempdir;

struct Fixture {
    _temporary: tempfile::TempDir,
    project_dir: std::path::PathBuf,
    project_id: fraia_core::ProjectId,
    design_id: fraia_core::DesignId,
    database: std::path::PathBuf,
    snapshot_id: fraia_revision::SnapshotId,
}

#[test]
fn analysis_evidence_and_canonical_run_share_exact_interpretation_dependencies() {
    let fixture = Fixture::new_with_metadata(
        analysed_model(),
        "Interpretation-bound run",
        serde_json::json!({
            "operation": "accepted",
            "agentProvenance": {
                "provider": "fake",
                "model": "gpt-5.6-luna",
                "turnId": "turn-1",
                "drawingInterpretationRevisionIds": [
                    "drawing-interpretation-sha256-a"
                ],
                "drawingInterpretationInferenceIds": [
                    "drawing-interpretation-sha256-a:inference:grid-a"
                ]
            }
        })
        .to_string(),
    );
    let mut repository = SqliteRevisionRepository::open(&fixture.database).unwrap();
    let context = fixture.context("2026-08-14T06:00:00Z");
    let expected_dependencies = fraia_core::DesignRunInterpretationDependencies {
        revision_ids: vec!["drawing-interpretation-sha256-a".into()],
        inference_ids: vec!["drawing-interpretation-sha256-a:inference:grid-a".into()],
    };
    let response = execute_sqlite_operation_with_design_runs(
        &mut repository,
        OperationRequest {
            contract_version: OPERATION_CONTRACT_VERSION.into(),
            request_id: "analyse-interpretation-bound".into(),
            operation: Operation::AnalyseSnapshot {
                revision_id: RevisionId::from("accepted-r1"),
                expected_snapshot_id: fixture.snapshot_id.clone(),
                evidence_id: EvidenceId::from("evidence-interpretation-bound"),
                settings: AnalysisSettings::frame2d(),
            },
        },
        &context,
    );
    let run = returned_run(&response);
    assert_eq!(
        run.evidence.interpretation_dependencies(),
        expected_dependencies
    );
    let run_id = run.canonical_run_id.as_deref().unwrap();
    let fraia_core::InspectedDesignRun::Canonical { manifest } =
        inspect_design_run(&fixture.project_dir, &fixture.design_id, run_id).unwrap()
    else {
        panic!("expected canonical run");
    };
    assert_eq!(manifest.interpretation_dependencies, expected_dependencies);
    assert_eq!(
        list_design_runs(&fixture.project_dir, &fixture.design_id)
            .unwrap()
            .runs[0]
            .interpretation_dependencies,
        expected_dependencies
    );
}

impl Fixture {
    fn new(model: StructuralModel, label: &str) -> Self {
        Self::new_with_metadata(model, label, "{\"operation\":\"accepted\"}".into())
    }

    fn new_with_metadata(model: StructuralModel, label: &str, metadata_json: String) -> Self {
        let temporary = tempdir().unwrap();
        let project_dir = temporary.path().join("project");
        let package = fraia_core::create_named_project_package(&project_dir, label).unwrap();
        let project_id = package.manifest.id;
        let design_id = package.designs[0].manifest.id.clone();
        let database = design_package_paths(&project_dir, &design_id)
            .unwrap()
            .workspace_database;
        let snapshot = ModelSnapshot::capture(model).unwrap();
        let snapshot_id = snapshot.id().clone();
        let mut repository = SqliteRevisionRepository::open(&database).unwrap();
        repository
            .create_project(StoredProjectRoot {
                project_id: ProjectId::new(design_id.as_str()),
                root_conversation: StoredConversation {
                    id: ConversationId::from("overall"),
                    project_id: ProjectId::new(design_id.as_str()),
                    purpose: "accepted design".into(),
                    origin_json: "{\"kind\":\"root\"}".into(),
                    head_revision_id: RevisionId::from("accepted-r1"),
                },
                root_revision: StoredRevision {
                    id: RevisionId::from("accepted-r1"),
                    snapshot_id: snapshot_id.clone(),
                    parent_revision_id: None,
                    conversation_id: ConversationId::from("overall"),
                    metadata_json,
                },
                root_snapshot: StoredSnapshot {
                    id: snapshot_id.clone(),
                    format_version: snapshot.canonical_format_version().as_str().into(),
                    canonical_bytes: snapshot.canonical_bytes().to_vec(),
                },
            })
            .unwrap();
        drop(repository);
        Self {
            _temporary: temporary,
            project_dir,
            project_id,
            design_id,
            database,
            snapshot_id,
        }
    }

    fn context(&self, attempted_at: &str) -> DesignRunOperationContext {
        DesignRunOperationContext::new(
            &self.project_dir,
            self.project_id.clone(),
            self.design_id.clone(),
            DesignRunActor {
                actor_type: "test".into(),
                actor_id: "operations.v1".into(),
            },
            attempted_at,
        )
    }

    fn analyse(
        &self,
        evidence_id: &str,
        request_id: &str,
        settings: AnalysisSettings,
        attempted_at: &str,
    ) -> fraia_revision::operations::OperationResponse {
        let mut repository = SqliteRevisionRepository::open(&self.database).unwrap();
        execute_sqlite_operation_with_design_runs(
            &mut repository,
            OperationRequest {
                contract_version: OPERATION_CONTRACT_VERSION.into(),
                request_id: request_id.into(),
                operation: Operation::AnalyseSnapshot {
                    revision_id: RevisionId::from("accepted-r1"),
                    expected_snapshot_id: self.snapshot_id.clone(),
                    evidence_id: EvidenceId::from(evidence_id),
                    settings,
                },
            },
            &self.context(attempted_at),
        )
    }
}

fn returned_run(
    response: &fraia_revision::operations::OperationResponse,
) -> &fraia_revision::analysis_service::SnapshotAnalysisRun {
    let OperationOutcome::Success { result } = &response.outcome else {
        panic!("analysis operation failed: {response:?}");
    };
    let OperationResult::SnapshotAnalysed { run } = &**result else {
        panic!("expected analysed snapshot");
    };
    run
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

#[test]
fn accepted_analysis_publishes_bound_completed_and_unsupported_runs_with_restart_move_and_rerun() {
    let fixture = Fixture::new(analysed_model(), "Run operations");
    let completed = fixture.analyse(
        "evidence-completed-1",
        "analyse-completed-1",
        AnalysisSettings::frame2d(),
        "2026-08-13T06:00:00Z",
    );
    let first = returned_run(&completed);
    assert!(matches!(
        first.outcome,
        SnapshotAnalysisOutcome::Completed { .. }
    ));
    let first_id = first.canonical_run_id.as_deref().unwrap();
    assert_eq!(first.evidence.canonical_run_id(), Some(first_id));
    assert!(first.metrics().is_some());

    let replay = fixture.analyse(
        "evidence-completed-1",
        "analyse-completed-1",
        AnalysisSettings::frame2d(),
        "a later adapter timestamp is ignored by the stored receipt",
    );
    assert_eq!(
        serde_json::to_value(&replay).unwrap(),
        serde_json::to_value(&completed).unwrap()
    );

    let unsupported = fixture.analyse(
        "evidence-unsupported",
        "analyse-unsupported",
        AnalysisSettings::frame3d(),
        "2026-08-13T06:01:00Z",
    );
    let unsupported = returned_run(&unsupported);
    assert!(matches!(
        unsupported.outcome,
        SnapshotAnalysisOutcome::Unsupported { .. }
    ));
    assert!(unsupported.metrics().is_none());
    assert!(unsupported.evidence.result_identity().is_none());

    let rerun = fixture.analyse(
        "evidence-completed-2",
        "analyse-completed-2",
        AnalysisSettings::frame2d(),
        "2026-08-13T06:02:00Z",
    );
    let rerun_id = returned_run(&rerun).canonical_run_id.as_deref().unwrap();
    assert_ne!(first_id, rerun_id);
    assert_eq!(
        list_design_runs(&fixture.project_dir, &fixture.design_id)
            .unwrap()
            .runs
            .len(),
        3
    );

    let current = list_design_run_statuses(
        &fixture.project_dir,
        &fixture.design_id,
        fixture.snapshot_id.as_str(),
        &[],
    )
    .unwrap();
    assert!(
        current
            .iter()
            .all(|run| run.staleness == DesignRunStaleness::Current)
    );
    let stale = list_design_run_statuses(
        &fixture.project_dir,
        &fixture.design_id,
        "sha256:new-descendant",
        &[fixture.snapshot_id.to_string()],
    )
    .unwrap();
    assert!(
        stale
            .iter()
            .all(|run| run.staleness == DesignRunStaleness::StaleDescendant)
    );

    let moved = fixture._temporary.path().join("moved-project");
    std::fs::rename(&fixture.project_dir, &moved).unwrap();
    let moved_database = design_package_paths(&moved, &fixture.design_id)
        .unwrap()
        .workspace_database;
    let repository = SqliteRevisionRepository::open(moved_database).unwrap();
    assert_eq!(
        repository
            .evidence(&EvidenceId::from("evidence-completed-1"))
            .unwrap()
            .blob_ref,
        None
    );
    assert_eq!(
        list_design_runs(&moved, &fixture.design_id)
            .unwrap()
            .runs
            .len(),
        3
    );
}

#[test]
fn failed_analysis_publishes_diagnostics_without_result_metrics() {
    let mut model = StructuralModel::empty();
    model.dimension = "2d-in-3d".into();
    let fixture = Fixture::new(model, "Failed run");
    let response = fixture.analyse(
        "evidence-failed",
        "analyse-failed",
        AnalysisSettings::frame2d(),
        "2026-08-13T06:03:00Z",
    );
    let run = returned_run(&response);
    assert!(matches!(
        run.outcome,
        SnapshotAnalysisOutcome::Failed { .. }
    ));
    assert!(run.canonical_run_id.is_some());
    assert_eq!(
        run.evidence.canonical_run_id(),
        run.canonical_run_id.as_deref()
    );
    assert!(run.metrics().is_none());
    assert!(run.evidence.result_identity().is_none());
    assert!(!run.outcome.diagnostics().is_empty());
}
