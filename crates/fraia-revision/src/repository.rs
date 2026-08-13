//! In-memory composition of the immutable revision primitives.
//!
//! This is the Stage S5 domain service.  It deliberately has no persistence,
//! HTTP, prompt, UI, or legacy project-workflow dependencies.  SQLite can
//! implement the same operations later without changing their engineering
//! semantics.

use crate::agent_contract::AgentTurnProvenance;
use crate::conversation::{
    ConversationError, ConversationHead, ConversationOrigin, DesignConversation,
    InMemoryDesignProject,
};
use crate::diff::{SemanticDiff, semantic_diff};
use crate::evidence::{AnalysisEvidence, EvidenceDependency, EvidenceStaleness, staleness_for};
use crate::graph::{RevisionGraphError, RevisionHistoryEntry};
use crate::patch::{PatchError, StructuralPatch, apply_patch};
use crate::snapshot::{ModelSnapshot, SnapshotError};
use crate::working_copy::{WorkingCopy, WorkingCopyError};
use crate::{ArtefactId, ConversationId, EvidenceId, ProjectId, RevisionId, SnapshotId};
use fraia_core::StructuralModel;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

/// Typed identity for an uncommitted agent proposal.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct ProposalId(String);

impl ProposalId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ProposalId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for ProposalId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Who accepted an immutable revision.  The proposal itself remains distinct
/// from an accepted engineering state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevisionAuthorKind {
    System,
    Agent,
    Manual,
    User,
}

/// The meaningful domain operation that created an accepted revision.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RevisionOperation {
    Root,
    AcceptedProposal { proposal_id: ProposalId },
    ManualEdit,
    UserPatch,
}

/// Repository metadata layered over the minimal graph lineage record.
#[derive(Debug, Clone)]
pub struct RevisionRecord {
    revision_id: RevisionId,
    snapshot_id: SnapshotId,
    parent_revision_id: Option<RevisionId>,
    conversation_id: ConversationId,
    author_kind: RevisionAuthorKind,
    operation: RevisionOperation,
    semantic_diff: SemanticDiff,
    agent_provenance: Option<AgentTurnProvenance>,
}

impl RevisionRecord {
    pub fn revision_id(&self) -> &RevisionId {
        &self.revision_id
    }

    pub fn snapshot_id(&self) -> &SnapshotId {
        &self.snapshot_id
    }

    pub fn parent_revision_id(&self) -> Option<&RevisionId> {
        self.parent_revision_id.as_ref()
    }

    pub fn conversation_id(&self) -> &ConversationId {
        &self.conversation_id
    }

    pub fn author_kind(&self) -> RevisionAuthorKind {
        self.author_kind
    }

    pub fn operation(&self) -> &RevisionOperation {
        &self.operation
    }

    pub fn semantic_diff(&self) -> &SemanticDiff {
        &self.semantic_diff
    }
    pub fn agent_provenance(&self) -> Option<&AgentTurnProvenance> {
        self.agent_provenance.as_ref()
    }
}

/// An agent's uncommitted, typed request to create one child revision.
#[derive(Debug, Clone)]
pub struct RevisionProposal {
    id: ProposalId,
    conversation_id: ConversationId,
    parent_revision_id: RevisionId,
    proposed_revision_id: RevisionId,
    patch: StructuralPatch,
    status: ProposalStatus,
    agent_provenance: Option<AgentTurnProvenance>,
}

impl RevisionProposal {
    pub fn id(&self) -> &ProposalId {
        &self.id
    }

    pub fn conversation_id(&self) -> &ConversationId {
        &self.conversation_id
    }

    pub fn parent_revision_id(&self) -> &RevisionId {
        &self.parent_revision_id
    }

    pub fn proposed_revision_id(&self) -> &RevisionId {
        &self.proposed_revision_id
    }

    pub fn patch(&self) -> &StructuralPatch {
        &self.patch
    }

    pub fn status(&self) -> &ProposalStatus {
        &self.status
    }

    pub fn agent_provenance(&self) -> Option<&AgentTurnProvenance> {
        self.agent_provenance.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProposalStatus {
    Pending,
    Accepted { revision_id: RevisionId },
    Rejected,
}

/// Immutable visual material attached to an exact model/evidence source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualArtefact {
    id: ArtefactId,
    kind: String,
    source_snapshot_id: SnapshotId,
    source_evidence_id: Option<EvidenceId>,
    object_refs: Vec<String>,
    renderer_version: String,
    render_payload: Vec<u8>,
}

impl VisualArtefact {
    pub fn new(
        id: ArtefactId,
        kind: impl Into<String>,
        source_snapshot_id: SnapshotId,
        source_evidence_id: Option<EvidenceId>,
        object_refs: Vec<String>,
        renderer_version: impl Into<String>,
        render_payload: Vec<u8>,
    ) -> Self {
        Self {
            id,
            kind: kind.into(),
            source_snapshot_id,
            source_evidence_id,
            object_refs,
            renderer_version: renderer_version.into(),
            render_payload,
        }
    }

