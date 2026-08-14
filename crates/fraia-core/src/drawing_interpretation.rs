use crate::{DesignId, DrawingViewRole, ProjectId, ShelfRect, ShelfTransform, SourceId};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const DRAWING_INTERPRETATION_SCHEMA_VERSION: &str = "fraia.drawing-interpretation.v1";
pub const DRAWING_INTERPRETATION_REVISION_VERSION: &str =
    "fraia.drawing-interpretation-revision.v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrawingInterpretation {
    pub schema_version: String,
    pub project_id: ProjectId,
    pub design_id: DesignId,
    pub revision_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_revision_id: Option<String>,
    pub created_at: String,
    pub method: InterpretationMethod,
    pub observations: BTreeMap<String, DrawingObservation>,
    #[serde(default)]
    pub correspondences: BTreeMap<String, CrossViewCorrespondence>,
    #[serde(default)]
    pub alignment_transforms: BTreeMap<String, ConfirmedAlignmentTransform>,
    #[serde(default)]
    pub conflicts: BTreeMap<String, InterpretationConflict>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrawingInterpretationRevision {
    pub project_id: ProjectId,
    pub design_id: DesignId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_revision_id: Option<String>,
    pub created_at: String,
    pub method: InterpretationMethod,
    pub observations: BTreeMap<String, DrawingObservation>,
    #[serde(default)]
    pub correspondences: BTreeMap<String, CrossViewCorrespondence>,
    #[serde(default)]
    pub alignment_transforms: BTreeMap<String, ConfirmedAlignmentTransform>,
    #[serde(default)]
    pub conflicts: BTreeMap<String, InterpretationConflict>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterpretationMethod {
    Manual,
    NativeVectorExtraction,
    NativeTextExtraction,
    OpticalCharacterRecognition,
    AgentVisualInterpretation,
    Reconciled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrawingObservation {
    pub id: String,
    pub shelf_item_id: String,
    pub source_id: SourceId,
    pub source_sha256: String,
    pub source_locator: DrawingSourceLocator,
    pub view_role: DrawingViewRole,
    pub source_geometry: ObservationSourceGeometry,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub design_geometry: Option<ObservationDesignGeometry>,
    pub extraction: ObservationExtraction,
    pub confirmation: ObservationConfirmation,
    #[serde(flatten)]
    pub feature: ObservationFeature,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "locatorKind", rename_all = "snake_case")]
pub enum DrawingSourceLocator {
    PdfPage {
        page_number: u32,
        coordinate_space: String,
    },
    Image {
        coordinate_space: String,
    },
    CadLayout {
        layout: String,
        coordinate_space: String,
    },
    CadEntities {
        layout: String,
        coordinate_space: String,
        entity_ids: Vec<String>,
        transforms: Vec<ShelfTransform>,
    },
    IfcView {
        view_id: String,
        coordinate_space: String,
        object_ids: Vec<String>,
        transforms: Vec<ShelfTransform>,
    },
    Saved3dView {
        view_id: String,
        coordinate_space: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "sourceGeometryKind", rename_all = "snake_case")]
pub enum ObservationSourceGeometry {
    Point {
        coordinate: [f64; 2],
    },
    Polyline {
        coordinates: Vec<[f64; 2]>,
        closed: bool,
    },
    Region {
        boundary: Vec<[f64; 2]>,
    },
    Anchor {
        bounds: ShelfRect,
        anchor: [f64; 2],
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "designGeometryKind", rename_all = "snake_case")]
pub enum ObservationDesignGeometry {
    Point {
        coordinate: [f64; 3],
        alignment_transform_id: String,
    },
    Polyline {
        coordinates: Vec<[f64; 3]>,
        closed: bool,
        alignment_transform_id: String,
    },
    Region {
        boundary: Vec<[f64; 3]>,
        alignment_transform_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "featureKind", rename_all = "snake_case")]
pub enum ObservationFeature {
    Point {
        point_role: String,
    },
    Curve {
        curve_role: String,
    },
    Region {
        region_role: String,
    },
    Grid {
        grid_label: String,
    },
    Level {
        level_label: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        elevation: Option<MeasuredValue>,
    },
    Label {
        text: String,
    },
    Dimension {
        label: String,
        measured: MeasuredValue,
        first_witness: [f64; 2],
        second_witness: [f64; 2],
    },
    Symbol {
        symbol_kind: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        text: Option<String>,
    },
    SemanticHint {
        suggested_role: String,
        rationale: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeasuredValue {
    pub value: f64,
    pub unit: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObservationExtraction {
    pub method: InterpretationMethod,
    pub producer: String,
    pub producer_version: String,
    pub confidence: f64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub uncertainty: Vec<InterpretationUncertainty>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterpretationUncertainty {
    pub kind: InterpretationUncertaintyKind,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterpretationUncertaintyKind {
    LowResolution,
    AmbiguousLinework,
    AmbiguousText,
    Occlusion,
    ScaleUnconfirmed,
    AlignmentUncertain,
    ConflictingEvidence,
    ParserLimitation,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ObservationConfirmation {
    Unconfirmed,
    Confirmed {
        confirmed_by: String,
        confirmed_at: String,
    },
    Rejected {
        rejected_by: String,
        rejected_at: String,
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossViewCorrespondence {
    pub id: String,
    pub observation_ids: Vec<String>,
    pub relation: CorrespondenceRelation,
    pub confidence: f64,
    pub confirmation: ObservationConfirmation,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub uncertainty: Vec<InterpretationUncertainty>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorrespondenceRelation {
    SamePoint,
    SameAxis,
    SameGrid,
    SameLevel,
    SameRegion,
    SameFeature,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmedAlignmentTransform {
    pub id: String,
    pub from_shelf_item_id: String,
    pub to_design_coordinate_space: String,
    /// Row-major homogeneous 4x4 transform from source coordinates to design coordinates.
    pub matrix: [f64; 16],
    pub established_by_correspondence_ids: Vec<String>,
    pub confirmed_by: String,
    pub confirmed_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterpretationConflict {
    pub id: String,
    pub observation_ids: Vec<String>,
    pub conflict_kind: InterpretationConflictKind,
    pub message: String,
    pub resolution: ConflictResolution,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterpretationConflictKind {
    ConflictingDimension,
    AlignmentMismatch,
    LabelMismatch,
    AmbiguousCorrespondence,
    UnsupportedTransform,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ConflictResolution {
    Unresolved,
    Resolved {
        resolution: String,
        resolved_by: String,
        resolved_at: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConfirmedProposalConstraint<'a> {
    pub observation: &'a DrawingObservation,
    pub design_geometry: &'a ObservationDesignGeometry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrawingInterpretationError {
    Invalid(String),
    UnconfirmedObservation(String),
    ObservationRejected(String),
    MissingDesignGeometry(String),
    UnresolvedConflict {
        observation_id: String,
        conflict_ids: Vec<String>,
    },
    RevisionIdentityMismatch {
        expected: String,
        actual: String,
    },
    Serialization(String),
}

impl std::fmt::Display for DrawingInterpretationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(message) | Self::Serialization(message) => formatter.write_str(message),
            Self::UnconfirmedObservation(id) => write!(
                formatter,
                "observation `{id}` is not confirmed and cannot constrain a structural proposal"
            ),
            Self::ObservationRejected(id) => write!(
                formatter,
                "observation `{id}` was rejected and cannot constrain a structural proposal"
            ),
            Self::MissingDesignGeometry(id) => write!(
                formatter,
                "observation `{id}` has no confirmed design-coordinate geometry"
            ),
            Self::UnresolvedConflict {
                observation_id,
                conflict_ids,
            } => write!(
                formatter,
                "observation `{observation_id}` has unresolved conflicts: {}",
                conflict_ids.join(", ")
            ),
            Self::RevisionIdentityMismatch { expected, actual } => write!(
                formatter,
                "drawing interpretation revision id `{actual}` does not match deterministic id `{expected}`"
            ),
        }
    }
}
impl std::error::Error for DrawingInterpretationError {}

impl DrawingInterpretation {
    pub fn new(
        revision: DrawingInterpretationRevision,
    ) -> Result<Self, DrawingInterpretationError> {
        let mut interpretation = Self {
            schema_version: DRAWING_INTERPRETATION_SCHEMA_VERSION.into(),
            project_id: revision.project_id,
            design_id: revision.design_id,
            revision_id: String::new(),
            parent_revision_id: revision.parent_revision_id,
            created_at: revision.created_at,
            method: revision.method,
            observations: revision.observations,
            correspondences: revision.correspondences,
            alignment_transforms: revision.alignment_transforms,
            conflicts: revision.conflicts,
        };
        interpretation.revision_id = interpretation.deterministic_revision_id()?;
        interpretation.validate()?;
        Ok(interpretation)
    }

    pub fn validate(&self) -> Result<(), DrawingInterpretationError> {
        if self.schema_version != DRAWING_INTERPRETATION_SCHEMA_VERSION {
            return Err(DrawingInterpretationError::Invalid(
                "unsupported drawing interpretation schema".into(),
            ));
        }
        for (id, observation) in &self.observations {
            if id != &observation.id {
                return Err(DrawingInterpretationError::Invalid(
                    "observation key does not match observation id".into(),
                ));
            }
            validate_observation(observation, &self.alignment_transforms)?;
        }
        for (id, correspondence) in &self.correspondences {
            if id != &correspondence.id || correspondence.observation_ids.len() < 2 {
                return Err(DrawingInterpretationError::Invalid(
                    "cross-view correspondence identity or cardinality is invalid".into(),
                ));
            }
            for observation_id in &correspondence.observation_ids {
                if !self.observations.contains_key(observation_id) {
                    return Err(DrawingInterpretationError::Invalid(format!(
                        "correspondence references missing observation `{observation_id}`"
                    )));
                }
            }
            validate_confidence(correspondence.confidence)?;
        }
        for (id, transform) in &self.alignment_transforms {
            if id != &transform.id
                || transform.established_by_correspondence_ids.is_empty()
                || transform.matrix.iter().any(|value| !value.is_finite())
            {
                return Err(DrawingInterpretationError::Invalid(
                    "alignment transform is invalid".into(),
                ));
            }
            for correspondence_id in &transform.established_by_correspondence_ids {
                let correspondence =
                    self.correspondences.get(correspondence_id).ok_or_else(|| {
                        DrawingInterpretationError::Invalid(format!(
                            "alignment references missing correspondence `{correspondence_id}`"
                        ))
                    })?;
                if !matches!(
                    correspondence.confirmation,
                    ObservationConfirmation::Confirmed { .. }
                ) {
                    return Err(DrawingInterpretationError::Invalid(
                        "alignment transforms require confirmed correspondences".into(),
                    ));
                }
            }
        }
        for (id, conflict) in &self.conflicts {
            if id != &conflict.id || conflict.observation_ids.len() < 2 {
                return Err(DrawingInterpretationError::Invalid(
                    "conflict identity or cardinality is invalid".into(),
                ));
            }
            for observation_id in &conflict.observation_ids {
                if !self.observations.contains_key(observation_id) {
                    return Err(DrawingInterpretationError::Invalid(format!(
                        "conflict references missing observation `{observation_id}`"
                    )));
                }
            }
        }
        let expected = self.deterministic_revision_id()?;
        if self.revision_id != expected {
            return Err(DrawingInterpretationError::RevisionIdentityMismatch {
                expected,
                actual: self.revision_id.clone(),
            });
        }
        Ok(())
    }

    pub fn deterministic_revision_id(&self) -> Result<String, DrawingInterpretationError> {
        #[derive(Serialize)]
        struct RevisionMaterial<'a> {
            version: &'static str,
            schema_version: &'a str,
            project_id: &'a ProjectId,
            design_id: &'a DesignId,
            parent_revision_id: &'a Option<String>,
            created_at: &'a str,
            method: &'a InterpretationMethod,
            observations: &'a BTreeMap<String, DrawingObservation>,
            correspondences: &'a BTreeMap<String, CrossViewCorrespondence>,
            alignment_transforms: &'a BTreeMap<String, ConfirmedAlignmentTransform>,
            conflicts: &'a BTreeMap<String, InterpretationConflict>,
        }
        let bytes = serde_json::to_vec(&RevisionMaterial {
            version: DRAWING_INTERPRETATION_REVISION_VERSION,
            schema_version: &self.schema_version,
            project_id: &self.project_id,
            design_id: &self.design_id,
            parent_revision_id: &self.parent_revision_id,
            created_at: &self.created_at,
            method: &self.method,
            observations: &self.observations,
            correspondences: &self.correspondences,
            alignment_transforms: &self.alignment_transforms,
            conflicts: &self.conflicts,
        })
        .map_err(|error| DrawingInterpretationError::Serialization(error.to_string()))?;
        Ok(format!(
            "drawing-interpretation-sha256-{:x}",
            Sha256::digest(bytes)
        ))
    }

    pub fn confirmed_proposal_constraint(
        &self,
        observation_id: &str,
    ) -> Result<ConfirmedProposalConstraint<'_>, DrawingInterpretationError> {
        let observation = self.observations.get(observation_id).ok_or_else(|| {
            DrawingInterpretationError::Invalid(format!(
                "observation `{observation_id}` was not found"
            ))
        })?;
        match observation.confirmation {
            ObservationConfirmation::Unconfirmed => {
                return Err(DrawingInterpretationError::UnconfirmedObservation(
                    observation_id.into(),
                ));
            }
            ObservationConfirmation::Rejected { .. } => {
                return Err(DrawingInterpretationError::ObservationRejected(
                    observation_id.into(),
                ));
            }
            ObservationConfirmation::Confirmed { .. } => {}
        }
        let conflict_ids = self
            .conflicts
            .values()
            .filter(|conflict| {
                conflict
                    .observation_ids
                    .iter()
                    .any(|id| id == observation_id)
                    && matches!(conflict.resolution, ConflictResolution::Unresolved)
            })
            .map(|conflict| conflict.id.clone())
            .collect::<Vec<_>>();
        if !conflict_ids.is_empty() {
            return Err(DrawingInterpretationError::UnresolvedConflict {
                observation_id: observation_id.into(),
                conflict_ids,
            });
        }
        let design_geometry = observation.design_geometry.as_ref().ok_or_else(|| {
            DrawingInterpretationError::MissingDesignGeometry(observation_id.into())
        })?;
        Ok(ConfirmedProposalConstraint {
            observation,
            design_geometry,
        })
    }
}

fn validate_observation(
    observation: &DrawingObservation,
    transforms: &BTreeMap<String, ConfirmedAlignmentTransform>,
) -> Result<(), DrawingInterpretationError> {
    validate_token("observation id", &observation.id)?;
    validate_token("shelf item id", &observation.shelf_item_id)?;
    if observation.source_sha256.len() != 64
        || !observation
            .source_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(DrawingInterpretationError::Invalid(
            "observation source hash is invalid".into(),
        ));
    }
    if observation
        .source_id
        .sha256()
        .map_err(|error| DrawingInterpretationError::Invalid(error.to_string()))?
        != observation.source_sha256
    {
        return Err(DrawingInterpretationError::Invalid(
            "observation source id does not match its source hash".into(),
        ));
    }
    validate_confidence(observation.extraction.confidence)?;
    match &observation.source_locator {
        DrawingSourceLocator::PdfPage {
            page_number,
            coordinate_space,
        } => {
            if *page_number == 0 {
                return Err(DrawingInterpretationError::Invalid(
                    "drawing observation PDF page is one-based".into(),
                ));
            }
            validate_token("source coordinate space", coordinate_space)?;
        }
        DrawingSourceLocator::Image { coordinate_space } => {
            validate_token("source coordinate space", coordinate_space)?
        }
        DrawingSourceLocator::CadLayout {
            layout,
            coordinate_space,
        }
        | DrawingSourceLocator::CadEntities {
            layout,
            coordinate_space,
            ..
        } => {
            validate_token("CAD layout", layout)?;
            validate_token("source coordinate space", coordinate_space)?;
            if let DrawingSourceLocator::CadEntities {
                entity_ids,
                transforms,
                ..
            } = &observation.source_locator
            {
                if entity_ids.is_empty() || entity_ids.len() != transforms.len() {
                    return Err(DrawingInterpretationError::Invalid(
                        "CAD observation requires one transform for each source entity".into(),
                    ));
                }
                for id in entity_ids {
                    validate_token("CAD entity id", id)?;
                }
                for transform in transforms {
                    if !transform
                        .translation
                        .into_iter()
                        .chain(transform.rotation_degrees)
                        .chain(transform.scale)
                        .all(f64::is_finite)
                    {
                        return Err(DrawingInterpretationError::Invalid(
                            "CAD entity transform contains a non-finite value".into(),
                        ));
                    }
                }
            }
        }
        DrawingSourceLocator::IfcView {
            view_id,
            coordinate_space,
            object_ids,
            transforms,
        } => {
            validate_token("source view id", view_id)?;
            validate_token("source coordinate space", coordinate_space)?;
            if object_ids.is_empty() || object_ids.len() != transforms.len() {
                return Err(DrawingInterpretationError::Invalid(
                    "IFC locator requires exact object ids and matching transforms".into(),
                ));
            }
            for id in object_ids {
                validate_token("IFC object id", id)?;
            }
            if transforms.iter().any(|transform| {
                !transform
                    .translation
                    .into_iter()
                    .chain(transform.rotation_degrees)
                    .chain(transform.scale)
                    .all(f64::is_finite)
            }) {
                return Err(DrawingInterpretationError::Invalid(
                    "IFC object transform contains a non-finite value".into(),
                ));
            }
        }
        DrawingSourceLocator::Saved3dView {
            view_id,
            coordinate_space,
        } => {
            validate_token("source view id", view_id)?;
            validate_token("source coordinate space", coordinate_space)?;
        }
    }
    validate_source_geometry(&observation.source_geometry)?;
    validate_feature(&observation.feature)?;
    if let Some(geometry) = &observation.design_geometry {
        let transform_id = match geometry {
            ObservationDesignGeometry::Point {
                alignment_transform_id,
                ..
            }
            | ObservationDesignGeometry::Polyline {
                alignment_transform_id,
                ..
            }
            | ObservationDesignGeometry::Region {
                alignment_transform_id,
                ..
            } => alignment_transform_id,
        };
        if !transforms.contains_key(transform_id) {
            return Err(DrawingInterpretationError::Invalid(format!(
                "observation references missing confirmed alignment `{transform_id}`"
            )));
        }
        validate_design_geometry(geometry)?;
    }
    Ok(())
}
fn validate_feature(feature: &ObservationFeature) -> Result<(), DrawingInterpretationError> {
    match feature {
        ObservationFeature::Level {
            elevation: Some(measured),
            ..
        } => validate_measured_value(measured),
        ObservationFeature::Dimension {
            measured,
            first_witness,
            second_witness,
            ..
        } => {
            validate_measured_value(measured)?;
            finite(
                first_witness
                    .iter()
                    .chain(second_witness)
                    .copied()
                    .collect(),
            )
        }
        _ => Ok(()),
    }
}
fn validate_measured_value(measured: &MeasuredValue) -> Result<(), DrawingInterpretationError> {
    if !measured.value.is_finite() || measured.unit.trim().is_empty() {
        return Err(DrawingInterpretationError::Invalid(
            "interpreted measurement is invalid".into(),
        ));
    }
    Ok(())
}
fn validate_source_geometry(
    geometry: &ObservationSourceGeometry,
) -> Result<(), DrawingInterpretationError> {
    let values = match geometry {
        ObservationSourceGeometry::Point { coordinate } => coordinate.to_vec(),
        ObservationSourceGeometry::Polyline { coordinates, .. } => {
            if coordinates.len() < 2 {
                return Err(DrawingInterpretationError::Invalid(
                    "source polyline requires at least two points".into(),
                ));
            }
            coordinates.iter().flatten().copied().collect()
        }
        ObservationSourceGeometry::Region { boundary } => {
            if boundary.len() < 3 {
                return Err(DrawingInterpretationError::Invalid(
                    "source region requires at least three points".into(),
                ));
            }
            boundary.iter().flatten().copied().collect()
        }
        ObservationSourceGeometry::Anchor { bounds, anchor } => vec![
            bounds.x,
            bounds.y,
            bounds.width,
            bounds.height,
            anchor[0],
            anchor[1],
        ],
    };
    finite(values)
}
fn validate_design_geometry(
    geometry: &ObservationDesignGeometry,
) -> Result<(), DrawingInterpretationError> {
    let values = match geometry {
        ObservationDesignGeometry::Point { coordinate, .. } => coordinate.to_vec(),
        ObservationDesignGeometry::Polyline { coordinates, .. } => {
            if coordinates.len() < 2 {
                return Err(DrawingInterpretationError::Invalid(
                    "design polyline requires at least two points".into(),
                ));
            }
            coordinates.iter().flatten().copied().collect()
        }
        ObservationDesignGeometry::Region { boundary, .. } => {
            if boundary.len() < 3 {
                return Err(DrawingInterpretationError::Invalid(
                    "design region requires at least three points".into(),
                ));
            }
            boundary.iter().flatten().copied().collect()
        }
    };
    finite(values)
}
fn finite(values: Vec<f64>) -> Result<(), DrawingInterpretationError> {
    if values.into_iter().any(|value| !value.is_finite()) {
        Err(DrawingInterpretationError::Invalid(
            "interpretation coordinates must be finite".into(),
        ))
    } else {
        Ok(())
    }
}
fn validate_confidence(value: f64) -> Result<(), DrawingInterpretationError> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(DrawingInterpretationError::Invalid(
            "interpretation confidence must be between zero and one".into(),
        ))
    }
}
fn validate_token(label: &str, value: &str) -> Result<(), DrawingInterpretationError> {
    if value.trim().is_empty()
        || value.len() > 255
        || value.chars().any(char::is_control)
        || value.contains(['/', '\\'])
        || matches!(value, "." | "..")
    {
        Err(DrawingInterpretationError::Invalid(format!(
            "{label} is invalid"
        )))
    } else {
        Ok(())
    }
}

pub fn unresolved_conflict_observation_ids(
    interpretation: &DrawingInterpretation,
) -> BTreeSet<String> {
    interpretation
        .conflicts
        .values()
        .filter(|conflict| matches!(conflict.resolution, ConflictResolution::Unresolved))
        .flat_map(|conflict| conflict.observation_ids.iter().cloned())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn confirmed() -> ObservationConfirmation {
        ObservationConfirmation::Confirmed {
            confirmed_by: "user".into(),
            confirmed_at: "2026-08-13T00:00:00Z".into(),
        }
    }
    fn observation(
        id: &str,
        shelf: &str,
        role: DrawingViewRole,
        feature: ObservationFeature,
        source: Vec<[f64; 2]>,
        design: Vec<[f64; 3]>,
    ) -> DrawingObservation {
        DrawingObservation {
            id: id.into(),
            shelf_item_id: shelf.into(),
            source_id: SourceId::from_sha256(&"a".repeat(64)).unwrap(),
            source_sha256: "a".repeat(64),
            view_role: role,
            source_locator: DrawingSourceLocator::PdfPage {
                page_number: if shelf == "plan-crop" { 2 } else { 5 },
                coordinate_space: "pdf_user_space_points".into(),
            },
            source_geometry: ObservationSourceGeometry::Polyline {
                coordinates: source,
                closed: false,
            },
            design_geometry: Some(ObservationDesignGeometry::Polyline {
                coordinates: design,
                closed: false,
                alignment_transform_id: "alignment-1".into(),
            }),
            extraction: ObservationExtraction {
                method: InterpretationMethod::NativeVectorExtraction,
                producer: "fixture".into(),
                producer_version: "1".into(),
                confidence: 0.98,
                uncertainty: Vec::new(),
            },
            confirmation: confirmed(),
            feature,
        }
    }
    fn fixture() -> DrawingInterpretation {
        let observations = BTreeMap::from([
            (
                "plan-grid-a".into(),
                observation(
                    "plan-grid-a",
                    "plan-crop",
                    DrawingViewRole::Plan,
                    ObservationFeature::Grid {
                        grid_label: "A".into(),
                    },
                    vec![[100.0, 20.0], [100.0, 520.0]],
                    vec![[0.0, 0.0, 0.0], [0.0, 10.0, 0.0]],
                ),
            ),
            (
                "elevation-grid-a".into(),
                observation(
                    "elevation-grid-a",
                    "elevation-crop",
                    DrawingViewRole::Elevation,
                    ObservationFeature::Grid {
                        grid_label: "A".into(),
                    },
                    vec![[60.0, 30.0], [60.0, 330.0]],
                    vec![[0.0, 0.0, 0.0], [0.0, 0.0, 6.0]],
                ),
            ),
            (
                "plan-level-1".into(),
                observation(
                    "plan-level-1",
                    "plan-crop",
                    DrawingViewRole::Plan,
                    ObservationFeature::Level {
                        level_label: "Level 1".into(),
                        elevation: Some(MeasuredValue {
                            value: 0.0,
                            unit: "m".into(),
                        }),
                    },
                    vec![[20.0, 200.0], [500.0, 200.0]],
                    vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]],
                ),
            ),
            (
                "elevation-level-1".into(),
                observation(
                    "elevation-level-1",
                    "elevation-crop",
                    DrawingViewRole::Elevation,
                    ObservationFeature::Level {
                        level_label: "Level 1".into(),
                        elevation: Some(MeasuredValue {
                            value: 0.0,
                            unit: "m".into(),
                        }),
                    },
                    vec![[20.0, 300.0], [500.0, 300.0]],
                    vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]],
                ),
            ),
        ]);
        let correspondences = BTreeMap::from([
            (
                "same-grid".into(),
                CrossViewCorrespondence {
                    id: "same-grid".into(),
                    observation_ids: vec!["plan-grid-a".into(), "elevation-grid-a".into()],
                    relation: CorrespondenceRelation::SameGrid,
                    confidence: 1.0,
                    confirmation: confirmed(),
                    uncertainty: Vec::new(),
                },
            ),
            (
                "same-level".into(),
                CrossViewCorrespondence {
                    id: "same-level".into(),
                    observation_ids: vec!["plan-level-1".into(), "elevation-level-1".into()],
                    relation: CorrespondenceRelation::SameLevel,
                    confidence: 1.0,
                    confirmation: confirmed(),
                    uncertainty: Vec::new(),
                },
            ),
        ]);
        let alignment = ConfirmedAlignmentTransform {
            id: "alignment-1".into(),
            from_shelf_item_id: "plan-crop".into(),
            to_design_coordinate_space: "design_metres".into(),
            matrix: [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
            established_by_correspondence_ids: vec!["same-grid".into(), "same-level".into()],
            confirmed_by: "user".into(),
            confirmed_at: "2026-08-13T00:00:00Z".into(),
        };
        DrawingInterpretation::new(DrawingInterpretationRevision {
            project_id: ProjectId::new("project-golden"),
            design_id: DesignId::new("design-golden"),
            parent_revision_id: None,
            created_at: "2026-08-13T00:00:00Z".into(),
            method: InterpretationMethod::Reconciled,
            observations,
            correspondences,
            alignment_transforms: BTreeMap::from([("alignment-1".into(), alignment)]),
            conflicts: BTreeMap::new(),
        })
        .unwrap()
    }

    #[test]
    fn plan_elevation_reconcile_through_confirmed_grid_and_level() {
        let interpretation = fixture();
        interpretation.validate().unwrap();
        assert!(
            interpretation
                .confirmed_proposal_constraint("plan-grid-a")
                .is_ok()
        );
        let mut invalid = interpretation.clone();
        invalid
            .correspondences
            .get_mut("same-grid")
            .unwrap()
            .confirmation = ObservationConfirmation::Unconfirmed;
        invalid.revision_id = invalid.deterministic_revision_id().unwrap();
        assert!(
            matches!(invalid.validate(), Err(DrawingInterpretationError::Invalid(message)) if message.contains("confirmed correspondences"))
        );
    }

    #[test]
    fn conflicting_dimensions_remain_visible_and_block_constraints() {
        let mut interpretation = fixture();
        for (id, shelf, value) in [
            ("width-plan", "plan-crop", 10.0),
            ("width-elevation", "elevation-crop", 9.6),
        ] {
            interpretation.observations.insert(
                id.into(),
                observation(
                    id,
                    shelf,
                    if shelf == "plan-crop" {
                        DrawingViewRole::Plan
                    } else {
                        DrawingViewRole::Elevation
                    },
                    ObservationFeature::Dimension {
                        label: "overall width".into(),
                        measured: MeasuredValue {
                            value,
                            unit: "m".into(),
                        },
                        first_witness: [20.0, 20.0],
                        second_witness: [220.0, 20.0],
                    },
                    vec![[20.0, 20.0], [220.0, 20.0]],
                    vec![[0.0, 0.0, 0.0], [value, 0.0, 0.0]],
                ),
            );
        }
        interpretation.conflicts.insert(
            "conflict-width".into(),
            InterpretationConflict {
                id: "conflict-width".into(),
                observation_ids: vec!["width-plan".into(), "width-elevation".into()],
                conflict_kind: InterpretationConflictKind::ConflictingDimension,
                message: "Plan states 10.0 m; elevation states 9.6 m.".into(),
                resolution: ConflictResolution::Unresolved,
            },
        );
        interpretation.revision_id = interpretation.deterministic_revision_id().unwrap();
        interpretation.validate().unwrap();
        assert!(matches!(
            interpretation.confirmed_proposal_constraint("width-plan"),
            Err(DrawingInterpretationError::UnresolvedConflict { .. })
        ));
        assert_eq!(
            unresolved_conflict_observation_ids(&interpretation),
            BTreeSet::from(["width-elevation".into(), "width-plan".into()])
        );
    }

    #[test]
    fn unconfirmed_observation_cannot_constrain_proposal() {
        let mut interpretation = fixture();
        interpretation
            .observations
            .get_mut("plan-grid-a")
            .unwrap()
            .confirmation = ObservationConfirmation::Unconfirmed;
        interpretation.revision_id = interpretation.deterministic_revision_id().unwrap();
        interpretation.validate().unwrap();
        assert!(matches!(
            interpretation.confirmed_proposal_constraint("plan-grid-a"),
            Err(DrawingInterpretationError::UnconfirmedObservation(_))
        ));
    }

    #[test]
    fn golden_serialization_and_revision_are_reproducible_and_tamper_evident() {
        let interpretation = fixture();
        assert_eq!(
            interpretation.revision_id,
            "drawing-interpretation-sha256-79fce3465e9fc5ad276a273c990e1698deea505f869ca3302bbbb5820899d5ba"
        );
        let encoded = serde_json::to_string_pretty(&interpretation).unwrap();
        let decoded: DrawingInterpretation = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, interpretation);
        assert_eq!(
            decoded.revision_id,
            decoded.deterministic_revision_id().unwrap()
        );
        assert!(encoded.contains("\"schemaVersion\": \"fraia.drawing-interpretation.v1\""));
        assert!(encoded.contains("\"sourceGeometryKind\": \"polyline\""));
        assert!(encoded.contains("\"featureKind\": \"grid\""));
        let mut tampered = decoded;
        tampered
            .observations
            .get_mut("plan-grid-a")
            .unwrap()
            .extraction
            .confidence = 0.5;
        assert!(matches!(
            tampered.validate(),
            Err(DrawingInterpretationError::RevisionIdentityMismatch { .. })
        ));
    }
}
