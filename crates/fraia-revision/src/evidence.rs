//! Immutable analysis-evidence manifests and dependency-aware staleness.
//!
//! Evidence is tied to the exact snapshot that produced it. A later revision
//! may still use it only when the dependencies it declares are unchanged or
//! are unaffected by that revision's semantic diff. This is domain data, not
//! UI state, elapsed time, or a mutable status flag.

use crate::diff::{DiffCategory, SemanticDiff};
use crate::{EvidenceId, SnapshotId};
use fraia_core::DeterministicDerivationRequest;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

/// A named, immutable input that an analysis/check actually depended upon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceDependency {
    pub key: String,
    /// Stable content identity for this input, never a timestamp or UI state.
    pub identity: String,
    /// Semantic change categories that can invalidate this dependency when its
    /// identity changes.
    pub invalidated_by: BTreeSet<DiffCategory>,
    /// Some dependencies are independent of model diff categories. A changed
    /// solver/runtime/settings identity must stale the evidence even when the
    /// authored snapshot itself is unchanged.
    #[serde(default)]
    pub identity_change_invalidates: bool,
}

impl EvidenceDependency {
    pub fn new(
        key: impl Into<String>,
        identity: impl Into<String>,
        invalidated_by: impl IntoIterator<Item = DiffCategory>,
    ) -> Self {
        Self {
            key: key.into(),
            identity: identity.into(),
            invalidated_by: invalidated_by.into_iter().collect(),
            identity_change_invalidates: false,
        }
    }

    /// Creates a dependency whose identity change is sufficient to stale the
    /// evidence, without requiring a structural semantic diff category.
    pub fn always_invalidating(key: impl Into<String>, identity: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            identity: identity.into(),
            invalidated_by: BTreeSet::new(),
            identity_change_invalidates: true,
        }
    }
}

/// The result state of one immutable analysis attempt. Staleness is a
/// separate concern calculated from dependencies and the inspected snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisEvidenceStatus {
    Completed,
    Failed,
    Unsupported,
}

/// A deterministic metric row for one resolved load combination.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisComboMetrics {
    pub combo_id: String,
    pub max_utilization: f64,
    pub max_ux_m: f64,
    pub max_uy_m: f64,
    pub max_reaction_n: f64,
}

/// Actual, comparable metrics retained with completed analysis evidence.
/// Values are envelopes over the recorded combination rows, not prose claims.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisMetrics {
    pub combo_metrics: Vec<AnalysisComboMetrics>,
    pub max_utilization: f64,
    pub max_ux_m: f64,
    pub max_uy_m: f64,
    pub max_reaction_n: f64,
}

impl AnalysisMetrics {
    pub fn is_finite(&self) -> bool {
        self.combo_metrics.iter().all(|metrics| {
            metrics.max_utilization.is_finite()
                && metrics.max_ux_m.is_finite()
                && metrics.max_uy_m.is_finite()
                && metrics.max_reaction_n.is_finite()
        }) && self.max_utilization.is_finite()
            && self.max_ux_m.is_finite()
            && self.max_uy_m.is_finite()
            && self.max_reaction_n.is_finite()
    }

    pub fn combo(&self, combo_id: &str) -> Option<&AnalysisComboMetrics> {
        self.combo_metrics
            .iter()
            .find(|metrics| metrics.combo_id == combo_id)
    }
}

/// A typed immutable payload reference attached to analysis evidence. The
/// payload is kept as canonical JSON so appd/API consumers can display or
/// persist it without introducing a dependency on a mutable project object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisEvidenceAttachment {
    pub kind: AnalysisAttachmentKind,
    pub identity: String,
    pub payload_json: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisAttachmentKind {
    DesignActions,
    CheckInputs,
    CheckResults,
}

/// Immutable, exact inputs for a single analysis result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisEvidence {
    id: EvidenceId,
    authored_snapshot_id: SnapshotId,
    resolved_snapshot_id: Option<SnapshotId>,
    dependencies: Vec<EvidenceDependency>,
    analysis_manifest: Option<AnalysisEvidenceManifest>,
}

