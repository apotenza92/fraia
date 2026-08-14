//! Isolated HTTP/service adapter for the conversation-first spike.
//! It deliberately owns no legacy project state and performs no LLM calls.

use axum::{Extension, Json, Router, http::StatusCode, response::IntoResponse, routing::post};
use fraia_app_api::*;
use fraia_core::{
    AssignmentTargetRef, LoadVector, MemberEnd, MemberEndTarget, ReleaseAssignment,
    StructuralMember, StructuralModel, StructuralPlate, SupportAssignment, design_package_paths,
    load_project_package, understand_structural_model,
};
use fraia_revision::{
    ConversationId, EvidenceId, ProjectId, RevisionId,
    agent_contract::AgentTurnProvenance,
    analysis_service::{
        AnalysisSettings, ResolvedSnapshotRecord, SnapshotAnalysisOutcome, SnapshotAnalysisRun,
        compare_completed_runs,
    },
    conversation::ConversationOrigin,
    diff::SemanticDiff,
    evidence::{
        AnalysisEvidence, AnalysisEvidenceManifest, AnalysisEvidenceStatus, AnalysisMetrics,
        EvidenceDependency,
    },
    operations::{
        AnalysisOperationControl, DesignRunOperationContext, OPERATION_CONTRACT_VERSION, Operation,
        OperationErrorCode, OperationOutcome, OperationRequest, OperationResponse, OperationResult,
        execute_sqlite_operation, execute_sqlite_operation_with_design_runs,
        execute_sqlite_operation_with_design_runs_controlled,
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
        SqliteRevisionRepository, StoredConversation, StoredProjectRoot, StoredRevision,
        StoredSnapshot,
    },
    working_copy::WorkingCopy,
};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU8, Ordering},
    },
    time::{Duration, Instant},
};

const TEST_ANALYSIS_DELAY_ENV: &str = "FRAIA_TEST_ANALYSIS_DELAY_MS";
const TEST_ANALYSIS_FAILURE_ENV: &str = "FRAIA_TEST_ANALYSIS_FAILURE";
const MAX_TEST_ANALYSIS_DELAY_MILLIS: u64 = 30_000;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct AnalysisAttemptTestControl {
    delay_millis: u64,
    force_failure: bool,
}

fn resolve_analysis_attempt_test_control(
    debug_build: bool,
    mut read_env: impl FnMut(&str) -> Option<String>,
) -> AnalysisAttemptTestControl {
    if !debug_build {
        return AnalysisAttemptTestControl::default();
    }
    let delay_millis = read_env(TEST_ANALYSIS_DELAY_ENV)
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or_default()
        .min(MAX_TEST_ANALYSIS_DELAY_MILLIS);
    let force_failure = read_env(TEST_ANALYSIS_FAILURE_ENV).is_some_and(|value| value == "1");
    AnalysisAttemptTestControl {
        delay_millis,
        force_failure,
    }
}

fn analysis_attempt_test_control() -> AnalysisAttemptTestControl {
    resolve_analysis_attempt_test_control(cfg!(debug_assertions), |name| std::env::var(name).ok())
}

#[derive(Debug)]
struct AnalysisAttemptEntry {
    lifecycle: Arc<AtomicU8>,
    started: Instant,
    response: AnalysisAttemptResponse,
    sequence: u64,
}

#[derive(Debug)]
struct AnalysisAttemptRegistry {
    attempts: Mutex<BTreeMap<String, AnalysisAttemptEntry>>,
    test_control: AnalysisAttemptTestControl,
}

impl Default for AnalysisAttemptRegistry {
    fn default() -> Self {
        Self {
            attempts: Mutex::default(),
            test_control: analysis_attempt_test_control(),
        }
    }
}

fn persist_analysis_attempt(
    directory: &Path,
    sequence: u64,
    response: &AnalysisAttemptResponse,
) -> Result<(), String> {
    std::fs::create_dir_all(directory).map_err(display)?;
    let path = directory.join(format!("{sequence:020}.json"));
    let bytes = serde_json::to_vec_pretty(response).map_err(display)?;
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)
        .map_err(display)?;
    use std::io::Write;
    file.write_all(&bytes).map_err(display)?;
    file.sync_all().map_err(display)?;
    Ok(())
}

fn load_latest_analysis_attempt(
    directory: &Path,
) -> Result<(u64, AnalysisAttemptResponse), String> {
    let mut files = std::fs::read_dir(directory)
        .map_err(display)?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
        .collect::<Vec<_>>();
    files.sort_by_key(|entry| entry.file_name());
    let file = files
        .last()
        .ok_or_else(|| "analysis attempt has no durable state".to_string())?;
    let sequence = file
        .path()
        .file_stem()
        .and_then(|value| value.to_str())
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| "analysis attempt state sequence is invalid".to_string())?;
    let response =
        serde_json::from_slice(&std::fs::read(file.path()).map_err(display)?).map_err(display)?;
    Ok((sequence, response))
}

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
    #[serde(default)]
    agent_responses: Vec<ConversationAgentRespondResponse>,
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
struct PersistedRevisionMetadata {
    #[serde(default)]
    author: String,
    #[serde(default)]
    semantic_diff: SemanticDiff,
    #[serde(default)]
    operation: Option<serde_json::Value>,
    #[serde(default)]
    proposal_id: Option<ProposalId>,
    #[serde(default)]
    agent_provenance: Option<AgentTurnProvenance>,
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
    working_copy_operations: BTreeMap<String, Vec<StructuralOperation>>,
    analysis_runs: BTreeMap<EvidenceId, SnapshotAnalysisRun>,
    agent_responses: BTreeMap<ConversationId, Vec<ConversationAgentRespondResponse>>,
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
    pub(crate) fn proposal_model_context(
        &self,
        project: &ProjectId,
        conversation: &ConversationId,
    ) -> Result<serde_json::Value, String> {
        let store = self.store(project)?;
        let head = store.repository.head(conversation).map_err(display)?;
        let revision = store
            .repository
            .revision(&head.head_revision_id)
            .map_err(display)?;
        let model = store
            .repository
            .snapshot(revision.snapshot_id())
            .map_err(display)?
            .model();
        Ok(serde_json::json!({
            "currentNodeIds": model.nodes.iter().map(|item| item.id.clone()).collect::<Vec<_>>(),
            "currentMemberIds": model.members.iter().map(|item| item.id.clone()).collect::<Vec<_>>(),
            "allowedSectionIds": fraia_core::section_catalog().iter().map(|item| item.id.clone()).collect::<Vec<_>>(),
            "allowedMaterialIds": ["steel"],
        }))
    }

    pub(crate) fn validate_proposal_operations(
        &self,
        project: &ProjectId,
        conversation: &ConversationId,
        operations: &[ConversationProposalOperation],
    ) -> Result<(), String> {
        let store = self.store(project)?;
        let head = store.repository.head(conversation).map_err(display)?;
        let revision = store
            .repository
            .revision(&head.head_revision_id)
            .map_err(display)?;
        let model = store
            .repository
            .snapshot(revision.snapshot_id())
            .map_err(display)?
            .model();
        let operations = operations
            .iter()
            .cloned()
            .map(transport_operation)
            .collect::<Result<Vec<_>, _>>()?;
        fraia_revision::patch::apply_patch(model, &StructuralPatch { operations })
            .map(|_| ())
            .map_err(display)
    }

    fn analysis_attempt_path(
        &self,
        project_id: &ProjectId,
        attempt_id: &str,
    ) -> Result<PathBuf, String> {
        if attempt_id.len() != 48 || !attempt_id.chars().all(|value| value.is_ascii_hexdigit()) {
            return Err("analysis attempt id is invalid".into());
        }
        let workspace = self
            .store(project_id)?
            .sqlite_path
            .as_ref()
            .ok_or_else(|| "conversation project has no durable workspace".to_string())?;
        Ok(workspace
            .parent()
            .ok_or_else(|| "design workspace has no parent directory".to_string())?
            .join("analysis-attempts")
            .join(attempt_id))
    }
    pub(crate) fn workspace_path(&self, project_id: &ProjectId) -> Result<&Path, String> {
        self.store(project_id)?
            .sqlite_path
            .as_deref()
            .ok_or_else(|| "conversation project has no durable workspace".to_string())
    }
    pub fn unload(&mut self, project_id: &str) -> bool {
        self.projects.remove(&ProjectId::new(project_id)).is_some()
    }

    pub fn unload_workspace_path(&mut self, workspace: &Path) -> usize {
        let before = self.projects.len();
        self.projects
            .retain(|_, store| store.sqlite_path.as_deref() != Some(workspace));
        before - self.projects.len()
    }

    /// Executes the shared versioned operation contract against the workspace
    /// database resolved from an already-open design. The request cannot
    /// select an arbitrary filesystem path.
    pub fn execute_operation(
        &mut self,
        project_id: &ProjectId,
        request: OperationRequest,
    ) -> Result<OperationResponse, String> {
        self.execute_operation_maybe_controlled(project_id, request, None)
    }

