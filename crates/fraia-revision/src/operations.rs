//! Versioned, framework-independent operations over revision state.
//!
//! This boundary owns no transport, persistence, prompt, or user-interface
//! behavior. Adapters deserialize one request, call [`execute_operation`], and
//! serialize the returned response. Structural changes remain pending until a
//! separate explicit accept operation succeeds against the exact expected
//! conversation head.

use crate::agent_contract::AgentTurnProvenance;
use crate::analysis_service::{
    AnalysisExecutionStage, AnalysisSettings, SnapshotAnalysisError, SnapshotAnalysisOutcome,
    SnapshotAnalysisRun, analyse_accepted_revision_with_control,
    analyse_accepted_revision_with_settings, dependencies_for_snapshot_with_settings,
};
use crate::conversation::{ConversationError, ConversationHead};
use crate::diff::{SemanticDiff, semantic_diff};
use crate::evidence::{AnalysisEvidence, EvidenceStaleness, staleness_for};
use crate::patch::{StructuralPatch, apply_patch};
use crate::repository::{InMemoryRevisionRepository, ProposalId, RepositoryError};
use crate::snapshot::ModelSnapshot;
use crate::sqlite::{
    SqliteRepositoryError, SqliteRevisionRepository, StoredOperationReceipt, StoredProposal,
    StoredRevision, StoredSnapshot,
};
use crate::{ConversationId, EvidenceId, ProjectId, RevisionId, SnapshotId};
use fraia_core::{StructuralModel, ValidationReport, validate_structural_model};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// The only operation-envelope version accepted by this implementation.
pub const OPERATION_CONTRACT_VERSION: &str = "fraia.operations.v1";

#[derive(Debug, Clone)]
pub struct DesignRunOperationContext {
    pub project_dir: PathBuf,
    pub project_id: fraia_core::ProjectId,
    pub design_id: fraia_core::DesignId,
    pub actor: fraia_core::DesignRunActor,
    pub created_at: String,
    pub parent_run_id: Option<String>,
}

impl DesignRunOperationContext {
    pub fn new(
        project_dir: impl AsRef<Path>,
        project_id: fraia_core::ProjectId,
        design_id: fraia_core::DesignId,
        actor: fraia_core::DesignRunActor,
        created_at: impl Into<String>,
    ) -> Self {
        Self {
            project_dir: project_dir.as_ref().to_path_buf(),
            project_id,
            design_id,
            actor,
            created_at: created_at.into(),
            parent_run_id: None,
        }
    }
}

/// One transport-neutral operation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationRequest {
    pub contract_version: String,
    pub request_id: String,
    #[serde(flatten)]
    pub operation: Operation,
}

/// The first closed operation vocabulary. It can be extended only by adding a
/// new contract version or a backwards-compatible variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "operation", content = "parameters", rename_all = "snake_case")]
pub enum Operation {
    Capabilities,
    Inspect {
        conversation_id: ConversationId,
    },
    ProposeStructuralPatch {
        proposal_id: ProposalId,
        conversation_id: ConversationId,
        expected_head_revision_id: RevisionId,
        proposed_revision_id: RevisionId,
        patch: StructuralPatch,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        agent_provenance: Option<AgentTurnProvenance>,
    },
    AcceptStructuralPatch {
        proposal_id: ProposalId,
        conversation_id: ConversationId,
        expected_head_revision_id: RevisionId,
    },
    RejectStructuralPatch {
        proposal_id: ProposalId,
        conversation_id: ConversationId,
        expected_head_revision_id: RevisionId,
    },
    ValidateSnapshot {
        revision_id: RevisionId,
        expected_snapshot_id: SnapshotId,
    },
    AnalyseSnapshot {
        revision_id: RevisionId,
        expected_snapshot_id: SnapshotId,
        evidence_id: EvidenceId,
        settings: AnalysisSettings,
    },
    InspectAnalysisEvidence {
        evidence_id: EvidenceId,
        against_revision_id: RevisionId,
    },
}

/// One transport-neutral operation response. A response always uses the
/// implementation's current contract version, including version failures.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationResponse {
    pub contract_version: String,
    pub request_id: String,
    #[serde(flatten)]
    pub outcome: OperationOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum OperationOutcome {
    Success { result: Box<OperationResult> },
    Error { error: OperationError },
}

/// Machine-readable outputs from the closed operation vocabulary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OperationResult {
    Capabilities {
        contract_version: String,
        operations: Vec<String>,
        features: Vec<String>,
    },
    Inspection {
        project_id: ProjectId,
        conversation: ConversationHead,
        authored_model: Box<StructuralModel>,
    },
    StructuralPatchProposed {
        proposal_id: ProposalId,
        parent_revision_id: RevisionId,
        proposed_revision_id: RevisionId,
        preview_diff: SemanticDiff,
    },
    StructuralPatchAccepted {
        proposal_id: ProposalId,
        revision_id: RevisionId,
        parent_revision_id: RevisionId,
        snapshot_id: SnapshotId,
        semantic_diff: SemanticDiff,
    },
    StructuralPatchRejected {
        proposal_id: ProposalId,
    },
    SnapshotValidated {
        revision_id: RevisionId,
        snapshot_id: SnapshotId,
        report: ValidationReport,
    },
    SnapshotAnalysed {
        run: Box<SnapshotAnalysisRun>,
    },
    AnalysisEvidenceInspection {
        evidence: Box<AnalysisEvidence>,
        against_revision_id: RevisionId,
        staleness: EvidenceStaleness,
    },
}

/// Stable error categories for transport adapters and automation clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationErrorCode {
    UnsupportedContractVersion,
    InvalidRequest,
    ExpectedHeadMismatch,
    ExpectedSnapshotMismatch,
    InvalidPatch,
    RepositoryError,
    Cancelled,
}