/// Exact, immutable record of one deterministic analysis attempt. Hashes are
/// deliberately separate from display ids so a caller can verify every input
/// and output without trusting an elapsed-time run label.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisEvidenceManifest {
    /// These fields predate the accessor names but contain exact content
    /// identities. They remain as serialized compatibility fields while the
    /// identity accessors below make their meaning explicit to consumers.
    pub authored_snapshot_hash: String,
    pub resolved_snapshot_hash: Option<String>,
    pub input_hash: Option<String>,
    pub result_hash: Option<String>,
    pub solver_identity: String,
    pub diagnostics: Vec<String>,
    pub status: AnalysisEvidenceStatus,
    pub request: DeterministicDerivationRequest,
    pub runtime_identity: String,
    pub settings_identity: String,
    pub settings_payload: String,
    pub metrics: Option<AnalysisMetrics>,
    pub attachments: Vec<AnalysisEvidenceAttachment>,
}

impl AnalysisEvidence {
    pub fn new(
        id: EvidenceId,
        authored_snapshot_id: SnapshotId,
        resolved_snapshot_id: Option<SnapshotId>,
        dependencies: Vec<EvidenceDependency>,
    ) -> Result<Self, EvidenceError> {
        let mut keys = BTreeSet::new();
        for dependency in &dependencies {
            if dependency.key.is_empty() || dependency.identity.is_empty() {
                return Err(EvidenceError::EmptyDependencyField);
            }
            if !keys.insert(dependency.key.clone()) {
                return Err(EvidenceError::DuplicateDependencyKey(
                    dependency.key.clone(),
                ));
            }
        }
        Ok(Self {
            id,
            authored_snapshot_id,
            resolved_snapshot_id,
            dependencies,
            analysis_manifest: None,
        })
    }

    /// Constructs evidence with the immutable deterministic run manifest that
    /// produced it. The supplied authored/resolved identities must agree with
    /// the evidence binding; callers cannot attach a result to another model.
    pub fn with_analysis_manifest(
        id: EvidenceId,
        authored_snapshot_id: SnapshotId,
        resolved_snapshot_id: Option<SnapshotId>,
        dependencies: Vec<EvidenceDependency>,
        analysis_manifest: AnalysisEvidenceManifest,
    ) -> Result<Self, EvidenceError> {
        if analysis_manifest.authored_snapshot_hash != authored_snapshot_id.as_str() {
            return Err(EvidenceError::ManifestAuthoredSnapshotMismatch);
        }
        if analysis_manifest.resolved_snapshot_hash.as_deref()
            != resolved_snapshot_id.as_ref().map(SnapshotId::as_str)
        {
            return Err(EvidenceError::ManifestResolvedSnapshotMismatch);
        }
        if analysis_manifest.solver_identity.is_empty()
            || analysis_manifest.runtime_identity.is_empty()
            || analysis_manifest.settings_identity.is_empty()
            || analysis_manifest.settings_payload.is_empty()
        {
            return Err(EvidenceError::EmptyAnalysisExecutionIdentity);
        }
        if sha256_identity(analysis_manifest.settings_payload.as_bytes())
            != analysis_manifest.settings_identity
        {
            return Err(EvidenceError::SettingsIdentityMismatch);
        }

        let mut attachment_kinds = BTreeSet::new();
        for attachment in &analysis_manifest.attachments {
            if attachment.identity.is_empty() || attachment.payload_json.is_empty() {
                return Err(EvidenceError::EmptyAttachmentField);
            }
            if !attachment_kinds.insert(attachment.kind) {
                return Err(EvidenceError::DuplicateAttachmentKind(attachment.kind));
            }
            if sha256_identity(attachment.payload_json.as_bytes()) != attachment.identity {
                return Err(EvidenceError::AttachmentIdentityMismatch(attachment.kind));
            }
        }

        match analysis_manifest.status {
            AnalysisEvidenceStatus::Completed => {
                if analysis_manifest.resolved_snapshot_hash.is_none()
                    || analysis_manifest.input_hash.is_none()
                    || analysis_manifest.result_hash.is_none()
                    || analysis_manifest.metrics.is_none()
                {
                    return Err(EvidenceError::CompletedWithoutResultIdentity);
                }
                if !analysis_manifest
                    .metrics
                    .as_ref()
                    .is_some_and(AnalysisMetrics::is_finite)
                {
                    return Err(EvidenceError::NonFiniteMetrics);
                }
            }
            AnalysisEvidenceStatus::Failed => {
                if analysis_manifest.result_hash.is_some()
                    || analysis_manifest.metrics.is_some()
                    || !analysis_manifest.attachments.is_empty()
                {
                    return Err(EvidenceError::FailedWithResultIdentity);
                }
            }
            AnalysisEvidenceStatus::Unsupported => {
                if analysis_manifest.resolved_snapshot_hash.is_some()
                    || analysis_manifest.result_hash.is_some()
                    || analysis_manifest.metrics.is_some()
                    || !analysis_manifest.attachments.is_empty()
                {
                    return Err(EvidenceError::UnsupportedWithResultIdentity);
                }
            }
        }
        let mut evidence = Self::new(id, authored_snapshot_id, resolved_snapshot_id, dependencies)?;
        evidence.analysis_manifest = Some(analysis_manifest);
        Ok(evidence)
    }

