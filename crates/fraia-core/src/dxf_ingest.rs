//! Bounded, dependency-free ASCII DXF indexing and selection preparation.
//!
//! The parser preserves drawing evidence. It does not infer structural members,
//! extrude 2D geometry, or mutate an authored structural model.

use crate::{
    DesignId, DrawingContext, DrawingInterpretationRevision, DrawingObservation,
    DrawingSourceLocator, DrawingViewRole, InterpretationMethod, InterpretationUncertainty,
    InterpretationUncertaintyKind, ObservationConfirmation, ObservationExtraction,
    ObservationFeature, ObservationSourceGeometry, ShelfConfirmation, ShelfItem, ShelfItemContent,
    ShelfOrientation, ShelfProvenance, ShelfSourceRef, ShelfTransform, SourceDerivative,
    SourceDerivativeKind, SourceDerivativeRequest, SourceId, SourceLibraryError,
    SourceLibraryPolicy, SourceMediaType, SourceWarning, SourceWarningCode, inspect_source,
    load_project_package, read_source_original, store_source_derivative,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::time::{Duration, Instant};

pub const DXF_INDEX_SCHEMA_VERSION: &str = "fraia.dxf-index.v1";
pub const DXF_PARSER_ID: &str = "fraia.ascii-dxf.bounded";
pub const DXF_PARSER_VERSION: &str = "1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DxfSpace {
    Model,
    Paper,
    BlockDefinition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DxfLayer {
    pub name: String,
    pub frozen: bool,
    pub hidden: bool,
    pub locked: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DxfBlock {
    pub name: String,
    pub base_point: [f64; 3],
    pub entity_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DxfGeometry {
    Line {
        start: [f64; 3],
        end: [f64; 3],
    },
    Polyline {
        vertices: Vec<[f64; 3]>,
        closed: bool,
    },
    Circle {
        center: [f64; 3],
        radius: f64,
    },
    Arc {
        center: [f64; 3],
        radius: f64,
        start_degrees: f64,
        end_degrees: f64,
    },
    Text {
        insertion: [f64; 3],
        text: String,
        height: Option<f64>,
        rotation_degrees: f64,
    },
    Dimension {
        definition: [f64; 3],
        first_witness: Option<[f64; 3]>,
        second_witness: Option<[f64; 3]>,
        text: Option<String>,
        measurement: Option<f64>,
    },
    Insert {
        block_name: String,
        insertion: [f64; 3],
        scale: [f64; 3],
        rotation_degrees: f64,
        columns: u32,
        rows: u32,
    },
    Unsupported {
        entity_type: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DxfEntity {
    pub id: String,
    pub handle: Option<String>,
    pub entity_type: String,
    pub layer: String,
    pub layout: String,
    pub space: DxfSpace,
    pub hidden: bool,
    pub frozen: bool,
    pub block_name: Option<String>,
    pub transform: ShelfTransform,
    pub geometry: DxfGeometry,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DxfDiagnosticCode {
    UnitsUnknown,
    UnsupportedEntity,
    MissingBlock,
    MalformedRecord,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DxfDiagnostic {
    pub code: DxfDiagnosticCode,
    pub message: String,
    pub entity_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DxfDocumentIndex {
    pub schema_version: String,
    pub source_id: SourceId,
    pub source_sha256: String,
    pub parser: String,
    pub parser_version: String,
    pub acad_version: Option<String>,
    pub units: Option<String>,
    pub model_space_name: String,
    pub paper_layouts: Vec<String>,
    pub layers: BTreeMap<String, DxfLayer>,
    pub blocks: BTreeMap<String, DxfBlock>,
    pub entities: BTreeMap<String, DxfEntity>,
    pub diagnostics: Vec<DxfDiagnostic>,
}

#[derive(Debug, Clone)]
pub struct DxfParsePolicy {
    pub max_bytes: usize,
    pub max_entities: usize,
    pub max_pairs: usize,
    pub max_vertices_per_entity: usize,
    pub max_block_depth: usize,
    pub max_parse_millis: u64,
}

impl Default for DxfParsePolicy {
    fn default() -> Self {
        Self {
            max_bytes: 256 * 1024 * 1024,
            max_entities: 5_000_000,
            max_pairs: 50_000_000,
            max_vertices_per_entity: 1_000_000,
            max_block_depth: 32,
            max_parse_millis: 120_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DxfIndexResult {
    pub index: DxfDocumentIndex,
    pub derivative: SourceDerivative,
    pub resumed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DxfViewRelation {
    pub confirmed: bool,
    pub confirmed_by: String,
    pub confirmed_at: String,
    pub transform: ShelfTransform,
    pub orientation: ShelfOrientation,
    pub scale: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DxfSelectionRequest {
    pub shelf_item_id: String,
    pub label: String,
    pub source_id: SourceId,
    pub layout: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entity_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layer_names: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub block_names: Vec<String>,
    pub view_role: Option<DrawingViewRole>,
    pub relation_to_design: Option<DxfViewRelation>,
    pub created_at: String,
    pub created_by: String,
    pub interpretation_parent_revision_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreparedDxfSelection {
    pub shelf_item: ShelfItem,
    pub interpretation: DrawingInterpretationRevision,
}

#[derive(Debug)]
pub enum DxfError {
    Invalid(String),
    UnsupportedBinary,
    Malformed { line: usize, message: String },
    EntityLimit { limit: usize },
    PairLimit { limit: usize },
    VertexLimit { limit: usize },
    TimeLimit { limit_millis: u64 },
    Source(SourceLibraryError),
    Json(serde_json::Error),
    Package(String),
}

impl std::fmt::Display for DxfError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(message) | Self::Package(message) => formatter.write_str(message),
            Self::UnsupportedBinary => formatter
                .write_str("binary DXF is not supported by the bounded offline ASCII DXF parser"),
            Self::Malformed { line, message } => {
                write!(formatter, "malformed DXF at line {line}: {message}")
            }
            Self::EntityLimit { limit } => write!(formatter, "DXF exceeds {limit} entities"),
            Self::PairLimit { limit } => write!(formatter, "DXF exceeds {limit} group pairs"),
            Self::VertexLimit { limit } => {
                write!(formatter, "DXF entity exceeds {limit} vertices")
            }
            Self::TimeLimit { limit_millis } => {
                write!(formatter, "DXF parsing exceeded {limit_millis} ms")
            }
            Self::Source(error) => write!(formatter, "{error}"),
            Self::Json(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for DxfError {}
impl From<SourceLibraryError> for DxfError {
    fn from(value: SourceLibraryError) -> Self {
        Self::Source(value)
    }
}
impl From<serde_json::Error> for DxfError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

#[derive(Clone)]
struct Pair {
    code: i16,
    value: String,
    line: usize,
}

pub fn index_and_store_dxf(
    project_dir: &Path,
    source_id: &SourceId,
    source_policy: &SourceLibraryPolicy,
    parse_policy: &DxfParsePolicy,
) -> Result<DxfIndexResult, DxfError> {
    let source = inspect_source(project_dir, source_id)?;
    if source.detected_media_type != SourceMediaType::Dxf {
        return Err(DxfError::Invalid("source is not a sniffed DXF".into()));
    }
    for derivative in crate::source_derivatives(project_dir, source_id)? {
        if derivative.kind == SourceDerivativeKind::CadIndex
            && derivative.parser == DXF_PARSER_ID
            && derivative.parser_version == DXF_PARSER_VERSION
        {
            let (_, payload) = crate::read_source_derivative(project_dir, &derivative.id)?;
            let index: DxfDocumentIndex = serde_json::from_slice(&payload)?;
            validate_index(&index, &source.id, &source.sha256)?;
            return Ok(DxfIndexResult {
                index,
                derivative,
                resumed: true,
            });
        }
    }
    let bytes = read_source_original(project_dir, source_id)?;
    let index = parse_ascii_dxf(&bytes, &source.id, &source.sha256, parse_policy)?;
    let warnings = if index.units.is_none() {
        vec![SourceWarning {
            code: SourceWarningCode::UnitsUnknown,
            message: "DXF insertion units are not declared; calibration is required.".into(),
        }]
    } else {
        Vec::new()
    };
    let derivative = store_source_derivative(
        project_dir,
        SourceDerivativeRequest {
            source_id: source.id,
            kind: SourceDerivativeKind::CadIndex,
            payload: serde_json::to_vec(&index)?,
            media_type: "application/vnd.fraia.dxf-index+json".into(),
            parser: DXF_PARSER_ID.into(),
            parser_version: DXF_PARSER_VERSION.into(),
            units: index.units.clone(),
            coordinate_system: Some("dxf_world_coordinates".into()),
            warnings,
        },
        source_policy,
    )?;
    Ok(DxfIndexResult {
        index,
        derivative,
        resumed: false,
    })
}

pub fn parse_ascii_dxf(
    bytes: &[u8],
    source_id: &SourceId,
    source_sha256: &str,
    policy: &DxfParsePolicy,
) -> Result<DxfDocumentIndex, DxfError> {
    let started = Instant::now();
    if bytes.len() > policy.max_bytes {
        return Err(DxfError::Invalid(format!(
            "DXF exceeds {} bytes",
            policy.max_bytes
        )));
    }
    if bytes.starts_with(b"AutoCAD Binary DXF") {
        return Err(DxfError::UnsupportedBinary);
    }
    let text = std::str::from_utf8(bytes)
        .map_err(|_| DxfError::Invalid("ASCII DXF is not valid UTF-8/ASCII text".into()))?;
    if text.contains('\0') {
        return Err(DxfError::Invalid("DXF contains NUL bytes".into()));
    }
    let lines = text.lines().collect::<Vec<_>>();
    if lines.len() % 2 != 0 {
        return Err(DxfError::Malformed {
            line: lines.len(),
            message: "group code has no value line".into(),
        });
    }
    if lines.len() / 2 > policy.max_pairs {
        return Err(DxfError::PairLimit {
            limit: policy.max_pairs,
        });
    }
    let mut pairs = Vec::with_capacity(lines.len() / 2);
    for index in (0..lines.len()).step_by(2) {
        check_time(started, policy)?;
        let code = lines[index]
            .trim()
            .parse::<i16>()
            .map_err(|_| DxfError::Malformed {
                line: index + 1,
                message: "group code is not an integer".into(),
            })?;
        pairs.push(Pair {
            code,
            value: lines[index + 1].trim_end_matches('\r').to_string(),
            line: index + 2,
        });
    }
    if !pairs
        .iter()
        .any(|pair| pair.code == 0 && pair.value.trim() == "EOF")
    {
        return Err(DxfError::Invalid("DXF has no EOF record".into()));
    }

    let mut parser = Parser {
        pairs: &pairs,
        position: 0,
        started,
        policy,
        acad_version: None,
        units: None,
        layers: BTreeMap::new(),
        blocks: BTreeMap::new(),
        entities: BTreeMap::new(),
        paper_layouts: BTreeSet::new(),
        diagnostics: Vec::new(),
        entity_sequence: 0,
    };
    parser.parse()?;
    parser.validate_block_references()?;
    let index = DxfDocumentIndex {
        schema_version: DXF_INDEX_SCHEMA_VERSION.into(),
        source_id: source_id.clone(),
        source_sha256: source_sha256.into(),
        parser: DXF_PARSER_ID.into(),
        parser_version: DXF_PARSER_VERSION.into(),
        acad_version: parser.acad_version,
        units: parser.units,
        model_space_name: "Model".into(),
        paper_layouts: parser.paper_layouts.into_iter().collect(),
        layers: parser.layers,
        blocks: parser.blocks,
        entities: parser.entities,
        diagnostics: parser.diagnostics,
    };
    validate_index(&index, source_id, source_sha256)?;
    Ok(index)
}

struct Parser<'a> {
    pairs: &'a [Pair],
    position: usize,
    started: Instant,
    policy: &'a DxfParsePolicy,
    acad_version: Option<String>,
    units: Option<String>,
    layers: BTreeMap<String, DxfLayer>,
    blocks: BTreeMap<String, DxfBlock>,
    entities: BTreeMap<String, DxfEntity>,
    paper_layouts: BTreeSet<String>,
    diagnostics: Vec<DxfDiagnostic>,
    entity_sequence: usize,
}

impl Parser<'_> {
    fn parse(&mut self) -> Result<(), DxfError> {
        while self.position < self.pairs.len() {
            check_time(self.started, self.policy)?;
            if self.is(0, "SECTION") {
                self.position += 1;
                let name = self
                    .pairs
                    .get(self.position)
                    .filter(|pair| pair.code == 2)
                    .ok_or_else(|| self.malformed("SECTION has no name"))?
                    .value
                    .trim()
                    .to_string();
                self.position += 1;
                match name.as_str() {
                    "HEADER" => self.parse_header()?,
                    "TABLES" => self.parse_tables()?,
                    "BLOCKS" => self.parse_blocks()?,
                    "ENTITIES" => self.parse_entities(None)?,
                    _ => self.skip_to(0, "ENDSEC"),
                }
            } else {
                self.position += 1;
            }
        }
        Ok(())
    }

    fn parse_header(&mut self) -> Result<(), DxfError> {
        while self.position < self.pairs.len() && !self.is(0, "ENDSEC") {
            if self.pairs[self.position].code == 9 {
                let variable = self.pairs[self.position].value.trim().to_string();
                self.position += 1;
                if let Some(value) = self.pairs.get(self.position) {
                    match variable.as_str() {
                        "$ACADVER" => self.acad_version = Some(value.value.trim().into()),
                        "$INSUNITS" => {
                            let code = parse_i32(value)?;
                            self.units = insertion_units(code).map(str::to_owned);
                        }
                        _ => {}
                    }
                }
            }
            self.position += 1;
        }
        Ok(())
    }

    fn parse_tables(&mut self) -> Result<(), DxfError> {
        while self.position < self.pairs.len() && !self.is(0, "ENDSEC") {
            if self.is(0, "LAYER") {
                let record = self.record();
                let name = value(&record, 2).unwrap_or("0").to_string();
                let flags = int(&record, 70).unwrap_or(0);
                let colour = int(&record, 62).unwrap_or(7);
                self.layers.insert(
                    name.clone(),
                    DxfLayer {
                        name,
                        frozen: flags & 1 != 0 || flags & 2 != 0,
                        hidden: colour < 0,
                        locked: flags & 4 != 0,
                    },
                );
            } else {
                self.position += 1;
            }
        }
        Ok(())
    }

    fn parse_blocks(&mut self) -> Result<(), DxfError> {
        while self.position < self.pairs.len() && !self.is(0, "ENDSEC") {
            if self.is(0, "BLOCK") {
                let record = self.record();
                let name = value(&record, 2).unwrap_or("*anonymous").to_string();
                let base_point = point(&record, 10, 20, 30);
                let before = self.entities.keys().cloned().collect::<BTreeSet<_>>();
                self.parse_entities(Some(name.clone()))?;
                let entity_ids = self
                    .entities
                    .keys()
                    .filter(|id| !before.contains(*id))
                    .cloned()
                    .collect();
                self.blocks.insert(
                    name.clone(),
                    DxfBlock {
                        name,
                        base_point,
                        entity_ids,
                    },
                );
            } else {
                self.position += 1;
            }
        }
        Ok(())
    }

    fn parse_entities(&mut self, block: Option<String>) -> Result<(), DxfError> {
        while self.position < self.pairs.len() {
            if self.is(0, "ENDSEC") || (block.is_some() && self.is(0, "ENDBLK")) {
                self.position += usize::from(block.is_some());
                return Ok(());
            }
            if self.pairs[self.position].code != 0 {
                self.position += 1;
                continue;
            }
            let record = self.record();
            if record.is_empty() {
                continue;
            }
            if record[0].value.trim().eq_ignore_ascii_case("POLYLINE") {
                let mut vertices = Vec::new();
                while self.is(0, "VERTEX") {
                    let vertex = self.record();
                    validate_numeric_record(&vertex)?;
                    if vertices.len() >= self.policy.max_vertices_per_entity {
                        return Err(DxfError::VertexLimit {
                            limit: self.policy.max_vertices_per_entity,
                        });
                    }
                    vertices.push(point(&vertex, 10, 20, 30));
                }
                if self.is(0, "SEQEND") {
                    self.record();
                }
                let closed = int(&record, 70).unwrap_or(0) & 1 != 0;
                self.add_entity_with_geometry(
                    record,
                    block.clone(),
                    Some(DxfGeometry::Polyline { vertices, closed }),
                )?;
            } else {
                self.add_entity(record, block.clone())?;
            }
        }
        Ok(())
    }

    fn add_entity(&mut self, record: Vec<Pair>, block: Option<String>) -> Result<(), DxfError> {
        self.add_entity_with_geometry(record, block, None)
    }

    fn add_entity_with_geometry(
        &mut self,
        record: Vec<Pair>,
        block: Option<String>,
        geometry_override: Option<DxfGeometry>,
    ) -> Result<(), DxfError> {
        if self.entities.len() >= self.policy.max_entities {
            return Err(DxfError::EntityLimit {
                limit: self.policy.max_entities,
            });
        }
        let entity_type = record[0].value.trim().to_ascii_uppercase();
        if matches!(entity_type.as_str(), "SEQEND" | "VERTEX") {
            return Ok(());
        }
        validate_numeric_record(&record)?;
        let handle = value(&record, 5).map(str::to_owned);
        let id = handle
            .as_ref()
            .map(|handle| format!("dxf:{handle}"))
            .unwrap_or_else(|| {
                let mut hash = Sha256::new();
                hash.update(self.entity_sequence.to_le_bytes());
                for pair in &record {
                    hash.update(pair.code.to_le_bytes());
                    hash.update(pair.value.as_bytes());
                }
                format!("dxf:generated:{:x}", hash.finalize())
            });
        self.entity_sequence += 1;
        if self.entities.contains_key(&id) {
            return Err(DxfError::Invalid(format!("duplicate DXF entity id `{id}`")));
        }
        let layer = value(&record, 8).unwrap_or("0").to_string();
        let layer_state = self.layers.get(&layer);
        let paper = int(&record, 67).unwrap_or(0) == 1;
        let layout = value(&record, 410).map(str::to_owned).unwrap_or_else(|| {
            if paper {
                "Layout1".into()
            } else {
                "Model".into()
            }
        });
        if paper {
            self.paper_layouts.insert(layout.clone());
        }
        let geometry = match geometry_override {
            Some(geometry) => geometry,
            None => self.geometry(&entity_type, &record, &id)?,
        };
        let transform = match &geometry {
            DxfGeometry::Insert {
                insertion,
                scale,
                rotation_degrees,
                ..
            } => ShelfTransform {
                translation: *insertion,
                rotation_degrees: [0.0, 0.0, *rotation_degrees],
                scale: *scale,
            },
            _ => identity_transform(),
        };
        if let DxfGeometry::Unsupported { .. } = &geometry {
            self.diagnostics.push(DxfDiagnostic {
                code: DxfDiagnosticCode::UnsupportedEntity,
                message: format!(
                    "DXF entity type `{entity_type}` is preserved but not interpreted"
                ),
                entity_id: Some(id.clone()),
            });
        }
        self.entities.insert(
            id.clone(),
            DxfEntity {
                id,
                handle,
                entity_type,
                layer,
                layout,
                space: if block.is_some() {
                    DxfSpace::BlockDefinition
                } else if paper {
                    DxfSpace::Paper
                } else {
                    DxfSpace::Model
                },
                hidden: layer_state.is_some_and(|layer| layer.hidden),
                frozen: layer_state.is_some_and(|layer| layer.frozen),
                block_name: block,
                transform,
                geometry,
            },
        );
        Ok(())
    }

    fn validate_block_references(&mut self) -> Result<(), DxfError> {
        let mut references = BTreeMap::<String, Vec<String>>::new();
        for entity in self.entities.values() {
            if let DxfGeometry::Insert { block_name, .. } = &entity.geometry {
                if !self.blocks.contains_key(block_name) {
                    self.diagnostics.push(DxfDiagnostic {
                        code: DxfDiagnosticCode::MissingBlock,
                        message: format!("DXF insert references missing block `{block_name}`"),
                        entity_id: Some(entity.id.clone()),
                    });
                }
                if let Some(owner) = &entity.block_name {
                    references
                        .entry(owner.clone())
                        .or_default()
                        .push(block_name.clone());
                }
            }
        }
        for block in self.blocks.keys() {
            let mut visiting = BTreeSet::new();
            validate_block_depth(
                block,
                0,
                &references,
                &mut visiting,
                self.policy.max_block_depth,
            )?;
        }
        Ok(())
    }

    fn geometry(&self, kind: &str, record: &[Pair], id: &str) -> Result<DxfGeometry, DxfError> {
        Ok(match kind {
            "LINE" => DxfGeometry::Line {
                start: point(record, 10, 20, 30),
                end: point(record, 11, 21, 31),
            },
            "LWPOLYLINE" => {
                let xs = values_f64(record, 10)?;
                let ys = values_f64(record, 20)?;
                if xs.len() != ys.len() || xs.len() > self.policy.max_vertices_per_entity {
                    return Err(DxfError::VertexLimit {
                        limit: self.policy.max_vertices_per_entity,
                    });
                }
                DxfGeometry::Polyline {
                    vertices: xs
                        .into_iter()
                        .zip(ys)
                        .map(|(x, y)| [x, y, number(record, 38).unwrap_or(0.0)])
                        .collect(),
                    closed: int(record, 70).unwrap_or(0) & 1 != 0,
                }
            }
            "CIRCLE" => DxfGeometry::Circle {
                center: point(record, 10, 20, 30),
                radius: required_number(record, 40, id)?,
            },
            "ARC" => DxfGeometry::Arc {
                center: point(record, 10, 20, 30),
                radius: required_number(record, 40, id)?,
                start_degrees: required_number(record, 50, id)?,
                end_degrees: required_number(record, 51, id)?,
            },
            "TEXT" | "MTEXT" => DxfGeometry::Text {
                insertion: point(record, 10, 20, 30),
                text: record
                    .iter()
                    .filter(|pair| matches!(pair.code, 1 | 3))
                    .map(|pair| pair.value.as_str())
                    .collect::<String>(),
                height: number(record, 40),
                rotation_degrees: number(record, 50).unwrap_or(0.0),
            },
            "DIMENSION" => DxfGeometry::Dimension {
                definition: point(record, 10, 20, 30),
                first_witness: optional_point(record, 13, 23, 33),
                second_witness: optional_point(record, 14, 24, 34),
                text: value(record, 1).map(str::to_owned),
                measurement: number(record, 42),
            },
            "INSERT" => DxfGeometry::Insert {
                block_name: value(record, 2).unwrap_or("*missing").to_string(),
                insertion: point(record, 10, 20, 30),
                scale: [
                    number(record, 41).unwrap_or(1.0),
                    number(record, 42).unwrap_or(1.0),
                    number(record, 43).unwrap_or(1.0),
                ],
                rotation_degrees: number(record, 50).unwrap_or(0.0),
                columns: int(record, 70).unwrap_or(1).max(1) as u32,
                rows: int(record, 71).unwrap_or(1).max(1) as u32,
            },
            other => DxfGeometry::Unsupported {
                entity_type: other.into(),
            },
        })
    }

    fn record(&mut self) -> Vec<Pair> {
        let start = self.position;
        self.position += 1;
        while self.position < self.pairs.len() && self.pairs[self.position].code != 0 {
            self.position += 1;
        }
        self.pairs[start..self.position].to_vec()
    }

    fn skip_to(&mut self, code: i16, value: &str) {
        while self.position < self.pairs.len() && !self.is(code, value) {
            self.position += 1;
        }
    }

    fn is(&self, code: i16, value: &str) -> bool {
        self.pairs
            .get(self.position)
            .is_some_and(|pair| pair.code == code && pair.value.trim() == value)
    }

    fn malformed(&self, message: &str) -> DxfError {
        DxfError::Malformed {
            line: self.pairs.get(self.position).map_or(0, |pair| pair.line),
            message: message.into(),
        }
    }
}

pub fn prepare_dxf_selection(
    project_dir: &Path,
    design_id: &DesignId,
    index: &DxfDocumentIndex,
    request: DxfSelectionRequest,
) -> Result<PreparedDxfSelection, DxfError> {
    let package =
        load_project_package(project_dir).map_err(|error| DxfError::Package(error.to_string()))?;
    if !package
        .manifest
        .designs
        .iter()
        .any(|entry| &entry.id == design_id)
    {
        return Err(DxfError::Invalid(
            "DXF selection design does not belong to project".into(),
        ));
    }
    if request.source_id != index.source_id
        || (request.entity_ids.is_empty()
            && request.layer_names.is_empty()
            && request.block_names.is_empty())
    {
        return Err(DxfError::Invalid(
            "DXF selection has no exact indexed entities, layers, or blocks".into(),
        ));
    }
    let role = request.view_role.ok_or_else(|| {
        DxfError::Invalid("DXF selection requires an explicit drawing view role".into())
    })?;
    let relation = request.relation_to_design.ok_or_else(|| {
        DxfError::Invalid(
            "DXF selection requires an explicit relation to design coordinates".into(),
        )
    })?;
    if !relation.confirmed
        || relation.confirmed_by.trim().is_empty()
        || relation.confirmed_at.trim().is_empty()
        || !relation.scale.is_finite()
        || relation.scale <= 0.0
    {
        return Err(DxfError::Invalid(
            "DXF view role and 3D relation must be explicitly confirmed".into(),
        ));
    }
    let mut selected_ids = BTreeSet::new();
    for id in &request.entity_ids {
        if !selected_ids.insert(id.clone()) {
            return Err(DxfError::Invalid(format!("duplicate DXF entity `{id}`")));
        }
        let entity = index
            .entities
            .get(id)
            .ok_or_else(|| DxfError::Invalid(format!("DXF entity `{id}` was not indexed")))?;
        if entity.layout != request.layout {
            return Err(DxfError::Invalid(format!(
                "DXF entity `{id}` belongs to layout `{}`, not `{}`",
                entity.layout, request.layout
            )));
        }
    }
    for layer_name in &request.layer_names {
        if !index.layers.contains_key(layer_name) {
            return Err(DxfError::Invalid(format!(
                "DXF layer `{layer_name}` was not indexed"
            )));
        }
        selected_ids.extend(
            index
                .entities
                .values()
                .filter(|entity| entity.layout == request.layout && entity.layer == *layer_name)
                .map(|entity| entity.id.clone()),
        );
    }
    for block_name in &request.block_names {
        if !index.blocks.contains_key(block_name) {
            return Err(DxfError::Invalid(format!(
                "DXF block `{block_name}` was not indexed"
            )));
        }
        selected_ids.extend(index.entities.values().filter_map(|entity| {
            if entity.layout == request.layout
                && matches!(
                    &entity.geometry,
                    DxfGeometry::Insert { block_name: inserted, .. } if inserted == block_name
                )
            {
                Some(entity.id.clone())
            } else {
                None
            }
        }));
    }
    if selected_ids.is_empty() {
        return Err(DxfError::Invalid(
            "DXF selection resolved to no entities in the requested layout".into(),
        ));
    }
    let selected = selected_ids
        .iter()
        .map(|id| &index.entities[id])
        .collect::<Vec<_>>();
    let resolved_entity_ids = selected_ids.into_iter().collect::<Vec<_>>();
    let source = inspect_source(project_dir, &request.source_id)?;
    if source.sha256 != index.source_sha256 {
        return Err(DxfError::Invalid(
            "DXF source changed after indexing".into(),
        ));
    }
    let shelf_item = ShelfItem {
        id: request.shelf_item_id.clone(),
        label: request.label,
        annotations: Vec::new(),
        confirmation: ShelfConfirmation {
            confirmed: true,
            confirmed_by: Some(relation.confirmed_by.clone()),
            confirmed_at: Some(relation.confirmed_at.clone()),
        },
        provenance: ShelfProvenance {
            created_at: request.created_at.clone(),
            created_by: request.created_by,
            method: "dxf_index_selection".into(),
            derivative_id: None,
        },
        drawing_context: Some(DrawingContext {
            view_role: role,
            orientation: relation.orientation.clone(),
            calibration: None,
        }),
        content: ShelfItemContent::CadSelection {
            source: ShelfSourceRef {
                source_id: source.id.clone(),
                source_sha256: source.sha256.clone(),
            },
            layout: request.layout.clone(),
            object_ids: resolved_entity_ids,
            transform: relation.transform,
            orientation: relation.orientation,
            scale: relation.scale,
        },
    };
    let mut observations = BTreeMap::new();
    for entity in selected {
        if let Some(observation) = observation_from_entity(
            entity,
            &request.shelf_item_id,
            &source.id,
            &source.sha256,
            &request.layout,
            role,
        ) {
            observations.insert(observation.id.clone(), observation);
        }
    }
    if observations.is_empty() {
        return Err(DxfError::Invalid(
            "selected DXF entities contain no supported observation geometry".into(),
        ));
    }
    Ok(PreparedDxfSelection {
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

fn observation_from_entity(
    entity: &DxfEntity,
    shelf_item_id: &str,
    source_id: &SourceId,
    source_sha256: &str,
    layout: &str,
    view_role: DrawingViewRole,
) -> Option<DrawingObservation> {
    let (source_geometry, feature, limitation) = match &entity.geometry {
        DxfGeometry::Line { start, end } => (
            ObservationSourceGeometry::Polyline {
                coordinates: vec![[start[0], start[1]], [end[0], end[1]]],
                closed: false,
            },
            ObservationFeature::Curve {
                curve_role: "dxf_linework_unclassified".into(),
            },
            None,
        ),
        DxfGeometry::Polyline { vertices, closed } => (
            ObservationSourceGeometry::Polyline {
                coordinates: vertices.iter().map(|point| [point[0], point[1]]).collect(),
                closed: *closed,
            },
            ObservationFeature::Curve {
                curve_role: "dxf_polyline_unclassified".into(),
            },
            None,
        ),
        DxfGeometry::Circle { center, radius } => (
            sampled_arc(*center, *radius, 0.0, 360.0, true),
            ObservationFeature::Curve {
                curve_role: "dxf_circle_unclassified".into(),
            },
            Some(
                "Circle is sampled for observation display; exact circle remains in the DXF index.",
            ),
        ),
        DxfGeometry::Arc {
            center,
            radius,
            start_degrees,
            end_degrees,
        } => (
            sampled_arc(*center, *radius, *start_degrees, *end_degrees, false),
            ObservationFeature::Curve {
                curve_role: "dxf_arc_unclassified".into(),
            },
            Some("Arc is sampled for observation display; exact arc remains in the DXF index."),
        ),
        DxfGeometry::Text {
            insertion, text, ..
        } => (
            ObservationSourceGeometry::Point {
                coordinate: [insertion[0], insertion[1]],
            },
            ObservationFeature::Label { text: text.clone() },
            None,
        ),
        DxfGeometry::Dimension {
            definition,
            first_witness,
            second_witness,
            text,
            measurement,
        } => {
            let first = first_witness.unwrap_or(*definition);
            let second = second_witness.unwrap_or(*definition);
            (
                ObservationSourceGeometry::Polyline {
                    coordinates: vec![[first[0], first[1]], [second[0], second[1]]],
                    closed: false,
                },
                ObservationFeature::Dimension {
                    label: text.clone().unwrap_or_else(|| "DXF dimension".into()),
                    measured: crate::MeasuredValue {
                        value: measurement.unwrap_or(0.0),
                        unit: "dxf_drawing_unit".into(),
                    },
                    first_witness: [first[0], first[1]],
                    second_witness: [second[0], second[1]],
                },
                measurement.is_none().then_some(
                    "DXF dimension has no explicit measurement; displayed geometry is not a confirmed value.",
                ),
            )
        }
        DxfGeometry::Insert {
            insertion,
            block_name,
            ..
        } => (
            ObservationSourceGeometry::Point {
                coordinate: [insertion[0], insertion[1]],
            },
            ObservationFeature::Symbol {
                symbol_kind: "dxf_block_insert".into(),
                text: Some(block_name.clone()),
            },
            Some(
                "Block insert remains symbolic until its nested geometry and 3D relation are reviewed.",
            ),
        ),
        DxfGeometry::Unsupported { .. } => return None,
    };
    let uncertainty = limitation
        .map(|message| {
            vec![InterpretationUncertainty {
                kind: InterpretationUncertaintyKind::ParserLimitation,
                message: message.into(),
            }]
        })
        .unwrap_or_default();
    Some(DrawingObservation {
        id: format!("dxf-observation-{}", entity.id.replace(':', "-")),
        shelf_item_id: shelf_item_id.into(),
        source_id: source_id.clone(),
        source_sha256: source_sha256.into(),
        source_locator: DrawingSourceLocator::CadEntities {
            layout: layout.into(),
            coordinate_space: "dxf_world_coordinates".into(),
            entity_ids: vec![entity.id.clone()],
            transforms: vec![entity.transform.clone()],
        },
        view_role,
        source_geometry,
        design_geometry: None,
        extraction: ObservationExtraction {
            method: InterpretationMethod::NativeVectorExtraction,
            producer: DXF_PARSER_ID.into(),
            producer_version: DXF_PARSER_VERSION.into(),
            confidence: 1.0,
            uncertainty,
        },
        confirmation: ObservationConfirmation::Unconfirmed,
        feature,
    })
}

fn sampled_arc(
    center: [f64; 3],
    radius: f64,
    start: f64,
    end: f64,
    closed: bool,
) -> ObservationSourceGeometry {
    let span = if end <= start {
        end + 360.0 - start
    } else {
        end - start
    };
    let steps = 32usize;
    let coordinates = (0..=steps)
        .map(|index| {
            let angle = (start + span * index as f64 / steps as f64).to_radians();
            [
                center[0] + radius * angle.cos(),
                center[1] + radius * angle.sin(),
            ]
        })
        .collect();
    ObservationSourceGeometry::Polyline {
        coordinates,
        closed,
    }
}

fn validate_index(
    index: &DxfDocumentIndex,
    source_id: &SourceId,
    source_sha256: &str,
) -> Result<(), DxfError> {
    if index.schema_version != DXF_INDEX_SCHEMA_VERSION
        || &index.source_id != source_id
        || index.source_sha256 != source_sha256
        || index.parser != DXF_PARSER_ID
        || index.parser_version != DXF_PARSER_VERSION
    {
        return Err(DxfError::Invalid(
            "DXF index identity or schema is invalid".into(),
        ));
    }
    for (id, entity) in &index.entities {
        if id != &entity.id || !finite_entity(entity) {
            return Err(DxfError::Invalid(
                "DXF entity identity or geometry is invalid".into(),
            ));
        }
    }
    Ok(())
}

fn finite_entity(entity: &DxfEntity) -> bool {
    let transform = entity
        .transform
        .translation
        .into_iter()
        .chain(entity.transform.rotation_degrees)
        .chain(entity.transform.scale)
        .all(f64::is_finite);
    transform
        && match &entity.geometry {
            DxfGeometry::Line { start, end } => {
                start.iter().chain(end).all(|value| value.is_finite())
            }
            DxfGeometry::Polyline { vertices, .. } => {
                vertices.iter().flatten().all(|value| value.is_finite())
            }
            DxfGeometry::Circle { center, radius } => {
                center.iter().all(|value| value.is_finite()) && radius.is_finite() && *radius > 0.0
            }
            DxfGeometry::Arc {
                center,
                radius,
                start_degrees,
                end_degrees,
            } => {
                center.iter().all(|value| value.is_finite())
                    && [radius, start_degrees, end_degrees]
                        .iter()
                        .all(|value| value.is_finite())
                    && *radius > 0.0
            }
            DxfGeometry::Text {
                insertion,
                height,
                rotation_degrees,
                ..
            } => {
                insertion.iter().all(|value| value.is_finite())
                    && height.is_none_or(|value| value.is_finite())
                    && rotation_degrees.is_finite()
            }
            DxfGeometry::Dimension {
                definition,
                first_witness,
                second_witness,
                measurement,
                ..
            } => {
                definition.iter().all(|value| value.is_finite())
                    && first_witness
                        .iter()
                        .chain(second_witness)
                        .flatten()
                        .all(|value| value.is_finite())
                    && measurement.is_none_or(|value| value.is_finite())
            }
            DxfGeometry::Insert {
                insertion,
                scale,
                rotation_degrees,
                ..
            } => {
                insertion.iter().chain(scale).all(|value| value.is_finite())
                    && rotation_degrees.is_finite()
            }
            DxfGeometry::Unsupported { .. } => true,
        }
}

fn identity_transform() -> ShelfTransform {
    ShelfTransform {
        translation: [0.0; 3],
        rotation_degrees: [0.0; 3],
        scale: [1.0; 3],
    }
}

fn validate_numeric_record(record: &[Pair]) -> Result<(), DxfError> {
    for pair in record {
        let is_float = matches!(
            pair.code,
            10..=59 | 110..=149 | 210..=239 | 460..=469 | 1010..=1059
        );
        let is_integer = matches!(pair.code, 60..=79 | 90..=99 | 160..=179 | 270..=289 | 370..=389 | 400..=409 | 420..=459 | 1060..=1071);
        let valid = if is_float {
            pair.value.trim().parse::<f64>().is_ok()
        } else if is_integer {
            pair.value.trim().parse::<i64>().is_ok()
        } else {
            true
        };
        if !valid {
            return Err(DxfError::Malformed {
                line: pair.line,
                message: format!("group {} has an invalid numeric value", pair.code),
            });
        }
    }
    Ok(())
}

fn validate_block_depth(
    block: &str,
    depth: usize,
    references: &BTreeMap<String, Vec<String>>,
    visiting: &mut BTreeSet<String>,
    limit: usize,
) -> Result<(), DxfError> {
    if depth > limit {
        return Err(DxfError::Invalid(format!(
            "DXF block nesting exceeds {limit} levels"
        )));
    }
    if !visiting.insert(block.to_string()) {
        return Err(DxfError::Invalid(format!(
            "DXF block reference cycle includes `{block}`"
        )));
    }
    if let Some(children) = references.get(block) {
        for child in children {
            if references.contains_key(child) {
                validate_block_depth(child, depth + 1, references, visiting, limit)?;
            }
        }
    }
    visiting.remove(block);
    Ok(())
}

fn value(record: &[Pair], code: i16) -> Option<&str> {
    record
        .iter()
        .find(|pair| pair.code == code)
        .map(|pair| pair.value.trim())
}
fn number(record: &[Pair], code: i16) -> Option<f64> {
    value(record, code)?.parse().ok()
}
fn required_number(record: &[Pair], code: i16, id: &str) -> Result<f64, DxfError> {
    number(record, code)
        .ok_or_else(|| DxfError::Invalid(format!("DXF entity `{id}` has no numeric group {code}")))
}
fn int(record: &[Pair], code: i16) -> Option<i32> {
    value(record, code)?.parse().ok()
}
fn parse_i32(pair: &Pair) -> Result<i32, DxfError> {
    pair.value.trim().parse().map_err(|_| DxfError::Malformed {
        line: pair.line,
        message: "expected integer group value".into(),
    })
}
fn values_f64(record: &[Pair], code: i16) -> Result<Vec<f64>, DxfError> {
    record
        .iter()
        .filter(|pair| pair.code == code)
        .map(|pair| {
            pair.value.trim().parse().map_err(|_| DxfError::Malformed {
                line: pair.line,
                message: "expected numeric group value".into(),
            })
        })
        .collect()
}
fn point(record: &[Pair], x: i16, y: i16, z: i16) -> [f64; 3] {
    [
        number(record, x).unwrap_or(0.0),
        number(record, y).unwrap_or(0.0),
        number(record, z).unwrap_or(0.0),
    ]
}
fn optional_point(record: &[Pair], x: i16, y: i16, z: i16) -> Option<[f64; 3]> {
    value(record, x)?;
    Some(point(record, x, y, z))
}
fn insertion_units(value: i32) -> Option<&'static str> {
    match value {
        1 => Some("in"),
        2 => Some("ft"),
        4 => Some("mm"),
        5 => Some("cm"),
        6 => Some("m"),
        7 => Some("km"),
        8 => Some("microin"),
        9 => Some("mil"),
        10 => Some("yd"),
        11 => Some("angstrom"),
        12 => Some("nm"),
        13 => Some("micrometre"),
        14 => Some("dm"),
        15 => Some("dam"),
        16 => Some("hm"),
        17 => Some("gm"),
        18 => Some("au"),
        19 => Some("light_year"),
        20 => Some("parsec"),
        _ => None,
    }
}
fn check_time(started: Instant, policy: &DxfParsePolicy) -> Result<(), DxfError> {
    if started.elapsed() > Duration::from_millis(policy.max_parse_millis) {
        Err(DxfError::TimeLimit {
            limit_millis: policy.max_parse_millis,
        })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SourceImportRequest, import_source};
    use std::fs;

    fn source_identity() -> (SourceId, String) {
        let hash = "a".repeat(64);
        (SourceId::from_sha256(&hash).unwrap(), hash)
    }

    fn wrap(sections: &str) -> Vec<u8> {
        format!("{sections}  0\nEOF\n").into_bytes()
    }

    fn header(units: Option<i32>) -> String {
        let units = units
            .map(|value| format!("  9\n$INSUNITS\n 70\n{value}\n"))
            .unwrap_or_default();
        format!("  0\nSECTION\n  2\nHEADER\n  9\n$ACADVER\n  1\nAC1032\n{units}  0\nENDSEC\n")
    }

    fn entity_section(records: &str) -> String {
        format!("  0\nSECTION\n  2\nENTITIES\n{records}  0\nENDSEC\n")
    }

    #[test]
    fn unitless_and_declared_model_and_paper_spaces_preserve_text_dimensions_and_ids() {
        let (source_id, hash) = source_identity();
        let records = concat!(
            "  0\nLINE\n  5\n10A\n  8\nGRID\n 10\n0\n 20\n0\n 11\n6\n 21\n0\n",
            "  0\nTEXT\n  5\n10B\n 67\n1\n410\nElevation A\n  8\nANNO\n 10\n2\n 20\n3\n 40\n0.25\n  1\nLEVEL 1\n",
            "  0\nDIMENSION\n  5\n10C\n 67\n1\n410\nElevation A\n 10\n0\n 20\n1\n 13\n0\n 23\n0\n 14\n6\n 24\n0\n 42\n6\n  1\n6000\n"
        );
        let declared = parse_ascii_dxf(
            &wrap(&(header(Some(4)) + &entity_section(records))),
            &source_id,
            &hash,
            &DxfParsePolicy::default(),
        )
        .unwrap();
        assert_eq!(declared.units.as_deref(), Some("mm"));
        assert_eq!(declared.acad_version.as_deref(), Some("AC1032"));
        assert_eq!(declared.entities["dxf:10A"].space, DxfSpace::Model);
        assert_eq!(declared.entities["dxf:10B"].space, DxfSpace::Paper);
        assert_eq!(declared.paper_layouts, vec!["Elevation A"]);
        assert!(matches!(
            declared.entities["dxf:10B"].geometry,
            DxfGeometry::Text { .. }
        ));
        assert!(matches!(
            declared.entities["dxf:10C"].geometry,
            DxfGeometry::Dimension {
                measurement: Some(6.0),
                ..
            }
        ));

        let unitless = parse_ascii_dxf(
            &wrap(&(header(None) + &entity_section(records))),
            &source_id,
            &hash,
            &DxfParsePolicy::default(),
        )
        .unwrap();
        assert!(unitless.units.is_none());
    }

    #[test]
    fn nested_rotated_blocks_and_hidden_frozen_layers_retain_exact_state() {
        let (source_id, hash) = source_identity();
        let tables = concat!(
            "  0\nSECTION\n  2\nTABLES\n",
            "  0\nLAYER\n  2\nFROZEN\n 70\n1\n 62\n7\n",
            "  0\nLAYER\n  2\nHIDDEN\n 70\n0\n 62\n-3\n",
            "  0\nENDSEC\n"
        );
        let blocks = concat!(
            "  0\nSECTION\n  2\nBLOCKS\n",
            "  0\nBLOCK\n  2\nINNER\n 10\n0\n 20\n0\n",
            "  0\nLINE\n  5\nB1\n  8\nFROZEN\n 10\n0\n 20\n0\n 11\n1\n 21\n0\n",
            "  0\nENDBLK\n",
            "  0\nBLOCK\n  2\nOUTER\n 10\n0\n 20\n0\n",
            "  0\nINSERT\n  5\nB2\n  8\nHIDDEN\n  2\nINNER\n 10\n2\n 20\n3\n 41\n2\n 42\n2\n 50\n30\n",
            "  0\nENDBLK\n  0\nENDSEC\n"
        );
        let entities =
            entity_section("  0\nINSERT\n  5\nE1\n  2\nOUTER\n 10\n10\n 20\n20\n 50\n90\n");
        let index = parse_ascii_dxf(
            &wrap(&(header(Some(6)) + tables + blocks + &entities)),
            &source_id,
            &hash,
            &DxfParsePolicy::default(),
        )
        .unwrap();
        assert!(index.entities["dxf:B1"].frozen);
        assert!(index.entities["dxf:B2"].hidden);
        assert_eq!(index.blocks["INNER"].entity_ids, vec!["dxf:B1"]);
        assert_eq!(index.blocks["OUTER"].entity_ids, vec!["dxf:B2"]);
        assert_eq!(
            index.entities["dxf:B2"].transform.translation,
            [2.0, 3.0, 0.0]
        );
        assert_eq!(index.entities["dxf:B2"].transform.rotation_degrees[2], 30.0);
        assert_eq!(index.entities["dxf:E1"].transform.rotation_degrees[2], 90.0);
    }

    #[test]
    fn unsupported_malformed_and_excessive_entities_fail_or_diagnose_without_partial_index() {
        let (source_id, hash) = source_identity();
        let spline = parse_ascii_dxf(
            &wrap(&(header(None) + &entity_section("  0\nSPLINE\n  5\nS1\n"))),
            &source_id,
            &hash,
            &DxfParsePolicy::default(),
        )
        .unwrap();
        assert!(matches!(
            spline.entities["dxf:S1"].geometry,
            DxfGeometry::Unsupported { .. }
        ));
        assert_eq!(
            spline.diagnostics[0].code,
            DxfDiagnosticCode::UnsupportedEntity
        );

        assert!(matches!(
            parse_ascii_dxf(
                b"  0\nSECTION\n  2",
                &source_id,
                &hash,
                &DxfParsePolicy::default()
            ),
            Err(DxfError::Invalid(_)) | Err(DxfError::Malformed { .. })
        ));
        let limited = DxfParsePolicy {
            max_entities: 1,
            ..DxfParsePolicy::default()
        };
        let two = entity_section(concat!(
            "  0\nLINE\n  5\n1\n 10\n0\n 20\n0\n 11\n1\n 21\n0\n",
            "  0\nLINE\n  5\n2\n 10\n0\n 20\n1\n 11\n1\n 21\n1\n"
        ));
        assert!(matches!(
            parse_ascii_dxf(&wrap(&(header(None) + &two)), &source_id, &hash, &limited),
            Err(DxfError::EntityLimit { limit: 1 })
        ));

        let malformed_number =
            entity_section("  0\nLINE\n  5\nBAD\n 10\nnot-a-number\n 20\n0\n 11\n1\n 21\n0\n");
        assert!(matches!(
            parse_ascii_dxf(
                &wrap(&(header(None) + &malformed_number)),
                &source_id,
                &hash,
                &DxfParsePolicy::default()
            ),
            Err(DxfError::Malformed { .. })
        ));

        let classic_polyline = entity_section(concat!(
            "  0\nPOLYLINE\n  5\nP1\n 70\n1\n",
            "  0\nVERTEX\n 10\n0\n 20\n0\n 30\n0\n",
            "  0\nVERTEX\n 10\n2\n 20\n3\n 30\n4\n",
            "  0\nSEQEND\n"
        ));
        let classic = parse_ascii_dxf(
            &wrap(&(header(None) + &classic_polyline)),
            &source_id,
            &hash,
            &DxfParsePolicy::default(),
        )
        .unwrap();
        assert!(matches!(
            &classic.entities["dxf:P1"].geometry,
            DxfGeometry::Polyline { vertices, closed }
                if *closed && vertices == &vec![[0.0, 0.0, 0.0], [2.0, 3.0, 4.0]]
        ));

        let missing_block = parse_ascii_dxf(
            &wrap(
                &(header(None)
                    + &entity_section(
                        "  0\nINSERT\n  5\nI1\n  2\nDOES_NOT_EXIST\n 10\n0\n 20\n0\n",
                    )),
            ),
            &source_id,
            &hash,
            &DxfParsePolicy::default(),
        )
        .unwrap();
        assert!(
            missing_block
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == DxfDiagnosticCode::MissingBlock)
        );
    }

    #[test]
    fn selection_requires_confirmed_view_relation_and_creates_only_unconfirmed_observations() {
        let temporary = std::env::temp_dir().join(format!(
            "fraia-dxf-selection-{}",
            crate::utils::timestamp_id()
        ));
        fs::create_dir(&temporary).unwrap();
        let project = temporary.join("project");
        let package = crate::create_named_project_package(&project, "DXF selection").unwrap();
        let design_id = package.designs[0].manifest.id.clone();
        let file = temporary.join("plan.dxf");
        fs::write(
            &file,
            wrap(
                &(header(Some(4))
                    + &entity_section("  0\nLINE\n  5\nL1\n 10\n0\n 20\n0\n 11\n6000\n 21\n0\n")),
            ),
        )
        .unwrap();
        let imported = import_source(
            &project,
            SourceImportRequest {
                selected_path: file,
                display_alias: Some("Architect plan.dxf".into()),
                expected_media_type: Some(SourceMediaType::Dxf),
            },
        )
        .unwrap();
        let indexed = index_and_store_dxf(
            &project,
            &imported.record.id,
            &SourceLibraryPolicy::default(),
            &DxfParsePolicy::default(),
        )
        .unwrap();
        let request = DxfSelectionRequest {
            shelf_item_id: "cad-plan".into(),
            label: "Architect plan".into(),
            source_id: imported.record.id,
            layout: "Model".into(),
            entity_ids: vec!["dxf:L1".into()],
            layer_names: Vec::new(),
            block_names: Vec::new(),
            view_role: Some(DrawingViewRole::Plan),
            relation_to_design: Some(DxfViewRelation {
                confirmed: true,
                confirmed_by: "engineer".into(),
                confirmed_at: "2026-08-14T00:00:00Z".into(),
                transform: identity_transform(),
                orientation: ShelfOrientation {
                    forward: [0.0, 0.0, -1.0],
                    up: [0.0, 1.0, 0.0],
                },
                scale: 0.001,
            }),
            created_at: "2026-08-14T00:00:00Z".into(),
            created_by: "engineer".into(),
            interpretation_parent_revision_id: None,
        };
        let prepared =
            prepare_dxf_selection(&project, &design_id, &indexed.index, request.clone()).unwrap();
        assert!(prepared.shelf_item.confirmation.confirmed);
        assert!(
            prepared
                .interpretation
                .observations
                .values()
                .all(|observation| observation.design_geometry.is_none()
                    && observation.confirmation == ObservationConfirmation::Unconfirmed
                    && matches!(observation.feature, ObservationFeature::Curve { .. })
                    && matches!(
                        &observation.source_locator,
                        DrawingSourceLocator::CadEntities { entity_ids, transforms, .. }
                            if entity_ids == &["dxf:L1"] && transforms == &[identity_transform()]
                    ))
        );
        let mut unconfirmed = request;
        unconfirmed.relation_to_design.as_mut().unwrap().confirmed = false;
        assert!(prepare_dxf_selection(&project, &design_id, &indexed.index, unconfirmed).is_err());
        assert!(
            index_and_store_dxf(
                &project,
                &prepared
                    .interpretation
                    .observations
                    .values()
                    .next()
                    .unwrap()
                    .source_id,
                &SourceLibraryPolicy::default(),
                &DxfParsePolicy::default(),
            )
            .unwrap()
            .resumed
        );
        fs::remove_dir_all(temporary).unwrap();
    }
}