    pub fn id(&self) -> &ArtefactId {
        &self.id
    }
    pub fn kind(&self) -> &str {
        &self.kind
    }
    pub fn source_snapshot_id(&self) -> &SnapshotId {
        &self.source_snapshot_id
    }
    pub fn source_evidence_id(&self) -> Option<&EvidenceId> {
        self.source_evidence_id.as_ref()
    }
    pub fn object_refs(&self) -> &[String] {
        &self.object_refs
    }
    pub fn renderer_version(&self) -> &str {
        &self.renderer_version
    }
    pub fn render_payload(&self) -> &[u8] {
        &self.render_payload
    }
}

#[derive(Debug)]
pub enum RepositoryError {
    Conversation(ConversationError),
    Snapshot(SnapshotError),
    Patch(PatchError),
    WorkingCopy(WorkingCopyError),
    DuplicateProposalId(ProposalId),
    UnknownProposal(ProposalId),
    ProposalNotPending(ProposalId),
    DuplicateEvidenceId(EvidenceId),
    UnknownEvidence(EvidenceId),
    EvidenceSnapshotMismatch {
        evidence_id: EvidenceId,
        revision_id: RevisionId,
    },
    DuplicateArtefactId(ArtefactId),
    UnknownArtefact(ArtefactId),
    ArtefactSnapshotMissing(SnapshotId),
    ArtefactEvidenceMissing(EvidenceId),
    ArtefactEvidenceSnapshotMismatch {
        artefact_id: ArtefactId,
        evidence_id: EvidenceId,
    },
    UnknownSnapshot(SnapshotId),
    DuplicateRevisionId(RevisionId),
}

impl From<ConversationError> for RepositoryError {
    fn from(error: ConversationError) -> Self {
        Self::Conversation(error)
    }
}
impl From<RevisionGraphError> for RepositoryError {
    fn from(error: RevisionGraphError) -> Self {
        Self::Conversation(ConversationError::Graph(error))
    }
}
impl From<SnapshotError> for RepositoryError {
    fn from(error: SnapshotError) -> Self {
        Self::Snapshot(error)
    }
}
impl From<PatchError> for RepositoryError {
    fn from(error: PatchError) -> Self {
        Self::Patch(error)
    }
}
impl From<WorkingCopyError> for RepositoryError {
    fn from(error: WorkingCopyError) -> Self {
        Self::WorkingCopy(error)
    }
}

impl fmt::Display for RepositoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Conversation(error) => error.fmt(formatter),
            Self::Snapshot(error) => error.fmt(formatter),
            Self::Patch(error) => error.fmt(formatter),
            Self::WorkingCopy(error) => error.fmt(formatter),
            Self::DuplicateProposalId(id) => write!(formatter, "proposal `{id}` already exists"),
            Self::UnknownProposal(id) => write!(formatter, "proposal `{id}` does not exist"),
            Self::ProposalNotPending(id) => write!(formatter, "proposal `{id}` is not pending"),
            Self::DuplicateEvidenceId(id) => write!(formatter, "evidence `{id}` already exists"),
            Self::UnknownEvidence(id) => write!(formatter, "evidence `{id}` does not exist"),
            Self::EvidenceSnapshotMismatch {
                evidence_id,
                revision_id,
            } => write!(
                formatter,
                "evidence `{evidence_id}` is not bound to revision `{revision_id}`'s snapshot"
            ),
            Self::DuplicateArtefactId(id) => write!(formatter, "artefact `{id}` already exists"),
            Self::UnknownArtefact(id) => write!(formatter, "artefact `{id}` does not exist"),
            Self::ArtefactSnapshotMissing(id) => {
                write!(formatter, "artefact source snapshot `{id}` does not exist")
            }
            Self::ArtefactEvidenceMissing(id) => {
                write!(formatter, "artefact source evidence `{id}` does not exist")
            }
            Self::ArtefactEvidenceSnapshotMismatch {
                artefact_id,
                evidence_id,
            } => write!(
                formatter,
                "artefact `{artefact_id}` source snapshot does not match evidence `{evidence_id}`"
            ),
            Self::UnknownSnapshot(id) => write!(formatter, "snapshot `{id}` does not exist"),
            Self::DuplicateRevisionId(id) => write!(formatter, "revision `{id}` already exists"),
        }
    }
}

