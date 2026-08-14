//! Snapshot-bound deterministic analysis service.
//!
//! The service receives only an accepted immutable revision. It resolves that
//! snapshot through `fraia-core`'s deterministic adapter, runs the internal
//! frame solver when supported, and stores exact evidence on that source
//! snapshot. It has no HTTP, UI, or mutable-project dependency.

use crate::evidence::{
    AnalysisAttachmentKind, AnalysisComboMetrics, AnalysisEvidence, AnalysisEvidenceAttachment,
    AnalysisEvidenceManifest, AnalysisEvidenceStatus, AnalysisMetrics, EvidenceDependency,
    EvidenceError,
};
use crate::repository::{InMemoryRevisionRepository, RepositoryError};
use crate::{EvidenceId, RevisionId, SnapshotId};
use fraia_core::{
    CheckReport, DeterministicDerivation, DeterministicDerivationOutcome,
    DeterministicDerivationRequest, FrameModel2D, Intent, ProjectFile, ProjectFiles, Requirements,
    SearchPermissions, SnapshotBoundStructuralModel, derive_conservative_check_report,
    derive_design_action_report, derive_from_snapshot, frame2d::solve_frame_2d,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fmt;

pub const INTERNAL_FRAME2D_SOLVER_IDENTITY: &str = "fraia.frame2d.internal.v1";
pub const INTERNAL_FRAME2D_RUNTIME_IDENTITY: &str = "fraia-core.frame2d.runtime.v1";
pub const UNSUPPORTED_FRAME3D_SOLVER_IDENTITY: &str = "fraia.frame3d.unavailable.v1";
pub const ANALYSIS_SETTINGS_FORMAT_VERSION: &str = "fraia.analysis.settings.v1";
pub const ANALYSIS_INPUT_FORMAT_VERSION: &str = "fraia.analysis.input.v1";
pub const ANALYSIS_RESULT_FORMAT_VERSION: &str = "fraia.analysis.result.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisExecutionStage {
    Preparing,
    Resolving,
    Solving,
    Collecting,
}

/// Conservative limits used only to produce deterministic check evidence.
/// They are explicitly persisted with the run and are not a code-compliance
/// claim.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisCheckLimits {
    pub max_utilization: f64,
    pub max_drift_ratio: f64,
    pub max_deflection_ratio: f64,
}

impl Default for AnalysisCheckLimits {
    fn default() -> Self {
        Self {
            max_utilization: 1.0,
            max_drift_ratio: 300.0,
            max_deflection_ratio: 360.0,
        }
    }
}

/// All non-model settings that affect a deterministic analysis attempt.
/// Solver/runtime identities are derived from the request so a caller cannot
/// label the internal solver as an unavailable external runtime.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisSettings {
    pub request: DeterministicDerivationRequest,
    pub check_limits: AnalysisCheckLimits,
}

impl AnalysisSettings {
    pub fn frame2d() -> Self {
        Self {
            request: DeterministicDerivationRequest::frame2d(),
            check_limits: AnalysisCheckLimits::default(),
        }
    }

    pub fn frame3d() -> Self {
        Self {
            request: DeterministicDerivationRequest::Frame3DRealization {
                configuration_version: "fraia.frame3d.realization.v1".into(),
            },
            check_limits: AnalysisCheckLimits::default(),
        }
    }

    pub fn with_request(request: DeterministicDerivationRequest) -> Self {
        Self {
            request,
            check_limits: AnalysisCheckLimits::default(),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        let limits = &self.check_limits;
        if !limits.max_utilization.is_finite()
            || !limits.max_drift_ratio.is_finite()
            || !limits.max_deflection_ratio.is_finite()
            || limits.max_utilization <= 0.0
            || limits.max_drift_ratio <= 0.0
            || limits.max_deflection_ratio <= 0.0
        {
            return Err("analysis check limits must be finite and positive".into());
        }
        Ok(())
    }

    pub fn solver_identity(&self) -> &'static str {
        match self.request {
            DeterministicDerivationRequest::Frame2DRealization { .. } => {
                INTERNAL_FRAME2D_SOLVER_IDENTITY
            }
            DeterministicDerivationRequest::Frame3DRealization { .. } => {
                UNSUPPORTED_FRAME3D_SOLVER_IDENTITY
            }
        }
    }

