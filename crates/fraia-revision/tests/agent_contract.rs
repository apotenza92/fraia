use fraia_core::{AssignmentTargetRef, LoadVector, StructuralMember};
use fraia_revision::agent_contract::{
    AgentContextInput, AgentContextProvenance, AgentTurnProvenance, AllowedAgentOperation,
    ContextValidationError, EvidenceReference, KnowledgeGuidance, ProjectFact,
    ProposalValidationError, TypedAgentContext, UntrustedAgentOperation, UntrustedAgentProposal,
    build_context, validate_agent_context, validate_agent_proposal,
};
use fraia_revision::conversation::ConversationHead;
use fraia_revision::patch::{LineLoadUnit, LoadInput, LoadMagnitude, MemberRole, PatchError};
use fraia_revision::repository::{InMemoryRevisionRepository, RepositoryError};
use fraia_revision::{ConversationId, EvidenceId, ProjectId, RevisionId, SnapshotId, root_fixture};

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

fn provenance() -> AgentTurnProvenance {
    AgentTurnProvenance {
        provider: "test-provider".into(),
        model: "test-model".into(),
        turn_id: "turn-7".into(),
    }
}

fn proposal(operations: Vec<UntrustedAgentOperation>) -> UntrustedAgentProposal {
    UntrustedAgentProposal {
        proposal_id: "agent-proposal".into(),
        proposed_revision_id: RevisionId::from("agent-revision"),
        conversation_id: ConversationId::from("overall-framing"),
        parent_revision_id: RevisionId::from("fixture-root-revision"),
        provenance: provenance(),
        operations,
    }
}

#[test]
fn raw_json_alone_is_rejected_as_agent_context() {
    assert!(matches!(
        validate_agent_context(AgentContextInput::RawStructuralJsonOnly {
            payload: "{}".into()
        }),
        Err(ContextValidationError::RawStructuralJsonInsufficient)
    ));
}

#[test]
fn typed_context_has_required_sections_and_knowledge_is_guidance_only() {
    let repository = repository();
    let context = build_context(
        &repository,
        &ConversationId::from("overall-framing"),
        vec![ProjectFact {
            key: "clear_span_m".into(),
            value: "20".into(),
            confirmed: true,
        }],
        vec![],
        vec![KnowledgeGuidance {
            reference_id: "portal-frame".into(),
            title: "Portal frame guidance".into(),
            excerpt: "Use as guidance only.".into(),
        }],
        vec![AllowedAgentOperation::AddLoad],
    )
    .unwrap();

    assert_eq!(context.project_id, ProjectId::from("fixture-project"));
    assert_eq!(
        context.current_revision_id,
        RevisionId::from("fixture-root-revision")
    );
    assert_eq!(context.semantic_model.counts.members, 3);
    assert_eq!(context.knowledge[0].excerpt, "Use as guidance only.");
    assert_eq!(KnowledgeGuidance::AUTHORITY, "guidance_only");
    assert!(context.evidence.is_empty());
}

#[test]
fn context_rejects_evidence_not_bound_to_the_head_snapshot() {
    let context = TypedAgentContext {
        project_id: ProjectId::from("project"),
        project_facts: vec![],
        unresolved_assumptions: vec![],
        conversation_summary: "lateral stability".into(),
        conversation: ConversationHead {
            conversation_id: ConversationId::from("conversation"),
            purpose: "lateral stability".into(),
            head_revision_id: RevisionId::from("revision"),
            head_snapshot_id: SnapshotId::from("snapshot-a"),
        },
        current_revision_id: RevisionId::from("revision"),
        provenance: AgentContextProvenance {
            source: "test".into(),
            snapshot_id: SnapshotId::from("snapshot-a"),
        },
        semantic_model: fraia_core::understand_structural_model(&root_fixture().model),
        evidence: vec![EvidenceReference {
            evidence_id: EvidenceId::from("run"),
            authored_snapshot_id: SnapshotId::from("snapshot-b"),
            provenance: "solver".into(),
        }],
        knowledge: vec![],
        allowed_operations: vec![AllowedAgentOperation::AddLoad],
    };
    assert!(matches!(
        validate_agent_context(AgentContextInput::Typed(context)),
        Err(ContextValidationError::EvidenceSnapshotDoesNotMatchHead { evidence_id })
            if evidence_id == EvidenceId::from("run")
    ));
}