impl Error for RepositoryError {}

/// A pure in-memory domain service.  It is intentionally the only S5 type
/// that composes snapshots, graph heads, proposals, working copies, evidence,
/// and artefacts.
#[derive(Debug, Clone)]
pub struct InMemoryRevisionRepository {
    project: InMemoryDesignProject,
    snapshots: BTreeMap<SnapshotId, ModelSnapshot>,
    revisions: BTreeMap<RevisionId, RevisionRecord>,
    proposals: BTreeMap<ProposalId, RevisionProposal>,
    evidence: BTreeMap<EvidenceId, AnalysisEvidence>,
    artefacts: BTreeMap<ArtefactId, VisualArtefact>,
}

impl InMemoryRevisionRepository {
    /// Creates one project, one root conversation, and one immutable root
    /// snapshot.  The supplied root revision is a durable engineering state.
    pub fn create(
        project_id: ProjectId,
        root_conversation_id: ConversationId,
        root_purpose: impl Into<String>,
        root_revision_id: RevisionId,
        root_model: StructuralModel,
    ) -> Result<Self, RepositoryError> {
        let snapshot = ModelSnapshot::capture(root_model)?;
        let snapshot_id = snapshot.id().clone();
        let project = InMemoryDesignProject::create(
            project_id,
            root_conversation_id.clone(),
            root_purpose,
            root_revision_id.clone(),
            snapshot_id.clone(),
        )?;
        let mut snapshots = BTreeMap::new();
        snapshots.insert(snapshot_id.clone(), snapshot);
        let mut revisions = BTreeMap::new();
        revisions.insert(
            root_revision_id.clone(),
            RevisionRecord {
                revision_id: root_revision_id,
                snapshot_id,
                parent_revision_id: None,
                conversation_id: root_conversation_id,
                author_kind: RevisionAuthorKind::System,
                operation: RevisionOperation::Root,
                semantic_diff: SemanticDiff::default(),
                agent_provenance: None,
            },
        );
        Ok(Self {
            project,
            snapshots,
            revisions,
            proposals: BTreeMap::new(),
            evidence: BTreeMap::new(),
            artefacts: BTreeMap::new(),
        })
    }

    pub fn project_id(&self) -> &ProjectId {
        self.project.project_id()
    }
    pub fn root_revision_id(&self) -> &RevisionId {
        self.project.root_revision_id()
    }
    pub fn revision_count(&self) -> usize {
        self.project.graph().revision_count()
    }
    pub fn snapshot_count(&self) -> usize {
        self.snapshots.len()
    }

    /// Restores a durable conversation reference without changing any
    /// existing revision or snapshot. This is used only at a process restart
    /// boundary; normal conversation creation still goes through fork/resume.
    pub fn restore_conversation(
        &mut self,
        id: ConversationId,
        purpose: impl Into<String>,
        origin: ConversationOrigin,
    ) -> Result<DesignConversation, RepositoryError> {
        let start_revision_id = match &origin {
            ConversationOrigin::ProjectRoot => self.root_revision_id().clone(),
            ConversationOrigin::StartedFromRevision { revision_id }
            | ConversationOrigin::ForkedFromRevision { revision_id }
            | ConversationOrigin::ResumedFromRevision { revision_id } => revision_id.clone(),
        };
        let conversation = match origin {
            ConversationOrigin::ProjectRoot => {
                return Err(RepositoryError::Conversation(
                    ConversationError::DuplicateConversationId(id),
                ));
            }
            ConversationOrigin::StartedFromRevision { .. } => {
                self.project
                    .start_conversation(id, purpose, start_revision_id)?
            }
            ConversationOrigin::ForkedFromRevision { .. } => {
                self.project
                    .fork_conversation(id, purpose, start_revision_id)?
            }
            ConversationOrigin::ResumedFromRevision { .. } => {
                self.project
                    .resume_conversation(id, purpose, start_revision_id)?
            }
        };
        Ok(conversation.clone())
    }

