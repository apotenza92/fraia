//! Conversation and head projections over the append-only revision graph.
//!
//! A conversation is a durable work-item reference to a graph head. Starting,
//! forking, and resuming only create new conversations or descendants; they do
//! not rewrite existing heads or revisions.

use crate::graph::{Revision, RevisionGraph, RevisionGraphError, RevisionHistoryEntry};
use crate::{ConversationId, ProjectId, RevisionId, SnapshotId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

/// Why a conversation began at its initial revision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConversationOrigin {
    ProjectRoot,
    StartedFromRevision { revision_id: RevisionId },
    ForkedFromRevision { revision_id: RevisionId },
    ResumedFromRevision { revision_id: RevisionId },
}

/// Durable work-item metadata and its currently selected graph head.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignConversation {
    id: ConversationId,
    project_id: ProjectId,
    purpose: String,
    origin: ConversationOrigin,
    head_revision_id: RevisionId,
}

impl DesignConversation {
    pub fn id(&self) -> &ConversationId {
        &self.id
    }

    pub fn project_id(&self) -> &ProjectId {
        &self.project_id
    }

    pub fn purpose(&self) -> &str {
        &self.purpose
    }

    pub fn origin(&self) -> &ConversationOrigin {
        &self.origin
    }

    pub fn head_revision_id(&self) -> &RevisionId {
        &self.head_revision_id
    }
}

/// Compact conversation state for navigation without exposing the full graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationHead {
    pub conversation_id: ConversationId,
    pub purpose: String,
    pub head_revision_id: RevisionId,
    pub head_snapshot_id: SnapshotId,
}

/// Errors from conversation operations or optimistic head movement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConversationError {
    DuplicateConversationId(ConversationId),
    UnknownConversation(ConversationId),
    ExpectedHeadMismatch {
        conversation_id: ConversationId,
        expected_revision_id: RevisionId,
        actual_revision_id: RevisionId,
    },
    Graph(RevisionGraphError),
}

impl From<RevisionGraphError> for ConversationError {
    fn from(error: RevisionGraphError) -> Self {
        Self::Graph(error)
    }
}

impl fmt::Display for ConversationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateConversationId(id) => {
                write!(formatter, "conversation `{id}` already exists")
            }
            Self::UnknownConversation(id) => {
                write!(formatter, "conversation `{id}` does not exist")
            }
            Self::ExpectedHeadMismatch {
                conversation_id,
                expected_revision_id,
                actual_revision_id,
            } => write!(
                formatter,
                "conversation `{conversation_id}` head is `{actual_revision_id}`, not expected `{expected_revision_id}`"
            ),
            Self::Graph(error) => error.fmt(formatter),
        }
    }
}

impl Error for ConversationError {}

/// In-memory project state for the S3 graph/conversation contract.
///
/// This is intentionally not a persistence format. A later repository slice
/// will store immutable graph records and heads atomically.
#[derive(Debug, Clone)]
pub struct InMemoryDesignProject {
    project_id: ProjectId,
    root_revision_id: RevisionId,
    graph: RevisionGraph,
    conversations: BTreeMap<ConversationId, DesignConversation>,
}

impl InMemoryDesignProject {
    /// Creates the project's one root revision and its initial conversation.
    pub fn create(
        project_id: ProjectId,
        root_conversation_id: ConversationId,
        root_purpose: impl Into<String>,
        root_revision_id: RevisionId,
        root_snapshot_id: SnapshotId,
    ) -> Result<Self, ConversationError> {
        let mut graph = RevisionGraph::new();
        graph.insert_root(root_revision_id.clone(), root_snapshot_id)?;

        let root_conversation = DesignConversation {
            id: root_conversation_id.clone(),
            project_id: project_id.clone(),
            purpose: root_purpose.into(),
            origin: ConversationOrigin::ProjectRoot,
            head_revision_id: root_revision_id.clone(),
        };
        let mut conversations = BTreeMap::new();
        conversations.insert(root_conversation_id, root_conversation);

        Ok(Self {
            project_id,
            root_revision_id,
            graph,
            conversations,
        })
    }

    pub fn project_id(&self) -> &ProjectId {
        &self.project_id
    }

    pub fn root_revision_id(&self) -> &RevisionId {
        &self.root_revision_id
    }

    pub fn graph(&self) -> &RevisionGraph {
        &self.graph
    }

    pub fn conversation(
        &self,
        conversation_id: &ConversationId,
    ) -> Result<&DesignConversation, ConversationError> {
        self.conversations
            .get(conversation_id)
            .ok_or_else(|| ConversationError::UnknownConversation(conversation_id.clone()))
    }

