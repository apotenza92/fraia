use fraia_core::{
    AssignmentTargetRef, LoadAssignment, LoadKind, LoadVector, StructuralMember, StructuralModel,
    StructuralNode, SupportAssignment,
};
use fraia_revision::analysis_service::{
    AnalysisExecutionStage, AnalysisSettings, SnapshotAnalysisError, SnapshotAnalysisOutcome,
    analyse_accepted_revision, analyse_accepted_revision_with_control, dependencies_for_snapshot,
};
use fraia_revision::patch::{
    LineLoadUnit, LoadInput, LoadMagnitude, StructuralOperation, StructuralPatch,
};
use fraia_revision::repository::{InMemoryRevisionRepository, ProposalId};
use fraia_revision::{ConversationId, EvidenceId, ProjectId, RevisionId};

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

fn repository() -> InMemoryRevisionRepository {
    InMemoryRevisionRepository::create(
        ProjectId::from("analysis-project"),
        ConversationId::from("overall-framing"),
        "overall framing",
        RevisionId::from("root"),
        analysed_model(),
    )
    .unwrap()
}

#[test]
fn same_snapshot_produces_the_same_exact_analysis_hashes() {
    let mut first_repository = repository();
    let first = analyse_accepted_revision(
        &mut first_repository,
        &RevisionId::from("root"),
        EvidenceId::from("run-a"),
    )
    .unwrap();
    let mut second_repository = repository();
    let second = analyse_accepted_revision(
        &mut second_repository,
        &RevisionId::from("root"),
        EvidenceId::from("run-b"),
    )
    .unwrap();
    let first_manifest = first.evidence.analysis_manifest().unwrap();
    let second_manifest = second.evidence.analysis_manifest().unwrap();

    assert!(
        first.outcome.completed(),
        "fixture must solve: {:?}",
        first.outcome
    );
    assert!(
        second.outcome.completed(),
        "fixture must solve: {:?}",
        second.outcome
    );
    assert_eq!(
        first_manifest.authored_snapshot_hash,
        second_manifest.authored_snapshot_hash
    );
    assert_eq!(
        first_manifest.resolved_snapshot_hash,
        second_manifest.resolved_snapshot_hash
    );
    assert_eq!(first_manifest.input_hash, second_manifest.input_hash);
    assert_eq!(first_manifest.result_hash, second_manifest.result_hash);
}

#[test]
fn completed_analysis_publishes_through_the_canonical_design_run_store() {
    let directory = tempfile::tempdir().unwrap();
    let project_dir = directory.path().join("project");
    let package = fraia_core::create_named_project_package(&project_dir, "Analysis run").unwrap();
    let design_id = package.designs[0].manifest.id.clone();
    let mut repository = repository();
    let revision_id = RevisionId::from("root");
    let mut analysis = analyse_accepted_revision(
        &mut repository,
        &revision_id,
        EvidenceId::from("canonical-run"),
    )
    .unwrap();
    let published = fraia_revision::design_run_adapter::publish_analysis_evidence_design_run(
        fraia_revision::design_run_adapter::PublishAnalysisEvidenceDesignRun {
            project_dir: &project_dir,
            project_id: package.manifest.id,
            design_id: design_id.clone(),
            revision_id: &revision_id,
            evidence: &mut analysis.evidence,
            actor: fraia_core::DesignRunActor {
                actor_type: "test".into(),
                actor_id: "analysis-service".into(),
            },
            created_at: "2026-08-13T04:02:00Z".into(),
            parent_run_id: None,
        },
    )
    .unwrap();
    assert_eq!(published.status, fraia_core::DesignRunStatus::Completed);
    assert_eq!(published.authored_revision_id, "root");
    assert!(published.result_identity.is_some());
    assert_eq!(
        analysis.evidence.canonical_run_id(),
        Some(published.run_id.as_str())
    );
    assert_eq!(
        fraia_core::list_design_runs(&project_dir, &design_id)
            .unwrap()
            .runs[0]
            .run_id,
        published.run_id
    );
}

#[test]
fn descendant_edit_marks_exact_snapshot_evidence_stale() {
    let mut repository = repository();
    let root = RevisionId::from("root");
    let run =
        analyse_accepted_revision(&mut repository, &root, EvidenceId::from("root-run")).unwrap();
    assert!(matches!(
        run.outcome,
        SnapshotAnalysisOutcome::Completed { .. }
    ));
    repository
        .create_proposal(
            ProposalId::from("more-load"),
            ConversationId::from("overall-framing"),
            root.clone(),
            RevisionId::from("child"),
            StructuralPatch {
                operations: vec![StructuralOperation::UpdateLoad(LoadInput {
                    id: "gravity".into(),
                    target: AssignmentTargetRef::Member("beam".into()),
                    load_case_id: "dead".into(),
                    direction: LoadVector {
                        x: 0.0,
                        y: -1.0,
                        z: 0.0,
                    },
                    magnitude: LoadMagnitude::LineLoad {
                        value: 12.0,
                        unit: LineLoadUnit::KiloNewtonsPerMeter,
                    },
                })],
            },
        )
        .unwrap();
    repository
        .accept_proposal(&ProposalId::from("more-load"))
        .unwrap();
    let child = repository.revision(&RevisionId::from("child")).unwrap();
    let stale = repository
        .evidence_staleness(
            &EvidenceId::from("root-run"),
            &RevisionId::from("child"),
            &dependencies_for_snapshot(child.snapshot_id()),
        )
        .unwrap();
    assert!(
        stale.is_stale(),
        "descendant load change must stale root evidence"
    );
}

#[test]
fn controlled_analysis_reports_truthful_stages_and_cancel_publishes_no_evidence() {
    let mut repository = repository();
    let revision_id = RevisionId::from("root");
    let evidence_id = EvidenceId::from("cancelled-run");
    let mut stages = Vec::new();
    let solving = std::cell::Cell::new(false);
    let result = analyse_accepted_revision_with_control(
        &mut repository,
        &revision_id,
        evidence_id.clone(),
        AnalysisSettings::frame2d(),
        |stage| {
            stages.push(stage);
            if stage == AnalysisExecutionStage::Solving {
                solving.set(true);
            }
        },
        || solving.get(),
    );
    assert!(matches!(result, Err(SnapshotAnalysisError::Cancelled)));
    assert_eq!(
        stages,
        vec![
            AnalysisExecutionStage::Preparing,
            AnalysisExecutionStage::Resolving,
            AnalysisExecutionStage::Solving,
        ]
    );
    assert!(repository.evidence(&evidence_id).is_err());
}