    pub fn runtime_identity(&self) -> &'static str {
        INTERNAL_FRAME2D_RUNTIME_IDENTITY
    }

    pub fn payload(&self) -> Result<String, SnapshotAnalysisError> {
        serde_json::to_string(self)
            .map_err(|error| SnapshotAnalysisError::Serialization(error.to_string()))
    }

    pub fn identity(&self) -> Result<String, SnapshotAnalysisError> {
        Ok(hash_bytes(self.payload()?.as_bytes()))
    }
}

/// Immutable, caller-readable result of one analysis attempt. `evidence` is
/// always bound to the exact authored snapshot, including unsuccessful runs;
/// callers must inspect `outcome` and diagnostics rather than infer success.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapshotAnalysisRun {
    pub revision_id: RevisionId,
    /// Canonical design-run identity after a project adapter publishes this
    /// attempt. Transient analysis can leave it absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_run_id: Option<String>,
    pub evidence: AnalysisEvidence,
    pub outcome: SnapshotAnalysisOutcome,
    /// Durable canonical payload for the resolved derivative, when a
    /// realization was available. The app adapter may persist this alongside
    /// the authored snapshot so evidence never points at an unmaterialised
    /// foreign identity.
    pub resolved_snapshot: Option<ResolvedSnapshotRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedSnapshotRecord {
    pub id: SnapshotId,
    pub format_version: String,
    pub canonical_bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SnapshotAnalysisOutcome {
    Completed {
        resolved_snapshot_id: SnapshotId,
        input_hash: String,
        result_hash: String,
        combo_count: usize,
        metrics: AnalysisMetrics,
    },
    Unsupported {
        diagnostics: Vec<String>,
    },
    Failed {
        diagnostics: Vec<String>,
    },
}

impl SnapshotAnalysisOutcome {
    pub fn completed(&self) -> bool {
        matches!(self, Self::Completed { .. })
    }

    pub fn status(&self) -> AnalysisEvidenceStatus {
        match self {
            Self::Completed { .. } => AnalysisEvidenceStatus::Completed,
            Self::Failed { .. } => AnalysisEvidenceStatus::Failed,
            Self::Unsupported { .. } => AnalysisEvidenceStatus::Unsupported,
        }
    }

    pub fn metrics(&self) -> Option<&AnalysisMetrics> {
        match self {
            Self::Completed { metrics, .. } => Some(metrics),
            Self::Failed { .. } | Self::Unsupported { .. } => None,
        }
    }

    pub fn diagnostics(&self) -> &[String] {
        match self {
            Self::Completed { .. } => &[],
            Self::Failed { diagnostics } | Self::Unsupported { diagnostics } => diagnostics,
        }
    }
}

impl SnapshotAnalysisRun {
    pub fn revision_id(&self) -> &RevisionId {
        &self.revision_id
    }

    pub fn evidence(&self) -> &AnalysisEvidence {
        &self.evidence
    }

    pub fn outcome(&self) -> &SnapshotAnalysisOutcome {
        &self.outcome
    }

    pub fn metrics(&self) -> Option<&AnalysisMetrics> {
        self.outcome.metrics()
    }
}

/// One alternative's exact analysis identity and actual metric envelope.
#[derive(Debug, Clone, PartialEq)]
pub struct AnalysisComparisonEntry {
    pub evidence_id: EvidenceId,
    pub authored_snapshot_id: SnapshotId,
    pub resolved_snapshot_id: SnapshotId,
    pub input_identity: String,
    pub result_identity: String,
    pub metrics: AnalysisMetrics,
}

/// Comparable schemes share the exact execution request, solver/runtime, and
/// settings identity. Their authored/resolved/input/result identities and
/// actual metric envelopes remain separate.
#[derive(Debug, Clone, PartialEq)]
pub struct AnalysisComparison {
    pub solver_identity: String,
    pub runtime_identity: String,
    pub settings_identity: String,
    pub settings_payload: String,
    pub request: DeterministicDerivationRequest,
    pub baseline: AnalysisComparisonEntry,
    pub candidate: AnalysisComparisonEntry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnalysisComparisonError {
    RunNotCompleted { evidence_id: EvidenceId },
    MissingManifest { evidence_id: EvidenceId },
    MismatchedExecution { field: &'static str },
}

impl fmt::Display for AnalysisComparisonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RunNotCompleted { evidence_id } => {
                write!(
                    formatter,
                    "analysis evidence `{evidence_id}` is not completed"
                )
            }
            Self::MissingManifest { evidence_id } => {
                write!(
                    formatter,
                    "analysis evidence `{evidence_id}` has no analysis manifest"
                )
            }
            Self::MismatchedExecution { field } => {
                write!(formatter, "analysis alternatives do not share {field}")
            }
        }
    }
}

