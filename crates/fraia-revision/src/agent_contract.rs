//! Framework-independent boundary for agent context and typed proposals.
//!
//! This module deliberately does not call an LLM, persist data, or mutate a
//! repository by itself.  It turns untrusted agent output into the existing
//! closed [`StructuralPatch`] vocabulary before a caller may create a pending
//! repository proposal.
//!
use crate::conversation::ConversationHead;
use crate::patch::{
    LoadInput, MemberRole, NodeInput, Position, StructuralOperation, StructuralPatch, apply_patch,
};
use crate::repository::{
    InMemoryRevisionRepository, ProposalId, RepositoryError, RevisionProposal,
};
use crate::{ConversationId, EvidenceId, ProjectId, RevisionId, SnapshotId};
use fraia_core::{
    ReleaseAssignment, StructuralMember, StructuralPlate, SupportAssignment,
    understand_structural_model,
};
use std::fmt;

/// A confirmed project fact or assumption. Engineering facts must be supplied
/// explicitly instead of being inferred from conversation prose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectFact {
    pub key: String,
    pub value: String,
    pub confirmed: bool,
}

/// Provider/model/turn lineage carried with an agent proposal. It is distinct
/// from structural authority: it explains where a proposal came from, not
/// whether the proposal is valid or approved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentTurnProvenance {
    pub provider: String,
    pub model: String,
    pub turn_id: String,
}

/// Provenance for the context projection itself. This binds the semantic
/// understanding and evidence selection to the exact authored snapshot sent
/// to a turn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentContextProvenance {
    pub source: String,
    pub snapshot_id: SnapshotId,
}

/// Exact, immutable evidence made available to a turn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceReference {
    pub evidence_id: EvidenceId,
    pub authored_snapshot_id: SnapshotId,
    pub provenance: String,
}

/// Knowledge is guidance only. This closed type deliberately has no
/// "authoritative" variant, so compiled wiki text cannot be mistaken for
/// project fact, analysis evidence, or code-compliance approval.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeGuidance {
    pub reference_id: String,
    pub title: String,
    pub excerpt: String,
}

impl KnowledgeGuidance {
    pub const AUTHORITY: &'static str = "guidance_only";
}

/// The closed operation vocabulary an agent may request. Deletions remain
/// explicit policy choices and are still subject to reference-safe patch
/// validation before they can become a pending proposal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllowedAgentOperation {
    AddNode,
    MoveNode,
    AddMember,
    AddSupport,
    AddLoad,
    SetMemberRole,
    SetSection,
    SetRelease,
    AddPlate,
    AddTopologyMember,
    DeleteNode,
    DeleteMember,
    DeletePlate,
    DeleteSupport,
    DeleteLoad,
    DeleteRelease,
}

/// Structured but untrusted operation output received from an agent runtime.
/// `Unknown` and `UnsafeRawStructuralJson` exist so a transport adapter can
/// represent rejected output without attempting an unsafe fallback.
#[derive(Debug, Clone)]
pub enum UntrustedAgentOperation {
    AddNode(NodeInput),
    MoveNode {
        node_id: String,
        position: Position,
    },
    AddMember(StructuralMember),
    AddSupport(SupportAssignment),
    AddLoad(LoadInput),
    SetMemberRole {
        member_id: String,
        role: MemberRole,
    },
    SetSection {
        member_id: String,
        section_id: String,
    },
    SetRelease(ReleaseAssignment),
    AddPlate(StructuralPlate),
    AddTopologyMember(StructuralMember),
    DeleteNode {
        node_id: String,
    },
    DeleteMember {
        member_id: String,
    },
    DeletePlate {
        plate_id: String,
    },
    DeleteSupport {
        support_id: String,
    },
    DeleteLoad {
        load_id: String,
    },
    DeleteRelease {
        release_id: String,
    },
    Unknown {
        kind: String,
    },
    UnsafeRawStructuralJson {
        payload: String,
    },
}

/// A transport-neutral proposal before it has passed the closed vocabulary
/// validation gate.
#[derive(Debug, Clone)]
pub struct UntrustedAgentProposal {
    pub proposal_id: ProposalId,
    pub proposed_revision_id: RevisionId,
    pub conversation_id: ConversationId,
    pub parent_revision_id: RevisionId,
    pub provenance: AgentTurnProvenance,
    pub operations: Vec<UntrustedAgentOperation>,
}