    fn execute_operation_maybe_controlled(
        &mut self,
        project_id: &ProjectId,
        request: OperationRequest,
        mut control: Option<&mut AnalysisOperationControl<'_>>,
    ) -> Result<OperationResponse, String> {
        let path = self
            .store(project_id)?
            .sqlite_path
            .clone()
            .ok_or_else(|| "conversation project has no durable workspace".to_string())?;
        let refresh = matches!(
            &request.operation,
            Operation::ProposeStructuralPatch { .. }
                | Operation::AcceptStructuralPatch { .. }
                | Operation::RejectStructuralPatch { .. }
                | Operation::AnalyseSnapshot { .. }
        );
        let mut sqlite = SqliteRevisionRepository::open(&path).map_err(display)?;
        let design_dir = path
            .parent()
            .ok_or_else(|| "design workspace has no design directory".to_string())?;
        let project_dir = design_dir
            .parent()
            .and_then(Path::parent)
            .ok_or_else(|| "design workspace has no project directory".to_string())?;
        let response = if let Ok(package) = load_project_package(project_dir) {
            let design_id = fraia_core::DesignId::new(
                design_dir
                    .file_name()
                    .and_then(|value| value.to_str())
                    .ok_or_else(|| "design workspace directory has no valid id".to_string())?,
            );
            if !package
                .designs
                .iter()
                .any(|design| design.manifest.id == design_id)
            {
                return Err(
                    "active revision workspace does not belong to the project package".into(),
                );
            }
            let run_context = DesignRunOperationContext::new(
                project_dir,
                package.manifest.id,
                design_id,
                fraia_core::DesignRunActor {
                    actor_type: "app".into(),
                    actor_id: "fraia-appd".into(),
                },
                fraia_core::utils::iso_now(),
            );
            match control.as_deref_mut() {
                Some(control) => execute_sqlite_operation_with_design_runs_controlled(
                    &mut sqlite,
                    request,
                    &run_context,
                    control,
                ),
                None => {
                    execute_sqlite_operation_with_design_runs(&mut sqlite, request, &run_context)
                }
            }
        } else {
            // Legacy and isolated fixture workspaces have no package-owned run
            // store. They retain the operation contract without claiming a
            // canonical design-run identity.
            execute_sqlite_operation(&mut sqlite, request)
        };
        drop(sqlite);

        if refresh && matches!(response.outcome, OperationOutcome::Success { .. }) {
            let previous = self
                .projects
                .remove(project_id)
                .ok_or_else(|| "unknown conversation project".to_string())?;
            match hydrate_project(&path, project_id) {
                Ok(mut hydrated) => {
                    hydrated.working_copies = previous.working_copies;
                    hydrated.working_copy_operations = previous.working_copy_operations;
                    self.projects.insert(project_id.clone(), hydrated);
                }
                Err(error) => {
                    self.projects.insert(project_id.clone(), previous);
                    return Err(format!(
                        "operation persisted but the active design could not be refreshed: {error}"
                    ));
                }
            }
        }
        Ok(response)
    }

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
        let path = project_workspace_database(&request.project_dir, &request.project_id)?;
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
            working_copy_operations: BTreeMap::new(),
            analysis_runs: BTreeMap::new(),
            agent_responses: BTreeMap::new(),
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
            let agent_responses = store
                .agent_responses
                .get(&request.conversation_id)
                .cloned()
                .unwrap_or_default();
            sqlite
                .update_conversation_origin(
                    &request.conversation_id,
                    &persisted_origin_json(
                        conversation.origin(),
                        &store.project_facts,
                        &messages,
                        &agent_responses,
                    )?,
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
                        store
                            .agent_responses
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
        let fixture_dir = tempfile::tempdir().map_err(display)?.keep();
        let sqlite_path = fixture_dir.join("workspace.sqlite");
        let mut sqlite = SqliteRevisionRepository::open(&sqlite_path).map_err(display)?;
        persist_root(
            &mut sqlite,
            &request.project_id,
            &repository,
            &request.conversation_id,
            &request.project_facts,
        )?;
        self.projects.insert(
            request.project_id.clone(),
            ProjectConversationStore {
                repository,
                project_facts: std::mem::take(&mut request.project_facts),
                sqlite: Some(sqlite),
                sqlite_path: Some(sqlite_path),
                messages: BTreeMap::new(),
                working_copies: BTreeMap::new(),
                working_copy_operations: BTreeMap::new(),
                analysis_runs: BTreeMap::new(),
                agent_responses: BTreeMap::new(),
            },
        );
        self.state(&request.project_id, &request.conversation_id)
    }
    pub fn propose(&mut self, request: ConversationProposalRequest) -> Result<(), String> {
        let agent_provenance = AgentTurnProvenance {
            provider: required_provenance(Some(request.provider.clone()), "provider")?,
            model: required_provenance(Some(request.model.clone()), "model")?,
            turn_id: required_provenance(Some(request.turn_id.clone()), "turn")?,
            reasoning_effort: request.reasoning_effort.clone(),
            catalogue_refreshed_at: request.catalogue_refreshed_at.clone(),
            response_id: request.response_id.clone(),
            response_text: request.response_text.clone(),
            response_questions: request.response_questions.clone(),
            shelf_item_ids: request
                .source_context
                .as_ref()
                .map(|context| context.shelf_item_ids.clone())
                .unwrap_or_default(),
            drawing_interpretation_revision_ids: request
                .source_context
                .as_ref()
                .map(|context| context.drawing_interpretation_revision_ids.clone())
                .unwrap_or_default(),
            drawing_interpretation_inference_ids: request
                .source_context
                .as_ref()
                .map(|context| context.drawing_interpretation_inference_ids.clone())
                .unwrap_or_default(),
            assumptions: request
                .source_context
                .as_ref()
                .map(|context| context.assumptions.clone())
                .unwrap_or_default(),
            evidence_limits: request
                .source_context
                .as_ref()
                .map(|context| context.evidence_limits.clone())
                .unwrap_or_default(),
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
        let parent_revision_id = normalize_legacy_root_revision(
            &self.store(&request.project_id)?.repository,
            request.parent_revision_id,
        );
        if let Some(source_context) = &request.source_context {
            validate_proposal_source_context(
                self.store(&request.project_id)?,
                &parent_revision_id,
                source_context,
            )?;
        }
        let proposal_id = ProposalId::new(request.proposal_id.clone());
        let response = self.execute_operation(
            &request.project_id,
            OperationRequest {
                contract_version: OPERATION_CONTRACT_VERSION.into(),
                request_id: format!("conversation-propose:{}:{}", request.turn_id, proposal_id),
                operation: Operation::ProposeStructuralPatch {
                    proposal_id,
                    conversation_id: request.conversation_id,
                    expected_head_revision_id: parent_revision_id,
                    proposed_revision_id: request.proposed_revision_id,
                    patch: StructuralPatch { operations },
                    agent_provenance: Some(agent_provenance),
                },
            },
        )?;
        operation_result(response).map(|_| ())
    }
    pub fn accept(
        &mut self,
        request: ConversationProposalActionRequest,
    ) -> Result<ConversationRevisionResponse, String> {
        let proposal_id = ProposalId::new(request.proposal_id.clone());
        let proposal = self
            .store(&request.project_id)?
            .repository
            .proposal(&proposal_id)
            .map_err(display)?
            .clone();
        if let Some(authored) = proposal.agent_provenance() {
            validate_current_interpretation_bindings(self.store(&request.project_id)?, authored)?;
        }
        let provenance = ConversationAgentProvenance {
            provider: required_provenance(request.provider.clone(), "provider")?,
            model: required_provenance(request.model.clone(), "model")?,
            turn_id: required_provenance(request.turn_id.clone(), "turn")?,
        };
        let provenance = proposal
            .agent_provenance()
            .map(|authored| ConversationAgentProvenance {
                provider: authored.provider.clone(),
                model: authored.model.clone(),
                turn_id: authored.turn_id.clone(),
            })
            .unwrap_or(provenance);
        let response = self.execute_operation(
            &request.project_id,
            OperationRequest {
                contract_version: OPERATION_CONTRACT_VERSION.into(),
                request_id: format!("conversation-accept:{}:{}", provenance.turn_id, proposal_id),
                operation: Operation::AcceptStructuralPatch {
                    proposal_id,
                    conversation_id: proposal.conversation_id().clone(),
                    expected_head_revision_id: proposal.parent_revision_id().clone(),
                },
            },
        )?;
        match operation_result(response)? {
            OperationResult::StructuralPatchAccepted {
                revision_id,
                parent_revision_id,
                snapshot_id,
                ..
            } => Ok(ConversationRevisionResponse {
                revision_id,
                snapshot_id,
                parent_revision_id: Some(parent_revision_id),
                author: "agent".into(),
                agent_provenance: Some(provenance),
            }),
            result => Err(format!("unexpected accept operation result: {result:?}")),
        }
    }
    pub fn reject(&mut self, request: ConversationProposalActionRequest) -> Result<(), String> {
        let proposal_id = ProposalId::new(request.proposal_id);
        let proposal = self
            .store(&request.project_id)?
            .repository
            .proposal(&proposal_id)
            .map_err(display)?
            .clone();
        let response = self.execute_operation(
            &request.project_id,
            OperationRequest {
                contract_version: OPERATION_CONTRACT_VERSION.into(),
                request_id: format!("conversation-reject:{proposal_id}"),
                operation: Operation::RejectStructuralPatch {
                    proposal_id,
                    conversation_id: proposal.conversation_id().clone(),
                    expected_head_revision_id: proposal.parent_revision_id().clone(),
                },
            },
        )?;
        operation_result(response).map(|_| ())
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
        let store = self.store(&request.project_id)?;
        store
            .repository
            .conversation(&request.conversation_id)
            .map_err(display)?;
        let revision = store
            .repository
            .revision(&request.revision_id)
            .map_err(display)?;
        if let Some(provenance) = revision.agent_provenance() {
            validate_current_interpretation_bindings(store, provenance)?;
        }
        let snapshot_id = revision.snapshot_id().clone();
        let response = self.execute_operation(
            &request.project_id,
            OperationRequest {
                contract_version: OPERATION_CONTRACT_VERSION.into(),
                request_id: format!(
                    "conversation-analyse:{}:{}:{}",
                    request.revision_id, snapshot_id, request.evidence_id
                ),
                operation: Operation::AnalyseSnapshot {
                    revision_id: request.revision_id,
                    expected_snapshot_id: snapshot_id.clone(),
                    evidence_id: request.evidence_id.clone(),
                    settings: AnalysisSettings::frame2d(),
                },
            },
        )?;
        let run = match operation_result(response)? {
            OperationResult::SnapshotAnalysed { run } => *run,
            result => return Err(format!("unexpected analysis operation result: {result:?}")),
        };
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
        Ok(ConversationEvidenceResponse {
            evidence_id: request.evidence_id,
            authored_snapshot_id: snapshot_id,
            stale: false,
            status,
            summary,
            resolved_snapshot_id: run.evidence.resolved_snapshot_id().cloned(),
            canonical_run_id: run.canonical_run_id.clone(),
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
            canonical_run_id: evidence_record.canonical_run_id().map(str::to_owned),
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
        store
            .working_copy_operations
            .insert(response.working_copy_id.clone(), Vec::new());
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
                operations: vec![operation.clone()],
            })
            .map_err(display)?;
        store
            .working_copy_operations
            .entry(request.working_copy_id)
            .or_default()
            .push(operation);
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
        let operations = store
            .working_copy_operations
            .remove(&request.working_copy_id)
            .unwrap_or_default();
        if operations.is_empty() {
            store
                .working_copies
                .insert(request.working_copy_id, working_copy);
            return Err("working copy has no typed operations to commit".into());
        }
        let parent_revision_id = working_copy.parent_revision_id().clone();
        let proposal_id = ProposalId::new(format!("{}:proposal", request.working_copy_id));
        let project_id = request.project_id.clone();
        let conversation_id = request.conversation_id;
        let revision_id = request.revision_id;
        let working_copy_id = request.working_copy_id;
        let _ = store;
        let proposed = self.execute_operation(
            &project_id,
            OperationRequest {
                contract_version: OPERATION_CONTRACT_VERSION.into(),
                request_id: format!("working-copy-propose:{working_copy_id}"),
                operation: Operation::ProposeStructuralPatch {
                    proposal_id: proposal_id.clone(),
                    conversation_id: conversation_id.clone(),
                    expected_head_revision_id: parent_revision_id.clone(),
                    proposed_revision_id: revision_id,
                    patch: StructuralPatch { operations },
                    agent_provenance: None,
                },
            },
        );
        let proposed = match proposed {
            Ok(response) => response,
            Err(error) => {
                self.store_mut(&project_id)?
                    .working_copies
                    .insert(working_copy_id, working_copy);
                return Err(error);
            }
        };
        operation_result(proposed)?;
        let accepted = self.execute_operation(
            &project_id,
            OperationRequest {
                contract_version: OPERATION_CONTRACT_VERSION.into(),
                request_id: format!("working-copy-accept:{proposal_id}"),
                operation: Operation::AcceptStructuralPatch {
                    proposal_id,
                    conversation_id,
                    expected_head_revision_id: parent_revision_id.clone(),
                },
            },
        )?;
        match operation_result(accepted)? {
            OperationResult::StructuralPatchAccepted {
                revision_id,
                snapshot_id,
                ..
            } => Ok(ConversationRevisionResponse {
                revision_id,
                snapshot_id,
                parent_revision_id: Some(parent_revision_id),
                author: "manual".into(),
                agent_provenance: None,
            }),
            result => Err(format!("unexpected working-copy result: {result:?}")),
        }
    }
    pub(crate) fn state(
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
            agent_responses: store
                .agent_responses
                .get(conversation)
                .cloned()
                .unwrap_or_default(),
        })
    }
    pub(crate) fn persist_agent_response(
        &mut self,
        project: &ProjectId,
        conversation_id: &ConversationId,
        response: ConversationAgentRespondResponse,
    ) -> Result<(), String> {
        let store = self.store_mut(project)?;
        let responses = store
            .agent_responses
            .entry(conversation_id.clone())
            .or_default();
        if responses
            .iter()
            .any(|existing| existing.response_id == response.response_id)
        {
            return Ok(());
        }
        responses.push(response);
        let conversation = store
            .repository
            .conversation(conversation_id)
            .map_err(display)?;
        let messages = store
            .messages
            .get(conversation_id)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        if let Some(sqlite) = store.sqlite.as_mut() {
            sqlite
                .update_conversation_origin(
                    conversation_id,
                    &persisted_origin_json(
                        conversation.origin(),
                        &store.project_facts,
                        messages,
                        responses,
                    )?,
                )
                .map_err(display)?;
        }
        Ok(())
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
            canonical_run_id: serde_json::from_str::<PersistedEvidenceEnvelope>(
                &stored.manifest_json,
            )
            .ok()
            .and_then(|value| value.analysis_manifest)
            .and_then(|value| value.canonical_run_id),
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

fn project_workspace_database(
    project_dir: &str,
    revision_scope_id: &ProjectId,
) -> Result<PathBuf, String> {
    let project_dir = PathBuf::from(project_dir);
    if !project_dir.is_absolute() {
        return Err("conversation project directory must be absolute".into());
    }
    if let Ok(package) = load_project_package(&project_dir) {
        let design = package
            .designs
            .iter()
            .find(|design| design.manifest.id.as_str() == revision_scope_id.as_str())
            .or_else(|| (package.designs.len() == 1).then(|| &package.designs[0]))
            .ok_or_else(|| {
                format!(
                    "revision scope `{revision_scope_id}` does not identify a design in the project package"
                )
            })?;
        return design_package_paths(&project_dir, &design.manifest.id)
            .map(|paths| paths.workspace_database)
            .map_err(display);
    }
    if project_dir
        .parent()
        .and_then(Path::file_name)
        .is_some_and(|name| name == "designs")
    {
        let design_id = project_dir
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| "design directory name is not valid UTF-8".to_string())?;
        let root = project_dir
            .parent()
            .and_then(Path::parent)
            .ok_or_else(|| "design directory has no project root".to_string())?;
        let package = load_project_package(root).map_err(display)?;
        if !package
            .designs
            .iter()
            .any(|design| design.manifest.id.as_str() == design_id)
        {
            return Err("selected design is not present in the project package".into());
        }
        return Ok(project_dir.join("workspace.sqlite"));
    }
    Ok(project_dir.join(".fraia").join("workspace.sqlite"))
}