    /// Starts a related work item at any existing revision.
    pub fn start_conversation(
        &mut self,
        conversation_id: ConversationId,
        purpose: impl Into<String>,
        start_revision_id: RevisionId,
    ) -> Result<&DesignConversation, ConversationError> {
        self.create_conversation(
            conversation_id,
            purpose.into(),
            start_revision_id.clone(),
            ConversationOrigin::StartedFromRevision {
                revision_id: start_revision_id,
            },
        )
    }

    /// Starts an explicitly alternative conversation from any existing revision.
    pub fn fork_conversation(
        &mut self,
        conversation_id: ConversationId,
        purpose: impl Into<String>,
        fork_revision_id: RevisionId,
    ) -> Result<&DesignConversation, ConversationError> {
        self.create_conversation(
            conversation_id,
            purpose.into(),
            fork_revision_id.clone(),
            ConversationOrigin::ForkedFromRevision {
                revision_id: fork_revision_id,
            },
        )
    }

    /// Resuming never rewrites a current head. It starts a distinct descendant
    /// conversation at the selected historical revision; callers then append a
    /// new revision with [`Self::append_revision`].
    pub fn resume_conversation(
        &mut self,
        conversation_id: ConversationId,
        purpose: impl Into<String>,
        resume_revision_id: RevisionId,
    ) -> Result<&DesignConversation, ConversationError> {
        self.create_conversation(
            conversation_id,
            purpose.into(),
            resume_revision_id.clone(),
            ConversationOrigin::ResumedFromRevision {
                revision_id: resume_revision_id,
            },
        )
    }

    /// Appends one child revision and moves a head only if the caller still
    /// expects that conversation's current parent. A failed optimistic check
    /// leaves both graph and conversation state unchanged.
    pub fn append_revision(
        &mut self,
        conversation_id: &ConversationId,
        expected_parent_revision_id: &RevisionId,
        revision_id: RevisionId,
        snapshot_id: SnapshotId,
    ) -> Result<&Revision, ConversationError> {
        let actual_head = self.conversation(conversation_id)?.head_revision_id.clone();
        if actual_head != *expected_parent_revision_id {
            return Err(ConversationError::ExpectedHeadMismatch {
                conversation_id: conversation_id.clone(),
                expected_revision_id: expected_parent_revision_id.clone(),
                actual_revision_id: actual_head,
            });
        }

        self.graph.insert_child(
            revision_id.clone(),
            snapshot_id,
            expected_parent_revision_id.clone(),
        )?;
        self.conversations
            .get_mut(conversation_id)
            .expect("queried conversation must remain present")
            .head_revision_id = revision_id.clone();

        Ok(self
            .graph
            .revision(&revision_id)
            .expect("inserted revision must be queryable"))
    }

    pub fn head(
        &self,
        conversation_id: &ConversationId,
    ) -> Result<ConversationHead, ConversationError> {
        let conversation = self.conversation(conversation_id)?;
        let revision = self.graph.revision(&conversation.head_revision_id)?;
        Ok(ConversationHead {
            conversation_id: conversation.id.clone(),
            purpose: conversation.purpose.clone(),
            head_revision_id: revision.id().clone(),
            head_snapshot_id: revision.snapshot_id().clone(),
        })
    }

    pub fn history(
        &self,
        conversation_id: &ConversationId,
    ) -> Result<Vec<RevisionHistoryEntry>, ConversationError> {
        let conversation = self.conversation(conversation_id)?;
        Ok(self.graph.history_from(&conversation.head_revision_id)?)
    }

    fn create_conversation(
        &mut self,
        conversation_id: ConversationId,
        purpose: String,
        start_revision_id: RevisionId,
        origin: ConversationOrigin,
    ) -> Result<&DesignConversation, ConversationError> {
        if self.conversations.contains_key(&conversation_id) {
            return Err(ConversationError::DuplicateConversationId(conversation_id));
        }
        self.graph.revision(&start_revision_id)?;

        self.conversations.insert(
            conversation_id.clone(),
            DesignConversation {
                id: conversation_id.clone(),
                project_id: self.project_id.clone(),
                purpose,
                origin,
                head_revision_id: start_revision_id,
            },
        );
        Ok(self
            .conversations
            .get(&conversation_id)
            .expect("inserted conversation must be queryable"))
    }
}

#[cfg(test)]
mod tests {
    use super::{ConversationError, ConversationOrigin, InMemoryDesignProject};
    use crate::{ConversationId, ProjectId, RevisionId, SnapshotId};