/// A proposal admitted by the contract. It still cannot mutate engineering
/// state until an explicit repository acceptance call is made.
#[derive(Debug, Clone)]
pub struct ValidatedAgentProposal {
    proposal_id: ProposalId,
    proposed_revision_id: RevisionId,
    conversation_id: ConversationId,
    parent_revision_id: RevisionId,
    provenance: AgentTurnProvenance,
    patch: StructuralPatch,
}

impl ValidatedAgentProposal {
    pub fn proposal_id(&self) -> &ProposalId {
        &self.proposal_id
    }
    pub fn proposed_revision_id(&self) -> &RevisionId {
        &self.proposed_revision_id
    }
    pub fn conversation_id(&self) -> &ConversationId {
        &self.conversation_id
    }
    pub fn parent_revision_id(&self) -> &RevisionId {
        &self.parent_revision_id
    }
    pub fn provenance(&self) -> &AgentTurnProvenance {
        &self.provenance
    }
    pub fn patch(&self) -> &StructuralPatch {
        &self.patch
    }

    /// Creates only a pending proposal. The existing repository requires a
    /// separate explicit `accept_proposal` call to create a revision.
    pub fn create_pending<'a>(
        &self,
        repository: &'a mut InMemoryRevisionRepository,
    ) -> Result<&'a RevisionProposal, RepositoryError> {
        // Admission is pure and happens before the repository records pending
        // state. Acceptance repeats the same check against the current head,
        // preserving the optimistic concurrency boundary.
        let parent = repository.revision(&self.parent_revision_id)?.clone();
        let snapshot = repository.snapshot(parent.snapshot_id())?;
        apply_patch(snapshot.model(), &self.patch)?;
        repository.create_proposal(
            self.proposal_id.clone(),
            self.conversation_id.clone(),
            self.parent_revision_id.clone(),
            self.proposed_revision_id.clone(),
            self.patch.clone(),
        )
    }

    /// Convenience path for callers that want the proposal's provider/model/
    /// turn lineage to be carried onto the accepted revision as well.
    pub fn accept<'a>(
        &self,
        repository: &'a mut InMemoryRevisionRepository,
    ) -> Result<&'a crate::repository::RevisionRecord, RepositoryError> {
        self.create_pending(repository)?;
        repository.accept_proposal_with_provenance(&self.proposal_id, Some(self.provenance.clone()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProposalValidationError {
    EmptyProposal,
    InvalidProvenance,
    OperationNotAllowed { operation: &'static str },
    UnknownOperation { kind: String },
    RawStructuralJsonForbidden,
}

impl fmt::Display for ProposalValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyProposal => f.write_str("agent proposal contains no operations"),
            Self::InvalidProvenance => {
                f.write_str("agent proposal requires provider, model, and turn provenance")
            }
            Self::OperationNotAllowed { operation } => {
                write!(f, "agent operation `{operation}` is not allowed")
            }
            Self::UnknownOperation { kind } => write!(f, "unknown agent operation `{kind}`"),
            Self::RawStructuralJsonForbidden => {
                f.write_str("raw structural JSON cannot mutate a model")
            }
        }
    }
}

impl std::error::Error for ProposalValidationError {}