    /// Rehydrates one immutable revision from its already identity-verified
    /// durable snapshot. The stored snapshot is authoritative at restart;
    /// patch application is reserved for new proposals and manual edits.
    pub fn restore_revision(
        &mut self,
        revision_id: RevisionId,
        snapshot: ModelSnapshot,
        parent_revision_id: RevisionId,
        conversation_id: ConversationId,
        author_kind: RevisionAuthorKind,
        operation: RevisionOperation,
        semantic_diff: SemanticDiff,
        agent_provenance: Option<AgentTurnProvenance>,
    ) -> Result<&RevisionRecord, RepositoryError> {
        if self.revisions.contains_key(&revision_id) || self.project.graph().contains(&revision_id)
        {
            return Err(RepositoryError::DuplicateRevisionId(revision_id));
        }
        if !self.snapshots.contains_key(snapshot.id()) {
            self.snapshots
                .insert(snapshot.id().clone(), snapshot.clone());
        }
        self.project.append_revision(
            &conversation_id,
            &parent_revision_id,
            revision_id.clone(),
            snapshot.id().clone(),
        )?;
        self.revisions.insert(
            revision_id.clone(),
            RevisionRecord {
                revision_id: revision_id.clone(),
                snapshot_id: snapshot.id().clone(),
                parent_revision_id: Some(parent_revision_id),
                conversation_id,
                author_kind,
                operation,
                semantic_diff,
                agent_provenance,
            },
        );
        Ok(self
            .revisions
            .get(&revision_id)
            .expect("restored revision metadata must be present"))
    }

    pub fn snapshot(&self, snapshot_id: &SnapshotId) -> Result<&ModelSnapshot, RepositoryError> {
        self.snapshots
            .get(snapshot_id)
            .ok_or_else(|| RepositoryError::UnknownSnapshot(snapshot_id.clone()))
    }

    pub fn revision(&self, revision_id: &RevisionId) -> Result<&RevisionRecord, RepositoryError> {
        self.revisions.get(revision_id).ok_or_else(|| {
            match self.project.graph().revision(revision_id) {
                Err(RevisionGraphError::UnknownRevision(id)) => RepositoryError::Conversation(
                    ConversationError::Graph(RevisionGraphError::UnknownRevision(id)),
                ),
                Err(error) => RepositoryError::Conversation(ConversationError::Graph(error)),
                Ok(_) => unreachable!("all graph revisions have repository metadata"),
            }
        })
    }

    pub fn conversation(
        &self,
        id: &ConversationId,
    ) -> Result<&DesignConversation, RepositoryError> {
        Ok(self.project.conversation(id)?)
    }

    pub fn head(&self, id: &ConversationId) -> Result<ConversationHead, RepositoryError> {
        Ok(self.project.head(id)?)
    }

    /// Compact, newest-first lineage for one current conversation head.
    pub fn history(
        &self,
        id: &ConversationId,
    ) -> Result<Vec<RevisionHistoryEntry>, RepositoryError> {
        Ok(self.project.history(id)?)
    }

    pub fn create_conversation(
        &mut self,
        id: ConversationId,
        purpose: impl Into<String>,
        start_revision_id: RevisionId,
    ) -> Result<DesignConversation, RepositoryError> {
        Ok(self
            .project
            .start_conversation(id, purpose, start_revision_id)?
            .clone())
    }

    pub fn fork(
        &mut self,
        id: ConversationId,
        purpose: impl Into<String>,
        revision_id: RevisionId,
    ) -> Result<DesignConversation, RepositoryError> {
        Ok(self
            .project
            .fork_conversation(id, purpose, revision_id)?
            .clone())
    }

    pub fn resume(
        &mut self,
        id: ConversationId,
        purpose: impl Into<String>,
        revision_id: RevisionId,
    ) -> Result<DesignConversation, RepositoryError> {
        Ok(self
            .project
            .resume_conversation(id, purpose, revision_id)?
            .clone())
    }

    /// Records an uncommitted agent proposal.  It cannot mutate a snapshot,
    /// revision graph, or conversation head.
    pub fn create_proposal(
        &mut self,
        id: ProposalId,
        conversation_id: ConversationId,
        parent_revision_id: RevisionId,
        proposed_revision_id: RevisionId,
        patch: StructuralPatch,
    ) -> Result<&RevisionProposal, RepositoryError> {
        if self.proposals.contains_key(&id) {
            return Err(RepositoryError::DuplicateProposalId(id));
        }
        self.project.conversation(&conversation_id)?;
        self.project.graph().revision(&parent_revision_id)?;
        if self.project.graph().contains(&proposed_revision_id)
            || self.revisions.contains_key(&proposed_revision_id)
        {
            return Err(RepositoryError::DuplicateRevisionId(proposed_revision_id));
        }
        self.proposals.insert(
            id.clone(),
            RevisionProposal {
                id: id.clone(),
                conversation_id,
                parent_revision_id,
                proposed_revision_id,
                patch,
                status: ProposalStatus::Pending,
                agent_provenance: None,
            },
        );
        Ok(self
            .proposals
            .get(&id)
            .expect("inserted proposal must be queryable"))
    }