fn validate_proposal_source_context(
    store: &ProjectConversationStore,
    parent_revision_id: &RevisionId,
    context: &ConversationProposalSourceContext,
) -> Result<(), String> {
    let revision = store
        .repository
        .revision(parent_revision_id)
        .map_err(display)?;
    if revision.snapshot_id() != &context.expected_snapshot_id {
        return Err(format!(
            "proposal source context is stale: expected snapshot `{}`, current parent snapshot is `{}`",
            context.expected_snapshot_id,
            revision.snapshot_id()
        ));
    }
    let workspace = store
        .sqlite_path
        .as_ref()
        .ok_or_else(|| "proposal source context requires a durable design workspace".to_string())?;
    let design_dir = workspace
        .parent()
        .ok_or_else(|| "design workspace has no design directory".to_string())?;
    let actual_design_id = design_dir
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "design directory has no valid design id".to_string())?;
    if context.design_id.as_str() != actual_design_id {
        return Err("proposal source context belongs to a different design".into());
    }
    let project_dir = design_dir
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| "design workspace has no project root".to_string())?;
    let shelf = fraia_core::load_design_shelf(project_dir, &context.design_id).map_err(display)?;
    let mut seen = std::collections::BTreeSet::new();
    for item_id in &context.shelf_item_ids {
        if !seen.insert(item_id) {
            return Err(format!(
                "duplicate design reference `{item_id}` in proposal context"
            ));
        }
        let item = shelf
            .items
            .get(item_id)
            .ok_or_else(|| format!("unknown current-design reference `{item_id}`"))?;
        if !item.confirmation.confirmed {
            return Err(format!("design reference `{item_id}` is not confirmed"));
        }
    }
    let current_head = fraia_core::list_drawing_interpretations(project_dir, &context.design_id)
        .map_err(display)?
        .head_revision_id;
    let mut eligible_inferences = std::collections::BTreeSet::new();
    for interpretation_revision_id in &context.drawing_interpretation_revision_ids {
        if current_head.as_deref() != Some(interpretation_revision_id) {
            return Err(format!(
                "drawing interpretation binding `{interpretation_revision_id}` is stale"
            ));
        }
        let agent_context = fraia_core::drawing_interpretation_agent_context(
            project_dir,
            &context.design_id,
            interpretation_revision_id,
        )
        .map_err(display)?;
        eligible_inferences.extend(
            agent_context
                .inferred_assumptions
                .into_iter()
                .filter(|inference| !inference.materially_conflicted)
                .map(|inference| inference.inference_id),
        );
    }
    for inference_id in &context.drawing_interpretation_inference_ids {
        if !eligible_inferences.contains(inference_id) {
            return Err(format!(
                "drawing inference `{inference_id}` is missing, low-confidence, conflicted, or not bound to the exact interpretation revision"
            ));
        }
    }
    Ok(())
}