    pub fn id(&self) -> &EvidenceId {
        &self.id
    }

    pub fn authored_snapshot_id(&self) -> &SnapshotId {
        &self.authored_snapshot_id
    }

    pub fn resolved_snapshot_id(&self) -> Option<&SnapshotId> {
        self.resolved_snapshot_id.as_ref()
    }

    pub fn dependencies(&self) -> &[EvidenceDependency] {
        &self.dependencies
    }

    pub fn analysis_manifest(&self) -> Option<&AnalysisEvidenceManifest> {
        self.analysis_manifest.as_ref()
    }

    pub fn analysis_status(&self) -> Option<AnalysisEvidenceStatus> {
        self.analysis_manifest
            .as_ref()
            .map(|manifest| manifest.status)
    }

    pub fn authored_snapshot_identity(&self) -> &str {
        self.analysis_manifest
            .as_ref()
            .map(AnalysisEvidenceManifest::authored_snapshot_identity)
            .unwrap_or_else(|| self.authored_snapshot_id.as_str())
    }

    pub fn resolved_snapshot_identity(&self) -> Option<&str> {
        self.analysis_manifest
            .as_ref()
            .and_then(AnalysisEvidenceManifest::resolved_snapshot_identity)
    }

    pub fn input_identity(&self) -> Option<&str> {
        self.analysis_manifest
            .as_ref()
            .and_then(AnalysisEvidenceManifest::input_identity)
    }

    pub fn result_identity(&self) -> Option<&str> {
        self.analysis_manifest
            .as_ref()
            .and_then(AnalysisEvidenceManifest::result_identity)
    }

    pub fn solver_identity(&self) -> Option<&str> {
        self.analysis_manifest
            .as_ref()
            .map(AnalysisEvidenceManifest::solver_identity)
    }

    pub fn runtime_identity(&self) -> Option<&str> {
        self.analysis_manifest
            .as_ref()
            .map(AnalysisEvidenceManifest::runtime_identity)
    }

    pub fn settings_identity(&self) -> Option<&str> {
        self.analysis_manifest
            .as_ref()
            .map(AnalysisEvidenceManifest::settings_identity)
    }

    pub fn metrics(&self) -> Option<&AnalysisMetrics> {
        self.analysis_manifest
            .as_ref()
            .and_then(AnalysisEvidenceManifest::metrics)
    }

    pub fn attachments(&self) -> &[AnalysisEvidenceAttachment] {
        self.analysis_manifest
            .as_ref()
            .map_or(&[], AnalysisEvidenceManifest::attachments)
    }
}

impl AnalysisEvidenceManifest {
    pub fn authored_snapshot_identity(&self) -> &str {
        &self.authored_snapshot_hash
    }

    pub fn resolved_snapshot_identity(&self) -> Option<&str> {
        self.resolved_snapshot_hash.as_deref()
    }

    pub fn input_identity(&self) -> Option<&str> {
        self.input_hash.as_deref()
    }

    pub fn result_identity(&self) -> Option<&str> {
        self.result_hash.as_deref()
    }

    pub fn solver_identity(&self) -> &str {
        &self.solver_identity
    }

    pub fn runtime_identity(&self) -> &str {
        &self.runtime_identity
    }

    pub fn settings_identity(&self) -> &str {
        &self.settings_identity
    }

    pub fn settings_payload(&self) -> &str {
        &self.settings_payload
    }