pub struct AnalysisOperationControl<'a> {
    pub progress: &'a mut dyn FnMut(AnalysisExecutionStage),
    pub cancelled: &'a mut dyn FnMut() -> bool,
    /// Atomically reserves the terminal publication boundary. Returns false
    /// when cancellation won the race.
    pub begin_publication: &'a mut dyn FnMut() -> bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationError {
    pub code: OperationErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head_conflict: Option<HeadConflict>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot_conflict: Option<SnapshotConflict>,
}

/// Exact optimistic-concurrency evidence returned for a stale request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeadConflict {
    pub conversation_id: ConversationId,
    pub expected_revision_id: RevisionId,
    pub actual_revision_id: RevisionId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotConflict {
    pub revision_id: RevisionId,
    pub expected_snapshot_id: SnapshotId,
    pub actual_snapshot_id: SnapshotId,
}

/// Executes one operation without assuming a transport or framework.
pub fn execute_operation(
    repository: &mut InMemoryRevisionRepository,
    request: OperationRequest,
) -> OperationResponse {
    let request_id = request.request_id;
    let outcome = if request.contract_version != OPERATION_CONTRACT_VERSION {
        OperationOutcome::Error {
            error: OperationError {
                code: OperationErrorCode::UnsupportedContractVersion,
                message: format!(
                    "unsupported operation contract version `{}`; expected `{OPERATION_CONTRACT_VERSION}`",
                    request.contract_version
                ),
                head_conflict: None,
                snapshot_conflict: None,
            },
        }
    } else if request_id.trim().is_empty() {
        OperationOutcome::Error {
            error: OperationError {
                code: OperationErrorCode::InvalidRequest,
                message: "operation request id must not be empty".into(),
                head_conflict: None,
                snapshot_conflict: None,
            },
        }
    } else {
        match execute_supported_operation(repository, request.operation) {
            Ok(result) => OperationOutcome::Success {
                result: Box::new(result),
            },
            Err(error) => OperationOutcome::Error { error },
        }
    };

    OperationResponse {
        contract_version: OPERATION_CONTRACT_VERSION.into(),
        request_id,
        outcome,
    }
}

fn execute_supported_operation(
    repository: &mut InMemoryRevisionRepository,
    operation: Operation,
) -> Result<OperationResult, OperationError> {
    match operation {
        Operation::Capabilities => Ok(capabilities_result()),
        Operation::Inspect { conversation_id } => {
            let conversation = repository
                .head(&conversation_id)
                .map_err(repository_error)?;
            let revision = repository
                .revision(&conversation.head_revision_id)
                .map_err(repository_error)?;
            let snapshot = repository
                .snapshot(revision.snapshot_id())
                .map_err(repository_error)?;
            let authored_model = ModelSnapshot::from_canonical(
                snapshot.id().clone(),
                snapshot.canonical_format_version().clone(),
                snapshot.canonical_bytes(),
            )
            .map_err(|error| OperationError {
                code: OperationErrorCode::RepositoryError,
                message: error.to_string(),
                head_conflict: None,
                snapshot_conflict: None,
            })?
            .model()
            .clone();
            Ok(OperationResult::Inspection {
                project_id: repository.project_id().clone(),
                conversation,
                authored_model: Box::new(authored_model),
            })
        }
        Operation::ProposeStructuralPatch {
            proposal_id,
            conversation_id,
            expected_head_revision_id,
            proposed_revision_id,
            patch,
            agent_provenance,
        } => {
            ensure_expected_head(repository, &conversation_id, &expected_head_revision_id)?;
            let parent = repository
                .revision(&expected_head_revision_id)
                .map_err(repository_error)?;
            let parent_model = repository
                .snapshot(parent.snapshot_id())
                .map_err(repository_error)?
                .model();
            let preview_diff = apply_patch(parent_model, &patch)
                .map_err(|error| OperationError {
                    code: OperationErrorCode::InvalidPatch,
                    message: error.to_string(),
                    head_conflict: None,
                    snapshot_conflict: None,
                })?
                .diff;
            if let Some(provenance) = agent_provenance {
                repository
                    .create_proposal_with_provenance(
                        proposal_id.clone(),
                        conversation_id,
                        expected_head_revision_id.clone(),
                        proposed_revision_id.clone(),
                        patch,
                        provenance,
                    )
                    .map_err(repository_error)?;
            } else {
                repository
                    .create_proposal(
                        proposal_id.clone(),
                        conversation_id,
                        expected_head_revision_id.clone(),
                        proposed_revision_id.clone(),
                        patch,
                    )
                    .map_err(repository_error)?;
            }
            Ok(OperationResult::StructuralPatchProposed {
                proposal_id,
                parent_revision_id: expected_head_revision_id,
                proposed_revision_id,
                preview_diff,
            })
        }
        Operation::AcceptStructuralPatch {
            proposal_id,
            conversation_id,
            expected_head_revision_id,
        } => {
            ensure_expected_head(repository, &conversation_id, &expected_head_revision_id)?;
            let proposal = repository
                .proposal(&proposal_id)
                .map_err(repository_error)?;
            if proposal.conversation_id() != &conversation_id {
                return Err(OperationError {
                    code: OperationErrorCode::InvalidRequest,
                    message: format!(
                        "proposal `{proposal_id}` belongs to conversation `{}`, not `{conversation_id}`",
                        proposal.conversation_id()
                    ),
                    head_conflict: None,
                    snapshot_conflict: None,
                });
            }
            if proposal.parent_revision_id() != &expected_head_revision_id {
                return Err(OperationError {
                    code: OperationErrorCode::InvalidRequest,
                    message: format!(
                        "proposal `{proposal_id}` expects parent `{}`, not `{expected_head_revision_id}`",
                        proposal.parent_revision_id()
                    ),
                    head_conflict: None,
                    snapshot_conflict: None,
                });
            }
            let accepted = repository
                .accept_proposal(&proposal_id)
                .map_err(repository_error)?;
            Ok(OperationResult::StructuralPatchAccepted {
                proposal_id,
                revision_id: accepted.revision_id().clone(),
                parent_revision_id: accepted
                    .parent_revision_id()
                    .expect("accepted structural patch must have a parent")
                    .clone(),
                snapshot_id: accepted.snapshot_id().clone(),
                semantic_diff: accepted.semantic_diff().clone(),
            })
        }
        Operation::RejectStructuralPatch {
            proposal_id,
            conversation_id,
            expected_head_revision_id,
        } => {
            ensure_expected_head(repository, &conversation_id, &expected_head_revision_id)?;
            let proposal = repository
                .proposal(&proposal_id)
                .map_err(repository_error)?;
            if proposal.conversation_id() != &conversation_id
                || proposal.parent_revision_id() != &expected_head_revision_id
            {
                return Err(invalid_request(format!(
                    "proposal `{proposal_id}` does not match the supplied conversation and expected head"
                )));
            }
            repository
                .reject_proposal(&proposal_id)
                .map_err(repository_error)?;
            Ok(OperationResult::StructuralPatchRejected { proposal_id })
        }
        Operation::ValidateSnapshot {
            revision_id,
            expected_snapshot_id,
        } => {
            let snapshot = exact_memory_snapshot(repository, &revision_id, &expected_snapshot_id)?;
            Ok(OperationResult::SnapshotValidated {
                revision_id,
                snapshot_id: expected_snapshot_id,
                report: validate_structural_model(snapshot.model()),
            })
        }
        Operation::AnalyseSnapshot {
            revision_id,
            expected_snapshot_id,
            evidence_id,
            settings,
        } => {
            let stored = exact_memory_snapshot(repository, &revision_id, &expected_snapshot_id)?;
            let canonical = ModelSnapshot::from_canonical(
                stored.id().clone(),
                stored.canonical_format_version().clone(),
                stored.canonical_bytes(),
            )
            .map_err(|error| repository_message(error.to_string()))?;
            let revision = repository
                .revision(&revision_id)
                .map_err(repository_error)?;
            let conversation = repository
                .conversation(revision.conversation_id())
                .map_err(repository_error)?;
            let mut transient = InMemoryRevisionRepository::create(
                repository.project_id().clone(),
                conversation.id().clone(),
                conversation.purpose(),
                revision_id.clone(),
                canonical.model().clone(),
            )
            .map_err(repository_error)?;
            let run = analyse_accepted_revision_with_settings(
                &mut transient,
                &revision_id,
                evidence_id,
                settings,
            )
            .map_err(|error| repository_message(error.to_string()))?;
            repository
                .attach_evidence(&revision_id, run.evidence.clone())
                .map_err(repository_error)?;
            Ok(OperationResult::SnapshotAnalysed { run: Box::new(run) })
        }
        Operation::InspectAnalysisEvidence {
            evidence_id,
            against_revision_id,
        } => {
            let evidence = repository
                .evidence(&evidence_id)
                .map_err(repository_error)?
                .clone();
            let dependencies = current_dependencies(
                &evidence,
                repository
                    .revision(&against_revision_id)
                    .map_err(repository_error)?
                    .snapshot_id(),
            )?;
            let staleness = repository
                .evidence_staleness(&evidence_id, &against_revision_id, &dependencies)
                .map_err(repository_error)?;
            Ok(OperationResult::AnalysisEvidenceInspection {
                evidence: Box::new(evidence),
                against_revision_id,
                staleness,
            })
        }
    }
}

