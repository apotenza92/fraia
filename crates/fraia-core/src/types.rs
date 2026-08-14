use crate::structural_app::StructuralModel;
use crate::units::{UnitProfile, serde_f64};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    pub id: String,
    pub name: String,
    #[serde(
        rename = "elasticModulus",
        alias = "E",
        serialize_with = "serde_f64::serialize_stress",
        deserialize_with = "serde_f64::deserialize_stress"
    )]
    pub e: f64,
    #[serde(
        serialize_with = "serde_f64::serialize_stress",
        deserialize_with = "serde_f64::deserialize_stress"
    )]
    pub fy: f64,
    #[serde(
        serialize_with = "serde_f64::serialize_density",
        deserialize_with = "serde_f64::deserialize_density"
    )]
    pub density: f64,
    pub cost_per_kg: f64,
    pub carbon_per_kg: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub id: String,
    pub name: String,
    #[serde(
        serialize_with = "serde_f64::serialize_area",
        deserialize_with = "serde_f64::deserialize_area"
    )]
    pub area: f64,
    #[serde(
        rename = "secondMomentArea",
        alias = "I",
        serialize_with = "serde_f64::serialize_second_moment_area",
        deserialize_with = "serde_f64::deserialize_second_moment_area"
    )]
    pub i: f64,
    #[serde(
        serialize_with = "serde_f64::serialize_length",
        deserialize_with = "serde_f64::deserialize_length"
    )]
    pub depth: f64,
    #[serde(
        rename = "massPerLength",
        alias = "mass_kg_per_m",
        serialize_with = "serde_f64::serialize_mass_per_length",
        deserialize_with = "serde_f64::deserialize_mass_per_length"
    )]
    pub mass_kg_per_m: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPermissions {
    pub resize_sections: bool,
    pub add_internal_columns: bool,
    pub change_topology: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    pub building_type: String,
    pub design_stage: String,
    pub objective_priority: String,
    pub option_count: usize,
    pub hard_constraints: Vec<String>,
    pub soft_preferences: Vec<String>,
    pub search_permissions: SearchPermissions,
    pub approval_triggers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirements {
    #[serde(
        rename = "span",
        alias = "span_m",
        serialize_with = "serde_f64::serialize_length",
        deserialize_with = "serde_f64::deserialize_length"
    )]
    pub span_m: f64,
    #[serde(
        rename = "height",
        alias = "height_m",
        serialize_with = "serde_f64::serialize_length",
        deserialize_with = "serde_f64::deserialize_length"
    )]
    pub height_m: f64,
    #[serde(
        rename = "gravityLoad",
        alias = "gravity_load_kn_per_m",
        serialize_with = "serde_f64::serialize_kilonewtons_per_meter_as_line_load",
        deserialize_with = "serde_f64::deserialize_line_load_as_kilonewtons_per_meter"
    )]
    pub gravity_load_kn_per_m: f64,
    #[serde(
        rename = "lateralLoad",
        alias = "lateral_load_kn",
        serialize_with = "serde_f64::serialize_kilonewtons_as_force",
        deserialize_with = "serde_f64::deserialize_force_as_kilonewtons"
    )]
    pub lateral_load_kn: f64,
    pub max_deflection_ratio: f64,
    pub max_drift_ratio: f64,
    pub max_utilization: f64,
    pub max_internal_columns: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningProjectIntent {
    pub name: String,
    pub building_type: String,
    pub design_stage: String,
    pub objective_priority: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningSystemBrief {
    pub system_family_hint: String,
    pub structural_form_hint: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningGeometryAndLoads {
    #[serde(
        rename = "span",
        alias = "span_m",
        serialize_with = "serde_f64::serialize_length",
        deserialize_with = "serde_f64::deserialize_length"
    )]
    pub span_m: f64,
    #[serde(
        rename = "height",
        alias = "height_m",
        serialize_with = "serde_f64::serialize_length",
        deserialize_with = "serde_f64::deserialize_length"
    )]
    pub height_m: f64,
    #[serde(
        rename = "gravityLineLoad",
        alias = "gravity_line_load_kn_per_m",
        serialize_with = "serde_f64::serialize_kilonewtons_per_meter_as_line_load",
        deserialize_with = "serde_f64::deserialize_line_load_as_kilonewtons_per_meter"
    )]
    pub gravity_line_load_kn_per_m: f64,
    #[serde(
        rename = "lateralLoad",
        alias = "lateral_load_kn",
        serialize_with = "serde_f64::serialize_kilonewtons_as_force",
        deserialize_with = "serde_f64::deserialize_force_as_kilonewtons"
    )]
    pub lateral_load_kn: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningDesignConstraints {
    pub max_deflection_ratio: f64,
    pub max_drift_ratio: f64,
    pub max_utilization: f64,
    pub allow_internal_columns: bool,
    pub max_internal_columns: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningAnalysisBrief {
    pub requested_analysis_intent: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferred_backend: Option<String>,
    pub summary_goals: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub struct ProjectFiles {
    pub planning: String,
}

/// Stable opaque identity for a Fraia project container.
///
/// Display names and filesystem locations can change without changing this
/// value. Package validation rejects empty or path-like identities before any
/// path is derived from them.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProjectId(String);

impl ProjectId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ProjectId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Stable opaque identity for one design inside a project.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DesignId(String);

impl DesignId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for DesignId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectManifestFiles {
    pub planning: String,
    pub sources: String,
    pub designs: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectDesignEntry {
    pub id: DesignId,
    pub name: String,
}

/// Versioned project-container metadata. Authored engineering state belongs in
/// the referenced design manifests and revision repositories, not here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectManifest {
    pub schema_version: String,
    pub id: ProjectId,
    pub name: String,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    pub files: ProjectManifestFiles,
    pub designs: Vec<ProjectDesignEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignManifestFiles {
    /// Transitional mutable app state. This keeps the legacy `ProjectFile`
    /// model design-local until revision-native authored state replaces it.
    pub state: String,
    pub planning: String,
    pub shelf: String,
    pub workspace: String,
    pub runs: String,
    /// Exact pre-package project input retained for migration recovery and
    /// compatibility. New designs do not have this file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_project: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegacyProjectMigration {
    pub source_schema_version: String,
    pub archive: String,
    pub source_sha256: String,
    pub migrated_at: String,
}

/// Versioned identity and file ownership for one arbitrary-size design.
/// Canonical authored snapshots remain in the Rust-owned revision repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignManifest {
    pub schema_version: String,
    pub id: DesignId,
    pub name: String,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    pub files: DesignManifestFiles,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_migration: Option<LegacyProjectMigration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuilderArchetypeInstance {
    pub id: String,
    pub archetype_id: String,
    pub topology_id: String,
    pub beam_section: String,
    pub column_section: String,
    #[serde(
        rename = "span",
        alias = "span_m",
        serialize_with = "serde_f64::serialize_length",
        deserialize_with = "serde_f64::deserialize_length"
    )]
    pub span_m: f64,
    #[serde(
        rename = "height",
        alias = "height_m",
        serialize_with = "serde_f64::serialize_length",
        deserialize_with = "serde_f64::deserialize_length"
    )]
    pub height_m: f64,
    #[serde(
        rename = "gravityLoad",
        alias = "gravity_load_kn_per_m",
        serialize_with = "serde_f64::serialize_kilonewtons_per_meter_as_line_load",
        deserialize_with = "serde_f64::deserialize_line_load_as_kilonewtons_per_meter"
    )]
    pub gravity_load_kn_per_m: f64,
    #[serde(
        rename = "lateralLoad",
        alias = "lateral_load_kn",
        serialize_with = "serde_f64::serialize_kilonewtons_as_force",
        deserialize_with = "serde_f64::deserialize_force_as_kilonewtons"
    )]
    pub lateral_load_kn: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_option_index: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchetypeDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub scale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BuilderNodeStatus {
    Proposed,
    Materialized,
    DivergedFromMaterialization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortalFrame2DBuilderParams {
    pub topology_id: String,
    pub beam_section: String,
    pub column_section: String,
    #[serde(
        rename = "span",
        alias = "span_m",
        serialize_with = "serde_f64::serialize_length",
        deserialize_with = "serde_f64::deserialize_length"
    )]
    pub span_m: f64,
    #[serde(
        rename = "height",
        alias = "height_m",
        serialize_with = "serde_f64::serialize_length",
        deserialize_with = "serde_f64::deserialize_length"
    )]
    pub height_m: f64,
    #[serde(
        rename = "gravityLoad",
        alias = "gravity_load_kn_per_m",
        serialize_with = "serde_f64::serialize_kilonewtons_per_meter_as_line_load",
        deserialize_with = "serde_f64::deserialize_line_load_as_kilonewtons_per_meter"
    )]
    pub gravity_load_kn_per_m: f64,
    #[serde(
        rename = "lateralLoad",
        alias = "lateral_load_kn",
        serialize_with = "serde_f64::serialize_kilonewtons_as_force",
        deserialize_with = "serde_f64::deserialize_force_as_kilonewtons"
    )]
    pub lateral_load_kn: f64,
    #[serde(default)]
    pub origin_x_m: f64,
    #[serde(default)]
    pub origin_y_m: f64,
    #[serde(default)]
    pub origin_z_m: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplySupportedBeam2DBuilderParams {
    pub section: String,
    #[serde(
        rename = "span",
        alias = "span_m",
        serialize_with = "serde_f64::serialize_length",
        deserialize_with = "serde_f64::deserialize_length"
    )]
    pub span_m: f64,
    #[serde(
        default,
        rename = "distributedLoad",
        alias = "distributed_load_kn_per_m",
        serialize_with = "serde_f64::serialize_kilonewtons_per_meter_as_line_load",
        deserialize_with = "serde_f64::deserialize_line_load_as_kilonewtons_per_meter"
    )]
    pub distributed_load_kn_per_m: f64,
    #[serde(
        default,
        rename = "pointLoad",
        alias = "point_load_kn",
        skip_serializing_if = "Option::is_none"
    )]
    pub point_load_kn: Option<f64>,
    #[serde(
        default,
        rename = "pointLoadX",
        alias = "point_load_x_m",
        skip_serializing_if = "Option::is_none"
    )]
    pub point_load_x_m: Option<f64>,
    #[serde(default)]
    pub origin_x_m: f64,
    #[serde(default)]
    pub origin_y_m: f64,
    #[serde(default)]
    pub origin_z_m: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum BuilderNodeParameters {
    ConceptRoot,
    PortalFrame2D(PortalFrame2DBuilderParams),
    SimplySupportedBeam2D(SimplySupportedBeam2DBuilderParams),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuilderNode {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub archetype_id: String,
    pub parameters: BuilderNodeParameters,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub child_node_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_option_index: Option<usize>,
    pub status: BuilderNodeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuilderGraph {
    pub root_node_ids: Vec<String>,
    pub nodes: Vec<BuilderNode>,
}

pub const FRAIA_AI_PROVIDER_ID: &str = "openai-codex";
pub const FRAIA_AI_MODEL_ID: &str = "gpt-5.6-luna";
pub const FRAIA_AI_REASONING_EFFORT: &str = "high";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentModelSettings {
    #[serde(default = "default_agent_provider_id")]
    pub provider_id: String,
    #[serde(rename = "modelId", alias = "model")]
    pub model: String,
    pub reasoning_effort: String,
}

fn default_agent_provider_id() -> String {
    FRAIA_AI_PROVIDER_ID.into()
}

impl Default for AgentModelSettings {
    fn default() -> Self {
        Self {
            provider_id: default_agent_provider_id(),
            model: FRAIA_AI_MODEL_ID.into(),
            reasoning_effort: FRAIA_AI_REASONING_EFFORT.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentTarget {
    pub kind: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentProposedActionState {
    pub action_kind: String,
    pub target_kind: String,
    pub target_id: String,
    pub field: String,
    pub value: Value,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentQuestionOption {
    pub id: String,
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentQuestion {
    pub id: String,
    pub prompt: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<AgentQuestionOption>,
    #[serde(default)]
    pub allows_free_text: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSuggestedReplyGroup {
    pub title: String,
    pub prompt: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub replies: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub default_replies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentMessage {
    pub author: String,
    pub text: String,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub catalogue_refreshed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suggested_replies: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suggested_reply_groups: Vec<AgentSuggestedReplyGroup>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub proposed_actions: Vec<AgentProposedActionState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentPlanItem {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected_targets: Vec<AgentTarget>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub proposed_actions: Vec<AgentProposedActionState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSession {
    pub id: String,
    pub surface: String,
    pub title: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub messages: Vec<AgentMessage>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub plan_items: Vec<AgentPlanItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_question: Option<AgentQuestion>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentState {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sessions: Vec<AgentSession>,
    #[serde(default)]
    pub settings_by_surface: BTreeMap<String, AgentModelSettings>,
}

impl Default for AgentState {
    fn default() -> Self {
        let mut settings_by_surface = BTreeMap::new();
        settings_by_surface.insert("default".into(), AgentModelSettings::default());
        settings_by_surface.insert("pre_solve".into(), AgentModelSettings::default());
        settings_by_surface.insert("comment_review".into(), AgentModelSettings::default());
        Self {
            sessions: Vec::new(),
            settings_by_surface,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BaseModelBriefReadiness {
    pub ready_for_schemas: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unresolved_topics: Vec<String>,
    pub manual_override_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BaseModelBriefVisualIntent {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub support_locations: Vec<BaseModelBriefSupportLocationIntent>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub loads: Vec<BaseModelBriefLoadIntent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseModelBriefSupportLocationIntent {
    pub id: String,
    pub target_node: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseModelBriefLoadIntent {
    pub id: String,
    pub kind: String,
    pub target: BaseModelBriefLoadTarget,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub magnitude_n: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub magnitude_n_per_m: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direction: Option<BaseModelBriefLoadDirection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseModelBriefLoadTarget {
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseModelBriefLoadDirection {
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_node: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_node: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub z: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseModelBrief {
    pub version: u32,
    pub session_id: String,
    pub current_understanding: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub confirmed_intent: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub open_questions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub soft_assumptions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub schema_guidance: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub do_not_decide_yet: Vec<String>,
    #[serde(default, skip_serializing_if = "is_default_visual_intent")]
    pub visual_intent: BaseModelBriefVisualIntent,
    pub readiness: BaseModelBriefReadiness,
    pub updated_at: String,
}

fn is_default_visual_intent(intent: &BaseModelBriefVisualIntent) -> bool {
    intent.support_locations.is_empty() && intent.loads.is_empty()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignOptionDecisionState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_batch_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub batches: Vec<DesignOptionBatch>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_development_path_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub development_paths: Vec<DevelopmentPath>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignOptionBatch {
    pub id: String,
    pub generated_at: String,
    pub base_model_fingerprint: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ai_provenance: Option<AiProvenance>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub option_revisions: Vec<DesignOptionRevision>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub comparison_runs: Vec<DesignOptionComparisonRun>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignOptionRevision {
    #[serde(default)]
    pub revision_id: String,
    pub option_id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision_of: Option<String>,
    pub included: bool,
    pub analysis_status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_analysis_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ai_provenance: Option<AiProvenance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiProvenance {
    pub provider_id: String,
    pub model_id: String,
    pub reasoning_effort: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub catalogue_refreshed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignOptionComparisonRun {
    pub run_id: String,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub option_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_references: Vec<DesignOptionComparisonEvidenceReference>,
    pub objective: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recommended_option_id: Option<String>,
    pub explanation: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignOptionComparisonEvidenceReference {
    pub option_revision_id: String,
    pub analysis_run_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevelopmentPath {
    pub id: String,
    pub option_id: String,
    pub option_revision_id: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_analysis_run_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFile {
    pub schema_version: String,
    pub name: String,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    pub intent: Intent,
    pub requirements: Requirements,
    #[serde(default)]
    pub unit_profile: UnitProfile,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub planning_draft: Option<PlanningDraft>,
    pub files: ProjectFiles,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub builder_graph: Option<BuilderGraph>,
    #[serde(default, rename = "builder_instance", skip_serializing)]
    pub legacy_builder_instance: Option<BuilderArchetypeInstance>,
    #[serde(default)]
    pub agent_state: AgentState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_model_brief: Option<BaseModelBrief>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub structural_model: Option<StructuralModel>,
    #[serde(default)]
    pub design_option_decisions: DesignOptionDecisionState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node2D {
    pub id: String,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Support2D {
    pub node: String,
    pub ux: bool,
    pub uy: bool,
    pub rz: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodalLoad2D {
    pub node: String,
    /// Canonical backend force in SI units (N).
    #[serde(default)]
    pub fx: f64,
    /// Canonical backend force in SI units (N).
    #[serde(default)]
    pub fy: f64,
    /// Canonical backend moment in SI units (N*m).
    #[serde(default)]
    pub mz: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadCase2D {
    pub id: String,
    pub nodal_loads: Vec<NodalLoad2D>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Combo2D {
    pub id: String,
    pub factors: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topology {
    pub id: String,
    pub name: String,
    pub internal_columns: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameElement2D {
    pub id: String,
    pub i: String,
    pub j: String,
    pub role: String,
    pub section: Section,
    pub material: Material,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameModel2D {
    pub model_type: String,
    pub topology: Topology,
    pub nodes: Vec<Node2D>,
    pub elements: Vec<FrameElement2D>,
    pub supports: Vec<Support2D>,
    pub load_cases: Vec<LoadCase2D>,
    pub combos: Vec<Combo2D>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeResult2D {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub ux_m: f64,
    pub uy_m: f64,
    pub rz_rad: f64,
    pub rxn_fx_n: f64,
    pub rxn_fy_n: f64,
    pub rxn_mz_nm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementResult2D {
    pub id: String,
    pub role: String,
    pub length_m: f64,
    pub local_end_forces: Vec<f64>,
    pub axial_n: f64,
    pub moment_nm: f64,
    pub utilization: f64,
    pub stress_pa: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolveMetrics2D {
    pub max_utilization: f64,
    pub max_ux_m: f64,
    pub max_uy_m: f64,
    pub max_reaction_n: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolveResult2D {
    pub combo: Combo2D,
    pub node_results: Vec<NodeResult2D>,
    pub element_results: Vec<ElementResult2D>,
    pub metrics: SolveMetrics2D,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateOption {
    pub feasible: bool,
    pub topology: String,
    pub topology_id: String,
    pub internal_columns: usize,
    pub beam_section: String,
    pub column_section: String,
    pub mass_kg: f64,
    pub cost: f64,
    pub carbon: f64,
    pub max_utilization: f64,
    pub max_deflection_mm: f64,
    pub max_drift_mm: f64,
    pub deflection_ratio: Option<u64>,
    pub drift_ratio: Option<u64>,
    pub score: f64,
    pub summary: String,
    pub tradeoffs: Vec<String>,
    pub combo_metrics: BTreeMap<String, SolveMetrics2D>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRun {
    pub run_id: String,
    pub generated_at: String,
    pub project_name: String,
    pub project_intent: Intent,
    pub requirements: Requirements,
    pub option_count: usize,
    pub options: Vec<CandidateOption>,
}
