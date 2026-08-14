//! Bounded read-only glTF/GLB, OBJ, and STL reference indexing.

use crate::{
    DesignId, DrawingContext, DrawingViewRole, ShelfCamera, ShelfConfirmation, ShelfItem,
    ShelfItemContent, ShelfOrientation, ShelfProvenance, ShelfSectionPlane, ShelfSourceRef,
    ShelfTransform, ShelfUnitCalibration, SourceDerivative, SourceDerivativeKind,
    SourceDerivativeRequest, SourceId, SourceLibraryError, SourceLibraryPolicy, SourceMediaType,
    inspect_source, load_project_package, read_source_original, store_source_derivative,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};

pub const MESH_INDEX_SCHEMA_VERSION: &str = "fraia.mesh-index.v1";
pub const MESH_PARSER_ID: &str = "fraia.neutral-mesh.bounded";
pub const MESH_PARSER_VERSION: &str = "1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeshFormat {
    Gltf,
    Glb,
    Obj,
    Stl,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeshBounds {
    pub minimum: [f64; 3],
    pub maximum: [f64; 3],
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeshObject {
    pub id: String,
    pub name: Option<String>,
    pub group: Option<String>,
    pub frame: ShelfTransform,
    /// Exact source-local object transform, in column-major order.
    pub source_matrix: [f64; 16],
    pub bounds: MeshBounds,
    pub vertex_count: u64,
    pub triangle_count: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeshDiagnosticCode {
    UnitsMissing,
    UnsupportedTopology,
    ExternalResource,
    Malformed,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeshDiagnostic {
    pub code: MeshDiagnosticCode,
    pub object_id: Option<String>,
    pub message: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeshDocumentIndex {
    pub schema_version: String,
    pub source_id: SourceId,
    pub source_sha256: String,
    pub parser: String,
    pub parser_version: String,
    pub format: MeshFormat,
    pub units: Option<String>,
    pub coordinate_frame: String,
    pub objects: BTreeMap<String, MeshObject>,
    pub bounds: MeshBounds,
    pub vertex_count: u64,
    pub triangle_count: u64,
    pub diagnostics: Vec<MeshDiagnostic>,
}
#[derive(Debug, Clone)]
pub struct MeshParsePolicy {
    pub max_bytes: usize,
    pub max_vertices: u64,
    pub max_triangles: u64,
    pub max_objects: usize,
    pub max_millis: u64,
}
impl Default for MeshParsePolicy {
    fn default() -> Self {
        Self {
            max_bytes: 256 * 1024 * 1024,
            max_vertices: 20_000_000,
            max_triangles: 20_000_000,
            max_objects: 1_000_000,
            max_millis: 120_000,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeshIndexResult {
    pub index: MeshDocumentIndex,
    pub derivative: SourceDerivative,
    pub resumed: bool,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ManagedMeshContent {
    pub source: crate::SourceRecord,
    pub bytes: Vec<u8>,
}
pub type MeshCalibration = ShelfUnitCalibration;
pub type MeshSectionPlane = ShelfSectionPlane;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeshSavedViewRequest {
    pub shelf_item_id: String,
    pub label: String,
    pub source_id: SourceId,
    pub object_ids: Vec<String>,
    pub camera: ShelfCamera,
    pub transform: ShelfTransform,
    pub orientation: ShelfOrientation,
    pub scale: f64,
    pub section_planes: Vec<MeshSectionPlane>,
    pub calibration: Option<MeshCalibration>,
    pub created_at: String,
    pub created_by: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreparedMeshSavedView {
    pub shelf_item: ShelfItem,
    pub section_planes: Vec<MeshSectionPlane>,
    pub source_units: Option<String>,
    pub units_to_metres: f64,
}

#[derive(Debug)]
pub enum MeshError {
    Invalid(String),
    Malformed(String),
    Limit(String),
    Cancelled,
    TimeLimit,
    Source(SourceLibraryError),
    Json(serde_json::Error),
    Package(String),
}
impl std::fmt::Display for MeshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(v) | Self::Malformed(v) | Self::Limit(v) | Self::Package(v) => {
                f.write_str(v)
            }
            Self::Cancelled => f.write_str("mesh indexing was cancelled"),
            Self::TimeLimit => f.write_str("mesh indexing exceeded its time limit"),
            Self::Source(v) => write!(f, "{v}"),
            Self::Json(v) => write!(f, "{v}"),
        }
    }
}
impl std::error::Error for MeshError {}
impl From<SourceLibraryError> for MeshError {
    fn from(v: SourceLibraryError) -> Self {
        Self::Source(v)
    }
}
impl From<serde_json::Error> for MeshError {
    fn from(v: serde_json::Error) -> Self {
        Self::Json(v)
    }
}

pub fn read_managed_mesh_content(
    project: &std::path::Path,
    source_id: &SourceId,
    max_bytes: usize,
) -> Result<ManagedMeshContent, MeshError> {
    let source = inspect_source(project, source_id)?;
    if !matches!(
        source.detected_media_type,
        SourceMediaType::Gltf | SourceMediaType::Glb | SourceMediaType::Obj | SourceMediaType::Stl
    ) {
        return Err(MeshError::Invalid(
            "source is not a sniffed Phase 1 neutral mesh".into(),
        ));
    }
    if source.byte_size > max_bytes as u64 {
        return Err(MeshError::Limit(format!(
            "managed mesh content exceeds {max_bytes} bytes"
        )));
    }
    let bytes = read_source_original(project, source_id)?;
    if bytes.len() != source.byte_size as usize {
        return Err(MeshError::Invalid(
            "managed mesh content size changed after verification".into(),
        ));
    }
    Ok(ManagedMeshContent { source, bytes })
}

pub fn index_and_store_mesh(
    project: &std::path::Path,
    source_id: &SourceId,
    source_policy: &SourceLibraryPolicy,
    policy: &MeshParsePolicy,
) -> Result<MeshIndexResult, MeshError> {
    index_and_store_mesh_with_cancel(project, source_id, source_policy, policy, || false)
}

pub fn index_and_store_mesh_with_cancel<F: FnMut() -> bool>(
    project: &std::path::Path,
    source_id: &SourceId,
    source_policy: &SourceLibraryPolicy,
    policy: &MeshParsePolicy,
    mut cancelled: F,
) -> Result<MeshIndexResult, MeshError> {
    if cancelled() {
        return Err(MeshError::Cancelled);
    }
    let source = inspect_source(project, source_id)?;
    if !matches!(
        source.detected_media_type,
        SourceMediaType::Gltf | SourceMediaType::Glb | SourceMediaType::Obj | SourceMediaType::Stl
    ) {
        return Err(MeshError::Invalid(
            "source is not a sniffed Phase 1 neutral mesh".into(),
        ));
    }
    for derivative in crate::source_derivatives(project, source_id)? {
        if derivative.kind == SourceDerivativeKind::MeshIndex
            && derivative.parser == MESH_PARSER_ID
            && derivative.parser_version == MESH_PARSER_VERSION
        {
            let (_, payload) = crate::read_source_derivative(project, &derivative.id)?;
            let index: MeshDocumentIndex = serde_json::from_slice(&payload)?;
            validate_index(&index, &source.id, &source.sha256)?;
            return Ok(MeshIndexResult {
                index,
                derivative,
                resumed: true,
            });
        }
    }
    let bytes = read_source_original(project, source_id)?;
    let index = parse_mesh(
        &bytes,
        source.detected_media_type,
        &source.id,
        &source.sha256,
        policy,
        &mut cancelled,
    )?;
    let derivative = store_source_derivative(
        project,
        SourceDerivativeRequest {
            source_id: source.id,
            kind: SourceDerivativeKind::MeshIndex,
            payload: serde_json::to_vec(&index)?,
            media_type: "application/vnd.fraia.mesh-index+json".into(),
            parser: MESH_PARSER_ID.into(),
            parser_version: MESH_PARSER_VERSION.into(),
            units: index.units.clone(),
            coordinate_system: Some(index.coordinate_frame.clone()),
            warnings: Vec::new(),
        },
        source_policy,
    )?;
    Ok(MeshIndexResult {
        index,
        derivative,
        resumed: false,
    })
}
pub fn parse_mesh<F: FnMut() -> bool>(
    bytes: &[u8],
    media: SourceMediaType,
    source_id: &SourceId,
    hash: &str,
    policy: &MeshParsePolicy,
    mut cancelled: F,
) -> Result<MeshDocumentIndex, MeshError> {
    let started = Instant::now();
    if bytes.len() > policy.max_bytes {
        return Err(MeshError::Limit(format!(
            "mesh exceeds {} bytes",
            policy.max_bytes
        )));
    }
    if cancelled() {
        return Err(MeshError::Cancelled);
    }
    let (format, units, frame, objects, mut diagnostics) = match media {
        SourceMediaType::Obj => {
            let (objects, d) = parse_obj(bytes, policy, started, &mut cancelled)?;
            (
                MeshFormat::Obj,
                None,
                "obj_unspecified_axes".into(),
                objects,
                d,
            )
        }
        SourceMediaType::Stl => {
            let (objects, d) = parse_stl(bytes, policy, started, &mut cancelled)?;
            (
                MeshFormat::Stl,
                None,
                "stl_unspecified_axes".into(),
                objects,
                d,
            )
        }
        SourceMediaType::Gltf => {
            let (objects, d) = parse_gltf_json(bytes, policy, started, &mut cancelled)?;
            (
                MeshFormat::Gltf,
                Some("m".into()),
                "gltf_y_up_right_handed".into(),
                objects,
                d,
            )
        }
        SourceMediaType::Glb => {
            let json = glb_json(bytes)?;
            let (objects, d) = parse_gltf_json(json, policy, started, &mut cancelled)?;
            (
                MeshFormat::Glb,
                Some("m".into()),
                "gltf_y_up_right_handed".into(),
                objects,
                d,
            )
        }
        _ => return Err(MeshError::Invalid("unsupported mesh media type".into())),
    };
    if units.is_none() {
        diagnostics.push(MeshDiagnostic{code:MeshDiagnosticCode::UnitsMissing,object_id:None,message:"Source units are absent; explicit calibration is required before the view can be used.".into()});
    }
    let bounds = combined_bounds(objects.values().map(|o| &o.bounds))
        .ok_or_else(|| MeshError::Invalid("mesh contains no bounded reference geometry".into()))?;
    let vertex_count = objects.values().map(|o| o.vertex_count).sum();
    let triangle_count = objects.values().map(|o| o.triangle_count).sum();
    let index = MeshDocumentIndex {
        schema_version: MESH_INDEX_SCHEMA_VERSION.into(),
        source_id: source_id.clone(),
        source_sha256: hash.into(),
        parser: MESH_PARSER_ID.into(),
        parser_version: MESH_PARSER_VERSION.into(),
        format,
        units,
        coordinate_frame: frame,
        objects,
        bounds,
        vertex_count,
        triangle_count,
        diagnostics,
    };
    validate_index(&index, source_id, hash)?;
    Ok(index)
}

fn parse_obj<F: FnMut() -> bool>(
    bytes: &[u8],
    p: &MeshParsePolicy,
    started: Instant,
    cancelled: &mut F,
) -> Result<(BTreeMap<String, MeshObject>, Vec<MeshDiagnostic>), MeshError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| MeshError::Malformed("OBJ is not UTF-8 text".into()))?;
    let mut vertices = Vec::<[f64; 3]>::new();
    let mut groups = BTreeMap::<(String, String), (BTreeSet<usize>, u64)>::new();
    let mut current_object = "default".to_string();
    let mut current_group = "default".to_string();
    let mut diagnostics = Vec::new();
    for (line_number, line) in text.lines().enumerate() {
        check(started, p, cancelled)?;
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("v ") {
            let values = rest
                .split_whitespace()
                .map(str::parse::<f64>)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| {
                    MeshError::Malformed(format!(
                        "OBJ vertex is invalid at line {}",
                        line_number + 1
                    ))
                })?;
            if values.len() < 3 {
                return Err(MeshError::Malformed(
                    "OBJ vertex has fewer than three coordinates".into(),
                ));
            }
            vertices.push([values[0], values[1], values[2]]);
            if vertices.len() as u64 > p.max_vertices {
                return Err(MeshError::Limit("OBJ vertex limit exceeded".into()));
            }
        } else if let Some(name) = line.strip_prefix("o ") {
            current_object = nonempty_name(name);
        } else if let Some(name) = line.strip_prefix("g ") {
            current_group = nonempty_name(name);
        } else if let Some(rest) = line.strip_prefix("f ") {
            let refs = rest
                .split_whitespace()
                .filter_map(|token| token.split('/').next()?.parse::<isize>().ok())
                .map(|id| {
                    if id < 0 {
                        vertices.len() as isize + id
                    } else {
                        id - 1
                    }
                })
                .collect::<Vec<_>>();
            if refs.len() < 3 {
                return Err(MeshError::Malformed(format!(
                    "OBJ face at line {} has fewer than three vertices",
                    line_number + 1
                )));
            }
            let triangle_count = refs.len().saturating_sub(2) as u64;
            let entry = groups
                .entry((current_object.clone(), current_group.clone()))
                .or_default();
            for id in refs {
                if id < 0 || id as usize >= vertices.len() {
                    return Err(MeshError::Malformed(format!(
                        "OBJ face at line {} references a missing vertex",
                        line_number + 1
                    )));
                }
                entry.0.insert(id as usize);
            }
            entry.1 += triangle_count;
        } else if line.starts_with("l ") || line.starts_with("p ") {
            diagnostics.push(MeshDiagnostic{code:MeshDiagnosticCode::UnsupportedTopology,object_id:Some(format!("obj:{}:{}", safe_id(&current_object), safe_id(&current_group))),message:"OBJ line or point topology is retained as a diagnostic and not indexed as triangles.".into()});
        } else if line.starts_with("mtllib ") {
            diagnostics.push(MeshDiagnostic{code:MeshDiagnosticCode::ExternalResource,object_id:None,message:"External OBJ material libraries are not loaded by the offline managed-source parser.".into()});
        }
    }
    let mut objects = BTreeMap::new();
    for ((object_name, group_name), (ids, triangles)) in groups {
        let bounds = bounds_from_points(ids.iter().map(|id| vertices[*id]))?;
        let id = format!("obj:{}:{}", safe_id(&object_name), safe_id(&group_name));
        objects.insert(
            id.clone(),
            MeshObject {
                id,
                name: Some(object_name),
                group: Some(group_name),
                frame: identity(),
                source_matrix: identity_matrix(),
                bounds,
                vertex_count: ids.len() as u64,
                triangle_count: triangles,
            },
        );
    }
    enforce_counts(&objects, p)?;
    Ok((objects, diagnostics))
}
fn parse_stl<F: FnMut() -> bool>(
    bytes: &[u8],
    p: &MeshParsePolicy,
    started: Instant,
    cancelled: &mut F,
) -> Result<(BTreeMap<String, MeshObject>, Vec<MeshDiagnostic>), MeshError> {
    let mut points = Vec::new();
    let triangles;
    if bytes.len() >= 84 {
        let count = u32::from_le_bytes(bytes[80..84].try_into().unwrap()) as usize;
        if 84 + count.saturating_mul(50) == bytes.len() {
            triangles = count as u64;
            for chunk in bytes[84..].chunks_exact(50) {
                check(started, p, cancelled)?;
                for offset in [12, 24, 36] {
                    points.push([
                        f32::from_le_bytes(chunk[offset..offset + 4].try_into().unwrap()) as f64,
                        f32::from_le_bytes(chunk[offset + 4..offset + 8].try_into().unwrap())
                            as f64,
                        f32::from_le_bytes(chunk[offset + 8..offset + 12].try_into().unwrap())
                            as f64,
                    ]);
                }
            }
        } else {
            let (text_points, count) = parse_ascii_stl(bytes, p, started, cancelled)?;
            points = text_points;
            triangles = count
        }
    } else {
        let (text_points, count) = parse_ascii_stl(bytes, p, started, cancelled)?;
        points = text_points;
        triangles = count
    }
    if triangles > p.max_triangles || points.len() as u64 > p.max_vertices {
        return Err(MeshError::Limit("STL geometry limit exceeded".into()));
    }
    let bounds = bounds_from_points(points.into_iter())?;
    let object = MeshObject {
        id: "stl:solid".into(),
        name: Some("solid".into()),
        group: None,
        frame: identity(),
        source_matrix: identity_matrix(),
        bounds,
        vertex_count: triangles * 3,
        triangle_count: triangles,
    };
    Ok((BTreeMap::from([(object.id.clone(), object)]), Vec::new()))
}
fn parse_ascii_stl<F: FnMut() -> bool>(
    bytes: &[u8],
    p: &MeshParsePolicy,
    started: Instant,
    cancelled: &mut F,
) -> Result<(Vec<[f64; 3]>, u64), MeshError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| MeshError::Malformed("STL is neither valid binary nor ASCII".into()))?;
    if !text.trim_start().starts_with("solid ") || !text.contains("endsolid") {
        return Err(MeshError::Malformed("ASCII STL envelope is invalid".into()));
    }
    let mut points = Vec::new();
    for line in text.lines() {
        check(started, p, cancelled)?;
        if let Some(rest) = line.trim().strip_prefix("vertex ") {
            let values = rest
                .split_whitespace()
                .map(str::parse::<f64>)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| MeshError::Malformed("ASCII STL vertex is invalid".into()))?;
            if values.len() != 3 {
                return Err(MeshError::Malformed(
                    "ASCII STL vertex has wrong dimension".into(),
                ));
            }
            points.push([values[0], values[1], values[2]]);
        }
    }
    if points.len() % 3 != 0 || points.is_empty() {
        return Err(MeshError::Malformed(
            "ASCII STL has incomplete triangles".into(),
        ));
    }
    Ok((points.clone(), (points.len() / 3) as u64))
}
fn parse_gltf_json<F: FnMut() -> bool>(
    bytes: &[u8],
    p: &MeshParsePolicy,
    started: Instant,
    cancelled: &mut F,
) -> Result<(BTreeMap<String, MeshObject>, Vec<MeshDiagnostic>), MeshError> {
    check(started, p, cancelled)?;
    let root: serde_json::Value = serde_json::from_slice(bytes)?;
    if root
        .pointer("/asset/version")
        .and_then(|v| v.as_str())
        .is_none()
    {
        return Err(MeshError::Malformed("glTF has no asset version".into()));
    }
    let mut diagnostics = Vec::new();
    if root
        .get("buffers")
        .and_then(|v| v.as_array())
        .is_some_and(|buffers| {
            buffers.iter().any(|buffer| {
                buffer
                    .get("uri")
                    .and_then(|v| v.as_str())
                    .is_some_and(|uri| !uri.starts_with("data:"))
            })
        })
    {
        diagnostics.push(MeshDiagnostic{code:MeshDiagnosticCode::ExternalResource,object_id:None,message:"External glTF buffers are not loaded; import a self-contained GLB or embedded glTF.".into()});
    }
    let accessors = root
        .get("accessors")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let meshes = root
        .get("meshes")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if meshes.len() > p.max_objects {
        return Err(MeshError::Limit("glTF object limit exceeded".into()));
    }
    let mut objects = BTreeMap::new();
    for (index, mesh) in meshes.iter().enumerate() {
        check(started, p, cancelled)?;
        let name = mesh.get("name").and_then(|v| v.as_str()).map(str::to_owned);
        let mut min = [f64::INFINITY; 3];
        let mut max = [f64::NEG_INFINITY; 3];
        let mut vertices = 0;
        let mut triangles = 0;
        for primitive in mesh
            .get("primitives")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
        {
            let mode = primitive.get("mode").and_then(|v| v.as_u64()).unwrap_or(4);
            if mode != 4 {
                diagnostics.push(MeshDiagnostic {
                    code: MeshDiagnosticCode::UnsupportedTopology,
                    object_id: Some(format!("gltf:mesh:{index}")),
                    message: format!("glTF primitive mode {mode} is not triangle topology"),
                });
                continue;
            }
            if let Some(position) = primitive
                .pointer("/attributes/POSITION")
                .and_then(|v| v.as_u64())
                .and_then(|id| accessors.get(id as usize))
            {
                vertices += position.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
                if let (Some(lo), Some(hi)) =
                    (array3(position.get("min")), array3(position.get("max")))
                {
                    for axis in 0..3 {
                        min[axis] = min[axis].min(lo[axis]);
                        max[axis] = max[axis].max(hi[axis]);
                    }
                }
            }
            triangles += primitive
                .get("indices")
                .and_then(|v| v.as_u64())
                .and_then(|id| accessors.get(id as usize))
                .and_then(|a| a.get("count"))
                .and_then(|v| v.as_u64())
                .map(|v| v / 3)
                .unwrap_or(vertices / 3);
        }
        if min.iter().any(|v| !v.is_finite()) {
            return Err(MeshError::Invalid(format!(
                "glTF mesh {index} has no finite POSITION bounds"
            )));
        }
        let id = format!("gltf:mesh:{index}");
        objects.insert(
            id.clone(),
            MeshObject {
                id,
                name,
                group: None,
                frame: identity(),
                source_matrix: identity_matrix(),
                bounds: MeshBounds {
                    minimum: min,
                    maximum: max,
                },
                vertex_count: vertices,
                triangle_count: triangles,
            },
        );
    }
    let nodes = root
        .get("nodes")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    if nodes.iter().any(|node| node.get("mesh").is_some()) {
        let mesh_objects = objects;
        objects = BTreeMap::new();
        for (node_index, node) in nodes.iter().enumerate() {
            check(started, p, cancelled)?;
            let Some(mesh_index) = node.get("mesh").and_then(|value| value.as_u64()) else {
                continue;
            };
            let mesh_id = format!("gltf:mesh:{mesh_index}");
            let mesh = mesh_objects.get(&mesh_id).ok_or_else(|| {
                MeshError::Malformed(format!(
                    "glTF node {node_index} refers to missing mesh {mesh_index}"
                ))
            })?;
            let source_matrix = gltf_node_matrix(node)?;
            let bounds = transformed_bounds(&mesh.bounds, &source_matrix)?;
            let translation = [source_matrix[12], source_matrix[13], source_matrix[14]];
            let scale = [
                vector_length([source_matrix[0], source_matrix[1], source_matrix[2]]),
                vector_length([source_matrix[4], source_matrix[5], source_matrix[6]]),
                vector_length([source_matrix[8], source_matrix[9], source_matrix[10]]),
            ];
            let id = format!("gltf:node:{node_index}");
            objects.insert(
                id.clone(),
                MeshObject {
                    id,
                    name: node
                        .get("name")
                        .and_then(|value| value.as_str())
                        .map(str::to_owned)
                        .or_else(|| mesh.name.clone()),
                    group: Some(mesh_id),
                    frame: ShelfTransform {
                        translation,
                        rotation_degrees: [0.0; 3],
                        scale,
                    },
                    source_matrix,
                    bounds,
                    vertex_count: mesh.vertex_count,
                    triangle_count: mesh.triangle_count,
                },
            );
        }
    }
    enforce_counts(&objects, p)?;
    Ok((objects, diagnostics))
}
fn glb_json(bytes: &[u8]) -> Result<&[u8], MeshError> {
    if bytes.len() < 20
        || &bytes[..4] != b"glTF"
        || u32::from_le_bytes(bytes[4..8].try_into().unwrap()) != 2
    {
        return Err(MeshError::Malformed("GLB header is invalid".into()));
    }
    let total = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
    if total != bytes.len() {
        return Err(MeshError::Malformed("GLB length is invalid".into()));
    }
    let length = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    let kind = u32::from_le_bytes(bytes[16..20].try_into().unwrap());
    if kind != 0x4e4f534a || 20 + length > bytes.len() {
        return Err(MeshError::Malformed("GLB JSON chunk is invalid".into()));
    }
    Ok(&bytes[20..20 + length])
}