/// Executes the same operation contract against durable SQLite state.
/// Responses are cached by request ID when receipt persistence succeeds.
/// Reusing a persisted request ID with different JSON is rejected, while an
/// exact replay returns the original byte-equivalent response. A receipt
/// persistence failure is returned as a repository error and is never hidden.
pub fn execute_sqlite_operation(
    repository: &mut SqliteRevisionRepository,
    request: OperationRequest,
) -> OperationResponse {
    execute_sqlite_operation_internal(repository, request, None, None)
}

pub fn execute_sqlite_operation_with_design_runs(
    repository: &mut SqliteRevisionRepository,
    request: OperationRequest,
    run_context: &DesignRunOperationContext,
) -> OperationResponse {
    execute_sqlite_operation_internal(repository, request, Some(run_context), None)
}

pub fn execute_sqlite_operation_with_design_runs_controlled(
    repository: &mut SqliteRevisionRepository,
    request: OperationRequest,
    run_context: &DesignRunOperationContext,
    control: &mut AnalysisOperationControl<'_>,
) -> OperationResponse {
    execute_sqlite_operation_internal(repository, request, Some(run_context), Some(control))
}

fn execute_sqlite_operation_internal(
    repository: &mut SqliteRevisionRepository,
    request: OperationRequest,
    run_context: Option<&DesignRunOperationContext>,
    mut analysis_control: Option<&mut AnalysisOperationControl<'_>>,
) -> OperationResponse {
    let request_id = request.request_id.clone();
    let request_json = match serde_json::to_string(&request) {
        Ok(json) => json,
        Err(error) => {
            return error_response(
                request_id,
                OperationErrorCode::InvalidRequest,
                error.to_string(),
                None,
            );
        }
    };
    match repository.operation_receipt(&request_id) {
        Ok(Some(receipt)) if receipt.request_json == request_json => {
            return serde_json::from_str(&receipt.response_json).unwrap_or_else(|error| {
                error_response(
                    request_id,
                    OperationErrorCode::RepositoryError,
                    format!("stored operation receipt is invalid: {error}"),
                    None,
                )
            });
        }
        Ok(Some(receipt)) => {
            return error_response(
                request_id,
                OperationErrorCode::InvalidRequest,
                format!(
                    "operation request `{}` was already used for different content",
                    receipt.request_id
                ),
                None,
            );
        }
        Ok(None) => {}
        Err(error) => return sqlite_repository_response(request_id, error),
    }

    let (response, receipt_persisted) = if request.contract_version != OPERATION_CONTRACT_VERSION {
        (
            error_response(
                request_id.clone(),
                OperationErrorCode::UnsupportedContractVersion,
                format!(
                    "unsupported operation contract version `{}`; expected `{OPERATION_CONTRACT_VERSION}`",
                    request.contract_version
                ),
                None,
            ),
            false,
        )
    } else if request_id.trim().is_empty() {
        (
            error_response(
                request_id.clone(),
                OperationErrorCode::InvalidRequest,
                "operation request id must not be empty".into(),
                None,
            ),
            false,
        )
    } else {
        match execute_supported_sqlite_operation(
            repository,
            request.operation,
            &request_id,
            &request_json,
            run_context,
            analysis_control.as_deref_mut(),
        ) {
            Ok(execution) => (
                success_response(request_id.clone(), execution.result),
                execution.receipt_persisted,
            ),
            Err(error) => (
                OperationResponse {
                    contract_version: OPERATION_CONTRACT_VERSION.into(),
                    request_id: request_id.clone(),
                    outcome: OperationOutcome::Error { error },
                },
                false,
            ),
        }
    };

    if !request_id.trim().is_empty() && !receipt_persisted {
        let receipt = match operation_receipt(&request_id, request_json, &response) {
            Ok(receipt) => receipt,
            Err(error) => {
                return error_response(
                    request_id,
                    OperationErrorCode::RepositoryError,
                    error,
                    None,
                );
            }
        };
        if let Err(error) = repository.store_operation_receipt(&receipt) {
            return sqlite_repository_response(request_id, error);
        }
    }
    response
}