    pub fn create_proposal_with_provenance(
        &mut self,
        id: ProposalId,
        conversation_id: ConversationId,
        parent_revision_id: RevisionId,
        proposed_revision_id: RevisionId,
        patch: StructuralPatch,
        agent_provenance: AgentTurnProvenance,
    ) -> Result<&RevisionProposal, RepositoryError> {
        if self.proposals.contains_key(&id) {
            return Err(RepositoryError::DuplicateProposalId(id));
        }
        self.project.conversation(&conversation_id)?;
        self.project.graph().revision(&parent_revision_id)?;
        if self.project.graph().contains(&proposed_revision_id)
            || self.revisions.contains_key(&proposed_revision_id)
        {
            return Err(RepositoryError::DuplicateRevisionId(proposed_revision_id));
        }
        self.proposals.insert(
            id.clone(),
            RevisionProposal {
                id: id.clone(),
                conversation_id,
                parent_revision_id,
                proposed_revision_id,
                patch,
                status: ProposalStatus::Pending,
                agent_provenance: Some(agent_provenance),
            },
        );
        Ok(self
            .proposals
            .get(&id)
            .expect("inserted proposal must be queryable"))
    }

    /// Rehydrates proposal audit state after the immutable revision graph has
    /// been restored. Accepted proposals point at an already-restored child;
    /// pending and rejected proposals intentionally do not create a child.
    pub fn restore_proposal(
        &mut self,
        id: ProposalId,
        conversation_id: ConversationId,
        parent_revision_id: RevisionId,
        proposed_revision_id: RevisionId,
        patch: StructuralPatch,
        status: ProposalStatus,
        agent_provenance: Option<AgentTurnProvenance>,
    ) -> Result<&RevisionProposal, RepositoryError> {
        if self.proposals.contains_key(&id) {
            return Err(RepositoryError::DuplicateProposalId(id));
        }
        self.project.conversation(&conversation_id)?;
        self.project.graph().revision(&parent_revision_id)?;
        if matches!(status, ProposalStatus::Accepted { .. })
            && !self.revisions.contains_key(&proposed_revision_id)
        {
            return Err(RepositoryError::UnknownSnapshot(
                self.revision(&parent_revision_id)?.snapshot_id().clone(),
            ));
        }
        self.proposals.insert(
            id.clone(),
            RevisionProposal {
                id: id.clone(),
                conversation_id,
                parent_revision_id,
                proposed_revision_id,
                patch,
                status,
                agent_provenance,
            },
        );
        Ok(self
            .proposals
            .get(&id)
            .expect("restored proposal must be queryable"))
    }

    pub fn proposal(&self, id: &ProposalId) -> Result<&RevisionProposal, RepositoryError> {
        self.proposals
            .get(id)
            .ok_or_else(|| RepositoryError::UnknownProposal(id.clone()))
    }

    /// Explicit rejection retains only non-engineering proposal audit state.
    /// It creates no snapshot, revision, or head update.
    pub fn reject_proposal(&mut self, id: &ProposalId) -> Result<(), RepositoryError> {
        let proposal = self
            .proposals
            .get_mut(id)
            .ok_or_else(|| RepositoryError::UnknownProposal(id.clone()))?;
        if proposal.status != ProposalStatus::Pending {
            return Err(RepositoryError::ProposalNotPending(id.clone()));
        }
        proposal.status = ProposalStatus::Rejected;
        Ok(())
    }

    /// Validates and accepts a pending proposal as exactly one agent revision.
    pub fn accept_proposal(&mut self, id: &ProposalId) -> Result<&RevisionRecord, RepositoryError> {
        self.accept_proposal_with_provenance(id, None)
    }

