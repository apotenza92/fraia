use fraia_core::{
    AssignmentTargetRef, LoadAssignment, LoadKind, LoadVector, StructuralMember, StructuralModel,
    StructuralNode, SupportAssignment,
};
use fraia_revision::analysis_service::{
    SnapshotAnalysisOutcome, analyse_accepted_revision, dependencies_for_snapshot,
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