struct SqliteExecution {
    result: OperationResult,
    receipt_persisted: bool,
}

fn execute_supported_sqlite_operation(
    repository: &mut SqliteRevisionRepository,
    operation: Operation,
    request_id: &str,
    request_json: &str,
    run_context: Option<&DesignRunOperationContext>,
    mut analysis_control: Option<&mut AnalysisOperationControl<'_>>,
) -> Result<SqliteExecution, OperationError> {
    match operation {
        Operation::Capabilities => Ok(SqliteExecution {
            result: capabilities_result(),
            receipt_persisted: false,
        }),
        Operation::Inspect { conversation_id } => {
            let conversation = repository
                .conversation(&conversation_id)
                .map_err(sqlite_error)?;
            let revision = repository
                .revision(&conversation.head_revision_id)
                .map_err(sqlite_error)?;
            let authored_model = repository
                .hydrate_snapshot(&revision.snapshot_id)
                .map_err(sqlite_error)?
                .model()
                .clone();
            Ok(SqliteExecution {
                result: OperationResult::Inspection {
                    project_id: conversation.project_id.clone(),
                    conversation: ConversationHead {
                        conversation_id: conversation.id,
                        purpose: conversation.purpose,
                        head_revision_id: conversation.head_revision_id,
                        head_snapshot_id: revision.snapshot_id,
                    },
                    authored_model: Box::new(authored_model),
                },
                receipt_persisted: false,
            })
        }
        Operation::ProposeStructuralPatch {
            proposal_id,
            conversation_id,
            expected_head_revision_id,
            proposed_revision_id,
            patch,
            agent_provenance,
        } => {
            let conversation = repository
                .conversation(&conversation_id)
                .map_err(sqlite_error)?;
            if conversation.head_revision_id != expected_head_revision_id {
                return Err(expected_head_error(
                    conversation_id,
                    expected_head_revision_id,
                    conversation.head_revision_id,
                ));
            }
            let parent = repository
                .revision(&expected_head_revision_id)
                .map_err(sqlite_error)?;
            let parent_model = repository
                .hydrate_snapshot(&parent.snapshot_id)
                .map_err(sqlite_error)?;
            let preview_diff = apply_patch(parent_model.model(), &patch)
                .map_err(|error| OperationError {
                    code: OperationErrorCode::InvalidPatch,
                    message: error.to_string(),
                    head_conflict: None,
                    snapshot_conflict: None,
                })?
                .diff;
            let stored = StoredProposal {
                id: proposal_id.clone(),
                project_id: conversation.project_id,
                conversation_id,
                parent_revision_id: expected_head_revision_id.clone(),
                proposed_revision_id: proposed_revision_id.clone(),
                patch_json: serde_json::to_string(&patch).map_err(invalid_json)?,
                status: "pending".into(),
                accepted_revision_id: None,
                agent_provenance,
            };
            let result = OperationResult::StructuralPatchProposed {
                proposal_id,
                parent_revision_id: expected_head_revision_id.clone(),
                proposed_revision_id,
                preview_diff,
            };
            let response = success_response(request_id.into(), result.clone());
            let receipt = operation_receipt(request_id, request_json, &response)
                .map_err(repository_message)?;
            repository
                .insert_proposal_at_expected_head_and_operation_receipt(
                    &stored,
                    &expected_head_revision_id,
                    &receipt,
                )
                .map_err(sqlite_error)?;
            Ok(SqliteExecution {
                result,
                receipt_persisted: true,
            })
        }
        Operation::AcceptStructuralPatch {
            proposal_id,
            conversation_id,
            expected_head_revision_id,
        } => {
            let proposal = repository.proposal(&proposal_id).map_err(sqlite_error)?;
            if proposal.conversation_id != conversation_id
                || proposal.parent_revision_id != expected_head_revision_id
            {
                return Err(OperationError {
                    code: OperationErrorCode::InvalidRequest,
                    message: format!(
                        "proposal `{proposal_id}` does not match the supplied conversation and expected head"
                    ),
                    head_conflict: None,
                    snapshot_conflict: None,
                });
            }
            let patch: StructuralPatch =
                serde_json::from_str(&proposal.patch_json).map_err(invalid_json)?;
            let parent = repository
                .revision(&proposal.parent_revision_id)
                .map_err(sqlite_error)?;
            let parent_snapshot = repository
                .hydrate_snapshot(&parent.snapshot_id)
                .map_err(sqlite_error)?;
            let applied =
                apply_patch(parent_snapshot.model(), &patch).map_err(|error| OperationError {
                    code: OperationErrorCode::InvalidPatch,
                    message: error.to_string(),
                    head_conflict: None,
                    snapshot_conflict: None,
                })?;
            let snapshot =
                ModelSnapshot::capture(applied.model).map_err(|error| OperationError {
                    code: OperationErrorCode::RepositoryError,
                    message: error.to_string(),
                    head_conflict: None,
                    snapshot_conflict: None,
                })?;

            if proposal.status == "accepted" {
                let revision_id = proposal
                    .accepted_revision_id
                    .unwrap_or_else(|| proposal.proposed_revision_id.clone());
                let revision = repository.revision(&revision_id).map_err(sqlite_error)?;
                return Ok(SqliteExecution {
                    result: OperationResult::StructuralPatchAccepted {
                        proposal_id,
                        revision_id,
                        parent_revision_id: expected_head_revision_id,
                        snapshot_id: revision.snapshot_id,
                        semantic_diff: applied.diff,
                    },
                    receipt_persisted: false,
                });
            }
            if proposal.status != "pending" {
                return Err(OperationError {
                    code: OperationErrorCode::InvalidRequest,
                    message: format!("proposal `{proposal_id}` is not pending"),
                    head_conflict: None,
                    snapshot_conflict: None,
                });
            }

            let stored_snapshot = StoredSnapshot {
                id: snapshot.id().clone(),
                format_version: snapshot.canonical_format_version().as_str().into(),
                canonical_bytes: snapshot.canonical_bytes().to_vec(),
            };
            let stored_revision = StoredRevision {
                id: proposal.proposed_revision_id.clone(),
                snapshot_id: stored_snapshot.id.clone(),
                parent_revision_id: Some(expected_head_revision_id.clone()),
                conversation_id,
                metadata_json: serde_json::json!({
                    "author": if proposal.agent_provenance.is_some() { "agent" } else { "manual" },
                    "operation": "accepted_proposal",
                    "proposalId": proposal_id.as_str(),
                    "semanticDiff": &applied.diff,
                    "agentProvenance": proposal.agent_provenance,
                })
                .to_string(),
            };
            let result = OperationResult::StructuralPatchAccepted {
                proposal_id: proposal_id.clone(),
                revision_id: stored_revision.id.clone(),
                parent_revision_id: expected_head_revision_id.clone(),
                snapshot_id: stored_snapshot.id.clone(),
                semantic_diff: applied.diff,
            };
            let response = success_response(request_id.into(), result.clone());
            let receipt = operation_receipt(request_id, request_json, &response)
                .map_err(repository_message)?;
            repository
                .append_revision_with_snapshot_proposal_and_operation_receipt(
                    &stored_revision,
                    &stored_snapshot,
                    &expected_head_revision_id,
                    &proposal_id,
                    proposal.agent_provenance.as_ref(),
                    &receipt,
                )
                .map_err(sqlite_error)?;
            Ok(SqliteExecution {
                result,
                receipt_persisted: true,
            })
        }
        Operation::RejectStructuralPatch {
            proposal_id,
            conversation_id,
            expected_head_revision_id,
        } => {
            let head = repository
                .conversation(&conversation_id)
                .map_err(sqlite_error)?
                .head_revision_id;
            if head != expected_head_revision_id {
                return Err(expected_head_error(
                    conversation_id,
                    expected_head_revision_id,
                    head,
                ));
            }
            let proposal = repository.proposal(&proposal_id).map_err(sqlite_error)?;
            if proposal.conversation_id != conversation_id
                || proposal.parent_revision_id != expected_head_revision_id
            {
                return Err(invalid_request(format!(
                    "proposal `{proposal_id}` does not match the supplied conversation and expected head"
                )));
            }
            let result = OperationResult::StructuralPatchRejected {
                proposal_id: proposal_id.clone(),
            };
            if proposal.status == "rejected" {
                return Ok(SqliteExecution {
                    result,
                    receipt_persisted: false,
                });
            }
            let response = success_response(request_id.into(), result.clone());
            let receipt = operation_receipt(request_id, request_json, &response)
                .map_err(repository_message)?;
            repository
                .reject_proposal_at_expected_head_and_operation_receipt(
                    &proposal_id,
                    &conversation_id,
                    &expected_head_revision_id,
                    &receipt,
                )
                .map_err(sqlite_error)?;
            Ok(SqliteExecution {
                result,
                receipt_persisted: true,
            })
        }
        Operation::ValidateSnapshot {
            revision_id,
            expected_snapshot_id,
        } => {
            let snapshot = exact_sqlite_snapshot(repository, &revision_id, &expected_snapshot_id)?;
            Ok(SqliteExecution {
                result: OperationResult::SnapshotValidated {
                    revision_id,
                    snapshot_id: expected_snapshot_id,
                    report: validate_structural_model(snapshot.model()),
                },
                receipt_persisted: false,
            })
        }
        Operation::AnalyseSnapshot {
            revision_id,
            expected_snapshot_id,
            evidence_id,
            settings,
        } => {
            let snapshot = exact_sqlite_snapshot(repository, &revision_id, &expected_snapshot_id)?;
            match repository.evidence(&evidence_id) {
                Ok(stored) => {
                    let evidence: AnalysisEvidence =
                        serde_json::from_str(&stored.manifest_json).map_err(invalid_json)?;
                    ensure_analysis_replay(&evidence, &expected_snapshot_id, &settings)?;
                    if run_context.is_some() && evidence.canonical_run_id().is_none() {
                        return Err(invalid_request(format!(
                            "evidence `{}` predates canonical design runs; rerun with a new evidence id",
                            evidence.id()
                        )));
                    }
                    let resolved = stored
                        .resolved_snapshot_id
                        .as_ref()
                        .map(|id| repository.snapshot(id))
                        .transpose()
                        .map_err(sqlite_error)?;
                    let run = run_from_evidence(revision_id, evidence, resolved)?;
                    return Ok(SqliteExecution {
                        result: OperationResult::SnapshotAnalysed { run: Box::new(run) },
                        receipt_persisted: false,
                    });
                }
                Err(SqliteRepositoryError::UnknownEvidence(_)) => {}
                Err(error) => return Err(sqlite_error(error)),
            }

            let conversation = repository
                .conversation(
                    &repository
                        .revision(&revision_id)
                        .map_err(sqlite_error)?
                        .conversation_id,
                )
                .map_err(sqlite_error)?;
            if let Some(context) = run_context
                && conversation.project_id.as_str() != context.design_id.as_str()
            {
                return Err(invalid_request(
                    "design-run design identity does not match the accepted revision scope".into(),
                ));
            }
            let mut transient = InMemoryRevisionRepository::create(
                conversation.project_id,
                conversation.id,
                conversation.purpose,
                revision_id.clone(),
                snapshot.model().clone(),
            )
            .map_err(repository_error)?;
            let mut run = match analysis_control.as_deref_mut() {
                Some(control) => analyse_accepted_revision_with_control(
                    &mut transient,
                    &revision_id,
                    evidence_id,
                    settings,
                    &mut *control.progress,
                    &mut *control.cancelled,
                ),
                None => analyse_accepted_revision_with_settings(
                    &mut transient,
                    &revision_id,
                    evidence_id,
                    settings,
                ),
            }
            .map_err(|error| match error {
                SnapshotAnalysisError::Cancelled => OperationError {
                    code: OperationErrorCode::Cancelled,
                    message: error.to_string(),
                    head_conflict: None,
                    snapshot_conflict: None,
                },
                _ => repository_message(error.to_string()),
            })?;
            if let Some(control) = analysis_control.as_deref_mut()
                && !(control.begin_publication)()
            {
                return Err(OperationError {
                    code: OperationErrorCode::Cancelled,
                    message: "analysis attempt was cancelled before publication".into(),
                    head_conflict: None,
                    snapshot_conflict: None,
                });
            }
            if let Some(context) = run_context {
                let interpretation_dependencies =
                    interpretation_dependencies_for_revision(repository, &revision_id)?;
                publish_canonical_design_run(
                    context,
                    &revision_id,
                    &interpretation_dependencies,
                    &mut run,
                )?;
            }
            let stored_evidence = crate::sqlite::StoredEvidence {
                id: run.evidence.id().clone(),
                authored_snapshot_id: run.evidence.authored_snapshot_id().clone(),
                resolved_snapshot_id: run.evidence.resolved_snapshot_id().cloned(),
                manifest_json: serde_json::to_string(&run.evidence).map_err(invalid_json)?,
                blob_ref: None,
            };
            let stored_resolved = run
                .resolved_snapshot
                .as_ref()
                .map(|resolved| StoredSnapshot {
                    id: resolved.id.clone(),
                    format_version: resolved.format_version.clone(),
                    canonical_bytes: resolved.canonical_bytes.clone(),
                });
            let result = OperationResult::SnapshotAnalysed { run: Box::new(run) };
            let response = success_response(request_id.into(), result.clone());
            let receipt = operation_receipt(request_id, request_json, &response)
                .map_err(repository_message)?;
            repository
                .attach_evidence_with_snapshot_and_operation_receipt(
                    &stored_evidence,
                    stored_resolved.as_ref(),
                    &receipt,
                )
                .map_err(sqlite_error)?;
            Ok(SqliteExecution {
                result,
                receipt_persisted: true,
            })
        }
        Operation::InspectAnalysisEvidence {
            evidence_id,
            against_revision_id,
        } => {
            let stored = repository.evidence(&evidence_id).map_err(sqlite_error)?;
            let evidence: AnalysisEvidence =
                serde_json::from_str(&stored.manifest_json).map_err(invalid_json)?;
            let target_revision = repository
                .revision(&against_revision_id)
                .map_err(sqlite_error)?;
            let source = repository
                .hydrate_snapshot(evidence.authored_snapshot_id())
                .map_err(sqlite_error)?;
            let target = repository
                .hydrate_snapshot(&target_revision.snapshot_id)
                .map_err(sqlite_error)?;
            let dependencies = current_dependencies(&evidence, &target_revision.snapshot_id)?;
            let staleness = staleness_for(
                &evidence,
                &target_revision.snapshot_id,
                &dependencies,
                &semantic_diff(source.model(), target.model()),
            );
            Ok(SqliteExecution {
                result: OperationResult::AnalysisEvidenceInspection {
                    evidence: Box::new(evidence),
                    against_revision_id,
                    staleness,
                },
                receipt_persisted: false,
            })
        }
    }
}