fn validate_current_interpretation_bindings(
    store: &ProjectConversationStore,
    provenance: &AgentTurnProvenance,
) -> Result<(), String> {
    if provenance.drawing_interpretation_revision_ids.is_empty() {
        return Ok(());
    }
    let workspace = store
        .sqlite_path
        .as_ref()
        .ok_or_else(|| "interpretation-bound proposal requires a durable workspace".to_string())?;
    let design_dir = workspace
        .parent()
        .ok_or_else(|| "design workspace has no design directory".to_string())?;
    let project_dir = design_dir
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| "design workspace has no project root".to_string())?;
    let design_id = fraia_core::DesignId::new(
        design_dir
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| "design workspace has no design id".to_string())?,
    );
    let head = fraia_core::list_drawing_interpretations(project_dir, &design_id)
        .map_err(display)?
        .head_revision_id;
    for revision_id in &provenance.drawing_interpretation_revision_ids {
        if head.as_deref() != Some(revision_id) {
            return Err(format!(
                "drawing interpretation binding `{revision_id}` is stale; current head is `{}`",
                head.as_deref().unwrap_or("none")
            ));
        }
    }
    for inference_id in &provenance.drawing_interpretation_inference_ids {
        if !provenance
            .drawing_interpretation_revision_ids
            .iter()
            .any(|revision| inference_id.starts_with(&format!("{revision}:inference:")))
        {
            return Err(format!(
                "drawing inference `{inference_id}` is not bound to an exact interpretation revision"
            ));
        }
    }
    Ok(())
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
            let operation = metadata
                .operation
                .and_then(|value| {
                    serde_json::from_value::<RevisionOperation>(value.clone())
                        .ok()
                        .or_else(|| match value.as_str() {
                            Some("accepted_proposal") => {
                                Some(RevisionOperation::AcceptedProposal {
                                    proposal_id: metadata.proposal_id.clone().unwrap_or_else(
                                        || ProposalId::new(format!("restored:{}", stored.id)),
                                    ),
                                })
                            }
                            Some("manual_edit") => Some(RevisionOperation::ManualEdit),
                            Some("user_patch") => Some(RevisionOperation::UserPatch),
                            Some("root") => Some(RevisionOperation::Root),
                            _ => None,
                        })
                })
                .unwrap_or_else(|| {
                    if author_kind == RevisionAuthorKind::Agent {
                        RevisionOperation::AcceptedProposal {
                            proposal_id: ProposalId::new(format!("restored:{}", stored.id)),
                        }
                    } else {
                        RevisionOperation::ManualEdit
                    }
                });
            let provenance = metadata.agent_provenance;
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
        let evidence = if let Ok(evidence) =
            serde_json::from_str::<AnalysisEvidence>(&stored.manifest_json)
        {
            evidence
        } else {
            let envelope = serde_json::from_str::<PersistedEvidenceEnvelope>(&stored.manifest_json)
                .map_err(display)?;
            let Some(manifest) = envelope.analysis_manifest else {
                continue;
            };
            AnalysisEvidence::with_analysis_manifest(
                stored.id.clone(),
                stored.authored_snapshot_id.clone(),
                stored.resolved_snapshot_id.clone(),
                envelope.dependencies,
                manifest,
            )
            .map_err(display)?
        };
        let manifest = evidence
            .analysis_manifest()
            .cloned()
            .ok_or_else(|| "analysis evidence has no typed manifest".to_string())?;
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
        analysis_runs.insert(
            stored.id,
            SnapshotAnalysisRun {
                revision_id,
                canonical_run_id: evidence.canonical_run_id().map(str::to_owned),
                evidence,
                outcome,
                resolved_snapshot,
            },
        );
    }

    let mut agent_responses = BTreeMap::from([(
        root.root_conversation.id.clone(),
        root_state.state.agent_responses,
    )]);
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
        if let Some(provenance) = stored.agent_provenance.as_ref()
            && let (Some(response_id), Some(text)) = (
                provenance.response_id.clone(),
                provenance.response_text.clone(),
            )
        {
            let responses = agent_responses
                .entry(stored.conversation_id.clone())
                .or_default();
            let operations = patch
                .operations
                .iter()
                .map(conversation_operation_from_structural)
                .collect::<Result<Vec<_>, _>>()?;
            let hydrated = ConversationAgentRespondResponse {
                response_id: response_id.clone(),
                text,
                questions: provenance.response_questions.clone(),
                proposal: Some(ConversationAgentProposalResponse {
                    proposal_id: stored.id.to_string(),
                    proposed_revision_id: stored.proposed_revision_id.clone(),
                    parent_revision_id: stored.parent_revision_id.clone(),
                    status: stored.status.clone(),
                    assumptions: provenance.assumptions.clone(),
                    evidence_limits: provenance.evidence_limits.clone(),
                    operations,
                }),
                provider: provenance.provider.clone(),
                model: provenance.model.clone(),
                reasoning_effort: provenance.reasoning_effort.clone().unwrap_or_default(),
                catalogue_refreshed_at: provenance.catalogue_refreshed_at.clone(),
                turn_id: provenance.turn_id.clone(),
            };
            if let Some(existing) = responses
                .iter_mut()
                .find(|response| response.response_id == response_id)
            {
                *existing = hydrated;
            } else {
                responses.push(hydrated);
            }
        }
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
        working_copy_operations: BTreeMap::new(),
        analysis_runs,
        agent_responses,
    })
}

