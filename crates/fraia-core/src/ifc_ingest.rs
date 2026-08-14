//! Bounded, read-only indexing for a conservative IFC STEP Part 21 subset.
//!
//! This module preserves evidence and identity. It never authors structural objects.

use crate::{
    DesignId, DrawingContext, DrawingInterpretationRevision, DrawingObservation,
    DrawingSourceLocator, DrawingViewRole, InterpretationMethod, InterpretationUncertainty,
    InterpretationUncertaintyKind, ObservationConfirmation, ObservationExtraction,
    ObservationFeature, ObservationSourceGeometry, ShelfConfirmation, ShelfItem, ShelfItemContent,
    ShelfOrientation, ShelfProvenance, ShelfSourceRef, ShelfTransform, SourceDerivative,
    SourceDerivativeKind, SourceDerivativeRequest, SourceId, SourceLibraryError,
    SourceLibraryPolicy, SourceMediaType, inspect_source, load_project_package,
    read_source_original, store_source_derivative,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};

pub const IFC_INDEX_SCHEMA_VERSION: &str = "fraia.ifc-index.v1";
pub const IFC_PARSER_ID: &str = "fraia.ifc-step.bounded";
pub const IFC_PARSER_VERSION: &str = "1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IfcObject {
    pub step_id: u64,
    pub global_id: Option<String>,
    pub class_name: String,
    pub name: Option<String>,
    pub placement_id: Option<u64>,
    pub transform: ShelfTransform,
    pub storey_id: Option<u64>,
    pub representation_ids: Vec<u64>,
    pub property_set_ids: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IfcStorey {
    pub step_id: u64,
    pub global_id: Option<String>,
    pub name: Option<String>,
    pub elevation: Option<f64>,
    pub transform: ShelfTransform,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IfcGrid {
    pub step_id: u64,
    pub global_id: Option<String>,
    pub name: Option<String>,
    pub axis_ids: Vec<u64>,
    pub transform: ShelfTransform,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IfcDiagnosticCode {
    UnsupportedRepresentation,
    MissingReference,
    UnsupportedEntity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IfcDiagnostic {
    pub code: IfcDiagnosticCode,
    pub step_id: Option<u64>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IfcDocumentIndex {
    pub schema_version: String,
    pub source_id: SourceId,
    pub source_sha256: String,
    pub parser: String,
    pub parser_version: String,
    pub file_schema: Vec<String>,
    pub length_unit: Option<String>,
    pub objects: BTreeMap<String, IfcObject>,
    pub storeys: BTreeMap<u64, IfcStorey>,
    pub grids: BTreeMap<u64, IfcGrid>,
    pub properties: BTreeMap<u64, BTreeMap<String, String>>,
    pub diagnostics: Vec<IfcDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IfcIndexResult {
    pub index: IfcDocumentIndex,
    pub derivative: SourceDerivative,
    pub resumed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IfcSelectionRequest {
    pub shelf_item_id: String,
    pub label: String,
    pub source_id: SourceId,
    pub view_id: String,
    #[serde(default)]
    pub object_ids: Vec<String>,
    #[serde(default)]
    pub storey_ids: Vec<u64>,
    #[serde(default)]
    pub grid_ids: Vec<u64>,
    #[serde(default)]
    pub class_names: Vec<String>,
    pub created_at: String,
    pub created_by: String,
    pub interpretation_parent_revision_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreparedIfcSelection {
    pub shelf_item: ShelfItem,
    pub interpretation: DrawingInterpretationRevision,
}

#[derive(Debug, Clone)]
pub struct IfcParsePolicy {
    pub max_bytes: usize,
    pub max_entities: usize,
    pub max_arguments_per_entity: usize,
    pub max_record_bytes: usize,
    pub max_parse_millis: u64,
}
impl Default for IfcParsePolicy {
    fn default() -> Self {
        Self {
            max_bytes: 256 * 1024 * 1024,
            max_entities: 5_000_000,
            max_arguments_per_entity: 100_000,
            max_record_bytes: 16 * 1024 * 1024,
            max_parse_millis: 120_000,
        }
    }
}

#[derive(Debug)]
pub enum IfcError {
    Invalid(String),
    Malformed(String),
    Limit(String),
    TimeLimit,
    Source(SourceLibraryError),
    Json(serde_json::Error),
    Package(String),
}
impl std::fmt::Display for IfcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(v) | Self::Malformed(v) | Self::Limit(v) | Self::Package(v) => {
                f.write_str(v)
            }
            Self::TimeLimit => f.write_str("IFC parsing exceeded its time limit"),
            Self::Source(v) => write!(f, "{v}"),
            Self::Json(v) => write!(f, "{v}"),
        }
    }
}
impl std::error::Error for IfcError {}
impl From<SourceLibraryError> for IfcError {
    fn from(value: SourceLibraryError) -> Self {
        Self::Source(value)
    }
}
impl From<serde_json::Error> for IfcError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

#[derive(Clone)]
struct Entity {
    id: u64,
    class_name: String,
    args: String,
}

pub fn index_and_store_ifc(
    project_dir: &std::path::Path,
    source_id: &SourceId,
    source_policy: &SourceLibraryPolicy,
    parse_policy: &IfcParsePolicy,
) -> Result<IfcIndexResult, IfcError> {
    let source = inspect_source(project_dir, source_id)?;
    if source.detected_media_type != SourceMediaType::IfcStep {
        return Err(IfcError::Invalid(
            "source is not a sniffed IFC STEP file".into(),
        ));
    }
    for derivative in crate::source_derivatives(project_dir, source_id)? {
        if derivative.kind == SourceDerivativeKind::BimIndex
            && derivative.parser == IFC_PARSER_ID
            && derivative.parser_version == IFC_PARSER_VERSION
        {
            let (_, payload) = crate::read_source_derivative(project_dir, &derivative.id)?;
            let index: IfcDocumentIndex = serde_json::from_slice(&payload)?;
            validate_index(&index, &source.id, &source.sha256)?;
            return Ok(IfcIndexResult {
                index,
                derivative,
                resumed: true,
            });
        }
    }
    let bytes = read_source_original(project_dir, source_id)?;
    let index = parse_ifc_step(&bytes, &source.id, &source.sha256, parse_policy)?;
    let derivative = store_source_derivative(
        project_dir,
        SourceDerivativeRequest {
            source_id: source.id,
            kind: SourceDerivativeKind::BimIndex,
            payload: serde_json::to_vec(&index)?,
            media_type: "application/vnd.fraia.ifc-index+json".into(),
            parser: IFC_PARSER_ID.into(),
            parser_version: IFC_PARSER_VERSION.into(),
            units: index.length_unit.clone(),
            coordinate_system: Some("ifc_local_placement".into()),
            warnings: Vec::new(),
        },
        source_policy,
    )?;
    Ok(IfcIndexResult {
        index,
        derivative,
        resumed: false,
    })
}

pub fn prepare_ifc_selection(
    project_dir: &std::path::Path,
    design_id: &DesignId,
    index: &IfcDocumentIndex,
    request: IfcSelectionRequest,
) -> Result<PreparedIfcSelection, IfcError> {
    let package =
        load_project_package(project_dir).map_err(|error| IfcError::Package(error.to_string()))?;
    if !package
        .manifest
        .designs
        .iter()
        .any(|entry| &entry.id == design_id)
    {
        return Err(IfcError::Invalid(
            "IFC selection design does not belong to project".into(),
        ));
    }
    let source = inspect_source(project_dir, &request.source_id)?;
    if request.source_id != index.source_id || source.sha256 != index.source_sha256 {
        return Err(IfcError::Invalid(
            "IFC selection source identity changed after indexing".into(),
        ));
    }
    let mut ids = request.object_ids.into_iter().collect::<BTreeSet<_>>();
    for storey in request.storey_ids {
        if !index.storeys.contains_key(&storey) {
            return Err(IfcError::Invalid(format!(
                "IFC storey #{storey} was not indexed"
            )));
        }
        ids.extend(
            index
                .objects
                .iter()
                .filter(|(_, o)| o.storey_id == Some(storey))
                .map(|(id, _)| id.clone()),
        );
    }
    for grid in request.grid_ids {
        let Some(grid) = index.grids.get(&grid) else {
            return Err(IfcError::Invalid(format!("IFC grid was not indexed")));
        };
        ids.insert(
            grid.global_id
                .clone()
                .unwrap_or_else(|| format!("step:{}", grid.step_id)),
        );
    }
    for class in request.class_names {
        let class = class.to_ascii_uppercase();
        ids.extend(
            index
                .objects
                .iter()
                .filter(|(_, o)| o.class_name == class)
                .map(|(id, _)| id.clone()),
        );
    }
    if ids.is_empty() {
        return Err(IfcError::Invalid(
            "IFC selection resolved to no objects".into(),
        ));
    }
    let selected = ids
        .iter()
        .map(|id| {
            index
                .objects
                .get(id)
                .ok_or_else(|| IfcError::Invalid(format!("IFC object `{id}` was not indexed")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let orientation = ShelfOrientation {
        forward: [0.0, 0.0, -1.0],
        up: [0.0, 1.0, 0.0],
    };
    let shelf_item = ShelfItem {
        id: request.shelf_item_id.clone(),
        label: request.label,
        annotations: Vec::new(),
        confirmation: ShelfConfirmation {
            confirmed: true,
            confirmed_by: Some(request.created_by.clone()),
            confirmed_at: Some(request.created_at.clone()),
        },
        provenance: ShelfProvenance {
            created_at: request.created_at.clone(),
            created_by: request.created_by.clone(),
            method: "ifc_index_selection".into(),
            derivative_id: None,
        },
        drawing_context: Some(DrawingContext {
            view_role: DrawingViewRole::Reference,
            orientation: orientation.clone(),
            calibration: None,
        }),
        content: ShelfItemContent::IfcSelection {
            source: ShelfSourceRef {
                source_id: source.id.clone(),
                source_sha256: source.sha256.clone(),
            },
            object_ids: ids.iter().cloned().collect(),
            transform: identity_transform(),
            orientation,
            scale: 1.0,
        },
    };
    let mut observations = BTreeMap::new();
    for object in selected {
        let id = object
            .global_id
            .clone()
            .unwrap_or_else(|| format!("step:{}", object.step_id));
        observations.insert(format!("ifc-observation-{id}"),DrawingObservation{id:format!("ifc-observation-{id}"),shelf_item_id:request.shelf_item_id.clone(),source_id:source.id.clone(),source_sha256:source.sha256.clone(),source_locator:DrawingSourceLocator::IfcView{view_id:request.view_id.clone(),coordinate_space:"ifc_local_placement".into(),object_ids:vec![id.clone()],transforms:vec![object.transform.clone()]},view_role:DrawingViewRole::Reference,source_geometry:ObservationSourceGeometry::Point{coordinate:[object.transform.translation[0],object.transform.translation[1]]},design_geometry:None,extraction:ObservationExtraction{method:InterpretationMethod::NativeVectorExtraction,producer:IFC_PARSER_ID.into(),producer_version:IFC_PARSER_VERSION.into(),confidence:0.95,uncertainty:vec![InterpretationUncertainty{kind:InterpretationUncertaintyKind::ParserLimitation,message:"IFC class and placement are exact parser evidence, but no centre-line or Fraia structural role is inferred.".into()}]},confirmation:ObservationConfirmation::Unconfirmed,feature:ObservationFeature::SemanticHint{suggested_role:object.class_name.clone(),rationale:"Fraia inferred the IFC semantic class from exact STEP evidence. It remains an assumption until corrected or confirmed.".into()}});
    }
    Ok(PreparedIfcSelection {
        shelf_item,
        interpretation: DrawingInterpretationRevision {
            project_id: package.manifest.id,
            design_id: design_id.clone(),
            parent_revision_id: request.interpretation_parent_revision_id,
            created_at: request.created_at,
            method: InterpretationMethod::NativeVectorExtraction,
            observations,
            correspondences: BTreeMap::new(),
            alignment_transforms: BTreeMap::new(),
            conflicts: BTreeMap::new(),
        },
    })
}

fn validate_index(
    index: &IfcDocumentIndex,
    source_id: &SourceId,
    hash: &str,
) -> Result<(), IfcError> {
    if index.schema_version != IFC_INDEX_SCHEMA_VERSION
        || &index.source_id != source_id
        || index.source_sha256 != hash
        || index.parser != IFC_PARSER_ID
        || index.parser_version != IFC_PARSER_VERSION
    {
        return Err(IfcError::Invalid(
            "IFC index identity or schema is invalid".into(),
        ));
    }
    Ok(())
}
fn identity_transform() -> ShelfTransform {
    ShelfTransform {
        translation: [0.0; 3],
        rotation_degrees: [0.0; 3],
        scale: [1.0; 3],
    }
}

pub fn parse_ifc_step(
    bytes: &[u8],
    source_id: &SourceId,
    source_sha256: &str,
    policy: &IfcParsePolicy,
) -> Result<IfcDocumentIndex, IfcError> {
    let started = Instant::now();
    if bytes.len() > policy.max_bytes {
        return Err(IfcError::Limit(format!(
            "IFC exceeds {} bytes",
            policy.max_bytes
        )));
    }
    let text = std::str::from_utf8(bytes)
        .map_err(|_| IfcError::Malformed("IFC STEP is not UTF-8/ASCII text".into()))?;
    if text.contains('\0')
        || !text.to_ascii_uppercase().contains("ISO-10303-21;")
        || !text.to_ascii_uppercase().contains("END-ISO-10303-21;")
    {
        return Err(IfcError::Malformed(
            "IFC STEP envelope is missing or corrupt".into(),
        ));
    }
    let records = split_records(text, policy, started)?;
    let mut entities = BTreeMap::new();
    let mut schema = Vec::new();
    for record in records {
        check_time(started, policy)?;
        let trimmed = record.trim();
        if trimmed.to_ascii_uppercase().starts_with("FILE_SCHEMA") {
            schema = quoted(trimmed)
                .into_iter()
                .filter(|v| v.to_ascii_uppercase().starts_with("IFC"))
                .collect();
            continue;
        }
        if !trimmed.starts_with('#') {
            continue;
        }
        let (left, right) = trimmed
            .split_once('=')
            .ok_or_else(|| IfcError::Malformed("IFC entity has no equals sign".into()))?;
        let id = left[1..]
            .trim()
            .parse()
            .map_err(|_| IfcError::Malformed("IFC entity id is invalid".into()))?;
        let open = right
            .find('(')
            .ok_or_else(|| IfcError::Malformed(format!("IFC entity #{id} has no argument list")))?;
        if !right.ends_with(')') {
            return Err(IfcError::Malformed(format!(
                "IFC entity #{id} has an unterminated argument list"
            )));
        }
        let class_name = right[..open].trim().to_ascii_uppercase();
        let args = right[open + 1..right.len() - 1].to_string();
        if top_args(&args)?.len() > policy.max_arguments_per_entity {
            return Err(IfcError::Limit(format!(
                "IFC entity #{id} has too many arguments"
            )));
        }
        if entities
            .insert(
                id,
                Entity {
                    id,
                    class_name,
                    args,
                },
            )
            .is_some()
        {
            return Err(IfcError::Malformed(format!("duplicate IFC entity #{id}")));
        }
        if entities.len() > policy.max_entities {
            return Err(IfcError::Limit(format!(
                "IFC exceeds {} entities",
                policy.max_entities
            )));
        }
    }
    if schema.is_empty() {
        return Err(IfcError::Invalid(
            "IFC has no supported FILE_SCHEMA identity".into(),
        ));
    }
    build_index(entities, schema, source_id, source_sha256)
}

fn build_index(
    entities: BTreeMap<u64, Entity>,
    file_schema: Vec<String>,
    source_id: &SourceId,
    hash: &str,
) -> Result<IfcDocumentIndex, IfcError> {
    let mut point = BTreeMap::<u64, [f64; 3]>::new();
    let mut axis = BTreeMap::<u64, [f64; 3]>::new();
    let mut local = BTreeMap::<u64, (Option<u64>, Option<u64>)>::new();
    let mut containment = BTreeMap::<u64, u64>::new();
    let mut psets = BTreeMap::<u64, Vec<u64>>::new();
    let mut properties = BTreeMap::<u64, BTreeMap<String, String>>::new();
    let mut property_values = BTreeMap::<u64, (String, String)>::new();
    for entity in entities.values() {
        let args = top_args(&entity.args)?;
        match entity.class_name.as_str() {
            "IFCCARTESIANPOINT" => {
                if let Some(list) = args.first() {
                    let n = numbers(list);
                    point.insert(
                        entity.id,
                        [
                            *n.first().unwrap_or(&0.0),
                            *n.get(1).unwrap_or(&0.0),
                            *n.get(2).unwrap_or(&0.0),
                        ],
                    );
                }
            }
            "IFCAXIS2PLACEMENT3D" | "IFCAXIS2PLACEMENT2D" => {
                if let Some(id) = args.first().and_then(|v| reference(v)) {
                    axis.insert(entity.id, *point.get(&id).unwrap_or(&[0.0; 3]));
                }
            }
            "IFCLOCALPLACEMENT" => {
                local.insert(
                    entity.id,
                    (
                        args.first().and_then(|v| reference(v)),
                        args.get(1).and_then(|v| reference(v)),
                    ),
                );
            }
            "IFCRELCONTAINEDINSPATIALSTRUCTURE" => {
                if args.len() > 5 {
                    if let Some(storey) = reference(&args[5]) {
                        for id in references(&args[4]) {
                            containment.insert(id, storey);
                        }
                    }
                }
            }
            "IFCRELDEFINESBYPROPERTIES" => {
                if args.len() > 5 {
                    if let Some(pset) = reference(&args[5]) {
                        for id in references(&args[4]) {
                            psets.entry(id).or_default().push(pset);
                        }
                    }
                }
            }
            "IFCPROPERTYSINGLEVALUE" => {
                property_values.insert(
                    entity.id,
                    (
                        string_arg(args.first()),
                        args.get(2).cloned().unwrap_or_default(),
                    ),
                );
            }
            _ => {}
        }
    }
    for entity in entities
        .values()
        .filter(|e| e.class_name == "IFCPROPERTYSET")
    {
        let args = top_args(&entity.args)?;
        let mut set = BTreeMap::new();
        if let Some(ids) = args.get(4) {
            for id in references(ids) {
                if let Some((k, v)) = property_values.get(&id) {
                    set.insert(k.clone(), v.clone());
                }
            }
        }
        properties.insert(entity.id, set);
    }
    let transform_for = |placement: Option<u64>| resolve_transform(placement, &local, &axis);
    let mut storeys = BTreeMap::new();
    let mut grids = BTreeMap::new();
    let mut objects = BTreeMap::new();
    let mut diagnostics = Vec::new();
    let product_classes = [
        "IFCBUILDINGELEMENTPROXY",
        "IFCBEAM",
        "IFCCOLUMN",
        "IFCSLAB",
        "IFCWALL",
        "IFCROOF",
        "IFCFOOTING",
        "IFCMEMBER",
        "IFCPLATE",
        "IFCSPACE",
        "IFCGRID",
        "IFCBUILDINGSTOREY",
    ];
    for entity in entities.values() {
        let args = top_args(&entity.args)?;
        let global = string_opt(args.first());
        let name = string_opt(args.get(2));
        let placement = args.get(5).and_then(|v| reference(v));
        let representation = args.get(6).map(|v| references(v)).unwrap_or_default();
        if entity.class_name == "IFCBUILDINGSTOREY" {
            storeys.insert(
                entity.id,
                IfcStorey {
                    step_id: entity.id,
                    global_id: global,
                    name,
                    elevation: args.get(9).and_then(|v| v.parse().ok()),
                    transform: transform_for(placement),
                },
            );
            continue;
        }
        if entity.class_name == "IFCGRID" {
            grids.insert(
                entity.id,
                IfcGrid {
                    step_id: entity.id,
                    global_id: global.clone(),
                    name: name.clone(),
                    axis_ids: args.iter().skip(7).flat_map(|v| references(v)).collect(),
                    transform: transform_for(placement),
                },
            );
        }
        if product_classes.contains(&entity.class_name.as_str()) {
            let key = global
                .clone()
                .unwrap_or_else(|| format!("step:{}", entity.id));
            if !representation.is_empty() {
                diagnostics.push(IfcDiagnostic{code:IfcDiagnosticCode::UnsupportedRepresentation,step_id:Some(entity.id),message:"Representation identity is preserved; this bounded subset does not tessellate IFC geometry.".into()});
            }
            objects.insert(
                key,
                IfcObject {
                    step_id: entity.id,
                    global_id: global,
                    class_name: entity.class_name.clone(),
                    name,
                    placement_id: placement,
                    transform: transform_for(placement),
                    storey_id: containment.get(&entity.id).copied(),
                    representation_ids: representation,
                    property_set_ids: psets.remove(&entity.id).unwrap_or_default(),
                },
            );
        } else if entity.class_name.starts_with("IFC")
            && (entity.class_name.ends_with("ELEMENT") || entity.class_name.ends_with("FEATURE"))
        {
            diagnostics.push(IfcDiagnostic {
                code: IfcDiagnosticCode::UnsupportedEntity,
                step_id: Some(entity.id),
                message: format!(
                    "{} is preserved as an unsupported STEP entity",
                    entity.class_name
                ),
            });
        }
    }
    let length_unit = entities
        .values()
        .find(|e| {
            e.class_name == "IFCSIUNIT" && e.args.to_ascii_uppercase().contains(".LENGTHUNIT.")
        })
        .map(|e| {
            if e.args.to_ascii_uppercase().contains(".MILLI.") {
                "mm".into()
            } else {
                "m".into()
            }
        });
    Ok(IfcDocumentIndex {
        schema_version: IFC_INDEX_SCHEMA_VERSION.into(),
        source_id: source_id.clone(),
        source_sha256: hash.into(),
        parser: IFC_PARSER_ID.into(),
        parser_version: IFC_PARSER_VERSION.into(),
        file_schema,
        length_unit,
        objects,
        storeys,
        grids,
        properties,
        diagnostics,
    })
}

fn split_records(
    text: &str,
    policy: &IfcParsePolicy,
    started: Instant,
) -> Result<Vec<String>, IfcError> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    for ch in text.chars() {
        check_time(started, policy)?;
        if ch == '\'' {
            quoted = !quoted;
        }
        if ch == ';' && !quoted {
            if current.len() > policy.max_record_bytes {
                return Err(IfcError::Limit("IFC record exceeds byte limit".into()));
            }
            out.push(std::mem::take(&mut current));
        } else {
            current.push(ch);
        }
    }
    if !current.trim().is_empty() {
        return Err(IfcError::Malformed("IFC has an unterminated record".into()));
    }
    Ok(out)
}
fn top_args(value: &str) -> Result<Vec<String>, IfcError> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut depth = 0_i32;
    let mut quoted = false;
    for ch in value.chars() {
        if ch == '\'' {
            quoted = !quoted;
        }
        if !quoted {
            if ch == '(' {
                depth += 1;
            }
            if ch == ')' {
                depth -= 1;
                if depth < 0 {
                    return Err(IfcError::Malformed("unbalanced IFC arguments".into()));
                }
            }
            if ch == ',' && depth == 0 {
                out.push(current.trim().into());
                current.clear();
                continue;
            }
        }
        current.push(ch);
    }
    if quoted || depth != 0 {
        return Err(IfcError::Malformed("unterminated IFC argument".into()));
    }
    out.push(current.trim().into());
    Ok(out)
}
fn quoted(v: &str) -> Vec<String> {
    let mut r = Vec::new();
    let mut q = false;
    let mut c = String::new();
    for ch in v.chars() {
        if ch == '\'' {
            if q {
                r.push(std::mem::take(&mut c));
            }
            q = !q;
        } else if q {
            c.push(ch);
        }
    }
    r
}
fn string_opt(v: Option<&String>) -> Option<String> {
    v.and_then(|x| {
        let q = quoted(x);
        q.first().cloned()
    })
}
fn string_arg(v: Option<&String>) -> String {
    v.and_then(|x| quoted(x).first().cloned())
        .unwrap_or_default()
}
fn reference(v: &str) -> Option<u64> {
    v.trim().strip_prefix('#')?.parse().ok()
}
fn references(v: &str) -> Vec<u64> {
    v.split(|c: char| !c.is_ascii_digit() && c != '#')
        .filter_map(reference)
        .collect()
}
fn numbers(v: &str) -> Vec<f64> {
    v.trim_matches(|c| c == '(' || c == ')')
        .split(',')
        .filter_map(|v| v.trim().parse().ok())
        .collect()
}
fn resolve_transform(
    mut id: Option<u64>,
    local: &BTreeMap<u64, (Option<u64>, Option<u64>)>,
    axis: &BTreeMap<u64, [f64; 3]>,
) -> ShelfTransform {
    let mut t = [0.0; 3];
    let mut seen = BTreeSet::new();
    while let Some(current) = id {
        if !seen.insert(current) {
            break;
        }
        let Some((parent, relative)) = local.get(&current) else {
            break;
        };
        if let Some(p) = relative.and_then(|v| axis.get(&v)) {
            for i in 0..3 {
                t[i] += p[i];
            }
        }
        id = *parent;
    }
    ShelfTransform {
        translation: t,
        rotation_degrees: [0.0; 3],
        scale: [1.0; 3],
    }
}
fn check_time(started: Instant, p: &IfcParsePolicy) -> Result<(), IfcError> {
    if started.elapsed() > Duration::from_millis(p.max_parse_millis) {
        Err(IfcError::TimeLimit)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn identity() -> (SourceId, String) {
        let h = "a".repeat(64);
        (SourceId::from_sha256(&h).unwrap(), h)
    }
    #[test]
    fn multistorey_ids_transforms_and_unsupported_representation_are_preserved() {
        let (s, h) = identity();
        let f=b"ISO-10303-21;HEADER;FILE_SCHEMA(('IFC4'));ENDSEC;DATA;#1=IFCCARTESIANPOINT((0.,0.,3000.));#2=IFCAXIS2PLACEMENT3D(#1,$,$);#3=IFCLOCALPLACEMENT($,#2);#4=IFCCARTESIANPOINT((0.,0.,0.));#5=IFCAXIS2PLACEMENT3D(#4,$,$);#6=IFCLOCALPLACEMENT($,#5);#9=IFCBUILDINGSTOREY('STOREY1',$,'Level 1',$,$,#6,$,$,.ELEMENT.,0.);#10=IFCBUILDINGSTOREY('STOREY2',$,'Level 2',$,$,#3,$,$,.ELEMENT.,3000.);#20=IFCBEAM('BEAM1',$,'Beam A',$,$,#3,#99,$,$);#21=IFCGRID('GRID1',$,'Main grids',$,$,#6,$,(#70),(#71),$);#30=IFCRELCONTAINEDINSPATIALSTRUCTURE('R',$,$,$,(#20),#10);#31=IFCPROPERTYSINGLEVALUE('FireRating',$,IFCLABEL('60 min'),$);#32=IFCPROPERTYSET('PSET1',$,'Pset_BeamCommon',$,(#31));#33=IFCRELDEFINESBYPROPERTIES('RP',$,$,$,(#20),#32);#40=IFCSIUNIT(*,.LENGTHUNIT.,.MILLI.,.METRE.);ENDSEC;END-ISO-10303-21;";
        let i = parse_ifc_step(f, &s, &h, &IfcParsePolicy::default()).unwrap();
        assert_eq!(i.length_unit.as_deref(), Some("mm"));
        assert_eq!(i.objects["BEAM1"].storey_id, Some(10));
        assert_eq!(i.objects["BEAM1"].transform.translation, [0., 0., 3000.]);
        assert_eq!(i.objects["BEAM1"].representation_ids, vec![99]);
        assert_eq!(i.storeys.len(), 2);
        assert_eq!(i.grids[&21].global_id.as_deref(), Some("GRID1"));
        assert_eq!(i.objects["BEAM1"].property_set_ids, vec![32]);
        assert_eq!(i.properties[&32]["FireRating"], "IFCLABEL('60 min')");
        assert!(
            i.diagnostics
                .iter()
                .any(|d| d.code == IfcDiagnosticCode::UnsupportedRepresentation)
        );
    }
    #[test]
    fn malformed_and_bounds_fail_closed() {
        let (s, h) = identity();
        assert!(
            parse_ifc_step(
                b"ISO-10303-21;#1=IFCBEAM('x';END-ISO-10303-21;",
                &s,
                &h,
                &IfcParsePolicy::default()
            )
            .is_err()
        );
        let p = IfcParsePolicy {
            max_entities: 0,
            ..IfcParsePolicy::default()
        };
        assert!(matches!(parse_ifc_step(b"ISO-10303-21;HEADER;FILE_SCHEMA(('IFC4'));ENDSEC;DATA;#1=IFCBEAM('x');ENDSEC;END-ISO-10303-21;",&s,&h,&p),Err(IfcError::Limit(_))));
    }
}
