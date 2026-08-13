//! Isolated HTTP/service adapter for the conversation-first spike.
//! It deliberately owns no legacy project state and performs no LLM calls.

use axum::{Extension, Json, Router, http::StatusCode, response::IntoResponse, routing::post};
use fraia_app_api::*;
use fraia_core::{
    AssignmentTargetRef, LoadVector, MemberEnd, MemberEndTarget, ReleaseAssignment,
    StructuralMember, StructuralModel, StructuralPlate, SupportAssignment,
    understand_structural_model,
};
use fraia_revision::{
    ConversationId, EvidenceId, ProjectId, RevisionId,
    agent_contract::AgentTurnProvenance,
    analysis_service::{
        ResolvedSnapshotRecord, SnapshotAnalysisOutcome, SnapshotAnalysisRun,
        analyse_accepted_revision, compare_completed_runs,
    },
    conversation::ConversationOrigin,
    diff::SemanticDiff,
    evidence::{
        AnalysisEvidence, AnalysisEvidenceManifest, AnalysisEvidenceStatus, AnalysisMetrics,
        EvidenceDependency,
    },
    patch::{
        ForceUnit, Length, LineLoadUnit, LoadInput, LoadMagnitude, MemberRole, Position,
        PressureUnit, StructuralOperation, StructuralPatch,
    },
    repository::{
        InMemoryRevisionRepository, ProposalId, ProposalStatus, RevisionAuthorKind,
        RevisionOperation, RevisionRecord,
    },
    snapshot::ModelSnapshot,
    sqlite::{
        SqliteRevisionRepository, StoredConversation, StoredEvidence, StoredProjectRoot,
        StoredProposal, StoredRevision, StoredSnapshot,
    },
    working_copy::WorkingCopy,
};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

