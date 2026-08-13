//! Append-only, single-parent revision graph primitives.
//!
//! This module deliberately models only immutable revisions and their lineage.
//! Conversation heads and optimistic updates belong to `conversation`; durable
//! persistence belongs to a later repository slice.

use crate::{RevisionId, SnapshotId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

/// One immutable accepted design state in the initial single-parent graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Revision {
    id: RevisionId,
    snapshot_id: SnapshotId,
    parent_revision_id: Option<RevisionId>,
}

impl Revision {
    pub fn id(&self) -> &RevisionId {
        &self.id
    }

    pub fn snapshot_id(&self) -> &SnapshotId {
        &self.snapshot_id
    }

    pub fn parent_revision_id(&self) -> Option<&RevisionId> {
        self.parent_revision_id.as_ref()
    }
}

/// A compact, navigation-oriented revision record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevisionHistoryEntry {
    pub revision_id: RevisionId,
    pub snapshot_id: SnapshotId,
}

/// Errors produced when an append-only graph operation would violate lineage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RevisionGraphError {
    DuplicateRevisionId(RevisionId),
    MissingParentRevision(RevisionId),
    UnknownRevision(RevisionId),
}

impl fmt::Display for RevisionGraphError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateRevisionId(id) => write!(formatter, "revision `{id}` already exists"),
            Self::MissingParentRevision(id) => {
                write!(formatter, "parent revision `{id}` does not exist")
            }
            Self::UnknownRevision(id) => write!(formatter, "revision `{id}` does not exist"),
        }
    }
}

impl Error for RevisionGraphError {}

/// In-memory, append-only revision lineage.
///
/// The initial contract intentionally supports exactly zero or one parent per
/// revision. It has no merge operation and never mutates an existing revision.
#[derive(Debug, Default, Clone)]
pub struct RevisionGraph {
    revisions: BTreeMap<RevisionId, Revision>,
}

impl RevisionGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn revision(&self, revision_id: &RevisionId) -> Result<&Revision, RevisionGraphError> {
        self.revisions
            .get(revision_id)
            .ok_or_else(|| RevisionGraphError::UnknownRevision(revision_id.clone()))
    }

    pub fn contains(&self, revision_id: &RevisionId) -> bool {
        self.revisions.contains_key(revision_id)
    }

    pub fn revision_count(&self) -> usize {
        self.revisions.len()
    }

    pub fn insert_root(
        &mut self,
        revision_id: RevisionId,
        snapshot_id: SnapshotId,
    ) -> Result<&Revision, RevisionGraphError> {
        self.insert(revision_id, snapshot_id, None)
    }

    pub fn insert_child(
        &mut self,
        revision_id: RevisionId,
        snapshot_id: SnapshotId,
        parent_revision_id: RevisionId,
    ) -> Result<&Revision, RevisionGraphError> {
        self.insert(revision_id, snapshot_id, Some(parent_revision_id))
    }

    /// Returns the selected revision followed by each immutable ancestor.
    pub fn history_from(
        &self,
        revision_id: &RevisionId,
    ) -> Result<Vec<RevisionHistoryEntry>, RevisionGraphError> {
        let mut history = Vec::new();
        let mut current_id = revision_id.clone();

        loop {
            let current = self.revision(&current_id)?;
            history.push(RevisionHistoryEntry {
                revision_id: current.id.clone(),
                snapshot_id: current.snapshot_id.clone(),
            });

            match &current.parent_revision_id {
                Some(parent_id) => current_id = parent_id.clone(),
                None => return Ok(history),
            }
        }
    }

    fn insert(
        &mut self,
        revision_id: RevisionId,
        snapshot_id: SnapshotId,
        parent_revision_id: Option<RevisionId>,
    ) -> Result<&Revision, RevisionGraphError> {
        if self.revisions.contains_key(&revision_id) {
            return Err(RevisionGraphError::DuplicateRevisionId(revision_id));
        }

        if let Some(parent_id) = &parent_revision_id {
            if !self.revisions.contains_key(parent_id) {
                return Err(RevisionGraphError::MissingParentRevision(parent_id.clone()));
            }
        }

        self.revisions.insert(
            revision_id.clone(),
            Revision {
                id: revision_id.clone(),
                snapshot_id,
                parent_revision_id,
            },
        );

        // The entry was inserted above with the same key.
        Ok(self
            .revisions
            .get(&revision_id)
            .expect("inserted revision must be queryable"))
    }
}