    pub fn accept_proposal_with_provenance(
        &mut self,
        id: &ProposalId,
        agent_provenance: Option<AgentTurnProvenance>,
    ) -> Result<&RevisionRecord, RepositoryError> {
        let proposal = self.proposal(id)?.clone();
        if proposal.status != ProposalStatus::Pending {
            return Err(RepositoryError::ProposalNotPending(id.clone()));
        }
        self.ensure_conversation_head(&proposal.conversation_id, &proposal.parent_revision_id)?;
        if self
            .project
            .graph()
            .contains(&proposal.proposed_revision_id)
        {
            return Err(RepositoryError::DuplicateRevisionId(
                proposal.proposed_revision_id,
            ));
        }
        let parent = self.revision(&proposal.parent_revision_id)?.clone();
        let parent_snapshot = self.snapshot(&parent.snapshot_id)?.to_working_model();
        let applied = apply_patch(&parent_snapshot, &proposal.patch)?;
        let snapshot = ModelSnapshot::capture(applied.model)?;
        let snapshot_id = snapshot.id().clone();

        self.project.append_revision(
            &proposal.conversation_id,
            &proposal.parent_revision_id,
            proposal.proposed_revision_id.clone(),
            snapshot_id.clone(),
        )?;
        self.snapshots
            .entry(snapshot_id.clone())
            .or_insert(snapshot);
        self.revisions.insert(
            proposal.proposed_revision_id.clone(),
            RevisionRecord {
                revision_id: proposal.proposed_revision_id.clone(),
                snapshot_id,
                parent_revision_id: Some(proposal.parent_revision_id.clone()),
                conversation_id: proposal.conversation_id.clone(),
                author_kind: RevisionAuthorKind::Agent,
                operation: RevisionOperation::AcceptedProposal {
                    proposal_id: proposal.id.clone(),
                },
                semantic_diff: applied.diff,
                agent_provenance,
            },
        );
        self.proposals
            .get_mut(id)
            .expect("proposal remains present")
            .status = ProposalStatus::Accepted {
            revision_id: proposal.proposed_revision_id.clone(),
        };
        Ok(self
            .revisions
            .get(&proposal.proposed_revision_id)
            .expect("accepted revision metadata must be present"))
    }

    /// Attaches immutable analysis evidence only to the exact revision snapshot
    /// that produced it.
    pub fn attach_evidence(
        &mut self,
        revision_id: &RevisionId,
        evidence: AnalysisEvidence,
    ) -> Result<&AnalysisEvidence, RepositoryError> {
        let revision = self.revision(revision_id)?;
        if evidence.authored_snapshot_id() != revision.snapshot_id() {
            return Err(RepositoryError::EvidenceSnapshotMismatch {
                evidence_id: evidence.id().clone(),
                revision_id: revision_id.clone(),
            });
        }
        if self.evidence.contains_key(evidence.id()) {
            return Err(RepositoryError::DuplicateEvidenceId(evidence.id().clone()));
        }
        let id = evidence.id().clone();
        self.evidence.insert(id.clone(), evidence);
        Ok(self
            .evidence
            .get(&id)
            .expect("inserted evidence must be queryable"))
    }

    pub fn evidence(&self, id: &EvidenceId) -> Result<&AnalysisEvidence, RepositoryError> {
        self.evidence
            .get(id)
            .ok_or_else(|| RepositoryError::UnknownEvidence(id.clone()))
    }

    /// Computes staleness from exact snapshots and supplied current dependency
    /// identities; no UI status or timestamp participates.
    pub fn evidence_staleness(
        &self,
        evidence_id: &EvidenceId,
        inspected_revision_id: &RevisionId,
        current_dependencies: &[EvidenceDependency],
    ) -> Result<EvidenceStaleness, RepositoryError> {
        let evidence = self.evidence(evidence_id)?;
        let inspected = self.revision(inspected_revision_id)?;
        let source = self.snapshot(evidence.authored_snapshot_id())?;
        let target = self.snapshot(inspected.snapshot_id())?;
        Ok(staleness_for(
            evidence,
            target.id(),
            current_dependencies,
            &semantic_diff(source.model(), target.model()),
        ))
    }

    pub fn attach_artefact(
        &mut self,
        artefact: VisualArtefact,
    ) -> Result<&VisualArtefact, RepositoryError> {
        if self.artefacts.contains_key(artefact.id()) {
            return Err(RepositoryError::DuplicateArtefactId(artefact.id().clone()));
        }
        if !self.snapshots.contains_key(artefact.source_snapshot_id()) {
            return Err(RepositoryError::ArtefactSnapshotMissing(
                artefact.source_snapshot_id().clone(),
            ));
        }
        if let Some(evidence_id) = artefact.source_evidence_id() {
            let evidence = self
                .evidence(evidence_id)
                .map_err(|_| RepositoryError::ArtefactEvidenceMissing(evidence_id.clone()))?;
            if evidence.authored_snapshot_id() != artefact.source_snapshot_id() {
                return Err(RepositoryError::ArtefactEvidenceSnapshotMismatch {
                    artefact_id: artefact.id().clone(),
                    evidence_id: evidence_id.clone(),
                });
            }
        }
        let id = artefact.id().clone();
        self.artefacts.insert(id.clone(), artefact);
        Ok(self
            .artefacts
            .get(&id)
            .expect("inserted artefact must be queryable"))
    }

