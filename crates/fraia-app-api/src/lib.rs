use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub type AgentModelSettings = fraia_core::AgentModelSettings;
pub type UnitProfile = fraia_core::UnitProfile;
use fraia_core::serde_f64;

pub type AgentState = fraia_core::AgentState;
pub type BaseModelBrief = fraia_core::BaseModelBrief;
pub type DesignOptionDecisionState = fraia_core::DesignOptionDecisionState;

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

    #[test]
    fn workbench_state_round_trips_design_schemes() {
        let state = WorkbenchProjectState {
            overview: WorkbenchProjectOverview {
                project_dir: "/tmp/fraia".into(),
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