fn persisted_conversation_envelope(origin_json: &str) -> PersistedConversationEnvelope {
    serde_json::from_str(origin_json).unwrap_or_default()
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
    agent_responses: &[ConversationAgentRespondResponse],
) -> Result<String, String> {
    serde_json::to_string(&serde_json::json!({
        "origin": origin,
        "projectFacts": project_facts,
        "messages": messages,
        "agentResponses": agent_responses,
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

fn conversation_operation_from_structural(
    operation: &StructuralOperation,
) -> Result<ConversationProposalOperation, String> {
    match operation {
        StructuralOperation::AddNode(node) => Ok(ConversationProposalOperation::AddNode {
            id: node.id.clone(),
            x: node.position.x.value,
            y: node.position.y.value,
            z: node.position.z.value,
        }),
        StructuralOperation::MoveNode { node_id, position } => {
            Ok(ConversationProposalOperation::MoveNode {
                node_id: node_id.clone(),
                x: position.x.value,
                y: position.y.value,
                z: position.z.value,
            })
        }
        StructuralOperation::AddMember(member) => Ok(ConversationProposalOperation::AddMember {
            id: member.id.clone(),
            start_node: member.start_node.clone(),
            end_node: member.end_node.clone(),
            role: member.role.clone(),
            section_id: member.section_id.clone(),
            material_id: member.material_id.clone(),
        }),
        StructuralOperation::AddSupport(support) => Ok(ConversationProposalOperation::AddSupport {
            id: support.id.clone(),
            target_node: support.target_node.clone(),
            ux: support.ux,
            uy: support.uy,
            uz: support.uz,
            rx: support.rx,
            ry: support.ry,
            rz: support.rz,
        }),
        StructuralOperation::SetMemberRole { member_id, role } => {
            Ok(ConversationProposalOperation::SetMemberRole {
                member_id: member_id.clone(),
                role: role.as_str().into(),
            })
        }
        _ => Err("persisted agent proposal contains an operation that the conversation adapter cannot project".into()),
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
                origin_json: persisted_origin_json(conversation.origin(), project_facts, &[], &[])?,
                head_revision_id: conversation.head_revision_id().clone(),
            },
            root_revision: stored_revision(revision, None)?,
            root_snapshot: stored_snapshot(snapshot),
        })
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

pub fn router(service: ConversationServiceHandle) -> Router {
    let analysis_attempts = Arc::new(AnalysisAttemptRegistry::default());
    Router::new()
        .route("/operations/v1/execute", post(execute_operation))
        .route("/analysis-attempts/start", post(start_analysis_attempt))
        .route("/analysis-attempts/status", post(analysis_attempt_status))
        .route("/analysis-attempts/cancel", post(cancel_analysis_attempt))
        .route("/conversations/create", post(create))
        .route("/conversations/converse", post(converse))
        .route("/conversations/facts", post(update_facts))
        .route("/conversations/propose", post(propose))
        .route("/conversations/accept", post(accept))
        .route("/conversations/reject", post(reject))
        .route("/conversations/fork", post(fork))
        .route("/conversations/resume", post(resume))
        .route("/conversations/unload", post(unload))
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
        .layer(Extension(analysis_attempts))
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct VersionedOperationRequest {
    project_id: ProjectId,
    request: OperationRequest,
}

async fn execute_operation(
    Extension(service): Extension<ConversationServiceHandle>,
    Json(request): Json<VersionedOperationRequest>,
) -> impl IntoResponse {
    match service
        .lock()
        .unwrap()
        .execute_operation(&request.project_id, request.request)
    {
        Ok(response) => {
            let status = operation_status(&response);
            (status, Json(response)).into_response()
        }
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error }),
        )
            .into_response(),
    }
}

async fn start_analysis_attempt(
    Extension(service): Extension<ConversationServiceHandle>,
    Extension(registry): Extension<Arc<AnalysisAttemptRegistry>>,
    Json(request): Json<AnalysisAttemptStartRequest>,
) -> impl IntoResponse {
    let (revision_id, authored_snapshot_id, evidence_id) = match &request.request.operation {
        Operation::AnalyseSnapshot {
            revision_id,
            expected_snapshot_id,
            evidence_id,
            ..
        } => (
            revision_id.clone(),
            expected_snapshot_id.clone(),
            evidence_id.clone(),
        ),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "analysis attempt start requires analyse_snapshot".into(),
                }),
            )
                .into_response();
        }
    };
    let mut random = [0u8; 24];
    if let Err(error) = getrandom::fill(&mut random) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: error.to_string(),
            }),
        )
            .into_response();
    }
    let attempt_id = random
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    // 0 running, 1 cancellation reserved, 2 publication reserved.
    let lifecycle = Arc::new(AtomicU8::new(0));
    let initial = AnalysisAttemptResponse {
        attempt_id: attempt_id.clone(),
        project_id: request.project_id.clone(),
        revision_id,
        authored_snapshot_id,
        evidence_id,
        stage: fraia_revision::analysis_service::AnalysisExecutionStage::Preparing,
        status: AnalysisAttemptStatus::Running,
        elapsed_millis: 0,
        canonical_run_id: None,
        diagnostics: Vec::new(),
    };
    let attempt_directory = match service
        .lock()
        .unwrap()
        .analysis_attempt_path(&request.project_id, &attempt_id)
    {
        Ok(path) => path,
        Err(error) => {
            return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error })).into_response();
        }
    };
    if let Err(error) = persist_analysis_attempt(&attempt_directory, 0, &initial) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error }),
        )
            .into_response();
    }
    registry.attempts.lock().unwrap().insert(
        attempt_id.clone(),
        AnalysisAttemptEntry {
            lifecycle: lifecycle.clone(),
            started: Instant::now(),
            response: initial.clone(),
            sequence: 0,
        },
    );
    let registry_for_job = registry.clone();
    let attempt_id_for_job = attempt_id.clone();
    let attempt_directory_for_job = attempt_directory.clone();
    let test_control = registry.test_control;
    tokio::spawn(async move {
        // Let the start handler publish its 202 response before any worker can
        // contend for the design service lock. The CPU-bound operation then
        // runs only on Tokio's blocking pool.
        tokio::task::yield_now().await;
        let _ = tokio::task::spawn_blocking(move || {
        let registry_for_progress = registry_for_job.clone();
        let progress_attempt_id = attempt_id_for_job.clone();
        let progress_directory = attempt_directory_for_job.clone();
        let mut progress = move |stage| {
            if let Ok(mut attempts) = registry_for_progress.attempts.lock()
                && let Some(entry) = attempts.get_mut(&progress_attempt_id)
            {
                entry.response.stage = stage;
                entry.response.elapsed_millis = entry
                    .started
                    .elapsed()
                    .as_millis()
                    .try_into()
                    .unwrap_or(u64::MAX);
                entry.sequence += 1;
                let _ =
                    persist_analysis_attempt(&progress_directory, entry.sequence, &entry.response);
            }
        };
        let lifecycle_for_cancel = lifecycle.clone();
        let mut cancelled = move || lifecycle_for_cancel.load(Ordering::Acquire) == 1;
        let lifecycle_for_publication = lifecycle.clone();
        let mut begin_publication = move || {
            lifecycle_for_publication
                .compare_exchange(0, 2, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
        };
        let mut control = AnalysisOperationControl {
            progress: &mut progress,
            cancelled: &mut cancelled,
            begin_publication: &mut begin_publication,
        };
        let delay_started = Instant::now();
        while delay_started.elapsed() < Duration::from_millis(test_control.delay_millis)
            && lifecycle.load(Ordering::Acquire) == 0
        {
            let remaining = Duration::from_millis(test_control.delay_millis)
                .saturating_sub(delay_started.elapsed());
            std::thread::sleep(remaining.min(Duration::from_millis(10)));
        }
        let result = if test_control.force_failure && lifecycle.load(Ordering::Acquire) == 0 {
            Err("analysis.test-forced-failure: unpackaged test control requested failure".into())
        } else {
            service.lock().unwrap().execute_operation_maybe_controlled(
                &request.project_id,
                request.request,
                Some(&mut control),
            )
        };
        if let Ok(mut attempts) = registry_for_job.attempts.lock()
            && let Some(entry) = attempts.get_mut(&attempt_id_for_job)
        {
            entry.response.elapsed_millis = entry
                .started
                .elapsed()
                .as_millis()
                .try_into()
                .unwrap_or(u64::MAX);
            if lifecycle.load(Ordering::Acquire) == 1 {
                entry.response.status = AnalysisAttemptStatus::Cancelled;
                entry.response.diagnostics = vec!["analysis.cancelled: cancelled by user".into()];
                entry.sequence += 1;
                let _ = persist_analysis_attempt(
                    &attempt_directory_for_job,
                    entry.sequence,
                    &entry.response,
                );
                return;
            }
            match result {
                Ok(response) => {
                    match response.outcome {
                        OperationOutcome::Success { result } => match *result {
                            OperationResult::SnapshotAnalysed { run } => {
                                entry.response.canonical_run_id = run.canonical_run_id.clone();
                                entry.response.diagnostics = run.outcome.diagnostics().to_vec();
                                entry.response.status = match run.outcome {
                                    SnapshotAnalysisOutcome::Completed { .. } => {
                                        AnalysisAttemptStatus::Completed
                                    }
                                    SnapshotAnalysisOutcome::Failed { .. } => {
                                        AnalysisAttemptStatus::Failed
                                    }
                                    SnapshotAnalysisOutcome::Unsupported { .. } => {
                                        AnalysisAttemptStatus::Unsupported
                                    }
                                };
                            }
                            _ => {
                                entry.response.status = AnalysisAttemptStatus::Failed;
                                entry.response.diagnostics = vec!["analysis.invalid-result: operation returned a non-analysis result".into()];
                            }
                        },
                        OperationOutcome::Error { error } => {
                            entry.response.status = if error.code == OperationErrorCode::Cancelled {
                                AnalysisAttemptStatus::Cancelled
                            } else {
                                AnalysisAttemptStatus::Failed
                            };
                            entry.response.diagnostics = vec![error.message];
                        }
                    }
                }
                Err(error) => {
                    entry.response.status = AnalysisAttemptStatus::Failed;
                    entry.response.diagnostics = vec![error];
                }
            }
            entry.sequence += 1;
            let _ = persist_analysis_attempt(
                &attempt_directory_for_job,
                entry.sequence,
                &entry.response,
            );
        }
        })
        .await;
    });
    (StatusCode::ACCEPTED, Json(initial)).into_response()
}

async fn analysis_attempt_status(
    Extension(service): Extension<ConversationServiceHandle>,
    Extension(registry): Extension<Arc<AnalysisAttemptRegistry>>,
    Json(request): Json<AnalysisAttemptIdRequest>,
) -> impl IntoResponse {
    let mut attempts = registry.attempts.lock().unwrap();
    if !attempts.contains_key(&request.attempt_id) {
        let directory = match service
            .lock()
            .unwrap()
            .analysis_attempt_path(&request.project_id, &request.attempt_id)
        {
            Ok(path) => path,
            Err(error) => {
                return (StatusCode::NOT_FOUND, Json(ErrorResponse { error })).into_response();
            }
        };
        let (mut sequence, mut response) = match load_latest_analysis_attempt(&directory) {
            Ok(value) => value,
            Err(error) => {
                return (StatusCode::NOT_FOUND, Json(ErrorResponse { error })).into_response();
            }
        };
        if matches!(
            response.status,
            AnalysisAttemptStatus::Running | AnalysisAttemptStatus::Cancelling
        ) {
            response.status = AnalysisAttemptStatus::Failed;
            response.diagnostics.push("analysis.interrupted: app restarted before a terminal evidence boundary; retry creates a new attempt".into());
            sequence += 1;
            if let Err(error) = persist_analysis_attempt(&directory, sequence, &response) {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse { error }),
                )
                    .into_response();
            }
        }
        attempts.insert(
            request.attempt_id.clone(),
            AnalysisAttemptEntry {
                lifecycle: Arc::new(AtomicU8::new(2)),
                started: Instant::now(),
                response,
                sequence,
            },
        );
    }
    let entry = attempts.get(&request.attempt_id).unwrap();
    if entry.response.project_id != request.project_id {
        return (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "analysis attempt was not found".into(),
            }),
        )
            .into_response();
    }
    let mut response = entry.response.clone();
    response.elapsed_millis = entry
        .started
        .elapsed()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX);
    (StatusCode::OK, Json(response)).into_response()
}