/// Validates a proposal against the current explicit operation policy before
/// any repository method can be called.
pub fn validate_agent_proposal(
    proposal: UntrustedAgentProposal,
    allowed: &[AllowedAgentOperation],
) -> Result<ValidatedAgentProposal, ProposalValidationError> {
    if proposal.operations.is_empty() {
        return Err(ProposalValidationError::EmptyProposal);
    }
    if proposal.provenance.provider.trim().is_empty()
        || proposal.provenance.model.trim().is_empty()
        || proposal.provenance.turn_id.trim().is_empty()
    {
        return Err(ProposalValidationError::InvalidProvenance);
    }

    let mut operations = Vec::with_capacity(proposal.operations.len());
    for operation in proposal.operations {
        match operation {
            UntrustedAgentOperation::AddNode(node) => {
                require_allowed(allowed, AllowedAgentOperation::AddNode, "add_node")?;
                operations.push(StructuralOperation::AddNode(node));
            }
            UntrustedAgentOperation::MoveNode { node_id, position } => {
                require_allowed(allowed, AllowedAgentOperation::MoveNode, "move_node")?;
                operations.push(StructuralOperation::MoveNode { node_id, position });
            }
            UntrustedAgentOperation::AddMember(member) => {
                require_allowed(allowed, AllowedAgentOperation::AddMember, "add_member")?;
                operations.push(StructuralOperation::AddMember(member));
            }
            UntrustedAgentOperation::AddSupport(support) => {
                require_allowed(allowed, AllowedAgentOperation::AddSupport, "add_support")?;
                operations.push(StructuralOperation::AddSupport(support));
            }
            UntrustedAgentOperation::AddLoad(load) => {
                require_allowed(allowed, AllowedAgentOperation::AddLoad, "add_load")?;
                operations.push(StructuralOperation::AddLoad(load));
            }
            UntrustedAgentOperation::SetMemberRole { member_id, role } => {
                require_allowed(
                    allowed,
                    AllowedAgentOperation::SetMemberRole,
                    "set_member_role",
                )?;
                operations.push(StructuralOperation::SetMemberRole { member_id, role });
            }
            UntrustedAgentOperation::SetSection {
                member_id,
                section_id,
            } => {
                require_allowed(allowed, AllowedAgentOperation::SetSection, "set_section")?;
                operations.push(StructuralOperation::SetSection {
                    member_id,
                    section_id,
                });
            }
            UntrustedAgentOperation::SetRelease(release) => {
                require_allowed(allowed, AllowedAgentOperation::SetRelease, "set_release")?;
                operations.push(StructuralOperation::SetRelease(release));
            }
            UntrustedAgentOperation::AddPlate(plate) => {
                require_allowed(allowed, AllowedAgentOperation::AddPlate, "add_plate")?;
                operations.push(StructuralOperation::AddPlate(plate));
            }
            UntrustedAgentOperation::AddTopologyMember(member) => {
                require_allowed(
                    allowed,
                    AllowedAgentOperation::AddTopologyMember,
                    "add_topology_member",
                )?;
                operations.push(StructuralOperation::AddMember(member));
            }
            UntrustedAgentOperation::DeleteNode { node_id } => {
                require_allowed(allowed, AllowedAgentOperation::DeleteNode, "delete_node")?;
                operations.push(StructuralOperation::DeleteNode { node_id });
            }
            UntrustedAgentOperation::DeleteMember { member_id } => {
                require_allowed(
                    allowed,
                    AllowedAgentOperation::DeleteMember,
                    "delete_member",
                )?;
                operations.push(StructuralOperation::DeleteMember { member_id });
            }
            UntrustedAgentOperation::DeletePlate { plate_id } => {
                require_allowed(allowed, AllowedAgentOperation::DeletePlate, "delete_plate")?;
                operations.push(StructuralOperation::DeletePlate { plate_id });
            }
            UntrustedAgentOperation::DeleteSupport { support_id } => {
                require_allowed(
                    allowed,
                    AllowedAgentOperation::DeleteSupport,
                    "delete_support",
                )?;
                operations.push(StructuralOperation::DeleteSupport { support_id });
            }
            UntrustedAgentOperation::DeleteLoad { load_id } => {
                require_allowed(allowed, AllowedAgentOperation::DeleteLoad, "delete_load")?;
                operations.push(StructuralOperation::DeleteLoad { load_id });
            }
            UntrustedAgentOperation::DeleteRelease { release_id } => {
                require_allowed(
                    allowed,
                    AllowedAgentOperation::DeleteRelease,
                    "delete_release",
                )?;
                operations.push(StructuralOperation::DeleteRelease { release_id });
            }
            UntrustedAgentOperation::Unknown { kind } => {
                return Err(ProposalValidationError::UnknownOperation { kind });
            }
            UntrustedAgentOperation::UnsafeRawStructuralJson { .. } => {
                return Err(ProposalValidationError::RawStructuralJsonForbidden);
            }
        }
    }

    Ok(ValidatedAgentProposal {
        proposal_id: proposal.proposal_id,
        proposed_revision_id: proposal.proposed_revision_id,
        conversation_id: proposal.conversation_id,
        parent_revision_id: proposal.parent_revision_id,
        provenance: proposal.provenance,
        patch: StructuralPatch { operations },
    })
}

