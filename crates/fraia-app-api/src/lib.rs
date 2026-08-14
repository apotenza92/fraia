use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub type AgentModelSettings = fraia_core::AgentModelSettings;
pub type UnitProfile = fraia_core::UnitProfile;
use fraia_core::serde_f64;
use fraia_revision::{ArtefactId, ConversationId, EvidenceId, ProjectId, RevisionId, SnapshotId};

pub type AgentState = fraia_core::AgentState;
pub type BaseModelBrief = fraia_core::BaseModelBrief;
pub type DesignOptionDecisionState = fraia_core::DesignOptionDecisionState;

pub type SourceId = fraia_core::SourceId;
pub type SourceRecord = fraia_core::SourceRecord;
pub type SourceDerivative = fraia_core::SourceDerivative;
pub type SourceImportJob = fraia_core::SourceImportJob;
pub type SourceMediaType = fraia_core::SourceMediaType;
pub type ShelfDocument = fraia_core::ShelfDocument;
pub type ShelfItem = fraia_core::ShelfItem;
pub type AcceptedDesignRevisionRef = fraia_core::AcceptedDesignRevisionRef;
pub type PdfDocumentIndex = fraia_core::PdfDocumentIndex;
pub type PdfViewRoleInference = fraia_core::PdfViewRoleInference;
pub type PdfDiagnostic = fraia_core::PdfDiagnostic;
pub type DrawingInterpretation = fraia_core::DrawingInterpretation;
pub type DrawingInterpretationRevision = fraia_core::DrawingInterpretationRevision;
pub type DrawingInterpretationList = fraia_core::DrawingInterpretationList;
pub type InterpretationCreateAuthority = fraia_core::InterpretationCreateAuthority;
pub type ConfirmObservationsOperation = fraia_core::ConfirmObservationsOperation;
pub type ReconcileInterpretationOperation = fraia_core::ReconcileInterpretationOperation;
pub type ResolveInterpretationConflictOperation =
    fraia_core::ResolveInterpretationConflictOperation;