async fn cancel_analysis_attempt(
    Extension(service): Extension<ConversationServiceHandle>,
    Extension(registry): Extension<Arc<AnalysisAttemptRegistry>>,
    Json(request): Json<AnalysisAttemptIdRequest>,
) -> impl IntoResponse {
    let durable_directory = service
        .lock()
        .unwrap()
        .analysis_attempt_path(&request.project_id, &request.attempt_id)
        .ok();
    let mut attempts = registry.attempts.lock().unwrap();
    let Some(entry) = attempts.get_mut(&request.attempt_id) else {
        return (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "analysis attempt was not found".into(),
            }),
        )
            .into_response();
    };
    if entry.response.project_id != request.project_id {
        return (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "analysis attempt was not found".into(),
            }),
        )
            .into_response();
    }
    if entry.response.status == AnalysisAttemptStatus::Running {
        if entry
            .lifecycle
            .compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            entry.response.status = AnalysisAttemptStatus::Cancelling;
            entry.sequence += 1;
            if let Some(directory) = durable_directory {
                let _ = persist_analysis_attempt(&directory, entry.sequence, &entry.response);
            }
        }
    }
    (StatusCode::OK, Json(entry.response.clone())).into_response()
}

fn operation_status(response: &OperationResponse) -> StatusCode {
    match &response.outcome {
        OperationOutcome::Success { .. } => StatusCode::OK,
        OperationOutcome::Error { error }
            if matches!(
                error.code,
                OperationErrorCode::ExpectedHeadMismatch
                    | OperationErrorCode::ExpectedSnapshotMismatch
            ) =>
        {
            StatusCode::CONFLICT
        }
        OperationOutcome::Error { error } if error.code == OperationErrorCode::RepositoryError => {
            StatusCode::INTERNAL_SERVER_ERROR
        }
        OperationOutcome::Error { .. } => StatusCode::BAD_REQUEST,
    }
}