fn require_allowed(
    allowed: &[AllowedAgentOperation],
    required: AllowedAgentOperation,
    operation: &'static str,
) -> Result<(), ProposalValidationError> {
    if allowed.contains(&required) {
        Ok(())
    } else {
        Err(ProposalValidationError::OperationNotAllowed { operation })
    }
}

/// All information an agent may treat as typed engineering context for one
/// turn. A raw model serialization is deliberately absent from this type.
#[derive(Debug, Clone)]
pub struct TypedAgentContext {
    pub project_id: ProjectId,
    pub project_facts: Vec<ProjectFact>,
    pub unresolved_assumptions: Vec<ProjectFact>,
    pub conversation_summary: String,
    pub conversation: ConversationHead,
    pub current_revision_id: RevisionId,
    pub provenance: AgentContextProvenance,
    pub semantic_model: fraia_core::ModelUnderstandingReport,
    pub evidence: Vec<EvidenceReference>,
    pub knowledge: Vec<KnowledgeGuidance>,
    pub allowed_operations: Vec<AllowedAgentOperation>,
}

/// Transport adapters must distinguish a complete typed projection from raw
/// diagnostic data. The latter cannot be used as an agent engineering context.
#[derive(Debug, Clone)]
pub enum AgentContextInput {
    Typed(TypedAgentContext),
    RawStructuralJsonOnly { payload: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextValidationError {
    RawStructuralJsonInsufficient,
    EmptyProjectId,
    EmptyConversationPurpose,
    EmptyConversationSummary,
    RevisionDoesNotMatchConversationHead,
    EvidenceSnapshotDoesNotMatchHead { evidence_id: EvidenceId },
    EmptyAllowedOperationVocabulary,
    InvalidProjectFact,
    InvalidContextProvenance,
    InvalidEvidenceReference,
    InvalidKnowledgeGuidance,
    Repository(String),
}

impl fmt::Display for ContextValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RawStructuralJsonInsufficient => {
                f.write_str("raw structural JSON alone is insufficient agent context")
            }
            Self::EmptyProjectId => f.write_str("agent context requires a project id"),
            Self::EmptyConversationPurpose => {
                f.write_str("agent context requires a conversation purpose")
            }
            Self::EmptyConversationSummary => {
                f.write_str("agent context requires a conversation summary")
            }
            Self::RevisionDoesNotMatchConversationHead => {
                f.write_str("agent context revision must match its conversation head")
            }
            Self::EvidenceSnapshotDoesNotMatchHead { evidence_id } => write!(
                f,
                "evidence `{evidence_id}` is not bound to the current snapshot"
            ),
            Self::EmptyAllowedOperationVocabulary => {
                f.write_str("agent context requires an allowed-operation vocabulary")
            }
            Self::InvalidProjectFact => f.write_str("agent context contains an empty project fact"),
            Self::InvalidContextProvenance => {
                f.write_str("agent context provenance must identify its exact snapshot")
            }
            Self::InvalidEvidenceReference => {
                f.write_str("agent context evidence requires an id and provenance")
            }
            Self::InvalidKnowledgeGuidance => f.write_str(
                "agent context knowledge guidance requires a reference, title, and excerpt",
            ),
            Self::Repository(error) => write!(f, "could not build agent context: {error}"),
        }
    }
}

impl std::error::Error for ContextValidationError {}