    pub fn artefact(&self, id: &ArtefactId) -> Result<&VisualArtefact, RepositoryError> {
        self.artefacts
            .get(id)
            .ok_or_else(|| RepositoryError::UnknownArtefact(id.clone()))
    }

    pub fn artefacts_for_snapshot(&self, snapshot_id: &SnapshotId) -> Vec<&VisualArtefact> {
        self.artefacts
            .values()
            .filter(|artefact| artefact.source_snapshot_id() == snapshot_id)
            .collect()
    }

    pub fn open_working_copy(
        &self,
        revision_id: &RevisionId,
    ) -> Result<WorkingCopy, RepositoryError> {
        let revision = self.revision(revision_id)?;
        Ok(WorkingCopy::open(
            revision_id.clone(),
            self.snapshot(revision.snapshot_id())?,
        ))
    }

    /// Records one explicit manual revision from a closed editor session.  It
    /// preflights the parent/head before closing the copy, so a stale editor
    /// cannot consume its one commit on a failed optimistic update.
    pub fn commit_working_copy(
        &mut self,
        conversation_id: &ConversationId,
        working_copy: &mut WorkingCopy,
        revision_id: RevisionId,
    ) -> Result<&RevisionRecord, RepositoryError> {
        let parent_revision_id = working_copy.parent_revision_id().clone();
        self.ensure_conversation_head(conversation_id, &parent_revision_id)?;
        if self.project.graph().contains(&revision_id) {
            return Err(RepositoryError::DuplicateRevisionId(revision_id));
        }
        let commit = working_copy.commit(revision_id.clone())?;
        let snapshot_id = commit.snapshot().id().clone();
        self.project.append_revision(
            conversation_id,
            &parent_revision_id,
            revision_id.clone(),
            snapshot_id.clone(),
        )?;
        self.snapshots
            .entry(snapshot_id.clone())
            .or_insert_with(|| commit.snapshot().clone());
        self.revisions.insert(
            revision_id.clone(),
            RevisionRecord {
                revision_id: revision_id.clone(),
                snapshot_id,
                parent_revision_id: Some(parent_revision_id),
                conversation_id: conversation_id.clone(),
                author_kind: RevisionAuthorKind::Manual,
                operation: RevisionOperation::ManualEdit,
                semantic_diff: commit.semantic_diff().clone(),
                agent_provenance: None,
            },
        );
        Ok(self
            .revisions
            .get(&revision_id)
            .expect("manual revision metadata must be present"))
    }