impl Error for AnalysisComparisonError {}

#[derive(Debug)]
pub enum SnapshotAnalysisError {
    Repository(RepositoryError),
    Evidence(EvidenceError),
    DuplicateEvidence(EvidenceId),
    InvalidSettings(String),
    Serialization(String),
    Cancelled,
}

impl From<RepositoryError> for SnapshotAnalysisError {
    fn from(value: RepositoryError) -> Self {
        Self::Repository(value)
    }
}

impl From<EvidenceError> for SnapshotAnalysisError {
    fn from(value: EvidenceError) -> Self {
        Self::Evidence(value)
    }
}

impl fmt::Display for SnapshotAnalysisError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Repository(error) => error.fmt(formatter),
            Self::Evidence(error) => error.fmt(formatter),
            Self::DuplicateEvidence(id) => {
                write!(formatter, "analysis evidence `{id}` already exists")
            }
            Self::InvalidSettings(error) => write!(formatter, "invalid analysis settings: {error}"),
            Self::Serialization(error) => {
                write!(
                    formatter,
                    "could not hash deterministic analysis data: {error}"
                )
            }
            Self::Cancelled => formatter.write_str("analysis attempt was cancelled"),
        }
    }
}

impl Error for SnapshotAnalysisError {}

/// Uses the pinned default frame2d settings.
pub fn analyse_accepted_revision(
    repository: &mut InMemoryRevisionRepository,
    revision_id: &RevisionId,
    evidence_id: EvidenceId,
) -> Result<SnapshotAnalysisRun, SnapshotAnalysisError> {
    analyse_accepted_revision_with_settings(
        repository,
        revision_id,
        evidence_id,
        AnalysisSettings::frame2d(),
    )
}

/// Runs one accepted revision with an explicit deterministic request and the
/// default check limits. This is the supported way to exercise an unsupported
/// request such as the currently unavailable frame3d realization.
pub fn analyse_accepted_revision_with_request(
    repository: &mut InMemoryRevisionRepository,
    revision_id: &RevisionId,
    evidence_id: EvidenceId,
    request: DeterministicDerivationRequest,
) -> Result<SnapshotAnalysisRun, SnapshotAnalysisError> {
    analyse_accepted_revision_with_settings(
        repository,
        revision_id,
        evidence_id,
        AnalysisSettings::with_request(request),
    )
}

/// Resolves, runs, and attaches one deterministic analysis record. A failed
/// realization/solve is recorded as failed evidence with diagnostics, never as
/// a fabricated successful result.
pub fn analyse_accepted_revision_with_settings(
    repository: &mut InMemoryRevisionRepository,
    revision_id: &RevisionId,
    evidence_id: EvidenceId,
    settings: AnalysisSettings,
) -> Result<SnapshotAnalysisRun, SnapshotAnalysisError> {
    analyse_accepted_revision_with_control(
        repository,
        revision_id,
        evidence_id,
        settings,
        |_| {},
        || false,
    )
}