    fn id(value: &str) -> RevisionId {
        RevisionId::from(value)
    }

    fn snapshot(value: &str) -> SnapshotId {
        SnapshotId::from(value)
    }

    fn project() -> InMemoryDesignProject {
        InMemoryDesignProject::create(
            ProjectId::from("warehouse"),
            ConversationId::from("overall-framing"),
            "Overall framing",
            id("A"),
            snapshot("snapshot-A"),
        )
        .unwrap()
    }

    #[test]
    fn acceptance_a_to_b_then_fork_and_resume_preserves_four_lineages() {
        let mut project = project();
        let overall = ConversationId::from("overall-framing");

        project
            .append_revision(&overall, &id("A"), id("B"), snapshot("snapshot-B"))
            .unwrap();
        project
            .fork_conversation(
                ConversationId::from("lateral-stability"),
                "Lateral stability",
                id("A"),
            )
            .unwrap();
        let lateral = ConversationId::from("lateral-stability");
        project
            .append_revision(&lateral, &id("A"), id("C"), snapshot("snapshot-C"))
            .unwrap();
        project
            .resume_conversation(
                ConversationId::from("low-carbon-review"),
                "Low-carbon review",
                id("A"),
            )
            .unwrap();
        let resumed = ConversationId::from("low-carbon-review");
        project
            .append_revision(&resumed, &id("A"), id("D"), snapshot("snapshot-D"))
            .unwrap();

        assert_eq!(project.graph().revision_count(), 4);
        assert_eq!(
            project
                .graph()
                .revision(&id("A"))
                .unwrap()
                .parent_revision_id(),
            None
        );
        assert_eq!(
            project
                .graph()
                .revision(&id("B"))
                .unwrap()
                .parent_revision_id(),
            Some(&id("A"))
        );
        assert_eq!(
            project
                .graph()
                .revision(&id("C"))
                .unwrap()
                .parent_revision_id(),
            Some(&id("A"))
        );
        assert_eq!(
            project
                .graph()
                .revision(&id("D"))
                .unwrap()
                .parent_revision_id(),
            Some(&id("A"))
        );
        assert_eq!(project.head(&overall).unwrap().head_revision_id, id("B"));
        assert_eq!(project.head(&lateral).unwrap().head_revision_id, id("C"));
        assert_eq!(project.head(&resumed).unwrap().head_revision_id, id("D"));
        assert_eq!(
            project
                .history(&overall)
                .unwrap()
                .iter()
                .map(|entry| entry.revision_id.clone())
                .collect::<Vec<_>>(),
            vec![id("B"), id("A")]
        );
        assert_eq!(
            project.conversation(&lateral).unwrap().origin(),
            &ConversationOrigin::ForkedFromRevision {
                revision_id: id("A")
            }
        );
        assert_eq!(
            project.conversation(&resumed).unwrap().origin(),
            &ConversationOrigin::ResumedFromRevision {
                revision_id: id("A")
            }
        );
    }

    #[test]
    fn incorrect_expected_parent_rejects_without_an_orphan_or_head_change() {
        let mut project = project();
        let overall = ConversationId::from("overall-framing");
        project
            .append_revision(&overall, &id("A"), id("B"), snapshot("snapshot-B"))
            .unwrap();

        let error = project
            .append_revision(&overall, &id("A"), id("C"), snapshot("snapshot-C"))
            .unwrap_err();

        assert_eq!(
            error,
            ConversationError::ExpectedHeadMismatch {
                conversation_id: overall.clone(),
                expected_revision_id: id("A"),
                actual_revision_id: id("B"),
            }
        );
        assert_eq!(project.graph().revision_count(), 2);
        assert!(!project.graph().contains(&id("C")));
        assert_eq!(project.head(&overall).unwrap().head_revision_id, id("B"));
    }

    #[test]
    fn a_related_conversation_can_start_from_any_existing_revision() {
        let mut project = project();
        let overall = ConversationId::from("overall-framing");
        project
            .append_revision(&overall, &id("A"), id("B"), snapshot("snapshot-B"))
            .unwrap();
        let review = ConversationId::from("roof-review");

        project
            .start_conversation(review.clone(), "Roof review", id("B"))
            .unwrap();

        assert_eq!(project.head(&review).unwrap().head_revision_id, id("B"));
        assert_eq!(
            project.conversation(&review).unwrap().origin(),
            &ConversationOrigin::StartedFromRevision {
                revision_id: id("B"),
            }
        );
    }
}