fn publish_canonical_design_run(
    context: &DesignRunOperationContext,
    revision_id: &RevisionId,
    interpretation_dependencies: &fraia_core::DesignRunInterpretationDependencies,
    run: &mut SnapshotAnalysisRun,
) -> Result<(), OperationError> {
    run.evidence
        .bind_interpretation_dependencies(interpretation_dependencies)
        .map_err(|error| {
            repository_message(format!("bind interpretation dependencies: {error}"))
        })?;
    let published = crate::design_run_adapter::publish_analysis_evidence_design_run(
        crate::design_run_adapter::PublishAnalysisEvidenceDesignRun {
            project_dir: &context.project_dir,
            project_id: context.project_id.clone(),
            design_id: context.design_id.clone(),
            revision_id,
            evidence: &mut run.evidence,
            actor: context.actor.clone(),
            created_at: context.created_at.clone(),
            parent_run_id: context.parent_run_id.clone(),
        },
    )
    .map_err(|error| repository_message(format!("publish canonical design run: {error}")))?;
    run.canonical_run_id = Some(published.run_id);
    Ok(())
}

fn interpretation_dependencies_for_revision(
    repository: &SqliteRevisionRepository,
    revision_id: &RevisionId,
) -> Result<fraia_core::DesignRunInterpretationDependencies, OperationError> {
    #[derive(Default, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct RevisionMetadata {
        #[serde(default)]
        agent_provenance: Option<AgentTurnProvenance>,
    }
    let stored = repository.revision(revision_id).map_err(sqlite_error)?;
    let metadata: RevisionMetadata =
        serde_json::from_str(&stored.metadata_json).map_err(invalid_json)?;
    let Some(provenance) = metadata.agent_provenance else {
        return Ok(Default::default());
    };
    let mut dependencies = fraia_core::DesignRunInterpretationDependencies {
        revision_ids: provenance.drawing_interpretation_revision_ids,
        inference_ids: provenance.drawing_interpretation_inference_ids,
    };
    dependencies.revision_ids.sort();
    dependencies.revision_ids.dedup();
    dependencies.inference_ids.sort();
    dependencies.inference_ids.dedup();
    Ok(dependencies)
}

