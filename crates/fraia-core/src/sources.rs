use crate::project::{load_project_package, project_package_paths};
use crate::utils::{iso_now, timestamp_id};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

pub const SOURCE_INDEX_SCHEMA_VERSION: &str = "fraia.sources.v1";
pub const SOURCE_LIBRARY_OPERATION_VERSION: &str = "fraia.sources.operation.v1";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SourceId(String);

impl SourceId {
    pub fn from_sha256(hash: &str) -> Result<Self, SourceLibraryError> {
        validate_sha256(hash)?;
        Ok(Self(format!("sha256-{hash}")))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn sha256(&self) -> Result<&str, SourceLibraryError> {
        self.0
            .strip_prefix("sha256-")
            .ok_or_else(|| SourceLibraryError::InvalidSourceId(self.0.clone()))
            .and_then(|hash| {
                validate_sha256(hash)?;
                Ok(hash)
            })
    }
}

impl std::fmt::Display for SourceId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SourceDerivativeId(String);

impl SourceDerivativeId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SourceDerivativeId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceMediaType {
    Pdf,
    Png,
    Jpeg,
    Tiff,
    Dxf,
    IfcStep,
    Step,
    Dwg,
    Gltf,
    Glb,
    Obj,
    Stl,
}

impl SourceMediaType {
    pub fn media_type(self) -> &'static str {
        match self {
            Self::Pdf => "application/pdf",
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Tiff => "image/tiff",
            Self::Dxf => "image/vnd.dxf",
            Self::IfcStep => "application/x-step+ifc",
            Self::Step => "model/step",
            Self::Dwg => "image/vnd.dwg",
            Self::Gltf => "model/gltf+json",
            Self::Glb => "model/gltf-binary",
            Self::Obj => "model/obj",
            Self::Stl => "model/stl",
        }
    }