pub fn analyse_accepted_revision_with_control<P, C>(
    repository: &mut InMemoryRevisionRepository,
    revision_id: &RevisionId,
    evidence_id: EvidenceId,
    settings: AnalysisSettings,
    mut progress: P,
    mut cancelled: C,
) -> Result<SnapshotAnalysisRun, SnapshotAnalysisError>
where
    P: FnMut(AnalysisExecutionStage),
    C: FnMut() -> bool,
{
    progress(AnalysisExecutionStage::Preparing);
    if cancelled() {
        return Err(SnapshotAnalysisError::Cancelled);
    }
    if repository.evidence(&evidence_id).is_ok() {
        return Err(SnapshotAnalysisError::DuplicateEvidence(evidence_id));
    }
    settings
        .validate()
        .map_err(SnapshotAnalysisError::InvalidSettings)?;
    let settings_payload = settings.payload()?;
    let settings_identity = settings.identity()?;
    let revision = repository.revision(revision_id)?.clone();
    let snapshot = repository.snapshot(revision.snapshot_id())?.clone();
    let authored_id = snapshot.id().clone();
    let input_identity = |resolved_snapshot_id: Option<&SnapshotId>,
                          model: Option<&FrameModel2D>| {
        hash_json(&AnalysisInputPayload {
            format_version: ANALYSIS_INPUT_FORMAT_VERSION,
            authored_snapshot_id: authored_id.as_str(),
            resolved_snapshot_id: resolved_snapshot_id.map(SnapshotId::as_str),
            request: &settings.request,
            settings_identity: &settings_identity,
            realized_model: model,
        })
    };
    progress(AnalysisExecutionStage::Resolving);
    if cancelled() {
        return Err(SnapshotAnalysisError::Cancelled);
    }
    let derivation = derive_from_snapshot(
        SnapshotBoundStructuralModel::new(authored_id.as_str(), snapshot.model()),
        settings.request.clone(),
    );

    let (evidence, outcome, resolved_snapshot) = match derivation {
        DeterministicDerivationOutcome::Unsupported { diagnostic, .. } => {
            let diagnostics = vec![format!("{}: {}", diagnostic.code, diagnostic.message)];
            let input_hash = input_identity(None, None)?;
            let manifest = manifest(
                &authored_id,
                None,
                Some(input_hash.clone()),
                None,
                AnalysisEvidenceStatus::Unsupported,
                &settings,
                settings_payload.clone(),
                diagnostics.clone(),
                None,
                Vec::new(),
            )?;
            (
                AnalysisEvidence::with_analysis_manifest(
                    evidence_id.clone(),
                    authored_id.clone(),
                    None,
                    dependencies_for_snapshot_with_settings(&authored_id, &settings),
                    manifest,
                )?,
                SnapshotAnalysisOutcome::Unsupported { diagnostics },
                None,
            )
        }
        DeterministicDerivationOutcome::Derived(DeterministicDerivation::Frame2D(derived)) => {
            progress(AnalysisExecutionStage::Solving);
            if cancelled() {
                return Err(SnapshotAnalysisError::Cancelled);
            }
            let resolved_id = SnapshotId::from(derived.manifest.derived_id.clone());
            let input_hash = input_identity(Some(&resolved_id), Some(&derived.realization.model))?;
            let mut diagnostics: Vec<String> = derived
                .realization
                .diagnostics
                .iter()
                .map(|diagnostic| format!("{}: {}", diagnostic.code, diagnostic.message))
                .collect();
            let mut solved = Vec::new();
            let mut solve_error = None;
            for combo in &derived.realization.model.combos {
                if cancelled() {
                    return Err(SnapshotAnalysisError::Cancelled);
                }
                match solve_frame_2d(&derived.realization.model, combo) {
                    Ok(result) => solved.push(result),
                    Err(error) => {
                        solve_error = Some(error);
                        break;
                    }
                }
            }
            let mut results: Vec<_> = match solve_error {
                None => solved,
                Some(error) => {
                    diagnostics.push(format!("solver.frame2d-failed: {error}"));
                    let manifest = manifest(
                        &authored_id,
                        Some(&resolved_id),
                        Some(input_hash),
                        None,
                        AnalysisEvidenceStatus::Failed,
                        &settings,
                        settings_payload.clone(),
                        diagnostics.clone(),
                        None,
                        Vec::new(),
                    )?;
                    let evidence = AnalysisEvidence::with_analysis_manifest(
                        evidence_id.clone(),
                        authored_id.clone(),
                        Some(resolved_id),
                        dependencies_for_snapshot_with_settings(&authored_id, &settings),
                        manifest,
                    )?;
                    let outcome = SnapshotAnalysisOutcome::Failed { diagnostics };
                    progress(AnalysisExecutionStage::Collecting);
                    if cancelled() {
                        return Err(SnapshotAnalysisError::Cancelled);
                    }
                    repository.attach_evidence(revision_id, evidence.clone())?;
                    return Ok(SnapshotAnalysisRun {
                        revision_id: revision_id.clone(),
                        canonical_run_id: None,
                        evidence,
                        outcome,
                        resolved_snapshot: Some(resolved_snapshot_record(&derived)?),
                    });
                }
            };
            if cancelled() {
                return Err(SnapshotAnalysisError::Cancelled);
            }
            progress(AnalysisExecutionStage::Collecting);
            results.sort_by(|left, right| left.combo.id.cmp(&right.combo.id));
            let metrics = metrics_for_results(&results);
            if !metrics.is_finite() {
                diagnostics
                    .push("solver.non-finite-metrics: solver returned a non-finite metric".into());
                let manifest = manifest(
                    &authored_id,
                    Some(&resolved_id),
                    Some(input_hash),
                    None,
                    AnalysisEvidenceStatus::Failed,
                    &settings,
                    settings_payload.clone(),
                    diagnostics.clone(),
                    None,
                    Vec::new(),
                )?;
                (
                    AnalysisEvidence::with_analysis_manifest(
                        evidence_id.clone(),
                        authored_id.clone(),
                        Some(resolved_id),
                        dependencies_for_snapshot_with_settings(&authored_id, &settings),
                        manifest,
                    )?,
                    SnapshotAnalysisOutcome::Failed { diagnostics },
                    Some(resolved_snapshot_record(&derived)?),
                )
            } else {
                let result_hash = hash_json(&AnalysisResultPayload {
                    format_version: ANALYSIS_RESULT_FORMAT_VERSION,
                    results: &results,
                })?;
                let (attachments, attachment_diagnostics) =
                    derive_attachments(&derived.realization.model, &settings);
                diagnostics.extend(attachment_diagnostics);
                let manifest = manifest(
                    &authored_id,
                    Some(&resolved_id),
                    Some(input_hash.clone()),
                    Some(result_hash.clone()),
                    AnalysisEvidenceStatus::Completed,
                    &settings,
                    settings_payload,
                    diagnostics,
                    Some(metrics.clone()),
                    attachments,
                )?;
                (
                    AnalysisEvidence::with_analysis_manifest(
                        evidence_id.clone(),
                        authored_id.clone(),
                        Some(resolved_id.clone()),
                        dependencies_for_snapshot_with_settings(&authored_id, &settings),
                        manifest,
                    )?,
                    SnapshotAnalysisOutcome::Completed {
                        resolved_snapshot_id: resolved_id,
                        input_hash,
                        result_hash,
                        combo_count: results.len(),
                        metrics,
                    },
                    Some(resolved_snapshot_record(&derived)?),
                )
            }
        }
    };
    if cancelled() {
        return Err(SnapshotAnalysisError::Cancelled);
    }
    repository.attach_evidence(revision_id, evidence.clone())?;
    Ok(SnapshotAnalysisRun {
        revision_id: revision_id.clone(),
        canonical_run_id: None,
        evidence,
        outcome,
        resolved_snapshot,
    })
}