fn operation_result(response: OperationResponse) -> Result<OperationResult, String> {
    match response.outcome {
        OperationOutcome::Success { result } => Ok(*result),
        OperationOutcome::Error { error } => Err(error.message),
    }
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConversationUnloadRequest {
    project_id: String,
}

async fn unload(
    Extension(service): Extension<ConversationServiceHandle>,
    Json(request): Json<ConversationUnloadRequest>,
) -> impl IntoResponse {
    Json(serde_json::json!({
        "unloaded": service.lock().unwrap().unload(&request.project_id),
    }))
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
    use fraia_revision::{
        analysis_service::AnalysisSettings,
        operations::{OPERATION_CONTRACT_VERSION, OperationResult},
        patch::NodeInput,
        repository::ProposalId,
    };
    use tempfile::tempdir;

    #[test]
    fn unpackaged_debug_analysis_test_controls_are_bounded_and_explicit() {
        let values = BTreeMap::from([
            (TEST_ANALYSIS_DELAY_ENV, "999999".to_string()),
            (TEST_ANALYSIS_FAILURE_ENV, "1".to_string()),
        ]);
        let control = resolve_analysis_attempt_test_control(true, |name| values.get(name).cloned());
        assert_eq!(control.delay_millis, MAX_TEST_ANALYSIS_DELAY_MILLIS);
        assert!(control.force_failure);

        let invalid = BTreeMap::from([
            (TEST_ANALYSIS_DELAY_ENV, "not-a-number".to_string()),
            (TEST_ANALYSIS_FAILURE_ENV, "true".to_string()),
        ]);
        assert_eq!(
            resolve_analysis_attempt_test_control(true, |name| invalid.get(name).cloned()),
            AnalysisAttemptTestControl::default()
        );
    }

    #[test]
    fn production_analysis_ignores_test_control_environment() {
        let control = resolve_analysis_attempt_test_control(false, |_| Some("1".to_string()));
        assert_eq!(control, AnalysisAttemptTestControl::default());
    }

    #[tokio::test]
    async fn analysis_attempt_start_returns_before_delayed_worker_takes_service_lock() {
        let parent = tempdir().unwrap();
        let project_dir = parent.path().join("attempt-start-project");
        let package =
            fraia_core::create_named_project_package(&project_dir, "Attempt start").unwrap();
        let project_id = ProjectId::from(package.designs[0].manifest.id.as_str());
        let conversation_id = ConversationId::from("overall");
        let service = ConversationService::default();
        let handle = Arc::new(Mutex::new(service));
        let created = handle
            .lock()
            .unwrap()
            .create(ConversationCreateRequest {
                project_id: project_id.clone(),
                project_dir: project_dir.display().to_string(),
                conversation_id,
                purpose: "Overall framing".into(),
                project_facts: Default::default(),
            })
            .unwrap();
        let registry = Arc::new(AnalysisAttemptRegistry {
            attempts: Mutex::default(),
            test_control: AnalysisAttemptTestControl {
                delay_millis: 500,
                force_failure: false,
            },
        });
        let request = AnalysisAttemptStartRequest {
            project_id,
            request: OperationRequest {
                contract_version: OPERATION_CONTRACT_VERSION.into(),
                request_id: "delayed-start".into(),
                operation: Operation::AnalyseSnapshot {
                    revision_id: created.head_revision_id,
                    expected_snapshot_id: created.head_snapshot_id,
                    evidence_id: EvidenceId::from("delayed-start-evidence"),
                    settings: AnalysisSettings::frame3d(),
                },
            },
        };
        let response = tokio::time::timeout(
            Duration::from_millis(100),
            start_analysis_attempt(Extension(handle), Extension(registry), Json(request)),
        )
        .await
        .expect("start response must not wait for the analysis worker")
        .into_response();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }

    #[test]
    fn analysis_attempt_journal_recovers_latest_terminal_state_and_keeps_prior_entries() {
        let directory = tempdir().unwrap();
        let attempt_dir = directory.path().join("attempt-a");
        let mut response = AnalysisAttemptResponse {
            attempt_id: "a".repeat(48),
            project_id: ProjectId::new("design-a"),
            revision_id: RevisionId::from("revision-a"),
            authored_snapshot_id: fraia_revision::SnapshotId::from("snapshot-a"),
            evidence_id: EvidenceId::from("evidence-a"),
            stage: fraia_revision::analysis_service::AnalysisExecutionStage::Preparing,
            status: AnalysisAttemptStatus::Running,
            elapsed_millis: 0,
            canonical_run_id: None,
            diagnostics: Vec::new(),
        };
        persist_analysis_attempt(&attempt_dir, 0, &response).unwrap();
        response.stage = fraia_revision::analysis_service::AnalysisExecutionStage::Collecting;
        response.status = AnalysisAttemptStatus::Failed;
        response.elapsed_millis = 42;
        response.diagnostics = vec!["solver.failed: fixture".into()];
        persist_analysis_attempt(&attempt_dir, 1, &response).unwrap();
        let (sequence, recovered) = load_latest_analysis_attempt(&attempt_dir).unwrap();
        assert_eq!(sequence, 1);
        assert_eq!(recovered, response);
        assert_eq!(std::fs::read_dir(&attempt_dir).unwrap().count(), 2);
    }

    #[test]
    fn two_designs_reopen_distinct_user_and_agent_histories() {
        let directory = tempdir().unwrap();
        let mut package =
            fraia_core::create_named_project_package(directory.path(), "House").unwrap();
        let first = package.designs[0].clone();
        let second_id = fraia_core::DesignId::new("design-second");
        let mut second = first.clone();
        second.manifest.id = second_id.clone();
        second.manifest.name = "Braced option".into();
        package
            .manifest
            .designs
            .push(fraia_core::ProjectDesignEntry {
                id: second_id.clone(),
                name: second.manifest.name.clone(),
            });
        package.designs.push(second);
        fraia_core::save_project_package(directory.path(), &package).unwrap();

        let first_id = first.manifest.id;
        let conversation_id = ConversationId::from("overall-framing");
        let mut service = ConversationService::default();
        for (design_id, user_text, response_id, response_text) in [
            (
                &first_id,
                "Design the main frame.",
                "response-main",
                "Main-frame reply.",
            ),
            (
                &second_id,
                "Design the braced frame.",
                "response-braced",
                "Braced-frame reply.",
            ),
        ] {
            let project_id = ProjectId::from(design_id.as_str());
            service
                .create(ConversationCreateRequest {
                    project_id: project_id.clone(),
                    project_dir: directory.path().display().to_string(),
                    conversation_id: conversation_id.clone(),
                    purpose: "Overall framing".into(),
                    project_facts: Default::default(),
                })
                .unwrap();
            service
                .converse(ConversationMessageRequest {
                    project_id: project_id.clone(),
                    conversation_id: conversation_id.clone(),
                    message: user_text.into(),
                })
                .unwrap();
            service
                .persist_agent_response(
                    &project_id,
                    &conversation_id,
                    ConversationAgentRespondResponse {
                        response_id: response_id.into(),
                        text: response_text.into(),
                        questions: Vec::new(),
                        proposal: None,
                        provider: "fake".into(),
                        model: "gpt-5.6-luna".into(),
                        reasoning_effort: "high".into(),
                        catalogue_refreshed_at: None,
                        turn_id: format!("turn-{response_id}"),
                    },
                )
                .unwrap();
        }
        assert_ne!(
            service
                .workspace_path(&ProjectId::from(first_id.as_str()))
                .unwrap(),
            service
                .workspace_path(&ProjectId::from(second_id.as_str()))
                .unwrap()
        );
        drop(service);

        let mut restarted = ConversationService::default();
        for (design_id, user_text, response_id) in [
            (&first_id, "Design the main frame.", "response-main"),
            (&second_id, "Design the braced frame.", "response-braced"),
        ] {
            let state = restarted
                .create(ConversationCreateRequest {
                    project_id: ProjectId::from(design_id.as_str()),
                    project_dir: directory.path().display().to_string(),
                    conversation_id: conversation_id.clone(),
                    purpose: "Overall framing".into(),
                    project_facts: Default::default(),
                })
                .unwrap();
            assert_eq!(state.messages, vec![user_text]);
            assert_eq!(state.agent_responses.len(), 1);
            assert_eq!(state.agent_responses[0].response_id, response_id);
        }
    }

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

    #[test]
    fn versioned_operation_adapter_uses_design_database_and_survives_restart() {
        let parent = tempdir().unwrap();
        let project_dir = parent.path().join("operation-project");
        let created =
            fraia_core::create_named_project_package(&project_dir, "Operation project").unwrap();
        let request = ConversationCreateRequest {
            project_id: ProjectId::from(created.designs[0].manifest.id.as_str()),
            project_dir: project_dir.display().to_string(),
            conversation_id: ConversationId::from("overall"),
            purpose: "Overall framing".into(),
            project_facts: Default::default(),
        };
        let mut service = ConversationService::default();
        let root = service.create(request.clone()).unwrap();
        let proposal = OperationRequest {
            contract_version: OPERATION_CONTRACT_VERSION.into(),
            request_id: "appd-propose".into(),
            operation: Operation::ProposeStructuralPatch {
                proposal_id: ProposalId::from("proposal-1"),
                conversation_id: request.conversation_id.clone(),
                expected_head_revision_id: root.head_revision_id.clone(),
                proposed_revision_id: RevisionId::from("revision-1"),
                patch: StructuralPatch {
                    operations: vec![StructuralOperation::AddNode(NodeInput {
                        id: "node-1".into(),
                        position: Position {
                            x: Length::meters(0.0),
                            y: Length::meters(0.0),
                            z: Length::meters(0.0),
                        },
                    })],
                },
                agent_provenance: None,
            },
        };
        assert_eq!(
            operation_status(
                &service
                    .execute_operation(&request.project_id, proposal)
                    .unwrap()
            ),
            StatusCode::OK
        );
        let accepted = service
            .execute_operation(
                &request.project_id,
                OperationRequest {
                    contract_version: OPERATION_CONTRACT_VERSION.into(),
                    request_id: "appd-accept".into(),
                    operation: Operation::AcceptStructuralPatch {
                        proposal_id: ProposalId::from("proposal-1"),
                        conversation_id: request.conversation_id.clone(),
                        expected_head_revision_id: root.head_revision_id,
                    },
                },
            )
            .unwrap();
        let state = service
            .state(&request.project_id, &request.conversation_id)
            .unwrap();
        assert_eq!(state.head_revision_id, RevisionId::from("revision-1"));

        let analysis_request = OperationRequest {
            contract_version: OPERATION_CONTRACT_VERSION.into(),
            request_id: "appd-analysis".into(),
            operation: Operation::AnalyseSnapshot {
                revision_id: state.head_revision_id.clone(),
                expected_snapshot_id: state.head_snapshot_id.clone(),
                evidence_id: EvidenceId::from("evidence-1"),
                settings: AnalysisSettings::frame3d(),
            },
        };
        let analysed = service
            .execute_operation(&request.project_id, analysis_request.clone())
            .unwrap();
        let canonical_run_id = match &analysed.outcome {
            OperationOutcome::Success { result } => match result.as_ref() {
                OperationResult::SnapshotAnalysed { run } => run
                    .canonical_run_id
                    .clone()
                    .expect("managed package analysis publishes a canonical design run"),
                other => panic!("unexpected analysis result: {other:?}"),
            },
            other => panic!("unexpected analysis response: {other:?}"),
        };
        match &analysed.outcome {
            OperationOutcome::Success { result } => match result.as_ref() {
                OperationResult::SnapshotAnalysed { run } => assert!(matches!(
                    run.outcome,
                    SnapshotAnalysisOutcome::Unsupported { .. }
                )),
                other => panic!("unexpected analysis result: {other:?}"),
            },
            other => panic!("unexpected analysis response: {other:?}"),
        }

        let package = load_project_package(&project_dir).unwrap();
        let listed_runs =
            fraia_core::list_design_runs(&project_dir, &package.designs[0].manifest.id).unwrap();
        assert!(
            listed_runs
                .runs
                .iter()
                .any(|run| run.run_id == canonical_run_id)
        );
        let workspace = design_package_paths(&project_dir, &package.designs[0].manifest.id)
            .unwrap()
            .workspace_database;
        assert!(workspace.is_file());
        let direct = execute_sqlite_operation(
            &mut SqliteRevisionRepository::open(&workspace).unwrap(),
            analysis_request.clone(),
        );
        assert_eq!(
            serde_json::to_value(direct).unwrap(),
            serde_json::to_value(&analysed).unwrap()
        );
        drop(service);

        let mut restarted = ConversationService::open_durable(&workspace).unwrap();
        let restored = restarted.create(request.clone()).unwrap();
        assert_eq!(restored.head_revision_id, RevisionId::from("revision-1"));
        let replay = restarted
            .execute_operation(&request.project_id, analysis_request)
            .unwrap();
        assert_eq!(
            serde_json::to_value(replay).unwrap(),
            serde_json::to_value(analysed).unwrap()
        );
        let inspected = restarted
            .execute_operation(
                &request.project_id,
                OperationRequest {
                    contract_version: OPERATION_CONTRACT_VERSION.into(),
                    request_id: "appd-inspect-evidence".into(),
                    operation: Operation::InspectAnalysisEvidence {
                        evidence_id: EvidenceId::from("evidence-1"),
                        against_revision_id: restored.head_revision_id,
                    },
                },
            )
            .unwrap();
        assert_eq!(operation_status(&inspected), StatusCode::OK);
        assert_eq!(operation_status(&accepted), StatusCode::OK);
    }

    #[test]
    fn versioned_operation_adapter_maps_exact_snapshot_conflict_to_http_conflict() {
        let mut service = ConversationService::default();
        let request = create_request();
        let root = service.create(request.clone()).unwrap();
        let response = service
            .execute_operation(
                &request.project_id,
                OperationRequest {
                    contract_version: OPERATION_CONTRACT_VERSION.into(),
                    request_id: "snapshot-conflict".into(),
                    operation: Operation::ValidateSnapshot {
                        revision_id: root.head_revision_id,
                        expected_snapshot_id: fraia_revision::SnapshotId::from("wrong"),
                    },
                },
            )
            .unwrap();
        assert_eq!(operation_status(&response), StatusCode::CONFLICT);
        assert!(matches!(
            response.outcome,
            OperationOutcome::Error { error }
                if error.code == OperationErrorCode::ExpectedSnapshotMismatch
        ));
    }

    #[test]
    fn proposal_rejects_stale_or_cross_design_source_binding_before_mutation() {
        let mut service = ConversationService::default();
        let root = service.create_fixture_demo(create_request()).unwrap();

        let mut stale = proposal();
        stale.source_context = Some(ConversationProposalSourceContext {
            design_id: fraia_core::DesignId::new("wrong-design"),
            expected_snapshot_id: fraia_revision::SnapshotId::from("stale-snapshot"),
            shelf_item_ids: Vec::new(),
            assumptions: vec!["Drawing scale is not confirmed.".into()],
            evidence_limits: vec!["No elevation was selected.".into()],
            drawing_interpretation_revision_ids: Vec::new(),
            drawing_interpretation_inference_ids: Vec::new(),
        });
        assert!(service.propose(stale).unwrap_err().contains("is stale"));

        let mut cross_design = proposal();
        cross_design.source_context = Some(ConversationProposalSourceContext {
            design_id: fraia_core::DesignId::new("wrong-design"),
            expected_snapshot_id: root.head_snapshot_id.clone(),
            shelf_item_ids: Vec::new(),
            assumptions: Vec::new(),
            evidence_limits: Vec::new(),
            drawing_interpretation_revision_ids: Vec::new(),
            drawing_interpretation_inference_ids: Vec::new(),
        });
        assert!(
            service
                .propose(cross_design)
                .unwrap_err()
                .contains("different design")
        );
        assert!(
            service
                .store(&ProjectId::from("p"))
                .unwrap()
                .repository
                .proposal(&ProposalId::from("pr1"))
                .is_err()
        );
    }

    #[test]
    fn structured_agent_response_and_full_provenance_survive_restart() {
        let directory = tempdir().unwrap();
        let mut request = create_request();
        request.project_dir = directory.path().display().to_string();
        let mut service = ConversationService::default();
        service.create(request.clone()).unwrap();
        let response = ConversationAgentRespondResponse {
            response_id: "response-1".into(),
            text: "Review this exact proposal.".into(),
            questions: vec!["Confirm the support assumption?".into()],
            proposal: None,
            provider: "openai-codex".into(),
            model: "gpt-5.6-luna".into(),
            reasoning_effort: "high".into(),
            catalogue_refreshed_at: Some("2026-08-13T00:00:00Z".into()),
            turn_id: "turn-1".into(),
        };
        service
            .persist_agent_response(
                &request.project_id,
                &request.conversation_id,
                response.clone(),
            )
            .unwrap();
        drop(service);

        let mut restarted = ConversationService::default();
        let state = restarted.create(request).unwrap();
        assert_eq!(state.agent_responses.len(), 1);
        assert_eq!(state.agent_responses[0].response_id, response.response_id);
        assert_eq!(state.agent_responses[0].text, response.text);
        assert_eq!(state.agent_responses[0].questions, response.questions);
        assert_eq!(state.agent_responses[0].reasoning_effort, "high");
    }

    #[test]
    fn pending_proposal_recovers_its_response_from_atomic_provenance() {
        let directory = tempdir().unwrap();
        fraia_core::create_named_project_package(directory.path(), "Recovery project").unwrap();
        let mut request = create_request();
        request.project_dir = directory.path().display().to_string();
        let mut service = ConversationService::default();
        let root = service.create(request.clone()).unwrap();
        let mut proposal = first_geometry_proposal(root.head_revision_id.clone());
        proposal.reasoning_effort = Some("high".into());
        proposal.catalogue_refreshed_at = Some("2026-08-13T00:00:00Z".into());
        proposal.response_id = Some("atomic-response".into());
        proposal.response_text = Some("Review this recovered proposal.".into());
        proposal.response_questions = vec!["Confirm the assumption?".into()];
        let design_id = service
            .projects
            .get(&ProjectId::from("p"))
            .unwrap()
            .sqlite_path
            .as_ref()
            .unwrap()
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        proposal.source_context = Some(ConversationProposalSourceContext {
            design_id: fraia_core::DesignId::new(design_id),
            expected_snapshot_id: root.head_snapshot_id,
            shelf_item_ids: Vec::new(),
            assumptions: vec!["Support locations need review.".into()],
            evidence_limits: vec!["No analysis results are available.".into()],
            drawing_interpretation_revision_ids: Vec::new(),
            drawing_interpretation_inference_ids: Vec::new(),
        });
        let response_operations = proposal.operations.clone();
        service.propose(proposal).unwrap();
        service
            .persist_agent_response(
                &request.project_id,
                &request.conversation_id,
                ConversationAgentRespondResponse {
                    response_id: "atomic-response".into(),
                    text: "Review this recovered proposal.".into(),
                    questions: vec!["Confirm the assumption?".into()],
                    proposal: Some(ConversationAgentProposalResponse {
                        proposal_id: "first-geometry".into(),
                        proposed_revision_id: RevisionId::from("r1"),
                        parent_revision_id: root.head_revision_id,
                        status: "pending".into(),
                        assumptions: vec!["Support locations need review.".into()],
                        evidence_limits: vec!["No analysis results are available.".into()],
                        operations: response_operations,
                    }),
                    provider: "fake".into(),
                    model: "test".into(),
                    reasoning_effort: "high".into(),
                    catalogue_refreshed_at: Some("2026-08-13T00:00:00Z".into()),
                    turn_id: "first".into(),
                },
            )
            .unwrap();
        service
            .accept(ConversationProposalActionRequest {
                project_id: ProjectId::from("p"),
                proposal_id: "first-geometry".into(),
                provider: Some("fake".into()),
                model: Some("test".into()),
                turn_id: Some("first".into()),
            })
            .unwrap();
        drop(service);

        let mut restarted = ConversationService::default();
        let state = restarted.create(request).unwrap();
        assert_eq!(state.agent_responses.len(), 1);
        assert_eq!(state.agent_responses[0].response_id, "atomic-response");
        assert_eq!(
            state.agent_responses[0].proposal.as_ref().unwrap().status,
            "accepted"
        );
        assert_eq!(
            state.agent_responses[0]
                .proposal
                .as_ref()
                .unwrap()
                .operations
                .len(),
            5
        );
        let accepted = restarted
            .projects
            .get(&ProjectId::from("p"))
            .unwrap()
            .repository
            .revision(&RevisionId::from("r1"))
            .unwrap();
        let provenance = accepted.agent_provenance().unwrap();
        assert_eq!(provenance.response_id.as_deref(), Some("atomic-response"));
        assert!(provenance.shelf_item_ids.is_empty());
        assert!(provenance.drawing_interpretation_revision_ids.is_empty());
        assert_eq!(
            provenance.evidence_limits,
            vec!["No analysis results are available."]
        );
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
            reasoning_effort: None,
            catalogue_refreshed_at: None,
            response_id: None,
            response_text: None,
            response_questions: Vec::new(),
            source_context: None,
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
            reasoning_effort: None,
            catalogue_refreshed_at: None,
            response_id: None,
            response_text: None,
            response_questions: Vec::new(),
            source_context: None,
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
            assert_eq!(accepted.agent_provenance.unwrap().turn_id, "first");
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
            "Analysis completed against the accepted snapshot."
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
        assert_eq!(restored_provenance.turn_id, "first");
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
                reasoning_effort: None,
                catalogue_refreshed_at: None,
                response_id: None,
                response_text: None,
                response_questions: Vec::new(),
                source_context: None,
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

    #[test]
    fn accepted_revision_and_snapshot_survive_legacy_package_migration_and_restart() {
        let parent = tempdir().unwrap();
        let project_dir = parent.path().join("legacy-lineage-project");
        fraia_core::create_project(&project_dir, "Legacy lineage").expect("legacy project");
        let legacy_workspace = project_dir.join(".fraia/workspace.sqlite");
        let mut request = create_request();
        request.project_dir = project_dir.display().to_string();

        let accepted_revision;
        let accepted_snapshot;
        {
            let mut service = ConversationService::open_durable(&legacy_workspace).unwrap();
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
                    turn_id: Some("migration-turn".into()),
                })
                .unwrap();
            accepted_revision = accepted.revision_id;
            accepted_snapshot = accepted.snapshot_id;
        }

        crate::migrate_legacy_app_project(&project_dir)
            .expect("appd migrates legacy package on open");
        let package = load_project_package(&project_dir).expect("migrated package");
        let design_paths = design_package_paths(&project_dir, &package.designs[0].manifest.id)
            .expect("design package paths");
        assert!(design_paths.workspace_database.is_file());
        assert_eq!(
            project_workspace_database(&request.project_dir, &request.project_id).unwrap(),
            design_paths.workspace_database
        );

        let migrated_repository =
            SqliteRevisionRepository::open(&design_paths.workspace_database).unwrap();
        assert_eq!(
            migrated_repository
                .revision(&accepted_revision)
                .unwrap()
                .snapshot_id,
            accepted_snapshot
        );
        assert!(migrated_repository.snapshot(&accepted_snapshot).is_ok());
        let proposal = migrated_repository
            .proposal(&ProposalId::from("first-geometry"))
            .expect("accepted proposal survives migration");
        assert_eq!(
            proposal.accepted_revision_id,
            Some(accepted_revision.clone())
        );

        let mut restarted = ConversationService::open_durable(&design_paths.workspace_database)
            .expect("restart conversation service");
        let restored = restarted
            .create(request.clone())
            .expect("hydrate migrated workspace");
        assert_eq!(restored.head_revision_id, accepted_revision.clone());
        assert_eq!(restored.head_snapshot_id, accepted_snapshot);
        assert_eq!(restored.semantic_summary.counts.nodes, 2);
        assert_eq!(restored.semantic_summary.counts.members, 1);

        let (mut app_project, _) = crate::load_project(&project_dir).expect("load package state");
        app_project.requirements.span_m = 31.0;
        crate::save_project(&project_dir, &app_project)
            .expect("save package state while design workspace is open");
        restarted
            .propose(ConversationProposalRequest {
                project_id: ProjectId::from("p"),
                conversation_id: ConversationId::from("overall"),
                proposal_id: "post-package-save".into(),
                proposed_revision_id: RevisionId::from("post-package-save-r2"),
                parent_revision_id: accepted_revision,
                provider: "provider".into(),
                model: "model".into(),
                turn_id: "post-package-save-turn".into(),
                reasoning_effort: None,
                catalogue_refreshed_at: None,
                response_id: None,
                response_text: None,
                response_questions: Vec::new(),
                source_context: None,
                operations: vec![ConversationProposalOperation::AddNode {
                    id: "post-save-node".into(),
                    x: 8.0,
                    y: 0.0,
                    z: 0.0,
                }],
                operation: None,
            })
            .expect("open workspace remains writable after package save");
        restarted
            .accept(ConversationProposalActionRequest {
                project_id: ProjectId::from("p"),
                proposal_id: "post-package-save".into(),
                provider: Some("provider".into()),
                model: Some("model".into()),
                turn_id: Some("post-package-save-turn".into()),
            })
            .expect("accept after package save");
        drop(restarted);
        let mut reopened = ConversationService::default();
        let final_state = reopened
            .create(request)
            .expect("restart after package state save");
        assert_eq!(
            final_state.head_revision_id,
            RevisionId::from("post-package-save-r2")
        );
    }
}