    fn accepts_extension(self, extension: &str) -> bool {
        match self {
            Self::Pdf => extension.eq_ignore_ascii_case("pdf"),
            Self::Png => extension.eq_ignore_ascii_case("png"),
            Self::Jpeg => {
                extension.eq_ignore_ascii_case("jpg") || extension.eq_ignore_ascii_case("jpeg")
            }
            Self::Tiff => {
                extension.eq_ignore_ascii_case("tif") || extension.eq_ignore_ascii_case("tiff")
            }
            Self::Dxf => extension.eq_ignore_ascii_case("dxf"),
            Self::IfcStep => extension.eq_ignore_ascii_case("ifc"),
            Self::Step => {
                extension.eq_ignore_ascii_case("step") || extension.eq_ignore_ascii_case("stp")
            }
            Self::Dwg => extension.eq_ignore_ascii_case("dwg"),
            Self::Gltf => extension.eq_ignore_ascii_case("gltf"),
            Self::Glb => extension.eq_ignore_ascii_case("glb"),
            Self::Obj => extension.eq_ignore_ascii_case("obj"),
            Self::Stl => extension.eq_ignore_ascii_case("stl"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceWarningCode {
    UnitsUnknown,
    CoordinateSystemUnknown,
    ContentAlreadyPresent,
    ExtensionMissing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceWarning {
    pub code: SourceWarningCode,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceAlias {
    pub display_name: String,
    pub added_at: String,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceProvenance {
    pub origin_kind: String,
    pub supplied_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRecord {
    pub id: SourceId,
    pub sha256: String,
    pub byte_size: u64,
    pub detected_media_type: SourceMediaType,
    pub media_type: String,
    pub object_path: String,
    pub imported_at: String,
    pub aliases: Vec<SourceAlias>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub units: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coordinate_system: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<SourceWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceDerivativeKind {
    Thumbnail,
    PageImage,
    ExtractedText,
    VectorIndex,
    CadIndex,
    BimIndex,
    MeshIndex,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceDerivative {
    pub id: SourceDerivativeId,
    pub source_id: SourceId,
    pub source_sha256: String,
    pub kind: SourceDerivativeKind,
    pub payload_sha256: String,
    pub byte_size: u64,
    pub media_type: String,
    pub object_path: String,
    pub parser: String,
    pub parser_version: String,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub units: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coordinate_system: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<SourceWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceImportJobStatus {
    Completed,
    Deduplicated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceImportJob {
    pub id: String,
    pub operation_version: String,
    pub source_id: SourceId,
    pub status: SourceImportJobStatus,
    pub alias: String,
    pub detected_media_type: SourceMediaType,
    pub byte_size: u64,
    pub started_at: String,
    pub completed_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLibraryIndex {
    pub schema_version: String,
    #[serde(default)]
    pub sources: BTreeMap<SourceId, SourceRecord>,
    #[serde(default)]
    pub derivatives: BTreeMap<SourceDerivativeId, SourceDerivative>,
    #[serde(default)]
    pub import_jobs: BTreeMap<String, SourceImportJob>,
}

impl Default for SourceLibraryIndex {
    fn default() -> Self {
        Self {
            schema_version: SOURCE_INDEX_SCHEMA_VERSION.into(),
            sources: BTreeMap::new(),
            derivatives: BTreeMap::new(),
            import_jobs: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLibraryPolicy {
    pub max_source_bytes: u64,
    pub max_sources: usize,
    pub max_aliases_per_source: usize,
    pub max_derivative_bytes: u64,
    pub max_derivatives_per_source: usize,
    pub max_import_millis: u64,
    /// Parser adapters must apply this limit before they publish page derivatives.
    pub max_pages: usize,
    /// CAD and BIM parser adapters must apply this limit while decoding entities.
    pub max_entities: usize,
    /// Archive-capable parser adapters must apply this decompression limit.
    pub max_decompressed_bytes: u64,
}

impl Default for SourceLibraryPolicy {
    fn default() -> Self {
        Self {
            max_source_bytes: 256 * 1024 * 1024,
            max_sources: 2_000,
            max_aliases_per_source: 64,
            max_derivative_bytes: 512 * 1024 * 1024,
            max_derivatives_per_source: 20_000,
            max_import_millis: 120_000,
            max_pages: 10_000,
            max_entities: 5_000_000,
            max_decompressed_bytes: 2 * 1024 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SourceImportRequest {
    pub selected_path: PathBuf,
    pub display_alias: Option<String>,
    pub expected_media_type: Option<SourceMediaType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceImportResult {
    pub record: SourceRecord,
    pub job: SourceImportJob,
    pub deduplicated: bool,
}

#[derive(Debug, Clone)]
pub struct SourceDerivativeRequest {
    pub source_id: SourceId,
    pub kind: SourceDerivativeKind,
    pub payload: Vec<u8>,
    pub media_type: String,
    pub parser: String,
    pub parser_version: String,
    pub units: Option<String>,
    pub coordinate_system: Option<String>,
    pub warnings: Vec<SourceWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceReference {
    pub owner_kind: String,
    pub owner_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locator: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRemovalResult {
    pub source_id: SourceId,
    pub removed_derivatives: usize,
    pub removed_files: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceImportCheckpoint {
    ReadProgress { bytes_read: u64 },
    Staged,
    Sniffed,
    OriginalPublished,
    BeforeIndexCommit,
}

#[derive(Debug)]
pub enum SourceLibraryError {
    ProjectPackage(String),
    Io(String),
    InvalidIndex(String),
    InvalidSourceId(String),
    InvalidSelectionPath(String),
    SymlinkRejected(String),
    UnsafeAlias(String),
    UnsupportedContent,
    CorruptContent(String),
    TypeSpoofing {
        alias: String,
        detected: SourceMediaType,
    },
    ExpectedTypeMismatch {
        expected: SourceMediaType,
        detected: SourceMediaType,
    },
    SourceTooLarge {
        size: u64,
        limit: u64,
    },
    SourceCountLimit {
        limit: usize,
    },
    AliasCountLimit {
        limit: usize,
    },
    DerivativeTooLarge {
        size: u64,
        limit: u64,
    },
    DerivativeCountLimit {
        limit: usize,
    },
    ImportTimedOut {
        limit_millis: u64,
    },
    SourceNotFound(SourceId),
    SourceReferenced {
        source_id: SourceId,
        references: Vec<SourceReference>,
    },
    UnsafeStoredPath(String),
    PolicyRejected(String),
}

impl std::fmt::Display for SourceLibraryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProjectPackage(message)
            | Self::Io(message)
            | Self::InvalidIndex(message)
            | Self::InvalidSelectionPath(message)
            | Self::SymlinkRejected(message)
            | Self::UnsafeAlias(message)
            | Self::CorruptContent(message)
            | Self::UnsafeStoredPath(message)
            | Self::PolicyRejected(message) => formatter.write_str(message),
            Self::InvalidSourceId(id) => write!(formatter, "invalid source id `{id}`"),
            Self::UnsupportedContent => formatter.write_str("source content type is unsupported"),
            Self::TypeSpoofing { alias, detected } => write!(
                formatter,
                "source alias `{alias}` does not match detected media type {detected:?}"
            ),
            Self::ExpectedTypeMismatch { expected, detected } => write!(
                formatter,
                "expected media type {expected:?}, but detected {detected:?}"
            ),
            Self::SourceTooLarge { size, limit } => {
                write!(formatter, "source size {size} exceeds limit {limit}")
            }
            Self::SourceCountLimit { limit } => {
                write!(formatter, "source count exceeds limit {limit}")
            }
            Self::AliasCountLimit { limit } => {
                write!(formatter, "source alias count exceeds limit {limit}")
            }
            Self::DerivativeTooLarge { size, limit } => {
                write!(formatter, "derivative size {size} exceeds limit {limit}")
            }
            Self::DerivativeCountLimit { limit } => {
                write!(formatter, "derivative count exceeds limit {limit}")
            }
            Self::ImportTimedOut { limit_millis } => write!(
                formatter,
                "source import exceeded time limit {limit_millis} ms"
            ),
            Self::SourceNotFound(id) => write!(formatter, "source `{id}` was not found"),
            Self::SourceReferenced {
                source_id,
                references,
            } => write!(
                formatter,
                "source `{source_id}` has {} active references",
                references.len()
            ),
        }
    }
}

impl std::error::Error for SourceLibraryError {}

impl From<std::io::Error> for SourceLibraryError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

impl From<serde_json::Error> for SourceLibraryError {
    fn from(value: serde_json::Error) -> Self {
        Self::InvalidIndex(value.to_string())
    }
}

pub fn list_sources(project_dir: &Path) -> Result<Vec<SourceRecord>, SourceLibraryError> {
    Ok(load_source_index(project_dir)?
        .sources
        .into_values()
        .collect())
}

pub fn inspect_source(
    project_dir: &Path,
    source_id: &SourceId,
) -> Result<SourceRecord, SourceLibraryError> {
    load_source_index(project_dir)?
        .sources
        .remove(source_id)
        .ok_or_else(|| SourceLibraryError::SourceNotFound(source_id.clone()))
}

pub fn read_source_original(
    project_dir: &Path,
    source_id: &SourceId,
) -> Result<Vec<u8>, SourceLibraryError> {
    let record = inspect_source(project_dir, source_id)?;
    let paths = project_package_paths(project_dir);
    let object = checked_library_path(&paths.sources_dir, &record.object_path)?;
    reject_symlinked_object_path(&paths.sources_dir, &object)?;
    verify_object(&object, &record.sha256, record.byte_size)?;
    Ok(fs::read(object)?)
}

pub fn source_derivatives(
    project_dir: &Path,
    source_id: &SourceId,
) -> Result<Vec<SourceDerivative>, SourceLibraryError> {
    let index = load_source_index(project_dir)?;
    if !index.sources.contains_key(source_id) {
        return Err(SourceLibraryError::SourceNotFound(source_id.clone()));
    }
    Ok(index
        .derivatives
        .into_values()
        .filter(|derivative| &derivative.source_id == source_id)
        .collect())
}

pub fn read_source_derivative(
    project_dir: &Path,
    derivative_id: &SourceDerivativeId,
) -> Result<(SourceDerivative, Vec<u8>), SourceLibraryError> {
    let index = load_source_index(project_dir)?;
    let derivative = index
        .derivatives
        .get(derivative_id)
        .cloned()
        .ok_or_else(|| {
            SourceLibraryError::InvalidIndex(format!("derivative `{derivative_id}` was not found"))
        })?;
    let paths = project_package_paths(project_dir);
    let object = checked_library_path(&paths.sources_dir, &derivative.object_path)?;
    reject_symlinked_object_path(&paths.sources_dir, &object)?;
    verify_object(&object, &derivative.payload_sha256, derivative.byte_size)?;
    Ok((derivative, fs::read(object)?))
}

pub fn import_source(
    project_dir: &Path,
    request: SourceImportRequest,
) -> Result<SourceImportResult, SourceLibraryError> {
    import_source_with_policy_and_hook(
        project_dir,
        request,
        &SourceLibraryPolicy::default(),
        |_| Ok(()),
    )
}

pub fn import_source_with_policy_and_hook<F>(
    project_dir: &Path,
    request: SourceImportRequest,
    policy: &SourceLibraryPolicy,
    mut hook: F,
) -> Result<SourceImportResult, SourceLibraryError>
where
    F: FnMut(SourceImportCheckpoint) -> Result<(), SourceLibraryError>,
{
    validate_policy(policy)?;
    ensure_source_library(project_dir)?;
    validate_selected_path(&request.selected_path)?;
    let (mut source, metadata) = open_selected_file(&request.selected_path)?;
    if metadata.len() > policy.max_source_bytes {
        return Err(SourceLibraryError::SourceTooLarge {
            size: metadata.len(),
            limit: policy.max_source_bytes,
        });
    }

    let alias = request.display_alias.unwrap_or_else(|| {
        request
            .selected_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_owned()
    });
    validate_alias(&alias)?;
    let started = Instant::now();
    let started_at = iso_now();
    let paths = project_package_paths(project_dir);
    let staging_dir = paths.sources_dir.join(".staging");
    ensure_internal_directory(&paths.sources_dir, &staging_dir)?;
    let stage_path = staging_dir.join(unique_operation_id("import"));
    let mut staged = StagedFile::create(stage_path)?;
    let mut hasher = Sha256::new();
    let mut total = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        enforce_time(started, policy)?;
        let read = source.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        total = total.saturating_add(read as u64);
        if total > policy.max_source_bytes {
            return Err(SourceLibraryError::SourceTooLarge {
                size: total,
                limit: policy.max_source_bytes,
            });
        }
        staged.file.write_all(&buffer[..read])?;
        hasher.update(&buffer[..read]);
        hook(SourceImportCheckpoint::ReadProgress { bytes_read: total })?;
    }
    staged.file.sync_all()?;
    if total != metadata.len() {
        return Err(SourceLibraryError::CorruptContent(
            "source changed while it was being imported".into(),
        ));
    }
    hook(SourceImportCheckpoint::Staged)?;
    enforce_time(started, policy)?;

    let detected = sniff_staged_media(staged.path(), total)?;
    validate_alias_extension(&alias, detected)?;
    if let Some(expected) = request.expected_media_type
        && expected != detected
    {
        return Err(SourceLibraryError::ExpectedTypeMismatch { expected, detected });
    }
    hook(SourceImportCheckpoint::Sniffed)?;

    let sha256 = format!("{:x}", hasher.finalize());
    let source_id = SourceId::from_sha256(&sha256)?;
    let mut index = load_source_index(project_dir)?;
    let existing = index.sources.get(&source_id).cloned();
    if existing.is_none() && index.sources.len() >= policy.max_sources {
        return Err(SourceLibraryError::SourceCountLimit {
            limit: policy.max_sources,
        });
    }
    if existing.as_ref().is_some_and(|record| {
        record.aliases.len() >= policy.max_aliases_per_source
            && !record.aliases.iter().any(|item| item.display_name == alias)
    }) {
        return Err(SourceLibraryError::AliasCountLimit {
            limit: policy.max_aliases_per_source,
        });
    }

    let object_path = original_relative_path(&sha256);
    let object = checked_library_path(&paths.sources_dir, &object_path)?;
    ensure_internal_directory(
        &paths.sources_dir,
        object.parent().expect("source object has a parent"),
    )?;
    let deduplicated = existing.is_some() || object.exists();
    if object.exists() {
        verify_object(&object, &sha256, total)?;
    } else {
        staged.publish(&object)?;
        sync_directory(object.parent().expect("source object has a parent"))?;
    }
    hook(SourceImportCheckpoint::OriginalPublished)?;

    let provenance = SourceProvenance {
        origin_kind: "local_file_selection".into(),
        supplied_name: alias.clone(),
    };
    let alias_record = SourceAlias {
        display_name: alias.clone(),
        added_at: iso_now(),
        provenance,
    };
    let mut warnings = vec![
        SourceWarning {
            code: SourceWarningCode::UnitsUnknown,
            message: "Units have not been extracted or confirmed.".into(),
        },
        SourceWarning {
            code: SourceWarningCode::CoordinateSystemUnknown,
            message: "Coordinate system has not been extracted or confirmed.".into(),
        },
    ];
    if Path::new(&alias).extension().is_none() {
        warnings.push(SourceWarning {
            code: SourceWarningCode::ExtensionMissing,
            message: "The display alias has no file extension; content was identified from bytes."
                .into(),
        });
    }
    if deduplicated {
        warnings.push(SourceWarning {
            code: SourceWarningCode::ContentAlreadyPresent,
            message: "Identical source bytes were already present; the original was reused.".into(),
        });
    }
    let record = if let Some(mut record) = existing {
        if !record.aliases.iter().any(|item| item.display_name == alias) {
            record.aliases.push(alias_record);
        }
        record
    } else {
        SourceRecord {
            id: source_id.clone(),
            sha256: sha256.clone(),
            byte_size: total,
            detected_media_type: detected,
            media_type: detected.media_type().into(),
            object_path: object_path.clone(),
            imported_at: iso_now(),
            aliases: vec![alias_record],
            units: None,
            coordinate_system: None,
            warnings,
        }
    };
    validate_record(&record)?;
    index.sources.insert(source_id.clone(), record.clone());
    let job_id = unique_operation_id("source-import");
    let job = SourceImportJob {
        id: job_id.clone(),
        operation_version: SOURCE_LIBRARY_OPERATION_VERSION.into(),
        source_id,
        status: if deduplicated {
            SourceImportJobStatus::Deduplicated
        } else {
            SourceImportJobStatus::Completed
        },
        alias,
        detected_media_type: detected,
        byte_size: total,
        started_at,
        completed_at: iso_now(),
    };
    index.import_jobs.insert(job_id, job.clone());
    hook(SourceImportCheckpoint::BeforeIndexCommit)?;
    enforce_time(started, policy)?;
    save_source_index(project_dir, &index)?;
    Ok(SourceImportResult {
        record,
        job,
        deduplicated,
    })
}

pub fn store_source_derivative(
    project_dir: &Path,
    request: SourceDerivativeRequest,
    policy: &SourceLibraryPolicy,
) -> Result<SourceDerivative, SourceLibraryError> {
    validate_policy(policy)?;
    validate_short_token("parser", &request.parser)?;
    validate_short_token("parser version", &request.parser_version)?;
    validate_media_type(&request.media_type)?;
    if request.payload.len() as u64 > policy.max_derivative_bytes {
        return Err(SourceLibraryError::DerivativeTooLarge {
            size: request.payload.len() as u64,
            limit: policy.max_derivative_bytes,
        });
    }
    let mut index = load_source_index(project_dir)?;
    let source = index
        .sources
        .get(&request.source_id)
        .ok_or_else(|| SourceLibraryError::SourceNotFound(request.source_id.clone()))?
        .clone();
    let count = index
        .derivatives
        .values()
        .filter(|item| item.source_id == request.source_id)
        .count();
    if count >= policy.max_derivatives_per_source {
        return Err(SourceLibraryError::DerivativeCountLimit {
            limit: policy.max_derivatives_per_source,
        });
    }
    let payload_sha256 = sha256_bytes(&request.payload);
    let identity_material = serde_json::to_vec(&(
        request.source_id.as_str(),
        &request.kind,
        &payload_sha256,
        &request.parser,
        &request.parser_version,
        &request.media_type,
    ))?;
    let identity = sha256_bytes(&identity_material);
    let id = SourceDerivativeId(format!("derivative-{identity}"));
    if let Some(existing) = index.derivatives.get(&id) {
        return Ok(existing.clone());
    }
    let object_path = format!("derived/{}/{identity}", source.sha256);
    let paths = project_package_paths(project_dir);
    let object = checked_library_path(&paths.sources_dir, &object_path)?;
    ensure_internal_directory(
        &paths.sources_dir,
        object.parent().expect("derivative object has a parent"),
    )?;
    publish_bytes_if_absent(&object, &request.payload, &payload_sha256)?;
    let derivative = SourceDerivative {
        id: id.clone(),
        source_id: request.source_id,
        source_sha256: source.sha256.clone(),
        kind: request.kind,
        payload_sha256,
        byte_size: request.payload.len() as u64,
        media_type: request.media_type,
        object_path,
        parser: request.parser,
        parser_version: request.parser_version,
        created_at: iso_now(),
        units: request.units,
        coordinate_system: request.coordinate_system,
        warnings: request.warnings,
    };
    validate_derivative(&derivative, &source)?;
    index.derivatives.insert(id, derivative.clone());
    save_source_index(project_dir, &index)?;
    Ok(derivative)
}

pub fn remove_source(
    project_dir: &Path,
    source_id: &SourceId,
    active_references: &[SourceReference],
) -> Result<SourceRemovalResult, SourceLibraryError> {
    if !active_references.is_empty() {
        return Err(SourceLibraryError::SourceReferenced {
            source_id: source_id.clone(),
            references: active_references.to_vec(),
        });
    }
    let mut index = load_source_index(project_dir)?;
    let record = index
        .sources
        .remove(source_id)
        .ok_or_else(|| SourceLibraryError::SourceNotFound(source_id.clone()))?;
    let derivative_ids = index
        .derivatives
        .iter()
        .filter(|(_, derivative)| &derivative.source_id == source_id)
        .map(|(id, _)| id.clone())
        .collect::<Vec<_>>();
    let derivatives = derivative_ids
        .iter()
        .filter_map(|id| index.derivatives.remove(id))
        .collect::<Vec<_>>();
    index
        .import_jobs
        .retain(|_, job| &job.source_id != source_id);
    save_source_index(project_dir, &index)?;

    let paths = project_package_paths(project_dir);
    let mut removed_files = 0usize;
    for derivative in &derivatives {
        let path = checked_library_path(&paths.sources_dir, &derivative.object_path)?;
        reject_symlinked_object_path(&paths.sources_dir, &path)?;
        if path.is_file() {
            fs::remove_file(path)?;
            removed_files += 1;
        }
    }
    let original = checked_library_path(&paths.sources_dir, &record.object_path)?;
    reject_symlinked_object_path(&paths.sources_dir, &original)?;
    if original.is_file() {
        fs::remove_file(original)?;
        removed_files += 1;
    }
    Ok(SourceRemovalResult {
        source_id: source_id.clone(),
        removed_derivatives: derivatives.len(),
        removed_files,
    })
}

pub fn remove_source_alias(
    project_dir: &Path,
    source_id: &SourceId,
    alias: &str,
) -> Result<SourceRecord, SourceLibraryError> {
    validate_alias(alias)?;
    let mut index = load_source_index(project_dir)?;
    let record = index
        .sources
        .get_mut(source_id)
        .ok_or_else(|| SourceLibraryError::SourceNotFound(source_id.clone()))?;
    record.aliases.retain(|item| item.display_name != alias);
    let result = record.clone();
    save_source_index(project_dir, &index)?;
    Ok(result)
}

fn load_source_index(project_dir: &Path) -> Result<SourceLibraryIndex, SourceLibraryError> {
    ensure_source_library(project_dir)?;
    let path = project_package_paths(project_dir).source_index;
    reject_symlink_if_present(&path)?;
    let bytes = fs::read(&path)?;
    let index: SourceLibraryIndex = serde_json::from_slice(&bytes)?;
    if index.schema_version != SOURCE_INDEX_SCHEMA_VERSION {
        return Err(SourceLibraryError::InvalidIndex(format!(
            "unsupported source index schema `{}`",
            index.schema_version
        )));
    }
    validate_index(&index)?;
    Ok(index)
}

fn save_source_index(
    project_dir: &Path,
    index: &SourceLibraryIndex,
) -> Result<(), SourceLibraryError> {
    validate_index(index)?;
    let path = project_package_paths(project_dir).source_index;
    atomic_write_json(&path, index)
}

fn ensure_source_library(project_dir: &Path) -> Result<(), SourceLibraryError> {
    load_project_package(project_dir)
        .map_err(|error| SourceLibraryError::ProjectPackage(error.to_string()))?;
    let paths = project_package_paths(project_dir);
    ensure_internal_directory(project_dir, &paths.sources_dir)?;
    ensure_internal_directory(&paths.sources_dir, &paths.source_originals_dir)?;
    ensure_internal_directory(&paths.sources_dir, &paths.source_derived_dir)?;
    recover_atomic_json(&paths.source_index)?;
    if !paths.source_index.exists() {
        atomic_write_json(&paths.source_index, &SourceLibraryIndex::default())?;
    }
    reject_symlink_if_present(&paths.source_index)
}

fn validate_index(index: &SourceLibraryIndex) -> Result<(), SourceLibraryError> {
    for (id, record) in &index.sources {
        if id != &record.id {
            return Err(SourceLibraryError::InvalidIndex(
                "source index key does not match record id".into(),
            ));
        }
        validate_record(record)?;
    }
    for (id, derivative) in &index.derivatives {
        if id != &derivative.id {
            return Err(SourceLibraryError::InvalidIndex(
                "derivative index key does not match record id".into(),
            ));
        }
        let source = index.sources.get(&derivative.source_id).ok_or_else(|| {
            SourceLibraryError::InvalidIndex("derivative references a missing source".into())
        })?;
        validate_derivative(derivative, source)?;
    }
    Ok(())
}

fn validate_record(record: &SourceRecord) -> Result<(), SourceLibraryError> {
    if record.id.sha256()? != record.sha256 {
        return Err(SourceLibraryError::InvalidIndex(
            "source id does not match source hash".into(),
        ));
    }
    if record.object_path != original_relative_path(&record.sha256) {
        return Err(SourceLibraryError::UnsafeStoredPath(
            record.object_path.clone(),
        ));
    }
    if record.media_type != record.detected_media_type.media_type() {
        return Err(SourceLibraryError::InvalidIndex(
            "source media type does not match detection".into(),
        ));
    }
    for alias in &record.aliases {
        validate_alias(&alias.display_name)?;
    }
    Ok(())
}

fn validate_derivative(
    derivative: &SourceDerivative,
    source: &SourceRecord,
) -> Result<(), SourceLibraryError> {
    if derivative.source_sha256 != source.sha256 {
        return Err(SourceLibraryError::InvalidIndex(
            "derivative original hash does not match source".into(),
        ));
    }
    validate_sha256(&derivative.payload_sha256)?;
    validate_short_token("parser", &derivative.parser)?;
    validate_short_token("parser version", &derivative.parser_version)?;
    validate_media_type(&derivative.media_type)?;
    let identity = derivative
        .id
        .as_str()
        .strip_prefix("derivative-")
        .ok_or_else(|| SourceLibraryError::InvalidIndex("invalid derivative id".into()))?;
    validate_sha256(identity)?;
    let expected_path = format!("derived/{}/{identity}", source.sha256);
    if derivative.object_path != expected_path || !safe_relative_path(&derivative.object_path) {
        return Err(SourceLibraryError::UnsafeStoredPath(
            derivative.object_path.clone(),
        ));
    }
    Ok(())
}

fn validate_policy(policy: &SourceLibraryPolicy) -> Result<(), SourceLibraryError> {
    if policy.max_source_bytes == 0
        || policy.max_sources == 0
        || policy.max_aliases_per_source == 0
        || policy.max_derivative_bytes == 0
        || policy.max_derivatives_per_source == 0
        || policy.max_import_millis == 0
        || policy.max_pages == 0
        || policy.max_entities == 0
        || policy.max_decompressed_bytes == 0
    {
        return Err(SourceLibraryError::PolicyRejected(
            "source policy limits must be positive".into(),
        ));
    }
    Ok(())
}

fn validate_selected_path(path: &Path) -> Result<(), SourceLibraryError> {
    if !path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
    {
        return Err(SourceLibraryError::InvalidSelectionPath(
            "source selection must be an absolute normalized path without traversal".into(),
        ));
    }
    Ok(())
}

fn open_selected_file(path: &Path) -> Result<(File, fs::Metadata), SourceLibraryError> {
    let before = fs::symlink_metadata(path)?;
    if before.file_type().is_symlink() {
        return Err(SourceLibraryError::SymlinkRejected(
            path.display().to_string(),
        ));
    }
    if !before.is_file() {
        return Err(SourceLibraryError::InvalidSelectionPath(
            "source selection must be a regular file".into(),
        ));
    }
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
        options.custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
    }
    let file = options.open(path).map_err(|error| {
        #[cfg(unix)]
        if error.raw_os_error() == Some(libc::ELOOP) {
            SourceLibraryError::SymlinkRejected(path.display().to_string())
        } else {
            error.into()
        }
        #[cfg(not(unix))]
        {
            error.into()
        }
    })?;
    let opened = file.metadata()?;
    if !opened.is_file() {
        return Err(SourceLibraryError::InvalidSelectionPath(
            "source selection must be a regular file".into(),
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if before.dev() != opened.dev() || before.ino() != opened.ino() {
            return Err(SourceLibraryError::InvalidSelectionPath(
                "source selection changed before it could be opened".into(),
            ));
        }
    }
    Ok((file, opened))
}

fn validate_alias(alias: &str) -> Result<(), SourceLibraryError> {
    let path = Path::new(alias);
    if alias.trim().is_empty()
        || alias.len() > 255
        || alias.chars().any(char::is_control)
        || path.is_absolute()
        || path.components().count() != 1
        || !matches!(path.components().next(), Some(Component::Normal(_)))
    {
        return Err(SourceLibraryError::UnsafeAlias(alias.into()));
    }
    Ok(())
}

fn validate_alias_extension(
    alias: &str,
    detected: SourceMediaType,
) -> Result<(), SourceLibraryError> {
    if let Some(extension) = Path::new(alias)
        .extension()
        .and_then(|value| value.to_str())
        && !detected.accepts_extension(extension)
    {
        return Err(SourceLibraryError::TypeSpoofing {
            alias: alias.into(),
            detected,
        });
    }
    Ok(())
}

fn validate_short_token(label: &str, value: &str) -> Result<(), SourceLibraryError> {
    if value.trim().is_empty() || value.len() > 128 || value.chars().any(char::is_control) {
        return Err(SourceLibraryError::PolicyRejected(format!(
            "{label} must be a short non-empty token"
        )));
    }
    Ok(())
}

fn validate_media_type(value: &str) -> Result<(), SourceLibraryError> {
    if value.trim().is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'+' | b'-' | b'.'))
    {
        return Err(SourceLibraryError::PolicyRejected(
            "invalid derivative media type".into(),
        ));
    }
    Ok(())
}

fn validate_sha256(hash: &str) -> Result<(), SourceLibraryError> {
    if hash.len() != 64
        || !hash
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(SourceLibraryError::InvalidSourceId(hash.into()));
    }
    Ok(())
}

fn original_relative_path(hash: &str) -> String {
    format!("originals/sha256/{}/{}", &hash[..2], hash)
}

fn safe_relative_path(value: &str) -> bool {
    let path = Path::new(value);
    !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn checked_library_path(root: &Path, relative: &str) -> Result<PathBuf, SourceLibraryError> {
    if !safe_relative_path(relative) {
        return Err(SourceLibraryError::UnsafeStoredPath(relative.into()));
    }
    Ok(root.join(relative))
}

fn reject_symlinked_object_path(root: &Path, path: &Path) -> Result<(), SourceLibraryError> {
    if !path.starts_with(root) {
        return Err(SourceLibraryError::UnsafeStoredPath(
            path.display().to_string(),
        ));
    }
    let relative = path
        .strip_prefix(root)
        .map_err(|_| SourceLibraryError::UnsafeStoredPath(path.display().to_string()))?;
    let mut current = root.to_path_buf();
    reject_symlink_if_present(&current)?;
    for component in relative.components() {
        let Component::Normal(name) = component else {
            return Err(SourceLibraryError::UnsafeStoredPath(
                path.display().to_string(),
            ));
        };
        current.push(name);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(SourceLibraryError::SymlinkRejected(
                    current.display().to_string(),
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn ensure_internal_directory(root: &Path, directory: &Path) -> Result<(), SourceLibraryError> {
    if !directory.starts_with(root) {
        return Err(SourceLibraryError::UnsafeStoredPath(
            directory.display().to_string(),
        ));
    }
    let relative = directory
        .strip_prefix(root)
        .map_err(|_| SourceLibraryError::UnsafeStoredPath(directory.display().to_string()))?;
    let mut current = root.to_path_buf();
    reject_symlink_if_present(&current)?;
    for component in relative.components() {
        let Component::Normal(name) = component else {
            return Err(SourceLibraryError::UnsafeStoredPath(
                directory.display().to_string(),
            ));
        };
        current.push(name);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(SourceLibraryError::SymlinkRejected(
                    current.display().to_string(),
                ));
            }
            Ok(metadata) if !metadata.is_dir() => {
                return Err(SourceLibraryError::UnsafeStoredPath(
                    current.display().to_string(),
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => fs::create_dir(&current)?,
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn reject_symlink_if_present(path: &Path) -> Result<(), SourceLibraryError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(
            SourceLibraryError::SymlinkRejected(path.display().to_string()),
        ),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn sniff_staged_media(path: &Path, size: u64) -> Result<SourceMediaType, SourceLibraryError> {
    if size == 0 {
        return Err(SourceLibraryError::CorruptContent(
            "source file is empty".into(),
        ));
    }
    let mut file = File::open(path)?;
    let prefix_len = usize::try_from(size.min(64 * 1024)).unwrap_or(64 * 1024);
    let mut prefix = vec![0u8; prefix_len];
    file.read_exact(&mut prefix)?;
    let tail_len = usize::try_from(size.min(64 * 1024)).unwrap_or(64 * 1024);
    let tail = if size as usize <= prefix_len {
        prefix.clone()
    } else {
        use std::io::{Seek, SeekFrom};
        file.seek(SeekFrom::End(-(tail_len as i64)))?;
        let mut tail = vec![0u8; tail_len];
        file.read_exact(&mut tail)?;
        tail
    };
    if prefix.starts_with(b"%PDF-") {
        if !tail.windows(5).any(|window| window == b"%%EOF") {
            return Err(SourceLibraryError::CorruptContent(
                "PDF has no end-of-file marker".into(),
            ));
        }
        return Ok(SourceMediaType::Pdf);
    }
    if prefix.starts_with(b"\x89PNG\r\n\x1a\n") {
        if prefix.len() < 24
            || &prefix[12..16] != b"IHDR"
            || !tail.windows(4).any(|window| window == b"IEND")
        {
            return Err(SourceLibraryError::CorruptContent(
                "PNG structure is incomplete".into(),
            ));
        }
        return Ok(SourceMediaType::Png);
    }
    if prefix.starts_with(&[0xff, 0xd8, 0xff]) {
        if !tail.ends_with(&[0xff, 0xd9]) {
            return Err(SourceLibraryError::CorruptContent(
                "JPEG has no end marker".into(),
            ));
        }
        return Ok(SourceMediaType::Jpeg);
    }
    if prefix.starts_with(b"II*\0") || prefix.starts_with(b"MM\0*") {
        if size < 8 {
            return Err(SourceLibraryError::CorruptContent(
                "TIFF header is incomplete".into(),
            ));
        }
        return Ok(SourceMediaType::Tiff);
    }
    if prefix.starts_with(b"AC10") {
        if size < 64 {
            return Err(SourceLibraryError::CorruptContent(
                "DWG container is too short".into(),
            ));
        }
        return Ok(SourceMediaType::Dwg);
    }
    if prefix.starts_with(b"glTF") {
        if size < 20 {
            return Err(SourceLibraryError::CorruptContent(
                "GLB header is incomplete".into(),
            ));
        }
        return Ok(SourceMediaType::Glb);
    }
    let text = std::str::from_utf8(&prefix).ok();
    let tail_text = std::str::from_utf8(&tail).ok();
    if text.is_some_and(|value| {
        let trimmed = value.trim_start();
        trimmed.starts_with('{') && trimmed.contains("\"asset\"") && trimmed.contains("\"version\"")
    }) {
        return Ok(SourceMediaType::Gltf);
    }
    if text.is_some_and(|value| {
        value
            .lines()
            .any(|line| line.trim_start().starts_with("v "))
            && value
                .lines()
                .any(|line| line.trim_start().starts_with("f "))
    }) {
        return Ok(SourceMediaType::Obj);
    }
    if size >= 84 && prefix.len() >= 84 {
        let triangle_count = u32::from_le_bytes(prefix[80..84].try_into().unwrap()) as u64;
        if 84_u64.saturating_add(triangle_count.saturating_mul(50)) == size {
            return Ok(SourceMediaType::Stl);
        }
    }
    if text.is_some_and(|value| {
        value.trim_start().starts_with("solid ") && value.contains("facet normal")
    }) && tail_text.is_some_and(|value| value.contains("endsolid"))
    {
        return Ok(SourceMediaType::Stl);
    }
    if text.is_some_and(|value| {
        value.contains("SECTION")
            && value.contains("HEADER")
            && value.lines().any(|line| line.trim() == "$ACADVER")
    }) {
        if !tail_text.is_some_and(|value| value.lines().any(|line| line.trim() == "EOF")) {
            return Err(SourceLibraryError::CorruptContent(
                "DXF has no EOF record".into(),
            ));
        }
        return Ok(SourceMediaType::Dxf);
    }
    if text.is_some_and(|value| value.trim_start().starts_with("ISO-10303-21;")) {
        if !tail_text.is_some_and(|value| value.contains("END-ISO-10303-21;")) {
            return Err(SourceLibraryError::CorruptContent(
                "STEP container has no end marker".into(),
            ));
        }
        if text.is_some_and(|value| value.to_ascii_uppercase().contains("FILE_SCHEMA(('IFC")) {
            return Ok(SourceMediaType::IfcStep);
        }
        return Ok(SourceMediaType::Step);
    }
    Err(SourceLibraryError::UnsupportedContent)
}

fn enforce_time(started: Instant, policy: &SourceLibraryPolicy) -> Result<(), SourceLibraryError> {
    let limit = Duration::from_millis(policy.max_import_millis);
    if started.elapsed() > limit {
        return Err(SourceLibraryError::ImportTimedOut {
            limit_millis: policy.max_import_millis,
        });
    }
    Ok(())
}

fn verify_object(
    path: &Path,
    expected_hash: &str,
    expected_size: u64,
) -> Result<(), SourceLibraryError> {
    reject_symlink_if_present(path)?;
    let metadata = fs::metadata(path)?;
    if !metadata.is_file() || metadata.len() != expected_size {
        return Err(SourceLibraryError::CorruptContent(
            "content-addressed object has an unexpected size".into(),
        ));
    }
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut HashWriter(&mut hasher))?;
    let actual = format!("{:x}", hasher.finalize());
    if actual != expected_hash {
        return Err(SourceLibraryError::CorruptContent(
            "content-addressed object hash mismatch".into(),
        ));
    }
    Ok(())
}

struct HashWriter<'a>(&'a mut Sha256);

impl Write for HashWriter<'_> {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        self.0.update(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn publish_bytes_if_absent(
    path: &Path,
    bytes: &[u8],
    hash: &str,
) -> Result<(), SourceLibraryError> {
    if path.exists() {
        return verify_object(path, hash, bytes.len() as u64);
    }
    let parent = path
        .parent()
        .ok_or_else(|| SourceLibraryError::UnsafeStoredPath(path.display().to_string()))?;
    let stage = parent.join(format!(".{}.tmp", unique_operation_id("derivative")));
    let mut staged = StagedFile::create(stage)?;
    staged.file.write_all(bytes)?;
    staged.file.sync_all()?;
    staged.publish(path)?;
    sync_directory(parent)
}

fn atomic_write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), SourceLibraryError> {
    reject_symlink_if_present(path)?;
    let parent = path
        .parent()
        .ok_or_else(|| SourceLibraryError::UnsafeStoredPath(path.display().to_string()))?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| SourceLibraryError::UnsafeStoredPath(path.display().to_string()))?;
    let temporary = parent.join(format!(".{name}.{}", unique_operation_id("sources-tmp")));
    let backup = parent.join(format!(".{name}.sources-bak"));
    recover_atomic_json(path)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    serde_json::to_writer_pretty(&mut file, value)?;
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

fn recover_atomic_json(path: &Path) -> Result<(), SourceLibraryError> {
    let parent = path
        .parent()
        .ok_or_else(|| SourceLibraryError::UnsafeStoredPath(path.display().to_string()))?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| SourceLibraryError::UnsafeStoredPath(path.display().to_string()))?;
    let backup = parent.join(format!(".{name}.sources-bak"));
    reject_symlink_if_present(&backup)?;
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

fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn unique_operation_id(prefix: &str) -> String {
    static NEXT_ID: AtomicU64 = AtomicU64::new(0);
    format!(
        "{prefix}-{}-{}-{}",
        std::process::id(),
        timestamp_id(),
        NEXT_ID.fetch_add(1, Ordering::Relaxed)
    )
}

struct StagedFile {
    path: PathBuf,
    file: File,
    published: bool,
}

impl StagedFile {
    fn create(path: PathBuf) -> Result<Self, SourceLibraryError> {
        reject_symlink_if_present(&path)?;
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        Ok(Self {
            path,
            file,
            published: false,
        })
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn publish(&mut self, target: &Path) -> Result<(), SourceLibraryError> {
        reject_symlink_if_present(target)?;
        fs::hard_link(&self.path, target)?;
        File::open(target)?.sync_all()?;
        fs::remove_file(&self.path)?;
        self.published = true;
        Ok(())
    }
}

impl Drop for StagedFile {
    fn drop(&mut self) {
        if !self.published {
            let _ = fs::remove_file(&self.path);
        }
    }
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<(), SourceLibraryError> {
    File::open(path)?.sync_all()?;
    Ok(())
}

#[cfg(windows)]
fn sync_directory(_path: &Path) -> Result<(), SourceLibraryError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_named_project_package;

    struct Fixture {
        root: PathBuf,
        project: PathBuf,
    }

    impl Fixture {
        fn new(label: &str) -> Self {
            let root =
                std::env::temp_dir().join(unique_operation_id(&format!("fraia-sources-{label}")));
            fs::create_dir(&root).expect("create fixture root");
            let project = root.join("project");
            create_named_project_package(&project, "Source fixture")
                .expect("create project package");
            Self { root, project }
        }

        fn input(&self, name: &str, bytes: &[u8]) -> PathBuf {
            let path = self.root.join(name);
            fs::write(&path, bytes).expect("write source fixture");
            path
        }

        fn import(&self, name: &str, bytes: &[u8]) -> SourceImportResult {
            let path = self.input(name, bytes);
            import_source(
                &self.project,
                SourceImportRequest {
                    selected_path: path,
                    display_alias: None,
                    expected_media_type: None,
                },
            )
            .expect("import source fixture")
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn pdf() -> &'static [u8] {
        b"%PDF-1.7\n1 0 obj\n<<>>\nendobj\nstartxref\n0\n%%EOF\n"
    }

    fn png() -> Vec<u8> {
        let mut bytes = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
        bytes.extend_from_slice(&[0; 20]);
        bytes.extend_from_slice(b"IEND");
        bytes
    }

    #[test]
    fn imports_deduplicate_bytes_add_safe_aliases_and_survive_project_move() {
        let fixture = Fixture::new("dedup-move");
        let first = fixture.import("plan.pdf", pdf());
        let duplicate_path = fixture.input("elevation.pdf", pdf());
        let second = import_source(
            &fixture.project,
            SourceImportRequest {
                selected_path: duplicate_path,
                display_alias: None,
                expected_media_type: Some(SourceMediaType::Pdf),
            },
        )
        .expect("deduplicate source");

        assert_eq!(first.record.id, second.record.id);
        assert!(second.deduplicated);
        assert_eq!(second.job.status, SourceImportJobStatus::Deduplicated);
        assert_eq!(second.record.aliases.len(), 2);
        assert!(!Path::new(&second.record.object_path).is_absolute());

        let moved = fixture.root.join("moved-project");
        fs::rename(&fixture.project, &moved).expect("move project package");
        let inspected = inspect_source(&moved, &first.record.id).expect("inspect moved source");
        let object = project_package_paths(&moved)
            .sources_dir
            .join(inspected.object_path);
        assert_eq!(fs::read(object).expect("read moved original"), pdf());
    }

    #[test]
    fn stores_deterministic_derivatives_with_exact_original_and_parser_provenance() {
        let fixture = Fixture::new("derivative");
        let source = fixture.import("plan.pdf", pdf()).record;
        let request = SourceDerivativeRequest {
            source_id: source.id.clone(),
            kind: SourceDerivativeKind::ExtractedText,
            payload: b"level 1 plan".to_vec(),
            media_type: "text/plain".into(),
            parser: "fixture-parser".into(),
            parser_version: "1.2.3".into(),
            units: None,
            coordinate_system: None,
            warnings: Vec::new(),
        };
        let first = store_source_derivative(
            &fixture.project,
            request.clone(),
            &SourceLibraryPolicy::default(),
        )
        .expect("store derivative");
        let second =
            store_source_derivative(&fixture.project, request, &SourceLibraryPolicy::default())
                .expect("deduplicate derivative");

        assert_eq!(first, second);
        assert_eq!(first.source_sha256, source.sha256);
        assert_eq!(first.parser, "fixture-parser");
        assert_eq!(first.parser_version, "1.2.3");
        assert_eq!(
            source_derivatives(&fixture.project, &source.id).expect("query derivatives"),
            vec![first]
        );
    }

    #[test]
    fn import_interruption_never_publishes_an_index_record_and_retry_succeeds() {
        for checkpoint in [
            SourceImportCheckpoint::Staged,
            SourceImportCheckpoint::Sniffed,
            SourceImportCheckpoint::OriginalPublished,
            SourceImportCheckpoint::BeforeIndexCommit,
        ] {
            let fixture = Fixture::new("interrupt");
            let path = fixture.input("plan.pdf", pdf());
            let target = std::mem::discriminant(&checkpoint);
            let result = import_source_with_policy_and_hook(
                &fixture.project,
                SourceImportRequest {
                    selected_path: path.clone(),
                    display_alias: None,
                    expected_media_type: None,
                },
                &SourceLibraryPolicy::default(),
                |actual| {
                    if std::mem::discriminant(&actual) == target {
                        Err(SourceLibraryError::PolicyRejected(
                            "injected interruption".into(),
                        ))
                    } else {
                        Ok(())
                    }
                },
            );
            assert!(matches!(result, Err(SourceLibraryError::PolicyRejected(_))));
            assert!(
                list_sources(&fixture.project)
                    .expect("list after interruption")
                    .is_empty()
            );
            let retry = import_source(
                &fixture.project,
                SourceImportRequest {
                    selected_path: path,
                    display_alias: None,
                    expected_media_type: None,
                },
            );
            assert!(retry.is_ok());
        }
    }

    #[test]
    fn rejects_traversal_unsafe_aliases_spoofed_corrupt_unsupported_and_oversize_inputs() {
        let fixture = Fixture::new("adversarial");
        let traversal = fixture.root.join("missing").join("..").join("plan.pdf");
        let error = import_source(
            &fixture.project,
            SourceImportRequest {
                selected_path: traversal,
                display_alias: None,
                expected_media_type: None,
            },
        )
        .expect_err("reject traversal");
        assert!(matches!(error, SourceLibraryError::InvalidSelectionPath(_)));

        let png_path = fixture.input("actual.png", &png());
        let error = import_source(
            &fixture.project,
            SourceImportRequest {
                selected_path: png_path.clone(),
                display_alias: Some("drawing.pdf".into()),
                expected_media_type: None,
            },
        )
        .expect_err("reject spoofed extension");
        assert!(matches!(error, SourceLibraryError::TypeSpoofing { .. }));
        let error = import_source(
            &fixture.project,
            SourceImportRequest {
                selected_path: png_path,
                display_alias: Some("../drawing.png".into()),
                expected_media_type: None,
            },
        )
        .expect_err("reject unsafe alias");
        assert!(matches!(error, SourceLibraryError::UnsafeAlias(_)));

        let corrupt = fixture.input("broken.pdf", b"%PDF-1.7\nno trailer");
        let error = import_source(
            &fixture.project,
            SourceImportRequest {
                selected_path: corrupt,
                display_alias: None,
                expected_media_type: None,
            },
        )
        .expect_err("reject corrupt PDF");
        assert!(matches!(error, SourceLibraryError::CorruptContent(_)));
        let unsupported = fixture.input("archive.zip", b"PK\x03\x04not-a-supported-source");
        let error = import_source(
            &fixture.project,
            SourceImportRequest {
                selected_path: unsupported,
                display_alias: None,
                expected_media_type: None,
            },
        )
        .expect_err("reject unsupported source");
        assert!(matches!(error, SourceLibraryError::UnsupportedContent));

        let oversized = fixture.input("large.pdf", pdf());
        let policy = SourceLibraryPolicy {
            max_source_bytes: 4,
            ..SourceLibraryPolicy::default()
        };
        let error = import_source_with_policy_and_hook(
            &fixture.project,
            SourceImportRequest {
                selected_path: oversized,
                display_alias: None,
                expected_media_type: None,
            },
            &policy,
            |_| Ok(()),
        )
        .expect_err("reject oversize source");
        assert!(matches!(error, SourceLibraryError::SourceTooLarge { .. }));
        assert!(
            list_sources(&fixture.project)
                .expect("list after rejections")
                .is_empty()
        );
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_inputs_and_symlinked_internal_storage() {
        use std::os::unix::fs::symlink;

        let fixture = Fixture::new("symlink");
        let input = fixture.input("real.pdf", pdf());
        let link = fixture.root.join("link.pdf");
        symlink(&input, &link).expect("create input symlink");
        let error = import_source(
            &fixture.project,
            SourceImportRequest {
                selected_path: link,
                display_alias: None,
                expected_media_type: None,
            },
        )
        .expect_err("reject input symlink");
        assert!(matches!(error, SourceLibraryError::SymlinkRejected(_)));

        let paths = project_package_paths(&fixture.project);
        fs::remove_dir(&paths.source_derived_dir).expect("remove empty derived directory");
        symlink(&fixture.root, &paths.source_derived_dir).expect("create internal symlink");
        let error = list_sources(&fixture.project).expect_err("reject internal symlink");
        assert!(matches!(error, SourceLibraryError::SymlinkRejected(_)));
    }

    #[test]
    fn reference_safe_removal_refuses_live_owners_then_removes_original_and_derivatives() {
        let fixture = Fixture::new("remove");
        let source = fixture.import("plan.pdf", pdf()).record;
        let derivative = store_source_derivative(
            &fixture.project,
            SourceDerivativeRequest {
                source_id: source.id.clone(),
                kind: SourceDerivativeKind::Thumbnail,
                payload: png(),
                media_type: "image/png".into(),
                parser: "fixture-parser".into(),
                parser_version: "1".into(),
                units: None,
                coordinate_system: None,
                warnings: Vec::new(),
            },
            &SourceLibraryPolicy::default(),
        )
        .expect("store derivative");
        let reference = SourceReference {
            owner_kind: "design_shelf".into(),
            owner_id: "design-1".into(),
            locator: Some("page-1".into()),
        };
        let error = remove_source(
            &fixture.project,
            &source.id,
            std::slice::from_ref(&reference),
        )
        .expect_err("refuse referenced removal");
        assert!(
            matches!(error, SourceLibraryError::SourceReferenced { references, .. } if references == vec![reference])
        );
        assert!(inspect_source(&fixture.project, &source.id).is_ok());

        let removed =
            remove_source(&fixture.project, &source.id, &[]).expect("remove unreferenced source");
        assert_eq!(removed.removed_derivatives, 1);
        assert_eq!(removed.removed_files, 2);
        assert!(
            list_sources(&fixture.project)
                .expect("list after removal")
                .is_empty()
        );
        assert!(
            !project_package_paths(&fixture.project)
                .sources_dir
                .join(source.object_path)
                .exists()
        );
        assert!(
            !project_package_paths(&fixture.project)
                .sources_dir
                .join(derivative.object_path)
                .exists()
        );
    }

    #[test]
    fn recovers_interrupted_index_replacement_from_package_owned_backup() {
        let fixture = Fixture::new("index-recovery");
        let imported = fixture.import("plan.pdf", pdf()).record;
        let index = project_package_paths(&fixture.project).source_index;
        let backup = index
            .parent()
            .expect("index parent")
            .join(".source-index.json.sources-bak");
        fs::rename(&index, &backup).expect("simulate interrupted index replacement");

        assert_eq!(
            inspect_source(&fixture.project, &imported.id).expect("recover index"),
            imported
        );
        assert!(index.is_file());
        assert!(!backup.exists());
    }

    #[test]
    fn applies_count_derivative_and_time_policy_hooks_before_index_publication() {
        let fixture = Fixture::new("policy-hooks");
        let first_path = fixture.input("first.pdf", pdf());
        let timed_policy = SourceLibraryPolicy {
            max_import_millis: 1,
            ..SourceLibraryPolicy::default()
        };
        let timed = import_source_with_policy_and_hook(
            &fixture.project,
            SourceImportRequest {
                selected_path: first_path.clone(),
                display_alias: None,
                expected_media_type: None,
            },
            &timed_policy,
            |checkpoint| {
                if checkpoint == SourceImportCheckpoint::Staged {
                    std::thread::sleep(Duration::from_millis(3));
                }
                Ok(())
            },
        )
        .expect_err("apply import time limit");
        assert!(matches!(timed, SourceLibraryError::ImportTimedOut { .. }));
        assert!(
            list_sources(&fixture.project)
                .expect("list after timeout")
                .is_empty()
        );

        let first = import_source(
            &fixture.project,
            SourceImportRequest {
                selected_path: first_path,
                display_alias: None,
                expected_media_type: None,
            },
        )
        .expect("import first source")
        .record;
        let second_path = fixture.input("second.pdf", b"%PDF-1.7\nsecond\n%%EOF\n");
        let count_policy = SourceLibraryPolicy {
            max_sources: 1,
            ..SourceLibraryPolicy::default()
        };
        let count_error = import_source_with_policy_and_hook(
            &fixture.project,
            SourceImportRequest {
                selected_path: second_path,
                display_alias: None,
                expected_media_type: None,
            },
            &count_policy,
            |_| Ok(()),
        )
        .expect_err("apply source count limit");
        assert!(matches!(
            count_error,
            SourceLibraryError::SourceCountLimit { limit: 1 }
        ));

        let derivative_policy = SourceLibraryPolicy {
            max_derivatives_per_source: 1,
            ..SourceLibraryPolicy::default()
        };
        let derivative = |payload: &[u8]| SourceDerivativeRequest {
            source_id: first.id.clone(),
            kind: SourceDerivativeKind::ExtractedText,
            payload: payload.to_vec(),
            media_type: "text/plain".into(),
            parser: "fixture-parser".into(),
            parser_version: "1".into(),
            units: None,
            coordinate_system: None,
            warnings: Vec::new(),
        };
        store_source_derivative(&fixture.project, derivative(b"one"), &derivative_policy)
            .expect("store first derivative");
        let derivative_error =
            store_source_derivative(&fixture.project, derivative(b"two"), &derivative_policy)
                .expect_err("apply derivative count limit");
        assert!(matches!(
            derivative_error,
            SourceLibraryError::DerivativeCountLimit { limit: 1 }
        ));
    }

    #[test]
    fn rejects_tampered_index_paths_instead_of_following_them() {
        let fixture = Fixture::new("tampered-index");
        fixture.import("plan.pdf", pdf());
        let index_path = project_package_paths(&fixture.project).source_index;
        let mut index: SourceLibraryIndex =
            serde_json::from_slice(&fs::read(&index_path).expect("read source index"))
                .expect("decode source index");
        index
            .sources
            .values_mut()
            .next()
            .expect("source record")
            .object_path = "../outside.pdf".into();
        fs::write(
            &index_path,
            serde_json::to_vec_pretty(&index).expect("encode tampered index"),
        )
        .expect("write tampered index");

        let error = list_sources(&fixture.project).expect_err("reject tampered index path");
        assert!(matches!(error, SourceLibraryError::UnsafeStoredPath(_)));
    }
}
