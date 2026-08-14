use crate::{
    ConfirmedAlignmentTransform, ConflictResolution, CrossViewCorrespondence, DesignId,
    DrawingInterpretation, DrawingInterpretationError, DrawingInterpretationRevision,
    DrawingObservation, DrawingSourceLocator, InterpretationConflict, InterpretationMethod,
    ObservationConfirmation, ObservationDesignGeometry, ObservationExtraction, ObservationFeature,
    ObservationSourceGeometry, ProjectId, ShelfItemContent, ShelfSourceRef, SourceId,
    design_package_paths, load_design_shelf, load_project_package,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub const DRAWING_INTERPRETATION_INDEX_SCHEMA_VERSION: &str =
    "fraia.drawing-interpretation-index.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterpretationCreateAuthority {
    User,
    ParserAdapter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrawingInterpretationRevisionRef {
    pub revision_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_revision_id: Option<String>,
    pub created_at: String,
    pub observation_count: usize,
    pub unresolved_conflict_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrawingInterpretationList {
    pub project_id: ProjectId,
    pub design_id: DesignId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head_revision_id: Option<String>,
    pub revisions: Vec<DrawingInterpretationRevisionRef>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmObservationsOperation {
    pub expected_parent_revision_id: String,
    pub observation_ids: Vec<String>,
    pub confirmed_by: String,
    pub confirmed_at: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconcileInterpretationOperation {
    pub expected_parent_revision_id: String,
    pub design_geometries: BTreeMap<String, ObservationDesignGeometry>,
    pub correspondences: BTreeMap<String, CrossViewCorrespondence>,
    pub alignment_transforms: BTreeMap<String, ConfirmedAlignmentTransform>,
    pub conflicts: BTreeMap<String, InterpretationConflict>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveInterpretationConflictOperation {
    pub expected_parent_revision_id: String,
    pub conflict_id: String,
    pub resolution: String,
    pub resolved_by: String,
    pub resolved_at: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorrectInterpretationObservationOperation {
    pub expected_parent_revision_id: String,
    pub observation_id: String,
    pub corrected_view_role: Option<crate::DrawingViewRole>,
    pub corrected_feature: Option<ObservationFeature>,
    pub corrected_by: String,
    pub corrected_at: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentInterpretationContext {
    pub project_id: ProjectId,
    pub design_id: DesignId,
    pub revision_id: String,
    pub confirmed_constraints: Vec<AgentConfirmedInterpretationConstraint>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inferred_assumptions: Vec<AgentInferredInterpretationAssumption>,
    pub unresolved_conflicts: Vec<InterpretationConflict>,
    pub unconfirmed_observation_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentInferredInterpretationAssumption {
    pub inference_id: String,
    pub interpretation_revision_id: String,
    pub observation_id: String,
    pub shelf_item_id: String,
    pub source_id: SourceId,
    pub source_sha256: String,
    pub source_locator: DrawingSourceLocator,
    pub extraction: ObservationExtraction,
    pub feature: ObservationFeature,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub design_geometry: Option<ObservationDesignGeometry>,
    pub materially_conflicted: bool,
    pub requires_confirmation: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentConfirmedInterpretationConstraint {
    pub observation_id: String,
    pub shelf_item_id: String,
    pub source_id: SourceId,
    pub source_sha256: String,
    pub source_locator: DrawingSourceLocator,
    pub source_geometry: ObservationSourceGeometry,
    pub design_geometry: ObservationDesignGeometry,
    pub feature: ObservationFeature,
}

#[derive(Debug)]
pub enum DrawingInterpretationStoreError {
    Invalid(String),
    NotFound(String),
    ParentConflict {
        expected: Option<String>,
        actual: Option<String>,
    },
    Domain(DrawingInterpretationError),
    Io(std::io::Error),
    Json(serde_json::Error),
    Package(String),
}

impl std::fmt::Display for DrawingInterpretationStoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(message) | Self::Package(message) => formatter.write_str(message),
            Self::NotFound(id) => write!(formatter, "drawing interpretation `{id}` was not found"),
            Self::ParentConflict { expected, actual } => write!(
                formatter,
                "drawing interpretation head changed: expected {:?}, actual {:?}",
                expected, actual
            ),
            Self::Domain(error) => write!(formatter, "{error}"),
            Self::Io(error) => write!(formatter, "{error}"),
            Self::Json(error) => write!(formatter, "{error}"),
        }
    }
}
impl std::error::Error for DrawingInterpretationStoreError {}
impl From<std::io::Error> for DrawingInterpretationStoreError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
impl From<serde_json::Error> for DrawingInterpretationStoreError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}
impl From<DrawingInterpretationError> for DrawingInterpretationStoreError {
    fn from(value: DrawingInterpretationError) -> Self {
        Self::Domain(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct InterpretationIndex {
    schema_version: String,
    project_id: ProjectId,
    design_id: DesignId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    head_revision_id: Option<String>,
    revisions: BTreeMap<String, DrawingInterpretationRevisionRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StoreCheckpoint {
    RevisionPublished,
    IndexStaged,
    IndexBackedUp,
}

pub fn list_drawing_interpretations(
    project_dir: &Path,
    design_id: &DesignId,
) -> Result<DrawingInterpretationList, DrawingInterpretationStoreError> {
    let (project_id, paths) = owning_design(project_dir, design_id)?;
    let index = load_index(&paths, project_id.clone(), design_id.clone())?;
    for (revision_id, reference) in &index.revisions {
        let revision = load_revision_file(&paths, revision_id, &project_id, design_id)?;
        if revision_ref(&revision) != *reference {
            return Err(DrawingInterpretationStoreError::Invalid(
                "interpretation index metadata does not match its immutable revision".into(),
            ));
        }
    }
    let mut revision_ids = Vec::with_capacity(index.revisions.len());
    let mut cursor = index.head_revision_id.as_ref();
    while let Some(revision_id) = cursor {
        revision_ids.push(revision_id.clone());
        cursor = index.revisions[revision_id].parent_revision_id.as_ref();
    }
    revision_ids.reverse();
    let revisions = revision_ids
        .into_iter()
        .map(|revision_id| index.revisions[&revision_id].clone())
        .collect();
    Ok(DrawingInterpretationList {
        project_id,
        design_id: design_id.clone(),
        head_revision_id: index.head_revision_id,
        revisions,
    })
}

pub fn load_drawing_interpretation(
    project_dir: &Path,
    design_id: &DesignId,
    revision_id: &str,
) -> Result<DrawingInterpretation, DrawingInterpretationStoreError> {
    validate_revision_id(revision_id)?;
    let (project_id, paths) = owning_design(project_dir, design_id)?;
    let index = load_index(&paths, project_id.clone(), design_id.clone())?;
    if !index.revisions.contains_key(revision_id) {
        return Err(DrawingInterpretationStoreError::NotFound(
            revision_id.into(),
        ));
    }
    load_revision_file(&paths, revision_id, &project_id, design_id)
}

pub fn load_head_drawing_interpretation(
    project_dir: &Path,
    design_id: &DesignId,
) -> Result<Option<DrawingInterpretation>, DrawingInterpretationStoreError> {
    let list = list_drawing_interpretations(project_dir, design_id)?;
    list.head_revision_id
        .as_deref()
        .map(|id| load_drawing_interpretation(project_dir, design_id, id))
        .transpose()
}

pub fn drawing_interpretation_shelf_references(
    project_dir: &Path,
    design_id: &DesignId,
    shelf_item_id: &str,
) -> Result<Vec<String>, DrawingInterpretationStoreError> {
    let list = list_drawing_interpretations(project_dir, design_id)?;
    let mut revisions = Vec::new();
    for revision in list.revisions {
        let interpretation =
            load_drawing_interpretation(project_dir, design_id, &revision.revision_id)?;
        if interpretation
            .observations
            .values()
            .any(|observation| observation.shelf_item_id == shelf_item_id)
        {
            revisions.push(revision.revision_id);
        }
    }
    Ok(revisions)
}

pub fn create_drawing_interpretation(
    project_dir: &Path,
    design_id: &DesignId,
    expected_parent_revision_id: Option<&str>,
    authority: InterpretationCreateAuthority,
    revision: DrawingInterpretationRevision,
) -> Result<DrawingInterpretation, DrawingInterpretationStoreError> {
    create_with_hook(
        project_dir,
        design_id,
        expected_parent_revision_id,
        authority,
        revision,
        |_| Ok(()),
    )
}

fn create_with_hook<F>(
    project_dir: &Path,
    design_id: &DesignId,
    expected_parent_revision_id: Option<&str>,
    authority: InterpretationCreateAuthority,
    mut revision: DrawingInterpretationRevision,
    mut hook: F,
) -> Result<DrawingInterpretation, DrawingInterpretationStoreError>
where
    F: FnMut(StoreCheckpoint) -> Result<(), DrawingInterpretationStoreError>,
{
    let (project_id, paths) = owning_design(project_dir, design_id)?;
    if revision.project_id != project_id || &revision.design_id != design_id {
        return Err(DrawingInterpretationStoreError::Invalid(
            "interpretation ownership does not match the package".into(),
        ));
    }
    let expected = expected_parent_revision_id.map(str::to_owned);
    if revision.parent_revision_id != expected {
        return Err(DrawingInterpretationStoreError::Invalid(
            "revision parent must equal the compare-and-swap expected parent".into(),
        ));
    }
    if authority == InterpretationCreateAuthority::ParserAdapter
        && revision.observations.values().any(|observation| {
            !matches!(
                observation.confirmation,
                ObservationConfirmation::Unconfirmed
            )
        })
    {
        return Err(DrawingInterpretationStoreError::Invalid(
            "parser adapters may create unconfirmed observations only".into(),
        ));
    }
    fs::create_dir_all(&paths.interpretations_dir)?;
    reject_symlink(&paths.interpretations_dir)?;
    let mut index = load_index(&paths, project_id.clone(), design_id.clone())?;
    if index.head_revision_id != expected {
        return Err(DrawingInterpretationStoreError::ParentConflict {
            expected,
            actual: index.head_revision_id,
        });
    }
    if authority == InterpretationCreateAuthority::ParserAdapter {
        if let Some(parent_id) = expected_parent_revision_id {
            let parent = load_revision_file(&paths, parent_id, &project_id, design_id)?;
            let mut observations = parent.observations;
            observations.extend(revision.observations);
            revision.observations = observations;
            let mut correspondences = parent.correspondences;
            correspondences.extend(revision.correspondences);
            revision.correspondences = correspondences;
            let mut transforms = parent.alignment_transforms;
            transforms.extend(revision.alignment_transforms);
            revision.alignment_transforms = transforms;
            let mut conflicts = parent.conflicts;
            conflicts.extend(revision.conflicts);
            revision.conflicts = conflicts;
        }
    }
    validate_shelf_references(project_dir, design_id, revision.observations.values())?;
    let interpretation = DrawingInterpretation::new(revision)?;
    publish_revision(&paths, &interpretation)?;
    hook(StoreCheckpoint::RevisionPublished)?;
    index.head_revision_id = Some(interpretation.revision_id.clone());
    index.revisions.insert(
        interpretation.revision_id.clone(),
        revision_ref(&interpretation),
    );
    save_index_with_hook(&paths, &index, &mut hook)?;
    Ok(interpretation)
}

pub fn confirm_drawing_observations(
    project_dir: &Path,
    design_id: &DesignId,
    operation: ConfirmObservationsOperation,
) -> Result<DrawingInterpretation, DrawingInterpretationStoreError> {
    if operation.observation_ids.is_empty() {
        return Err(DrawingInterpretationStoreError::Invalid(
            "confirmation requires observation ids".into(),
        ));
    }
    let mut parent = load_drawing_interpretation(
        project_dir,
        design_id,
        &operation.expected_parent_revision_id,
    )?;
    for id in operation.observation_ids {
        let observation = parent
            .observations
            .get_mut(&id)
            .ok_or_else(|| DrawingInterpretationStoreError::NotFound(id.clone()))?;
        observation.confirmation = ObservationConfirmation::Confirmed {
            confirmed_by: operation.confirmed_by.clone(),
            confirmed_at: operation.confirmed_at.clone(),
        };
    }
    persist_child(
        project_dir,
        design_id,
        parent,
        operation.expected_parent_revision_id,
        operation.created_at,
        InterpretationMethod::Manual,
    )
}

pub fn correct_drawing_observation(
    project_dir: &Path,
    design_id: &DesignId,
    operation: CorrectInterpretationObservationOperation,
) -> Result<DrawingInterpretation, DrawingInterpretationStoreError> {
    if operation.corrected_by.trim().is_empty()
        || (operation.corrected_view_role.is_none() && operation.corrected_feature.is_none())
    {
        return Err(DrawingInterpretationStoreError::Invalid(
            "correction requires an actor and a changed role or feature".into(),
        ));
    }
    let mut parent = load_drawing_interpretation(
        project_dir,
        design_id,
        &operation.expected_parent_revision_id,
    )?;
    let observation = parent
        .observations
        .get_mut(&operation.observation_id)
        .ok_or_else(|| {
            DrawingInterpretationStoreError::NotFound(operation.observation_id.clone())
        })?;
    if let Some(role) = operation.corrected_view_role {
        observation.view_role = role;
    }
    if let Some(feature) = operation.corrected_feature {
        observation.feature = feature;
    }
    observation.confirmation = ObservationConfirmation::Confirmed {
        confirmed_by: operation.corrected_by,
        confirmed_at: operation.corrected_at,
    };
    create_drawing_interpretation(
        project_dir,
        design_id,
        Some(&operation.expected_parent_revision_id),
        InterpretationCreateAuthority::User,
        DrawingInterpretationRevision {
            project_id: parent.project_id,
            design_id: parent.design_id,
            parent_revision_id: Some(operation.expected_parent_revision_id.clone()),
            created_at: operation.created_at,
            method: InterpretationMethod::Manual,
            observations: parent.observations,
            correspondences: parent.correspondences,
            alignment_transforms: parent.alignment_transforms,
            conflicts: parent.conflicts,
        },
    )
}

pub fn reconcile_drawing_interpretation(
    project_dir: &Path,
    design_id: &DesignId,
    operation: ReconcileInterpretationOperation,
) -> Result<DrawingInterpretation, DrawingInterpretationStoreError> {
    let mut parent = load_drawing_interpretation(
        project_dir,
        design_id,
        &operation.expected_parent_revision_id,
    )?;
    for (observation_id, geometry) in operation.design_geometries {
        let observation = parent
            .observations
            .get_mut(&observation_id)
            .ok_or_else(|| DrawingInterpretationStoreError::NotFound(observation_id.clone()))?;
        observation.design_geometry = Some(geometry);
    }
    parent.correspondences.extend(operation.correspondences);
    parent
        .alignment_transforms
        .extend(operation.alignment_transforms);
    parent.conflicts.extend(operation.conflicts);
    persist_child(
        project_dir,
        design_id,
        parent,
        operation.expected_parent_revision_id,
        operation.created_at,
        InterpretationMethod::Reconciled,
    )
}

pub fn resolve_drawing_interpretation_conflict(
    project_dir: &Path,
    design_id: &DesignId,
    operation: ResolveInterpretationConflictOperation,
) -> Result<DrawingInterpretation, DrawingInterpretationStoreError> {
    let mut parent = load_drawing_interpretation(
        project_dir,
        design_id,
        &operation.expected_parent_revision_id,
    )?;
    let conflict = parent
        .conflicts
        .get_mut(&operation.conflict_id)
        .ok_or_else(|| DrawingInterpretationStoreError::NotFound(operation.conflict_id.clone()))?;
    conflict.resolution = ConflictResolution::Resolved {
        resolution: operation.resolution,
        resolved_by: operation.resolved_by,
        resolved_at: operation.resolved_at,
    };
    persist_child(
        project_dir,
        design_id,
        parent,
        operation.expected_parent_revision_id,
        operation.created_at,
        InterpretationMethod::Reconciled,
    )
}

fn persist_child(
    project_dir: &Path,
    design_id: &DesignId,
    parent: DrawingInterpretation,
    parent_id: String,
    created_at: String,
    method: InterpretationMethod,
) -> Result<DrawingInterpretation, DrawingInterpretationStoreError> {
    create_drawing_interpretation(
        project_dir,
        design_id,
        Some(&parent_id),
        InterpretationCreateAuthority::User,
        DrawingInterpretationRevision {
            project_id: parent.project_id,
            design_id: parent.design_id,
            parent_revision_id: Some(parent_id.clone()),
            created_at,
            method,
            observations: parent.observations,
            correspondences: parent.correspondences,
            alignment_transforms: parent.alignment_transforms,
            conflicts: parent.conflicts,
        },
    )
}

pub fn drawing_interpretation_agent_context(
    project_dir: &Path,
    design_id: &DesignId,
    revision_id: &str,
) -> Result<AgentInterpretationContext, DrawingInterpretationStoreError> {
    let interpretation = load_drawing_interpretation(project_dir, design_id, revision_id)?;
    let mut confirmed_constraints = Vec::new();
    let mut inferred_assumptions = Vec::new();
    let mut unconfirmed = Vec::new();
    for observation in interpretation.observations.values() {
        if let Ok(constraint) = interpretation.confirmed_proposal_constraint(&observation.id) {
            confirmed_constraints.push(AgentConfirmedInterpretationConstraint {
                observation_id: observation.id.clone(),
                shelf_item_id: observation.shelf_item_id.clone(),
                source_id: observation.source_id.clone(),
                source_sha256: observation.source_sha256.clone(),
                source_locator: observation.source_locator.clone(),
                source_geometry: observation.source_geometry.clone(),
                design_geometry: constraint.design_geometry.clone(),
                feature: observation.feature.clone(),
            });
        } else if matches!(
            observation.confirmation,
            ObservationConfirmation::Unconfirmed
        ) {
            let materially_conflicted = interpretation.conflicts.values().any(|conflict| {
                matches!(conflict.resolution, ConflictResolution::Unresolved)
                    && conflict.observation_ids.contains(&observation.id)
            });
            if observation.extraction.confidence >= 0.8 && !materially_conflicted {
                inferred_assumptions.push(AgentInferredInterpretationAssumption {
                    inference_id: format!(
                        "{}:inference:{}",
                        interpretation.revision_id, observation.id
                    ),
                    interpretation_revision_id: interpretation.revision_id.clone(),
                    observation_id: observation.id.clone(),
                    shelf_item_id: observation.shelf_item_id.clone(),
                    source_id: observation.source_id.clone(),
                    source_sha256: observation.source_sha256.clone(),
                    source_locator: observation.source_locator.clone(),
                    extraction: observation.extraction.clone(),
                    feature: observation.feature.clone(),
                    design_geometry: observation.design_geometry.clone(),
                    materially_conflicted,
                    requires_confirmation: true,
                });
            }
            unconfirmed.push(observation.id.clone());
        }
    }
    let unresolved_conflicts = interpretation
        .conflicts
        .values()
        .filter(|conflict| matches!(conflict.resolution, ConflictResolution::Unresolved))
        .cloned()
        .collect();
    Ok(AgentInterpretationContext {
        project_id: interpretation.project_id,
        design_id: interpretation.design_id,
        revision_id: interpretation.revision_id,
        confirmed_constraints,
        inferred_assumptions,
        unresolved_conflicts,
        unconfirmed_observation_ids: unconfirmed,
    })
}

fn owning_design(
    project_dir: &Path,
    design_id: &DesignId,
) -> Result<(ProjectId, crate::DesignPackagePaths), DrawingInterpretationStoreError> {
    let package = load_project_package(project_dir)
        .map_err(|error| DrawingInterpretationStoreError::Package(format!("{error:#}")))?;
    if !package
        .manifest
        .designs
        .iter()
        .any(|entry| &entry.id == design_id)
    {
        return Err(DrawingInterpretationStoreError::Invalid(
            "design does not belong to project".into(),
        ));
    }
    let paths = design_package_paths(project_dir, design_id)
        .map_err(|error| DrawingInterpretationStoreError::Package(error.to_string()))?;
    Ok((package.manifest.id, paths))
}

fn validate_shelf_references<'a>(
    project_dir: &Path,
    design_id: &DesignId,
    observations: impl Iterator<Item = &'a DrawingObservation>,
) -> Result<(), DrawingInterpretationStoreError> {
    let shelf = load_design_shelf(project_dir, design_id)
        .map_err(|error| DrawingInterpretationStoreError::Invalid(error.to_string()))?;
    for observation in observations {
        let item = shelf.items.get(&observation.shelf_item_id).ok_or_else(|| {
            DrawingInterpretationStoreError::Invalid(format!(
                "observation references missing shelf item `{}`",
                observation.shelf_item_id
            ))
        })?;
        let source = shelf_source_ref(&item.content).ok_or_else(|| {
            DrawingInterpretationStoreError::Invalid("observation shelf item has no source".into())
        })?;
        if source.source_id != observation.source_id
            || source.source_sha256 != observation.source_sha256
        {
            return Err(DrawingInterpretationStoreError::Invalid(
                "observation source does not match its Shelf item".into(),
            ));
        }
        validate_locator_matches_shelf(&observation.source_locator, &item.content)?;
    }
    Ok(())
}

fn validate_locator_matches_shelf(
    locator: &DrawingSourceLocator,
    content: &ShelfItemContent,
) -> Result<(), DrawingInterpretationStoreError> {
    let matches = match (locator, content) {
        (
            DrawingSourceLocator::PdfPage { page_number, .. },
            ShelfItemContent::PdfPage {
                page_number: shelf_page,
                ..
            }
            | ShelfItemContent::PdfCrop {
                page_number: shelf_page,
                ..
            },
        ) => page_number == shelf_page,
        (DrawingSourceLocator::Image { .. }, ShelfItemContent::ImageCrop { .. }) => true,
        (
            DrawingSourceLocator::CadLayout { layout, .. },
            ShelfItemContent::CadSelection {
                layout: shelf_layout,
                ..
            },
        ) => layout == shelf_layout,
        (
            DrawingSourceLocator::CadEntities {
                layout, entity_ids, ..
            },
            ShelfItemContent::CadSelection {
                layout: shelf_layout,
                object_ids,
                ..
            },
        ) => layout == shelf_layout && entity_ids.iter().all(|id| object_ids.contains(id)),
        (
            DrawingSourceLocator::IfcView { object_ids, .. },
            ShelfItemContent::IfcSelection {
                object_ids: shelf_object_ids,
                ..
            },
        ) => object_ids.iter().all(|id| shelf_object_ids.contains(id)),
        (DrawingSourceLocator::Saved3dView { .. }, ShelfItemContent::Saved3dView { .. }) => true,
        _ => false,
    };
    if matches {
        Ok(())
    } else {
        Err(DrawingInterpretationStoreError::Invalid(
            "observation locator does not match its exact Shelf selection".into(),
        ))
    }
}

fn shelf_source_ref(content: &ShelfItemContent) -> Option<&ShelfSourceRef> {
    match content {
        ShelfItemContent::PdfPage { source, .. }
        | ShelfItemContent::PdfCrop { source, .. }
        | ShelfItemContent::ImageCrop { source, .. }
        | ShelfItemContent::CadSelection { source, .. }
        | ShelfItemContent::IfcSelection { source, .. } => Some(source),
        ShelfItemContent::Saved3dView { source, .. } => source.as_ref(),
        ShelfItemContent::AcceptedDesignRevision { .. } => None,
    }
}

fn index_path(paths: &crate::DesignPackagePaths) -> PathBuf {
    paths.interpretations_dir.join("index.json")
}
fn revision_path(paths: &crate::DesignPackagePaths, id: &str) -> PathBuf {
    paths.interpretations_dir.join(format!("{id}.json"))
}
fn load_index(
    paths: &crate::DesignPackagePaths,
    project_id: ProjectId,
    design_id: DesignId,
) -> Result<InterpretationIndex, DrawingInterpretationStoreError> {
    fs::create_dir_all(&paths.interpretations_dir)?;
    reject_symlink(&paths.interpretations_dir)?;
    let path = index_path(paths);
    recover_index(&path)?;
    if !path.exists() {
        return Ok(InterpretationIndex {
            schema_version: DRAWING_INTERPRETATION_INDEX_SCHEMA_VERSION.into(),
            project_id,
            design_id,
            head_revision_id: None,
            revisions: BTreeMap::new(),
        });
    }
    reject_symlink(&path)?;
    let bytes = fs::read(&path)?;
    let raw: serde_json::Value = serde_json::from_slice(&bytes)?;
    let index: InterpretationIndex = serde_json::from_value(raw.clone())?;
    if serde_json::to_value(&index)? != raw {
        return Err(DrawingInterpretationStoreError::Invalid(
            "interpretation index contains unsupported future data".into(),
        ));
    }
    if index.schema_version != DRAWING_INTERPRETATION_INDEX_SCHEMA_VERSION
        || index.project_id != project_id
        || index.design_id != design_id
    {
        return Err(DrawingInterpretationStoreError::Invalid(
            "interpretation index schema or ownership is invalid".into(),
        ));
    }
    if index
        .head_revision_id
        .as_ref()
        .is_some_and(|id| !index.revisions.contains_key(id))
    {
        return Err(DrawingInterpretationStoreError::Invalid(
            "interpretation head is not indexed".into(),
        ));
    }
    validate_index_lineage(&index)?;
    Ok(index)
}

fn validate_index_lineage(
    index: &InterpretationIndex,
) -> Result<(), DrawingInterpretationStoreError> {
    if index.revisions.is_empty() {
        if index.head_revision_id.is_none() {
            return Ok(());
        }
        return Err(DrawingInterpretationStoreError::Invalid(
            "empty interpretation index has a head".into(),
        ));
    }
    if index
        .revisions
        .iter()
        .any(|(id, revision)| id != &revision.revision_id)
    {
        return Err(DrawingInterpretationStoreError::Invalid(
            "interpretation index key does not match revision identity".into(),
        ));
    }
    let roots = index
        .revisions
        .values()
        .filter(|revision| revision.parent_revision_id.is_none())
        .count();
    if roots != 1 {
        return Err(DrawingInterpretationStoreError::Invalid(
            "interpretation index must contain one revision root".into(),
        ));
    }
    let mut cursor = index.head_revision_id.as_ref();
    let mut visited = std::collections::BTreeSet::new();
    while let Some(revision_id) = cursor {
        if !visited.insert(revision_id.clone()) {
            return Err(DrawingInterpretationStoreError::Invalid(
                "interpretation revision lineage contains a cycle".into(),
            ));
        }
        let revision = index.revisions.get(revision_id).ok_or_else(|| {
            DrawingInterpretationStoreError::Invalid(
                "interpretation revision parent is not indexed".into(),
            )
        })?;
        cursor = revision.parent_revision_id.as_ref();
    }
    if visited.len() != index.revisions.len() {
        return Err(DrawingInterpretationStoreError::Invalid(
            "interpretation index contains detached revision lineage".into(),
        ));
    }
    Ok(())
}
fn load_revision_file(
    paths: &crate::DesignPackagePaths,
    id: &str,
    project_id: &ProjectId,
    design_id: &DesignId,
) -> Result<DrawingInterpretation, DrawingInterpretationStoreError> {
    let path = revision_path(paths, id);
    reject_symlink(&path)?;
    let bytes = fs::read(&path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            DrawingInterpretationStoreError::NotFound(id.into())
        } else {
            error.into()
        }
    })?;
    let raw: serde_json::Value = serde_json::from_slice(&bytes)?;
    let interpretation: DrawingInterpretation = serde_json::from_value(raw.clone())?;
    if serde_json::to_value(&interpretation)? != raw {
        return Err(DrawingInterpretationStoreError::Invalid(
            "interpretation revision contains unsupported future data".into(),
        ));
    }
    if &interpretation.project_id != project_id
        || &interpretation.design_id != design_id
        || interpretation.revision_id != id
    {
        return Err(DrawingInterpretationStoreError::Invalid(
            "interpretation file ownership or identity is invalid".into(),
        ));
    }
    interpretation.validate()?;
    Ok(interpretation)
}
fn publish_revision(
    paths: &crate::DesignPackagePaths,
    interpretation: &DrawingInterpretation,
) -> Result<(), DrawingInterpretationStoreError> {
    let path = revision_path(paths, &interpretation.revision_id);
    if path.exists() {
        let existing: DrawingInterpretation = serde_json::from_slice(&fs::read(&path)?)?;
        if existing == *interpretation {
            return Ok(());
        }
        return Err(DrawingInterpretationStoreError::Invalid(
            "immutable interpretation revision already exists with different bytes".into(),
        ));
    }
    atomic_create(&path, interpretation)
}
fn revision_ref(value: &DrawingInterpretation) -> DrawingInterpretationRevisionRef {
    DrawingInterpretationRevisionRef {
        revision_id: value.revision_id.clone(),
        parent_revision_id: value.parent_revision_id.clone(),
        created_at: value.created_at.clone(),
        observation_count: value.observations.len(),
        unresolved_conflict_count: value
            .conflicts
            .values()
            .filter(|conflict| matches!(conflict.resolution, ConflictResolution::Unresolved))
            .count(),
    }
}
fn validate_revision_id(id: &str) -> Result<(), DrawingInterpretationStoreError> {
    if id
        .strip_prefix("drawing-interpretation-sha256-")
        .is_some_and(|hash| {
            hash.len() == 64
                && hash
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        })
    {
        Ok(())
    } else {
        Err(DrawingInterpretationStoreError::Invalid(
            "drawing interpretation revision id is invalid".into(),
        ))
    }
}
fn save_index_with_hook<F>(
    paths: &crate::DesignPackagePaths,
    index: &InterpretationIndex,
    hook: &mut F,
) -> Result<(), DrawingInterpretationStoreError>
where
    F: FnMut(StoreCheckpoint) -> Result<(), DrawingInterpretationStoreError>,
{
    let path = index_path(paths);
    recover_index(&path)?;
    let parent = &paths.interpretations_dir;
    let temporary = parent.join(format!(".index.json.tmp-{}", unique_id()));
    let backup = parent.join(".index.json.bak");
    write_new(&temporary, index)?;
    hook(StoreCheckpoint::IndexStaged)?;
    if path.exists() {
        fs::rename(&path, &backup)?;
        hook(StoreCheckpoint::IndexBackedUp)?;
    }
    if let Err(error) = fs::rename(&temporary, &path) {
        if backup.exists() {
            fs::rename(&backup, &path)?;
        }
        return Err(error.into());
    }
    sync_directory(parent)?;
    if backup.exists() {
        fs::remove_file(backup)?;
        sync_directory(parent)?;
    }
    Ok(())
}
fn recover_index(path: &Path) -> Result<(), DrawingInterpretationStoreError> {
    let parent = path.parent().ok_or_else(|| {
        DrawingInterpretationStoreError::Invalid("interpretation index has no parent".into())
    })?;
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
fn atomic_create<T: Serialize>(
    path: &Path,
    value: &T,
) -> Result<(), DrawingInterpretationStoreError> {
    let parent = path.parent().unwrap();
    let temporary = parent.join(format!(".revision.tmp-{}", unique_id()));
    write_new(&temporary, value)?;
    fs::hard_link(&temporary, path)?;
    fs::remove_file(temporary)?;
    sync_directory(parent)
}
fn write_new<T: Serialize>(path: &Path, value: &T) -> Result<(), DrawingInterpretationStoreError> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    Ok(())
}
fn reject_symlink(path: &Path) -> Result<(), DrawingInterpretationStoreError> {
    if fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        Err(DrawingInterpretationStoreError::Invalid(
            "interpretation storage must not use symlinks".into(),
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
fn sync_directory(path: &Path) -> Result<(), DrawingInterpretationStoreError> {
    File::open(path)?.sync_all()?;
    Ok(())
}
#[cfg(windows)]
fn sync_directory(_: &Path) -> Result<(), DrawingInterpretationStoreError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CorrespondenceRelation, DrawingViewRole, InterpretationUncertainty,
        InterpretationUncertaintyKind, ShelfConfirmation, ShelfItem, ShelfItemContent, ShelfLayout,
        ShelfProvenance, ShelfRect, ShelfSourceRef, SourceImportRequest,
        create_named_project_package, import_source, upsert_shelf_item,
    };

    struct Fixture {
        root: PathBuf,
        project: PathBuf,
        design: DesignId,
        project_id: ProjectId,
        source: crate::SourceRecord,
    }

    impl Fixture {
        fn new() -> Self {
            let root =
                std::env::temp_dir().join(format!("fraia-interpretation-test-{}", unique_id()));
            fs::create_dir(&root).unwrap();
            let project = root.join("project");
            let package = create_named_project_package(&project, "Interpretation fixture").unwrap();
            let design = package.designs[0].manifest.id.clone();
            let project_id = package.manifest.id;
            let input = root.join("plan.pdf");
            fs::write(&input, b"%PDF-1.7\nfixture\n%%EOF\n").unwrap();
            let source = import_source(
                &project,
                SourceImportRequest {
                    selected_path: input,
                    display_alias: None,
                    expected_media_type: None,
                },
            )
            .unwrap()
            .record;
            upsert_shelf_item(
                &project,
                &design,
                ShelfItem {
                    id: "drawing-page".into(),
                    label: "Plan and elevation".into(),
                    annotations: Vec::new(),
                    confirmation: ShelfConfirmation {
                        confirmed: true,
                        confirmed_by: Some("user".into()),
                        confirmed_at: Some("fixture".into()),
                    },
                    provenance: ShelfProvenance {
                        created_at: "fixture".into(),
                        created_by: "user".into(),
                        method: "pdf_page".into(),
                        derivative_id: None,
                    },
                    drawing_context: None,
                    content: ShelfItemContent::PdfPage {
                        source: ShelfSourceRef {
                            source_id: source.id.clone(),
                            source_sha256: source.sha256.clone(),
                        },
                        page_number: 1,
                        layout: ShelfLayout {
                            media_box: ShelfRect {
                                x: 0.0,
                                y: 0.0,
                                width: 600.0,
                                height: 800.0,
                                coordinate_space: "pdf_user_space_points".into(),
                            },
                            crop_box: None,
                            rotation_degrees: 0,
                            user_unit: 1.0,
                        },
                    },
                },
            )
            .unwrap();
            Self {
                root,
                project,
                design,
                project_id,
                source,
            }
        }

        fn observations(&self) -> BTreeMap<String, DrawingObservation> {
            [
                (
                    "plan-grid-a",
                    DrawingViewRole::Plan,
                    vec![[10.0, 0.0], [10.0, 100.0]],
                ),
                (
                    "elevation-grid-a",
                    DrawingViewRole::Elevation,
                    vec![[20.0, 0.0], [20.0, 100.0]],
                ),
            ]
            .into_iter()
            .map(|(id, view_role, coordinates)| {
                (
                    id.into(),
                    DrawingObservation {
                        id: id.into(),
                        shelf_item_id: "drawing-page".into(),
                        source_id: self.source.id.clone(),
                        source_sha256: self.source.sha256.clone(),
                        source_locator: DrawingSourceLocator::PdfPage {
                            page_number: 1,
                            coordinate_space: "pdf_user_space_points".into(),
                        },
                        view_role,
                        source_geometry: ObservationSourceGeometry::Polyline {
                            coordinates,
                            closed: false,
                        },
                        design_geometry: None,
                        extraction: crate::ObservationExtraction {
                            method: InterpretationMethod::NativeVectorExtraction,
                            producer: "fixture-parser".into(),
                            producer_version: "1".into(),
                            confidence: 0.75,
                            uncertainty: vec![InterpretationUncertainty {
                                kind: InterpretationUncertaintyKind::ScaleUnconfirmed,
                                message: "Scale requires user confirmation".into(),
                            }],
                        },
                        confirmation: ObservationConfirmation::Unconfirmed,
                        feature: ObservationFeature::Grid {
                            grid_label: "A".into(),
                        },
                    },
                )
            })
            .collect()
        }

        fn initial(&self) -> DrawingInterpretationRevision {
            DrawingInterpretationRevision {
                project_id: self.project_id.clone(),
                design_id: self.design.clone(),
                parent_revision_id: None,
                created_at: "2026-08-13T01:00:00Z".into(),
                method: InterpretationMethod::NativeVectorExtraction,
                observations: self.observations(),
                correspondences: BTreeMap::new(),
                alignment_transforms: BTreeMap::new(),
                conflicts: BTreeMap::new(),
            }
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn confirm_and_reconcile(
        fixture: &Fixture,
        first: &DrawingInterpretation,
    ) -> DrawingInterpretation {
        let confirmed = confirm_drawing_observations(
            &fixture.project,
            &fixture.design,
            ConfirmObservationsOperation {
                expected_parent_revision_id: first.revision_id.clone(),
                observation_ids: first.observations.keys().cloned().collect(),
                confirmed_by: "engineer".into(),
                confirmed_at: "2026-08-13T01:05:00Z".into(),
                created_at: "2026-08-13T01:05:00Z".into(),
            },
        )
        .unwrap();
        let correspondence = CrossViewCorrespondence {
            id: "same-grid-a".into(),
            observation_ids: confirmed.observations.keys().cloned().collect(),
            relation: CorrespondenceRelation::SameGrid,
            confidence: 1.0,
            confirmation: ObservationConfirmation::Confirmed {
                confirmed_by: "engineer".into(),
                confirmed_at: "2026-08-13T01:06:00Z".into(),
            },
            uncertainty: Vec::new(),
        };
        let transform = ConfirmedAlignmentTransform {
            id: "alignment-a".into(),
            from_shelf_item_id: "drawing-page".into(),
            to_design_coordinate_space: "design_metres".into(),
            matrix: [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
            established_by_correspondence_ids: vec!["same-grid-a".into()],
            confirmed_by: "engineer".into(),
            confirmed_at: "2026-08-13T01:06:00Z".into(),
        };
        let design_geometries = confirmed
            .observations
            .keys()
            .map(|id| {
                (
                    id.clone(),
                    ObservationDesignGeometry::Polyline {
                        coordinates: vec![[0.0, 0.0, 0.0], [0.0, 10.0, 0.0]],
                        closed: false,
                        alignment_transform_id: "alignment-a".into(),
                    },
                )
            })
            .collect();
        reconcile_drawing_interpretation(
            &fixture.project,
            &fixture.design,
            ReconcileInterpretationOperation {
                expected_parent_revision_id: confirmed.revision_id,
                design_geometries,
                correspondences: BTreeMap::from([("same-grid-a".into(), correspondence)]),
                alignment_transforms: BTreeMap::from([("alignment-a".into(), transform)]),
                conflicts: BTreeMap::new(),
                created_at: "2026-08-13T01:06:00Z".into(),
            },
        )
        .unwrap()
    }

    #[test]
    fn parser_create_confirm_reconcile_move_and_reopen_preserve_exact_lineage() {
        let fixture = Fixture::new();
        let first = create_drawing_interpretation(
            &fixture.project,
            &fixture.design,
            None,
            InterpretationCreateAuthority::ParserAdapter,
            fixture.initial(),
        )
        .unwrap();
        let reconciled = confirm_and_reconcile(&fixture, &first);
        let list = list_drawing_interpretations(&fixture.project, &fixture.design).unwrap();
        assert_eq!(list.revisions.len(), 3);
        assert_eq!(
            list.head_revision_id.as_deref(),
            Some(reconciled.revision_id.as_str())
        );
        assert!(reconciled.parent_revision_id.is_some());
        let context = drawing_interpretation_agent_context(
            &fixture.project,
            &fixture.design,
            &reconciled.revision_id,
        )
        .unwrap();
        assert_eq!(context.confirmed_constraints.len(), 2);
        assert!(context.unresolved_conflicts.is_empty());
        let package = load_project_package(&fixture.project).unwrap();
        crate::save_project_package(&fixture.project, &package).unwrap();
        assert_eq!(
            load_drawing_interpretation(&fixture.project, &fixture.design, &reconciled.revision_id)
                .unwrap(),
            reconciled
        );
        let moved = fixture.root.join("moved-project");
        fs::rename(&fixture.project, &moved).unwrap();
        assert_eq!(
            load_drawing_interpretation(&moved, &fixture.design, &reconciled.revision_id).unwrap(),
            reconciled
        );
    }

    #[test]
    fn high_confidence_parser_observation_is_an_assumption_not_a_confirmed_fact() {
        let fixture = Fixture::new();
        let mut revision = fixture.initial();
        for observation in revision.observations.values_mut() {
            observation.extraction.confidence = 0.95;
        }
        let created = create_drawing_interpretation(
            &fixture.project,
            &fixture.design,
            None,
            InterpretationCreateAuthority::ParserAdapter,
            revision,
        )
        .unwrap();
        let context = drawing_interpretation_agent_context(
            &fixture.project,
            &fixture.design,
            &created.revision_id,
        )
        .unwrap();
        assert!(context.confirmed_constraints.is_empty());
        assert_eq!(context.inferred_assumptions.len(), 2);
        assert!(context.inferred_assumptions.iter().all(|candidate| {
            candidate.requires_confirmation
                && !candidate.materially_conflicted
                && candidate.interpretation_revision_id == created.revision_id
                && candidate.inference_id.contains(&candidate.observation_id)
        }));
    }

    #[test]
    fn user_correction_creates_new_head_and_supersedes_inference_identity() {
        let fixture = Fixture::new();
        let created = create_drawing_interpretation(
            &fixture.project,
            &fixture.design,
            None,
            InterpretationCreateAuthority::ParserAdapter,
            fixture.initial(),
        )
        .unwrap();
        let before = drawing_interpretation_agent_context(
            &fixture.project,
            &fixture.design,
            &created.revision_id,
        )
        .unwrap();
        let corrected = correct_drawing_observation(
            &fixture.project,
            &fixture.design,
            CorrectInterpretationObservationOperation {
                expected_parent_revision_id: created.revision_id.clone(),
                observation_id: "plan-grid-a".into(),
                corrected_view_role: Some(crate::DrawingViewRole::Section),
                corrected_feature: Some(ObservationFeature::SemanticHint {
                    suggested_role: "section_marker".into(),
                    rationale: "User corrected the inferred view role.".into(),
                }),
                corrected_by: "engineer".into(),
                corrected_at: "2026-08-14T01:00:00Z".into(),
                created_at: "2026-08-14T01:00:00Z".into(),
            },
        )
        .unwrap();
        assert_ne!(corrected.revision_id, created.revision_id);
        assert_eq!(
            list_drawing_interpretations(&fixture.project, &fixture.design)
                .unwrap()
                .head_revision_id
                .as_deref(),
            Some(corrected.revision_id.as_str())
        );
        assert_eq!(
            corrected.observations["plan-grid-a"].view_role,
            crate::DrawingViewRole::Section
        );
        assert!(matches!(
            corrected.observations["plan-grid-a"].confirmation,
            ObservationConfirmation::Confirmed { .. }
        ));
        assert!(
            before
                .inferred_assumptions
                .iter()
                .all(|candidate| candidate.interpretation_revision_id == created.revision_id)
        );
    }

    #[test]
    fn compare_and_swap_and_multi_design_isolation_fail_closed() {
        let fixture = Fixture::new();
        let first = create_drawing_interpretation(
            &fixture.project,
            &fixture.design,
            None,
            InterpretationCreateAuthority::ParserAdapter,
            fixture.initial(),
        )
        .unwrap();
        assert!(matches!(
            create_drawing_interpretation(
                &fixture.project,
                &fixture.design,
                None,
                InterpretationCreateAuthority::ParserAdapter,
                fixture.initial()
            ),
            Err(DrawingInterpretationStoreError::ParentConflict { .. })
        ));
        let other = DesignId::new("design-not-owned");
        assert!(list_drawing_interpretations(&fixture.project, &other).is_err());
        assert_eq!(
            load_head_drawing_interpretation(&fixture.project, &fixture.design)
                .unwrap()
                .unwrap()
                .revision_id,
            first.revision_id
        );
    }

    #[test]
    fn tamper_and_interrupted_index_update_are_detected_or_recovered() {
        let fixture = Fixture::new();
        let first = create_drawing_interpretation(
            &fixture.project,
            &fixture.design,
            None,
            InterpretationCreateAuthority::ParserAdapter,
            fixture.initial(),
        )
        .unwrap();
        let paths = design_package_paths(&fixture.project, &fixture.design).unwrap();
        let result = create_with_hook(
            &fixture.project,
            &fixture.design,
            Some(&first.revision_id),
            InterpretationCreateAuthority::ParserAdapter,
            DrawingInterpretationRevision {
                parent_revision_id: Some(first.revision_id.clone()),
                created_at: "2026-08-13T01:02:00Z".into(),
                ..fixture.initial()
            },
            |checkpoint| {
                if checkpoint == StoreCheckpoint::IndexBackedUp {
                    Err(DrawingInterpretationStoreError::Invalid(
                        "injected interruption".into(),
                    ))
                } else {
                    Ok(())
                }
            },
        );
        assert!(result.is_err());
        assert_eq!(
            list_drawing_interpretations(&fixture.project, &fixture.design)
                .unwrap()
                .head_revision_id
                .as_deref(),
            Some(first.revision_id.as_str())
        );
        let path = revision_path(&paths, &first.revision_id);
        let mut value: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        value["createdAt"] = serde_json::json!("tampered");
        fs::write(&path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
        assert!(
            load_drawing_interpretation(&fixture.project, &fixture.design, &first.revision_id)
                .is_err()
        );
    }

    #[test]
    fn parser_adapter_cannot_preconfirm_observations() {
        let fixture = Fixture::new();
        let mut revision = fixture.initial();
        revision
            .observations
            .values_mut()
            .next()
            .unwrap()
            .confirmation = ObservationConfirmation::Confirmed {
            confirmed_by: "parser".into(),
            confirmed_at: "fixture".into(),
        };
        assert!(
            matches!(create_drawing_interpretation(&fixture.project, &fixture.design, None, InterpretationCreateAuthority::ParserAdapter, revision), Err(DrawingInterpretationStoreError::Invalid(message)) if message.contains("unconfirmed"))
        );
    }

    #[test]
    fn parser_child_appends_to_confirmed_parent_without_erasing_lineage() {
        let fixture = Fixture::new();
        let first = create_drawing_interpretation(
            &fixture.project,
            &fixture.design,
            None,
            InterpretationCreateAuthority::ParserAdapter,
            fixture.initial(),
        )
        .unwrap();
        let confirmed = confirm_drawing_observations(
            &fixture.project,
            &fixture.design,
            ConfirmObservationsOperation {
                expected_parent_revision_id: first.revision_id,
                observation_ids: vec!["plan-grid-a".into()],
                confirmed_by: "engineer".into(),
                confirmed_at: "2026-08-14T00:00:00Z".into(),
                created_at: "2026-08-14T00:00:00Z".into(),
            },
        )
        .unwrap();
        let mut added = confirmed.observations["elevation-grid-a"].clone();
        added.id = "dxf-grid-b".into();
        added.confirmation = ObservationConfirmation::Unconfirmed;
        let child = create_drawing_interpretation(
            &fixture.project,
            &fixture.design,
            Some(&confirmed.revision_id),
            InterpretationCreateAuthority::ParserAdapter,
            DrawingInterpretationRevision {
                project_id: confirmed.project_id.clone(),
                design_id: confirmed.design_id.clone(),
                parent_revision_id: Some(confirmed.revision_id.clone()),
                created_at: "2026-08-14T00:01:00Z".into(),
                method: InterpretationMethod::NativeVectorExtraction,
                observations: BTreeMap::from([(added.id.clone(), added)]),
                correspondences: BTreeMap::new(),
                alignment_transforms: BTreeMap::new(),
                conflicts: BTreeMap::new(),
            },
        )
        .unwrap();
        assert_eq!(child.observations.len(), 3);
        assert!(matches!(
            child.observations["plan-grid-a"].confirmation,
            ObservationConfirmation::Confirmed { .. }
        ));
        assert!(matches!(
            child.observations["dxf-grid-b"].confirmation,
            ObservationConfirmation::Unconfirmed
        ));
        assert_eq!(
            load_head_drawing_interpretation(&fixture.project, &fixture.design)
                .unwrap()
                .unwrap(),
            child
        );
    }

    #[test]
    fn unresolved_conflict_is_visible_until_explicit_resolution() {
        let fixture = Fixture::new();
        let first = create_drawing_interpretation(
            &fixture.project,
            &fixture.design,
            None,
            InterpretationCreateAuthority::ParserAdapter,
            fixture.initial(),
        )
        .unwrap();
        let reconciled = confirm_and_reconcile(&fixture, &first);
        let conflicted = reconcile_drawing_interpretation(
            &fixture.project,
            &fixture.design,
            ReconcileInterpretationOperation {
                expected_parent_revision_id: reconciled.revision_id,
                design_geometries: BTreeMap::new(),
                correspondences: BTreeMap::new(),
                alignment_transforms: BTreeMap::new(),
                conflicts: BTreeMap::from([(
                    "grid-conflict".into(),
                    InterpretationConflict {
                        id: "grid-conflict".into(),
                        observation_ids: vec!["plan-grid-a".into(), "elevation-grid-a".into()],
                        conflict_kind: crate::InterpretationConflictKind::AlignmentMismatch,
                        message: "Plan and elevation grid locations disagree.".into(),
                        resolution: ConflictResolution::Unresolved,
                    },
                )]),
                created_at: "2026-08-13T01:07:00Z".into(),
            },
        )
        .unwrap();
        let context = drawing_interpretation_agent_context(
            &fixture.project,
            &fixture.design,
            &conflicted.revision_id,
        )
        .unwrap();
        assert!(context.confirmed_constraints.is_empty());
        assert_eq!(context.unresolved_conflicts.len(), 1);

        let resolved = resolve_drawing_interpretation_conflict(
            &fixture.project,
            &fixture.design,
            ResolveInterpretationConflictOperation {
                expected_parent_revision_id: conflicted.revision_id,
                conflict_id: "grid-conflict".into(),
                resolution: "Use the surveyed plan grid after engineer review.".into(),
                resolved_by: "engineer".into(),
                resolved_at: "2026-08-13T01:08:00Z".into(),
                created_at: "2026-08-13T01:08:00Z".into(),
            },
        )
        .unwrap();
        let context = drawing_interpretation_agent_context(
            &fixture.project,
            &fixture.design,
            &resolved.revision_id,
        )
        .unwrap();
        assert_eq!(context.confirmed_constraints.len(), 2);
        assert!(context.unresolved_conflicts.is_empty());
    }
}
