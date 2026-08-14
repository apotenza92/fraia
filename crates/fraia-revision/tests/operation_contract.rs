use fraia_revision::analysis_service::AnalysisSettings;
use fraia_revision::operations::{
    OPERATION_CONTRACT_VERSION, Operation, OperationErrorCode, OperationOutcome, OperationRequest,
    OperationResult, execute_operation,
};
use fraia_revision::patch::{Length, Position, StructuralOperation, StructuralPatch};
use fraia_revision::repository::{InMemoryRevisionRepository, ProposalId, ProposalStatus};
use fraia_revision::{ConversationId, RevisionId, root_fixture};
use fraia_revision::{EvidenceId, SnapshotId};
use serde_json::json;

fn repository() -> InMemoryRevisionRepository {
    let fixture = root_fixture();
    InMemoryRevisionRepository::create(
        fixture.project_id,
        fixture.conversation_id,
        "overall framing",
        fixture.root_revision_id,
        fixture.model,
    )
    .unwrap()
}

fn move_left_eave_patch() -> StructuralPatch {
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

#[test]
fn inspect_request_and_response_have_stable_versioned_json() {
    let request_json = json!({
        "contractVersion": "fraia.operations.v1",
        "requestId": "inspect-1",
        "operation": "inspect",
        "parameters": { "conversation_id": "overall-framing" }
    });
    let request: OperationRequest = serde_json::from_value(request_json.clone()).unwrap();
    assert_eq!(serde_json::to_value(&request).unwrap(), request_json);

    let mut repository = repository();
    let response = execute_operation(&mut repository, request);
    let response_json = serde_json::to_value(&response).unwrap();
    assert_eq!(response_json["contractVersion"], OPERATION_CONTRACT_VERSION);
    assert_eq!(response_json["requestId"], "inspect-1");
    assert_eq!(response_json["status"], "success");
    assert_eq!(response_json["result"]["type"], "inspection");
    assert_eq!(
        response_json["result"]["conversation"]["head_revision_id"],
        "fixture-root-revision"
    );
    assert_eq!(
        response_json["result"]["authored_model"]["nodes"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
}

#[test]
fn structural_patch_proposal_request_has_stable_json() {
    let request = OperationRequest {
        contract_version: OPERATION_CONTRACT_VERSION.into(),
        request_id: "propose-json-1".into(),
        operation: Operation::ProposeStructuralPatch {
            proposal_id: ProposalId::from("proposal-1"),
            conversation_id: ConversationId::from("overall-framing"),
            expected_head_revision_id: RevisionId::from("fixture-root-revision"),
            proposed_revision_id: RevisionId::from("revision-1"),
            patch: move_left_eave_patch(),
            agent_provenance: None,
        },
    };
    let expected = json!({
        "contractVersion": "fraia.operations.v1",
        "requestId": "propose-json-1",
        "operation": "propose_structural_patch",
        "parameters": {
            "proposal_id": "proposal-1",
            "conversation_id": "overall-framing",
            "expected_head_revision_id": "fixture-root-revision",
            "proposed_revision_id": "revision-1",
            "patch": {
                "operations": [{
                    "MoveNode": {
                        "node_id": "left-eave",
                        "position": {
                            "x": { "value": 0.0, "unit": "Meters" },
                            "y": { "value": 6.5, "unit": "Meters" },
                            "z": { "value": 0.0, "unit": "Meters" }
                        }
                    }
                }]
            }
        }
    });
    assert_eq!(serde_json::to_value(&request).unwrap(), expected);
    assert_eq!(
        serde_json::to_value(serde_json::from_value::<OperationRequest>(expected).unwrap())
            .unwrap(),
        serde_json::to_value(request).unwrap()
    );
}

#[test]
fn proposal_is_validated_without_moving_the_head_then_accepts_once() {
    let mut repository = repository();
    let proposal_id = ProposalId::from("proposal-1");
    let proposed_revision_id = RevisionId::from("revision-1");
    let proposed = execute_operation(
        &mut repository,
        OperationRequest {
            contract_version: OPERATION_CONTRACT_VERSION.into(),
            request_id: "propose-1".into(),
            operation: Operation::ProposeStructuralPatch {
                proposal_id: proposal_id.clone(),
                conversation_id: ConversationId::from("overall-framing"),
                expected_head_revision_id: RevisionId::from("fixture-root-revision"),
                proposed_revision_id: proposed_revision_id.clone(),
                patch: move_left_eave_patch(),
                agent_provenance: None,
            },
        },
    );
    assert!(matches!(
        proposed.outcome,
        OperationOutcome::Success { result }
            if matches!(*result, OperationResult::StructuralPatchProposed { .. })
    ));
    assert_eq!(
        repository
            .head(&ConversationId::from("overall-framing"))
            .unwrap()
            .head_revision_id,
        RevisionId::from("fixture-root-revision")
    );
    assert_eq!(
        repository.proposal(&proposal_id).unwrap().status(),
        &ProposalStatus::Pending
    );

    let accepted = execute_operation(
        &mut repository,
        OperationRequest {
            contract_version: OPERATION_CONTRACT_VERSION.into(),
            request_id: "accept-1".into(),
            operation: Operation::AcceptStructuralPatch {
                proposal_id: proposal_id.clone(),
                conversation_id: ConversationId::from("overall-framing"),
                expected_head_revision_id: RevisionId::from("fixture-root-revision"),
            },
        },
    );
    assert!(matches!(
        accepted.outcome,
        OperationOutcome::Success { result }
            if matches!(
                *result,
                OperationResult::StructuralPatchAccepted { ref revision_id, .. }
                    if revision_id == &proposed_revision_id
            )
    ));
    assert_eq!(repository.revision_count(), 2);
    assert_eq!(
        repository
            .head(&ConversationId::from("overall-framing"))
            .unwrap()
            .head_revision_id,
        proposed_revision_id
    );
}

#[test]
fn stale_expected_head_returns_exact_conflict_without_pending_state() {
    let mut repository = repository();
    let response = execute_operation(
        &mut repository,
        OperationRequest {
            contract_version: OPERATION_CONTRACT_VERSION.into(),
            request_id: "stale-1".into(),
            operation: Operation::ProposeStructuralPatch {
                proposal_id: ProposalId::from("stale-proposal"),
                conversation_id: ConversationId::from("overall-framing"),
                expected_head_revision_id: RevisionId::from("stale-revision"),
                proposed_revision_id: RevisionId::from("never-created"),
                patch: move_left_eave_patch(),
                agent_provenance: None,
            },
        },
    );
    assert_eq!(
        serde_json::to_value(&response).unwrap(),
        json!({
            "contractVersion": "fraia.operations.v1",
            "requestId": "stale-1",
            "status": "error",
            "error": {
                "code": "expected_head_mismatch",
                "message": "conversation `overall-framing` head is `fixture-root-revision`, not expected `stale-revision`",
                "headConflict": {
                    "conversationId": "overall-framing",
                    "expectedRevisionId": "stale-revision",
                    "actualRevisionId": "fixture-root-revision"
                }
            }
        })
    );
    let OperationOutcome::Error { error } = response.outcome else {
        panic!("stale request must fail");
    };
    assert_eq!(error.code, OperationErrorCode::ExpectedHeadMismatch);
    let conflict = error.head_conflict.unwrap();
    assert_eq!(
        conflict.expected_revision_id,
        RevisionId::from("stale-revision")
    );
    assert_eq!(
        conflict.actual_revision_id,
        RevisionId::from("fixture-root-revision")
    );
    assert!(
        repository
            .proposal(&ProposalId::from("stale-proposal"))
            .is_err()
    );
    assert_eq!(repository.revision_count(), 1);
}

#[test]
fn invalid_patch_and_unknown_version_fail_without_mutation() {
    let mut repository = repository();
    let invalid = execute_operation(
        &mut repository,
        OperationRequest {
            contract_version: OPERATION_CONTRACT_VERSION.into(),
            request_id: "invalid-patch".into(),
            operation: Operation::ProposeStructuralPatch {
                proposal_id: ProposalId::from("empty"),
                conversation_id: ConversationId::from("overall-framing"),
                expected_head_revision_id: RevisionId::from("fixture-root-revision"),
                proposed_revision_id: RevisionId::from("never-created"),
                patch: StructuralPatch::default(),
                agent_provenance: None,
            },
        },
    );
    assert!(matches!(
        invalid.outcome,
        OperationOutcome::Error { ref error }
            if error.code == OperationErrorCode::InvalidPatch
    ));

    let unsupported = execute_operation(
        &mut repository,
        OperationRequest {
            contract_version: "fraia.operations.v999".into(),
            request_id: "future-version".into(),
            operation: Operation::Inspect {
                conversation_id: ConversationId::from("overall-framing"),
            },
        },
    );
    assert_eq!(unsupported.contract_version, OPERATION_CONTRACT_VERSION);
    assert!(matches!(
        unsupported.outcome,
        OperationOutcome::Error { ref error }
            if error.code == OperationErrorCode::UnsupportedContractVersion
    ));
    assert_eq!(repository.revision_count(), 1);
    assert!(repository.proposal(&ProposalId::from("empty")).is_err());
}

#[test]
fn validation_analysis_rejection_and_evidence_requests_have_stable_json_tags() {
    let requests = [
        OperationRequest {
            contract_version: OPERATION_CONTRACT_VERSION.into(),
            request_id: "validate".into(),
            operation: Operation::ValidateSnapshot {
                revision_id: RevisionId::from("r1"),
                expected_snapshot_id: SnapshotId::from("sha256:one"),
            },
        },
        OperationRequest {
            contract_version: OPERATION_CONTRACT_VERSION.into(),
            request_id: "analyse".into(),
            operation: Operation::AnalyseSnapshot {
                revision_id: RevisionId::from("r1"),
                expected_snapshot_id: SnapshotId::from("sha256:one"),
                evidence_id: EvidenceId::from("run-1"),
                settings: AnalysisSettings::frame3d(),
            },
        },
        OperationRequest {
            contract_version: OPERATION_CONTRACT_VERSION.into(),
            request_id: "reject".into(),
            operation: Operation::RejectStructuralPatch {
                proposal_id: ProposalId::from("p1"),
                conversation_id: ConversationId::from("overall"),
                expected_head_revision_id: RevisionId::from("r1"),
            },
        },
        OperationRequest {
            contract_version: OPERATION_CONTRACT_VERSION.into(),
            request_id: "inspect-run".into(),
            operation: Operation::InspectAnalysisEvidence {
                evidence_id: EvidenceId::from("run-1"),
                against_revision_id: RevisionId::from("r2"),
            },
        },
    ];
    let tags = [
        "validate_snapshot",
        "analyse_snapshot",
        "reject_structural_patch",
        "inspect_analysis_evidence",
    ];
    for (request, tag) in requests.into_iter().zip(tags) {
        let value = serde_json::to_value(&request).unwrap();
        assert_eq!(value["contractVersion"], "fraia.operations.v1");
        assert_eq!(value["operation"], tag);
        assert_eq!(
            serde_json::to_value(
                serde_json::from_value::<OperationRequest>(value.clone()).unwrap()
            )
            .unwrap(),
            value
        );
    }
}