fn capabilities_result() -> OperationResult {
    OperationResult::Capabilities {
        contract_version: OPERATION_CONTRACT_VERSION.into(),
        operations: vec![
            "capabilities".into(),
            "inspect".into(),
            "propose_structural_patch".into(),
            "accept_structural_patch".into(),
            "reject_structural_patch".into(),
            "validate_snapshot".into(),
            "analyse_snapshot".into(),
            "inspect_analysis_evidence".into(),
        ],
        features: vec![
            "sqlite_durable_execution".into(),
            "exact_head_transactions".into(),
            "idempotent_request_replay".into(),
            "typed_structural_patches".into(),
            "patch_node".into(),
            "patch_member".into(),
            "patch_plate".into(),
            "patch_support".into(),
            "patch_load".into(),
            "patch_release".into(),
            "immutable_analysis_evidence".into(),
            "snapshot_bound_validation".into(),
        ],
    }
}

fn exact_memory_snapshot<'a>(
    repository: &'a InMemoryRevisionRepository,
    revision_id: &RevisionId,
    expected_snapshot_id: &SnapshotId,
) -> Result<&'a ModelSnapshot, OperationError> {
    let revision = repository.revision(revision_id).map_err(repository_error)?;
    if revision.snapshot_id() != expected_snapshot_id {
        return Err(expected_snapshot_error(
            revision_id.clone(),
            expected_snapshot_id.clone(),
            revision.snapshot_id().clone(),
        ));
    }
    repository
        .snapshot(expected_snapshot_id)
        .map_err(repository_error)
}