    pub fn metrics(&self) -> Option<&AnalysisMetrics> {
        self.metrics.as_ref()
    }

    pub fn attachments(&self) -> &[AnalysisEvidenceAttachment] {
        &self.attachments
    }

    pub fn attachment(&self, kind: AnalysisAttachmentKind) -> Option<&AnalysisEvidenceAttachment> {
        self.attachments
            .iter()
            .find(|attachment| attachment.kind == kind)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceError {
    EmptyDependencyField,
    DuplicateDependencyKey(String),
    ManifestAuthoredSnapshotMismatch,
    ManifestResolvedSnapshotMismatch,
    EmptyAnalysisExecutionIdentity,
    SettingsIdentityMismatch,
    EmptyAttachmentField,
    DuplicateAttachmentKind(AnalysisAttachmentKind),
    AttachmentIdentityMismatch(AnalysisAttachmentKind),
    CompletedWithoutResultIdentity,
    FailedWithResultIdentity,
    UnsupportedWithResultIdentity,
    NonFiniteMetrics,
}

impl fmt::Display for EvidenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyDependencyField => {
                formatter.write_str("evidence dependency key and identity must not be empty")
            }
            Self::DuplicateDependencyKey(key) => {
                write!(formatter, "duplicate evidence dependency key `{key}`")
            }
            Self::ManifestAuthoredSnapshotMismatch => {
                formatter.write_str("analysis manifest authored snapshot does not match evidence")
            }
            Self::ManifestResolvedSnapshotMismatch => {
                formatter.write_str("analysis manifest resolved snapshot does not match evidence")
            }
            Self::EmptyAnalysisExecutionIdentity => {
                formatter.write_str("analysis solver/runtime/settings identities must not be empty")
            }
            Self::SettingsIdentityMismatch => {
                formatter.write_str("analysis settings identity does not match its payload")
            }
            Self::EmptyAttachmentField => {
                formatter.write_str(
                    "analysis evidence attachment identity and payload must not be empty",
                )
            }
            Self::DuplicateAttachmentKind(kind) => {
                write!(formatter, "duplicate analysis evidence attachment kind {kind:?}")
            }
            Self::AttachmentIdentityMismatch(kind) => {
                write!(
                    formatter,
                    "analysis evidence attachment {kind:?} identity does not match its payload"
                )
            }
            Self::CompletedWithoutResultIdentity => formatter.write_str(
                "completed analysis evidence must include resolved, input, result, and metric identities",
            ),
            Self::FailedWithResultIdentity => formatter.write_str(
                "failed analysis evidence must not include a successful result identity",
            ),
            Self::UnsupportedWithResultIdentity => formatter.write_str(
                "unsupported analysis evidence must not include a resolved or result identity",
            ),
            Self::NonFiniteMetrics => {
                formatter.write_str("completed analysis evidence metrics must be finite")
            }
        }
    }
}