/// Compares two completed alternatives only when they share the exact
/// deterministic execution request, solver/runtime, and settings identity.
pub fn compare_completed_runs(
    baseline: &SnapshotAnalysisRun,
    candidate: &SnapshotAnalysisRun,
) -> Result<AnalysisComparison, AnalysisComparisonError> {
    let baseline_manifest = baseline.evidence.analysis_manifest().ok_or_else(|| {
        AnalysisComparisonError::MissingManifest {
            evidence_id: baseline.evidence.id().clone(),
        }
    })?;
    let candidate_manifest = candidate.evidence.analysis_manifest().ok_or_else(|| {
        AnalysisComparisonError::MissingManifest {
            evidence_id: candidate.evidence.id().clone(),
        }
    })?;
    if baseline_manifest.status != AnalysisEvidenceStatus::Completed
        || !baseline.outcome.completed()
    {
        return Err(AnalysisComparisonError::RunNotCompleted {
            evidence_id: baseline.evidence.id().clone(),
        });
    }
    if candidate_manifest.status != AnalysisEvidenceStatus::Completed
        || !candidate.outcome.completed()
    {
        return Err(AnalysisComparisonError::RunNotCompleted {
            evidence_id: candidate.evidence.id().clone(),
        });
    }
    for (field, left, right) in [
        (
            "solver identity",
            baseline_manifest.solver_identity(),
            candidate_manifest.solver_identity(),
        ),
        (
            "runtime identity",
            baseline_manifest.runtime_identity(),
            candidate_manifest.runtime_identity(),
        ),
        (
            "settings identity",
            baseline_manifest.settings_identity(),
            candidate_manifest.settings_identity(),
        ),
    ] {
        if left != right {
            return Err(AnalysisComparisonError::MismatchedExecution { field });
        }
    }
    if baseline_manifest.request != candidate_manifest.request {
        return Err(AnalysisComparisonError::MismatchedExecution { field: "request" });
    }
    let baseline_metrics =
        baseline
            .metrics()
            .cloned()
            .ok_or_else(|| AnalysisComparisonError::RunNotCompleted {
                evidence_id: baseline.evidence.id().clone(),
            })?;
    let candidate_metrics =
        candidate
            .metrics()
            .cloned()
            .ok_or_else(|| AnalysisComparisonError::RunNotCompleted {
                evidence_id: candidate.evidence.id().clone(),
            })?;
    Ok(AnalysisComparison {
        solver_identity: baseline_manifest.solver_identity().into(),
        runtime_identity: baseline_manifest.runtime_identity().into(),
        settings_identity: baseline_manifest.settings_identity().into(),
        settings_payload: baseline_manifest.settings_payload().into(),
        request: baseline_manifest.request.clone(),
        baseline: comparison_entry(&baseline.evidence, baseline_metrics),
        candidate: comparison_entry(&candidate.evidence, candidate_metrics),
    })
}