fn exact_sqlite_snapshot(
    repository: &SqliteRevisionRepository,
    revision_id: &RevisionId,
    expected_snapshot_id: &SnapshotId,
) -> Result<ModelSnapshot, OperationError> {
    let revision = repository.revision(revision_id).map_err(sqlite_error)?;
    if revision.snapshot_id != *expected_snapshot_id {
        return Err(expected_snapshot_error(
            revision_id.clone(),
            expected_snapshot_id.clone(),
            revision.snapshot_id,
        ));
    }
    repository
        .hydrate_snapshot(expected_snapshot_id)
        .map_err(sqlite_error)
}

fn current_dependencies(
    evidence: &AnalysisEvidence,
    target_snapshot_id: &SnapshotId,
) -> Result<Vec<crate::evidence::EvidenceDependency>, OperationError> {
    let manifest = evidence.analysis_manifest().ok_or_else(|| {
        invalid_request(format!(
            "evidence `{}` has no analysis manifest",
            evidence.id()
        ))
    })?;
    let settings: AnalysisSettings =
        serde_json::from_str(manifest.settings_payload()).map_err(invalid_json)?;
    Ok(dependencies_for_snapshot_with_settings(
        target_snapshot_id,
        &settings,
    ))
}

fn ensure_analysis_replay(
    evidence: &AnalysisEvidence,
    expected_snapshot_id: &SnapshotId,
    settings: &AnalysisSettings,
) -> Result<(), OperationError> {
    if evidence.authored_snapshot_id() != expected_snapshot_id {
        return Err(invalid_request(format!(
            "evidence `{}` belongs to snapshot `{}`, not `{expected_snapshot_id}`",
            evidence.id(),
            evidence.authored_snapshot_id()
        )));
    }
    let actual_settings = evidence.settings_identity().ok_or_else(|| {
        invalid_request(format!(
            "evidence `{}` has no analysis settings identity",
            evidence.id()
        ))
    })?;
    let expected_settings = settings
        .identity()
        .map_err(|error| invalid_request(error.to_string()))?;
    if actual_settings != expected_settings {
        return Err(invalid_request(format!(
            "evidence `{}` uses settings `{actual_settings}`, not `{expected_settings}`",
            evidence.id()
        )));
    }
    Ok(())
}

fn run_from_evidence(
    revision_id: RevisionId,
    evidence: AnalysisEvidence,
    resolved: Option<StoredSnapshot>,
) -> Result<SnapshotAnalysisRun, OperationError> {
    let manifest = evidence.analysis_manifest().ok_or_else(|| {
        invalid_request(format!(
            "evidence `{}` has no analysis manifest",
            evidence.id()
        ))
    })?;
    let outcome = match manifest.status {
        crate::evidence::AnalysisEvidenceStatus::Completed => {
            let metrics = manifest
                .metrics
                .clone()
                .ok_or_else(|| invalid_request("completed evidence has no metrics".into()))?;
            SnapshotAnalysisOutcome::Completed {
                resolved_snapshot_id: evidence.resolved_snapshot_id().cloned().ok_or_else(
                    || invalid_request("completed evidence has no resolved snapshot".into()),
                )?,
                input_hash: manifest.input_hash.clone().ok_or_else(|| {
                    invalid_request("completed evidence has no input identity".into())
                })?,
                result_hash: manifest.result_hash.clone().ok_or_else(|| {
                    invalid_request("completed evidence has no result identity".into())
                })?,
                combo_count: metrics.combo_metrics.len(),
                metrics,
            }
        }
        crate::evidence::AnalysisEvidenceStatus::Unsupported => {
            SnapshotAnalysisOutcome::Unsupported {
                diagnostics: manifest.diagnostics.clone(),
            }
        }
        crate::evidence::AnalysisEvidenceStatus::Failed => SnapshotAnalysisOutcome::Failed {
            diagnostics: manifest.diagnostics.clone(),
        },
    };
    let resolved_snapshot =
        resolved.map(|snapshot| crate::analysis_service::ResolvedSnapshotRecord {
            id: snapshot.id,
            format_version: snapshot.format_version,
            canonical_bytes: snapshot.canonical_bytes,
        });
    Ok(SnapshotAnalysisRun {
        revision_id,
        canonical_run_id: evidence.canonical_run_id().map(str::to_owned),
        evidence,
        outcome,
        resolved_snapshot,
    })
}