impl Error for EvidenceError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceStaleness {
    Current,
    Stale { reasons: Vec<StaleEvidenceReason> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StaleEvidenceReason {
    MissingDependency {
        key: String,
    },
    ChangedDependency {
        key: String,
        expected_identity: String,
        actual_identity: String,
        affected_categories: BTreeSet<DiffCategory>,
    },
}

impl EvidenceStaleness {
    pub fn is_stale(&self) -> bool {
        matches!(self, Self::Stale { .. })
    }
}

/// Calculates whether immutable evidence remains applicable at a later model
/// snapshot. If the inspected snapshot is the exact input, evidence is always
/// current. Otherwise, an identity must have changed *and* its declared
/// invalidation categories must intersect the recorded semantic diff.
pub fn staleness_for(
    evidence: &AnalysisEvidence,
    inspected_snapshot_id: &SnapshotId,
    current_dependencies: &[EvidenceDependency],
    change_from_evidence_snapshot: &SemanticDiff,
) -> EvidenceStaleness {
    if inspected_snapshot_id == evidence.authored_snapshot_id() && current_dependencies.is_empty() {
        // A caller may intentionally omit current dependency material when it
        // only wants to ask whether the exact authored snapshot is current.
        // Once dependencies are supplied, pinned execution dependencies must
        // also be checked for that exact snapshot.
        return EvidenceStaleness::Current;
    }

    let current_by_key: BTreeMap<_, _> = current_dependencies
        .iter()
        .map(|dependency| (dependency.key.as_str(), dependency))
        .collect();
    let changed_categories: BTreeSet<_> = change_from_evidence_snapshot
        .changes
        .iter()
        .flat_map(|change| change.categories.iter().copied())
        .collect();
    let mut reasons = Vec::new();

    for dependency in evidence.dependencies() {
        let Some(current) = current_by_key.get(dependency.key.as_str()) else {
            reasons.push(StaleEvidenceReason::MissingDependency {
                key: dependency.key.clone(),
            });
            continue;
        };
        if current.identity != dependency.identity {
            let affected_categories: BTreeSet<_> = dependency
                .invalidated_by
                .intersection(&changed_categories)
                .copied()
                .collect();
            if !affected_categories.is_empty() || dependency.identity_change_invalidates {
                reasons.push(StaleEvidenceReason::ChangedDependency {
                    key: dependency.key.clone(),
                    expected_identity: dependency.identity.clone(),
                    actual_identity: current.identity.clone(),
                    affected_categories,
                });
            }
        }
    }

    if reasons.is_empty() {
        EvidenceStaleness::Current
    } else {
        EvidenceStaleness::Stale { reasons }
    }
}

#[cfg(test)]
mod tests {
    use super::{AnalysisEvidence, EvidenceDependency, EvidenceStaleness, staleness_for};
    use crate::diff::{DiffCategory, semantic_diff};
    use crate::{EvidenceId, SnapshotId, root_fixture};

    #[test]
    fn affected_dependencies_become_stale_while_unaffected_evidence_stays_current() {
        let fixture = root_fixture();
        let mut changed_model = fixture.model.clone();
        changed_model.supports[1].ux = true;
        let diff = semantic_diff(&fixture.model, &changed_model);
        let source = SnapshotId::from("snapshot-a");
        let current = SnapshotId::from("snapshot-b");

        let support_evidence = AnalysisEvidence::new(
            EvidenceId::from("support-run"),
            source.clone(),
            None,
            vec![EvidenceDependency::new(
                "support-restraints",
                "supports:a",
                [DiffCategory::Support],
            )],
        )
        .unwrap();
        let load_evidence = AnalysisEvidence::new(
            EvidenceId::from("load-run"),
            source,
            None,
            vec![EvidenceDependency::new(
                "gravity-loads",
                "loads:a",
                [DiffCategory::Load],
            )],
        )
        .unwrap();
        let current_dependencies = vec![
            EvidenceDependency::new("support-restraints", "supports:b", [DiffCategory::Support]),
            EvidenceDependency::new("gravity-loads", "loads:a", [DiffCategory::Load]),
        ];

        assert!(
            staleness_for(&support_evidence, &current, &current_dependencies, &diff).is_stale()
        );
        assert_eq!(
            staleness_for(&load_evidence, &current, &current_dependencies, &diff),
            EvidenceStaleness::Current
        );
    }

    #[test]
    fn exact_input_snapshot_is_current_without_ui_or_time_state() {
        let evidence = AnalysisEvidence::new(
            EvidenceId::from("run"),
            SnapshotId::from("snapshot-a"),
            Some(SnapshotId::from("resolved-a")),
            vec![EvidenceDependency::new("solver", "calculix:1", [])],
        )
        .unwrap();
        assert_eq!(
            staleness_for(
                &evidence,
                &SnapshotId::from("snapshot-a"),
                &[],
                &Default::default()
            ),
            EvidenceStaleness::Current
        );
    }

    #[test]
    fn always_invalidating_dependency_stales_even_without_a_model_diff() {
        let source = SnapshotId::from("snapshot-a");
        let evidence = AnalysisEvidence::new(
            EvidenceId::from("run"),
            source.clone(),
            None,
            vec![EvidenceDependency::always_invalidating(
                "solver-runtime",
                "runtime:v1",
            )],
        )
        .unwrap();

        let stale = staleness_for(
            &evidence,
            &source,
            &[EvidenceDependency::always_invalidating(
                "solver-runtime",
                "runtime:v2",
            )],
            &Default::default(),
        );

        assert!(stale.is_stale());
    }
}

fn sha256_identity(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}