fn comparison_entry(
    evidence: &AnalysisEvidence,
    metrics: AnalysisMetrics,
) -> AnalysisComparisonEntry {
    AnalysisComparisonEntry {
        evidence_id: evidence.id().clone(),
        authored_snapshot_id: evidence.authored_snapshot_id().clone(),
        resolved_snapshot_id: SnapshotId::from(
            evidence
                .resolved_snapshot_identity()
                .expect("completed evidence has a resolved snapshot identity"),
        ),
        input_identity: evidence
            .input_identity()
            .expect("completed evidence has an input identity")
            .into(),
        result_identity: evidence
            .result_identity()
            .expect("completed evidence has a result identity")
            .into(),
        metrics,
    }
}

fn resolved_snapshot_record(
    derived: &fraia_core::Frame2DDerivation,
) -> Result<ResolvedSnapshotRecord, SnapshotAnalysisError> {
    let payload = serde_json::json!({
        "format_version": derived.manifest.format_version,
        "source_snapshot_id": derived.manifest.source_snapshot_id,
        "request": {
            "Frame2DRealization": {
                "configuration_version": derived.manifest.configuration_version,
            }
        },
        "understanding": derived.understanding,
        "validation": derived.validation,
        "realization": derived.realization,
    });
    let canonical_bytes = serde_json::to_vec(&payload)
        .map_err(|error| SnapshotAnalysisError::Serialization(error.to_string()))?;
    let id = SnapshotId::from(format!("sha256:{:x}", Sha256::digest(&canonical_bytes)));
    if id.as_str() != derived.manifest.derived_id {
        return Err(SnapshotAnalysisError::Serialization(
            "resolved derivation payload identity did not match its manifest".into(),
        ));
    }
    Ok(ResolvedSnapshotRecord {
        id,
        format_version: derived.manifest.format_version.clone(),
        canonical_bytes,
    })
}