fn invalid_request(message: String) -> OperationError {
    OperationError {
        code: OperationErrorCode::InvalidRequest,
        message,
        head_conflict: None,
        snapshot_conflict: None,
    }
}

fn expected_snapshot_error(
    revision_id: RevisionId,
    expected_snapshot_id: SnapshotId,
    actual_snapshot_id: SnapshotId,
) -> OperationError {
    OperationError {
        code: OperationErrorCode::ExpectedSnapshotMismatch,
        message: format!(
            "revision `{revision_id}` snapshot is `{actual_snapshot_id}`, not expected `{expected_snapshot_id}`"
        ),
        head_conflict: None,
        snapshot_conflict: Some(SnapshotConflict {
            revision_id,
            expected_snapshot_id,
            actual_snapshot_id,
        }),
    }
}

fn success_response(request_id: String, result: OperationResult) -> OperationResponse {
    OperationResponse {
        contract_version: OPERATION_CONTRACT_VERSION.into(),
        request_id,
        outcome: OperationOutcome::Success {
            result: Box::new(result),
        },
    }
}

fn operation_receipt(
    request_id: &str,
    request_json: impl Into<String>,
    response: &OperationResponse,
) -> Result<StoredOperationReceipt, String> {
    Ok(StoredOperationReceipt {
        request_id: request_id.into(),
        request_json: request_json.into(),
        response_json: serde_json::to_string(response).map_err(|error| error.to_string())?,
    })
}

fn repository_message(message: String) -> OperationError {
    OperationError {
        code: OperationErrorCode::RepositoryError,
        message,
        head_conflict: None,
        snapshot_conflict: None,
    }
}

fn sqlite_repository_response(
    request_id: String,
    error: SqliteRepositoryError,
) -> OperationResponse {
    error_response(
        request_id,
        OperationErrorCode::RepositoryError,
        error.to_string(),
        None,
    )
}

fn invalid_json(error: serde_json::Error) -> OperationError {
    OperationError {
        code: OperationErrorCode::InvalidRequest,
        message: error.to_string(),
        head_conflict: None,
        snapshot_conflict: None,
    }
}

fn sqlite_error(error: SqliteRepositoryError) -> OperationError {
    match error {
        SqliteRepositoryError::ExpectedHeadConflict {
            conversation_id,
            expected_revision_id,
            actual_revision_id,
        } => expected_head_error(conversation_id, expected_revision_id, actual_revision_id),
        SqliteRepositoryError::ConflictingProposal(_)
        | SqliteRepositoryError::ConflictingOperationRequest(_) => OperationError {
            code: OperationErrorCode::InvalidRequest,
            message: error.to_string(),
            head_conflict: None,
            snapshot_conflict: None,
        },
        error => OperationError {
            code: OperationErrorCode::RepositoryError,
            message: error.to_string(),
            head_conflict: None,
            snapshot_conflict: None,
        },
    }
}

fn error_response(
    request_id: String,
    code: OperationErrorCode,
    message: String,
    head_conflict: Option<HeadConflict>,
) -> OperationResponse {
    OperationResponse {
        contract_version: OPERATION_CONTRACT_VERSION.into(),
        request_id,
        outcome: OperationOutcome::Error {
            error: OperationError {
                code,
                message,
                head_conflict,
                snapshot_conflict: None,
            },
        },
    }
}

fn ensure_expected_head(
    repository: &InMemoryRevisionRepository,
    conversation_id: &ConversationId,
    expected_revision_id: &RevisionId,
) -> Result<(), OperationError> {
    let head = repository.head(conversation_id).map_err(repository_error)?;
    if head.head_revision_id == *expected_revision_id {
        return Ok(());
    }
    Err(expected_head_error(
        conversation_id.clone(),
        expected_revision_id.clone(),
        head.head_revision_id,
    ))
}

fn repository_error(error: RepositoryError) -> OperationError {
    match error {
        RepositoryError::Conversation(ConversationError::ExpectedHeadMismatch {
            conversation_id,
            expected_revision_id,
            actual_revision_id,
        }) => expected_head_error(conversation_id, expected_revision_id, actual_revision_id),
        RepositoryError::Patch(error) => OperationError {
            code: OperationErrorCode::InvalidPatch,
            message: error.to_string(),
            head_conflict: None,
            snapshot_conflict: None,
        },
        error => OperationError {
            code: OperationErrorCode::RepositoryError,
            message: error.to_string(),
            head_conflict: None,
            snapshot_conflict: None,
        },
    }
}

fn expected_head_error(
    conversation_id: ConversationId,
    expected_revision_id: RevisionId,
    actual_revision_id: RevisionId,
) -> OperationError {
    OperationError {
        code: OperationErrorCode::ExpectedHeadMismatch,
        message: format!(
            "conversation `{conversation_id}` head is `{actual_revision_id}`, not expected `{expected_revision_id}`"
        ),
        head_conflict: Some(HeadConflict {
            conversation_id,
            expected_revision_id,
            actual_revision_id,
        }),
        snapshot_conflict: None,
    }
}