/// Validates a context received from a framework/transport adapter.
pub fn validate_agent_context(
    input: AgentContextInput,
) -> Result<TypedAgentContext, ContextValidationError> {
    let context = match input {
        AgentContextInput::Typed(context) => context,
        AgentContextInput::RawStructuralJsonOnly { .. } => {
            return Err(ContextValidationError::RawStructuralJsonInsufficient);
        }
    };
    if context.project_id.as_str().is_empty() {
        return Err(ContextValidationError::EmptyProjectId);
    }
    if context.conversation.purpose.trim().is_empty() {
        return Err(ContextValidationError::EmptyConversationPurpose);
    }
    if context.conversation_summary.trim().is_empty() {
        return Err(ContextValidationError::EmptyConversationSummary);
    }
    if context.conversation.head_revision_id != context.current_revision_id {
        return Err(ContextValidationError::RevisionDoesNotMatchConversationHead);
    }
    if context.allowed_operations.is_empty() {
        return Err(ContextValidationError::EmptyAllowedOperationVocabulary);
    }
    if context.provenance.source.trim().is_empty()
        || context.provenance.snapshot_id != context.conversation.head_snapshot_id
    {
        return Err(ContextValidationError::InvalidContextProvenance);
    }
    if context
        .project_facts
        .iter()
        .chain(context.unresolved_assumptions.iter())
        .any(|fact| fact.key.trim().is_empty() || fact.value.trim().is_empty())
    {
        return Err(ContextValidationError::InvalidProjectFact);
    }
    if context
        .unresolved_assumptions
        .iter()
        .any(|assumption| assumption.confirmed)
    {
        return Err(ContextValidationError::InvalidProjectFact);
    }
    for evidence in &context.evidence {
        if evidence.authored_snapshot_id != context.conversation.head_snapshot_id {
            return Err(ContextValidationError::EvidenceSnapshotDoesNotMatchHead {
                evidence_id: evidence.evidence_id.clone(),
            });
        }
        if evidence.evidence_id.as_str().trim().is_empty() || evidence.provenance.trim().is_empty()
        {
            return Err(ContextValidationError::InvalidEvidenceReference);
        }
    }
    if context.knowledge.iter().any(|guidance| {
        guidance.reference_id.trim().is_empty()
            || guidance.title.trim().is_empty()
            || guidance.excerpt.trim().is_empty()
    }) {
        return Err(ContextValidationError::InvalidKnowledgeGuidance);
    }
    Ok(context)
}

/// Builds a complete typed context from the in-memory repository. Evidence is
/// supplied explicitly because the S5 repository has no revision-index query
/// yet; this makes exact evidence selection auditable at the caller boundary.
pub fn build_context(
    repository: &InMemoryRevisionRepository,
    conversation_id: &ConversationId,
    project_facts: Vec<ProjectFact>,
    evidence: Vec<EvidenceReference>,
    knowledge: Vec<KnowledgeGuidance>,
    allowed_operations: Vec<AllowedAgentOperation>,
) -> Result<TypedAgentContext, ContextValidationError> {
    build_context_with_assumptions(
        repository,
        conversation_id,
        project_facts,
        Vec::new(),
        evidence,
        knowledge,
        allowed_operations,
    )
}

/// Variant used when the conversation transport has captured explicit
/// unresolved assumptions separately from confirmed project facts.
pub fn build_context_with_assumptions(
    repository: &InMemoryRevisionRepository,
    conversation_id: &ConversationId,
    project_facts: Vec<ProjectFact>,
    unresolved_assumptions: Vec<ProjectFact>,
    evidence: Vec<EvidenceReference>,
    knowledge: Vec<KnowledgeGuidance>,
    allowed_operations: Vec<AllowedAgentOperation>,
) -> Result<TypedAgentContext, ContextValidationError> {
    let conversation = repository
        .head(conversation_id)
        .map_err(|error| ContextValidationError::Repository(error.to_string()))?;
    let revision = repository
        .revision(&conversation.head_revision_id)
        .map_err(|error| ContextValidationError::Repository(error.to_string()))?;
    let snapshot = repository
        .snapshot(revision.snapshot_id())
        .map_err(|error| ContextValidationError::Repository(error.to_string()))?;
    validate_agent_context(AgentContextInput::Typed(TypedAgentContext {
        project_id: repository.project_id().clone(),
        project_facts,
        unresolved_assumptions,
        conversation_summary: conversation.purpose.clone(),
        current_revision_id: revision.revision_id().clone(),
        provenance: AgentContextProvenance {
            source: "in-memory-revision-repository".into(),
            snapshot_id: revision.snapshot_id().clone(),
        },
        semantic_model: understand_structural_model(snapshot.model()),
        conversation,
        evidence,
        knowledge,
        allowed_operations,
    }))
}