/// Dependencies for the default whole-snapshot analysis boundary.
pub fn dependencies_for_snapshot(snapshot_id: &SnapshotId) -> Vec<EvidenceDependency> {
    dependencies_for_snapshot_with_settings(snapshot_id, &AnalysisSettings::frame2d())
}

/// Dependencies for an analysis run with explicit execution pinning.
pub fn dependencies_for_snapshot_with_settings(
    snapshot_id: &SnapshotId,
    settings: &AnalysisSettings,
) -> Vec<EvidenceDependency> {
    use crate::diff::DiffCategory;
    vec![
        EvidenceDependency::new(
            "authored-structural-snapshot",
            snapshot_id.to_string(),
            [
                DiffCategory::Geometry,
                DiffCategory::Topology,
                DiffCategory::Member,
                DiffCategory::Plate,
                DiffCategory::Support,
                DiffCategory::Load,
                DiffCategory::Release,
                DiffCategory::Role,
            ],
        ),
        EvidenceDependency::always_invalidating("analysis-solver", settings.solver_identity()),
        EvidenceDependency::always_invalidating("analysis-runtime", settings.runtime_identity()),
        EvidenceDependency::always_invalidating(
            "analysis-settings",
            settings
                .identity()
                .unwrap_or_else(|_| "invalid:analysis-settings".into()),
        ),
    ]
}

fn manifest(
    authored_snapshot_id: &SnapshotId,
    resolved_snapshot_id: Option<&SnapshotId>,
    input_hash: Option<String>,
    result_hash: Option<String>,
    status: AnalysisEvidenceStatus,
    settings: &AnalysisSettings,
    settings_payload: String,
    diagnostics: Vec<String>,
    metrics: Option<AnalysisMetrics>,
    attachments: Vec<AnalysisEvidenceAttachment>,
) -> Result<AnalysisEvidenceManifest, SnapshotAnalysisError> {
    Ok(AnalysisEvidenceManifest {
        authored_snapshot_hash: authored_snapshot_id.to_string(),
        resolved_snapshot_hash: resolved_snapshot_id.map(ToString::to_string),
        input_hash,
        result_hash,
        solver_identity: settings.solver_identity().into(),
        diagnostics,
        status,
        request: settings.request.clone(),
        runtime_identity: settings.runtime_identity().into(),
        settings_identity: settings.identity()?,
        settings_payload,
        metrics,
        attachments,
        canonical_run_id: None,
    })
}

#[derive(Serialize)]
struct AnalysisInputPayload<'a> {
    format_version: &'static str,
    authored_snapshot_id: &'a str,
    resolved_snapshot_id: Option<&'a str>,
    request: &'a DeterministicDerivationRequest,
    settings_identity: &'a str,
    realized_model: Option<&'a FrameModel2D>,
}

#[derive(Serialize)]
struct AnalysisResultPayload<'a> {
    format_version: &'static str,
    results: &'a [fraia_core::SolveResult2D],
}

fn metrics_for_results(results: &[fraia_core::SolveResult2D]) -> AnalysisMetrics {
    let combo_metrics: Vec<_> = results
        .iter()
        .map(|result| AnalysisComboMetrics {
            combo_id: result.combo.id.clone(),
            max_utilization: result.metrics.max_utilization,
            max_ux_m: result.metrics.max_ux_m,
            max_uy_m: result.metrics.max_uy_m,
            max_reaction_n: result.metrics.max_reaction_n,
        })
        .collect();
    AnalysisMetrics {
        max_utilization: combo_metrics
            .iter()
            .map(|metrics| metrics.max_utilization)
            .fold(0.0, f64::max),
        max_ux_m: combo_metrics
            .iter()
            .map(|metrics| metrics.max_ux_m)
            .fold(0.0, f64::max),
        max_uy_m: combo_metrics
            .iter()
            .map(|metrics| metrics.max_uy_m)
            .fold(0.0, f64::max),
        max_reaction_n: combo_metrics
            .iter()
            .map(|metrics| metrics.max_reaction_n)
            .fold(0.0, f64::max),
        combo_metrics,
    }
}

