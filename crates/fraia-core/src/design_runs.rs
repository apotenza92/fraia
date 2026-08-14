use crate::{DesignId, ProjectId, design_package_paths, load_project_package};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub const DESIGN_RUN_SCHEMA_VERSION: &str = "fraia.design-run.v1";
pub const DESIGN_RUN_INDEX_SCHEMA_VERSION: &str = "fraia.design-run-index.v1";
pub const DESIGN_RUN_IDENTITY_VERSION: &str = "fraia.design-run-identity.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesignRunStatus {
    Completed,
    Failed,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignRunActor {
    pub actor_type: String,
    pub actor_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesignRunAttachmentRole {
    AuthoredSnapshot,
    ResolvedSnapshot,
    SolverInput,
    Result,
    Diagnostic,
    Log,
    Report,
    DesignActions,
    CheckInputs,
    CheckResults,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignRunAttachment {
    pub name: String,
    pub role: DesignRunAttachmentRole,
    pub media_type: String,
    pub sha256: String,
    pub byte_size: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignRunManifest {
    pub schema_version: String,
    pub run_id: String,
    pub project_id: ProjectId,
    pub design_id: DesignId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_run_id: Option<String>,
    pub created_at: String,
    pub actor: DesignRunActor,
    pub run_kind: String,
    pub authored_revision_id: String,
    pub authored_snapshot_id: String,
    #[serde(
        default,
        skip_serializing_if = "DesignRunInterpretationDependencies::is_empty"
    )]
    pub interpretation_dependencies: DesignRunInterpretationDependencies,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_snapshot_id: Option<String>,
    pub request_identity: String,
    pub request: Value,
    pub settings_identity: String,
    pub settings: Value,
    pub solver_identity: String,
    pub runtime_identity: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_identity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_identity: Option<String>,
    pub status: DesignRunStatus,
    #[serde(default)]
    pub diagnostics: Vec<DesignRunDiagnostic>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metrics: Option<Value>,
    #[serde(default)]
    pub attachments: Vec<DesignRunAttachment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignRunDiagnostic {
    pub severity: DesignRunDiagnosticSeverity,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesignRunDiagnosticSeverity {
    Information,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DesignRunAttachmentInput {
    pub name: String,
    pub role: DesignRunAttachmentRole,
    pub media_type: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PublishDesignRunRequest {
    pub project_id: ProjectId,
    pub design_id: DesignId,
    pub parent_run_id: Option<String>,
    pub created_at: String,
    pub actor: DesignRunActor,
    pub run_kind: String,
    pub authored_revision_id: String,
    pub authored_snapshot_id: String,
    pub resolved_snapshot_id: Option<String>,
    pub request: Value,
    pub settings: Value,
    pub solver_identity: String,
    pub runtime_identity: String,
    pub input_identity: Option<String>,
    pub result_identity: Option<String>,
    pub status: DesignRunStatus,
    pub diagnostics: Vec<DesignRunDiagnostic>,
    pub metrics: Option<Value>,
    pub attachments: Vec<DesignRunAttachmentInput>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignRunInterpretationDependencies {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub revision_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inference_ids: Vec<String>,
}

impl DesignRunInterpretationDependencies {
    pub fn is_empty(&self) -> bool {
        self.revision_ids.is_empty() && self.inference_ids.is_empty()
    }

    fn normalise(mut self) -> Self {
        self.revision_ids.sort();
        self.revision_ids.dedup();
        self.inference_ids.sort();
        self.inference_ids.dedup();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignRunSummary {
    pub run_id: String,
    pub run_kind: String,
    pub created_at: String,
    pub status: DesignRunStatus,
    pub authored_revision_id: String,
    pub authored_snapshot_id: String,
    #[serde(
        default,
        skip_serializing_if = "DesignRunInterpretationDependencies::is_empty"
    )]
    pub interpretation_dependencies: DesignRunInterpretationDependencies,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_run_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyDesignRunSummary {
    pub directory_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignRunList {
    pub project_id: ProjectId,
    pub design_id: DesignId,
    pub runs: Vec<DesignRunSummary>,
    pub legacy_runs: Vec<LegacyDesignRunSummary>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "format", rename_all = "snake_case")]
pub enum InspectedDesignRun {
    Canonical {
        manifest: Box<DesignRunManifest>,
    },
    Legacy {
        directory_name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        run_json: Option<Value>,
        files: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesignRunStaleness {
    Current,
    StaleDescendant,
    StaleDependency,
    Unrelated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignRunStalenessReason {
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interpretation_revision_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inference_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_interpretation_revision_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignRunStatusProjection {
    pub run_id: String,
    pub status: DesignRunStatus,
    pub staleness: DesignRunStaleness,
    #[serde(default)]
    pub staleness_reasons: Vec<DesignRunStalenessReason>,
    #[serde(
        default,
        skip_serializing_if = "DesignRunInterpretationDependencies::is_empty"
    )]
    pub interpretation_dependencies: DesignRunInterpretationDependencies,
    pub authored_revision_id: String,
    pub authored_snapshot_id: String,
    pub resolved_snapshot_id: Option<String>,
    pub solver_identity: String,
    pub runtime_identity: String,
    pub settings_identity: String,
    pub diagnostics: Vec<DesignRunDiagnostic>,
}

#[derive(Debug)]
pub enum DesignRunStoreError {
    Invalid(String),
    NotFound(String),
    Io(std::io::Error),
    Json(serde_json::Error),
    Package(String),
}

impl std::fmt::Display for DesignRunStoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(message) | Self::Package(message) => formatter.write_str(message),
            Self::NotFound(id) => write!(formatter, "design run `{id}` was not found"),
            Self::Io(error) => write!(formatter, "{error}"),
            Self::Json(error) => write!(formatter, "{error}"),
        }
    }
}
impl std::error::Error for DesignRunStoreError {}
impl From<std::io::Error> for DesignRunStoreError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
impl From<serde_json::Error> for DesignRunStoreError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DesignRunIndex {
    schema_version: String,
    project_id: ProjectId,
    design_id: DesignId,
    runs: BTreeMap<String, DesignRunSummary>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PublicationCheckpoint {
    StageValidated,
    RunAdopted,
    IndexBackedUp,
}

pub fn publish_design_run(
    project_dir: &Path,
    request: PublishDesignRunRequest,
) -> Result<DesignRunManifest, DesignRunStoreError> {
    publish_design_run_with_hook(project_dir, request, |_| Ok(()))
}

fn publish_design_run_with_hook<F>(
    project_dir: &Path,
    request: PublishDesignRunRequest,
    mut hook: F,
) -> Result<DesignRunManifest, DesignRunStoreError>
where
    F: FnMut(PublicationCheckpoint) -> Result<(), DesignRunStoreError>,
{
    let paths = owning_design(project_dir, &request.project_id, &request.design_id)?;
    fs::create_dir_all(&paths.runs_dir)?;
    reject_symlink(&paths.runs_dir)?;
    let (manifest, attachment_inputs) = manifest_from_request(request)?;
    validate_manifest(&manifest)?;
    let current_index = load_index(
        &paths.runs_dir,
        manifest.project_id.clone(),
        manifest.design_id.clone(),
    )?;
    if let Some(parent) = &manifest.parent_run_id
        && !current_index.runs.contains_key(parent)
    {
        return Err(DesignRunStoreError::Invalid(
            "parent design run is not published".into(),
        ));
    }
    let destination = paths.runs_dir.join(&manifest.run_id);
    if destination.exists() {
        let existing = load_canonical_manifest(&destination)?;
        if existing == manifest {
            ensure_indexed(&paths.runs_dir, &manifest, &mut hook)?;
            return Ok(manifest);
        }
        return Err(DesignRunStoreError::Invalid(
            "immutable design run identity already exists with different content".into(),
        ));
    }
    let stage = paths.runs_dir.join(format!(".run-stage-{}", unique_id()));
    fs::create_dir(&stage)?;
    let result = (|| {
        for input in &attachment_inputs {
            write_new_bytes(&stage.join(&input.name), &input.bytes)?;
        }
        write_new_json(&stage.join("manifest.json"), &manifest)?;
        sync_directory(&stage)?;
        let staged = load_canonical_manifest(&stage)?;
        if staged != manifest {
            return Err(DesignRunStoreError::Invalid(
                "staged design run changed during validation".into(),
            ));
        }
        hook(PublicationCheckpoint::StageValidated)?;
        fs::rename(&stage, &destination)?;
        sync_directory(&paths.runs_dir)?;
        hook(PublicationCheckpoint::RunAdopted)?;
        ensure_indexed(&paths.runs_dir, &manifest, &mut hook)?;
        Ok(manifest)
    })();
    if stage.exists() {
        let _ = fs::remove_dir_all(&stage);
    }
    result
}

fn manifest_from_request(
    request: PublishDesignRunRequest,
) -> Result<(DesignRunManifest, Vec<DesignRunAttachmentInput>), DesignRunStoreError> {
    let request_value = canonical_value(request.request);
    let interpretation_dependencies = request_value
        .get("interpretationDependencies")
        .cloned()
        .map(serde_json::from_value::<DesignRunInterpretationDependencies>)
        .transpose()?
        .unwrap_or_default()
        .normalise();
    let settings = canonical_value(request.settings);
    let attachments = request
        .attachments
        .iter()
        .map(|input| DesignRunAttachment {
            name: input.name.clone(),
            role: input.role.clone(),
            media_type: input.media_type.clone(),
            sha256: sha256_hex(&input.bytes),
            byte_size: input.bytes.len() as u64,
        })
        .collect::<Vec<_>>();
    let mut manifest = DesignRunManifest {
        schema_version: DESIGN_RUN_SCHEMA_VERSION.into(),
        run_id: String::new(),
        project_id: request.project_id,
        design_id: request.design_id,
        parent_run_id: request.parent_run_id,
        created_at: request.created_at,
        actor: request.actor,
        run_kind: request.run_kind,
        authored_revision_id: request.authored_revision_id,
        authored_snapshot_id: request.authored_snapshot_id,
        interpretation_dependencies,
        resolved_snapshot_id: request.resolved_snapshot_id,
        request_identity: hash_value(&request_value)?,
        request: request_value,
        settings_identity: hash_value(&settings)?,
        settings,
        solver_identity: request.solver_identity,
        runtime_identity: request.runtime_identity,
        input_identity: request.input_identity,
        result_identity: request.result_identity,
        status: request.status,
        diagnostics: request.diagnostics,
        metrics: request.metrics.map(canonical_value),
        attachments,
    };
    manifest.run_id = deterministic_run_id(&manifest)?;
    Ok((manifest, request.attachments))
}

pub fn list_design_runs(
    project_dir: &Path,
    design_id: &DesignId,
) -> Result<DesignRunList, DesignRunStoreError> {
    let package = load_project_package(project_dir)
        .map_err(|error| DesignRunStoreError::Package(format!("{error:#}")))?;
    let paths = owning_design(project_dir, &package.manifest.id, design_id)?;
    fs::create_dir_all(&paths.runs_dir)?;
    let index = load_index(
        &paths.runs_dir,
        package.manifest.id.clone(),
        design_id.clone(),
    )?;
    let mut runs = Vec::new();
    for (id, summary) in &index.runs {
        let manifest = load_canonical_manifest(&paths.runs_dir.join(id))?;
        if id != &summary.run_id || id != &manifest.run_id || run_summary(&manifest) != *summary {
            return Err(DesignRunStoreError::Invalid(
                "run index metadata does not match immutable manifest".into(),
            ));
        }
        runs.push(summary.clone());
    }
    runs.sort_by(|a, b| {
        a.created_at
            .cmp(&b.created_at)
            .then_with(|| a.run_id.cmp(&b.run_id))
    });
    let indexed = index.runs.keys().cloned().collect::<BTreeSet<_>>();
    let mut legacy_runs = Vec::new();
    for entry in fs::read_dir(&paths.runs_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if entry.file_type()?.is_dir()
            && !name.starts_with('.')
            && !name.starts_with("design-run-sha256-")
            && !indexed.contains(&name)
        {
            legacy_runs.push(LegacyDesignRunSummary {
                directory_name: name,
            });
        }
    }
    legacy_runs.sort_by(|a, b| a.directory_name.cmp(&b.directory_name));
    Ok(DesignRunList {
        project_id: package.manifest.id,
        design_id: design_id.clone(),
        runs,
        legacy_runs,
    })
}

pub fn inspect_design_run(
    project_dir: &Path,
    design_id: &DesignId,
    run_id: &str,
) -> Result<InspectedDesignRun, DesignRunStoreError> {
    validate_safe_name("run id", run_id)?;
    let package = load_project_package(project_dir)
        .map_err(|error| DesignRunStoreError::Package(format!("{error:#}")))?;
    let paths = owning_design(project_dir, &package.manifest.id, design_id)?;
    let index = load_index(&paths.runs_dir, package.manifest.id, design_id.clone())?;
    let run_dir = paths.runs_dir.join(run_id);
    reject_symlink(&run_dir)?;
    if index.runs.contains_key(run_id) {
        let manifest = load_canonical_manifest(&run_dir)?;
        if manifest.run_id != run_id {
            return Err(DesignRunStoreError::Invalid(
                "run directory identity does not match immutable manifest".into(),
            ));
        }
        return Ok(InspectedDesignRun::Canonical {
            manifest: Box::new(manifest),
        });
    }
    if !run_dir.is_dir() {
        return Err(DesignRunStoreError::NotFound(run_id.into()));
    }
    let mut files = fs::read_dir(&run_dir)?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    files.sort();
    let legacy_manifest = run_dir.join("run.json");
    let run_json = if legacy_manifest.is_file() {
        serde_json::from_slice(&fs::read(legacy_manifest)?).ok()
    } else {
        None
    };
    Ok(InspectedDesignRun::Legacy {
        directory_name: run_id.into(),
        run_json,
        files,
    })
}

pub fn design_run_staleness(
    manifest: &DesignRunManifest,
    inspected_snapshot_id: &str,
    ancestor_snapshot_ids: &[String],
) -> DesignRunStaleness {
    if manifest.authored_snapshot_id == inspected_snapshot_id {
        DesignRunStaleness::Current
    } else if ancestor_snapshot_ids
        .iter()
        .any(|id| id == &manifest.authored_snapshot_id)
    {
        DesignRunStaleness::StaleDescendant
    } else {
        DesignRunStaleness::Unrelated
    }
}

pub fn list_design_run_statuses(
    project_dir: &Path,
    design_id: &DesignId,
    inspected_snapshot_id: &str,
    ancestor_snapshot_ids: &[String],
) -> Result<Vec<DesignRunStatusProjection>, DesignRunStoreError> {
    let listed = list_design_runs(project_dir, design_id)?;
    let mut statuses = Vec::with_capacity(listed.runs.len());
    for summary in listed.runs {
        let InspectedDesignRun::Canonical { manifest } =
            inspect_design_run(project_dir, design_id, &summary.run_id)?
        else {
            return Err(DesignRunStoreError::Invalid(
                "canonical run index resolved to legacy content".into(),
            ));
        };
        let (staleness, staleness_reasons) = design_run_status_staleness(
            project_dir,
            design_id,
            &manifest,
            inspected_snapshot_id,
            ancestor_snapshot_ids,
        )?;
        statuses.push(DesignRunStatusProjection {
            run_id: manifest.run_id.clone(),
            status: manifest.status,
            staleness,
            staleness_reasons,
            interpretation_dependencies: manifest.interpretation_dependencies.clone(),
            authored_revision_id: manifest.authored_revision_id.clone(),
            authored_snapshot_id: manifest.authored_snapshot_id.clone(),
            resolved_snapshot_id: manifest.resolved_snapshot_id.clone(),
            solver_identity: manifest.solver_identity.clone(),
            runtime_identity: manifest.runtime_identity.clone(),
            settings_identity: manifest.settings_identity.clone(),
            diagnostics: manifest.diagnostics.clone(),
        });
    }
    Ok(statuses)
}

fn design_run_status_staleness(
    project_dir: &Path,
    design_id: &DesignId,
    manifest: &DesignRunManifest,
    inspected_snapshot_id: &str,
    ancestor_snapshot_ids: &[String],
) -> Result<(DesignRunStaleness, Vec<DesignRunStalenessReason>), DesignRunStoreError> {
    let model_staleness =
        design_run_staleness(manifest, inspected_snapshot_id, ancestor_snapshot_ids);
    if manifest.interpretation_dependencies.is_empty() {
        return Ok((model_staleness, Vec::new()));
    }
    let current_head = crate::list_drawing_interpretations(project_dir, design_id)
        .map_err(|error| DesignRunStoreError::Package(error.to_string()))?
        .head_revision_id;
    let mut reasons = Vec::new();
    for revision_id in &manifest.interpretation_dependencies.revision_ids {
        if current_head.as_deref() != Some(revision_id) {
            reasons.push(DesignRunStalenessReason {
                code: "interpretation.revision_superseded".into(),
                message: format!(
                    "Drawing interpretation `{revision_id}` is no longer the current interpretation revision."
                ),
                interpretation_revision_id: Some(revision_id.clone()),
                inference_id: None,
                current_interpretation_revision_id: current_head.clone(),
            });
        }
    }
    let eligible_inference_ids = current_head
        .as_deref()
        .map(|revision_id| {
            crate::drawing_interpretation_agent_context(project_dir, design_id, revision_id)
                .map(|context| {
                    context
                        .inferred_assumptions
                        .into_iter()
                        .map(|inference| inference.inference_id)
                        .collect::<BTreeSet<_>>()
                })
                .map_err(|error| DesignRunStoreError::Package(error.to_string()))
        })
        .transpose()?
        .unwrap_or_default();
    for inference_id in &manifest.interpretation_dependencies.inference_ids {
        if !eligible_inference_ids.contains(inference_id) {
            reasons.push(DesignRunStalenessReason {
                code: "interpretation.inference_no_longer_eligible".into(),
                message: format!(
                    "Drawing inference `{inference_id}` was corrected, rejected, confirmed, or otherwise removed from the eligible inferred candidates."
                ),
                interpretation_revision_id: manifest
                    .interpretation_dependencies
                    .revision_ids
                    .iter()
                    .find(|revision| inference_id.starts_with(&format!("{revision}:inference:")))
                    .cloned(),
                inference_id: Some(inference_id.clone()),
                current_interpretation_revision_id: current_head.clone(),
            });
        }
    }
    if reasons.is_empty() {
        Ok((model_staleness, reasons))
    } else {
        Ok((DesignRunStaleness::StaleDependency, reasons))
    }
}

fn validate_manifest(manifest: &DesignRunManifest) -> Result<(), DesignRunStoreError> {
    if manifest.schema_version != DESIGN_RUN_SCHEMA_VERSION
        || manifest.run_id != deterministic_run_id(manifest)?
    {
        return Err(DesignRunStoreError::Invalid(
            "design run schema or deterministic identity is invalid".into(),
        ));
    }
    for (label, value) in [
        ("created at", &manifest.created_at),
        ("actor type", &manifest.actor.actor_type),
        ("actor id", &manifest.actor.actor_id),
        ("run kind", &manifest.run_kind),
        ("authored revision", &manifest.authored_revision_id),
        ("authored snapshot", &manifest.authored_snapshot_id),
        ("solver identity", &manifest.solver_identity),
        ("runtime identity", &manifest.runtime_identity),
    ] {
        validate_nonempty(label, value)?;
    }
    if manifest.request_identity != hash_value(&manifest.request)?
        || manifest.settings_identity != hash_value(&manifest.settings)?
    {
        return Err(DesignRunStoreError::Invalid(
            "run request or settings identity is invalid".into(),
        ));
    }
    if manifest.interpretation_dependencies.clone().normalise()
        != manifest.interpretation_dependencies
        || manifest
            .interpretation_dependencies
            .revision_ids
            .iter()
            .any(|revision| revision.trim().is_empty())
        || manifest
            .interpretation_dependencies
            .inference_ids
            .iter()
            .any(|inference| {
                inference.trim().is_empty()
                    || !manifest
                        .interpretation_dependencies
                        .revision_ids
                        .iter()
                        .any(|revision| inference.starts_with(&format!("{revision}:inference:")))
            })
    {
        return Err(DesignRunStoreError::Invalid(
            "run interpretation dependencies are invalid or not canonical".into(),
        ));
    }
    for identity in manifest
        .input_identity
        .iter()
        .chain(manifest.result_identity.iter())
    {
        validate_sha256_identity(identity)?;
    }
    if manifest.status == DesignRunStatus::Completed
        && (manifest.input_identity.is_none() || manifest.result_identity.is_none())
    {
        return Err(DesignRunStoreError::Invalid(
            "completed run requires input and result identities".into(),
        ));
    }
    if manifest.status != DesignRunStatus::Completed
        && (manifest.result_identity.is_some() || manifest.metrics.is_some())
    {
        return Err(DesignRunStoreError::Invalid(
            "failed and unsupported runs must not contain result metrics or identities".into(),
        ));
    }
    if manifest
        .metrics
        .as_ref()
        .is_some_and(|value| !finite_json(value))
    {
        return Err(DesignRunStoreError::Invalid(
            "run metrics must be finite".into(),
        ));
    }
    if manifest.status == DesignRunStatus::Failed
        && !manifest
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == DesignRunDiagnosticSeverity::Error)
    {
        return Err(DesignRunStoreError::Invalid(
            "failed run requires an error diagnostic".into(),
        ));
    }
    if manifest.status == DesignRunStatus::Unsupported && manifest.diagnostics.is_empty() {
        return Err(DesignRunStoreError::Invalid(
            "unsupported run requires a diagnostic".into(),
        ));
    }
    let mut names = BTreeSet::new();
    for attachment in &manifest.attachments {
        validate_safe_name("attachment name", &attachment.name)?;
        if attachment.name == "manifest.json" {
            return Err(DesignRunStoreError::Invalid(
                "run attachment name is reserved".into(),
            ));
        }
        validate_nonempty("attachment media type", &attachment.media_type)?;
        validate_sha256_hex(&attachment.sha256)?;
        if !names.insert(attachment.name.clone()) {
            return Err(DesignRunStoreError::Invalid(
                "duplicate run attachment name".into(),
            ));
        }
        if manifest.status != DesignRunStatus::Completed
            && matches!(
                attachment.role,
                DesignRunAttachmentRole::Result
                    | DesignRunAttachmentRole::DesignActions
                    | DesignRunAttachmentRole::CheckResults
            )
        {
            return Err(DesignRunStoreError::Invalid(
                "failed and unsupported runs cannot publish result attachments".into(),
            ));
        }
    }
    Ok(())
}

fn load_canonical_manifest(run_dir: &Path) -> Result<DesignRunManifest, DesignRunStoreError> {
    reject_symlink(run_dir)?;
    let path = run_dir.join("manifest.json");
    reject_symlink(&path)?;
    let raw: Value = serde_json::from_slice(&fs::read(&path)?)?;
    let manifest: DesignRunManifest = serde_json::from_value(raw.clone())?;
    if serde_json::to_value(&manifest)? != raw {
        return Err(DesignRunStoreError::Invalid(
            "run manifest contains unsupported future data".into(),
        ));
    }
    validate_manifest(&manifest)?;
    let mut expected_files = manifest
        .attachments
        .iter()
        .map(|attachment| attachment.name.clone())
        .collect::<BTreeSet<_>>();
    expected_files.insert("manifest.json".into());
    let mut actual_files = BTreeSet::new();
    for entry in fs::read_dir(run_dir)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        if !kind.is_file() || kind.is_symlink() {
            return Err(DesignRunStoreError::Invalid(
                "canonical design run contains an unsupported filesystem entry".into(),
            ));
        }
        actual_files.insert(entry.file_name().to_string_lossy().into_owned());
    }
    if actual_files != expected_files {
        return Err(DesignRunStoreError::Invalid(
            "canonical design run contains undeclared or missing files".into(),
        ));
    }
    for attachment in &manifest.attachments {
        let path = run_dir.join(&attachment.name);
        reject_symlink(&path)?;
        let bytes = fs::read(&path)?;
        if bytes.len() as u64 != attachment.byte_size || sha256_hex(&bytes) != attachment.sha256 {
            return Err(DesignRunStoreError::Invalid(format!(
                "run attachment `{}` failed checksum validation",
                attachment.name
            )));
        }
    }
    Ok(manifest)
}

fn deterministic_run_id(manifest: &DesignRunManifest) -> Result<String, DesignRunStoreError> {
    #[derive(Serialize)]
    struct Material<'a> {
        version: &'static str,
        project_id: &'a ProjectId,
        design_id: &'a DesignId,
        parent_run_id: &'a Option<String>,
        created_at: &'a str,
        actor: &'a DesignRunActor,
        run_kind: &'a str,
        authored_revision_id: &'a str,
        authored_snapshot_id: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        interpretation_dependencies: Option<&'a DesignRunInterpretationDependencies>,
        resolved_snapshot_id: &'a Option<String>,
        request_identity: &'a str,
        settings_identity: &'a str,
        solver_identity: &'a str,
        runtime_identity: &'a str,
        input_identity: &'a Option<String>,
        result_identity: &'a Option<String>,
        status: DesignRunStatus,
        diagnostics: &'a [DesignRunDiagnostic],
        metrics: &'a Option<Value>,
        attachments: &'a [DesignRunAttachment],
    }
    let bytes = serde_json::to_vec(&Material {
        version: DESIGN_RUN_IDENTITY_VERSION,
        project_id: &manifest.project_id,
        design_id: &manifest.design_id,
        parent_run_id: &manifest.parent_run_id,
        created_at: &manifest.created_at,
        actor: &manifest.actor,
        run_kind: &manifest.run_kind,
        authored_revision_id: &manifest.authored_revision_id,
        authored_snapshot_id: &manifest.authored_snapshot_id,
        interpretation_dependencies: (!manifest.interpretation_dependencies.is_empty())
            .then_some(&manifest.interpretation_dependencies),
        resolved_snapshot_id: &manifest.resolved_snapshot_id,
        request_identity: &manifest.request_identity,
        settings_identity: &manifest.settings_identity,
        solver_identity: &manifest.solver_identity,
        runtime_identity: &manifest.runtime_identity,
        input_identity: &manifest.input_identity,
        result_identity: &manifest.result_identity,
        status: manifest.status,
        diagnostics: &manifest.diagnostics,
        metrics: &manifest.metrics,
        attachments: &manifest.attachments,
    })?;
    Ok(format!("design-run-sha256-{}", sha256_hex(&bytes)))
}

fn ensure_indexed<F>(
    runs_dir: &Path,
    manifest: &DesignRunManifest,
    hook: &mut F,
) -> Result<(), DesignRunStoreError>
where
    F: FnMut(PublicationCheckpoint) -> Result<(), DesignRunStoreError>,
{
    let mut index = load_index(
        runs_dir,
        manifest.project_id.clone(),
        manifest.design_id.clone(),
    )?;
    if let Some(parent) = &manifest.parent_run_id
        && !index.runs.contains_key(parent)
    {
        return Err(DesignRunStoreError::Invalid(
            "parent design run is not published".into(),
        ));
    }
    index
        .runs
        .insert(manifest.run_id.clone(), run_summary(manifest));
    save_index(runs_dir, &index, hook)
}
fn run_summary(manifest: &DesignRunManifest) -> DesignRunSummary {
    DesignRunSummary {
        run_id: manifest.run_id.clone(),
        run_kind: manifest.run_kind.clone(),
        created_at: manifest.created_at.clone(),
        status: manifest.status,
        authored_revision_id: manifest.authored_revision_id.clone(),
        authored_snapshot_id: manifest.authored_snapshot_id.clone(),
        interpretation_dependencies: manifest.interpretation_dependencies.clone(),
        parent_run_id: manifest.parent_run_id.clone(),
    }
}
fn index_path(runs_dir: &Path) -> PathBuf {
    runs_dir.join("index.json")
}
fn load_index(
    runs_dir: &Path,
    project_id: ProjectId,
    design_id: DesignId,
) -> Result<DesignRunIndex, DesignRunStoreError> {
    fs::create_dir_all(runs_dir)?;
    let path = index_path(runs_dir);
    recover_index(&path)?;
    if !path.exists() {
        return Ok(DesignRunIndex {
            schema_version: DESIGN_RUN_INDEX_SCHEMA_VERSION.into(),
            project_id,
            design_id,
            runs: BTreeMap::new(),
        });
    }
    reject_symlink(&path)?;
    let raw: Value = serde_json::from_slice(&fs::read(path)?)?;
    let index: DesignRunIndex = serde_json::from_value(raw.clone())?;
    if serde_json::to_value(&index)? != raw
        || index.schema_version != DESIGN_RUN_INDEX_SCHEMA_VERSION
        || index.project_id != project_id
        || index.design_id != design_id
    {
        return Err(DesignRunStoreError::Invalid(
            "design run index schema, ownership, or future data is invalid".into(),
        ));
    }
    Ok(index)
}
fn save_index<F>(
    runs_dir: &Path,
    index: &DesignRunIndex,
    hook: &mut F,
) -> Result<(), DesignRunStoreError>
where
    F: FnMut(PublicationCheckpoint) -> Result<(), DesignRunStoreError>,
{
    let path = index_path(runs_dir);
    recover_index(&path)?;
    let temporary = runs_dir.join(format!(".index.json.tmp-{}", unique_id()));
    let backup = runs_dir.join(".index.json.bak");
    write_new_json(&temporary, index)?;
    if path.exists() {
        fs::rename(&path, &backup)?;
        hook(PublicationCheckpoint::IndexBackedUp)?;
    }
    if let Err(error) = fs::rename(&temporary, &path) {
        if backup.exists() {
            fs::rename(&backup, &path)?;
        }
        return Err(error.into());
    }
    sync_directory(runs_dir)?;
    if backup.exists() {
        fs::remove_file(backup)?;
        sync_directory(runs_dir)?;
    }
    Ok(())
}
fn recover_index(path: &Path) -> Result<(), DesignRunStoreError> {
    let parent = path
        .parent()
        .ok_or_else(|| DesignRunStoreError::Invalid("run index has no parent".into()))?;
    let backup = parent.join(".index.json.bak");
    reject_symlink(path)?;
    reject_symlink(&backup)?;
    if backup.exists() {
        if path.exists() {
            fs::remove_file(&backup)?;
        } else {
            fs::rename(&backup, path)?;
        }
        sync_directory(parent)?;
    }
    Ok(())
}
fn owning_design(
    project_dir: &Path,
    project_id: &ProjectId,
    design_id: &DesignId,
) -> Result<crate::DesignPackagePaths, DesignRunStoreError> {
    let package = load_project_package(project_dir)
        .map_err(|error| DesignRunStoreError::Package(format!("{error:#}")))?;
    if &package.manifest.id != project_id
        || !package
            .designs
            .iter()
            .any(|design| &design.manifest.id == design_id)
    {
        return Err(DesignRunStoreError::Invalid(
            "design run ownership does not match the package".into(),
        ));
    }
    design_package_paths(project_dir, design_id)
        .map_err(|error| DesignRunStoreError::Package(error.to_string()))
}
fn canonical_value(value: Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.into_iter().map(canonical_value).collect()),
        Value::Object(values) => Value::Object(
            values
                .into_iter()
                .map(|(key, value)| (key, canonical_value(value)))
                .collect(),
        ),
        other => other,
    }
}
fn hash_value(value: &Value) -> Result<String, DesignRunStoreError> {
    Ok(format!(
        "sha256-{}",
        sha256_hex(&serde_json::to_vec(value)?)
    ))
}
fn finite_json(value: &Value) -> bool {
    match value {
        Value::Number(number) => number.as_f64().is_some_and(f64::is_finite),
        Value::Array(values) => values.iter().all(finite_json),
        Value::Object(values) => values.values().all(finite_json),
        _ => true,
    }
}
fn validate_nonempty(label: &str, value: &str) -> Result<(), DesignRunStoreError> {
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        Err(DesignRunStoreError::Invalid(format!("{label} is invalid")))
    } else {
        Ok(())
    }
}
fn validate_safe_name(label: &str, value: &str) -> Result<(), DesignRunStoreError> {
    validate_nonempty(label, value)?;
    let path = Path::new(value);
    if value.len() > 255
        || path.is_absolute()
        || path.components().count() != 1
        || !path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
        || matches!(value, "." | "..")
    {
        Err(DesignRunStoreError::Invalid(format!(
            "{label} is not a safe package name"
        )))
    } else {
        Ok(())
    }
}
fn validate_sha256_identity(value: &str) -> Result<(), DesignRunStoreError> {
    value
        .strip_prefix("sha256-")
        .or_else(|| value.strip_prefix("sha256:"))
        .ok_or_else(|| {
            DesignRunStoreError::Invalid("content identity must start with sha256-".into())
        })
        .and_then(validate_sha256_hex)
}
fn validate_sha256_hex(value: &str) -> Result<(), DesignRunStoreError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err(DesignRunStoreError::Invalid(
            "SHA-256 value is invalid".into(),
        ))
    }
}
fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn write_new_json<T: Serialize>(path: &Path, value: &T) -> Result<(), DesignRunStoreError> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    Ok(())
}
fn write_new_bytes(path: &Path, bytes: &[u8]) -> Result<(), DesignRunStoreError> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}
fn reject_symlink(path: &Path) -> Result<(), DesignRunStoreError> {
    if fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        Err(DesignRunStoreError::Invalid(
            "design run storage must not use symlinks".into(),
        ))
    } else {
        Ok(())
    }
}
fn unique_id() -> String {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!(
        "{}-{}-{}",
        std::process::id(),
        now.as_nanos(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )
}
#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<(), DesignRunStoreError> {
    File::open(path)?.sync_all()?;
    Ok(())
}
#[cfg(windows)]
fn sync_directory(_: &Path) -> Result<(), DesignRunStoreError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{create_named_project_package, load_project_package, save_project_package};

    struct Fixture {
        root: PathBuf,
        project: PathBuf,
        project_id: ProjectId,
        design_id: DesignId,
    }

    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!("fraia-run-test-{}", unique_id()));
            fs::create_dir(&root).unwrap();
            let project = root.join("project");
            let package = create_named_project_package(&project, "Run fixture").unwrap();
            Self {
                root,
                project,
                project_id: package.manifest.id,
                design_id: package.designs[0].manifest.id.clone(),
            }
        }

        fn completed(&self) -> PublishDesignRunRequest {
            PublishDesignRunRequest {
                project_id: self.project_id.clone(),
                design_id: self.design_id.clone(),
                parent_run_id: None,
                created_at: "2026-08-13T03:00:00Z".into(),
                actor: DesignRunActor {
                    actor_type: "user".into(),
                    actor_id: "engineer".into(),
                },
                run_kind: "frame2d_analysis".into(),
                authored_revision_id: "revision-authored-1".into(),
                authored_snapshot_id: "snapshot-authored-1".into(),
                resolved_snapshot_id: Some("snapshot-resolved-1".into()),
                request: serde_json::json!({"analysis": "frame2d", "loadCases": ["gravity"]}),
                settings: serde_json::json!({"solver": {"tolerance": 1e-9}}),
                solver_identity: "fraia.frame2d.internal.v1".into(),
                runtime_identity: "fraia-core.frame2d.runtime.v1".into(),
                input_identity: Some(format!("sha256-{}", "a".repeat(64))),
                result_identity: Some(format!("sha256-{}", "b".repeat(64))),
                status: DesignRunStatus::Completed,
                diagnostics: Vec::new(),
                metrics: Some(serde_json::json!({"maxDisplacementM": 0.004})),
                attachments: vec![
                    DesignRunAttachmentInput {
                        name: "solver-input.json".into(),
                        role: DesignRunAttachmentRole::SolverInput,
                        media_type: "application/json".into(),
                        bytes: br#"{"model":"exact"}"#.to_vec(),
                    },
                    DesignRunAttachmentInput {
                        name: "results.json".into(),
                        role: DesignRunAttachmentRole::Result,
                        media_type: "application/json".into(),
                        bytes: br#"{"displacement":0.004}"#.to_vec(),
                    },
                ],
            }
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn deterministic_publish_move_restart_and_package_save_preserve_one_run() {
        let fixture = Fixture::new();
        let first = publish_design_run(&fixture.project, fixture.completed()).unwrap();
        let replay = publish_design_run(&fixture.project, fixture.completed()).unwrap();
        assert_eq!(first, replay);
        assert_eq!(first.request_identity, replay.request_identity);
        assert_eq!(first.settings_identity, replay.settings_identity);
        assert_eq!(
            list_design_runs(&fixture.project, &fixture.design_id)
                .unwrap()
                .runs
                .len(),
            1
        );
        let package = load_project_package(&fixture.project).unwrap();
        save_project_package(&fixture.project, &package).unwrap();
        assert_eq!(
            inspect_design_run(&fixture.project, &fixture.design_id, &first.run_id).unwrap(),
            InspectedDesignRun::Canonical {
                manifest: Box::new(first.clone())
            }
        );
        let moved = fixture.root.join("moved-project");
        fs::rename(&fixture.project, &moved).unwrap();
        assert_eq!(
            inspect_design_run(&moved, &fixture.design_id, &first.run_id).unwrap(),
            InspectedDesignRun::Canonical {
                manifest: Box::new(first)
            }
        );
    }

    #[test]
    fn completed_run_can_use_the_authored_snapshot_directly() {
        let fixture = Fixture::new();
        let mut request = fixture.completed();
        request.resolved_snapshot_id = None;
        let run = publish_design_run(&fixture.project, request).unwrap();
        assert_eq!(run.status, DesignRunStatus::Completed);
        assert!(run.resolved_snapshot_id.is_none());
        assert!(run.input_identity.is_some() && run.result_identity.is_some());
    }

    #[test]
    fn attachment_tamper_and_unknown_manifest_fields_fail_closed() {
        let fixture = Fixture::new();
        let run = publish_design_run(&fixture.project, fixture.completed()).unwrap();
        let paths = design_package_paths(&fixture.project, &fixture.design_id).unwrap();
        fs::write(
            paths.runs_dir.join(&run.run_id).join("results.json"),
            b"tampered",
        )
        .unwrap();
        assert!(
            matches!(inspect_design_run(&fixture.project, &fixture.design_id, &run.run_id), Err(DesignRunStoreError::Invalid(message)) if message.contains("checksum"))
        );
        let second = publish_design_run(&fixture.project, {
            let mut request = fixture.completed();
            request.created_at = "2026-08-13T03:01:00Z".into();
            request
        })
        .unwrap();
        let path = paths.runs_dir.join(&second.run_id).join("manifest.json");
        let mut raw: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        raw["futureField"] = Value::Bool(true);
        fs::write(&path, serde_json::to_vec_pretty(&raw).unwrap()).unwrap();
        assert!(inspect_design_run(&fixture.project, &fixture.design_id, &second.run_id).is_err());

        let third = publish_design_run(&fixture.project, {
            let mut request = fixture.completed();
            request.created_at = "2026-08-13T03:01:30Z".into();
            request
        })
        .unwrap();
        fs::write(
            paths.runs_dir.join(&third.run_id).join("undeclared.txt"),
            b"not in manifest",
        )
        .unwrap();
        assert!(
            matches!(inspect_design_run(&fixture.project, &fixture.design_id, &third.run_id), Err(DesignRunStoreError::Invalid(message)) if message.contains("undeclared"))
        );
    }

    #[test]
    fn failed_and_unsupported_runs_publish_truthful_evidence_without_metrics() {
        let fixture = Fixture::new();
        let mut failed = fixture.completed();
        failed.created_at = "2026-08-13T03:02:00Z".into();
        failed.status = DesignRunStatus::Failed;
        failed.result_identity = None;
        failed.metrics = None;
        failed
            .attachments
            .retain(|attachment| attachment.role == DesignRunAttachmentRole::SolverInput);
        failed.attachments.push(DesignRunAttachmentInput {
            name: "solver.log".into(),
            role: DesignRunAttachmentRole::Log,
            media_type: "text/plain".into(),
            bytes: b"solver stopped before results".to_vec(),
        });
        failed.diagnostics = vec![DesignRunDiagnostic {
            severity: DesignRunDiagnosticSeverity::Error,
            code: "solver.failed".into(),
            message: "Solver stopped before it produced results.".into(),
        }];
        let failed = publish_design_run(&fixture.project, failed).unwrap();
        assert_eq!(failed.status, DesignRunStatus::Failed);
        assert!(failed.result_identity.is_none() && failed.metrics.is_none());

        let mut unsupported = fixture.completed();
        unsupported.created_at = "2026-08-13T03:03:00Z".into();
        unsupported.status = DesignRunStatus::Unsupported;
        unsupported.resolved_snapshot_id = None;
        unsupported.input_identity = None;
        unsupported.result_identity = None;
        unsupported.metrics = None;
        unsupported.attachments.clear();
        unsupported.diagnostics = vec![DesignRunDiagnostic {
            severity: DesignRunDiagnosticSeverity::Warning,
            code: "solver.unsupported".into(),
            message: "No reviewed solver supports this request.".into(),
        }];
        let unsupported = publish_design_run(&fixture.project, unsupported).unwrap();
        assert_eq!(unsupported.status, DesignRunStatus::Unsupported);
        assert!(unsupported.result_identity.is_none() && unsupported.metrics.is_none());

        let mut fabricated = fixture.completed();
        fabricated.status = DesignRunStatus::Failed;
        assert!(publish_design_run(&fixture.project, fabricated).is_err());
    }

    #[test]
    fn failure_injection_never_lists_partial_run_and_retry_recovers_complete_run() {
        let fixture = Fixture::new();
        let request = fixture.completed();
        let expected = manifest_from_request(request.clone()).unwrap().0;
        let failed =
            publish_design_run_with_hook(&fixture.project, request.clone(), |checkpoint| {
                if checkpoint == PublicationCheckpoint::StageValidated {
                    Err(DesignRunStoreError::Invalid(
                        "injected before adoption".into(),
                    ))
                } else {
                    Ok(())
                }
            });
        assert!(failed.is_err());
        assert!(
            list_design_runs(&fixture.project, &fixture.design_id)
                .unwrap()
                .runs
                .is_empty()
        );
        let interrupted =
            publish_design_run_with_hook(&fixture.project, request.clone(), |checkpoint| {
                if checkpoint == PublicationCheckpoint::RunAdopted {
                    Err(DesignRunStoreError::Invalid(
                        "injected after adoption".into(),
                    ))
                } else {
                    Ok(())
                }
            });
        assert!(interrupted.is_err());
        let list = list_design_runs(&fixture.project, &fixture.design_id).unwrap();
        assert!(list.runs.is_empty() && list.legacy_runs.is_empty());
        let recovered = publish_design_run(&fixture.project, request).unwrap();
        assert_eq!(recovered, expected);
        assert_eq!(
            list_design_runs(&fixture.project, &fixture.design_id)
                .unwrap()
                .runs
                .len(),
            1
        );

        let mut child_request = fixture.completed();
        child_request.created_at = "2026-08-13T03:00:30Z".into();
        child_request.parent_run_id = Some(recovered.run_id.clone());
        let expected_child = manifest_from_request(child_request.clone()).unwrap().0;
        let index_interrupted =
            publish_design_run_with_hook(&fixture.project, child_request.clone(), |checkpoint| {
                if checkpoint == PublicationCheckpoint::IndexBackedUp {
                    Err(DesignRunStoreError::Invalid(
                        "injected during index publication".into(),
                    ))
                } else {
                    Ok(())
                }
            });
        assert!(index_interrupted.is_err());
        let after_recovery = list_design_runs(&fixture.project, &fixture.design_id).unwrap();
        assert_eq!(after_recovery.runs.len(), 1);
        assert_eq!(after_recovery.runs[0].run_id, recovered.run_id);
        assert!(after_recovery.legacy_runs.is_empty());
        assert_eq!(
            publish_design_run(&fixture.project, child_request).unwrap(),
            expected_child
        );
        assert_eq!(
            list_design_runs(&fixture.project, &fixture.design_id)
                .unwrap()
                .runs
                .len(),
            2
        );
    }

    #[test]
    fn parent_and_descendant_staleness_are_exact_and_history_is_retained() {
        let fixture = Fixture::new();
        let parent = publish_design_run(&fixture.project, fixture.completed()).unwrap();
        let mut child_request = fixture.completed();
        child_request.created_at = "2026-08-13T03:04:00Z".into();
        child_request.parent_run_id = Some(parent.run_id.clone());
        child_request.authored_revision_id = "revision-authored-2".into();
        child_request.authored_snapshot_id = "snapshot-authored-2".into();
        let child = publish_design_run(&fixture.project, child_request).unwrap();
        assert_eq!(
            list_design_runs(&fixture.project, &fixture.design_id)
                .unwrap()
                .runs
                .len(),
            2
        );
        assert_eq!(
            design_run_staleness(&parent, "snapshot-authored-1", &[]),
            DesignRunStaleness::Current
        );
        assert_eq!(
            design_run_staleness(
                &parent,
                "snapshot-authored-2",
                &["snapshot-authored-1".into()]
            ),
            DesignRunStaleness::StaleDescendant
        );
        assert_eq!(
            design_run_staleness(&child, "snapshot-other", &[]),
            DesignRunStaleness::Unrelated
        );
        let projected = list_design_run_statuses(
            &fixture.project,
            &fixture.design_id,
            "snapshot-authored-2",
            &["snapshot-authored-1".into()],
        )
        .unwrap();
        assert_eq!(projected[0].staleness, DesignRunStaleness::StaleDescendant);
        assert_eq!(projected[1].staleness, DesignRunStaleness::Current);
    }

    #[test]
    fn legacy_runs_are_inspected_read_only_without_rewrite() {
        let fixture = Fixture::new();
        let paths = design_package_paths(&fixture.project, &fixture.design_id).unwrap();
        let legacy = paths.runs_dir.join("legacy-validation-1");
        fs::create_dir(&legacy).unwrap();
        let bytes = br#"{"id":"legacy-validation-1","status":"complete"}"#;
        fs::write(legacy.join("run.json"), bytes).unwrap();
        fs::write(legacy.join("summary.md"), "Legacy result").unwrap();
        let list = list_design_runs(&fixture.project, &fixture.design_id).unwrap();
        assert_eq!(list.legacy_runs[0].directory_name, "legacy-validation-1");
        assert!(matches!(
            inspect_design_run(&fixture.project, &fixture.design_id, "legacy-validation-1")
                .unwrap(),
            InspectedDesignRun::Legacy { .. }
        ));
        assert_eq!(fs::read(legacy.join("run.json")).unwrap(), bytes);
    }
}