#[test]
fn unknown_or_unsafe_operations_are_rejected_before_repository_mutation() {
    let repository = repository();
    let unknown = validate_agent_proposal(
        proposal(vec![UntrustedAgentOperation::Unknown {
            kind: "delete_everything".into(),
        }]),
        &[AllowedAgentOperation::AddLoad],
    );
    assert!(matches!(
        unknown,
        Err(ProposalValidationError::UnknownOperation { kind }) if kind == "delete_everything"
    ));
    let unsafe_json = validate_agent_proposal(
        proposal(vec![UntrustedAgentOperation::UnsafeRawStructuralJson {
            payload: "{}".into(),
        }]),
        &[AllowedAgentOperation::AddLoad],
    );
    assert!(matches!(
        unsafe_json,
        Err(ProposalValidationError::RawStructuralJsonForbidden)
    ));
    assert_eq!(repository.revision_count(), 1);
    assert!(repository.proposal(&"agent-proposal".into()).is_err());
}

#[test]
fn validated_load_role_and_topology_proposal_becomes_pending_then_one_revision() {
    let mut repository = repository();
    let brace = StructuralMember {
        id: "brace-1".into(),
        start_node: "left-base".into(),
        end_node: "right-eave".into(),
        role: "brace".into(),
        semantic_tags: vec!["lateral".into()],
        section_id: "200UB".into(),
        material_id: "steel".into(),
    };
    let load = LoadInput {
        id: "roof-gravity".into(),
        target: AssignmentTargetRef::Member("rafter".into()),
        load_case_id: "dead".into(),
        direction: LoadVector {
            x: 0.0,
            y: -1.0,
            z: 0.0,
        },
        magnitude: LoadMagnitude::LineLoad {
            value: 2.0,
            unit: LineLoadUnit::NewtonsPerMeter,
        },
    };
    let validated = validate_agent_proposal(
        proposal(vec![
            UntrustedAgentOperation::AddLoad(load),
            UntrustedAgentOperation::SetMemberRole {
                member_id: "rafter".into(),
                role: MemberRole::Rafter,
            },
            UntrustedAgentOperation::AddTopologyMember(brace),
        ]),
        &[
            AllowedAgentOperation::AddLoad,
            AllowedAgentOperation::SetMemberRole,
            AllowedAgentOperation::AddTopologyMember,
        ],
    )
    .unwrap();
    assert_eq!(validated.provenance(), &provenance());
    let (accepted_revision_id, accepted_provenance) = {
        let accepted = validated.accept(&mut repository).unwrap();
        (
            accepted.revision_id().clone(),
            accepted.agent_provenance().cloned(),
        )
    };
    assert_eq!(accepted_revision_id, RevisionId::from("agent-revision"));
    assert_eq!(repository.revision_count(), 2);
    assert_eq!(accepted_provenance, Some(provenance()));
}

#[test]
fn typed_proposal_is_checked_against_the_parent_before_pending_state() {
    let mut repository = repository();
    let invalid = validate_agent_proposal(
        proposal(vec![UntrustedAgentOperation::AddLoad(LoadInput {
            id: "bad-target".into(),
            target: AssignmentTargetRef::Member("missing-member".into()),
            load_case_id: "dead".into(),
            direction: LoadVector {
                x: 0.0,
                y: -1.0,
                z: 0.0,
            },
            magnitude: LoadMagnitude::LineLoad {
                value: 1.0,
                unit: LineLoadUnit::KiloNewtonsPerMeter,
            },
        })]),
        &[AllowedAgentOperation::AddLoad],
    )
    .unwrap();
    assert!(matches!(
        invalid.create_pending(&mut repository),
        Err(RepositoryError::Patch(PatchError::InvalidLoadTarget { .. }))
    ));
    assert_eq!(repository.revision_count(), 1);
    assert!(repository.proposal(&"agent-proposal".into()).is_err());
}