fn derive_attachments(
    model: &FrameModel2D,
    settings: &AnalysisSettings,
) -> (Vec<AnalysisEvidenceAttachment>, Vec<String>) {
    let project = analysis_project(settings);
    let mut diagnostics = Vec::new();
    let Ok(mut actions) = derive_design_action_report(&project, model) else {
        diagnostics.push("design-actions.unavailable: could not derive design actions".into());
        return (Vec::new(), diagnostics);
    };
    actions
        .member_actions
        .sort_by(|left, right| left.member_id.cmp(&right.member_id));
    actions
        .support_reactions
        .sort_by(|left, right| left.support_node_id.cmp(&right.support_node_id));
    let mut attachments = Vec::new();
    attach_payload(
        &mut attachments,
        AnalysisAttachmentKind::DesignActions,
        &actions,
        &mut diagnostics,
    );

    let checks: CheckReport = derive_conservative_check_report(&project, &actions);
    let mut inputs = checks.inputs.clone();
    let mut results = checks.results.clone();
    inputs.sort_by_key(|input| serde_json::to_string(input).unwrap_or_default());
    results.sort_by(|left, right| left.id.cmp(&right.id));
    attach_payload(
        &mut attachments,
        AnalysisAttachmentKind::CheckInputs,
        &inputs,
        &mut diagnostics,
    );
    attach_payload(
        &mut attachments,
        AnalysisAttachmentKind::CheckResults,
        &results,
        &mut diagnostics,
    );
    (attachments, diagnostics)
}

fn attach_payload<T: Serialize>(
    attachments: &mut Vec<AnalysisEvidenceAttachment>,
    kind: AnalysisAttachmentKind,
    payload: &T,
    diagnostics: &mut Vec<String>,
) {
    match serde_json::to_string(payload) {
        Ok(payload_json) => attachments.push(AnalysisEvidenceAttachment {
            kind,
            identity: hash_bytes(payload_json.as_bytes()),
            payload_json,
        }),
        Err(error) => {
            diagnostics.push(format!("analysis-attachment.{kind:?}-unavailable: {error}"))
        }
    }
}

fn analysis_project(settings: &AnalysisSettings) -> ProjectFile {
    ProjectFile {
        schema_version: "fraia.analysis.project.v1".into(),
        name: "immutable-analysis-snapshot".into(),
        created_at: "deterministic".into(),
        updated_at: None,
        intent: Intent {
            building_type: "structural-model".into(),
            design_stage: "analysis".into(),
            objective_priority: "traceable-evidence".into(),
            option_count: 0,
            hard_constraints: Vec::new(),
            soft_preferences: Vec::new(),
            search_permissions: SearchPermissions {
                resize_sections: false,
                add_internal_columns: false,
                change_topology: false,
            },
            approval_triggers: Vec::new(),
        },
        requirements: Requirements {
            span_m: 1.0,
            height_m: 1.0,
            gravity_load_kn_per_m: 0.0,
            lateral_load_kn: 0.0,
            max_deflection_ratio: settings.check_limits.max_deflection_ratio,
            max_drift_ratio: settings.check_limits.max_drift_ratio,
            max_utilization: settings.check_limits.max_utilization,
            max_internal_columns: 0,
        },
        unit_profile: Default::default(),
        planning_draft: None,
        files: ProjectFiles {
            planning: String::new(),
        },
        builder_graph: None,
        legacy_builder_instance: None,
        agent_state: Default::default(),
        base_model_brief: None,
        structural_model: None,
        design_option_decisions: Default::default(),
    }
}

fn hash_json(value: &impl Serialize) -> Result<String, SnapshotAnalysisError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| SnapshotAnalysisError::Serialization(error.to_string()))?;
    Ok(hash_bytes(&bytes))
}

fn hash_bytes(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}