pub type AgentInterpretationContext = fraia_core::AgentInterpretationContext;
pub type DesignRunList = fraia_core::DesignRunList;
pub type InspectedDesignRun = fraia_core::InspectedDesignRun;
pub type DesignRunStatusProjection = fraia_core::DesignRunStatusProjection;
pub type AnalysisExecutionStage = fraia_revision::analysis_service::AnalysisExecutionStage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisAttemptStatus {
    Running,
    Cancelling,
    Completed,
    Failed,
    Unsupported,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisAttemptStartRequest {
    pub project_id: ProjectId,
    pub request: fraia_revision::operations::OperationRequest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisAttemptIdRequest {
    pub project_id: ProjectId,
    pub attempt_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisAttemptResponse {
    pub attempt_id: String,
    pub project_id: ProjectId,
    pub revision_id: RevisionId,
    pub authored_snapshot_id: SnapshotId,
    pub evidence_id: EvidenceId,
    pub stage: AnalysisExecutionStage,
    pub status: AnalysisAttemptStatus,
    pub elapsed_millis: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<String>,
}
pub type DxfIndexResult = fraia_core::DxfIndexResult;
pub type PreparedDxfSelection = fraia_core::PreparedDxfSelection;
pub type IfcIndexResult = fraia_core::IfcIndexResult;
pub type PreparedIfcSelection = fraia_core::PreparedIfcSelection;
pub type MeshIndexResult = fraia_core::MeshIndexResult;
pub type PreparedMeshSavedView = fraia_core::PreparedMeshSavedView;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignRunListRequest {
    pub project_dir: String,
    pub design_id: fraia_core::DesignId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignRunInspectRequest {
    pub project_dir: String,
    pub design_id: fraia_core::DesignId,
    pub run_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignRunStatusRequest {
    pub project_dir: String,
    pub design_id: fraia_core::DesignId,
    pub inspected_snapshot_id: String,
    #[serde(default)]
    pub ancestor_snapshot_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DxfIndexRequest {
    pub project_dir: String,
    pub source_id: fraia_core::SourceId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DxfPrepareSelectionRequest {
    pub project_dir: String,
    pub design_id: fraia_core::DesignId,
    pub selection: fraia_core::DxfSelectionRequest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IfcIndexRequest {
    pub project_dir: String,
    pub source_id: fraia_core::SourceId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IfcPrepareSelectionRequest {
    pub project_dir: String,
    pub design_id: fraia_core::DesignId,
    pub selection: fraia_core::IfcSelectionRequest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeshIndexRequest {
    pub project_dir: String,
    pub source_id: fraia_core::SourceId,
}

pub type MeshContentRequest = MeshIndexRequest;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeshIndexJobRequest {
    pub project_dir: String,
    pub source_id: fraia_core::SourceId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeshIndexJobIdRequest {
    pub job_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeshIndexJobStatus {
    Running,
    Cancelling,
    Completed,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeshIndexJobResponse {
    pub job_id: String,
    pub status: MeshIndexJobStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<MeshIndexResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeshPrepareSavedViewRequest {
    pub project_dir: String,
    pub design_id: fraia_core::DesignId,
    pub view: fraia_core::MeshSavedViewRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfIndexRequest {
    pub project_dir: String,
    pub source_id: SourceId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfViewRoleInferenceRequest {
    pub project_dir: String,
    pub source_id: SourceId,
    pub page_number: u32,
    pub crop: fraia_core::PdfBox,
    #[serde(default = "default_pdf_inference_margin")]
    pub margin_points: f64,
}

fn default_pdf_inference_margin() -> f64 {
    36.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfIndexResponse {
    pub index: PdfDocumentIndex,
    pub index_derivative: SourceDerivative,
    pub resumed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfCapabilityResponse {
    pub parser: String,
    pub parser_version: String,
    pub metadata_indexing_available: bool,
    pub packaged_renderer_available: bool,
    pub ocr_available: bool,
    pub diagnostics: Vec<PdfDiagnostic>,
}

/// Electron main sends the result of a native file dialog once. The returned
/// token is short-lived, single-use, and bound to this project directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceSelectionIssueRequest {
    pub project_dir: String,
    pub selected_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceSelectionIssueResponse {
    pub selection_token: String,
    pub expires_in_seconds: u64,
}

/// Requests import from a file selection that Electron main has already
/// authorized. The renderer never supplies an arbitrary filesystem path.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceImportRequest {
    pub project_dir: String,
    pub selection_token: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_alias: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_media_type: Option<SourceMediaType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceImportResponse {
    pub record: SourceRecord,
    pub job: SourceImportJob,
    pub deduplicated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceListRequest {
    pub project_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceListResponse {
    pub sources: Vec<SourceRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceInspectRequest {
    pub project_dir: String,
    pub source_id: SourceId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceInspectResponse {
    pub source: SourceRecord,
    pub derivatives: Vec<SourceDerivative>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceDerivativeQueryRequest {
    pub project_dir: String,
    pub source_id: SourceId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceDerivativeQueryResponse {
    pub derivatives: Vec<SourceDerivative>,
}

/// appd must resolve all shelf and design references before it calls the core
/// removal operation. This contract intentionally has no force flag.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceRemoveRequest {
    pub project_dir: String,
    pub source_id: SourceId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceRemoveResponse {
    pub source_id: SourceId,
    pub removed_derivatives: usize,
    pub removed_files: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShelfListRequest {
    pub project_dir: String,
    pub design_id: fraia_core::DesignId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShelfUpsertRequest {
    pub project_dir: String,
    pub design_id: fraia_core::DesignId,
    pub item: ShelfItem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShelfRemoveRequest {
    pub project_dir: String,
    pub design_id: fraia_core::DesignId,
    pub item_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShelfRetargetRequest {
    pub project_dir: String,
    pub design_id: fraia_core::DesignId,
    pub item_id: String,
    pub expected: AcceptedDesignRevisionRef,
    pub replacement: AcceptedDesignRevisionRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrawingInterpretationListRequest {
    pub project_dir: String,
    pub design_id: fraia_core::DesignId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrawingInterpretationInspectRequest {
    pub project_dir: String,
    pub design_id: fraia_core::DesignId,
    pub revision_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrawingInterpretationCreateRequest {
    pub project_dir: String,
    pub design_id: fraia_core::DesignId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_parent_revision_id: Option<String>,
    pub authority: InterpretationCreateAuthority,
    pub revision: DrawingInterpretationRevision,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrawingInterpretationConfirmRequest {
    pub project_dir: String,
    pub design_id: fraia_core::DesignId,
    pub operation: ConfirmObservationsOperation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrawingInterpretationReconcileRequest {
    pub project_dir: String,
    pub design_id: fraia_core::DesignId,
    pub operation: ReconcileInterpretationOperation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrawingInterpretationResolveConflictRequest {
    pub project_dir: String,
    pub design_id: fraia_core::DesignId,
    pub operation: ResolveInterpretationConflictOperation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrawingInterpretationCorrectRequest {
    pub project_dir: String,
    pub design_id: fraia_core::DesignId,
    pub operation: fraia_core::CorrectInterpretationObservationOperation,
}

pub type AgentSession = fraia_core::AgentSession;

pub type AgentQuestion = fraia_core::AgentQuestion;

pub type AgentQuestionOption = fraia_core::AgentQuestionOption;

pub type AgentSuggestedReplyGroup = fraia_core::AgentSuggestedReplyGroup;

pub type AgentPlanItem = fraia_core::AgentPlanItem;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppHealthResponse {
    pub status: String,
    pub api_version: String,
    pub calculix_runtime: fraia_core::CalculixRuntimeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

// Conversation-first transport contracts. These deliberately sit alongside the
// staged API during the cutover; they do not reuse option or readiness types.
//
// Compatibility note: `project_id` in this transport is the
// fraia-revision scope id. Fraia stores one revision database per design, so
// current adapters pass the design id here. It is not the package-level
// fraia-core ProjectId. New public UI state carries package project, design,
// and revision-scope identities separately.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationCreateRequest {
    pub project_id: ProjectId,
    pub project_dir: String,
    pub conversation_id: ConversationId,
    pub purpose: String,
    #[serde(default)]
    pub project_facts: ConversationProjectFacts,
}
/// Typed intent available before the project has any geometry. It is not a
/// hidden model and can remain deliberately incomplete while a conversation
/// starts from an empty structural snapshot.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationProjectFacts {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub building_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approximate_length_m: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approximate_width_m: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approximate_height_m: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub objective: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub loads_and_assumptions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unknowns: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationMessageRequest {
    pub project_id: ProjectId,
    pub conversation_id: ConversationId,
    pub message: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationFactsUpdateRequest {
    pub project_id: ProjectId,
    pub conversation_id: ConversationId,
    pub project_facts: ConversationProjectFacts,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationForkRequest {
    pub project_id: ProjectId,
    pub conversation_id: ConversationId,
    pub purpose: String,
    pub from_revision_id: RevisionId,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationProposalRequest {
    pub project_id: ProjectId,
    pub conversation_id: ConversationId,
    pub proposal_id: String,
    pub proposed_revision_id: RevisionId,
    pub parent_revision_id: RevisionId,
    pub provider: String,
    pub model: String,
    pub turn_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub catalogue_refreshed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_text: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub response_questions: Vec<String>,
    /// Exact current-design evidence selected for this turn. Omission means
    /// the proposal is text-only; it never implies access to every project
    /// source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_context: Option<ConversationProposalSourceContext>,
    /// A typed patch is a batch so an empty project can receive its first
    /// coherent set of nodes, members, and supports as one accepted revision.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub operations: Vec<ConversationProposalOperation>,
    /// Compatibility for the earliest transport spike; new callers use
    /// `operations` exclusively.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation: Option<ConversationProposalOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationProposalSourceContext {
    pub design_id: fraia_core::DesignId,
    pub expected_snapshot_id: SnapshotId,
    #[serde(default)]
    pub shelf_item_ids: Vec<String>,
    #[serde(default)]
    pub assumptions: Vec<String>,
    #[serde(default)]
    pub evidence_limits: Vec<String>,
    #[serde(default)]
    pub drawing_interpretation_revision_ids: Vec<String>,
    /// Exact high-confidence, non-conflicted inference candidates reviewed as
    /// assumptions for this proposal. These ids never become confirmed facts.
    #[serde(default)]
    pub drawing_interpretation_inference_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationAgentRespondRequest {
    pub project_dir: String,
    pub package_project_id: fraia_core::ProjectId,
    pub project_id: ProjectId,
    pub design_id: fraia_core::DesignId,
    pub conversation_id: ConversationId,
    pub expected_head_revision_id: RevisionId,
    pub expected_snapshot_id: SnapshotId,
    pub text: String,
    #[serde(default)]
    pub shelf_item_ids: Vec<String>,
    #[serde(default)]
    pub drawing_interpretation_revision_ids: Vec<String>,
    pub turn_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationAgentProposalResponse {
    pub proposal_id: String,
    pub proposed_revision_id: RevisionId,
    pub parent_revision_id: RevisionId,
    #[serde(default = "pending_proposal_status")]
    pub status: String,
    pub assumptions: Vec<String>,
    pub evidence_limits: Vec<String>,
    pub operations: Vec<ConversationProposalOperation>,
}

fn pending_proposal_status() -> String {
    "pending".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationAgentRespondResponse {
    pub response_id: String,
    pub text: String,
    #[serde(default)]
    pub questions: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposal: Option<ConversationAgentProposalResponse>,
    pub provider: String,
    pub model: String,
    pub reasoning_effort: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub catalogue_refreshed_at: Option<String>,
    pub turn_id: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum ConversationProposalOperation {
    SetMemberRole {
        member_id: String,
        role: String,
    },
    /// Move a node in canonical metres. The revision engine validates the
    /// resulting model before admitting the operation.
    MoveNode {
        node_id: String,
        x: f64,
        y: f64,
        z: f64,
    },
    AddNode {
        id: String,
        x: f64,
        y: f64,
        z: f64,
    },
    AddMember {
        id: String,
        start_node: String,
        end_node: String,
        role: String,
        section_id: String,
        material_id: String,
    },
    AddSupport {
        id: String,
        target_node: String,
        ux: bool,
        uy: bool,
        uz: bool,
        rx: bool,
        ry: bool,
        rz: bool,
    },
    SetSection {
        member_id: String,
        section_id: String,
    },
    AddPlate {
        id: String,
        boundary_nodes: Vec<String>,
        role: String,
        thickness_m: f64,
        material_id: String,
        generated_from: String,
    },
    AddLoad {
        id: String,
        target_kind: String,
        target_id: String,
        load_case_id: String,
        direction_x: f64,
        direction_y: f64,
        direction_z: f64,
        magnitude: f64,
        unit: String,
    },
    AddRelease {
        id: String,
        member_id: String,
        end: String,
        ux: bool,
        uy: bool,
        uz: bool,
        rx: bool,
        ry: bool,
        rz: bool,
    },
    SetRelease {
        id: String,
        member_id: String,
        end: String,
        ux: bool,
        uy: bool,
        uz: bool,
        rx: bool,
        ry: bool,
        rz: bool,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationProposalActionRequest {
    pub project_id: ProjectId,
    pub proposal_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_id: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationAnalysisRequest {
    pub project_id: ProjectId,
    pub conversation_id: ConversationId,
    pub revision_id: RevisionId,
    pub evidence_id: EvidenceId,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationComparisonRequest {
    pub project_id: ProjectId,
    pub conversation_id: ConversationId,
    pub baseline_evidence_id: EvidenceId,
    pub candidate_evidence_id: EvidenceId,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationAnalysisComboMetrics {
    pub combo_id: String,
    pub max_utilization: f64,
    pub max_ux_m: f64,
    pub max_uy_m: f64,
    pub max_reaction_n: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationAnalysisMetrics {
    pub combo_metrics: Vec<ConversationAnalysisComboMetrics>,
    pub max_utilization: f64,
    pub max_ux_m: f64,
    pub max_uy_m: f64,
    pub max_reaction_n: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationComparisonEntry {
    pub evidence_id: EvidenceId,
    pub authored_snapshot_id: SnapshotId,
    pub resolved_snapshot_id: SnapshotId,
    pub input_identity: String,
    pub result_identity: String,
    pub metrics: ConversationAnalysisMetrics,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationComparisonResponse {
    pub solver_identity: String,
    pub runtime_identity: String,
    pub settings_identity: String,
    pub settings_payload: String,
    pub request: Value,
    pub baseline: ConversationComparisonEntry,
    pub candidate: ConversationComparisonEntry,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationStateResponse {
    pub project_id: ProjectId,
    pub conversation_id: ConversationId,
    pub purpose: String,
    pub head_revision_id: RevisionId,
    pub head_snapshot_id: SnapshotId,
    pub project_facts: ConversationProjectFacts,
    pub semantic_summary: fraia_core::ModelUnderstandingReport,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub messages: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub agent_responses: Vec<ConversationAgentRespondResponse>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationRevisionResponse {
    pub revision_id: RevisionId,
    pub snapshot_id: SnapshotId,
    pub parent_revision_id: Option<RevisionId>,
    pub author: String,
    pub agent_provenance: Option<ConversationAgentProvenance>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationAgentProvenance {
    pub provider: String,
    pub model: String,
    pub turn_id: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationEvidenceResponse {
    pub evidence_id: EvidenceId,
    pub authored_snapshot_id: SnapshotId,
    pub stale: bool,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_snapshot_id: Option<SnapshotId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub solver_identity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metrics: Option<ConversationAnalysisMetrics>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationArtefactResponse {
    pub artefact_id: ArtefactId,
    pub source_snapshot_id: SnapshotId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationWorkingCopyOpenResponse {
    pub working_copy_id: String,
    pub source_revision_id: RevisionId,
    pub source_snapshot_id: SnapshotId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationWorkingCopyOpenRequest {
    pub project_id: ProjectId,
    pub conversation_id: ConversationId,
    pub revision_id: RevisionId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationWorkingCopyOperationRequest {
    pub project_id: ProjectId,
    pub working_copy_id: String,
    pub operation: ConversationProposalOperation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationWorkingCopyCommitRequest {
    pub project_id: ProjectId,
    pub conversation_id: ConversationId,
    pub working_copy_id: String,
    pub revision_id: RevisionId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectPathRequest {
    pub project_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectRequest {
    pub project_dir: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanningProjectIntent {
    pub name: String,
    pub building_type: String,
    pub design_stage: String,
    pub objective_priority: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanningSystemBrief {
    pub system_family_hint: String,
    pub structural_form_hint: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanningGeometryAndLoads {
    #[serde(
        rename = "span",
        alias = "spanM",
        alias = "span_m",
        serialize_with = "serde_f64::serialize_length",
        deserialize_with = "serde_f64::deserialize_length"
    )]
    pub span_m: f64,
    #[serde(
        rename = "height",
        alias = "heightM",
        alias = "height_m",
        serialize_with = "serde_f64::serialize_length",
        deserialize_with = "serde_f64::deserialize_length"
    )]
    pub height_m: f64,
    #[serde(
        rename = "gravityLineLoad",
        alias = "gravityLineLoadKnPerM",
        alias = "gravity_line_load_kn_per_m",
        serialize_with = "serde_f64::serialize_kilonewtons_per_meter_as_line_load",
        deserialize_with = "serde_f64::deserialize_line_load_as_kilonewtons_per_meter"
    )]
    pub gravity_line_load_kn_per_m: f64,
    #[serde(
        rename = "lateralLoad",
        alias = "lateralLoadKn",
        alias = "lateral_load_kn",
        serialize_with = "serde_f64::serialize_kilonewtons_as_force",
        deserialize_with = "serde_f64::deserialize_force_as_kilonewtons"
    )]
    pub lateral_load_kn: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanningDesignConstraints {
    pub max_deflection_ratio: f64,
    pub max_drift_ratio: f64,
    pub max_utilization: f64,
    pub allow_internal_columns: bool,
    pub max_internal_columns: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanningAnalysisBrief {
    pub requested_analysis_intent: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferred_backend: Option<String>,
    pub summary_goals: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanningDraft {
    pub project_intent: PlanningProjectIntent,
    pub system_brief: PlanningSystemBrief,
    pub geometry_and_loads: PlanningGeometryAndLoads,
    pub design_constraints: PlanningDesignConstraints,
    pub analysis_brief: PlanningAnalysisBrief,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub system_parameters: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanningDraftRequest {
    pub project_dir: String,
    pub draft: PlanningDraft,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SummaryArtifactRef {
    pub run_id: String,
    pub summary_md: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkbenchDiagnostic {
    pub severity: String,
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisReadiness {
    pub status: String,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<WorkbenchDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisRunSummary {
    pub status: String,
    pub analysis_kind: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignOptionAnalysisScope {
    pub kind: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub option_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignOptionAnalysisRequest {
    pub project_dir: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<DesignOptionAnalysisScope>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate_policy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub check_profile: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignOptionDecisionUpdateRequest {
    pub project_dir: String,
    pub action: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub option_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub included: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkbenchProjectOverview {
    pub project_dir: String,
    pub document_id: String,
    pub name: String,
    pub building_type: String,
    pub design_stage: String,
    #[serde(
        rename = "span",
        alias = "spanM",
        alias = "span_m",
        serialize_with = "serde_f64::serialize_length",
        deserialize_with = "serde_f64::deserialize_length"
    )]
    pub span_m: f64,
    #[serde(
        rename = "height",
        alias = "heightM",
        alias = "height_m",
        serialize_with = "serde_f64::serialize_length",
        deserialize_with = "serde_f64::deserialize_length"
    )]
    pub height_m: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneBounds {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneNode {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneSizeCoordination {
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneSectionCoordination {
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneMember {
    pub id: String,
    pub start_node: String,
    pub end_node: String,
    pub role: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_section_families: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coordination_group_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coordination_group_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub family_group_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub section_coordination: Option<SceneSectionCoordination>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_group_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_coordination: Option<SceneSizeCoordination>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scheme_note: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub semantic_tags: Vec<String>,
    pub section_id: String,
    pub material_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenePlate {
    pub id: String,
    pub boundary_nodes: Vec<String>,
    pub role: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub semantic_tags: Vec<String>,
    #[serde(
        rename = "thickness",
        alias = "thicknessM",
        alias = "thickness_m",
        serialize_with = "serde_f64::serialize_length",
        deserialize_with = "serde_f64::deserialize_length"
    )]
    pub thickness_m: f64,
    pub material_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneSupport {
    pub id: String,
    pub target_node: String,
    pub ux: bool,
    pub uy: bool,
    pub uz: bool,
    pub rx: bool,
    pub ry: bool,
    pub rz: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_group_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneLoad {
    pub id: String,
    pub target_label: String,
    pub kind: String,
    pub magnitude: f64,
    pub direction_x: f64,
    pub direction_y: f64,
    pub direction_z: f64,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneRelease {
    pub id: String,
    pub member_id: String,
    pub end: String,
    pub ux: bool,
    pub uy: bool,
    pub uz: bool,
    pub rx: bool,
    pub ry: bool,
    pub rz: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkbenchScene {
    pub bounds: SceneBounds,
    pub nodes: Vec<SceneNode>,
    pub members: Vec<SceneMember>,
    pub plates: Vec<ScenePlate>,
    pub supports: Vec<SceneSupport>,
    pub loads: Vec<SceneLoad>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub releases: Vec<SceneRelease>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoordinationGroup {
    pub id: String,
    pub label: String,
    pub role: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub member_group_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub member_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_section_families: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recommended_section_families: Vec<String>,
    pub section_selection_policy: String,
    pub same_size_preferred: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rationale: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub buildability_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignSchemeSectionCandidate {
    pub section_id: String,
    pub family: String,
    pub mass_kg_per_m: f64,
    pub approximate_mass_kg: f64,
    pub relative_to_selected_kg: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analysis_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analysis_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub passes_preliminary_check: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_utilization: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_stress_mpa: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_moment_knm: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_shear_kn: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_deflection_mm: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_drift_mm: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_reaction_kn: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governing_member_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnostic: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignSchemeGroupChoice {
    pub coordination_group_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_section_families: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub candidate_section_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub candidate_sections: Vec<DesignSchemeSectionCandidate>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unavailable_families: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_section_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approximate_mass_kg: Option<f64>,
    pub check_status: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignOptionIntent {
    pub id: String,
    pub label: String,
    pub hypothesis: String,
    pub exploration_band: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lifecycle_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub superseded_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub superseded_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision_of: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub objective_tags: Vec<String>,
    pub standardisation_strategy: String,
    pub connection_strategy: String,
    pub support_strategy: String,
    pub section_family_policy: String,
    pub coordination_group_policy: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub coordination_overrides: Vec<DesignOptionCoordinationOverride>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assumptions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub provenance: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignOptionCoordinationOverride {
    pub member_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub family_group_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub designation_group_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignSchemeAnalysisSummary {
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_utilization: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_moment_knm: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_shear_kn: Option<f64>,
    pub max_stress_mpa: f64,
    pub max_deflection_mm: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_drift_mm: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_reaction_kn: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governing_member_id: Option<String>,
    pub deflected_shape_scale: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignScheme {
    pub id: String,
    pub label: String,
    pub strategy: String,
    pub summary: String,
    pub differentiation: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lifecycle_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub superseded_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub superseded_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision_of: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pros: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cons: Vec<String>,
    pub support_strategy: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub standardisation_strategy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connection_strategy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intent: Option<DesignOptionIntent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene: Option<WorkbenchScene>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub group_choices: Vec<DesignSchemeGroupChoice>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approximate_mass_kg: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analysis_summary: Option<DesignSchemeAnalysisSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_preview: Option<Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<WorkbenchDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoordinationReport {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<CoordinationGroup>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub design_schemes: Vec<DesignScheme>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_design_scheme_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<WorkbenchDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkbenchProjectState {
    pub overview: WorkbenchProjectOverview,
    #[serde(default)]
    pub unit_profile: UnitProfile,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub planning_draft: Option<PlanningDraft>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_system_family: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analysis_readiness: Option<AnalysisReadiness>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_run_summary: Option<AnalysisRunSummary>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capability_diagnostics: Vec<WorkbenchDiagnostic>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene: Option<WorkbenchScene>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub design_schemes: Vec<DesignScheme>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coordination_report: Option<CoordinationReport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_validate: Option<SummaryArtifactRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_frame_calculix: Option<SummaryArtifactRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_beam_calculix: Option<SummaryArtifactRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_beam_analysis: Option<SummaryArtifactRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_beam_sizing: Option<SummaryArtifactRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_design_option_analysis: Option<SummaryArtifactRef>,
    #[serde(default)]
    pub agent_state: AgentState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_model_brief: Option<BaseModelBrief>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_import: Option<SummaryArtifactRef>,
    #[serde(default)]
    pub design_option_decisions: DesignOptionDecisionState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkbenchOperationResponse {
    pub message: String,
    pub state: WorkbenchProjectState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSettingsUpdateRequest {
    pub project_dir: String,
    pub surface: String,
    pub provider_id: String,
    #[serde(rename = "modelId", alias = "model")]
    pub model: String,
    pub reasoning_effort: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentProviderStatusRequest {
    pub project_dir: String,
    pub surface: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentReasoningOption {
    pub effort: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentModelOption {
    pub provider_id: String,
    #[serde(rename = "modelId", alias = "slug")]
    pub slug: String,
    pub display_name: String,
    pub default_reasoning_level: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supported_reasoning_levels: Vec<AgentReasoningOption>,
    #[serde(default)]
    pub available: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_window: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentAuthenticationMethod {
    pub r#type: String,
    pub label: String,
    pub interactive: bool,
    pub persistent_allowed: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requirements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentProviderDescriptor {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authentication: Vec<AgentAuthenticationMethod>,
    pub auth_state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AgentCatalogueFreshness {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refreshed_at: Option<String>,
    #[serde(default)]
    pub stale: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_error: Option<String>,
    #[serde(default)]
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentProviderStatusResponse {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub providers: Vec<AgentProviderDescriptor>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub models: Vec<AgentModelOption>,
    pub selected_provider_id: String,
    #[serde(rename = "selectedModelId")]
    pub selected_model: String,
    pub selected_reasoning_effort: String,
    #[serde(default)]
    pub catalogue: AgentCatalogueFreshness,
    #[serde(default)]
    pub secure_credential_storage_available: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<WorkbenchDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSessionStartRequest {
    pub project_dir: String,
    pub surface: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSessionRespondRequest {
    pub project_dir: String,
    pub surface: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(default)]
    pub text: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selected_option_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSessionCancelRequest {
    pub request_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentReviewMessage {
    pub author: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentReviewReplyRequest {
    pub project_dir: String,
    pub comment_id: String,
    pub comment: Value,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selected_chips: Vec<String>,
    #[serde(default)]
    pub reply: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub messages: Vec<AgentReviewMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCoordinatorTarget {
    pub kind: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentProposedAction {
    pub action_kind: String,
    pub target_kind: String,
    pub target_id: String,
    pub field: String,
    pub value: Value,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCoordinatorRequest {
    pub project_dir: String,
    #[serde(default)]
    pub instruction: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub review_comments: Vec<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus_comment_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub focus_targets: Vec<AgentCoordinatorTarget>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub messages: Vec<AgentReviewMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCoordinatorProposal {
    pub id: String,
    pub title: String,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<AgentProposedAction>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected_targets: Vec<AgentCoordinatorTarget>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCoordinatorResponse {
    pub agent_mode: String,
    pub model: String,
    pub reasoning_effort: String,
    pub status: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub follow_up: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suggested_chips: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub proposals: Vec<AgentCoordinatorProposal>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub proposed_actions: Vec<AgentProposedAction>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected_targets: Vec<AgentCoordinatorTarget>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub readiness_delta: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<WorkbenchDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentReviewReplyResponse {
    pub agent_mode: String,
    pub model: String,
    pub reasoning_effort: String,
    pub status: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub follow_up: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suggested_chips: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub resolution_summary: String,
    #[serde(default)]
    pub interpretation: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub proposed_actions: Vec<AgentProposedAction>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<WorkbenchDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentApplyReviewRequest {
    pub project_dir: String,
    pub comment_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub proposed_actions: Vec<AgentProposedAction>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn source_import_contract_uses_authorized_selection_tokens_not_renderer_paths() {
        let request = SourceImportRequest {
            project_dir: "/projects/example".into(),
            selection_token: "selection-7".into(),
            display_alias: Some("architectural-set.pdf".into()),
            expected_media_type: Some(SourceMediaType::Pdf),
        };

        let encoded = serde_json::to_value(&request).expect("serialize source import request");
        assert_eq!(encoded["selectionToken"], "selection-7");
        assert_eq!(encoded["expectedMediaType"], "pdf");
        assert!(encoded.get("selectedPath").is_none());
        assert!(encoded.get("sourcePath").is_none());
        let decoded: SourceImportRequest =
            serde_json::from_value(encoded).expect("deserialize source import request");
        assert_eq!(decoded.selection_token, "selection-7");
    }

    #[test]
    fn source_removal_contract_has_no_renderer_controlled_force_or_reference_list() {
        let request = SourceRemoveRequest {
            project_dir: "/projects/example".into(),
            source_id: SourceId::from_sha256(&"a".repeat(64)).expect("source id"),
        };

        let encoded = serde_json::to_value(request).expect("serialize source removal request");
        assert!(encoded.get("sourceId").is_some());
        assert!(encoded.get("force").is_none());
        assert!(encoded.get("references").is_none());
    }

    #[test]
    fn pdf_index_contract_uses_source_identity_and_exposes_truthful_capabilities() {
        let request = PdfIndexRequest {
            project_dir: "/projects/example".into(),
            source_id: SourceId::from_sha256(&"b".repeat(64)).expect("source id"),
        };
        let encoded = serde_json::to_value(request).expect("serialize PDF index request");
        assert!(encoded.get("sourceId").is_some());
        assert!(encoded.get("selectedPath").is_none());

        let capabilities = PdfCapabilityResponse {
            parser: "lopdf".into(),
            parser_version: "0.44.0".into(),
            metadata_indexing_available: true,
            packaged_renderer_available: false,
            ocr_available: false,
            diagnostics: vec![fraia_core::pdf_renderer_unavailable_diagnostic()],
        };
        let encoded = serde_json::to_value(capabilities).expect("serialize capabilities");
        assert_eq!(encoded["metadataIndexingAvailable"], true);
        assert_eq!(encoded["packagedRendererAvailable"], false);
        assert_eq!(encoded["ocrAvailable"], false);
    }

    #[test]
    fn pdf_view_role_inference_contract_is_crop_bound_and_evidence_bearing() {
        let request = PdfViewRoleInferenceRequest {
            project_dir: "/projects/example".into(),
            source_id: SourceId::from_sha256(&"f".repeat(64)).expect("source id"),
            page_number: 3,
            crop: fraia_core::PdfBox {
                x0: 10.0,
                y0: 20.0,
                x1: 200.0,
                y1: 140.0,
            },
            margin_points: 36.0,
        };
        let encoded = serde_json::to_value(request).expect("serialize PDF inference request");
        assert_eq!(encoded["pageNumber"], 3);
        assert_eq!(encoded["crop"]["x0"], 10.0);
        assert!(encoded.get("pageText").is_none());
        assert!(encoded.get("claimedRole").is_none());
    }

    #[test]
    fn dxf_selection_contract_requires_exact_managed_identity_and_one_relation_confirmation() {
        let source_id = SourceId::from_sha256(&"d".repeat(64)).expect("source id");
        let request = serde_json::json!({
            "projectDir": "/projects/example",
            "designId": "design-a",
            "selection": {
                "shelf_item_id": "cad-plan-selection",
                "label": "Ground floor plan",
                "source_id": source_id,
                "layout": "Model",
                "entity_ids": ["dxf:1A", "dxf:1B"],
                "view_role": "plan",
                "relation_to_design": {
                    "confirmed": true,
                    "confirmed_by": "user-a",
                    "confirmed_at": "2026-08-14T00:00:00Z",
                    "transform": {
                        "translation": [0.0, 0.0, 0.0],
                        "rotation_degrees": [0.0, 0.0, 0.0],
                        "scale": [1.0, 1.0, 1.0]
                    },
                    "orientation": {
                        "forward": [0.0, 0.0, -1.0],
                        "up": [0.0, 1.0, 0.0]
                    },
                    "scale": 1.0
                },
                "created_at": "2026-08-14T00:00:00Z",
                "created_by": "user-a",
                "interpretation_parent_revision_id": null
            }
        });
        let decoded: DxfPrepareSelectionRequest =
            serde_json::from_value(request).expect("decode exact DXF selection request");
        assert_eq!(decoded.selection.entity_ids, ["dxf:1A", "dxf:1B"]);
        assert!(
            decoded
                .selection
                .relation_to_design
                .as_ref()
                .is_some_and(|relation| relation.confirmed)
        );

        let encoded = serde_json::to_value(decoded).expect("encode DXF selection request");
        assert!(encoded.get("selectedPath").is_none());
        assert!(encoded.get("sourcePath").is_none());
        assert!(encoded["selection"].get("member_ids").is_none());
        assert!(encoded["selection"].get("structural_model").is_none());
    }

    #[test]
    fn ifc_selection_contract_uses_stable_selectors_and_no_structural_authoring_payload() {
        let request = IfcPrepareSelectionRequest {
            project_dir: "/projects/example".into(),
            design_id: fraia_core::DesignId::new("design-a"),
            selection: fraia_core::IfcSelectionRequest {
                shelf_item_id: "ifc-level-two".into(),
                label: "Level 2 reference".into(),
                source_id: SourceId::from_sha256(&"e".repeat(64)).expect("source id"),
                view_id: "level-two".into(),
                object_ids: vec!["2Vz0ObjectGlobalId".into()],
                storey_ids: vec![42],
                grid_ids: vec![81],
                class_names: vec!["IFCBEAM".into()],
                created_at: "2026-08-14T00:00:00Z".into(),
                created_by: "user-a".into(),
                interpretation_parent_revision_id: None,
            },
        };
        let encoded = serde_json::to_value(request).expect("serialize IFC selection");
        assert_eq!(encoded["selection"]["object_ids"][0], "2Vz0ObjectGlobalId");
        assert_eq!(encoded["selection"]["storey_ids"][0], 42);
        assert!(encoded.get("selectedPath").is_none());
        assert!(encoded["selection"].get("members").is_none());
        assert!(encoded["selection"].get("structural_model").is_none());
    }

    #[test]
    fn neutral_mesh_saved_view_contract_has_managed_identity_and_no_authored_structure() {
        let request = MeshPrepareSavedViewRequest {
            project_dir: "/projects/example".into(),
            design_id: fraia_core::DesignId::new("design-a"),
            view: fraia_core::MeshSavedViewRequest {
                shelf_item_id: "saved-reference-view".into(),
                label: "Reference view".into(),
                source_id: SourceId::from_sha256(&"b".repeat(64)).expect("source id"),
                object_ids: vec!["gltf:node:0".into()],
                camera: fraia_core::ShelfCamera {
                    position: [2.0, 3.0, 4.0],
                    target: [0.0, 0.0, 0.0],
                    up: [0.0, 1.0, 0.0],
                    projection: "perspective".into(),
                },
                transform: fraia_core::ShelfTransform {
                    translation: [0.0; 3],
                    rotation_degrees: [0.0; 3],
                    scale: [1.0; 3],
                },
                orientation: fraia_core::ShelfOrientation {
                    forward: [0.0, 0.0, -1.0],
                    up: [0.0, 1.0, 0.0],
                },
                scale: 1.0,
                section_planes: vec![fraia_core::ShelfSectionPlane {
                    id: "section-a".into(),
                    normal: [1.0, 0.0, 0.0],
                    constant: -1.0,
                }],
                calibration: None,
                created_at: "2026-08-14T00:00:00Z".into(),
                created_by: "engineer".into(),
            },
        };
        let encoded = serde_json::to_value(request).expect("encode mesh saved view request");
        assert_eq!(encoded["designId"], "design-a");
        assert_eq!(encoded["view"]["object_ids"][0], "gltf:node:0");
        assert!(encoded.get("selectedPath").is_none());
        assert!(encoded["view"].get("members").is_none());
        assert!(encoded["view"].get("structural_model").is_none());
    }

    #[test]
    fn neutral_mesh_job_contract_uses_opaque_identity_and_typed_terminal_state() {
        let request = MeshIndexJobIdRequest {
            job_id: "opaque-job-id".into(),
        };
        let encoded = serde_json::to_value(request).expect("encode mesh job request");
        assert_eq!(encoded["jobId"], "opaque-job-id");
        let response = MeshIndexJobResponse {
            job_id: "opaque-job-id".into(),
            status: MeshIndexJobStatus::Cancelled,
            result: None,
            error: None,
        };
        let encoded = serde_json::to_value(response).expect("encode mesh job response");
        assert_eq!(encoded["status"], "cancelled");
        assert!(encoded.get("projectDir").is_none());
        assert!(encoded.get("sourcePath").is_none());
    }

    #[test]
    fn analysis_attempt_contract_is_separate_from_agent_turn_progress() {
        let response = AnalysisAttemptResponse {
            attempt_id: "attempt-a".into(),
            project_id: ProjectId::new("design-scope-a"),
            revision_id: RevisionId::from("revision-a"),
            authored_snapshot_id: SnapshotId::from("snapshot-a"),
            evidence_id: EvidenceId::from("evidence-a"),
            stage: AnalysisExecutionStage::Solving,
            status: AnalysisAttemptStatus::Running,
            elapsed_millis: 1250,
            canonical_run_id: None,
            diagnostics: Vec::new(),
        };
        let value = serde_json::to_value(response).expect("serialize analysis attempt");
        assert_eq!(value["stage"], "solving");
        assert_eq!(value["status"], "running");
        assert_eq!(value["elapsedMillis"], 1250);
        assert!(value.get("agentRequestId").is_none());
        assert!(value.get("turnId").is_none());
        assert!(value.get("canonicalRunId").is_none());
    }

    #[test]
    fn interpretation_create_contract_requires_exact_design_parent_and_authority() {
        let request = DrawingInterpretationCreateRequest {
            project_dir: "/projects/example".into(),
            design_id: fraia_core::DesignId::new("design-a"),
            expected_parent_revision_id: Some("drawing-interpretation-sha256-parent".into()),
            authority: InterpretationCreateAuthority::ParserAdapter,
            revision: DrawingInterpretationRevision {
                project_id: fraia_core::ProjectId::new("project-a"),
                design_id: fraia_core::DesignId::new("design-a"),
                parent_revision_id: Some("drawing-interpretation-sha256-parent".into()),
                created_at: "fixture".into(),
                method: fraia_core::InterpretationMethod::NativeVectorExtraction,
                observations: BTreeMap::new(),
                correspondences: BTreeMap::new(),
                alignment_transforms: BTreeMap::new(),
                conflicts: BTreeMap::new(),
            },
        };
        let encoded = serde_json::to_value(request).expect("serialize interpretation request");
        assert_eq!(encoded["designId"], "design-a");
        assert_eq!(
            encoded["expectedParentRevisionId"],
            "drawing-interpretation-sha256-parent"
        );
        assert_eq!(encoded["authority"], "parser_adapter");
        assert!(encoded.get("latest").is_none());
    }

    #[test]
    fn design_run_inspection_contract_requires_exact_design_and_run_identity() {
        let request = DesignRunInspectRequest {
            project_dir: "/projects/example".into(),
            design_id: fraia_core::DesignId::new("design-a"),
            run_id: format!("design-run-sha256-{}", "a".repeat(64)),
        };
        let encoded = serde_json::to_value(request).unwrap();
        assert_eq!(encoded["designId"], "design-a");
        assert!(
            encoded["runId"]
                .as_str()
                .unwrap()
                .starts_with("design-run-sha256-")
        );
        assert!(encoded.get("latest").is_none());
    }

    #[test]
    fn design_run_status_contract_is_snapshot_and_ancestry_bound() {
        let request = DesignRunStatusRequest {
            project_dir: "/managed/project".into(),
            design_id: fraia_core::DesignId::new("design-1"),
            inspected_snapshot_id: "sha256:current".into(),
            ancestor_snapshot_ids: vec!["sha256:parent".into()],
        };
        let encoded = serde_json::to_value(&request).unwrap();
        assert_eq!(encoded["designId"], "design-1");
        assert_eq!(encoded["inspectedSnapshotId"], "sha256:current");
        assert_eq!(encoded["ancestorSnapshotIds"][0], "sha256:parent");
        assert!(encoded.get("latest").is_none());
    }

    #[test]
    fn workbench_state_round_trips_design_schemes() {
        let state = WorkbenchProjectState {
            overview: WorkbenchProjectOverview {
                project_dir: "/tmp/fraia".into(),
                document_id: "fixture-document".into(),
                name: "Fraia".into(),
                building_type: "test".into(),
                design_stage: "concept".into(),
                span_m: 0.0,
                height_m: 0.0,
            },
            unit_profile: UnitProfile::default(),
            planning_draft: None,
            active_system_family: None,
            analysis_readiness: None,
            latest_run_summary: None,
            capability_diagnostics: Vec::new(),
            scene: None,
            design_schemes: Vec::new(),
            coordination_report: None,
            latest_validate: None,
            latest_frame_calculix: None,
            latest_beam_calculix: None,
            latest_beam_analysis: None,
            latest_beam_sizing: None,
            latest_design_option_analysis: None,
            agent_state: AgentState::default(),
            base_model_brief: None,
            latest_import: None,
            design_option_decisions: DesignOptionDecisionState::default(),
        };

        let encoded = serde_json::to_string(&state).expect("serialize workbench state");
        let decoded: WorkbenchProjectState =
            serde_json::from_str(&encoded).expect("deserialize workbench state");

        assert!(decoded.design_schemes.is_empty());
    }

    #[test]
    fn design_scheme_serialization_exposes_selected_sections_and_mass() {
        let scheme = DesignScheme {
            id: "scheme-a".into(),
            label: "Scheme A".into(),
            strategy: "family-only".into(),
            summary: "Compare section-family constraints.".into(),
            differentiation: "Limits member families for this comparison.".into(),
            lifecycle_status: Some("active".into()),
            superseded_by: None,
            superseded_reason: None,
            revision_of: None,
            pros: vec!["Keeps the comparison focused.".into()],
            cons: vec!["Does not settle exact member sizes.".into()],
            support_strategy: "Use authored supports.".into(),
            standardisation_strategy: Some("Compare section-family constraints only.".into()),
            connection_strategy: Some(
                "No connection families are selected by this API record.".into(),
            ),
            intent: None,
            scene: None,
            group_choices: vec![DesignSchemeGroupChoice {
                coordination_group_id: "rafters".into(),
                allowed_section_families: vec!["UB".into(), "PFC".into()],
                candidate_section_ids: vec!["250UB".into()],
                candidate_sections: vec![DesignSchemeSectionCandidate {
                    section_id: "250UB".into(),
                    family: "UB".into(),
                    mass_kg_per_m: 31.4,
                    approximate_mass_kg: 100.0,
                    relative_to_selected_kg: 0.0,
                    analysis_status: Some("passes_preliminary_stress_screen".into()),
                    analysis_run_id: Some("design-option-analysis-test".into()),
                    passes_preliminary_check: Some(true),
                    max_utilization: Some(0.62),
                    max_stress_mpa: Some(155.0),
                    max_moment_knm: Some(85.0),
                    max_shear_kn: Some(40.0),
                    max_deflection_mm: Some(12.0),
                    max_drift_mm: Some(4.0),
                    max_reaction_kn: Some(80.0),
                    governing_member_id: Some("M2".into()),
                    diagnostic: None,
                }],
                unavailable_families: Vec::new(),
                selected_section_id: Some("250UB".into()),
                approximate_mass_kg: Some(100.0),
                check_status: "family_constraints".into(),
                notes: Vec::new(),
            }],
            approximate_mass_kg: Some(100.0),
            analysis_summary: Some(DesignSchemeAnalysisSummary {
                status: "preliminary".into(),
                max_utilization: Some(0.62),
                max_moment_knm: Some(42.0),
                max_shear_kn: Some(34.0),
                max_stress_mpa: 180.0,
                max_deflection_mm: 24.0,
                max_drift_mm: Some(3.0),
                max_reaction_kn: Some(80.0),
                governing_member_id: Some("M2".into()),
                deflected_shape_scale: 1.0,
            }),
            result_preview: None,
            diagnostics: Vec::new(),
        };

        let encoded = serde_json::to_string(&scheme).expect("serialize scheme");
        assert!(encoded.contains("allowedSectionFamilies"));
        assert!(encoded.contains("candidateSectionIds"));
        assert!(encoded.contains("candidateSections"));
        assert!(encoded.contains("analysisStatus"));
        assert!(encoded.contains("passesPreliminaryCheck"));
        assert!(encoded.contains("selectedSectionId"));
        assert!(encoded.contains("approximateMassKg"));
        assert!(encoded.contains("maxMomentKnm"));
        assert!(encoded.contains("maxStressMpa"));
        assert!(encoded.contains("maxDeflectionMm"));
        assert!(encoded.contains("250UB"));
    }
}
#[test]
fn drawing_reconcile_request_has_one_golden_wire_shape() {
    let request = DrawingInterpretationReconcileRequest {
        project_dir: "/project".into(),
        design_id: fraia_core::DesignId::new("design-a"),
        operation: fraia_core::ReconcileInterpretationOperation {
            expected_parent_revision_id: "revision-a".into(),
            design_geometries: BTreeMap::from([(
                "observation-a".into(),
                fraia_core::ObservationDesignGeometry::Point {
                    coordinate: [2.0, 1.0, 0.0],
                    alignment_transform_id: "alignment-a".into(),
                },
            )]),
            correspondences: BTreeMap::from([(
                "correspondence-a".into(),
                fraia_core::CrossViewCorrespondence {
                    id: "correspondence-a".into(),
                    observation_ids: vec!["observation-a".into(), "observation-b".into()],
                    relation: fraia_core::CorrespondenceRelation::SameAxis,
                    confidence: 1.0,
                    confirmation: fraia_core::ObservationConfirmation::Confirmed {
                        confirmed_by: "user".into(),
                        confirmed_at: "2026-08-14T00:00:00Z".into(),
                    },
                    uncertainty: Vec::new(),
                },
            )]),
            alignment_transforms: BTreeMap::from([(
                "alignment-a".into(),
                fraia_core::ConfirmedAlignmentTransform {
                    id: "alignment-a".into(),
                    from_shelf_item_id: "reference-a".into(),
                    to_design_coordinate_space: "fraia_design_m".into(),
                    matrix: [
                        1.0, 0.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0,
                        1.0,
                    ],
                    established_by_correspondence_ids: vec!["correspondence-a".into()],
                    confirmed_by: "user".into(),
                    confirmed_at: "2026-08-14T00:00:00Z".into(),
                },
            )]),
            conflicts: BTreeMap::new(),
            created_at: "2026-08-14T00:00:00Z".into(),
        },
    };
    let encoded = serde_json::to_value(&request).unwrap();
    assert_eq!(
        encoded,
        serde_json::json!({
          "projectDir":"/project","designId":"design-a","operation":{"expectedParentRevisionId":"revision-a","designGeometries":{"observation-a":{"designGeometryKind":"point","coordinate":[2.0,1.0,0.0],"alignment_transform_id":"alignment-a"}},"correspondences":{"correspondence-a":{"id":"correspondence-a","observationIds":["observation-a","observation-b"],"relation":"same_axis","confidence":1.0,"confirmation":{"status":"confirmed","confirmed_by":"user","confirmed_at":"2026-08-14T00:00:00Z"}}},"alignmentTransforms":{"alignment-a":{"id":"alignment-a","fromShelfItemId":"reference-a","toDesignCoordinateSpace":"fraia_design_m","matrix":[1.0,0.0,0.0,2.0,0.0,1.0,0.0,1.0,0.0,0.0,1.0,0.0,0.0,0.0,0.0,1.0],"establishedByCorrespondenceIds":["correspondence-a"],"confirmedBy":"user","confirmedAt":"2026-08-14T00:00:00Z"}},"conflicts":{},"createdAt":"2026-08-14T00:00:00Z"}
        })
    );
    serde_json::from_value::<DrawingInterpretationReconcileRequest>(encoded).unwrap();
}
