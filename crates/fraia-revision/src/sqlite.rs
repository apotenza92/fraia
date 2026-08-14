//! SQLite storage for the append-only revision repository.
//!
//! This module deliberately stores the durable repository facts rather than a
//! serialised in-memory repository.  Rows are immutable after insertion;
//! conversation heads are the only mutable navigation references and are
//! advanced with an expected-parent optimistic check in the same transaction.
//!
//! The S5 in-memory service remains the domain composition layer.  This
//! adapter exposes the persistence primitives it needs without making SQLite
//! a source of engineering meaning or a replacement for typed patch
//! validation.

use crate::agent_contract::AgentTurnProvenance;
use crate::repository::ProposalId;
use crate::snapshot::{ModelSnapshot, Sha256SnapshotIdentityDeriver};
use crate::{
    ArtefactId, CanonicalFormatVersion, ConversationId, EvidenceId, ProjectId, RevisionId,
    SnapshotId, SnapshotIdentityDeriver,
};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, backup, params};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::{error::Error, fmt, fs, time::Duration};

const SCHEMA_VERSION: i64 = 4;
static BACKUP_TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub enum SqliteRepositoryError {
    Sqlite(rusqlite::Error),
    ProjectAlreadyExists(ProjectId),
    UnknownProject(ProjectId),
    UnknownConversation(ConversationId),
    UnknownRevision(RevisionId),
    UnknownSnapshot(SnapshotId),
    UnknownEvidence(EvidenceId),
    UnknownProposal(ProposalId),
    ConflictingProposal(ProposalId),
    ConflictingOperationRequest(String),
    BackupTargetExists(PathBuf),
    InvalidBackupTarget(PathBuf),
    BackupValidation(String),
    Io(std::io::Error),
    UnknownArtefact(ArtefactId),
    DuplicateRevision(RevisionId),
    DuplicateConversation(ConversationId),
    DuplicateEvidence(EvidenceId),
    DuplicateArtefact(ArtefactId),
    ImmutableSnapshotConflict(SnapshotId),
    InvalidSnapshotFormat(SnapshotId),
    InvalidSnapshotIdentity(SnapshotId),
    InvalidSnapshotPayload {
        snapshot_id: SnapshotId,
        reason: String,
    },
    RevisionNotInProject {
        project_id: ProjectId,
        revision_id: RevisionId,
    },
    InvalidRevisionSnapshotBinding {
        revision_id: RevisionId,
        revision_snapshot_id: SnapshotId,
        supplied_snapshot_id: SnapshotId,
    },
    InvalidRevisionParentBinding {
        revision_id: RevisionId,
        expected_parent_id: RevisionId,
        actual_parent_id: Option<RevisionId>,
    },
    ExpectedHeadConflict {
        conversation_id: ConversationId,
        expected_revision_id: RevisionId,
        actual_revision_id: RevisionId,
    },
    InvalidRoot,
    InvalidEvidenceBinding {
        evidence_id: EvidenceId,
        snapshot_id: SnapshotId,
    },
    InvalidArtefactBinding {
        artefact_id: ArtefactId,
        snapshot_id: SnapshotId,
    },
}

impl From<rusqlite::Error> for SqliteRepositoryError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl fmt::Display for SqliteRepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(error) => error.fmt(f),
            Self::ProjectAlreadyExists(id) => write!(f, "project `{id}` already exists"),
            Self::UnknownProject(id) => write!(f, "project `{id}` does not exist"),
            Self::UnknownConversation(id) => write!(f, "conversation `{id}` does not exist"),
            Self::UnknownRevision(id) => write!(f, "revision `{id}` does not exist"),
            Self::UnknownSnapshot(id) => write!(f, "snapshot `{id}` does not exist"),
            Self::UnknownEvidence(id) => write!(f, "evidence `{id}` does not exist"),
            Self::UnknownProposal(id) => write!(f, "proposal `{id}` does not exist"),
            Self::ConflictingProposal(id) => {
                write!(f, "proposal `{id}` already exists with different content")
            }
            Self::ConflictingOperationRequest(id) => write!(
                f,
                "operation request `{id}` was already used for different content"
            ),
            Self::BackupTargetExists(path) => {
                write!(
                    f,
                    "SQLite backup target `{}` already exists",
                    path.display()
                )
            }
            Self::InvalidBackupTarget(path) => {
                write!(
                    f,
                    "SQLite backup target `{}` has no file name",
                    path.display()
                )
            }
            Self::BackupValidation(reason) => {
                write!(f, "SQLite backup validation failed: {reason}")
            }
            Self::Io(error) => error.fmt(f),
            Self::UnknownArtefact(id) => write!(f, "artefact `{id}` does not exist"),
            Self::DuplicateRevision(id) => write!(f, "revision `{id}` already exists"),
            Self::DuplicateConversation(id) => write!(f, "conversation `{id}` already exists"),
            Self::DuplicateEvidence(id) => write!(f, "evidence `{id}` already exists"),
            Self::DuplicateArtefact(id) => write!(f, "artefact `{id}` already exists"),
            Self::ImmutableSnapshotConflict(id) => write!(
                f,
                "immutable snapshot `{id}` already exists with different content or format"
            ),
            Self::InvalidSnapshotFormat(id) => {
                write!(f, "snapshot `{id}` has an empty canonical format version")
            }
            Self::InvalidSnapshotIdentity(id) => {
                write!(
                    f,
                    "snapshot `{id}` does not match its canonical content identity"
                )
            }
            Self::InvalidSnapshotPayload {
                snapshot_id,
                reason,
            } => write!(
                f,
                "snapshot `{snapshot_id}` could not be hydrated as a typed model: {reason}"
            ),
            Self::RevisionNotInProject {
                project_id,
                revision_id,
            } => write!(
                f,
                "revision `{revision_id}` does not belong to project `{project_id}`"
            ),
            Self::InvalidRevisionSnapshotBinding {
                revision_id,
                revision_snapshot_id,
                supplied_snapshot_id,
            } => write!(
                f,
                "revision `{revision_id}` binds snapshot `{revision_snapshot_id}`, not supplied snapshot `{supplied_snapshot_id}`"
            ),
            Self::InvalidRevisionParentBinding {
                revision_id,
                expected_parent_id,
                actual_parent_id,
            } => {
                let actual_parent = actual_parent_id
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "<none>".into());
                write!(
                    f,
                    "revision `{revision_id}` binds parent `{actual_parent}`, not expected parent `{expected_parent_id}`"
                )
            }
            Self::ExpectedHeadConflict {
                conversation_id,
                expected_revision_id,
                actual_revision_id,
            } => write!(
                f,
                "conversation `{conversation_id}` head is `{actual_revision_id}`, not expected `{expected_revision_id}`"
            ),
            Self::InvalidRoot => f.write_str("root revision must not have a parent"),
            Self::InvalidEvidenceBinding {
                evidence_id,
                snapshot_id,
            } => write!(
                f,
                "evidence `{evidence_id}` is not bound to snapshot `{snapshot_id}`"
            ),
            Self::InvalidArtefactBinding {
                artefact_id,
                snapshot_id,
            } => write!(
                f,
                "artefact `{artefact_id}` is not bound to snapshot `{snapshot_id}`"
            ),
        }
    }
}
impl Error for SqliteRepositoryError {}

impl From<std::io::Error> for SqliteRepositoryError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