pub fn prepare_mesh_saved_view(
    project: &std::path::Path,
    design_id: &DesignId,
    index: &MeshDocumentIndex,
    request: MeshSavedViewRequest,
) -> Result<PreparedMeshSavedView, MeshError> {
    let package = load_project_package(project).map_err(|e| MeshError::Package(e.to_string()))?;
    if !package.manifest.designs.iter().any(|d| &d.id == design_id) {
        return Err(MeshError::Invalid(
            "saved view design does not belong to project".into(),
        ));
    }
    let source = inspect_source(project, &request.source_id)?;
    if source.sha256 != index.source_sha256 || source.id != index.source_id {
        return Err(MeshError::Invalid(
            "mesh source changed after indexing".into(),
        ));
    }
    if request.object_ids.is_empty()
        || request
            .object_ids
            .iter()
            .any(|id| !index.objects.contains_key(id))
    {
        return Err(MeshError::Invalid(
            "saved view requires exact indexed object ids".into(),
        ));
    }
    let units_to_metres = if index.units.as_deref() == Some("m") {
        1.0
    } else {
        let calibration = request.calibration.as_ref().ok_or_else(|| {
            MeshError::Invalid("unitless mesh requires explicit confirmed calibration".into())
        })?;
        if !calibration.confirmed
            || calibration.confirmed_by.trim().is_empty()
            || !calibration.units_to_metres.is_finite()
            || calibration.units_to_metres <= 0.0
        {
            return Err(MeshError::Invalid(
                "mesh calibration is not explicitly confirmed".into(),
            ));
        }
        calibration.units_to_metres
    };
    validate_view(&request)?;
    let item = ShelfItem {
        id: request.shelf_item_id,
        label: request.label,
        annotations: Vec::new(),
        confirmation: ShelfConfirmation {
            confirmed: true,
            confirmed_by: Some(request.created_by.clone()),
            confirmed_at: Some(request.created_at.clone()),
        },
        provenance: ShelfProvenance {
            created_at: request.created_at,
            created_by: request.created_by,
            method: "neutral_mesh_saved_view".into(),
            derivative_id: None,
        },
        drawing_context: Some(DrawingContext {
            view_role: DrawingViewRole::Reference,
            orientation: request.orientation.clone(),
            calibration: None,
        }),
        content: ShelfItemContent::Saved3dView {
            source: Some(ShelfSourceRef {
                source_id: source.id,
                source_sha256: source.sha256,
            }),
            camera: request.camera,
            object_ids: request.object_ids,
            transform: request.transform,
            orientation: request.orientation,
            scale: request.scale,
            section_planes: request.section_planes.clone(),
            unit_calibration: request.calibration.clone(),
        },
    };
    Ok(PreparedMeshSavedView {
        shelf_item: item,
        section_planes: request.section_planes,
        source_units: index.units.clone(),
        units_to_metres,
    })
}
fn validate_view(r: &MeshSavedViewRequest) -> Result<(), MeshError> {
    if !r.scale.is_finite() || r.scale <= 0.0 {
        return Err(MeshError::Invalid("saved view scale is invalid".into()));
    }
    let finite = r
        .camera
        .position
        .into_iter()
        .chain(r.camera.target)
        .chain(r.camera.up)
        .chain(r.transform.translation)
        .chain(r.transform.rotation_degrees)
        .chain(r.transform.scale)
        .all(f64::is_finite)
        && r.section_planes.iter().all(|plane| {
            plane
                .normal
                .into_iter()
                .chain([plane.constant])
                .all(f64::is_finite)
                && plane.normal.iter().any(|v| v.abs() > 1e-12)
        });
    if !finite {
        return Err(MeshError::Invalid(
            "saved view contains invalid geometry".into(),
        ));
    }
    Ok(())
}
fn validate_index(i: &MeshDocumentIndex, s: &SourceId, h: &str) -> Result<(), MeshError> {
    if i.schema_version != MESH_INDEX_SCHEMA_VERSION
        || &i.source_id != s
        || i.source_sha256 != h
        || i.parser != MESH_PARSER_ID
    {
        return Err(MeshError::Invalid("mesh index identity is invalid".into()));
    }
    Ok(())
}
fn identity() -> ShelfTransform {
    ShelfTransform {
        translation: [0.0; 3],
        rotation_degrees: [0.0; 3],
        scale: [1.0; 3],
    }
}
fn identity_matrix() -> [f64; 16] {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}
fn gltf_node_matrix(node: &serde_json::Value) -> Result<[f64; 16], MeshError> {
    if let Some(values) = node.get("matrix").and_then(|value| value.as_array()) {
        if values.len() != 16 {
            return Err(MeshError::Malformed(
                "glTF node matrix must contain 16 values".into(),
            ));
        }
        let mut matrix = [0.0; 16];
        for (index, value) in values.iter().enumerate() {
            matrix[index] = value.as_f64().ok_or_else(|| {
                MeshError::Malformed("glTF node matrix contains a non-number".into())
            })?;
        }
        return Ok(matrix);
    }
    let translation = array3(node.get("translation")).unwrap_or([0.0; 3]);
    let scale = array3(node.get("scale")).unwrap_or([1.0; 3]);
    let rotation = node
        .get("rotation")
        .and_then(|value| value.as_array())
        .map(|values| {
            if values.len() != 4 {
                return Err(MeshError::Malformed(
                    "glTF node rotation must contain four values".into(),
                ));
            }
            Ok([
                values[0]
                    .as_f64()
                    .ok_or_else(|| MeshError::Malformed("invalid glTF quaternion".into()))?,
                values[1]
                    .as_f64()
                    .ok_or_else(|| MeshError::Malformed("invalid glTF quaternion".into()))?,
                values[2]
                    .as_f64()
                    .ok_or_else(|| MeshError::Malformed("invalid glTF quaternion".into()))?,
                values[3]
                    .as_f64()
                    .ok_or_else(|| MeshError::Malformed("invalid glTF quaternion".into()))?,
            ])
        })
        .transpose()?
        .unwrap_or([0.0, 0.0, 0.0, 1.0]);
    let [x, y, z, w] = rotation;
    Ok([
        (1.0 - 2.0 * (y * y + z * z)) * scale[0],
        (2.0 * (x * y + z * w)) * scale[0],
        (2.0 * (x * z - y * w)) * scale[0],
        0.0,
        (2.0 * (x * y - z * w)) * scale[1],
        (1.0 - 2.0 * (x * x + z * z)) * scale[1],
        (2.0 * (y * z + x * w)) * scale[1],
        0.0,
        (2.0 * (x * z + y * w)) * scale[2],
        (2.0 * (y * z - x * w)) * scale[2],
        (1.0 - 2.0 * (x * x + y * y)) * scale[2],
        0.0,
        translation[0],
        translation[1],
        translation[2],
        1.0,
    ])
}
fn transformed_bounds(bounds: &MeshBounds, matrix: &[f64; 16]) -> Result<MeshBounds, MeshError> {
    let mut points = Vec::with_capacity(8);
    for x in [bounds.minimum[0], bounds.maximum[0]] {
        for y in [bounds.minimum[1], bounds.maximum[1]] {
            for z in [bounds.minimum[2], bounds.maximum[2]] {
                points.push([
                    matrix[0] * x + matrix[4] * y + matrix[8] * z + matrix[12],
                    matrix[1] * x + matrix[5] * y + matrix[9] * z + matrix[13],
                    matrix[2] * x + matrix[6] * y + matrix[10] * z + matrix[14],
                ]);
            }
        }
    }
    bounds_from_points(points.into_iter())
}
fn vector_length(value: [f64; 3]) -> f64 {
    value
        .into_iter()
        .map(|component| component * component)
        .sum::<f64>()
        .sqrt()
}
fn nonempty_name(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        "unnamed".into()
    } else {
        value.into()
    }
}
fn array3(v: Option<&serde_json::Value>) -> Option<[f64; 3]> {
    let a = v?.as_array()?;
    Some([
        a.first()?.as_f64()?,
        a.get(1)?.as_f64()?,
        a.get(2)?.as_f64()?,
    ])
}
fn bounds_from_points<I: Iterator<Item = [f64; 3]>>(points: I) -> Result<MeshBounds, MeshError> {
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for point in points {
        if point.iter().any(|v| !v.is_finite()) {
            return Err(MeshError::Malformed(
                "mesh contains non-finite coordinates".into(),
            ));
        }
        for i in 0..3 {
            min[i] = min[i].min(point[i]);
            max[i] = max[i].max(point[i]);
        }
    }
    if min.iter().any(|v| !v.is_finite()) {
        return Err(MeshError::Invalid("mesh object has no vertices".into()));
    }
    Ok(MeshBounds {
        minimum: min,
        maximum: max,
    })
}
fn combined_bounds<'a, I: Iterator<Item = &'a MeshBounds>>(bounds: I) -> Option<MeshBounds> {
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for b in bounds {
        for i in 0..3 {
            min[i] = min[i].min(b.minimum[i]);
            max[i] = max[i].max(b.maximum[i]);
        }
    }
    min.iter().all(|v| v.is_finite()).then_some(MeshBounds {
        minimum: min,
        maximum: max,
    })
}
fn enforce_counts(
    objects: &BTreeMap<String, MeshObject>,
    p: &MeshParsePolicy,
) -> Result<(), MeshError> {
    let vertices = objects.values().map(|o| o.vertex_count).sum::<u64>();
    let triangles = objects.values().map(|o| o.triangle_count).sum::<u64>();
    if objects.len() > p.max_objects || vertices > p.max_vertices || triangles > p.max_triangles {
        return Err(MeshError::Limit(
            "mesh object, vertex, or triangle limit exceeded".into(),
        ));
    }
    Ok(())
}
fn safe_id(v: &str) -> String {
    v.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '_' | '-') {
                c
            } else {
                '_'
            }
        })
        .collect()
}
fn check<F: FnMut() -> bool>(
    started: Instant,
    p: &MeshParsePolicy,
    cancelled: &mut F,
) -> Result<(), MeshError> {
    if cancelled() {
        Err(MeshError::Cancelled)
    } else if started.elapsed() > Duration::from_millis(p.max_millis) {
        Err(MeshError::TimeLimit)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SourceImportRequest, import_source, load_design_shelf, upsert_shelf_item};
    use std::fs;
    fn identity_source() -> (SourceId, String) {
        let h = "a".repeat(64);
        (SourceId::from_sha256(&h).unwrap(), h)
    }
    #[test]
    fn obj_groups_bounds_units_and_unsupported_are_truthful() {
        let (s, h) = identity_source();
        let bytes = b"o Frame\nv 0 0 0\nv 1 0 0\nv 0 2 0\nf 1 2 3\nl 1 2\n";
        let i = parse_mesh(
            bytes,
            SourceMediaType::Obj,
            &s,
            &h,
            &MeshParsePolicy::default(),
            || false,
        )
        .unwrap();
        assert_eq!(
            i.objects["obj:Frame:default"].bounds.maximum,
            [1.0, 2.0, 0.0]
        );
        assert!(i.units.is_none());
        assert!(
            i.diagnostics
                .iter()
                .any(|d| d.code == MeshDiagnosticCode::UnitsMissing)
        );
        assert!(
            i.diagnostics
                .iter()
                .any(|d| d.code == MeshDiagnosticCode::UnsupportedTopology)
        );
    }
    #[test]
    fn gltf_and_glb_preserve_mesh_ids_bounds_and_metre_frame() {
        let (s, h) = identity_source();
        let json=br#"{"asset":{"version":"2.0"},"accessors":[{"count":3,"min":[0,0,0],"max":[1,2,3]},{"count":3}],"meshes":[{"name":"Reference","primitives":[{"attributes":{"POSITION":0},"indices":1}]}]}"#;
        let i = parse_mesh(
            json,
            SourceMediaType::Gltf,
            &s,
            &h,
            &MeshParsePolicy::default(),
            || false,
        )
        .unwrap();
        assert_eq!(i.units.as_deref(), Some("m"));
        assert_eq!(i.objects["gltf:mesh:0"].triangle_count, 1);
        let mut glb = b"glTF".to_vec();
        glb.extend(2u32.to_le_bytes());
        glb.extend(((20 + json.len()) as u32).to_le_bytes());
        glb.extend((json.len() as u32).to_le_bytes());
        glb.extend(0x4e4f534au32.to_le_bytes());
        glb.extend(json);
        assert_eq!(
            parse_mesh(
                &glb,
                SourceMediaType::Glb,
                &s,
                &h,
                &MeshParsePolicy::default(),
                || false
            )
            .unwrap()
            .format,
            MeshFormat::Glb
        );
    }

    #[test]
    fn gltf_nodes_preserve_exact_source_transform_and_world_bounds() {
        let (s, h) = identity_source();
        let json = br#"{"asset":{"version":"2.0"},"accessors":[{"count":3,"min":[0,0,0],"max":[1,2,3]}],"meshes":[{"primitives":[{"attributes":{"POSITION":0}}]}],"nodes":[{"name":"Placed reference","mesh":0,"translation":[10,20,30],"scale":[2,2,2]}]}"#;
        let index = parse_mesh(
            json,
            SourceMediaType::Gltf,
            &s,
            &h,
            &MeshParsePolicy::default(),
            || false,
        )
        .unwrap();
        let object = &index.objects["gltf:node:0"];
        assert_eq!(object.group.as_deref(), Some("gltf:mesh:0"));
        assert_eq!(object.source_matrix[12..15], [10.0, 20.0, 30.0]);
        assert_eq!(object.bounds.maximum, [12.0, 24.0, 36.0]);
    }
    #[test]
    fn stl_malformed_limits_and_cancellation_fail_closed() {
        let (s, h) = identity_source();
        let ascii=b"solid x\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\nendloop\nendfacet\nendsolid x\n";
        assert_eq!(
            parse_mesh(
                ascii,
                SourceMediaType::Stl,
                &s,
                &h,
                &MeshParsePolicy::default(),
                || false
            )
            .unwrap()
            .triangle_count,
            1
        );
        assert!(matches!(
            parse_mesh(
                ascii,
                SourceMediaType::Stl,
                &s,
                &h,
                &MeshParsePolicy {
                    max_triangles: 0,
                    ..MeshParsePolicy::default()
                },
                || false
            ),
            Err(MeshError::Limit(_))
        ));
        assert!(matches!(
            parse_mesh(
                ascii,
                SourceMediaType::Stl,
                &s,
                &h,
                &MeshParsePolicy::default(),
                || true
            ),
            Err(MeshError::Cancelled)
        ));
    }

    #[test]
    fn managed_obj_index_resumes_and_calibrated_saved_view_survives_reopen() {
        let temporary = std::env::temp_dir().join(format!(
            "fraia-mesh-saved-view-{}",
            crate::utils::timestamp_id()
        ));
        fs::create_dir(&temporary).unwrap();
        let project = temporary.join("project");
        let package = crate::create_named_project_package(&project, "Mesh references").unwrap();
        let design_id = package.designs[0].manifest.id.clone();
        let source_path = temporary.join("reference.obj");
        fs::write(
            &source_path,
            b"o Frame\ng Primary\nv 0 0 0\nv 1000 0 0\nv 0 1000 0\nf 1 2 3\n",
        )
        .unwrap();
        let imported = import_source(
            &project,
            SourceImportRequest {
                selected_path: source_path,
                display_alias: Some("Reference mesh.obj".into()),
                expected_media_type: Some(SourceMediaType::Obj),
            },
        )
        .unwrap();
        assert!(matches!(
            index_and_store_mesh_with_cancel(
                &project,
                &imported.record.id,
                &SourceLibraryPolicy::default(),
                &MeshParsePolicy::default(),
                || true,
            ),
            Err(MeshError::Cancelled)
        ));
        assert!(
            crate::source_derivatives(&project, &imported.record.id)
                .unwrap()
                .is_empty()
        );
        let first = index_and_store_mesh(
            &project,
            &imported.record.id,
            &SourceLibraryPolicy::default(),
            &MeshParsePolicy::default(),
        )
        .unwrap();
        assert!(!first.resumed);
        assert!(
            index_and_store_mesh(
                &project,
                &imported.record.id,
                &SourceLibraryPolicy::default(),
                &MeshParsePolicy::default(),
            )
            .unwrap()
            .resumed
        );

        let uncalibrated = MeshSavedViewRequest {
            shelf_item_id: "saved-reference-view".into(),
            label: "Reference view".into(),
            source_id: imported.record.id.clone(),
            object_ids: vec!["obj:Frame:Primary".into()],
            camera: ShelfCamera {
                position: [2.0, 3.0, 4.0],
                target: [0.0, 0.0, 0.0],
                up: [0.0, 1.0, 0.0],
                projection: "perspective".into(),
            },
            transform: identity(),
            orientation: ShelfOrientation {
                forward: [0.0, 0.0, -1.0],
                up: [0.0, 1.0, 0.0],
            },
            scale: 1.0,
            section_planes: vec![ShelfSectionPlane {
                id: "section-a".into(),
                normal: [1.0, 0.0, 0.0],
                constant: -500.0,
            }],
            calibration: None,
            created_at: "2026-08-14T00:00:00Z".into(),
            created_by: "engineer".into(),
        };
        assert!(matches!(
            prepare_mesh_saved_view(&project, &design_id, &first.index, uncalibrated.clone()),
            Err(MeshError::Invalid(_))
        ));
        let prepared = prepare_mesh_saved_view(
            &project,
            &design_id,
            &first.index,
            MeshSavedViewRequest {
                calibration: Some(MeshCalibration {
                    confirmed: true,
                    confirmed_by: "engineer".into(),
                    confirmed_at: "2026-08-14T00:00:00Z".into(),
                    units: "mm".into(),
                    units_to_metres: 0.001,
                }),
                ..uncalibrated
            },
        )
        .unwrap();
        upsert_shelf_item(&project, &design_id, prepared.shelf_item.clone()).unwrap();
        let moved_project = temporary.join("moved-project");
        fs::rename(&project, &moved_project).unwrap();
        let reopened = load_design_shelf(&moved_project, &design_id).unwrap();
        assert!(
            index_and_store_mesh(
                &moved_project,
                &imported.record.id,
                &SourceLibraryPolicy::default(),
                &MeshParsePolicy::default(),
            )
            .unwrap()
            .resumed
        );
        let item = &reopened.items["saved-reference-view"];
        match &item.content {
            ShelfItemContent::Saved3dView {
                source,
                section_planes,
                unit_calibration,
                ..
            } => {
                assert_eq!(
                    source.as_ref().unwrap().source_sha256,
                    imported.record.sha256
                );
                assert_eq!(section_planes[0].id, "section-a");
                assert_eq!(section_planes[0].constant, -500.0);
                assert_eq!(unit_calibration.as_ref().unwrap().units_to_metres, 0.001);
            }
            _ => panic!("expected saved 3D view"),
        }
        fs::remove_dir_all(&temporary).unwrap();
    }
}
