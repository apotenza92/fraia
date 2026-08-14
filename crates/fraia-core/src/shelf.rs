use crate::{
    DesignId, ProjectId, SourceId, SourceLibraryError, SourceReference, design_package_paths,
    inspect_source, load_project_package,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub const SHELF_SCHEMA_VERSION: &str = "fraia.shelf.v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShelfDocument {
    pub schema_version: String,
    pub design_id: DesignId,
    #[serde(default)]
    pub items: BTreeMap<String, ShelfItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShelfItem {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub annotations: Vec<ShelfAnnotation>,
    pub confirmation: ShelfConfirmation,
    pub provenance: ShelfProvenance,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drawing_context: Option<DrawingContext>,
    #[serde(flatten)]
    pub content: ShelfItemContent,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DrawingContext {
    pub view_role: DrawingViewRole,
    pub orientation: ShelfOrientation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub calibration: Option<DrawingCalibration>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DrawingViewRole {
    Plan,
    Elevation,
    Section,
    Detail,
    Schedule,
    Reference,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DrawingCalibration {
    pub first_point: [f64; 2],
    pub second_point: [f64; 2],
    pub known_distance: f64,
    pub unit: String,
    pub source_units_per_known_unit: f64,
    pub confirmed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ShelfItemContent {
    PdfPage {
        source: ShelfSourceRef,
        page_number: u32,
        layout: ShelfLayout,
    },
    PdfCrop {
        source: ShelfSourceRef,
        page_number: u32,
        crop: ShelfRect,
        layout: ShelfLayout,
    },
    ImageCrop {
        source: ShelfSourceRef,
        crop: ShelfRect,
        image_width: u32,
        image_height: u32,
    },
    CadSelection {
        source: ShelfSourceRef,
        layout: String,
        object_ids: Vec<String>,
        transform: ShelfTransform,
        orientation: ShelfOrientation,
        scale: f64,
    },
    IfcSelection {
        source: ShelfSourceRef,
        object_ids: Vec<String>,
        transform: ShelfTransform,
        orientation: ShelfOrientation,
        scale: f64,
    },
    Saved3dView {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        source: Option<ShelfSourceRef>,
        camera: ShelfCamera,
        object_ids: Vec<String>,
        transform: ShelfTransform,
        orientation: ShelfOrientation,
        scale: f64,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        section_planes: Vec<ShelfSectionPlane>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        unit_calibration: Option<ShelfUnitCalibration>,
    },
    AcceptedDesignRevision {
        target: AcceptedDesignRevisionRef,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShelfSourceRef {
    pub source_id: SourceId,
    pub source_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShelfSectionPlane {
    pub id: String,
    pub normal: [f64; 3],
    pub constant: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShelfUnitCalibration {
    pub confirmed: bool,
    pub confirmed_by: String,
    pub confirmed_at: String,
    pub units: String,
    pub units_to_metres: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShelfRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub coordinate_space: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShelfLayout {
    pub media_box: ShelfRect,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crop_box: Option<ShelfRect>,
    pub rotation_degrees: i16,
    pub user_unit: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShelfTransform {
    pub translation: [f64; 3],
    pub rotation_degrees: [f64; 3],
    pub scale: [f64; 3],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShelfOrientation {
    pub forward: [f64; 3],
    pub up: [f64; 3],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShelfCamera {
    pub position: [f64; 3],
    pub target: [f64; 3],
    pub up: [f64; 3],
    pub projection: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShelfAnnotation {
    pub id: String,
    pub annotation_kind: String,
    #[serde(default)]
    pub points: Vec<[f64; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShelfConfirmation {
    pub confirmed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirmed_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirmed_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShelfProvenance {
    pub created_at: String,
    pub created_by: String,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub derivative_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptedDesignRevisionRef {
    pub project_id: ProjectId,
    pub design_id: DesignId,
    pub revision_id: String,
    pub snapshot_id: String,
    pub read_only: bool,
}

#[derive(Debug)]
pub enum ShelfError {
    Package(String),
    Io(String),
    Json(String),
    Invalid(String),
    Source(SourceLibraryError),
    ItemNotFound(String),
    ItemReferenced {
        item_id: String,
        interpretation_revision_ids: Vec<String>,
    },
    RetargetConflict,
}

impl std::fmt::Display for ShelfError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Package(message)
            | Self::Io(message)
            | Self::Json(message)
            | Self::Invalid(message) => formatter.write_str(message),
            Self::Source(error) => write!(formatter, "{error}"),
            Self::ItemNotFound(id) => write!(formatter, "shelf item `{id}` was not found"),
            Self::ItemReferenced {
                item_id,
                interpretation_revision_ids,
            } => write!(
                formatter,
                "shelf item `{item_id}` is referenced by drawing interpretation revisions: {}",
                interpretation_revision_ids.join(", ")
            ),
            Self::RetargetConflict => {
                formatter.write_str("the cross-design shelf target changed before retargeting")
            }
        }
    }
}

impl std::error::Error for ShelfError {}
impl From<std::io::Error> for ShelfError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}
impl From<serde_json::Error> for ShelfError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value.to_string())
    }
}
impl From<SourceLibraryError> for ShelfError {
    fn from(value: SourceLibraryError) -> Self {
        Self::Source(value)
    }
}

pub fn load_design_shelf(
    project_dir: &Path,
    design_id: &DesignId,
) -> Result<ShelfDocument, ShelfError> {
    let package = load_project_package(project_dir)
        .map_err(|error| ShelfError::Package(error.to_string()))?;
    if !package
        .designs
        .iter()
        .any(|design| &design.manifest.id == design_id)
    {
        return Err(ShelfError::Invalid(format!(
            "design `{design_id}` is not in this project"
        )));
    }
    let path = design_package_paths(project_dir, design_id)
        .map_err(|error| ShelfError::Invalid(error.to_string()))?
        .shelf_file;
    recover_atomic_shelf(&path)?;
    let bytes = fs::read(&path)?;
    let mut shelf: ShelfDocument = serde_json::from_slice(&bytes).or_else(|_| {
        let placeholder: serde_json::Value = serde_json::from_slice(&bytes)?;
        if placeholder
            .get("schema_version")
            .and_then(|value| value.as_str())
            == Some(SHELF_SCHEMA_VERSION)
            && placeholder
                .get("items")
                .is_some_and(|items| items.as_array().is_some_and(Vec::is_empty))
        {
            Ok::<ShelfDocument, serde_json::Error>(ShelfDocument {
                schema_version: SHELF_SCHEMA_VERSION.into(),
                design_id: design_id.clone(),
                items: BTreeMap::new(),
            })
        } else {
            serde_json::from_value(placeholder)
        }
    })?;
    if shelf.design_id.as_str().is_empty() {
        shelf.design_id = design_id.clone();
    }
    validate_shelf(project_dir, design_id, &shelf)?;
    Ok(shelf)
}

pub fn save_design_shelf(project_dir: &Path, shelf: &ShelfDocument) -> Result<(), ShelfError> {
    validate_shelf(project_dir, &shelf.design_id, shelf)?;
    let path = design_package_paths(project_dir, &shelf.design_id)
        .map_err(|error| ShelfError::Invalid(error.to_string()))?
        .shelf_file;
    atomic_write_shelf(&path, shelf)
}

pub fn upsert_shelf_item(
    project_dir: &Path,
    design_id: &DesignId,
    item: ShelfItem,
) -> Result<ShelfDocument, ShelfError> {
    let mut shelf = load_design_shelf(project_dir, design_id)?;
    validate_item(project_dir, design_id, &item)?;
    shelf.items.insert(item.id.clone(), item);
    save_design_shelf(project_dir, &shelf)?;
    Ok(shelf)
}

pub fn remove_shelf_item(
    project_dir: &Path,
    design_id: &DesignId,
    item_id: &str,
) -> Result<ShelfDocument, ShelfError> {
    let interpretation_revision_ids =
        crate::drawing_interpretation_shelf_references(project_dir, design_id, item_id)
            .map_err(|error| ShelfError::Invalid(error.to_string()))?;
    if !interpretation_revision_ids.is_empty() {
        return Err(ShelfError::ItemReferenced {
            item_id: item_id.into(),
            interpretation_revision_ids,
        });
    }
    let mut shelf = load_design_shelf(project_dir, design_id)?;
    shelf
        .items
        .remove(item_id)
        .ok_or_else(|| ShelfError::ItemNotFound(item_id.into()))?;
    save_design_shelf(project_dir, &shelf)?;
    Ok(shelf)
}

pub fn retarget_cross_design_shelf_item(
    project_dir: &Path,
    design_id: &DesignId,
    item_id: &str,
    expected: &AcceptedDesignRevisionRef,
    replacement: AcceptedDesignRevisionRef,
) -> Result<ShelfDocument, ShelfError> {
    let mut shelf = load_design_shelf(project_dir, design_id)?;
    let item = shelf
        .items
        .get_mut(item_id)
        .ok_or_else(|| ShelfError::ItemNotFound(item_id.into()))?;
    let ShelfItemContent::AcceptedDesignRevision { target } = &mut item.content else {
        return Err(ShelfError::Invalid(
            "only an accepted-design-revision shelf item can be retargeted".into(),
        ));
    };
    if target != expected {
        return Err(ShelfError::RetargetConflict);
    }
    *target = replacement;
    validate_item(project_dir, design_id, item)?;
    save_design_shelf(project_dir, &shelf)?;
    Ok(shelf)
}

pub fn source_shelf_references(
    project_dir: &Path,
    source_id: &SourceId,
) -> Result<Vec<SourceReference>, ShelfError> {
    let package = load_project_package(project_dir)
        .map_err(|error| ShelfError::Package(error.to_string()))?;
    let mut references = Vec::new();
    for design in package.designs {
        let shelf = load_design_shelf(project_dir, &design.manifest.id)?;
        for item in shelf.items.values() {
            if item_source(&item.content).is_some_and(|source| &source.source_id == source_id) {
                references.push(SourceReference {
                    owner_kind: "design_shelf_item".into(),
                    owner_id: format!("{}:{}", design.manifest.id, item.id),
                    locator: Some(item.label.clone()),
                });
            }
        }
    }
    Ok(references)
}

fn item_source(content: &ShelfItemContent) -> Option<&ShelfSourceRef> {
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

fn validate_shelf(
    project_dir: &Path,
    design_id: &DesignId,
    shelf: &ShelfDocument,
) -> Result<(), ShelfError> {
    if shelf.schema_version != SHELF_SCHEMA_VERSION || &shelf.design_id != design_id {
        return Err(ShelfError::Invalid(
            "shelf schema or owning design does not match".into(),
        ));
    }
    for (id, item) in &shelf.items {
        if id != &item.id {
            return Err(ShelfError::Invalid(
                "shelf item key does not match item id".into(),
            ));
        }
        validate_item(project_dir, design_id, item)?;
    }
    Ok(())
}

fn validate_item(
    project_dir: &Path,
    owner_design_id: &DesignId,
    item: &ShelfItem,
) -> Result<(), ShelfError> {
    validate_token("shelf item id", &item.id)?;
    if item.label.trim().is_empty()
        || item.label.len() > 255
        || item.label.chars().any(char::is_control)
    {
        return Err(ShelfError::Invalid("shelf label is invalid".into()));
    }
    if let Some(source) = item_source(&item.content) {
        let record = inspect_source(project_dir, &source.source_id)?;
        if record.sha256 != source.source_sha256 {
            return Err(ShelfError::Invalid(
                "shelf source hash does not match the immutable original".into(),
            ));
        }
    }
    if let Some(context) = &item.drawing_context {
        finite_values(
            context
                .orientation
                .forward
                .into_iter()
                .chain(context.orientation.up),
        )?;
        if let Some(calibration) = &context.calibration {
            finite_values(
                calibration
                    .first_point
                    .into_iter()
                    .chain(calibration.second_point)
                    .chain([
                        calibration.known_distance,
                        calibration.source_units_per_known_unit,
                    ]),
            )?;
            validate_token("calibration unit", &calibration.unit)?;
            if calibration.known_distance <= 0.0
                || calibration.source_units_per_known_unit <= 0.0
                || calibration.first_point == calibration.second_point
            {
                return Err(ShelfError::Invalid(
                    "drawing calibration requires two distinct points and positive scale".into(),
                ));
            }
        }
    }
    match &item.content {
        ShelfItemContent::PdfPage {
            page_number,
            layout,
            ..
        }
        | ShelfItemContent::PdfCrop {
            page_number,
            layout,
            ..
        } => {
            if *page_number == 0 {
                return Err(ShelfError::Invalid("PDF page numbers are one-based".into()));
            }
            validate_layout(layout)?;
        }
        ShelfItemContent::ImageCrop {
            crop,
            image_width,
            image_height,
            ..
        } => {
            validate_rect(crop)?;
            if *image_width == 0 || *image_height == 0 {
                return Err(ShelfError::Invalid(
                    "image dimensions must be positive".into(),
                ));
            }
        }
        ShelfItemContent::CadSelection {
            object_ids,
            transform,
            orientation,
            scale,
            ..
        }
        | ShelfItemContent::IfcSelection {
            object_ids,
            transform,
            orientation,
            scale,
            ..
        } => validate_spatial(object_ids, transform, orientation, *scale)?,
        ShelfItemContent::Saved3dView {
            camera,
            object_ids,
            transform,
            orientation,
            scale,
            section_planes,
            unit_calibration,
            ..
        } => {
            validate_spatial(object_ids, transform, orientation, *scale)?;
            finite_values(
                camera
                    .position
                    .into_iter()
                    .chain(camera.target)
                    .chain(camera.up),
            )?;
            validate_token("camera projection", &camera.projection)?;
            for plane in section_planes {
                validate_token("section plane id", &plane.id)?;
                finite_values(plane.normal.into_iter().chain([plane.constant]))?;
                if plane.normal.iter().all(|value| value.abs() <= 1e-12) {
                    return Err(ShelfError::Invalid(
                        "section plane normal must be non-zero".into(),
                    ));
                }
            }
            if let Some(calibration) = unit_calibration {
                if !calibration.confirmed
                    || calibration.confirmed_by.trim().is_empty()
                    || calibration.confirmed_at.trim().is_empty()
                    || calibration.units.trim().is_empty()
                    || !calibration.units_to_metres.is_finite()
                    || calibration.units_to_metres <= 0.0
                {
                    return Err(ShelfError::Invalid(
                        "3D reference unit calibration is invalid or unconfirmed".into(),
                    ));
                }
            }
        }
        ShelfItemContent::AcceptedDesignRevision { target } => {
            if !target.read_only {
                return Err(ShelfError::Invalid(
                    "cross-design shelf targets must be read-only".into(),
                ));
            }
            if &target.design_id == owner_design_id {
                return Err(ShelfError::Invalid(
                    "cross-design shelf targets must name another design".into(),
                ));
            }
            validate_token("revision id", &target.revision_id)?;
            validate_token("snapshot id", &target.snapshot_id)?;
            let package = load_project_package(project_dir)
                .map_err(|error| ShelfError::Package(error.to_string()))?;
            if package.manifest.id != target.project_id
                || !package
                    .designs
                    .iter()
                    .any(|design| design.manifest.id == target.design_id)
            {
                return Err(ShelfError::Invalid(
                    "cross-design shelf target is not in the owning project".into(),
                ));
            }
        }
    }
    Ok(())
}

fn validate_layout(layout: &ShelfLayout) -> Result<(), ShelfError> {
    validate_rect(&layout.media_box)?;
    if let Some(crop) = &layout.crop_box {
        validate_rect(crop)?;
    }
    if !matches!(layout.rotation_degrees, 0 | 90 | 180 | 270)
        || !layout.user_unit.is_finite()
        || layout.user_unit <= 0.0
    {
        return Err(ShelfError::Invalid(
            "PDF layout rotation or user unit is invalid".into(),
        ));
    }
    Ok(())
}
fn validate_rect(rect: &ShelfRect) -> Result<(), ShelfError> {
    finite_values([rect.x, rect.y, rect.width, rect.height])?;
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return Err(ShelfError::Invalid(
            "shelf rectangle dimensions must be positive".into(),
        ));
    }
    validate_token("coordinate space", &rect.coordinate_space)
}
fn validate_spatial(
    object_ids: &[String],
    transform: &ShelfTransform,
    orientation: &ShelfOrientation,
    scale: f64,
) -> Result<(), ShelfError> {
    if object_ids.is_empty() {
        return Err(ShelfError::Invalid(
            "a spatial shelf selection must contain object ids".into(),
        ));
    }
    for id in object_ids {
        validate_token("object id", id)?;
    }
    finite_values(
        transform
            .translation
            .into_iter()
            .chain(transform.rotation_degrees)
            .chain(transform.scale)
            .chain(orientation.forward)
            .chain(orientation.up)
            .chain([scale]),
    )?;
    if scale <= 0.0 || transform.scale.into_iter().any(|value| value <= 0.0) {
        return Err(ShelfError::Invalid("shelf scale must be positive".into()));
    }
    Ok(())
}
fn finite_values(values: impl IntoIterator<Item = f64>) -> Result<(), ShelfError> {
    if values.into_iter().any(|value| !value.is_finite()) {
        return Err(ShelfError::Invalid(
            "shelf coordinates must be finite".into(),
        ));
    }
    Ok(())
}
fn validate_token(label: &str, value: &str) -> Result<(), ShelfError> {
    if value.trim().is_empty()
        || value.len() > 255
        || value.chars().any(char::is_control)
        || matches!(value, "." | "..")
        || value.contains(['/', '\\'])
    {
        return Err(ShelfError::Invalid(format!("{label} is invalid")));
    }
    Ok(())
}

fn atomic_write_shelf(path: &Path, shelf: &ShelfDocument) -> Result<(), ShelfError> {
    recover_atomic_shelf(path)?;
    let parent = path
        .parent()
        .ok_or_else(|| ShelfError::Invalid("shelf path has no parent".into()))?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| ShelfError::Invalid("shelf filename is invalid".into()))?;
    let temporary = parent.join(format!(".{name}.tmp-{}", unique_id()));
    let backup = parent.join(format!(".{name}.bak"));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    serde_json::to_writer_pretty(&mut file, shelf)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    if path.exists() {
        fs::rename(path, &backup)?;
    }
    if let Err(error) = fs::rename(&temporary, path) {
        if backup.exists() {
            fs::rename(&backup, path)?;
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

fn recover_atomic_shelf(path: &Path) -> Result<(), ShelfError> {
    let parent = path
        .parent()
        .ok_or_else(|| ShelfError::Invalid("shelf path has no parent".into()))?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| ShelfError::Invalid("shelf filename is invalid".into()))?;
    let backup = parent.join(format!(".{name}.bak"));
    if fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink())
        || fs::symlink_metadata(&backup).is_ok_and(|metadata| metadata.file_type().is_symlink())
    {
        return Err(ShelfError::Invalid(
            "shelf storage must not use symlinks".into(),
        ));
    }
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
fn sync_directory(path: &Path) -> Result<(), ShelfError> {
    File::open(path)?.sync_all()?;
    Ok(())
}
#[cfg(windows)]
fn sync_directory(_: &Path) -> Result<(), ShelfError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        DesignManifest, DesignPackage, ProjectDesignEntry, SourceImportRequest,
        create_named_project_package, import_source, load_project_package, save_project_package,
    };

    struct Fixture {
        root: std::path::PathBuf,
        project: std::path::PathBuf,
        first: DesignId,
        second: DesignId,
        source: crate::SourceRecord,
    }

    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!("fraia-shelf-test-{}", unique_id()));
            fs::create_dir(&root).expect("create fixture root");
            let project = root.join("project");
            create_named_project_package(&project, "Shelf fixture").expect("create package");
            let mut package = load_project_package(&project).expect("load package");
            let first = package.designs[0].manifest.id.clone();
            let second = DesignId::new("design-second");
            let mut manifest: DesignManifest = package.designs[0].manifest.clone();
            manifest.id = second.clone();
            manifest.name = "Second design".into();
            manifest.created_at = "fixture".into();
            manifest.legacy_migration = None;
            let mut state = package.designs[0].project.clone();
            state.name = "Second design".into();
            package.manifest.designs.push(ProjectDesignEntry {
                id: second.clone(),
                name: "Second design".into(),
            });
            package.designs.push(DesignPackage {
                manifest,
                project: state,
                legacy_project: None,
            });
            save_project_package(&project, &package).expect("save second design");

            let input = root.join("plan.pdf");
            fs::write(&input, b"%PDF-1.7\nfixture\n%%EOF\n").expect("write source");
            let source = import_source(
                &project,
                SourceImportRequest {
                    selected_path: input,
                    display_alias: None,
                    expected_media_type: None,
                },
            )
            .expect("import source")
            .record;
            Self {
                root,
                project,
                first,
                second,
                source,
            }
        }

        fn pdf_crop(&self, id: &str) -> ShelfItem {
            ShelfItem {
                id: id.into(),
                label: "Level 1 framing area".into(),
                annotations: vec![ShelfAnnotation {
                    id: "note-1".into(),
                    annotation_kind: "note".into(),
                    points: vec![[10.0, 12.0]],
                    text: Some("Primary frame".into()),
                }],
                confirmation: ShelfConfirmation {
                    confirmed: true,
                    confirmed_by: Some("user".into()),
                    confirmed_at: Some("fixture".into()),
                },
                provenance: ShelfProvenance {
                    created_at: "fixture".into(),
                    created_by: "user".into(),
                    method: "pdf_crop".into(),
                    derivative_id: None,
                },
                drawing_context: None,
                content: ShelfItemContent::PdfCrop {
                    source: ShelfSourceRef {
                        source_id: self.source.id.clone(),
                        source_sha256: self.source.sha256.clone(),
                    },
                    page_number: 3,
                    crop: ShelfRect {
                        x: 10.0,
                        y: 20.0,
                        width: 300.0,
                        height: 200.0,
                        coordinate_space: "pdf_points".into(),
                    },
                    layout: ShelfLayout {
                        media_box: ShelfRect {
                            x: 0.0,
                            y: 0.0,
                            width: 841.89,
                            height: 595.28,
                            coordinate_space: "pdf_points".into(),
                        },
                        crop_box: None,
                        rotation_degrees: 90,
                        user_unit: 1.0,
                    },
                },
            }
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn shelf_round_trips_exact_source_coordinates_layout_annotations_and_move() {
        let fixture = Fixture::new();
        let item = fixture.pdf_crop("crop-1");
        let saved = upsert_shelf_item(&fixture.project, &fixture.first, item.clone())
            .expect("save shelf item");
        assert_eq!(saved.items["crop-1"], item);
        assert_eq!(
            source_shelf_references(&fixture.project, &fixture.source.id)
                .expect("source refs")
                .len(),
            1
        );

        let moved = fixture.root.join("moved");
        fs::rename(&fixture.project, &moved).expect("move project");
        let loaded = load_design_shelf(&moved, &fixture.first).expect("load moved shelf");
        assert_eq!(loaded.items["crop-1"], item);
    }

    #[test]
    fn cross_design_target_is_read_only_and_requires_explicit_compare_and_swap_retarget() {
        let fixture = Fixture::new();
        let target = AcceptedDesignRevisionRef {
            project_id: load_project_package(&fixture.project)
                .expect("package")
                .manifest
                .id,
            design_id: fixture.second.clone(),
            revision_id: "revision-1".into(),
            snapshot_id: "snapshot-1".into(),
            read_only: true,
        };
        let item = ShelfItem {
            id: "design-ref".into(),
            label: "Accepted framing reference".into(),
            annotations: Vec::new(),
            confirmation: ShelfConfirmation {
                confirmed: true,
                confirmed_by: Some("user".into()),
                confirmed_at: Some("fixture".into()),
            },
            provenance: ShelfProvenance {
                created_at: "fixture".into(),
                created_by: "user".into(),
                method: "accepted_revision".into(),
                derivative_id: None,
            },
            drawing_context: None,
            content: ShelfItemContent::AcceptedDesignRevision {
                target: target.clone(),
            },
        };
        upsert_shelf_item(&fixture.project, &fixture.first, item).expect("save cross-design item");
        let replacement = AcceptedDesignRevisionRef {
            revision_id: "revision-2".into(),
            snapshot_id: "snapshot-2".into(),
            ..target.clone()
        };
        let retargeted = retarget_cross_design_shelf_item(
            &fixture.project,
            &fixture.first,
            "design-ref",
            &target,
            replacement.clone(),
        )
        .expect("explicit retarget");
        assert!(
            matches!(&retargeted.items["design-ref"].content, ShelfItemContent::AcceptedDesignRevision { target } if target == &replacement)
        );
        let stale = retarget_cross_design_shelf_item(
            &fixture.project,
            &fixture.first,
            "design-ref",
            &target,
            replacement,
        );
        assert!(matches!(stale, Err(ShelfError::RetargetConflict)));
    }

    #[test]
    fn rejects_wrong_source_hash_and_recovers_atomic_backup() {
        let fixture = Fixture::new();
        let mut item = fixture.pdf_crop("crop-1");
        if let ShelfItemContent::PdfCrop { source, .. } = &mut item.content {
            source.source_sha256 = "0".repeat(64);
        }
        assert!(matches!(
            upsert_shelf_item(&fixture.project, &fixture.first, item),
            Err(ShelfError::Invalid(_))
        ));

        let item = fixture.pdf_crop("crop-2");
        upsert_shelf_item(&fixture.project, &fixture.first, item.clone()).expect("save valid item");
        let path = design_package_paths(&fixture.project, &fixture.first)
            .expect("paths")
            .shelf_file;
        let backup = path.parent().expect("parent").join(".shelf.json.bak");
        fs::rename(&path, &backup).expect("simulate interruption");
        assert_eq!(
            load_design_shelf(&fixture.project, &fixture.first)
                .expect("recover shelf")
                .items["crop-2"],
            item
        );
        assert!(path.is_file());
    }

    #[test]
    fn calibration_and_orientation_round_trip_without_becoming_structural_geometry() {
        let fixture = Fixture::new();
        let model_before = serde_json::to_value(
            &load_project_package(&fixture.project)
                .expect("project before shelf update")
                .designs[0]
                .project
                .structural_model,
        )
        .expect("serialize model before shelf update");
        let mut item = fixture.pdf_crop("calibrated-crop");
        item.drawing_context = Some(DrawingContext {
            view_role: DrawingViewRole::Plan,
            orientation: ShelfOrientation {
                forward: [0.0, 0.0, -1.0],
                up: [0.0, 1.0, 0.0],
            },
            calibration: Some(DrawingCalibration {
                first_point: [50.0, 100.0],
                second_point: [250.0, 100.0],
                known_distance: 10.0,
                unit: "m".into(),
                source_units_per_known_unit: 20.0,
                confirmed: true,
            }),
        });
        let saved = upsert_shelf_item(&fixture.project, &fixture.first, item.clone())
            .expect("save calibrated crop");
        assert_eq!(saved.items["calibrated-crop"], item);
        let model_after = serde_json::to_value(
            &load_project_package(&fixture.project)
                .expect("project remains model-owned")
                .designs[0]
                .project
                .structural_model,
        )
        .expect("serialize model after shelf update");
        assert_eq!(model_after, model_before);
    }
}
