use crate::RevisionId;
use crate::evidence::{AnalysisAttachmentKind, AnalysisEvidence, AnalysisEvidenceStatus};
use fraia_core::{
    DesignId, DesignRunActor, DesignRunAttachmentInput, DesignRunAttachmentRole,
    DesignRunDiagnostic, DesignRunDiagnosticSeverity, DesignRunManifest, DesignRunStatus,
    ProjectId, PublishDesignRunRequest, publish_design_run,
};
use serde_json::Value;
use std::path::Path;

#[derive(Debug)]
pub enum DesignRunAdapterError {
    MissingManifest,
    InvalidSettingsPayload(serde_json::Error),
    Serialization(serde_json::Error),
    Store(fraia_core::DesignRunStoreError),
    EvidenceBinding(String),
}

impl std::fmt::Display for DesignRunAdapterError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingManifest => {
                formatter.write_str("analysis evidence has no immutable manifest")
            }
            Self::InvalidSettingsPayload(error) => {
                write!(formatter, "analysis settings payload is invalid: {error}")
            }
            Self::Serialization(error) => write!(
                formatter,
                "analysis evidence could not be serialized: {error}"
            ),
            Self::Store(error) => write!(formatter, "{error}"),
            Self::EvidenceBinding(error) => {
                write!(
                    formatter,
                    "analysis evidence could not bind its canonical run: {error}"
                )
            }
        }
    }
}
impl std::error::Error for DesignRunAdapterError {}

pub struct PublishAnalysisEvidenceDesignRun<'a> {
    pub project_dir: &'a Path,
    pub project_id: ProjectId,
    pub design_id: DesignId,
    pub revision_id: &'a RevisionId,
    pub evidence: &'a mut AnalysisEvidence,
    pub actor: DesignRunActor,
    pub created_at: String,
    pub parent_run_id: Option<String>,
}

pub fn publish_analysis_evidence_design_run(
    publication: PublishAnalysisEvidenceDesignRun<'_>,
) -> Result<DesignRunManifest, DesignRunAdapterError> {
    let manifest = publication
        .evidence
        .analysis_manifest()
        .ok_or(DesignRunAdapterError::MissingManifest)?;
    let status = match manifest.status {
        AnalysisEvidenceStatus::Completed => DesignRunStatus::Completed,
        AnalysisEvidenceStatus::Failed => DesignRunStatus::Failed,
        AnalysisEvidenceStatus::Unsupported => DesignRunStatus::Unsupported,
    };
    let severity = match status {
        DesignRunStatus::Completed => DesignRunDiagnosticSeverity::Information,
        DesignRunStatus::Failed => DesignRunDiagnosticSeverity::Error,
        DesignRunStatus::Unsupported => DesignRunDiagnosticSeverity::Warning,
    };
    let diagnostics = manifest
        .diagnostics
        .iter()
        .enumerate()
        .map(|(index, message)| DesignRunDiagnostic {
            severity,
            code: format!("analysis.diagnostic.{index}"),
            message: message.clone(),
        })
        .collect();
    let attachments = manifest
        .attachments
        .iter()
        .map(|attachment| DesignRunAttachmentInput {
            name: match attachment.kind {
                AnalysisAttachmentKind::DesignActions => "design-actions.json",
                AnalysisAttachmentKind::CheckInputs => "check-inputs.json",
                AnalysisAttachmentKind::CheckResults => "check-results.json",
            }
            .into(),
            role: match attachment.kind {
                AnalysisAttachmentKind::DesignActions => DesignRunAttachmentRole::DesignActions,
                AnalysisAttachmentKind::CheckInputs => DesignRunAttachmentRole::CheckInputs,
                AnalysisAttachmentKind::CheckResults => DesignRunAttachmentRole::CheckResults,
            },
            media_type: "application/json".into(),
            bytes: attachment.payload_json.as_bytes().to_vec(),
        })
        .collect();
    let settings: Value = serde_json::from_str(&manifest.settings_payload)
        .map_err(DesignRunAdapterError::InvalidSettingsPayload)?;
    let interpretation_dependencies = publication.evidence.interpretation_dependencies();
    let request = serde_json::json!({
        "analysis": &manifest.request,
        "evidenceId": publication.evidence.id().to_string(),
        "interpretationDependencies": interpretation_dependencies,
    });
    let metrics = manifest
        .metrics
        .as_ref()
        .map(serde_json::to_value)
        .transpose()
        .map_err(DesignRunAdapterError::Serialization)?;
    let published = publish_design_run(
        publication.project_dir,
        PublishDesignRunRequest {
            project_id: publication.project_id,
            design_id: publication.design_id,
            parent_run_id: publication.parent_run_id,
            created_at: publication.created_at,
            actor: publication.actor,
            run_kind: "snapshot_analysis".into(),
            authored_revision_id: publication.revision_id.to_string(),
            authored_snapshot_id: publication.evidence.authored_snapshot_id().to_string(),
            resolved_snapshot_id: publication
                .evidence
                .resolved_snapshot_id()
                .map(ToString::to_string),
            request,
            settings,
            solver_identity: manifest.solver_identity.clone(),
            runtime_identity: manifest.runtime_identity.clone(),
            input_identity: manifest.input_hash.clone(),
            result_identity: manifest.result_hash.clone(),
            status,
            diagnostics,
            metrics,
            attachments,
        },
    )
    .map_err(DesignRunAdapterError::Store)?;
    publication
        .evidence
        .bind_canonical_run_id(published.run_id.clone())
        .map_err(|error| DesignRunAdapterError::EvidenceBinding(error.to_string()))?;
    Ok(published)
}