/// The SQLite schema stores an opaque origin JSON value. Keep the transport's
/// onboarding facts beside the revision origin so a restarted appd instance
/// can restore the same conversation-facing state without changing the shared
/// revision schema.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedConversationState {
    #[serde(default)]
    project_facts: ConversationProjectFacts,
    #[serde(default)]
    messages: Vec<String>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedConversationEnvelope {
    #[serde(default)]
    origin: Option<ConversationOrigin>,
    #[serde(flatten)]
    state: PersistedConversationState,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedAgentProvenance {
    provider: String,
    model: String,
    turn_id: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedRevisionMetadata {
    #[serde(default)]
    author: String,
    #[serde(default)]
    semantic_diff: SemanticDiff,
    #[serde(default)]
    operation: Option<RevisionOperation>,
    #[serde(default)]
    agent_provenance: Option<PersistedAgentProvenance>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedEvidenceEnvelope {
    #[serde(default)]
    dependencies: Vec<EvidenceDependency>,
    #[serde(default)]
    analysis_manifest: Option<AnalysisEvidenceManifest>,
}

pub struct ConversationService {
    projects: BTreeMap<ProjectId, ProjectConversationStore>,
}
struct ProjectConversationStore {
    repository: InMemoryRevisionRepository,
    project_facts: ConversationProjectFacts,
    sqlite: Option<SqliteRevisionRepository>,
    sqlite_path: Option<PathBuf>,
    messages: BTreeMap<ConversationId, Vec<String>>,
    working_copies: BTreeMap<String, WorkingCopy>,
    analysis_runs: BTreeMap<EvidenceId, SnapshotAnalysisRun>,
}
pub type ConversationServiceHandle = Arc<Mutex<ConversationService>>;

impl Default for ConversationService {
    fn default() -> Self {
        Self {
            projects: BTreeMap::new(),
        }
    }
}

impl ConversationService {
    /// Opens the durable appd transport repository. Electron remains only a
    /// client; revisions and evidence live in the local Rust-owned SQLite DB.
    pub fn open_durable(_legacy_path: impl AsRef<Path>) -> Result<Self, String> {
        Ok(Self::default())
    }
    pub fn create(
        &mut self,
        mut request: ConversationCreateRequest,
    ) -> Result<ConversationStateResponse, String> {
        if self.projects.contains_key(&request.project_id) {
            return self.state(&request.project_id, &request.conversation_id);
        }
        if request.purpose.trim().is_empty() {
            request.purpose = "Overall design".into();
        }
        let path = project_workspace_database(&request.project_dir)?;
        if path.exists() {
            if let Ok(store) = hydrate_project(&path, &request.project_id) {
                self.projects.insert(request.project_id.clone(), store);
                return self.state(&request.project_id, &request.conversation_id);
            }
        }
        std::fs::create_dir_all(path.parent().expect("workspace database has a parent"))
            .map_err(display)?;
        let root_revision_id = RevisionId::new(format!("{}:root", request.project_id));
        let repository = InMemoryRevisionRepository::create(
            request.project_id.clone(),
            request.conversation_id.clone(),
            request.purpose,
            root_revision_id,
            StructuralModel::empty(),
        )
        .map_err(display)?;
        let mut store = ProjectConversationStore {
            repository,
            project_facts: request.project_facts,
            sqlite: Some(SqliteRevisionRepository::open(&path).map_err(display)?),
            sqlite_path: Some(path),
            messages: BTreeMap::new(),
            working_copies: BTreeMap::new(),
            analysis_runs: BTreeMap::new(),
        };
        if let Some(sqlite) = store.sqlite.as_mut() {
            persist_root(
                sqlite,
                &request.project_id,
                &store.repository,
                &request.conversation_id,
                &store.project_facts,
            )?;
        }
        self.projects.insert(request.project_id.clone(), store);
        self.state(&request.project_id, &request.conversation_id)
    }
    pub fn converse(
        &mut self,
        request: ConversationMessageRequest,
    ) -> Result<ConversationStateResponse, String> {
        let store = self.store_mut(&request.project_id)?;
        store
            .repository
            .conversation(&request.conversation_id)
            .map_err(display)?;
        store
            .messages
            .entry(request.conversation_id.clone())
            .or_default()
            .push(request.message);
        if let Some(sqlite) = store.sqlite.as_mut() {
            let conversation = store
                .repository
                .conversation(&request.conversation_id)
                .map_err(display)?;
            let messages = store
                .messages
                .get(&request.conversation_id)
                .cloned()
                .unwrap_or_default();
            sqlite
                .update_conversation_origin(
                    &request.conversation_id,
                    &persisted_origin_json(conversation.origin(), &store.project_facts, &messages)?,
                )
                .map_err(display)?;
        }
        self.state(&request.project_id, &request.conversation_id)
    }
    pub fn update_facts(
        &mut self,
        request: ConversationFactsUpdateRequest,
    ) -> Result<ConversationStateResponse, String> {
        let store = self.store_mut(&request.project_id)?;
        store
            .repository
            .conversation(&request.conversation_id)
            .map_err(display)?;
        store.project_facts = request.project_facts;
        if let Some(sqlite) = store.sqlite.as_mut() {
            let root = sqlite
                .project_root(&request.project_id)
                .map_err(display)?
                .root_conversation;
            let conversation = store.repository.conversation(&root.id).map_err(display)?;
            sqlite
                .update_conversation_origin(
                    &root.id,
                    &persisted_origin_json(
                        conversation.origin(),
                        &store.project_facts,
                        store
                            .messages
                            .get(&root.id)
                            .map(Vec::as_slice)
                            .unwrap_or(&[]),
                    )?,
                )
                .map_err(display)?;
        }
        self.state(&request.project_id, &request.conversation_id)
    }
    /// Explicit demo/test helper. Production conversation creation always
    /// begins from an empty authored model.
    #[cfg(test)]
    pub fn create_fixture_demo(
        &mut self,
        mut request: ConversationCreateRequest,
    ) -> Result<ConversationStateResponse, String> {
        use fraia_revision::root_fixture;
        if self.projects.contains_key(&request.project_id) {
            return Err("project already exists in the conversation repository".into());
        }
        let fixture = root_fixture();
        let repository = InMemoryRevisionRepository::create(
            request.project_id.clone(),
            request.conversation_id.clone(),
            request.purpose,
            fixture.root_revision_id,
            fixture.model,
        )
        .map_err(display)?;
        self.projects.insert(
            request.project_id.clone(),
            ProjectConversationStore {
                repository,
                project_facts: std::mem::take(&mut request.project_facts),
                sqlite: None,
                sqlite_path: None,
                messages: BTreeMap::new(),
                working_copies: BTreeMap::new(),
                analysis_runs: BTreeMap::new(),
            },
        );
        self.state(&request.project_id, &request.conversation_id)
    }
    pub fn propose(&mut self, request: ConversationProposalRequest) -> Result<(), String> {
        let agent_provenance = AgentTurnProvenance {
            provider: required_provenance(Some(request.provider.clone()), "provider")?,
            model: required_provenance(Some(request.model.clone()), "model")?,
            turn_id: required_provenance(Some(request.turn_id.clone()), "turn")?,
        };
        let mut requested_operations = request.operations;
        if let Some(operation) = request.operation {
            requested_operations.push(operation);
        }
        if requested_operations.is_empty() {
            return Err("proposal requires at least one typed operation".into());
        }
        let operations = requested_operations
            .into_iter()
            .map(transport_operation)
            .collect::<Result<Vec<_>, _>>()?;
        let store = self.store_mut(&request.project_id)?;
        let parent_revision_id =
            normalize_legacy_root_revision(&store.repository, request.parent_revision_id);
        let proposal_id = ProposalId::new(request.proposal_id.clone());
        let patch = StructuralPatch { operations };
        let conversation_id = request.conversation_id;
        let proposed_revision_id = request.proposed_revision_id;
        if let Some(mut sqlite) = store.sqlite.take() {
            let mut candidate_repository = store.repository.clone();
            if let Err(error) = candidate_repository.create_proposal_with_provenance(
                proposal_id.clone(),
                conversation_id.clone(),
                parent_revision_id.clone(),
                proposed_revision_id.clone(),
                patch.clone(),
                agent_provenance.clone(),
            ) {
                store.sqlite = Some(sqlite);
                return Err(display(error));
            }
            let stored = stored_proposal(
                &request.project_id,
                &proposal_id,
                &conversation_id,
                &parent_revision_id,
                &proposed_revision_id,
                &patch,
                Some(&agent_provenance),
            )?;
            if let Err(error) = sqlite.insert_proposal(&stored) {
                store.sqlite = Some(sqlite);
                return Err(display(error));
            }
            store.repository = candidate_repository;
            store.sqlite = Some(sqlite);
        } else {
            store
                .repository
                .create_proposal_with_provenance(
                    proposal_id,
                    conversation_id,
                    parent_revision_id,
                    proposed_revision_id,
                    patch,
                    agent_provenance,
                )
                .map_err(display)?;
        }
        Ok(())
    }
    pub fn accept(
        &mut self,
        request: ConversationProposalActionRequest,
    ) -> Result<ConversationRevisionResponse, String> {
        let store = self.store_mut(&request.project_id)?;
        let proposal_id = ProposalId::new(request.proposal_id.clone());
        let proposal = store
            .repository
            .proposal(&proposal_id)
            .map_err(display)?
            .clone();
        let provenance = ConversationAgentProvenance {
            provider: required_provenance(request.provider.clone(), "provider")?,
            model: required_provenance(request.model.clone(), "model")?,
            turn_id: required_provenance(request.turn_id.clone(), "turn")?,
        };
        if let Some(mut sqlite) = store.sqlite.take() {
            let mut candidate_repository = store.repository.clone();
            let record = match candidate_repository.accept_proposal_with_provenance(
                &proposal_id,
                Some(AgentTurnProvenance {
                    provider: provenance.provider.clone(),
                    model: provenance.model.clone(),
                    turn_id: provenance.turn_id.clone(),
                }),
            ) {
                Ok(record) => record.clone(),
                Err(error) => {
                    store.sqlite = Some(sqlite);
                    return Err(display(error));
                }
            };
            if let Err(error) = persist_agent_revision(
                &mut sqlite,
                &candidate_repository,
                &record,
                Some(proposal.patch()),
                &proposal_id,
                &provenance,
            ) {
                store.sqlite = Some(sqlite);
                return Err(error);
            }
            let response = revision_response(&record);
            store.repository = candidate_repository;
            store.sqlite = Some(sqlite);
            return Ok(response);
        }
        let record = store
            .repository
            .accept_proposal_with_provenance(
                &proposal_id,
                Some(AgentTurnProvenance {
                    provider: provenance.provider,
                    model: provenance.model,
                    turn_id: provenance.turn_id,
                }),
            )
            .map_err(display)?;
        let response = revision_response(record);
        Ok(response)
    }
    pub fn reject(&mut self, request: ConversationProposalActionRequest) -> Result<(), String> {
        let store = self.store_mut(&request.project_id)?;
        let proposal_id = ProposalId::new(request.proposal_id);
        if let Some(mut sqlite) = store.sqlite.take() {
            let mut candidate_repository = store.repository.clone();
            if let Err(error) = candidate_repository.reject_proposal(&proposal_id) {
                store.sqlite = Some(sqlite);
                return Err(display(error));
            }
            if let Err(error) = sqlite.reject_proposal(&proposal_id) {
                store.sqlite = Some(sqlite);
                return Err(display(error));
            }
            store.repository = candidate_repository;
            store.sqlite = Some(sqlite);
            Ok(())
        } else {
            store
                .repository
                .reject_proposal(&proposal_id)
                .map_err(display)
        }
    }
    pub fn fork(
        &mut self,
        request: ConversationForkRequest,
        resume: bool,
    ) -> Result<ConversationStateResponse, String> {
        let store = self.store_mut(&request.project_id)?;
        if resume {
            store
                .repository
                .resume(
                    request.conversation_id.clone(),
                    request.purpose,
                    request.from_revision_id,
                )
                .map_err(display)?;
        } else {
            store
                .repository
                .fork(
                    request.conversation_id.clone(),
                    request.purpose,
                    request.from_revision_id,
                )
                .map_err(display)?;
        }
        if let Some(sqlite) = store.sqlite.as_mut() {
            let conversation = store
                .repository
                .conversation(&request.conversation_id)
                .map_err(display)?;
            sqlite
                .create_conversation(&StoredConversation {
                    id: conversation.id().clone(),
                    project_id: conversation.project_id().clone(),
                    purpose: conversation.purpose().to_owned(),
                    origin_json: persisted_origin_json(
                        conversation.origin(),
                        &store.project_facts,
                        &[],
                    )?,
                    head_revision_id: conversation.head_revision_id().clone(),
                })
                .map_err(display)?;
        }
        self.state(&request.project_id, &request.conversation_id)
    }
    pub fn analyse(
        &mut self,
        request: ConversationAnalysisRequest,
    ) -> Result<ConversationEvidenceResponse, String> {
        let store = self.store_mut(&request.project_id)?;
        store
            .repository
            .conversation(&request.conversation_id)
            .map_err(display)?;
        let revision = store
            .repository
            .revision(&request.revision_id)
            .map_err(display)?;
        let snapshot_id = revision.snapshot_id().clone();
        if let Some(sqlite) = store.sqlite.as_ref() {
            if let Ok(stored) = sqlite.evidence(&request.evidence_id) {
                if stored.authored_snapshot_id != snapshot_id {
                    return Err(format!(
                        "analysis evidence `{}` is already bound to snapshot `{}`",
                        request.evidence_id, stored.authored_snapshot_id
                    ));
                }
                return restored_evidence_response(stored);
            }
        }
        let mut candidate_repository = store.repository.clone();
        let run = analyse_accepted_revision(
            &mut candidate_repository,
            &request.revision_id,
            request.evidence_id.clone(),
        )
        .map_err(display)?;
        let (status, summary, diagnostics) = match &run.outcome {
            SnapshotAnalysisOutcome::Completed { .. } => (
                "success".to_owned(),
                "Analysis completed against the accepted snapshot.".to_owned(),
                Vec::new(),
            ),
            SnapshotAnalysisOutcome::Unsupported { diagnostics } => (
                "unsupported".to_owned(),
                "This accepted snapshot is not supported by the available analysis path."
                    .to_owned(),
                diagnostics.clone(),
            ),
            SnapshotAnalysisOutcome::Failed { diagnostics } => (
                "failed".to_owned(),
                "Analysis failed for the accepted snapshot; no result was fabricated.".to_owned(),
                diagnostics.clone(),
            ),
        };
        let manifest = run.evidence.analysis_manifest();
        let evidence = candidate_repository
            .evidence(&request.evidence_id)
            .map_err(display)?;
        if let Some(mut sqlite) = store.sqlite.take() {
            let resolved = run
                .resolved_snapshot
                .as_ref()
                .map(|snapshot| StoredSnapshot {
                    id: snapshot.id.clone(),
                    format_version: snapshot.format_version.clone(),
                    canonical_bytes: snapshot.canonical_bytes.clone(),
                });
            if let Err(error) =
                sqlite.attach_evidence_with_snapshot(&stored_evidence(evidence)?, resolved.as_ref())
            {
                store.sqlite = Some(sqlite);
                return Err(display(error));
            }
            store.repository = candidate_repository;
            store.sqlite = Some(sqlite);
        } else {
            store.repository = candidate_repository;
        }
        store
            .analysis_runs
            .insert(request.evidence_id.clone(), run.clone());
        Ok(ConversationEvidenceResponse {
            evidence_id: request.evidence_id,
            authored_snapshot_id: snapshot_id,
            stale: false,
            status,
            summary,
            resolved_snapshot_id: run.evidence.resolved_snapshot_id().cloned(),
            input_hash: manifest.and_then(|value| value.input_hash.clone()),
            result_hash: manifest.and_then(|value| value.result_hash.clone()),
            solver_identity: manifest.map(|value| value.solver_identity.clone()),
            metrics: manifest
                .and_then(|value| value.metrics.as_ref().map(analysis_metrics_response)),
            diagnostics,
        })
    }
    pub fn compare(
        &mut self,
        request: ConversationComparisonRequest,
    ) -> Result<ConversationComparisonResponse, String> {
        let store = self.store_mut(&request.project_id)?;
        store
            .repository
            .conversation(&request.conversation_id)
            .map_err(display)?;
        let baseline = store
            .analysis_runs
            .get(&request.baseline_evidence_id)
            .ok_or_else(|| {
                "baseline evidence is not available in this live analysis session".to_string()
            })?;
        let candidate = store
            .analysis_runs
            .get(&request.candidate_evidence_id)
            .ok_or_else(|| {
                "candidate evidence is not available in this live analysis session".to_string()
            })?;
        let comparison = compare_completed_runs(baseline, candidate).map_err(display)?;
        Ok(ConversationComparisonResponse {
            solver_identity: comparison.solver_identity,
            runtime_identity: comparison.runtime_identity,
            settings_identity: comparison.settings_identity,
            settings_payload: comparison.settings_payload,
            request: serde_json::to_value(comparison.request).map_err(display)?,
            baseline: comparison_entry_response(comparison.baseline),
            candidate: comparison_entry_response(comparison.candidate),
        })
    }
    pub fn evidence(
        &self,
        project: &ProjectId,
        evidence: &EvidenceId,
        revision: &RevisionId,
    ) -> Result<ConversationEvidenceResponse, String> {
        let Ok(store) = self.store(project) else {
            return self.durable_evidence_response(project, evidence, revision);
        };
        let target = store.repository.revision(revision).map_err(display)?;
        let evidence_record = match store.repository.evidence(evidence) {
            Ok(evidence_record) => evidence_record,
            Err(_) => return self.durable_evidence_response(project, evidence, revision),
        };
        let stale = evidence_record.authored_snapshot_id() != target.snapshot_id();
        Ok(ConversationEvidenceResponse {
            evidence_id: evidence.clone(),
            authored_snapshot_id: evidence_record.authored_snapshot_id().clone(),
            stale,
            status: if stale { "stale" } else { "current" }.into(),
            summary: if stale {
                "Evidence is stale for the selected revision.".into()
            } else {
                "Evidence is current for the selected revision.".into()
            },
            resolved_snapshot_id: evidence_record.resolved_snapshot_id().cloned(),
            input_hash: None,
            result_hash: None,
            solver_identity: None,
            metrics: evidence_record
                .analysis_manifest()
                .and_then(|value| value.metrics.as_ref().map(analysis_metrics_response)),
            diagnostics: Vec::new(),
        })
    }
    pub fn open_working_copy(
        &mut self,
        request: ConversationWorkingCopyOpenRequest,
    ) -> Result<ConversationWorkingCopyOpenResponse, String> {
        let store = self.store_mut(&request.project_id)?;
        let revision_id = normalize_legacy_root_revision(&store.repository, request.revision_id);
        let working_copy = store
            .repository
            .open_working_copy(&revision_id)
            .map_err(display)?;
        let working_copy_id = format!("working-copy-{}-{}", request.conversation_id, revision_id);
        let response = ConversationWorkingCopyOpenResponse {
            working_copy_id: working_copy_id.clone(),
            source_revision_id: working_copy.parent_revision_id().clone(),
            source_snapshot_id: working_copy.source_snapshot_id().clone(),
        };
        store.working_copies.insert(working_copy_id, working_copy);
        Ok(response)
    }
    pub fn apply_working_copy_operation(
        &mut self,
        request: ConversationWorkingCopyOperationRequest,
    ) -> Result<(), String> {
        let operation = transport_operation(request.operation)?;
        let store = self.store_mut(&request.project_id)?;
        let working_copy = store
            .working_copies
            .get_mut(&request.working_copy_id)
            .ok_or_else(|| "unknown working copy".to_string())?;
        working_copy
            .apply(&StructuralPatch {
                operations: vec![operation],
            })
            .map_err(display)?;
        Ok(())
    }
    pub fn commit_working_copy(
        &mut self,
        request: ConversationWorkingCopyCommitRequest,
    ) -> Result<ConversationRevisionResponse, String> {
        let store = self.store_mut(&request.project_id)?;
        let working_copy = store
            .working_copies
            .remove(&request.working_copy_id)
            .ok_or_else(|| "unknown working copy".to_string())?;
        if let Some(mut sqlite) = store.sqlite.take() {
            let mut candidate_repository = store.repository.clone();
            let mut candidate_working_copy = working_copy.clone();
            let record = match candidate_repository.commit_working_copy(
                &request.conversation_id,
                &mut candidate_working_copy,
                request.revision_id,
            ) {
                Ok(record) => record.clone(),
                Err(error) => {
                    store
                        .working_copies
                        .insert(request.working_copy_id, working_copy);
                    store.sqlite = Some(sqlite);
                    return Err(display(error));
                }
            };
            if let Err(error) = persist_revision(&mut sqlite, &candidate_repository, &record, None)
            {
                store
                    .working_copies
                    .insert(request.working_copy_id, working_copy);
                store.sqlite = Some(sqlite);
                return Err(error);
            }
            let response = revision_response(&record);
            store.repository = candidate_repository;
            store.sqlite = Some(sqlite);
            Ok(response)
        } else {
            let mut working_copy = working_copy;
            match store.repository.commit_working_copy(
                &request.conversation_id,
                &mut working_copy,
                request.revision_id,
            ) {
                Ok(record) => Ok(revision_response(record)),
                Err(error) => {
                    store
                        .working_copies
                        .insert(request.working_copy_id, working_copy);
                    Err(display(error))
                }
            }
        }
    }
    fn state(
        &self,
        project: &ProjectId,
        conversation: &ConversationId,
    ) -> Result<ConversationStateResponse, String> {
        let store = self.store(project)?;
        let head = store.repository.head(conversation).map_err(display)?;
        let revision = store
            .repository
            .revision(&head.head_revision_id)
            .map_err(display)?;
        let snapshot = store
            .repository
            .snapshot(revision.snapshot_id())
            .map_err(display)?;
        Ok(ConversationStateResponse {
            project_id: project.clone(),
            conversation_id: conversation.clone(),
            purpose: head.purpose,
            head_revision_id: head.head_revision_id,
            head_snapshot_id: head.head_snapshot_id,
            project_facts: store.project_facts.clone(),
            semantic_summary: understand_structural_model(snapshot.model()),
            messages: store
                .messages
                .get(conversation)
                .cloned()
                .unwrap_or_default(),
        })
    }
    fn store(&self, id: &ProjectId) -> Result<&ProjectConversationStore, String> {
        self.projects
            .get(id)
            .ok_or_else(|| "unknown conversation project".into())
    }
    fn store_mut(&mut self, id: &ProjectId) -> Result<&mut ProjectConversationStore, String> {
        if !self.projects.contains_key(id) {
            return Err("unknown conversation project; open the Fraia document first".into());
        }
        self.projects
            .get_mut(id)
            .ok_or_else(|| "unknown conversation project".into())
    }
    fn durable_repository(&self, project: &ProjectId) -> Result<SqliteRevisionRepository, String> {
        let path = self
            .store(project)?
            .sqlite_path
            .as_ref()
            .ok_or_else(|| "conversation project has no durable workspace".to_string())?;
        SqliteRevisionRepository::open(path).map_err(display)
    }

    fn durable_evidence_response(
        &self,
        project: &ProjectId,
        evidence: &EvidenceId,
        revision: &RevisionId,
    ) -> Result<ConversationEvidenceResponse, String> {
        let sqlite = self.durable_repository(project)?;
        let stored = sqlite.evidence(evidence).map_err(display)?;
        let stale = sqlite
            .evidence_is_stale_for_revision(evidence, revision)
            .map_err(display)?;
        Ok(ConversationEvidenceResponse {
            evidence_id: evidence.clone(),
            authored_snapshot_id: stored.authored_snapshot_id,
            stale,
            status: if stale { "stale" } else { "current" }.into(),
            summary: if stale {
                "Evidence is stale for the selected revision.".into()
            } else {
                "Evidence is current for the selected revision.".into()
            },
            resolved_snapshot_id: stored.resolved_snapshot_id,
            input_hash: None,
            result_hash: None,
            solver_identity: None,
            metrics: serde_json::from_str::<PersistedEvidenceEnvelope>(&stored.manifest_json)
                .ok()
                .and_then(|value| value.analysis_manifest)
                .and_then(|value| value.metrics.as_ref().map(analysis_metrics_response)),
            diagnostics: Vec::new(),
        })
    }
}

fn project_workspace_database(project_dir: &str) -> Result<PathBuf, String> {
    let project_dir = PathBuf::from(project_dir);
    if !project_dir.is_absolute() {
        return Err("conversation project directory must be absolute".into());
    }
    Ok(project_dir.join(".fraia").join("workspace.sqlite"))
}
fn hydrate_project(
    path: &Path,
    project_id: &ProjectId,
) -> Result<ProjectConversationStore, String> {
    let sqlite = SqliteRevisionRepository::open(path).map_err(display)?;
    let root = sqlite.project_root(project_id).map_err(display)?;
    let root_model = sqlite
        .hydrate_snapshot(&root.root_snapshot.id)
        .map_err(display)?
        .model()
        .clone();
    let mut repository = InMemoryRevisionRepository::create(
        root.project_id.clone(),
        root.root_conversation.id.clone(),
        root.root_conversation.purpose.clone(),
        root.root_revision.id.clone(),
        root_model,
    )
    .map_err(display)?;

    let conversations = sqlite.project_conversations(project_id).map_err(display)?;
    let mut messages = BTreeMap::new();
    let root_state = persisted_conversation_envelope(&root.root_conversation.origin_json);
    messages.insert(root.root_conversation.id.clone(), root_state.state.messages);
    let mut pending_conversations = conversations
        .into_iter()
        .filter(|conversation| conversation.id != root.root_conversation.id)
        .map(|conversation| {
            let envelope = persisted_conversation_envelope(&conversation.origin_json);
            let origin = envelope.origin.ok_or_else(|| {
                format!(
                    "conversation {} has no typed durable origin",
                    conversation.id
                )
            })?;
            messages.insert(conversation.id.clone(), envelope.state.messages);
            Ok((conversation, origin))
        })
        .collect::<Result<Vec<_>, String>>()?;

    let mut pending = sqlite
        .project_revisions(project_id)
        .map_err(display)?
        .into_iter()
        .filter(|revision| revision.parent_revision_id.is_some())
        .collect::<Vec<_>>();
    while !pending.is_empty() || !pending_conversations.is_empty() {
        let mut restored = false;
        let mut conversation_index = 0;
        while conversation_index < pending_conversations.len() {
            let (_, origin) = &pending_conversations[conversation_index];
            let start_revision_id = match origin {
                ConversationOrigin::ProjectRoot => repository.root_revision_id().clone(),
                ConversationOrigin::StartedFromRevision { revision_id }
                | ConversationOrigin::ForkedFromRevision { revision_id }
                | ConversationOrigin::ResumedFromRevision { revision_id } => revision_id.clone(),
            };
            if repository.revision(&start_revision_id).is_err() {
                conversation_index += 1;
                continue;
            }
            let (conversation, origin) = pending_conversations.remove(conversation_index);
            repository
                .restore_conversation(conversation.id, conversation.purpose, origin)
                .map_err(display)?;
            restored = true;
        }
        let mut index = 0;
        while index < pending.len() {
            let stored = &pending[index];
            let Some(parent_revision_id) = stored.parent_revision_id.clone() else {
                index += 1;
                continue;
            };
            let head_matches = repository
                .head(&stored.conversation_id)
                .map(|head| head.head_revision_id == parent_revision_id)
                .unwrap_or(false);
            if !head_matches {
                index += 1;
                continue;
            }
            let stored = pending.remove(index);
            let snapshot = sqlite
                .hydrate_snapshot(&stored.snapshot_id)
                .map_err(display)?;
            let metadata = serde_json::from_str::<PersistedRevisionMetadata>(&stored.metadata_json)
                .map_err(display)?;
            let author_kind = match metadata.author.as_str() {
                "agent" => RevisionAuthorKind::Agent,
                "user" => RevisionAuthorKind::User,
                "system" => RevisionAuthorKind::System,
                _ => RevisionAuthorKind::Manual,
            };
            let operation = metadata.operation.unwrap_or_else(|| {
                if author_kind == RevisionAuthorKind::Agent {
                    RevisionOperation::AcceptedProposal {
                        proposal_id: ProposalId::new(format!("restored:{}", stored.id)),
                    }
                } else {
                    RevisionOperation::ManualEdit
                }
            });
            let provenance = metadata.agent_provenance.map(|value| AgentTurnProvenance {
                provider: value.provider,
                model: value.model,
                turn_id: value.turn_id,
            });
            repository
                .restore_revision(
                    stored.id,
                    snapshot,
                    parent_revision_id,
                    stored.conversation_id,
                    author_kind,
                    operation,
                    metadata.semantic_diff,
                    provenance,
                )
                .map_err(display)?;
            restored = true;
        }
        if !restored {
            return Err("durable revision graph could not be rehydrated in parent order".into());
        }
    }

    let revision_records = sqlite.project_revisions(project_id).map_err(display)?;
    let revision_by_snapshot = revision_records
        .iter()
        .map(|revision| (revision.snapshot_id.clone(), revision.id.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut analysis_runs = BTreeMap::new();
    for stored in sqlite.project_evidence(project_id).map_err(display)? {
        let envelope = serde_json::from_str::<PersistedEvidenceEnvelope>(&stored.manifest_json)
            .map_err(display)?;
        let Some(manifest) = envelope.analysis_manifest else {
            continue;
        };
        let revision_id = revision_by_snapshot
            .get(&stored.authored_snapshot_id)
            .cloned()
            .ok_or_else(|| {
                format!(
                    "durable evidence {} is not bound to a project revision",
                    stored.id
                )
            })?;
        let outcome = match manifest.status {
            AnalysisEvidenceStatus::Completed => SnapshotAnalysisOutcome::Completed {
                resolved_snapshot_id: stored
                    .resolved_snapshot_id
                    .clone()
                    .ok_or_else(|| "completed evidence has no resolved snapshot".to_owned())?,
                input_hash: manifest
                    .input_hash
                    .clone()
                    .ok_or_else(|| "completed evidence has no input identity".to_owned())?,
                result_hash: manifest
                    .result_hash
                    .clone()
                    .ok_or_else(|| "completed evidence has no result identity".to_owned())?,
                combo_count: manifest
                    .metrics
                    .as_ref()
                    .map(|metrics| metrics.combo_metrics.len())
                    .unwrap_or_default(),
                metrics: manifest
                    .metrics
                    .clone()
                    .ok_or_else(|| "completed evidence has no metrics".to_owned())?,
            },
            AnalysisEvidenceStatus::Failed => SnapshotAnalysisOutcome::Failed {
                diagnostics: manifest.diagnostics.clone(),
            },
            AnalysisEvidenceStatus::Unsupported => SnapshotAnalysisOutcome::Unsupported {
                diagnostics: manifest.diagnostics.clone(),
            },
        };
        let resolved_snapshot = stored
            .resolved_snapshot_id
            .as_ref()
            .map(|snapshot_id| -> Result<ResolvedSnapshotRecord, String> {
                let snapshot = sqlite.snapshot(snapshot_id).map_err(display)?;
                Ok(ResolvedSnapshotRecord {
                    id: snapshot.id.clone(),
                    format_version: snapshot.format_version,
                    canonical_bytes: snapshot.canonical_bytes,
                })
            })
            .transpose()?;
        let evidence = AnalysisEvidence::with_analysis_manifest(
            stored.id.clone(),
            stored.authored_snapshot_id,
            stored.resolved_snapshot_id,
            envelope.dependencies,
            manifest,
        )
        .map_err(display)?;
        analysis_runs.insert(
            stored.id,
            SnapshotAnalysisRun {
                revision_id,
                evidence,
                outcome,
                resolved_snapshot,
            },
        );
    }

    for stored in sqlite.project_proposals(project_id).map_err(display)? {
        let patch = serde_json::from_str::<StructuralPatch>(&stored.patch_json).map_err(display)?;
        let status = match stored.status.as_str() {
            "pending" => ProposalStatus::Pending,
            "rejected" => ProposalStatus::Rejected,
            "accepted" => ProposalStatus::Accepted {
                revision_id: stored
                    .accepted_revision_id
                    .clone()
                    .unwrap_or_else(|| stored.proposed_revision_id.clone()),
            },
            other => return Err(format!("unknown durable proposal status `{other}`")),
        };
        repository
            .restore_proposal(
                stored.id,
                stored.conversation_id,
                stored.parent_revision_id,
                stored.proposed_revision_id,
                patch,
                status,
                stored.agent_provenance,
            )
            .map_err(display)?;
    }

    Ok(ProjectConversationStore {
        repository,
        project_facts: root_state.state.project_facts,
        sqlite: Some(sqlite),
        sqlite_path: Some(path.to_path_buf()),
        messages,
        working_copies: BTreeMap::new(),
        analysis_runs,
    })
}

fn persisted_conversation_envelope(origin_json: &str) -> PersistedConversationEnvelope {
    serde_json::from_str(origin_json).unwrap_or_default()
}

fn restored_evidence_response(
    stored: StoredEvidence,
) -> Result<ConversationEvidenceResponse, String> {
    let envelope = serde_json::from_str::<PersistedEvidenceEnvelope>(&stored.manifest_json)
        .map_err(display)?;
    let manifest = envelope.analysis_manifest.ok_or_else(|| {
        format!(
            "durable analysis evidence `{}` has no typed analysis manifest",
            stored.id
        )
    })?;
    let (status, summary) = match manifest.status {
        AnalysisEvidenceStatus::Completed => (
            "success",
            "Analysis evidence restored from durable storage.",
        ),
        AnalysisEvidenceStatus::Failed => (
            "failed",
            "The persisted analysis attempt failed; no result was fabricated.",
        ),
        AnalysisEvidenceStatus::Unsupported => (
            "unsupported",
            "The persisted analysis request was unsupported by the available analysis path.",
        ),
    };
    Ok(ConversationEvidenceResponse {
        evidence_id: stored.id,
        authored_snapshot_id: stored.authored_snapshot_id,
        stale: false,
        status: status.into(),
        summary: summary.into(),
        resolved_snapshot_id: stored.resolved_snapshot_id,
        input_hash: manifest.input_hash,
        result_hash: manifest.result_hash,
        solver_identity: Some(manifest.solver_identity),
        metrics: manifest.metrics.as_ref().map(analysis_metrics_response),
        diagnostics: manifest.diagnostics,
    })
}

fn normalize_legacy_root_revision(
    repository: &InMemoryRevisionRepository,
    revision_id: RevisionId,
) -> RevisionId {
    if revision_id.as_str() == "root-revision" {
        repository.root_revision_id().clone()
    } else {
        revision_id
    }
}

fn persisted_origin_json(
    origin: &impl serde::Serialize,
    project_facts: &ConversationProjectFacts,
    messages: &[String],
) -> Result<String, String> {
    serde_json::to_string(&serde_json::json!({
        "origin": origin,
        "projectFacts": project_facts,
        "messages": messages,
    }))
    .map_err(display)
}

fn parse_role(value: &str) -> Result<MemberRole, String> {
    match value {
        "beam" => Ok(MemberRole::Beam),
        "column" => Ok(MemberRole::Column),
        "rafter" => Ok(MemberRole::Rafter),
        "brace" => Ok(MemberRole::Brace),
        "joist" => Ok(MemberRole::Joist),
        "purlin" => Ok(MemberRole::Purlin),
        _ => Err("unsupported member role".into()),
    }
}
fn transport_operation(
    operation: ConversationProposalOperation,
) -> Result<StructuralOperation, String> {
    match operation {
        ConversationProposalOperation::SetMemberRole { member_id, role } => {
            Ok(StructuralOperation::SetMemberRole {
                member_id,
                role: parse_role(&role)?,
            })
        }
        ConversationProposalOperation::MoveNode { node_id, x, y, z } => {
            Ok(StructuralOperation::MoveNode {
                node_id,
                position: Position {
                    x: Length::meters(x),
                    y: Length::meters(y),
                    z: Length::meters(z),
                },
            })
        }
        ConversationProposalOperation::AddNode { id, x, y, z } => Ok(StructuralOperation::AddNode(
            fraia_revision::patch::NodeInput {
                id,
                position: Position {
                    x: Length::meters(x),
                    y: Length::meters(y),
                    z: Length::meters(z),
                },
            },
        )),
        ConversationProposalOperation::AddMember {
            id,
            start_node,
            end_node,
            role,
            section_id,
            material_id,
        } => Ok(StructuralOperation::AddMember(StructuralMember {
            id,
            start_node,
            end_node,
            role: parse_role(&role)?.as_str().into(),
            semantic_tags: Vec::new(),
            section_id,
            material_id,
        })),
        ConversationProposalOperation::AddSupport {
            id,
            target_node,
            ux,
            uy,
            uz,
            rx,
            ry,
            rz,
        } => Ok(StructuralOperation::AddSupport(SupportAssignment {
            id,
            target_node,
            ux,
            uy,
            uz,
            rx,
            ry,
            rz,
        })),
        ConversationProposalOperation::SetSection {
            member_id,
            section_id,
        } => Ok(StructuralOperation::SetSection {
            member_id,
            section_id,
        }),
        ConversationProposalOperation::AddPlate {
            id,
            boundary_nodes,
            role,
            thickness_m,
            material_id,
            generated_from,
        } => Ok(StructuralOperation::AddPlate(StructuralPlate {
            id,
            boundary_nodes,
            role,
            semantic_tags: Vec::new(),
            thickness_m,
            material_id,
            generated_from,
        })),
        ConversationProposalOperation::AddLoad {
            id,
            target_kind,
            target_id,
            load_case_id,
            direction_x,
            direction_y,
            direction_z,
            magnitude,
            unit,
        } => {
            let target = match target_kind.as_str() {
                "node" => AssignmentTargetRef::Node(target_id),
                "member" => AssignmentTargetRef::Member(target_id),
                "plate" => AssignmentTargetRef::Plate(target_id),
                _ => return Err("load target kind must be node, member, or plate".into()),
            };
            let magnitude = match &target {
                AssignmentTargetRef::Node(_) => LoadMagnitude::Force {
                    value: magnitude,
                    unit: match unit.as_str() {
                        "N" | "newtons" => ForceUnit::Newtons,
                        "kN" | "kilonewtons" => ForceUnit::KiloNewtons,
                        _ => return Err("node loads require N or kN units".into()),
                    },
                },
                AssignmentTargetRef::Member(_) => LoadMagnitude::LineLoad {
                    value: magnitude,
                    unit: match unit.as_str() {
                        "N/m" | "newtons_per_meter" => LineLoadUnit::NewtonsPerMeter,
                        "kN/m" | "kilonewtons_per_meter" => LineLoadUnit::KiloNewtonsPerMeter,
                        _ => return Err("member loads require N/m or kN/m units".into()),
                    },
                },
                AssignmentTargetRef::Plate(_) => LoadMagnitude::Pressure {
                    value: magnitude,
                    unit: match unit.as_str() {
                        "Pa" | "pascals" => PressureUnit::Pascals,
                        "kPa" | "kilopascals" => PressureUnit::KiloPascals,
                        _ => return Err("plate loads require Pa or kPa units".into()),
                    },
                },
            };
            Ok(StructuralOperation::AddLoad(LoadInput {
                id,
                target,
                load_case_id,
                direction: LoadVector {
                    x: direction_x,
                    y: direction_y,
                    z: direction_z,
                },
                magnitude,
            }))
        }
        ConversationProposalOperation::AddRelease {
            id,
            member_id,
            end,
            ux,
            uy,
            uz,
            rx,
            ry,
            rz,
        } => Ok(StructuralOperation::AddRelease(release_assignment(
            id, member_id, end, ux, uy, uz, rx, ry, rz,
        )?)),
        ConversationProposalOperation::SetRelease {
            id,
            member_id,
            end,
            ux,
            uy,
            uz,
            rx,
            ry,
            rz,
        } => Ok(StructuralOperation::SetRelease(release_assignment(
            id, member_id, end, ux, uy, uz, rx, ry, rz,
        )?)),
    }
}

fn release_assignment(
    id: String,
    member_id: String,
    end: String,
    ux: bool,
    uy: bool,
    uz: bool,
    rx: bool,
    ry: bool,
    rz: bool,
) -> Result<ReleaseAssignment, String> {
    let end = match end.as_str() {
        "start" => MemberEnd::Start,
        "end" => MemberEnd::End,
        _ => return Err("release end must be start or end".into()),
    };
    Ok(ReleaseAssignment {
        id,
        target: MemberEndTarget { member_id, end },
        ux,
        uy,
        uz,
        rx,
        ry,
        rz,
    })
}
fn display(error: impl std::fmt::Display) -> String {
    error.to_string()
}

fn analysis_metrics_response(metrics: &AnalysisMetrics) -> ConversationAnalysisMetrics {
    ConversationAnalysisMetrics {
        combo_metrics: metrics
            .combo_metrics
            .iter()
            .map(|combo| ConversationAnalysisComboMetrics {
                combo_id: combo.combo_id.clone(),
                max_utilization: combo.max_utilization,
                max_ux_m: combo.max_ux_m,
                max_uy_m: combo.max_uy_m,
                max_reaction_n: combo.max_reaction_n,
            })
            .collect(),
        max_utilization: metrics.max_utilization,
        max_ux_m: metrics.max_ux_m,
        max_uy_m: metrics.max_uy_m,
        max_reaction_n: metrics.max_reaction_n,
    }
}

fn comparison_entry_response(
    entry: fraia_revision::analysis_service::AnalysisComparisonEntry,
) -> ConversationComparisonEntry {
    let metrics = entry.metrics;
    ConversationComparisonEntry {
        evidence_id: entry.evidence_id,
        authored_snapshot_id: entry.authored_snapshot_id,
        resolved_snapshot_id: entry.resolved_snapshot_id,
        input_identity: entry.input_identity,
        result_identity: entry.result_identity,
        metrics: ConversationAnalysisMetrics {
            combo_metrics: metrics
                .combo_metrics
                .into_iter()
                .map(|combo| ConversationAnalysisComboMetrics {
                    combo_id: combo.combo_id,
                    max_utilization: combo.max_utilization,
                    max_ux_m: combo.max_ux_m,
                    max_uy_m: combo.max_uy_m,
                    max_reaction_n: combo.max_reaction_n,
                })
                .collect(),
            max_utilization: metrics.max_utilization,
            max_ux_m: metrics.max_ux_m,
            max_uy_m: metrics.max_uy_m,
            max_reaction_n: metrics.max_reaction_n,
        },
    }
}

fn required_provenance(value: Option<String>, field: &str) -> Result<String, String> {
    let value =
        value.ok_or_else(|| format!("accepted proposal requires agent {field} provenance"))?;
    if value.trim().is_empty() {
        return Err(format!(
            "accepted proposal requires non-empty agent {field} provenance"
        ));
    }
    Ok(value)
}

fn revision_response(
    record: &fraia_revision::repository::RevisionRecord,
) -> ConversationRevisionResponse {
    ConversationRevisionResponse {
        revision_id: record.revision_id().clone(),
        snapshot_id: record.snapshot_id().clone(),
        parent_revision_id: record.parent_revision_id().cloned(),
        author: match record.author_kind() {
            RevisionAuthorKind::Agent => "agent",
            RevisionAuthorKind::Manual => "manual",
            RevisionAuthorKind::System => "system",
            RevisionAuthorKind::User => "user",
        }
        .into(),
        agent_provenance: record
            .agent_provenance()
            .map(|p| ConversationAgentProvenance {
                provider: p.provider.clone(),
                model: p.model.clone(),
                turn_id: p.turn_id.clone(),
            }),
    }
}

fn stored_snapshot(snapshot: &ModelSnapshot) -> StoredSnapshot {
    StoredSnapshot {
        id: snapshot.id().clone(),
        format_version: snapshot.canonical_format_version().as_str().to_owned(),
        canonical_bytes: snapshot.canonical_bytes().to_vec(),
    }
}

fn persist_root(
    sqlite: &mut SqliteRevisionRepository,
    project_id: &ProjectId,
    repository: &InMemoryRevisionRepository,
    conversation_id: &ConversationId,
    project_facts: &ConversationProjectFacts,
) -> Result<(), String> {
    let root_id = repository.root_revision_id();
    let revision = repository.revision(root_id).map_err(display)?;
    let conversation = repository.conversation(conversation_id).map_err(display)?;
    let snapshot = repository
        .snapshot(revision.snapshot_id())
        .map_err(display)?;
    sqlite
        .create_project(StoredProjectRoot {
            project_id: project_id.clone(),
            root_conversation: StoredConversation {
                id: conversation.id().clone(),
                project_id: conversation.project_id().clone(),
                purpose: conversation.purpose().to_owned(),
                origin_json: persisted_origin_json(conversation.origin(), project_facts, &[])?,
                head_revision_id: conversation.head_revision_id().clone(),
            },
            root_revision: stored_revision(revision, None)?,
            root_snapshot: stored_snapshot(snapshot),
        })
        .map_err(display)
}

fn persist_revision(
    sqlite: &mut SqliteRevisionRepository,
    repository: &InMemoryRevisionRepository,
    revision: &RevisionRecord,
    patch: Option<&StructuralPatch>,
) -> Result<(), String> {
    let snapshot = repository
        .snapshot(revision.snapshot_id())
        .map_err(display)?;
    let stored_revision = stored_revision(revision, patch)?;
    sqlite
        .append_revision_with_snapshot(
            &stored_revision,
            &stored_snapshot(snapshot),
            revision
                .parent_revision_id()
                .ok_or_else(|| "non-root revision has no parent".to_string())?,
        )
        .map_err(display)
}

fn stored_revision(
    revision: &RevisionRecord,
    patch: Option<&StructuralPatch>,
) -> Result<StoredRevision, String> {
    let provenance = revision.agent_provenance().map(|value| {
        serde_json::json!({
            "provider": value.provider,
            "model": value.model,
            "turnId": value.turn_id,
        })
    });
    let metadata_json = serde_json::to_string(&serde_json::json!({
        "author": match revision.author_kind() {
            RevisionAuthorKind::System => "system",
            RevisionAuthorKind::Agent => "agent",
            RevisionAuthorKind::Manual => "manual",
            RevisionAuthorKind::User => "user",
        },
        "operation": revision.operation(),
        "semanticDiff": revision.semantic_diff(),
        "agentProvenance": provenance,
        "patch": patch,
    }))
    .map_err(display)?;
    Ok(StoredRevision {
        id: revision.revision_id().clone(),
        snapshot_id: revision.snapshot_id().clone(),
        parent_revision_id: revision.parent_revision_id().cloned(),
        conversation_id: revision.conversation_id().clone(),
        metadata_json,
    })
}

fn stored_proposal(
    project_id: &ProjectId,
    id: &ProposalId,
    conversation_id: &ConversationId,
    parent_revision_id: &RevisionId,
    proposed_revision_id: &RevisionId,
    patch: &StructuralPatch,
    agent_provenance: Option<&AgentTurnProvenance>,
) -> Result<StoredProposal, String> {
    Ok(StoredProposal {
        id: id.clone(),
        project_id: project_id.clone(),
        conversation_id: conversation_id.clone(),
        parent_revision_id: parent_revision_id.clone(),
        proposed_revision_id: proposed_revision_id.clone(),
        patch_json: serde_json::to_string(patch).map_err(display)?,
        status: "pending".into(),
        accepted_revision_id: None,
        agent_provenance: agent_provenance.cloned(),
    })
}

fn persist_agent_revision(
    sqlite: &mut SqliteRevisionRepository,
    repository: &InMemoryRevisionRepository,
    revision: &RevisionRecord,
    patch: Option<&StructuralPatch>,
    proposal_id: &ProposalId,
    provenance: &ConversationAgentProvenance,
) -> Result<(), String> {
    let snapshot = repository
        .snapshot(revision.snapshot_id())
        .map_err(display)?;
    let stored_revision = stored_revision(revision, patch)?;
    let agent_provenance = AgentTurnProvenance {
        provider: provenance.provider.clone(),
        model: provenance.model.clone(),
        turn_id: provenance.turn_id.clone(),
    };
    sqlite
        .append_revision_with_snapshot_and_proposal(
            &stored_revision,
            &stored_snapshot(snapshot),
            revision
                .parent_revision_id()
                .ok_or_else(|| "non-root revision has no parent".to_string())?,
            proposal_id,
            Some(&agent_provenance),
        )
        .map_err(display)
}

fn stored_evidence(evidence: &AnalysisEvidence) -> Result<StoredEvidence, String> {
    Ok(StoredEvidence {
        id: evidence.id().clone(),
        authored_snapshot_id: evidence.authored_snapshot_id().clone(),
        resolved_snapshot_id: evidence.resolved_snapshot_id().cloned(),
        manifest_json: serde_json::to_string(&serde_json::json!({
            "dependencies": evidence.dependencies(),
            "analysisManifest": evidence.analysis_manifest(),
        }))
        .map_err(display)?,
        blob_ref: None,
    })
}

pub fn router(service: ConversationServiceHandle) -> Router {
    Router::new()
        .route("/conversations/create", post(create))
        .route("/conversations/converse", post(converse))
        .route("/conversations/facts", post(update_facts))
        .route("/conversations/propose", post(propose))
        .route("/conversations/accept", post(accept))
        .route("/conversations/reject", post(reject))
        .route("/conversations/fork", post(fork))
        .route("/conversations/resume", post(resume))
        .route("/conversations/analyse", post(analyse))
        .route("/conversations/compare", post(compare))
        .route("/conversations/working-copy/open", post(open_working_copy))
        .route(
            "/conversations/working-copy/apply",
            post(apply_working_copy_operation),
        )
        .route(
            "/conversations/working-copy/commit",
            post(commit_working_copy),
        )
        .layer(Extension(service))
}
async fn create(
    Extension(service): Extension<ConversationServiceHandle>,
    Json(request): Json<ConversationCreateRequest>,
) -> impl IntoResponse {
    response(service.lock().unwrap().create(request))
}
async fn converse(
    Extension(service): Extension<ConversationServiceHandle>,
    Json(request): Json<ConversationMessageRequest>,
) -> impl IntoResponse {
    response(service.lock().unwrap().converse(request))
}
async fn update_facts(
    Extension(service): Extension<ConversationServiceHandle>,
    Json(request): Json<ConversationFactsUpdateRequest>,
) -> impl IntoResponse {
    response(service.lock().unwrap().update_facts(request))
}
async fn propose(
    Extension(service): Extension<ConversationServiceHandle>,
    Json(request): Json<ConversationProposalRequest>,
) -> impl IntoResponse {
    response(service.lock().unwrap().propose(request))
}
async fn accept(
    Extension(service): Extension<ConversationServiceHandle>,
    Json(request): Json<ConversationProposalActionRequest>,
) -> impl IntoResponse {
    response(service.lock().unwrap().accept(request))
}
async fn reject(
    Extension(service): Extension<ConversationServiceHandle>,
    Json(request): Json<ConversationProposalActionRequest>,
) -> impl IntoResponse {
    response(service.lock().unwrap().reject(request))
}
async fn fork(
    Extension(service): Extension<ConversationServiceHandle>,
    Json(request): Json<ConversationForkRequest>,
) -> impl IntoResponse {
    response(service.lock().unwrap().fork(request, false))
}
async fn resume(
    Extension(service): Extension<ConversationServiceHandle>,
    Json(request): Json<ConversationForkRequest>,
) -> impl IntoResponse {
    response(service.lock().unwrap().fork(request, true))
}
async fn analyse(
    Extension(service): Extension<ConversationServiceHandle>,
    Json(request): Json<ConversationAnalysisRequest>,
) -> impl IntoResponse {
    response(service.lock().unwrap().analyse(request))
}
async fn compare(
    Extension(service): Extension<ConversationServiceHandle>,
    Json(request): Json<ConversationComparisonRequest>,
) -> impl IntoResponse {
    response(service.lock().unwrap().compare(request))
}
async fn open_working_copy(
    Extension(service): Extension<ConversationServiceHandle>,
    Json(request): Json<ConversationWorkingCopyOpenRequest>,
) -> impl IntoResponse {
    response(service.lock().unwrap().open_working_copy(request))
}
async fn apply_working_copy_operation(
    Extension(service): Extension<ConversationServiceHandle>,
    Json(request): Json<ConversationWorkingCopyOperationRequest>,
) -> impl IntoResponse {
    response(
        service
            .lock()
            .unwrap()
            .apply_working_copy_operation(request),
    )
}
async fn commit_working_copy(
    Extension(service): Extension<ConversationServiceHandle>,
    Json(request): Json<ConversationWorkingCopyCommitRequest>,
) -> impl IntoResponse {
    response(service.lock().unwrap().commit_working_copy(request))
}
fn response<T: serde::Serialize>(result: Result<T, String>) -> axum::response::Response {
    match result {
        Ok(value) => (StatusCode::OK, Json(value)).into_response(),
        Err(error) => (StatusCode::BAD_REQUEST, Json(ErrorResponse { error })).into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    fn create_request() -> ConversationCreateRequest {
        ConversationCreateRequest {
            project_id: ProjectId::from("p"),
            project_dir: tempdir().expect("project dir").keep().display().to_string(),
            conversation_id: ConversationId::from("overall"),
            purpose: "Overall framing".into(),
            project_facts: ConversationProjectFacts {
                name: Some("New warehouse".into()),
                building_type: Some("industrial".into()),
                approximate_length_m: Some(20.0),
                approximate_width_m: Some(12.0),
                approximate_height_m: Some(6.0),
                objective: Some("clear span".into()),
                constraints: vec!["keep north elevation clear".into()],
                loads_and_assumptions: vec!["self weight included".into()],
                unknowns: vec!["final imposed load".into()],
            },
        }
    }
    fn proposal() -> ConversationProposalRequest {
        ConversationProposalRequest {
            project_id: ProjectId::from("p"),
            conversation_id: ConversationId::from("overall"),
            proposal_id: "pr1".into(),
            proposed_revision_id: RevisionId::from("r1"),
            parent_revision_id: fraia_revision::root_fixture().root_revision_id,
            provider: "fake".into(),
            model: "test".into(),
            turn_id: "t1".into(),
            operations: vec![ConversationProposalOperation::SetMemberRole {
                member_id: "rafter".into(),
                role: "beam".into(),
            }],
            operation: None,
        }
    }
    fn first_geometry_proposal(parent_revision_id: RevisionId) -> ConversationProposalRequest {
        ConversationProposalRequest {
            project_id: ProjectId::from("p"),
            conversation_id: ConversationId::from("overall"),
            proposal_id: "first-geometry".into(),
            proposed_revision_id: RevisionId::from("r1"),
            parent_revision_id,
            provider: "fake".into(),
            model: "test".into(),
            turn_id: "first".into(),
            operation: None,
            operations: vec![
                ConversationProposalOperation::AddNode {
                    id: "left-base".into(),
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                ConversationProposalOperation::AddNode {
                    id: "right-base".into(),
                    x: 20.0,
                    y: 0.0,
                    z: 0.0,
                },
                ConversationProposalOperation::AddMember {
                    id: "tie".into(),
                    start_node: "left-base".into(),
                    end_node: "right-base".into(),
                    role: "beam".into(),
                    section_id: "250UB".into(),
                    material_id: "steel".into(),
                },
                ConversationProposalOperation::AddSupport {
                    id: "left-support".into(),
                    target_node: "left-base".into(),
                    ux: true,
                    uy: true,
                    uz: true,
                    rx: false,
                    ry: false,
                    rz: false,
                },
                ConversationProposalOperation::AddSupport {
                    id: "right-support".into(),
                    target_node: "right-base".into(),
                    ux: false,
                    uy: true,
                    uz: true,
                    rx: false,
                    ry: false,
                    rz: false,
                },
            ],
        }
    }
    #[test]
    fn empty_project_accepts_first_typed_geometry_as_one_revision() {
        let mut service = ConversationService::default();
        let root = service.create(create_request()).unwrap();
        assert_eq!(root.semantic_summary.counts.nodes, 0);
        assert_eq!(root.semantic_summary.counts.members, 0);
        let mut proposal = first_geometry_proposal(root.head_revision_id.clone());
        proposal.proposal_id = "initial-geometry".into();
        proposal.proposed_revision_id = RevisionId::from("initial-r1");
        service.propose(proposal).unwrap();
        let accepted = service
            .accept(ConversationProposalActionRequest {
                project_id: ProjectId::from("p"),
                proposal_id: "initial-geometry".into(),
                provider: Some("fake".into()),
                model: Some("test".into()),
                turn_id: Some("first".into()),
            })
            .unwrap();
        assert_ne!(accepted.snapshot_id, root.head_snapshot_id);
        let state = service
            .state(&ProjectId::from("p"), &ConversationId::from("overall"))
            .unwrap();
        assert_eq!(state.semantic_summary.counts.nodes, 2);
        assert_eq!(state.semantic_summary.counts.members, 1);
        assert_eq!(state.semantic_summary.counts.supports, 2);
    }

    #[test]
    fn empty_create_defaults_to_overall_design_conversation() {
        let mut service = ConversationService::default();
        let mut request = create_request();
        request.purpose.clear();
        let state = service.create(request).unwrap();
        assert_eq!(state.purpose, "Overall design");
        assert_eq!(state.semantic_summary.counts.nodes, 0);
        assert_eq!(state.semantic_summary.counts.members, 0);
    }

    #[test]
    fn rich_precision_operations_keep_units_and_targets_typed() {
        let fixture = fraia_revision::root_fixture();
        let operations = vec![
            transport_operation(ConversationProposalOperation::SetSection {
                member_id: "rafter".into(),
                section_id: "250UB".into(),
            })
            .unwrap(),
            transport_operation(ConversationProposalOperation::AddPlate {
                id: "slab".into(),
                boundary_nodes: vec!["left-base".into(), "left-eave".into(), "right-eave".into()],
                role: "slab".into(),
                thickness_m: 0.15,
                material_id: "concrete".into(),
                generated_from: "precision-editor".into(),
            })
            .unwrap(),
            transport_operation(ConversationProposalOperation::AddLoad {
                id: "roof-dead".into(),
                target_kind: "member".into(),
                target_id: "rafter".into(),
                load_case_id: "dead".into(),
                direction_x: 0.0,
                direction_y: -1.0,
                direction_z: 0.0,
                magnitude: 2.5,
                unit: "kN/m".into(),
            })
            .unwrap(),
            transport_operation(ConversationProposalOperation::AddRelease {
                id: "rafter-start-release".into(),
                member_id: "rafter".into(),
                end: "start".into(),
                ux: false,
                uy: false,
                uz: false,
                rx: true,
                ry: true,
                rz: true,
            })
            .unwrap(),
        ];
        let applied =
            fraia_revision::patch::apply_patch(&fixture.model, &StructuralPatch { operations })
                .unwrap();
        assert_eq!(applied.model.members[1].section_id, "250UB");
        assert_eq!(applied.model.plates.len(), 1);
        assert_eq!(applied.model.loads[0].magnitude, 2_500.0);
        assert_eq!(applied.model.releases.len(), 1);
    }

    #[test]
    fn transport_flow_preserves_rejections_and_exposes_staleness() {
        let mut service = ConversationService::default();
        let root = service.create_fixture_demo(create_request()).unwrap();
        service
            .converse(ConversationMessageRequest {
                project_id: ProjectId::from("p"),
                conversation_id: ConversationId::from("overall"),
                message: "explore".into(),
            })
            .unwrap();
        service.propose(proposal()).unwrap();
        service
            .reject(ConversationProposalActionRequest {
                project_id: ProjectId::from("p"),
                proposal_id: "pr1".into(),
                provider: None,
                model: None,
                turn_id: None,
            })
            .unwrap();
        assert_eq!(
            service
                .state(&ProjectId::from("p"), &ConversationId::from("overall"))
                .unwrap()
                .head_revision_id,
            root.head_revision_id
        );
        let mut accepted = proposal();
        accepted.proposal_id = "pr2".into();
        accepted.proposed_revision_id = RevisionId::from("r2");
        service.propose(accepted).unwrap();
        let revision = service
            .accept(ConversationProposalActionRequest {
                project_id: ProjectId::from("p"),
                proposal_id: "pr2".into(),
                provider: Some("fake".into()),
                model: Some("test".into()),
                turn_id: Some("t2".into()),
            })
            .unwrap();
        assert!(revision.agent_provenance.is_some());
        let evidence = service
            .analyse(ConversationAnalysisRequest {
                project_id: ProjectId::from("p"),
                conversation_id: ConversationId::from("overall"),
                revision_id: RevisionId::from("r2"),
                evidence_id: EvidenceId::from("e1"),
            })
            .unwrap();
        assert!(!evidence.stale);
        service
            .fork(
                ConversationForkRequest {
                    project_id: ProjectId::from("p"),
                    conversation_id: ConversationId::from("fork"),
                    purpose: "alternative".into(),
                    from_revision_id: RevisionId::from("r2"),
                },
                false,
            )
            .unwrap();
        let mut fork = proposal();
        fork.conversation_id = ConversationId::from("fork");
        fork.parent_revision_id = RevisionId::from("r2");
        fork.proposal_id = "pr3".into();
        fork.proposed_revision_id = RevisionId::from("r3");
        fork.operations = vec![ConversationProposalOperation::SetMemberRole {
            member_id: "rafter".into(),
            role: "brace".into(),
        }];
        service.propose(fork).unwrap();
        service
            .accept(ConversationProposalActionRequest {
                project_id: ProjectId::from("p"),
                proposal_id: "pr3".into(),
                provider: Some("fake".into()),
                model: Some("test".into()),
                turn_id: Some("t3".into()),
            })
            .unwrap();
        assert!(
            service
                .evidence(
                    &ProjectId::from("p"),
                    &EvidenceId::from("e1"),
                    &RevisionId::from("r3")
                )
                .unwrap()
                .stale
        );
        service
            .fork(
                ConversationForkRequest {
                    project_id: ProjectId::from("p"),
                    conversation_id: ConversationId::from("resume"),
                    purpose: "retry".into(),
                    from_revision_id: RevisionId::from("r2"),
                },
                true,
            )
            .unwrap();
    }

    #[test]
    fn comparison_transport_returns_exact_execution_and_snapshot_identities() {
        let mut service = ConversationService::default();
        let root = service.create(create_request()).unwrap();
        let mut baseline = first_geometry_proposal(root.head_revision_id.clone());
        baseline.proposal_id = "baseline-proposal".into();
        baseline.proposed_revision_id = RevisionId::from("baseline-r1");
        service.propose(baseline).unwrap();
        let baseline_revision = service
            .accept(ConversationProposalActionRequest {
                project_id: ProjectId::from("p"),
                proposal_id: "baseline-proposal".into(),
                provider: Some("provider".into()),
                model: Some("model".into()),
                turn_id: Some("baseline-turn".into()),
            })
            .unwrap();
        service
            .analyse(ConversationAnalysisRequest {
                project_id: ProjectId::from("p"),
                conversation_id: ConversationId::from("overall"),
                revision_id: baseline_revision.revision_id,
                evidence_id: EvidenceId::from("baseline-evidence"),
            })
            .unwrap();
        service
            .fork(
                ConversationForkRequest {
                    project_id: ProjectId::from("p"),
                    conversation_id: ConversationId::from("candidate"),
                    purpose: "Candidate".into(),
                    from_revision_id: root.head_revision_id.clone(),
                },
                false,
            )
            .unwrap();
        let mut candidate = first_geometry_proposal(root.head_revision_id.clone());
        candidate.conversation_id = ConversationId::from("candidate");
        candidate.proposal_id = "candidate-proposal".into();
        candidate.proposed_revision_id = RevisionId::from("candidate-r1");
        candidate.operations = candidate
            .operations
            .into_iter()
            .map(|operation| match operation {
                ConversationProposalOperation::AddMember {
                    id,
                    start_node,
                    end_node,
                    role: _,
                    section_id,
                    material_id,
                } => ConversationProposalOperation::AddMember {
                    id,
                    start_node,
                    end_node,
                    role: "rafter".into(),
                    section_id,
                    material_id,
                },
                other => other,
            })
            .collect();
        service.propose(candidate).unwrap();
        let candidate_revision = service
            .accept(ConversationProposalActionRequest {
                project_id: ProjectId::from("p"),
                proposal_id: "candidate-proposal".into(),
                provider: Some("provider".into()),
                model: Some("model".into()),
                turn_id: Some("candidate-turn".into()),
            })
            .unwrap();
        service
            .analyse(ConversationAnalysisRequest {
                project_id: ProjectId::from("p"),
                conversation_id: ConversationId::from("candidate"),
                revision_id: candidate_revision.revision_id,
                evidence_id: EvidenceId::from("candidate-evidence"),
            })
            .unwrap();
        let comparison = service
            .compare(ConversationComparisonRequest {
                project_id: ProjectId::from("p"),
                conversation_id: ConversationId::from("overall"),
                baseline_evidence_id: EvidenceId::from("baseline-evidence"),
                candidate_evidence_id: EvidenceId::from("candidate-evidence"),
            })
            .unwrap();
        assert_eq!(comparison.solver_identity, "fraia.frame2d.internal.v1");
        assert_ne!(
            comparison.baseline.authored_snapshot_id,
            comparison.candidate.authored_snapshot_id
        );
        assert_ne!(
            comparison.baseline.result_identity,
            comparison.candidate.result_identity
        );
    }

    #[test]
    fn durable_branches_and_comparison_survive_service_restart() {
        let directory = tempdir().unwrap();
        let path = directory.path().join(".fraia/workspace.sqlite");
        let mut request = create_request();
        request.project_dir = directory.path().display().to_string();
        {
            let mut service = ConversationService::open_durable(&path).unwrap();
            let root = service.create(request.clone()).unwrap();
            let mut baseline = first_geometry_proposal(root.head_revision_id.clone());
            baseline.proposal_id = "durable-baseline-proposal".into();
            baseline.proposed_revision_id = RevisionId::from("durable-baseline-r1");
            service.propose(baseline).unwrap();
            let baseline_revision = service
                .accept(ConversationProposalActionRequest {
                    project_id: ProjectId::from("p"),
                    proposal_id: "durable-baseline-proposal".into(),
                    provider: Some("provider".into()),
                    model: Some("model".into()),
                    turn_id: Some("durable-baseline-turn".into()),
                })
                .unwrap();
            service
                .analyse(ConversationAnalysisRequest {
                    project_id: ProjectId::from("p"),
                    conversation_id: ConversationId::from("overall"),
                    revision_id: baseline_revision.revision_id,
                    evidence_id: EvidenceId::from("durable-baseline-evidence"),
                })
                .unwrap();
            service
                .fork(
                    ConversationForkRequest {
                        project_id: ProjectId::from("p"),
                        conversation_id: ConversationId::from("durable-candidate"),
                        purpose: "Candidate branch".into(),
                        from_revision_id: root.head_revision_id.clone(),
                    },
                    false,
                )
                .unwrap();
            let mut candidate = first_geometry_proposal(root.head_revision_id);
            candidate.conversation_id = ConversationId::from("durable-candidate");
            candidate.proposal_id = "durable-candidate-proposal".into();
            candidate.proposed_revision_id = RevisionId::from("durable-candidate-r1");
            candidate.operations = candidate
                .operations
                .into_iter()
                .map(|operation| match operation {
                    ConversationProposalOperation::AddMember {
                        id,
                        start_node,
                        end_node,
                        role: _,
                        section_id,
                        material_id,
                    } => ConversationProposalOperation::AddMember {
                        id,
                        start_node,
                        end_node,
                        role: "rafter".into(),
                        section_id,
                        material_id,
                    },
                    other => other,
                })
                .collect();
            service.propose(candidate).unwrap();
            let candidate_revision = service
                .accept(ConversationProposalActionRequest {
                    project_id: ProjectId::from("p"),
                    proposal_id: "durable-candidate-proposal".into(),
                    provider: Some("provider".into()),
                    model: Some("model".into()),
                    turn_id: Some("durable-candidate-turn".into()),
                })
                .unwrap();
            service
                .analyse(ConversationAnalysisRequest {
                    project_id: ProjectId::from("p"),
                    conversation_id: ConversationId::from("durable-candidate"),
                    revision_id: candidate_revision.revision_id,
                    evidence_id: EvidenceId::from("durable-candidate-evidence"),
                })
                .unwrap();
        }
        let mut service = ConversationService::open_durable(&path).unwrap();
        service.create(request).unwrap();
        let candidate_state = service
            .state(
                &ProjectId::from("p"),
                &ConversationId::from("durable-candidate"),
            )
            .unwrap();
        assert_eq!(
            candidate_state.head_revision_id,
            RevisionId::from("durable-candidate-r1")
        );
        assert_eq!(candidate_state.semantic_summary.counts.supports, 2);
        let comparison = service
            .compare(ConversationComparisonRequest {
                project_id: ProjectId::from("p"),
                conversation_id: ConversationId::from("overall"),
                baseline_evidence_id: EvidenceId::from("durable-baseline-evidence"),
                candidate_evidence_id: EvidenceId::from("durable-candidate-evidence"),
            })
            .unwrap();
        assert_ne!(
            comparison.baseline.result_identity,
            comparison.candidate.result_identity
        );
    }

    #[test]
    fn working_copy_transport_commits_one_manual_revision() {
        let mut service = ConversationService::default();
        let root = service.create_fixture_demo(create_request()).unwrap();
        let opened = service
            .open_working_copy(ConversationWorkingCopyOpenRequest {
                project_id: ProjectId::from("p"),
                conversation_id: ConversationId::from("overall"),
                revision_id: root.head_revision_id.clone(),
            })
            .unwrap();
        service
            .apply_working_copy_operation(ConversationWorkingCopyOperationRequest {
                project_id: ProjectId::from("p"),
                working_copy_id: opened.working_copy_id.clone(),
                operation: ConversationProposalOperation::SetMemberRole {
                    member_id: "rafter".into(),
                    role: "beam".into(),
                },
            })
            .unwrap();
        let revision = service
            .commit_working_copy(ConversationWorkingCopyCommitRequest {
                project_id: ProjectId::from("p"),
                conversation_id: ConversationId::from("overall"),
                working_copy_id: opened.working_copy_id.clone(),
                revision_id: RevisionId::from("manual-r1"),
            })
            .unwrap();
        assert_eq!(revision.author, "manual");
        assert_eq!(revision.parent_revision_id, Some(root.head_revision_id));
        assert!(
            service
                .commit_working_copy(ConversationWorkingCopyCommitRequest {
                    project_id: ProjectId::from("p"),
                    conversation_id: ConversationId::from("overall"),
                    working_copy_id: opened.working_copy_id,
                    revision_id: RevisionId::from("manual-r2"),
                })
                .is_err()
        );
    }

    #[test]
    fn working_copy_transport_moves_a_node_in_metres() {
        let mut service = ConversationService::default();
        let root = service.create_fixture_demo(create_request()).unwrap();
        let opened = service
            .open_working_copy(ConversationWorkingCopyOpenRequest {
                project_id: ProjectId::from("p"),
                conversation_id: ConversationId::from("overall"),
                revision_id: root.head_revision_id.clone(),
            })
            .unwrap();
        service
            .apply_working_copy_operation(ConversationWorkingCopyOperationRequest {
                project_id: ProjectId::from("p"),
                working_copy_id: opened.working_copy_id.clone(),
                operation: ConversationProposalOperation::MoveNode {
                    node_id: "left-eave".into(),
                    x: 1.5,
                    y: 6.0,
                    z: 0.0,
                },
            })
            .unwrap();
        let revision = service
            .commit_working_copy(ConversationWorkingCopyCommitRequest {
                project_id: ProjectId::from("p"),
                conversation_id: ConversationId::from("overall"),
                working_copy_id: opened.working_copy_id,
                revision_id: RevisionId::from("manual-node-move"),
            })
            .unwrap();
        let store = service.projects.get(&ProjectId::from("p")).unwrap();
        let source = store
            .repository
            .snapshot(&root.head_snapshot_id)
            .unwrap()
            .model();
        let moved = store
            .repository
            .snapshot(&revision.snapshot_id)
            .unwrap()
            .model();
        assert_eq!(
            source
                .nodes
                .iter()
                .find(|node| node.id == "left-eave")
                .unwrap()
                .x,
            0.0
        );
        assert_eq!(
            moved
                .nodes
                .iter()
                .find(|node| node.id == "left-eave")
                .unwrap()
                .x,
            1.5
        );
        assert_eq!(revision.parent_revision_id, Some(root.head_revision_id));
    }

    #[test]
    fn durable_transport_survives_service_restart_with_provenance_and_stale_evidence() {
        let directory = tempdir().unwrap();
        let path = directory.path().join(".fraia/workspace.sqlite");
        let mut request = create_request();
        request.project_dir = directory.path().display().to_string();
        let accepted_snapshot;
        {
            let mut service = ConversationService::open_durable(&path).unwrap();
            let root = service.create(request.clone()).unwrap();
            service
                .propose(first_geometry_proposal(root.head_revision_id))
                .unwrap();
            let accepted = service
                .accept(ConversationProposalActionRequest {
                    project_id: ProjectId::from("p"),
                    proposal_id: "first-geometry".into(),
                    provider: Some("provider".into()),
                    model: Some("model".into()),
                    turn_id: Some("turn".into()),
                })
                .unwrap();
            accepted_snapshot = accepted.snapshot_id.clone();
            assert_eq!(accepted.agent_provenance.unwrap().turn_id, "turn");
            assert!(
                SqliteRevisionRepository::open(&path)
                    .unwrap()
                    .snapshot(&accepted_snapshot)
                    .is_ok(),
                "accepted snapshot must be durable before analysis"
            );
            service
                .analyse(ConversationAnalysisRequest {
                    project_id: ProjectId::from("p"),
                    conversation_id: ConversationId::from("overall"),
                    revision_id: accepted.revision_id,
                    evidence_id: EvidenceId::from("e1"),
                })
                .unwrap();
        }

        let mut service = ConversationService::open_durable(&path).unwrap();
        service.create(request).unwrap();
        let restored = service
            .state(&ProjectId::from("p"), &ConversationId::from("overall"))
            .unwrap();
        assert_eq!(restored.head_revision_id, RevisionId::from("r1"));
        assert_eq!(restored.head_snapshot_id, accepted_snapshot);
        assert_eq!(restored.purpose, "Overall framing");
        assert_eq!(restored.semantic_summary.counts.nodes, 2);
        assert_eq!(restored.semantic_summary.counts.members, 1);
        assert_eq!(restored.semantic_summary.counts.supports, 2);
        assert_eq!(restored.project_facts.approximate_length_m, Some(20.0));
        assert_eq!(
            restored.project_facts.loads_and_assumptions,
            vec!["self weight included"]
        );
        assert_eq!(restored.project_facts.unknowns, vec!["final imposed load"]);
        let persisted = SqliteRevisionRepository::open(&path)
            .unwrap()
            .revision(&RevisionId::from("r1"))
            .unwrap();
        assert!(persisted.metadata_json.contains("provider"));
        assert!(persisted.metadata_json.contains("turn"));
        assert!(
            !service
                .evidence(
                    &ProjectId::from("p"),
                    &EvidenceId::from("e1"),
                    &RevisionId::from("r1")
                )
                .unwrap()
                .stale
        );
        let replayed = service
            .analyse(ConversationAnalysisRequest {
                project_id: ProjectId::from("p"),
                conversation_id: ConversationId::from("overall"),
                revision_id: RevisionId::from("r1"),
                evidence_id: EvidenceId::from("e1"),
            })
            .unwrap();
        assert_eq!(replayed.status, "success");
        assert_eq!(
            replayed.summary,
            "Analysis evidence restored from durable storage."
        );
        assert!(
            service
                .evidence(
                    &ProjectId::from("p"),
                    &EvidenceId::from("e1"),
                    &RevisionId::from("p:root")
                )
                .unwrap()
                .stale
        );
        service
            .converse(ConversationMessageRequest {
                project_id: ProjectId::from("p"),
                conversation_id: ConversationId::from("overall"),
                message: "continue after restart".into(),
            })
            .unwrap();
        let restored_provenance = service
            .projects
            .get(&ProjectId::from("p"))
            .unwrap()
            .repository
            .revision(&RevisionId::from("r1"))
            .unwrap()
            .agent_provenance()
            .unwrap()
            .clone();
        assert_eq!(restored_provenance.turn_id, "turn");
        assert_eq!(
            service
                .state(&ProjectId::from("p"), &ConversationId::from("overall"))
                .unwrap()
                .messages,
            vec!["continue after restart"]
        );
        service
            .propose(ConversationProposalRequest {
                project_id: ProjectId::from("p"),
                conversation_id: ConversationId::from("overall"),
                proposal_id: "restart-child".into(),
                proposed_revision_id: RevisionId::from("restart-r2"),
                parent_revision_id: RevisionId::from("r1"),
                provider: "provider".into(),
                model: "model".into(),
                turn_id: "restart-turn".into(),
                operations: vec![ConversationProposalOperation::AddNode {
                    id: "restart-node".into(),
                    x: 10.0,
                    y: 0.0,
                    z: 0.0,
                }],
                operation: None,
            })
            .unwrap();
        service
            .accept(ConversationProposalActionRequest {
                project_id: ProjectId::from("p"),
                proposal_id: "restart-child".into(),
                provider: Some("provider".into()),
                model: Some("model".into()),
                turn_id: Some("restart-turn".into()),
            })
            .unwrap();
        assert_eq!(
            service
                .state(&ProjectId::from("p"), &ConversationId::from("overall"))
                .unwrap()
                .head_revision_id,
            RevisionId::from("restart-r2")
        );
        let reopened = service
            .create(create_request())
            .expect("create is idempotent after durable restart");
        assert_eq!(reopened.head_revision_id, RevisionId::from("restart-r2"));
        assert_eq!(reopened.messages, vec!["continue after restart"]);
    }
}