    fn ensure_conversation_head(
        &self,
        conversation_id: &ConversationId,
        expected: &RevisionId,
    ) -> Result<(), RepositoryError> {
        let head = self.project.head(conversation_id)?;
        if head.head_revision_id != *expected {
            return Err(RepositoryError::Conversation(
                ConversationError::ExpectedHeadMismatch {
                    conversation_id: conversation_id.clone(),
                    expected_revision_id: expected.clone(),
                    actual_revision_id: head.head_revision_id,
                },
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversation::ConversationOrigin;
    use crate::diff::DiffCategory;
    use crate::patch::{Length, Position, StructuralOperation};
    use crate::root_fixture;

    fn move_node(node_id: &str, y: f64) -> StructuralPatch {
        StructuralPatch {
            operations: vec![StructuralOperation::MoveNode {
                node_id: node_id.into(),
                position: Position {
                    x: Length::meters(if node_id == "right-base" { 20.0 } else { 0.0 }),
                    y: Length::meters(y),
                    z: Length::meters(0.0),
                },
            }],
        }
    }

    #[test]
    fn s5_acceptance_repository_composes_revisions_proposals_evidence_artefacts_and_manual_edits() {
        let fixture = root_fixture();
        let overall = fixture.conversation_id.clone();
        let root = fixture.root_revision_id.clone();
        let mut repository = InMemoryRevisionRepository::create(
            fixture.project_id,
            overall.clone(),
            "Overall framing",
            root.clone(),
            fixture.model,
        )
        .unwrap();
        let root_snapshot_id = repository.revision(&root).unwrap().snapshot_id().clone();
        let root_bytes = repository
            .snapshot(&root_snapshot_id)
            .unwrap()
            .canonical_bytes()
            .to_vec();

        repository
            .create_proposal(
                ProposalId::from("raise-eaves"),
                overall.clone(),
                root.clone(),
                RevisionId::from("agent-r1"),
                move_node("left-eave", 7.0),
            )
            .unwrap();
        let accepted = repository
            .accept_proposal(&ProposalId::from("raise-eaves"))
            .unwrap()
            .clone();
        assert_eq!(repository.revision_count(), 2);
        assert_eq!(accepted.parent_revision_id(), Some(&root));
        assert_eq!(accepted.author_kind(), RevisionAuthorKind::Agent);
        assert_eq!(
            accepted.operation(),
            &RevisionOperation::AcceptedProposal {
                proposal_id: ProposalId::from("raise-eaves")
            }
        );
        assert_ne!(accepted.snapshot_id(), &root_snapshot_id);
        assert_eq!(
            repository
                .snapshot(&root_snapshot_id)
                .unwrap()
                .canonical_bytes(),
            root_bytes
        );

        repository
            .create_proposal(
                ProposalId::from("discarded"),
                overall.clone(),
                RevisionId::from("agent-r1"),
                RevisionId::from("rejected-r2"),
                move_node("right-eave", 8.0),
            )
            .unwrap();
        repository
            .reject_proposal(&ProposalId::from("discarded"))
            .unwrap();
        assert_eq!(repository.revision_count(), 2);
        assert_eq!(
            repository.head(&overall).unwrap().head_revision_id,
            RevisionId::from("agent-r1")
        );
        assert_eq!(
            repository
                .proposal(&ProposalId::from("discarded"))
                .unwrap()
                .status(),
            &ProposalStatus::Rejected
        );

        let fork = ConversationId::from("lateral-stability");
        repository
            .fork(fork.clone(), "Lateral stability", root.clone())
            .unwrap();
        assert_eq!(
            repository.conversation(&fork).unwrap().origin(),
            &ConversationOrigin::ForkedFromRevision {
                revision_id: root.clone()
            }
        );

        let evidence = AnalysisEvidence::new(
            EvidenceId::from("root-analysis"),
            root_snapshot_id.clone(),
            None,
            vec![EvidenceDependency::new(
                "supports",
                "supports:v1",
                [DiffCategory::Support, DiffCategory::Geometry],
            )],
        )
        .unwrap();
        repository.attach_evidence(&root, evidence).unwrap();
        let agent_snapshot_id = accepted.snapshot_id().clone();
        repository
            .attach_artefact(VisualArtefact::new(
                ArtefactId::from("agent-preview"),
                "model-preview",
                agent_snapshot_id.clone(),
                None,
                vec!["left-eave".into()],
                "renderer-v1",
                vec![1, 2, 3],
            ))
            .unwrap();
        assert_eq!(
            repository.artefacts_for_snapshot(&agent_snapshot_id).len(),
            1
        );
        assert_eq!(
            repository
                .evidence_staleness(
                    &EvidenceId::from("root-analysis"),
                    &root,
                    &[EvidenceDependency::new(
                        "supports",
                        "supports:v1",
                        [DiffCategory::Support, DiffCategory::Geometry]
                    ),]
                )
                .unwrap(),
            EvidenceStaleness::Current
        );
        assert!(
            repository
                .evidence_staleness(
                    &EvidenceId::from("root-analysis"),
                    &RevisionId::from("agent-r1"),
                    &[EvidenceDependency::new(
                        "supports",
                        "supports:v2",
                        [DiffCategory::Support, DiffCategory::Geometry]
                    ),]
                )
                .unwrap()
                .is_stale()
        );

        let mut copy = repository
            .open_working_copy(&RevisionId::from("agent-r1"))
            .unwrap();
        copy.apply(&move_node("left-eave", 7.5)).unwrap();
        copy.apply(&move_node("right-eave", 7.5)).unwrap();
        let manual = repository
            .commit_working_copy(&overall, &mut copy, RevisionId::from("manual-r2"))
            .unwrap()
            .clone();
        assert_eq!(repository.revision_count(), 3);
        assert_eq!(
            manual.parent_revision_id(),
            Some(&RevisionId::from("agent-r1"))
        );
        assert_eq!(manual.author_kind(), RevisionAuthorKind::Manual);
        assert!(manual.semantic_diff().affects(DiffCategory::Geometry));
        assert!(copy.is_closed());
        assert_eq!(
            repository
                .history(&overall)
                .unwrap()
                .iter()
                .map(|entry| entry.revision_id.clone())
                .collect::<Vec<_>>(),
            vec![
                RevisionId::from("manual-r2"),
                RevisionId::from("agent-r1"),
                root
            ]
        );
        assert!(
            repository
                .evidence_staleness(
                    &EvidenceId::from("root-analysis"),
                    &RevisionId::from("manual-r2"),
                    &[EvidenceDependency::new(
                        "supports",
                        "supports:v2",
                        [DiffCategory::Support, DiffCategory::Geometry]
                    ),]
                )
                .unwrap()
                .is_stale()
        );
    }
}