/// Durable snapshot content. `canonical_bytes` is the only large payload S6
/// stores directly; later blob storage may move it behind `blob_ref` without
/// changing the immutable identity or repository graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredSnapshot {
    pub id: SnapshotId,
    pub format_version: String,
    pub canonical_bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredRevision {
    pub id: RevisionId,
    pub snapshot_id: SnapshotId,
    pub parent_revision_id: Option<RevisionId>,
    pub conversation_id: ConversationId,
    /// Domain-owned serialised metadata: author, operation, semantic diff,
    /// provenance and message references. SQLite treats it as opaque JSON.
    pub metadata_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredConversation {
    pub id: ConversationId,
    pub project_id: ProjectId,
    pub purpose: String,
    pub origin_json: String,
    pub head_revision_id: RevisionId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredEvidence {
    pub id: EvidenceId,
    pub authored_snapshot_id: SnapshotId,
    pub resolved_snapshot_id: Option<SnapshotId>,
    /// Domain-owned exact analysis/request/dependency manifest.
    pub manifest_json: String,
    /// Optional content-addressed external output/log reference. S6 does not
    /// choose the final blob store.
    pub blob_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredProposal {
    pub id: ProposalId,
    pub project_id: ProjectId,
    pub conversation_id: ConversationId,
    pub parent_revision_id: RevisionId,
    pub proposed_revision_id: RevisionId,
    pub patch_json: String,
    pub status: String,
    pub accepted_revision_id: Option<RevisionId>,
    pub agent_provenance: Option<AgentTurnProvenance>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredOperationReceipt {
    pub request_id: String,
    pub request_json: String,
    pub response_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredArtefact {
    pub id: ArtefactId,
    pub kind: String,
    pub source_snapshot_id: SnapshotId,
    pub source_evidence_id: Option<EvidenceId>,
    pub manifest_json: String,
    pub blob_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredProjectRoot {
    pub project_id: ProjectId,
    pub root_conversation: StoredConversation,
    pub root_revision: StoredRevision,
    pub root_snapshot: StoredSnapshot,
}

/// A small, durable repository adapter. It has no global mutable singleton:
/// every operation is scoped to one SQLite connection and every head advance
/// is transactional.
pub struct SqliteRevisionRepository {
    connection: Connection,
}

impl SqliteRevisionRepository {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, SqliteRepositoryError> {
        let connection = Connection::open(path)?;
        Self::from_connection(connection)
    }

    pub fn open_in_memory() -> Result<Self, SqliteRepositoryError> {
        Self::from_connection(Connection::open_in_memory()?)
    }

    fn from_connection(mut connection: Connection) -> Result<Self, SqliteRepositoryError> {
        connection.pragma_update(None, "foreign_keys", "ON")?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.busy_timeout(Duration::from_secs(5))?;
        migrate(&mut connection)?;
        Ok(Self { connection })
    }

    /// Creates a transactionally consistent copy of this open repository at a
    /// new path. SQLite's online backup API includes committed pages that are
    /// still resident in the source WAL. The target is written and validated
    /// as a sibling temporary database, synced, and atomically published with
    /// a no-overwrite hard link. Existing targets and sidecars are never
    /// overwritten.
    pub fn backup_to_path(&self, target: impl AsRef<Path>) -> Result<(), SqliteRepositoryError> {
        let target = target.as_ref();
        let parent = target
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let file_name = target
            .file_name()
            .ok_or_else(|| SqliteRepositoryError::InvalidBackupTarget(target.to_path_buf()))?;
        for path in [
            target.to_path_buf(),
            sqlite_sidecar_path(target, "-wal"),
            sqlite_sidecar_path(target, "-shm"),
        ] {
            if path.exists() {
                return Err(SqliteRepositoryError::BackupTargetExists(path));
            }
        }

        let sequence = BACKUP_TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let mut temp_name = file_name.to_os_string();
        temp_name.push(format!(
            ".fraia-backup-{}-{sequence}.tmp",
            std::process::id()
        ));
        let temp_path = parent.join(temp_name);
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)?;

        let backup_result = (|| -> Result<(), SqliteRepositoryError> {
            let mut destination = Connection::open(&temp_path)?;
            destination.pragma_update(None, "foreign_keys", "ON")?;
            {
                let backup = backup::Backup::new(&self.connection, &mut destination)?;
                backup.run_to_completion(128, Duration::from_millis(1), None)?;
            }
            validate_backup_connection(&destination)?;
            destination.close().map_err(|(_, error)| error)?;
            fs::File::open(&temp_path)?.sync_all()?;
            // A same-directory hard link publishes the completed database
            // atomically and fails if another process created the target
            // after the preflight. Unlike rename, it never replaces a path.
            fs::hard_link(&temp_path, target)?;
            let _ = fs::remove_file(&temp_path);
            sync_parent_directory(parent)?;

            let reopened = Self::open(target)?;
            validate_backup_connection(&reopened.connection)?;
            drop(reopened);
            Ok(())
        })();

        if backup_result.is_err() {
            let _ = fs::remove_file(&temp_path);
            let _ = fs::remove_file(sqlite_sidecar_path(&temp_path, "-wal"));
            let _ = fs::remove_file(sqlite_sidecar_path(&temp_path, "-shm"));
        }
        backup_result
    }

    pub fn create_project(&mut self, root: StoredProjectRoot) -> Result<(), SqliteRepositoryError> {
        if root.root_revision.parent_revision_id.is_some()
            || root.root_conversation.project_id != root.project_id
            || root.root_conversation.head_revision_id != root.root_revision.id
            || root.root_revision.conversation_id != root.root_conversation.id
            || root.root_revision.snapshot_id != root.root_snapshot.id
        {
            return Err(SqliteRepositoryError::InvalidRoot);
        }
        let transaction = self.connection.transaction()?;
        if exists(
            &transaction,
            "SELECT 1 FROM projects WHERE id = ?1",
            root.project_id.as_str(),
        )? {
            return Err(SqliteRepositoryError::ProjectAlreadyExists(root.project_id));
        }
        transaction.execute(
            "INSERT INTO projects (id, root_revision_id) VALUES (?1, ?2)",
            params![root.project_id.as_str(), root.root_revision.id.as_str()],
        )?;
        insert_snapshot(&transaction, &root.root_snapshot)?;
        insert_revision(&transaction, &root.root_revision)?;
        insert_conversation(&transaction, &root.root_conversation)?;
        transaction.commit()?;
        Ok(())
    }

    pub fn insert_snapshot(
        &mut self,
        snapshot: &StoredSnapshot,
    ) -> Result<(), SqliteRepositoryError> {
        let transaction = self.connection.transaction()?;
        insert_snapshot(&transaction, snapshot)?;
        transaction.commit()?;
        Ok(())
    }

    /// Inserts an immutable child record and changes the current head as one
    /// transaction. A failed compare-and-swap rolls back the child insert.
    /// Call [`Self::append_revision_with_snapshot`] when the candidate
    /// snapshot has not already been stored, so that snapshot and revision
    /// persistence share the same rollback boundary.
    pub fn append_revision(
        &mut self,
        revision: &StoredRevision,
        expected_parent: &RevisionId,
    ) -> Result<(), SqliteRepositoryError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        append_revision_in_transaction(&transaction, revision, expected_parent)?;
        transaction.commit()?;
        Ok(())
    }

    /// Atomically stores a new snapshot, appends its immutable revision, and
    /// advances the conversation head. A failed CAS or validation rolls back
    /// both the candidate snapshot and revision, so callers do not leave an
    /// unreferenced snapshot behind while persisting a child.
    pub fn append_revision_with_snapshot(
        &mut self,
        revision: &StoredRevision,
        snapshot: &StoredSnapshot,
        expected_parent: &RevisionId,
    ) -> Result<(), SqliteRepositoryError> {
        if revision.snapshot_id != snapshot.id {
            return Err(SqliteRepositoryError::InvalidRevisionSnapshotBinding {
                revision_id: revision.id.clone(),
                revision_snapshot_id: revision.snapshot_id.clone(),
                supplied_snapshot_id: snapshot.id.clone(),
            });
        }

        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        insert_snapshot(&transaction, snapshot)?;
        append_revision_in_transaction(&transaction, revision, expected_parent)?;
        transaction.commit()?;
        Ok(())
    }

    /// Atomically stores a proposal's accepted child and marks the proposal
    /// accepted. The proposal row, snapshot, revision, and head advance share
    /// one rollback boundary.
    pub fn append_revision_with_snapshot_and_proposal(
        &mut self,
        revision: &StoredRevision,
        snapshot: &StoredSnapshot,
        expected_parent: &RevisionId,
        proposal_id: &ProposalId,
        provenance: Option<&AgentTurnProvenance>,
    ) -> Result<(), SqliteRepositoryError> {
        self.append_revision_with_snapshot_proposal_and_receipt(
            revision,
            snapshot,
            expected_parent,
            proposal_id,
            provenance,
            None,
        )
    }

    /// The accepted revision, head movement, proposal status, and request
    /// receipt share one commit. A receipt failure rolls back every mutation.
    pub fn append_revision_with_snapshot_proposal_and_operation_receipt(
        &mut self,
        revision: &StoredRevision,
        snapshot: &StoredSnapshot,
        expected_parent: &RevisionId,
        proposal_id: &ProposalId,
        provenance: Option<&AgentTurnProvenance>,
        receipt: &StoredOperationReceipt,
    ) -> Result<(), SqliteRepositoryError> {
        self.append_revision_with_snapshot_proposal_and_receipt(
            revision,
            snapshot,
            expected_parent,
            proposal_id,
            provenance,
            Some(receipt),
        )
    }

    fn append_revision_with_snapshot_proposal_and_receipt(
        &mut self,
        revision: &StoredRevision,
        snapshot: &StoredSnapshot,
        expected_parent: &RevisionId,
        proposal_id: &ProposalId,
        provenance: Option<&AgentTurnProvenance>,
        receipt: Option<&StoredOperationReceipt>,
    ) -> Result<(), SqliteRepositoryError> {
        if revision.snapshot_id != snapshot.id {
            return Err(SqliteRepositoryError::InvalidRevisionSnapshotBinding {
                revision_id: revision.id.clone(),
                revision_snapshot_id: revision.snapshot_id.clone(),
                supplied_snapshot_id: snapshot.id.clone(),
            });
        }
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let proposal = proposal_in_transaction(&transaction, proposal_id)?
            .ok_or_else(|| SqliteRepositoryError::UnknownProposal(proposal_id.clone()))?;
        if proposal.status != "pending"
            || proposal.conversation_id != revision.conversation_id
            || proposal.parent_revision_id != *expected_parent
            || proposal.proposed_revision_id != revision.id
        {
            return Err(SqliteRepositoryError::ConflictingProposal(
                proposal_id.clone(),
            ));
        }
        insert_snapshot(&transaction, snapshot)?;
        append_revision_in_transaction(&transaction, revision, expected_parent)?;
        let changed = transaction.execute(
            "UPDATE proposals SET status = 'accepted', accepted_revision_id = ?1, provider = ?2, model = ?3, turn_id = ?4 WHERE id = ?5 AND status = 'pending'",
            params![
                revision.id.as_str(),
                provenance.map(|value| value.provider.as_str()),
                provenance.map(|value| value.model.as_str()),
                provenance.map(|value| value.turn_id.as_str()),
                proposal_id.as_str(),
            ],
        )?;
        if changed != 1 {
            return Err(SqliteRepositoryError::UnknownProposal(proposal_id.clone()));
        }
        if let Some(receipt) = receipt {
            insert_operation_receipt(&transaction, receipt)?;
        }
        transaction.commit()?;
        Ok(())
    }

    pub fn create_conversation(
        &mut self,
        conversation: &StoredConversation,
    ) -> Result<(), SqliteRepositoryError> {
        let transaction = self.connection.transaction()?;
        if !exists(
            &transaction,
            "SELECT 1 FROM projects WHERE id = ?1",
            conversation.project_id.as_str(),
        )? {
            return Err(SqliteRepositoryError::UnknownProject(
                conversation.project_id.clone(),
            ));
        }
        if !exists(
            &transaction,
            "SELECT 1 FROM revisions WHERE id = ?1",
            conversation.head_revision_id.as_str(),
        )? {
            return Err(SqliteRepositoryError::UnknownRevision(
                conversation.head_revision_id.clone(),
            ));
        }
        if !revision_belongs_to_project(
            &transaction,
            &conversation.project_id,
            &conversation.head_revision_id,
        )? {
            return Err(SqliteRepositoryError::RevisionNotInProject {
                project_id: conversation.project_id.clone(),
                revision_id: conversation.head_revision_id.clone(),
            });
        }
        insert_conversation(&transaction, conversation)?;
        transaction.commit()?;
        Ok(())
    }

    pub fn insert_proposal(
        &mut self,
        proposal: &StoredProposal,
    ) -> Result<(), SqliteRepositoryError> {
        let transaction = self.connection.transaction()?;
        if !exists(
            &transaction,
            "SELECT 1 FROM projects WHERE id = ?1",
            proposal.project_id.as_str(),
        )? {
            return Err(SqliteRepositoryError::UnknownProject(
                proposal.project_id.clone(),
            ));
        }
        if !exists(
            &transaction,
            "SELECT 1 FROM conversations WHERE id = ?1",
            proposal.conversation_id.as_str(),
        )? {
            return Err(SqliteRepositoryError::UnknownConversation(
                proposal.conversation_id.clone(),
            ));
        }
        let result = transaction.execute(
            "INSERT INTO proposals (id, project_id, conversation_id, parent_revision_id, proposed_revision_id, patch_json, status, accepted_revision_id, provider, model, turn_id, provenance_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                proposal.id.as_str(),
                proposal.project_id.as_str(),
                proposal.conversation_id.as_str(),
                proposal.parent_revision_id.as_str(),
                proposal.proposed_revision_id.as_str(),
                proposal.patch_json,
                proposal.status,
                proposal.accepted_revision_id.as_ref().map(RevisionId::as_str),
                proposal.agent_provenance.as_ref().map(|value| value.provider.as_str()),
                proposal.agent_provenance.as_ref().map(|value| value.model.as_str()),
                proposal.agent_provenance.as_ref().map(|value| value.turn_id.as_str()),
                proposal.agent_provenance.as_ref().map(|value| serde_json::to_string(value).expect("agent provenance is serializable")),
            ],
        );
        match result {
            Ok(_) => {
                transaction.commit()?;
                Ok(())
            }
            Err(rusqlite::Error::SqliteFailure(error, _))
                if error.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_PRIMARYKEY =>
            {
                Err(SqliteRepositoryError::UnknownProposal(proposal.id.clone()))
            }
            Err(error) => Err(error.into()),
        }
    }

    /// Inserts a pending proposal only while the conversation is still at the
    /// exact parent used to validate its patch. An identical proposal is an
    /// idempotent replay; an identifier collision with different content is
    /// rejected.
    pub fn insert_proposal_at_expected_head(
        &mut self,
        proposal: &StoredProposal,
        expected_head: &RevisionId,
    ) -> Result<(), SqliteRepositoryError> {
        self.insert_proposal_at_expected_head_with_receipt(proposal, expected_head, None)
    }

    /// Proposal creation and its request receipt share one commit. This is the
    /// durable idempotency boundary used by transport adapters.
    pub fn insert_proposal_at_expected_head_and_operation_receipt(
        &mut self,
        proposal: &StoredProposal,
        expected_head: &RevisionId,
        receipt: &StoredOperationReceipt,
    ) -> Result<(), SqliteRepositoryError> {
        self.insert_proposal_at_expected_head_with_receipt(proposal, expected_head, Some(receipt))
    }

    fn insert_proposal_at_expected_head_with_receipt(
        &mut self,
        proposal: &StoredProposal,
        expected_head: &RevisionId,
        receipt: Option<&StoredOperationReceipt>,
    ) -> Result<(), SqliteRepositoryError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if proposal.parent_revision_id != *expected_head {
            return Err(SqliteRepositoryError::InvalidRevisionParentBinding {
                revision_id: proposal.proposed_revision_id.clone(),
                expected_parent_id: expected_head.clone(),
                actual_parent_id: Some(proposal.parent_revision_id.clone()),
            });
        }
        let actual_head = head_in(&transaction, &proposal.conversation_id)?;
        if actual_head != *expected_head {
            return Err(SqliteRepositoryError::ExpectedHeadConflict {
                conversation_id: proposal.conversation_id.clone(),
                expected_revision_id: expected_head.clone(),
                actual_revision_id: actual_head,
            });
        }
        let project_id = transaction
            .query_row(
                "SELECT project_id FROM conversations WHERE id = ?1",
                [proposal.conversation_id.as_str()],
                |row| row.get::<_, String>(0),
            )
            .map(ProjectId::from)?;
        if project_id != proposal.project_id {
            return Err(SqliteRepositoryError::UnknownProject(
                proposal.project_id.clone(),
            ));
        }

        let existing = proposal_in_transaction(&transaction, &proposal.id)?;
        if let Some(existing) = existing {
            if existing == *proposal {
                if let Some(receipt) = receipt {
                    insert_operation_receipt(&transaction, receipt)?;
                }
                transaction.commit()?;
                return Ok(());
            }
            return Err(SqliteRepositoryError::ConflictingProposal(
                proposal.id.clone(),
            ));
        }
        transaction.execute(
            "INSERT INTO proposals (id, project_id, conversation_id, parent_revision_id, proposed_revision_id, patch_json, status, accepted_revision_id, provider, model, turn_id, provenance_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                proposal.id.as_str(),
                proposal.project_id.as_str(),
                proposal.conversation_id.as_str(),
                proposal.parent_revision_id.as_str(),
                proposal.proposed_revision_id.as_str(),
                proposal.patch_json,
                proposal.status,
                proposal.accepted_revision_id.as_ref().map(RevisionId::as_str),
                proposal.agent_provenance.as_ref().map(|value| value.provider.as_str()),
                proposal.agent_provenance.as_ref().map(|value| value.model.as_str()),
                proposal.agent_provenance.as_ref().map(|value| value.turn_id.as_str()),
                proposal.agent_provenance.as_ref().map(|value| serde_json::to_string(value).expect("agent provenance is serializable")),
            ],
        )?;
        if let Some(receipt) = receipt {
            insert_operation_receipt(&transaction, receipt)?;
        }
        transaction.commit()?;
        Ok(())
    }

    pub fn proposal(&self, id: &ProposalId) -> Result<StoredProposal, SqliteRepositoryError> {
        proposal_in_connection(&self.connection, id)?
            .ok_or_else(|| SqliteRepositoryError::UnknownProposal(id.clone()))
    }

    pub fn operation_receipt(
        &self,
        request_id: &str,
    ) -> Result<Option<StoredOperationReceipt>, SqliteRepositoryError> {
        self.connection
            .query_row(
                "SELECT request_id, request_json, response_json FROM operation_receipts WHERE request_id = ?1",
                [request_id],
                |row| {
                    Ok(StoredOperationReceipt {
                        request_id: row.get(0)?,
                        request_json: row.get(1)?,
                        response_json: row.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn store_operation_receipt(
        &mut self,
        receipt: &StoredOperationReceipt,
    ) -> Result<(), SqliteRepositoryError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        insert_operation_receipt(&transaction, receipt)?;
        transaction.commit()?;
        Ok(())
    }

    pub fn reject_proposal(
        &mut self,
        proposal_id: &ProposalId,
    ) -> Result<(), SqliteRepositoryError> {
        let transaction = self.connection.transaction()?;
        let changed = transaction.execute(
            "UPDATE proposals SET status = 'rejected', accepted_revision_id = NULL WHERE id = ?1 AND status = 'pending'",
            [proposal_id.as_str()],
        )?;
        if changed != 1 {
            return Err(SqliteRepositoryError::UnknownProposal(proposal_id.clone()));
        }
        transaction.commit()?;
        Ok(())
    }

    /// Rejects a pending proposal against its exact conversation head and
    /// stores the operation receipt in the same transaction.
    pub fn reject_proposal_at_expected_head_and_operation_receipt(
        &mut self,
        proposal_id: &ProposalId,
        conversation_id: &ConversationId,
        expected_head: &RevisionId,
        receipt: &StoredOperationReceipt,
    ) -> Result<(), SqliteRepositoryError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let actual_head = head_in(&transaction, conversation_id)?;
        if actual_head != *expected_head {
            return Err(SqliteRepositoryError::ExpectedHeadConflict {
                conversation_id: conversation_id.clone(),
                expected_revision_id: expected_head.clone(),
                actual_revision_id: actual_head,
            });
        }
        let proposal = proposal_in_transaction(&transaction, proposal_id)?
            .ok_or_else(|| SqliteRepositoryError::UnknownProposal(proposal_id.clone()))?;
        if proposal.conversation_id != *conversation_id
            || proposal.parent_revision_id != *expected_head
            || proposal.status != "pending"
        {
            return Err(SqliteRepositoryError::ConflictingProposal(
                proposal_id.clone(),
            ));
        }
        let changed = transaction.execute(
            "UPDATE proposals SET status = 'rejected', accepted_revision_id = NULL WHERE id = ?1 AND status = 'pending'",
            [proposal_id.as_str()],
        )?;
        if changed != 1 {
            return Err(SqliteRepositoryError::ConflictingProposal(
                proposal_id.clone(),
            ));
        }
        insert_operation_receipt(&transaction, receipt)?;
        transaction.commit()?;
        Ok(())
    }

    pub fn project_proposals(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<StoredProposal>, SqliteRepositoryError> {
        if !self.project_exists(project_id)? {
            return Err(SqliteRepositoryError::UnknownProject(project_id.clone()));
        }
        let mut statement = self.connection.prepare(
            "SELECT id, project_id, conversation_id, parent_revision_id, proposed_revision_id, patch_json, status, accepted_revision_id, provider, model, turn_id, provenance_json FROM proposals WHERE project_id = ?1 ORDER BY id",
        )?;
        let rows = statement.query_map([project_id.as_str()], |row| {
            Ok(StoredProposal {
                id: ProposalId::from(row.get::<_, String>(0)?.as_str()),
                project_id: ProjectId::from(row.get::<_, String>(1)?),
                conversation_id: ConversationId::from(row.get::<_, String>(2)?),
                parent_revision_id: RevisionId::from(row.get::<_, String>(3)?),
                proposed_revision_id: RevisionId::from(row.get::<_, String>(4)?),
                patch_json: row.get(5)?,
                status: row.get(6)?,
                accepted_revision_id: row.get::<_, Option<String>>(7)?.map(RevisionId::from),
                agent_provenance: row
                    .get::<_, Option<String>>(11)?
                    .and_then(|json| serde_json::from_str(&json).ok())
                    .or_else(|| {
                        match (
                            row.get::<_, Option<String>>(8).ok().flatten(),
                            row.get::<_, Option<String>>(9).ok().flatten(),
                            row.get::<_, Option<String>>(10).ok().flatten(),
                        ) {
                            (Some(provider), Some(model), Some(turn_id)) => {
                                Some(AgentTurnProvenance {
                                    provider,
                                    model,
                                    turn_id,
                                    ..Default::default()
                                })
                            }
                            _ => None,
                        }
                    }),
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn conversation(
        &self,
        id: &ConversationId,
    ) -> Result<StoredConversation, SqliteRepositoryError> {
        self.connection.query_row(
            "SELECT id, project_id, purpose, origin_json, head_revision_id FROM conversations WHERE id = ?1", [id.as_str()], |row| Ok(StoredConversation { id: ConversationId::from(row.get::<_, String>(0)?), project_id: ProjectId::from(row.get::<_, String>(1)?), purpose: row.get(2)?, origin_json: row.get(3)?, head_revision_id: RevisionId::from(row.get::<_, String>(4)?) }),
        ).optional()?.ok_or_else(|| SqliteRepositoryError::UnknownConversation(id.clone()))
    }

    /// Updates only the opaque conversation-facing origin payload. Immutable
    /// revisions and snapshots are never rewritten by this operation.
    pub fn update_conversation_origin(
        &mut self,
        id: &ConversationId,
        origin_json: &str,
    ) -> Result<(), SqliteRepositoryError> {
        let changed = self.connection.execute(
            "UPDATE conversations SET origin_json = ?1 WHERE id = ?2",
            params![origin_json, id.as_str()],
        )?;
        if changed != 1 {
            return Err(SqliteRepositoryError::UnknownConversation(id.clone()));
        }
        Ok(())
    }

    /// Reconstructs the project's durable root aggregate without retaining a
    /// mutable project-file handle. The returned records are still immutable
    /// value projections; callers can use them to rebuild an in-memory
    /// service after process restart.
    pub fn project_root(
        &self,
        project_id: &ProjectId,
    ) -> Result<StoredProjectRoot, SqliteRepositoryError> {
        let root_revision_id = self
            .connection
            .query_row(
                "SELECT root_revision_id FROM projects WHERE id = ?1",
                [project_id.as_str()],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .map(RevisionId::from)
            .ok_or_else(|| SqliteRepositoryError::UnknownProject(project_id.clone()))?;
        let root_revision = self.revision(&root_revision_id)?;
        let root_conversation = self.conversation(&root_revision.conversation_id)?;
        let root_snapshot = self.snapshot(&root_revision.snapshot_id)?;
        if root_revision.parent_revision_id.is_some() || root_conversation.project_id != *project_id
        {
            return Err(SqliteRepositoryError::InvalidRoot);
        }
        Ok(StoredProjectRoot {
            project_id: project_id.clone(),
            root_conversation,
            root_revision,
            root_snapshot,
        })
    }

    /// Returns every conversation belonging to a project in stable ID order.
    pub fn project_conversations(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<StoredConversation>, SqliteRepositoryError> {
        if !self.project_exists(project_id)? {
            return Err(SqliteRepositoryError::UnknownProject(project_id.clone()));
        }
        let mut statement = self.connection.prepare(
            "SELECT id, project_id, purpose, origin_json, head_revision_id
             FROM conversations WHERE project_id = ?1 ORDER BY id",
        )?;
        let rows = statement.query_map([project_id.as_str()], |row| {
            Ok(StoredConversation {
                id: ConversationId::from(row.get::<_, String>(0)?),
                project_id: ProjectId::from(row.get::<_, String>(1)?),
                purpose: row.get(2)?,
                origin_json: row.get(3)?,
                head_revision_id: RevisionId::from(row.get::<_, String>(4)?),
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Returns every immutable revision reachable from a project's root,
    /// including branches owned by forked conversations.
    pub fn project_revisions(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<StoredRevision>, SqliteRepositoryError> {
        if !self.project_exists(project_id)? {
            return Err(SqliteRepositoryError::UnknownProject(project_id.clone()));
        }
        let mut statement = self.connection.prepare(
            "WITH RECURSIVE project_revisions(id) AS (
                 SELECT root_revision_id FROM projects WHERE id = ?1
                 UNION
                 SELECT revisions.id
                 FROM revisions
                 JOIN project_revisions ON revisions.parent_revision_id = project_revisions.id
             )
             SELECT revisions.id, revisions.snapshot_id, revisions.parent_revision_id,
                    revisions.conversation_id, revisions.metadata_json
             FROM revisions JOIN project_revisions ON project_revisions.id = revisions.id
             ORDER BY revisions.id",
        )?;
        let rows = statement.query_map([project_id.as_str()], |row| {
            Ok(StoredRevision {
                id: RevisionId::from(row.get::<_, String>(0)?),
                snapshot_id: SnapshotId::from(row.get::<_, String>(1)?),
                parent_revision_id: row.get::<_, Option<String>>(2)?.map(RevisionId::from),
                conversation_id: ConversationId::from(row.get::<_, String>(3)?),
                metadata_json: row.get(4)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Returns immutable analysis evidence whose authored snapshot belongs to
    /// the project's reachable revision graph. Evidence remains project
    /// isolated even though the compact schema stores it by snapshot identity.
    pub fn project_evidence(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<StoredEvidence>, SqliteRepositoryError> {
        if !self.project_exists(project_id)? {
            return Err(SqliteRepositoryError::UnknownProject(project_id.clone()));
        }
        let mut statement = self.connection.prepare(
            "WITH RECURSIVE project_revisions(id) AS (
                 SELECT root_revision_id FROM projects WHERE id = ?1
                 UNION
                 SELECT revisions.id
                 FROM revisions
                 JOIN project_revisions ON revisions.parent_revision_id = project_revisions.id
             )
             SELECT DISTINCT e.id, e.authored_snapshot_id, e.resolved_snapshot_id,
                    e.manifest_json, e.blob_ref
             FROM evidence e
             JOIN revisions r ON r.snapshot_id = e.authored_snapshot_id
             JOIN project_revisions p ON p.id = r.id
             ORDER BY e.id",
        )?;
        let rows = statement.query_map([project_id.as_str()], |row| {
            Ok(StoredEvidence {
                id: EvidenceId::from(row.get::<_, String>(0)?),
                authored_snapshot_id: SnapshotId::from(row.get::<_, String>(1)?),
                resolved_snapshot_id: row.get::<_, Option<String>>(2)?.map(SnapshotId::from),
                manifest_json: row.get(3)?,
                blob_ref: row.get(4)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn revision(&self, id: &RevisionId) -> Result<StoredRevision, SqliteRepositoryError> {
        self.connection.query_row(
            "SELECT id, snapshot_id, parent_revision_id, conversation_id, metadata_json FROM revisions WHERE id = ?1", [id.as_str()], |row| Ok(StoredRevision { id: RevisionId::from(row.get::<_, String>(0)?), snapshot_id: SnapshotId::from(row.get::<_, String>(1)?), parent_revision_id: row.get::<_, Option<String>>(2)?.map(RevisionId::from), conversation_id: ConversationId::from(row.get::<_, String>(3)?), metadata_json: row.get(4)? }),
        ).optional()?.ok_or_else(|| SqliteRepositoryError::UnknownRevision(id.clone()))
    }

    pub fn snapshot(&self, id: &SnapshotId) -> Result<StoredSnapshot, SqliteRepositoryError> {
        let snapshot = self
            .connection
            .query_row(
                "SELECT id, format_version, canonical_bytes FROM snapshots WHERE id = ?1",
                [id.as_str()],
                |row| {
                    Ok(StoredSnapshot {
                        id: SnapshotId::from(row.get::<_, String>(0)?),
                        format_version: row.get(1)?,
                        canonical_bytes: row.get(2)?,
                    })
                },
            )
            .optional()?
            .ok_or_else(|| SqliteRepositoryError::UnknownSnapshot(id.clone()))?;
        validate_stored_snapshot(&snapshot)?;
        Ok(snapshot)
    }

    /// Decodes and validates a persisted authored snapshot into the typed
    /// model used by the revision domain. This is the restart boundary for
    /// consumers that must rebuild an in-memory repository from SQLite.
    pub fn hydrate_snapshot(
        &self,
        id: &SnapshotId,
    ) -> Result<ModelSnapshot, SqliteRepositoryError> {
        let snapshot = self.snapshot(id)?;
        ModelSnapshot::from_canonical(
            snapshot.id.clone(),
            CanonicalFormatVersion::new(snapshot.format_version),
            &snapshot.canonical_bytes,
        )
        .map_err(|error| SqliteRepositoryError::InvalidSnapshotPayload {
            snapshot_id: id.clone(),
            reason: error.to_string(),
        })
    }

    fn project_exists(&self, project_id: &ProjectId) -> Result<bool, SqliteRepositoryError> {
        Ok(self
            .connection
            .query_row(
                "SELECT 1 FROM projects WHERE id = ?1",
                [project_id.as_str()],
                |_| Ok(()),
            )
            .optional()?
            .is_some())
    }

    pub fn history(
        &self,
        conversation_id: &ConversationId,
    ) -> Result<Vec<StoredRevision>, SqliteRepositoryError> {
        self.conversation(conversation_id)?;
        let mut statement = self.connection.prepare("WITH RECURSIVE lineage(id, snapshot_id, parent_revision_id, conversation_id, metadata_json) AS ( SELECT r.id, r.snapshot_id, r.parent_revision_id, r.conversation_id, r.metadata_json FROM revisions r JOIN conversations c ON c.head_revision_id = r.id WHERE c.id = ?1 UNION ALL SELECT r.id, r.snapshot_id, r.parent_revision_id, r.conversation_id, r.metadata_json FROM revisions r JOIN lineage l ON r.id = l.parent_revision_id ) SELECT id, snapshot_id, parent_revision_id, conversation_id, metadata_json FROM lineage")?;
        let rows = statement.query_map([conversation_id.as_str()], |row| {
            Ok(StoredRevision {
                id: RevisionId::from(row.get::<_, String>(0)?),
                snapshot_id: SnapshotId::from(row.get::<_, String>(1)?),
                parent_revision_id: row.get::<_, Option<String>>(2)?.map(RevisionId::from),
                conversation_id: ConversationId::from(row.get::<_, String>(3)?),
                metadata_json: row.get(4)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn attach_evidence(
        &mut self,
        evidence: &StoredEvidence,
    ) -> Result<(), SqliteRepositoryError> {
        let transaction = self.connection.transaction()?;
        insert_evidence(&transaction, evidence)?;
        transaction.commit()?;
        Ok(())
    }

    /// Stores a resolved snapshot and its evidence manifest in one transaction.
    pub fn attach_evidence_with_snapshot(
        &mut self,
        evidence: &StoredEvidence,
        resolved_snapshot: Option<&StoredSnapshot>,
    ) -> Result<(), SqliteRepositoryError> {
        let transaction = self.connection.transaction()?;
        if let Some(snapshot) = resolved_snapshot {
            insert_snapshot(&transaction, snapshot)?;
        }
        insert_evidence(&transaction, evidence)?;
        transaction.commit()?;
        Ok(())
    }

    /// Publishes resolved state, immutable evidence, and the request receipt
    /// in one transaction. Unsupported and failed evidence may omit the
    /// resolved snapshot but still receives the same atomic publication.
    pub fn attach_evidence_with_snapshot_and_operation_receipt(
        &mut self,
        evidence: &StoredEvidence,
        resolved_snapshot: Option<&StoredSnapshot>,
        receipt: &StoredOperationReceipt,
    ) -> Result<(), SqliteRepositoryError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let Some(snapshot) = resolved_snapshot {
            insert_snapshot(&transaction, snapshot)?;
        }
        insert_evidence(&transaction, evidence)?;
        insert_operation_receipt(&transaction, receipt)?;
        transaction.commit()?;
        Ok(())
    }

    pub fn evidence(&self, id: &EvidenceId) -> Result<StoredEvidence, SqliteRepositoryError> {
        self.connection.query_row("SELECT id, authored_snapshot_id, resolved_snapshot_id, manifest_json, blob_ref FROM evidence WHERE id = ?1", [id.as_str()], |row| Ok(StoredEvidence { id: EvidenceId::from(row.get::<_, String>(0)?), authored_snapshot_id: SnapshotId::from(row.get::<_, String>(1)?), resolved_snapshot_id: row.get::<_, Option<String>>(2)?.map(SnapshotId::from), manifest_json: row.get(3)?, blob_ref: row.get(4)? })).optional()?.ok_or_else(|| SqliteRepositoryError::UnknownEvidence(id.clone()))
    }

    /// Exact source identity is the first, durable staleness signal. The S5
    /// domain service applies dependency/category rules after loading its
    /// typed evidence manifest.
    pub fn evidence_is_stale_for_revision(
        &self,
        evidence_id: &EvidenceId,
        revision_id: &RevisionId,
    ) -> Result<bool, SqliteRepositoryError> {
        Ok(self.evidence(evidence_id)?.authored_snapshot_id
            != self.revision(revision_id)?.snapshot_id)
    }

    pub fn attach_artefact(
        &mut self,
        artefact: &StoredArtefact,
    ) -> Result<(), SqliteRepositoryError> {
        let transaction = self.connection.transaction()?;
        if !exists(
            &transaction,
            "SELECT 1 FROM snapshots WHERE id = ?1",
            artefact.source_snapshot_id.as_str(),
        )? {
            return Err(SqliteRepositoryError::UnknownSnapshot(
                artefact.source_snapshot_id.clone(),
            ));
        }
        if let Some(evidence_id) = &artefact.source_evidence_id {
            let source: Option<String> = transaction
                .query_row(
                    "SELECT authored_snapshot_id FROM evidence WHERE id = ?1",
                    [evidence_id.as_str()],
                    |row| row.get(0),
                )
                .optional()?;
            match source {
                None => return Err(SqliteRepositoryError::UnknownEvidence(evidence_id.clone())),
                Some(source) if source != artefact.source_snapshot_id.as_str() => {
                    return Err(SqliteRepositoryError::InvalidArtefactBinding {
                        artefact_id: artefact.id.clone(),
                        snapshot_id: artefact.source_snapshot_id.clone(),
                    });
                }
                _ => {}
            }
        }
        let result = transaction.execute("INSERT INTO artefacts (id, kind, source_snapshot_id, source_evidence_id, manifest_json, blob_ref) VALUES (?1, ?2, ?3, ?4, ?5, ?6)", params![artefact.id.as_str(), artefact.kind, artefact.source_snapshot_id.as_str(), artefact.source_evidence_id.as_ref().map(EvidenceId::as_str), artefact.manifest_json, artefact.blob_ref]);
        match result {
            Ok(_) => {
                transaction.commit()?;
                Ok(())
            }
            Err(rusqlite::Error::SqliteFailure(error, _))
                if error.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_PRIMARYKEY =>
            {
                Err(SqliteRepositoryError::DuplicateArtefact(
                    artefact.id.clone(),
                ))
            }
            Err(error) => Err(error.into()),
        }
    }

    pub fn artefact(&self, id: &ArtefactId) -> Result<StoredArtefact, SqliteRepositoryError> {
        self.connection
            .query_row(
                "SELECT id, kind, source_snapshot_id, source_evidence_id, manifest_json, blob_ref
                 FROM artefacts WHERE id = ?1",
                [id.as_str()],
                |row| {
                    Ok(StoredArtefact {
                        id: ArtefactId::from(row.get::<_, String>(0)?),
                        kind: row.get(1)?,
                        source_snapshot_id: SnapshotId::from(row.get::<_, String>(2)?),
                        source_evidence_id: row.get::<_, Option<String>>(3)?.map(EvidenceId::from),
                        manifest_json: row.get(4)?,
                        blob_ref: row.get(5)?,
                    })
                },
            )
            .optional()?
            .ok_or_else(|| SqliteRepositoryError::UnknownArtefact(id.clone()))
    }
}

fn migrate(connection: &mut Connection) -> Result<(), rusqlite::Error> {
    connection.execute_batch("CREATE TABLE IF NOT EXISTS schema_migrations (version INTEGER PRIMARY KEY); BEGIN; CREATE TABLE IF NOT EXISTS projects (id TEXT PRIMARY KEY NOT NULL, root_revision_id TEXT NOT NULL); CREATE TABLE IF NOT EXISTS snapshots (id TEXT PRIMARY KEY NOT NULL, format_version TEXT NOT NULL, canonical_bytes BLOB NOT NULL); CREATE TABLE IF NOT EXISTS revisions (id TEXT PRIMARY KEY NOT NULL, snapshot_id TEXT NOT NULL REFERENCES snapshots(id), parent_revision_id TEXT REFERENCES revisions(id), conversation_id TEXT NOT NULL, metadata_json TEXT NOT NULL); CREATE INDEX IF NOT EXISTS revisions_parent_idx ON revisions(parent_revision_id); CREATE TABLE IF NOT EXISTS conversations (id TEXT PRIMARY KEY NOT NULL, project_id TEXT NOT NULL REFERENCES projects(id), purpose TEXT NOT NULL, origin_json TEXT NOT NULL, head_revision_id TEXT NOT NULL REFERENCES revisions(id)); CREATE INDEX IF NOT EXISTS conversations_project_idx ON conversations(project_id); CREATE TABLE IF NOT EXISTS evidence (id TEXT PRIMARY KEY NOT NULL, authored_snapshot_id TEXT NOT NULL REFERENCES snapshots(id), resolved_snapshot_id TEXT REFERENCES snapshots(id), manifest_json TEXT NOT NULL, blob_ref TEXT); CREATE TABLE IF NOT EXISTS artefacts (id TEXT PRIMARY KEY NOT NULL, kind TEXT NOT NULL, source_snapshot_id TEXT NOT NULL REFERENCES snapshots(id), source_evidence_id TEXT REFERENCES evidence(id), manifest_json TEXT NOT NULL, blob_ref TEXT); INSERT OR IGNORE INTO schema_migrations(version) VALUES (1); COMMIT;")?;
    let mut version: i64 =
        connection.query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
            row.get(0)
        })?;
    if version < 2 {
        connection.execute_batch(
            "BEGIN;
             CREATE TABLE IF NOT EXISTS proposals (
                 id TEXT PRIMARY KEY NOT NULL,
                 project_id TEXT NOT NULL REFERENCES projects(id),
                 conversation_id TEXT NOT NULL REFERENCES conversations(id),
                 parent_revision_id TEXT NOT NULL REFERENCES revisions(id),
                 proposed_revision_id TEXT NOT NULL,
                 patch_json TEXT NOT NULL,
                 status TEXT NOT NULL,
                 accepted_revision_id TEXT,
                 provider TEXT,
                 model TEXT,
                 turn_id TEXT
             );
             CREATE INDEX IF NOT EXISTS proposals_project_idx ON proposals(project_id);
             INSERT OR IGNORE INTO schema_migrations(version) VALUES (2);
             COMMIT;",
        )?;
        version = 2;
    }
    if version < 3 {
        connection.execute_batch(
            "BEGIN;
             CREATE TABLE IF NOT EXISTS operation_receipts (
                 request_id TEXT PRIMARY KEY NOT NULL,
                 request_json TEXT NOT NULL,
                 response_json TEXT NOT NULL
             );
             INSERT OR IGNORE INTO schema_migrations(version) VALUES (3);
             COMMIT;",
        )?;
        version = 3;
    }
    if version < 4 {
        connection.execute_batch(
            "BEGIN;
             ALTER TABLE proposals ADD COLUMN provenance_json TEXT;
             INSERT OR IGNORE INTO schema_migrations(version) VALUES (4);
             COMMIT;",
        )?;
        version = 4;
    }
    debug_assert_eq!(version, SCHEMA_VERSION);
    Ok(())
}

fn sqlite_sidecar_path(path: &Path, suffix: &str) -> PathBuf {
    let mut value = path.as_os_str().to_os_string();
    value.push(suffix);
    PathBuf::from(value)
}

#[cfg(unix)]
fn sync_parent_directory(parent: &Path) -> Result<(), std::io::Error> {
    fs::File::open(parent)?.sync_all()
}

#[cfg(not(unix))]
fn sync_parent_directory(_parent: &Path) -> Result<(), std::io::Error> {
    Ok(())
}

fn validate_backup_connection(connection: &Connection) -> Result<(), SqliteRepositoryError> {
    let integrity =
        connection.query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))?;
    if integrity != "ok" {
        return Err(SqliteRepositoryError::BackupValidation(format!(
            "integrity_check returned `{integrity}`"
        )));
    }
    let foreign_key_violation: Option<(String, i64)> = connection
        .query_row("PRAGMA foreign_key_check", [], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .optional()?;
    if let Some((table, row_id)) = foreign_key_violation {
        return Err(SqliteRepositoryError::BackupValidation(format!(
            "foreign key violation in table `{table}` row {row_id}"
        )));
    }
    Ok(())
}

fn proposal_in_connection(
    connection: &Connection,
    id: &ProposalId,
) -> Result<Option<StoredProposal>, SqliteRepositoryError> {
    connection
        .query_row(
            "SELECT id, project_id, conversation_id, parent_revision_id, proposed_revision_id, patch_json, status, accepted_revision_id, provider, model, turn_id, provenance_json FROM proposals WHERE id = ?1",
            [id.as_str()],
            proposal_from_row,
        )
        .optional()
        .map_err(Into::into)
}

fn proposal_in_transaction(
    transaction: &Transaction<'_>,
    id: &ProposalId,
) -> Result<Option<StoredProposal>, SqliteRepositoryError> {
    transaction
        .query_row(
            "SELECT id, project_id, conversation_id, parent_revision_id, proposed_revision_id, patch_json, status, accepted_revision_id, provider, model, turn_id, provenance_json FROM proposals WHERE id = ?1",
            [id.as_str()],
            proposal_from_row,
        )
        .optional()
        .map_err(Into::into)
}

fn proposal_from_row(row: &rusqlite::Row<'_>) -> Result<StoredProposal, rusqlite::Error> {
    let id = row.get::<_, String>(0)?;
    Ok(StoredProposal {
        id: ProposalId::from(id.as_str()),
        project_id: ProjectId::from(row.get::<_, String>(1)?),
        conversation_id: ConversationId::from(row.get::<_, String>(2)?),
        parent_revision_id: RevisionId::from(row.get::<_, String>(3)?),
        proposed_revision_id: RevisionId::from(row.get::<_, String>(4)?),
        patch_json: row.get(5)?,
        status: row.get(6)?,
        accepted_revision_id: row.get::<_, Option<String>>(7)?.map(RevisionId::from),
        agent_provenance: row
            .get::<_, Option<String>>(11)?
            .and_then(|json| serde_json::from_str(&json).ok())
            .or_else(|| {
                match (
                    row.get::<_, Option<String>>(8).ok().flatten(),
                    row.get::<_, Option<String>>(9).ok().flatten(),
                    row.get::<_, Option<String>>(10).ok().flatten(),
                ) {
                    (Some(provider), Some(model), Some(turn_id)) => Some(AgentTurnProvenance {
                        provider,
                        model,
                        turn_id,
                        ..Default::default()
                    }),
                    _ => None,
                }
            }),
    })
}

fn insert_operation_receipt(
    transaction: &Transaction<'_>,
    receipt: &StoredOperationReceipt,
) -> Result<(), SqliteRepositoryError> {
    let existing = transaction
        .query_row(
            "SELECT request_id, request_json, response_json FROM operation_receipts WHERE request_id = ?1",
            [receipt.request_id.as_str()],
            |row| {
                Ok(StoredOperationReceipt {
                    request_id: row.get(0)?,
                    request_json: row.get(1)?,
                    response_json: row.get(2)?,
                })
            },
        )
        .optional()?;
    if let Some(existing) = existing {
        if existing == *receipt {
            return Ok(());
        }
        return Err(SqliteRepositoryError::ConflictingOperationRequest(
            receipt.request_id.clone(),
        ));
    }
    transaction.execute(
        "INSERT INTO operation_receipts (request_id, request_json, response_json) VALUES (?1, ?2, ?3)",
        params![receipt.request_id, receipt.request_json, receipt.response_json],
    )?;
    Ok(())
}

fn exists(transaction: &Transaction<'_>, sql: &str, value: &str) -> Result<bool, rusqlite::Error> {
    Ok(transaction
        .query_row(sql, [value], |_| Ok(()))
        .optional()?
        .is_some())
}

fn insert_snapshot(
    transaction: &Transaction<'_>,
    snapshot: &StoredSnapshot,
) -> Result<(), SqliteRepositoryError> {
    validate_stored_snapshot(snapshot)?;
    let existing: Option<(String, Vec<u8>)> = transaction
        .query_row(
            "SELECT format_version, canonical_bytes FROM snapshots WHERE id = ?1",
            [snapshot.id.as_str()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    match existing {
        Some((format, bytes))
            if format == snapshot.format_version && bytes == snapshot.canonical_bytes =>
        {
            Ok(())
        }
        Some(_) => Err(SqliteRepositoryError::ImmutableSnapshotConflict(
            snapshot.id.clone(),
        )),
        None => {
            transaction.execute(
                "INSERT INTO snapshots (id, format_version, canonical_bytes) VALUES (?1, ?2, ?3)",
                params![
                    snapshot.id.as_str(),
                    snapshot.format_version,
                    snapshot.canonical_bytes
                ],
            )?;
            Ok(())
        }
    }
}

fn insert_revision(
    transaction: &Transaction<'_>,
    revision: &StoredRevision,
) -> Result<(), SqliteRepositoryError> {
    let result = transaction.execute("INSERT INTO revisions (id, snapshot_id, parent_revision_id, conversation_id, metadata_json) VALUES (?1, ?2, ?3, ?4, ?5)", params![revision.id.as_str(), revision.snapshot_id.as_str(), revision.parent_revision_id.as_ref().map(RevisionId::as_str), revision.conversation_id.as_str(), revision.metadata_json]);
    match result {
        Ok(_) => Ok(()),
        Err(rusqlite::Error::SqliteFailure(error, _))
            if error.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_PRIMARYKEY =>
        {
            Err(SqliteRepositoryError::DuplicateRevision(
                revision.id.clone(),
            ))
        }
        Err(error) => Err(error.into()),
    }
}

fn insert_evidence(
    transaction: &Transaction<'_>,
    evidence: &StoredEvidence,
) -> Result<(), SqliteRepositoryError> {
    if !exists(
        transaction,
        "SELECT 1 FROM snapshots WHERE id = ?1",
        evidence.authored_snapshot_id.as_str(),
    )? {
        return Err(SqliteRepositoryError::UnknownSnapshot(
            evidence.authored_snapshot_id.clone(),
        ));
    }
    if let Some(resolved) = &evidence.resolved_snapshot_id {
        if !exists(
            transaction,
            "SELECT 1 FROM snapshots WHERE id = ?1",
            resolved.as_str(),
        )? {
            return Err(SqliteRepositoryError::UnknownSnapshot(resolved.clone()));
        }
    }
    let result = transaction.execute(
        "INSERT INTO evidence (id, authored_snapshot_id, resolved_snapshot_id, manifest_json, blob_ref) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            evidence.id.as_str(),
            evidence.authored_snapshot_id.as_str(),
            evidence.resolved_snapshot_id.as_ref().map(SnapshotId::as_str),
            evidence.manifest_json,
            evidence.blob_ref,
        ],
    );
    match result {
        Ok(_) => Ok(()),
        Err(rusqlite::Error::SqliteFailure(error, _))
            if error.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_PRIMARYKEY =>
        {
            Err(SqliteRepositoryError::DuplicateEvidence(
                evidence.id.clone(),
            ))
        }
        Err(error) => Err(error.into()),
    }
}

fn insert_conversation(
    transaction: &Transaction<'_>,
    conversation: &StoredConversation,
) -> Result<(), SqliteRepositoryError> {
    let result = transaction.execute("INSERT INTO conversations (id, project_id, purpose, origin_json, head_revision_id) VALUES (?1, ?2, ?3, ?4, ?5)", params![conversation.id.as_str(), conversation.project_id.as_str(), conversation.purpose, conversation.origin_json, conversation.head_revision_id.as_str()]);
    match result {
        Ok(_) => Ok(()),
        Err(rusqlite::Error::SqliteFailure(error, _))
            if error.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_PRIMARYKEY =>
        {
            Err(SqliteRepositoryError::DuplicateConversation(
                conversation.id.clone(),
            ))
        }
        Err(error) => Err(error.into()),
    }
}

fn append_revision_in_transaction(
    transaction: &Transaction<'_>,
    revision: &StoredRevision,
    expected_parent: &RevisionId,
) -> Result<(), SqliteRepositoryError> {
    append_revision_preflight(transaction, revision, expected_parent)?;
    let stored_snapshot = snapshot_in_transaction(transaction, &revision.snapshot_id)?
        .ok_or_else(|| SqliteRepositoryError::UnknownSnapshot(revision.snapshot_id.clone()))?;
    validate_stored_snapshot(&stored_snapshot)?;
    insert_revision_and_advance(transaction, revision, expected_parent)
}

fn append_revision_preflight(
    transaction: &Transaction<'_>,
    revision: &StoredRevision,
    expected_parent: &RevisionId,
) -> Result<ProjectId, SqliteRepositoryError> {
    if revision.parent_revision_id.as_ref() != Some(expected_parent) {
        return Err(SqliteRepositoryError::InvalidRevisionParentBinding {
            revision_id: revision.id.clone(),
            expected_parent_id: expected_parent.clone(),
            actual_parent_id: revision.parent_revision_id.clone(),
        });
    }

    let conversation = transaction
        .query_row(
            "SELECT project_id, head_revision_id FROM conversations WHERE id = ?1",
            [revision.conversation_id.as_str()],
            |row| {
                Ok((
                    ProjectId::from(row.get::<_, String>(0)?),
                    RevisionId::from(row.get::<_, String>(1)?),
                ))
            },
        )
        .optional()?;
    let Some((project_id, actual_head)) = conversation else {
        return Err(SqliteRepositoryError::UnknownConversation(
            revision.conversation_id.clone(),
        ));
    };
    if actual_head != *expected_parent {
        return Err(SqliteRepositoryError::ExpectedHeadConflict {
            conversation_id: revision.conversation_id.clone(),
            expected_revision_id: expected_parent.clone(),
            actual_revision_id: actual_head,
        });
    }
    if !exists(
        transaction,
        "SELECT 1 FROM revisions WHERE id = ?1",
        expected_parent.as_str(),
    )? {
        return Err(SqliteRepositoryError::UnknownRevision(
            expected_parent.clone(),
        ));
    }
    if !revision_belongs_to_project(transaction, &project_id, expected_parent)? {
        return Err(SqliteRepositoryError::RevisionNotInProject {
            project_id,
            revision_id: expected_parent.clone(),
        });
    }
    if !exists(
        transaction,
        "SELECT 1 FROM snapshots WHERE id = ?1",
        revision.snapshot_id.as_str(),
    )? {
        // The non-atomic append API requires callers to insert the snapshot
        // first. The atomic variant inserts it in the same transaction before
        // this preflight and rolls it back if the preflight or CAS fails.
        return Err(SqliteRepositoryError::UnknownSnapshot(
            revision.snapshot_id.clone(),
        ));
    }
    Ok(project_id)
}

fn insert_revision_and_advance(
    transaction: &Transaction<'_>,
    revision: &StoredRevision,
    expected_parent: &RevisionId,
) -> Result<(), SqliteRepositoryError> {
    insert_revision(transaction, revision)?;
    let changed = transaction.execute(
        "UPDATE conversations SET head_revision_id = ?1 WHERE id = ?2 AND head_revision_id = ?3",
        params![
            revision.id.as_str(),
            revision.conversation_id.as_str(),
            expected_parent.as_str()
        ],
    )?;
    if changed != 1 {
        return Err(SqliteRepositoryError::ExpectedHeadConflict {
            conversation_id: revision.conversation_id.clone(),
            expected_revision_id: expected_parent.clone(),
            actual_revision_id: head_in(transaction, &revision.conversation_id)?,
        });
    }
    Ok(())
}

fn revision_belongs_to_project(
    transaction: &Transaction<'_>,
    project_id: &ProjectId,
    revision_id: &RevisionId,
) -> Result<bool, rusqlite::Error> {
    transaction.query_row(
        "WITH RECURSIVE project_revisions(id) AS (
             SELECT root_revision_id FROM projects WHERE id = ?1
             UNION
             SELECT revisions.id
             FROM revisions
             JOIN project_revisions ON revisions.parent_revision_id = project_revisions.id
         )
         SELECT EXISTS(SELECT 1 FROM project_revisions WHERE id = ?2)",
        params![project_id.as_str(), revision_id.as_str()],
        |row| row.get::<_, bool>(0),
    )
}

fn snapshot_in_transaction(
    transaction: &Transaction<'_>,
    id: &SnapshotId,
) -> Result<Option<StoredSnapshot>, rusqlite::Error> {
    transaction
        .query_row(
            "SELECT id, format_version, canonical_bytes FROM snapshots WHERE id = ?1",
            [id.as_str()],
            |row| {
                Ok(StoredSnapshot {
                    id: SnapshotId::from(row.get::<_, String>(0)?),
                    format_version: row.get(1)?,
                    canonical_bytes: row.get(2)?,
                })
            },
        )
        .optional()
}

fn validate_stored_snapshot(snapshot: &StoredSnapshot) -> Result<(), SqliteRepositoryError> {
    if snapshot.format_version.trim().is_empty() {
        return Err(SqliteRepositoryError::InvalidSnapshotFormat(
            snapshot.id.clone(),
        ));
    }
    let derived = Sha256SnapshotIdentityDeriver
        .derive_snapshot_id(&snapshot.canonical_bytes)
        .expect("sha256 identity derivation is infallible");
    if derived != snapshot.id {
        return Err(SqliteRepositoryError::InvalidSnapshotIdentity(
            snapshot.id.clone(),
        ));
    }
    Ok(())
}

fn head_in(
    transaction: &Transaction<'_>,
    conversation_id: &ConversationId,
) -> Result<RevisionId, SqliteRepositoryError> {
    transaction
        .query_row(
            "SELECT head_revision_id FROM conversations WHERE id = ?1",
            [conversation_id.as_str()],
            |row| row.get::<_, String>(0),
        )
        .map(RevisionId::from)
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operations::{
        OPERATION_CONTRACT_VERSION, Operation, OperationErrorCode, OperationOutcome,
        OperationRequest, execute_sqlite_operation,
    };
    use tempfile::tempdir;

    fn snapshot(bytes: &[u8]) -> StoredSnapshot {
        let id = Sha256SnapshotIdentityDeriver
            .derive_snapshot_id(bytes)
            .unwrap();
        StoredSnapshot {
            id,
            format_version: "test-v1".into(),
            canonical_bytes: bytes.to_vec(),
        }
    }
    fn snapshot_id(bytes: &[u8]) -> SnapshotId {
        snapshot(bytes).id
    }
    fn root() -> StoredProjectRoot {
        let project = ProjectId::from("warehouse");
        let conversation = ConversationId::from("overall");
        let revision = RevisionId::from("r0");
        let snap = snapshot(b"root");
        StoredProjectRoot {
            project_id: project.clone(),
            root_conversation: StoredConversation {
                id: conversation.clone(),
                project_id: project,
                purpose: "Overall framing".into(),
                origin_json: "{\"kind\":\"root\"}".into(),
                head_revision_id: revision.clone(),
            },
            root_revision: StoredRevision {
                id: revision,
                snapshot_id: snap.id.clone(),
                parent_revision_id: None,
                conversation_id: conversation,
                metadata_json: "{\"operation\":\"root\"}".into(),
            },
            root_snapshot: snap,
        }
    }
    fn child(id: &str, snapshot_id: &SnapshotId, parent: &str) -> StoredRevision {
        StoredRevision {
            id: RevisionId::from(id),
            snapshot_id: snapshot_id.clone(),
            parent_revision_id: Some(RevisionId::from(parent)),
            conversation_id: ConversationId::from("overall"),
            metadata_json: "{\"operation\":\"manual_edit\"}".into(),
        }
    }
    fn proposal() -> StoredProposal {
        StoredProposal {
            id: ProposalId::from("p1"),
            project_id: ProjectId::from("warehouse"),
            conversation_id: ConversationId::from("overall"),
            parent_revision_id: RevisionId::from("r0"),
            proposed_revision_id: RevisionId::from("r1"),
            patch_json: "{\"operations\":[]}".into(),
            status: "pending".into(),
            accepted_revision_id: None,
            agent_provenance: None,
        }
    }
    fn receipt(id: &str) -> StoredOperationReceipt {
        StoredOperationReceipt {
            request_id: id.into(),
            request_json: format!("{{\"requestId\":\"{id}\"}}"),
            response_json: "{\"status\":\"success\"}".into(),
        }
    }

    #[test]
    fn same_expected_head_has_one_success_and_one_conflict() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("same-head.sqlite");
        let mut first = SqliteRevisionRepository::open(&path).unwrap();
        first.create_project(root()).unwrap();
        let a = snapshot(b"a");
        let b = snapshot(b"b");
        first.insert_snapshot(&a).unwrap();
        first.insert_snapshot(&b).unwrap();
        // Both clients begin from the same persisted r0 head. The second
        // client retains that expected parent after the first commits.
        let mut second = SqliteRevisionRepository::open(&path).unwrap();
        first
            .append_revision(&child("r1", &a.id, "r0"), &RevisionId::from("r0"))
            .unwrap();
        let error = second
            .append_revision(&child("r2", &b.id, "r0"), &RevisionId::from("r0"))
            .unwrap_err();
        assert!(matches!(
            error,
            SqliteRepositoryError::ExpectedHeadConflict { .. }
        ));
        assert_eq!(
            first
                .conversation(&ConversationId::from("overall"))
                .unwrap()
                .head_revision_id,
            RevisionId::from("r1")
        );
        assert!(matches!(
            first.revision(&RevisionId::from("r2")),
            Err(SqliteRepositoryError::UnknownRevision(_))
        ));
    }

    #[test]
    fn failed_head_movement_rolls_back_inserted_revision() {
        let mut repository = SqliteRevisionRepository::open_in_memory().unwrap();
        repository.create_project(root()).unwrap();
        let a = snapshot(b"a");
        repository.insert_snapshot(&a).unwrap();
        repository.connection.execute_batch("CREATE TRIGGER reject_head_update BEFORE UPDATE OF head_revision_id ON conversations BEGIN SELECT RAISE(ABORT, 'forced'); END;").unwrap();
        assert!(
            repository
                .append_revision(&child("r1", &a.id, "r0"), &RevisionId::from("r0"))
                .is_err()
        );
        assert!(matches!(
            repository.revision(&RevisionId::from("r1")),
            Err(SqliteRepositoryError::UnknownRevision(_))
        ));
        assert_eq!(
            repository
                .conversation(&ConversationId::from("overall"))
                .unwrap()
                .head_revision_id,
            RevisionId::from("r0")
        );
    }

    #[test]
    fn receipt_failure_rolls_back_proposal_creation() {
        let mut repository = SqliteRevisionRepository::open_in_memory().unwrap();
        repository.create_project(root()).unwrap();
        repository
            .connection
            .execute_batch(
                "CREATE TRIGGER reject_receipt BEFORE INSERT ON operation_receipts
                 BEGIN SELECT RAISE(ABORT, 'forced receipt failure'); END;",
            )
            .unwrap();
        assert!(
            repository
                .insert_proposal_at_expected_head_and_operation_receipt(
                    &proposal(),
                    &RevisionId::from("r0"),
                    &receipt("request-propose"),
                )
                .is_err()
        );
        assert!(matches!(
            repository.proposal(&ProposalId::from("p1")),
            Err(SqliteRepositoryError::UnknownProposal(_))
        ));
        assert!(
            repository
                .operation_receipt("request-propose")
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn receipt_failure_rolls_back_acceptance_snapshot_revision_head_and_status() {
        let mut repository = SqliteRevisionRepository::open_in_memory().unwrap();
        repository.create_project(root()).unwrap();
        repository
            .insert_proposal_at_expected_head(&proposal(), &RevisionId::from("r0"))
            .unwrap();
        repository
            .connection
            .execute_batch(
                "CREATE TRIGGER reject_receipt BEFORE INSERT ON operation_receipts
                 BEGIN SELECT RAISE(ABORT, 'forced receipt failure'); END;",
            )
            .unwrap();
        let candidate = snapshot(b"accepted");
        assert!(
            repository
                .append_revision_with_snapshot_proposal_and_operation_receipt(
                    &child("r1", &candidate.id, "r0"),
                    &candidate,
                    &RevisionId::from("r0"),
                    &ProposalId::from("p1"),
                    None,
                    &receipt("request-accept"),
                )
                .is_err()
        );
        assert!(matches!(
            repository.revision(&RevisionId::from("r1")),
            Err(SqliteRepositoryError::UnknownRevision(_))
        ));
        assert!(matches!(
            repository.snapshot(&candidate.id),
            Err(SqliteRepositoryError::UnknownSnapshot(_))
        ));
        assert_eq!(
            repository
                .conversation(&ConversationId::from("overall"))
                .unwrap()
                .head_revision_id,
            RevisionId::from("r0")
        );
        assert_eq!(
            repository.proposal(&ProposalId::from("p1")).unwrap().status,
            "pending"
        );
        assert!(
            repository
                .operation_receipt("request-accept")
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn receipt_failure_rolls_back_rejection_status() {
        let mut repository = SqliteRevisionRepository::open_in_memory().unwrap();
        repository.create_project(root()).unwrap();
        repository
            .insert_proposal_at_expected_head(&proposal(), &RevisionId::from("r0"))
            .unwrap();
        repository
            .connection
            .execute_batch(
                "CREATE TRIGGER reject_receipt BEFORE INSERT ON operation_receipts
                 BEGIN SELECT RAISE(ABORT, 'forced receipt failure'); END;",
            )
            .unwrap();
        assert!(
            repository
                .reject_proposal_at_expected_head_and_operation_receipt(
                    &ProposalId::from("p1"),
                    &ConversationId::from("overall"),
                    &RevisionId::from("r0"),
                    &receipt("request-reject"),
                )
                .is_err()
        );
        assert_eq!(
            repository.proposal(&ProposalId::from("p1")).unwrap().status,
            "pending"
        );
    }

    #[test]
    fn receipt_failure_rolls_back_resolved_snapshot_and_evidence() {
        let mut repository = SqliteRevisionRepository::open_in_memory().unwrap();
        repository.create_project(root()).unwrap();
        repository
            .connection
            .execute_batch(
                "CREATE TRIGGER reject_receipt BEFORE INSERT ON operation_receipts
                 BEGIN SELECT RAISE(ABORT, 'forced receipt failure'); END;",
            )
            .unwrap();
        let resolved = snapshot(b"resolved-analysis");
        let evidence = StoredEvidence {
            id: EvidenceId::from("analysis-evidence"),
            authored_snapshot_id: snapshot_id(b"root"),
            resolved_snapshot_id: Some(resolved.id.clone()),
            manifest_json: "{\"status\":\"completed\"}".into(),
            blob_ref: None,
        };
        assert!(
            repository
                .attach_evidence_with_snapshot_and_operation_receipt(
                    &evidence,
                    Some(&resolved),
                    &receipt("request-analysis"),
                )
                .is_err()
        );
        assert!(matches!(
            repository.evidence(&EvidenceId::from("analysis-evidence")),
            Err(SqliteRepositoryError::UnknownEvidence(_))
        ));
        assert!(matches!(
            repository.snapshot(&resolved.id),
            Err(SqliteRepositoryError::UnknownSnapshot(_))
        ));
    }

    #[test]
    fn executor_surfaces_receipt_read_and_read_only_write_failures() {
        let request = || OperationRequest {
            contract_version: OPERATION_CONTRACT_VERSION.into(),
            request_id: "capabilities-fault".into(),
            operation: Operation::Capabilities,
        };

        let mut read_failure = SqliteRevisionRepository::open_in_memory().unwrap();
        read_failure
            .connection
            .execute("DROP TABLE operation_receipts", [])
            .unwrap();
        let response = execute_sqlite_operation(&mut read_failure, request());
        assert!(matches!(
            response.outcome,
            OperationOutcome::Error { error }
                if error.code == OperationErrorCode::RepositoryError
        ));

        let mut write_failure = SqliteRevisionRepository::open_in_memory().unwrap();
        write_failure
            .connection
            .execute_batch(
                "CREATE TRIGGER reject_receipt BEFORE INSERT ON operation_receipts
                 BEGIN SELECT RAISE(ABORT, 'forced receipt failure'); END;",
            )
            .unwrap();
        let response = execute_sqlite_operation(&mut write_failure, request());
        assert!(matches!(
            response.outcome,
            OperationOutcome::Error { error }
                if error.code == OperationErrorCode::RepositoryError
        ));
    }

    #[test]
    fn close_reopen_preserves_lineage_evidence_artefact_and_snapshot_staleness() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("revision.sqlite");
        {
            let mut repository = SqliteRevisionRepository::open(&path).unwrap();
            repository.create_project(root()).unwrap();
            let a = snapshot(b"a");
            repository.insert_snapshot(&a).unwrap();
            repository
                .append_revision(&child("r1", &a.id, "r0"), &RevisionId::from("r0"))
                .unwrap();
            repository
                .attach_evidence(&StoredEvidence {
                    id: EvidenceId::from("e1"),
                    authored_snapshot_id: snapshot_id(b"root"),
                    resolved_snapshot_id: None,
                    manifest_json: "{\"solver\":\"test\"}".into(),
                    blob_ref: Some("blobs/e1".into()),
                })
                .unwrap();
            repository
                .attach_artefact(&StoredArtefact {
                    id: ArtefactId::from("a1"),
                    kind: "preview".into(),
                    source_snapshot_id: snapshot_id(b"root"),
                    source_evidence_id: Some(EvidenceId::from("e1")),
                    manifest_json: "{}".into(),
                    blob_ref: Some("blobs/a1".into()),
                })
                .unwrap();
        }
        let repository = SqliteRevisionRepository::open(&path).unwrap();
        assert_eq!(
            repository
                .snapshot(&snapshot_id(b"root"))
                .unwrap()
                .canonical_bytes,
            b"root"
        );
        assert_eq!(
            repository
                .history(&ConversationId::from("overall"))
                .unwrap()
                .iter()
                .map(|r| r.id.as_str())
                .collect::<Vec<_>>(),
            vec!["r1", "r0"]
        );
        assert_eq!(
            repository
                .evidence(&EvidenceId::from("e1"))
                .unwrap()
                .blob_ref
                .as_deref(),
            Some("blobs/e1")
        );
        assert_eq!(
            repository
                .artefact(&ArtefactId::from("a1"))
                .unwrap()
                .source_evidence_id,
            Some(EvidenceId::from("e1"))
        );
        assert!(
            repository
                .evidence_is_stale_for_revision(&EvidenceId::from("e1"), &RevisionId::from("r1"))
                .unwrap()
        );
    }
}
