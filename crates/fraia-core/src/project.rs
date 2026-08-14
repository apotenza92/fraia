use crate::archetypes::{
    builder_graph_from_legacy_builder, materialize_structural_model_from_builder_graph,
};
use crate::structural_app::StructuralModel;
use crate::types::{
    AgentState, BuilderNodeParameters, DesignId, DesignManifest, DesignManifestFiles, Intent,
    LegacyProjectMigration, PlanningAnalysisBrief, PlanningDesignConstraints, PlanningDraft,
    PlanningGeometryAndLoads, PlanningProjectIntent, PlanningSystemBrief, ProjectDesignEntry,
    ProjectFile, ProjectFiles, ProjectId, ProjectManifest, ProjectManifestFiles, Requirements,
    SearchPermissions,
};
use crate::units::metric_structural_unit_profile;
use crate::utils::{ensure_dir, iso_now, read_json, write_json};
use anyhow::{Context, Result, anyhow, bail};
use serde::Serialize;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

pub const PROJECT_FILE: &str = "fraia.project.json";
pub const DESIGN_FILE: &str = "fraia.design.json";
pub const DESIGN_STATE_FILE: &str = "design.json";
pub const PLANNING_FILE: &str = "planning.md";
pub const PROJECT_MANIFEST_SCHEMA_VERSION: &str = "fraia.project.v1";
pub const DESIGN_MANIFEST_SCHEMA_VERSION: &str = "fraia.design.v1";
pub const LEGACY_PROJECT_ARCHIVE: &str = "legacy/fraia.project.json";
const TRANSACTION_MARKER: &str = ".fraia-package-transaction";

static ID_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestValidationError {
    EmptyProjectName,
    EmptyDesignName { design_id: DesignId },
    DuplicateDesignName { name: String },
    DuplicateDesignId { design_id: DesignId },
    InvalidProjectId { project_id: ProjectId },
    InvalidDesignId { design_id: DesignId },
    UnsupportedProjectSchema { schema_version: String },
    UnsupportedDesignSchema { schema_version: String },
}

impl std::fmt::Display for ManifestValidationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyProjectName => formatter.write_str("project name must not be empty"),
            Self::EmptyDesignName { design_id } => {
                write!(formatter, "design `{design_id}` name must not be empty")
            }
            Self::DuplicateDesignName { name } => {
                write!(
                    formatter,
                    "design name `{name}` is already used in this project"
                )
            }
            Self::DuplicateDesignId { design_id } => {
                write!(
                    formatter,
                    "design id `{design_id}` is already used in this project"
                )
            }
            Self::InvalidProjectId { project_id } => {
                write!(
                    formatter,
                    "project id `{project_id}` is not a safe opaque id"
                )
            }
            Self::InvalidDesignId { design_id } => {
                write!(formatter, "design id `{design_id}` is not a safe opaque id")
            }
            Self::UnsupportedProjectSchema { schema_version } => {
                write!(
                    formatter,
                    "unsupported project manifest schema `{schema_version}`"
                )
            }
            Self::UnsupportedDesignSchema { schema_version } => {
                write!(
                    formatter,
                    "unsupported design manifest schema `{schema_version}`"
                )
            }
        }
    }
}

impl std::error::Error for ManifestValidationError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectPackagePaths {
    pub project_dir: PathBuf,
    pub project_manifest: PathBuf,
    pub planning_file: PathBuf,
    pub sources_dir: PathBuf,
    pub source_index: PathBuf,
    pub source_originals_dir: PathBuf,
    pub source_derived_dir: PathBuf,
    pub designs_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignPackagePaths {
    pub design_dir: PathBuf,
    pub design_manifest: PathBuf,
    pub project_state: PathBuf,
    pub planning_file: PathBuf,
    pub shelf_file: PathBuf,
    pub interpretations_dir: PathBuf,
    pub workspace_database: PathBuf,
    pub runs_dir: PathBuf,
    pub legacy_project_archive: PathBuf,
}

#[derive(Debug, Clone)]
pub struct DesignPackage {
    pub manifest: DesignManifest,
    pub project: ProjectFile,
    /// Deserialized compatibility view of the byte-for-byte legacy archive.
    /// New packages have no legacy project.
    pub legacy_project: Option<ProjectFile>,
}

#[derive(Debug, Clone)]
pub struct ProjectPackage {
    pub manifest: ProjectManifest,
    pub designs: Vec<DesignPackage>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegacyMigrationStage {
    Staged,
    OriginalBackedUp,
    PackageAdopted,
    PackageVerified,
}

pub fn project_package_paths(project_dir: &Path) -> ProjectPackagePaths {
    let sources_dir = project_dir.join("sources");
    ProjectPackagePaths {
        project_dir: project_dir.to_path_buf(),
        project_manifest: project_dir.join(PROJECT_FILE),
        planning_file: project_dir.join(PLANNING_FILE),
        source_index: sources_dir.join("source-index.json"),
        source_originals_dir: sources_dir.join("originals"),
        source_derived_dir: sources_dir.join("derived"),
        sources_dir,
        designs_dir: project_dir.join("designs"),
    }
}

pub fn design_package_paths(
    project_dir: &Path,
    design_id: &DesignId,
) -> Result<DesignPackagePaths, ManifestValidationError> {
    if !valid_opaque_id(design_id.as_str()) {
        return Err(ManifestValidationError::InvalidDesignId {
            design_id: design_id.clone(),
        });
    }
    let design_dir = project_dir.join("designs").join(design_id.as_str());
    Ok(DesignPackagePaths {
        design_manifest: design_dir.join(DESIGN_FILE),
        project_state: design_dir.join(DESIGN_STATE_FILE),
        planning_file: design_dir.join(PLANNING_FILE),
        shelf_file: design_dir.join("shelf.json"),
        interpretations_dir: design_dir.join("interpretations"),
        workspace_database: design_dir.join("workspace.sqlite"),
        runs_dir: design_dir.join("runs"),
        legacy_project_archive: design_dir.join(LEGACY_PROJECT_ARCHIVE),
        design_dir,
    })
}

pub fn new_blank_project_manifests() -> (ProjectManifest, DesignManifest) {
    let created_at = iso_now();
    let project_id = ProjectId::new(generate_opaque_id("project"));
    let design_id = DesignId::new(generate_opaque_id("design"));
    let design = DesignManifest {
        schema_version: DESIGN_MANIFEST_SCHEMA_VERSION.into(),
        id: design_id.clone(),
        name: "Design 1".into(),
        created_at: created_at.clone(),
        updated_at: None,
        files: DesignManifestFiles {
            state: DESIGN_STATE_FILE.into(),
            planning: PLANNING_FILE.into(),
            shelf: "shelf.json".into(),
            workspace: "workspace.sqlite".into(),
            runs: "runs".into(),
            legacy_project: None,
        },
        legacy_migration: None,
    };
    let project = ProjectManifest {
        schema_version: PROJECT_MANIFEST_SCHEMA_VERSION.into(),
        id: project_id,
        name: "Untitled Project".into(),
        created_at,
        updated_at: None,
        files: ProjectManifestFiles {
            planning: PLANNING_FILE.into(),
            sources: "sources".into(),
            designs: "designs".into(),
        },
        designs: vec![ProjectDesignEntry {
            id: design_id,
            name: "Design 1".into(),
        }],
    };
    (project, design)
}

/// Creates a new package through a sibling staging directory, then adopts the
/// complete directory with one rename. The destination must not contain user
/// files.
pub fn create_project_package(project_dir: &Path) -> Result<ProjectPackage> {
    create_named_project_package(project_dir, "Untitled Project")
}

pub fn create_named_project_package(
    project_dir: &Path,
    project_name: &str,
) -> Result<ProjectPackage> {
    if project_dir.exists() {
        let mut entries = fs::read_dir(project_dir)
            .with_context(|| format!("inspect project directory `{}`", project_dir.display()))?;
        if entries.next().transpose()?.is_some() {
            bail!(
                "refusing to create a Fraia package in non-empty directory `{}`",
                project_dir.display()
            );
        }
    }

    let (stage, backup) = sibling_transaction_paths(project_dir)?;
    remove_known_transaction_dir(&stage)?;
    if backup.exists() {
        bail!(
            "cannot create package while recovery backup `{}` exists",
            backup.display()
        );
    }
    initialize_transaction_dir(&stage)?;

    let (mut project, design) = new_blank_project_manifests();
    project.name = non_empty_migration_name(project_name, "Untitled Project");
    let project_state = new_project_file(&project.name);
    write_package_tree(
        &stage,
        &ProjectPackage {
            manifest: project,
            designs: vec![DesignPackage {
                manifest: design,
                project: project_state,
                legacy_project: None,
            }],
        },
        false,
    )?;
    load_project_package(&stage).context("verify staged blank project package")?;

    if project_dir.exists() {
        fs::remove_dir(project_dir)
            .with_context(|| format!("remove empty destination `{}`", project_dir.display()))?;
    }
    fs::rename(&stage, project_dir).with_context(|| {
        format!(
            "adopt staged package `{}` as `{}`",
            stage.display(),
            project_dir.display()
        )
    })?;
    remove_transaction_marker_if_present(project_dir)?;
    sync_parent(project_dir)?;
    load_project_package(project_dir)
}

pub fn load_project_package(project_dir: &Path) -> Result<ProjectPackage> {
    let paths = project_package_paths(project_dir);
    recover_owned_file(&paths.project_manifest)?;
    let manifest: ProjectManifest = read_json(&paths.project_manifest).with_context(|| {
        format!(
            "load project manifest `{}`",
            paths.project_manifest.display()
        )
    })?;
    manifest.validate()?;
    validate_project_file_contract(&manifest)?;

    let mut designs = Vec::with_capacity(manifest.designs.len());
    for entry in &manifest.designs {
        let design_paths = design_package_paths(project_dir, &entry.id)?;
        recover_owned_file(&design_paths.design_manifest)?;
        recover_owned_file(&design_paths.project_state)?;
        let design_manifest: DesignManifest = read_json(&design_paths.design_manifest)
            .with_context(|| {
                format!(
                    "load design manifest `{}`",
                    design_paths.design_manifest.display()
                )
            })?;
        design_manifest.validate()?;
        if design_manifest.id != entry.id || design_manifest.name != entry.name {
            bail!(
                "design index entry `{}` does not match its design manifest",
                entry.id
            );
        }
        validate_design_file_contract(&design_manifest)?;
        let project: ProjectFile = read_json(&design_paths.project_state).with_context(|| {
            format!(
                "load design state `{}`",
                design_paths.project_state.display()
            )
        })?;
        let legacy_project = match &design_manifest.files.legacy_project {
            Some(relative) => {
                let archive = safe_design_relative_path(&design_paths.design_dir, relative)?;
                let bytes = fs::read(&archive).with_context(|| {
                    format!("load archived legacy project `{}`", archive.display())
                })?;
                let migration = design_manifest
                    .legacy_migration
                    .as_ref()
                    .ok_or_else(|| anyhow!("legacy archive has no migration provenance"))?;
                if sha256_hex(&bytes) != migration.source_sha256 {
                    bail!(
                        "archived legacy project hash does not match design `{}` provenance",
                        design_manifest.id
                    );
                }
                let legacy: ProjectFile = serde_json::from_slice(&bytes).with_context(|| {
                    format!("decode archived legacy project `{}`", archive.display())
                })?;
                if legacy.schema_version != migration.source_schema_version {
                    bail!(
                        "archived legacy schema does not match design `{}` provenance",
                        design_manifest.id
                    );
                }
                Some(legacy)
            }
            None => None,
        };
        designs.push(DesignPackage {
            manifest: design_manifest,
            project,
            legacy_project,
        });
    }
    Ok(ProjectPackage { manifest, designs })
}

/// Saves package-owned manifests atomically while retaining all other files.
/// Each owned JSON file uses recoverable temporary and backup replacement.
/// The package directory itself is not swapped because a design-local SQLite
/// database can be open while app state is saved.
pub fn save_project_package(project_dir: &Path, package: &ProjectPackage) -> Result<()> {
    validate_package(package)?;
    write_package_tree(project_dir, package, true)?;
    load_project_package(project_dir)
        .context("verify saved project package")
        .map(|_| ())
}

/// Converts a legacy root `ProjectFile` into a one-design package. The input
/// directory is cloned and verified before adoption. The exact legacy JSON and
/// all unknown files remain in the adopted package.
pub fn migrate_legacy_project_package(project_dir: &Path) -> Result<ProjectPackage> {
    migrate_legacy_project_package_with_hook(project_dir, |_| Ok(()))
}

fn migrate_legacy_project_package_with_hook<F>(
    project_dir: &Path,
    mut hook: F,
) -> Result<ProjectPackage>
where
    F: FnMut(LegacyMigrationStage) -> Result<()>,
{
    let (stage, backup) = sibling_transaction_paths(project_dir)?;
    recover_interrupted_transaction(project_dir, &stage, &backup)?;

    if let Ok(package) = load_project_package(project_dir) {
        remove_known_transaction_dir(&stage)?;
        remove_known_transaction_dir(&backup)?;
        return Ok(package);
    }

    let legacy_paths = project_paths(project_dir);
    let raw_legacy = fs::read(&legacy_paths.project_file).with_context(|| {
        format!(
            "read legacy project file `{}`",
            legacy_paths.project_file.display()
        )
    })?;
    let legacy_project: ProjectFile = serde_json::from_slice(&raw_legacy)
        .context("root file is neither a project manifest nor a legacy ProjectFile")?;

    prepare_transaction_paths(project_dir, &stage, &backup)?;
    copy_tree(project_dir, &stage)?;
    let (mut project, mut design) = new_blank_project_manifests();
    project.name = non_empty_migration_name(&legacy_project.name, "Untitled Project");
    project.created_at = legacy_project.created_at.clone();
    project.designs[0].name = "Design 1".into();
    design.created_at = legacy_project.created_at.clone();
    design.files.legacy_project = Some(LEGACY_PROJECT_ARCHIVE.into());
    design.legacy_migration = Some(LegacyProjectMigration {
        source_schema_version: legacy_project.schema_version.clone(),
        archive: LEGACY_PROJECT_ARCHIVE.into(),
        source_sha256: sha256_hex(&raw_legacy),
        migrated_at: iso_now(),
    });
    let migrated_project = migrate_project_file(legacy_project.clone());

    let design_paths = design_package_paths(&stage, &design.id)?;
    ensure_dir(&design_paths.design_dir)?;
    ensure_dir(&design_paths.runs_dir)?;
    if legacy_paths.planning_file.exists() {
        fs::copy(stage.join(PLANNING_FILE), &design_paths.planning_file)
            .context("copy legacy planning into first design")?;
    } else {
        atomic_write_bytes(&design_paths.planning_file, b"")?;
    }
    let legacy_runs = stage.join("runs");
    if legacy_runs.is_dir() {
        copy_tree(&legacy_runs, &design_paths.runs_dir)
            .context("copy legacy runs into first design")?;
    }
    atomic_write_bytes(&design_paths.legacy_project_archive, &raw_legacy)?;
    atomic_write_json(&design_paths.project_state, &migrated_project)?;
    atomic_write_json(&design_paths.design_manifest, &design)?;
    atomic_write_json(&project_package_paths(&stage).project_manifest, &project)?;
    ensure_project_package_directories(&stage, &project, &[design.clone()])?;

    load_project_package(&stage).context("verify staged legacy migration")?;
    let archived = fs::read(&design_paths.legacy_project_archive)?;
    if archived != raw_legacy {
        bail!("staged migration changed the archived legacy project bytes");
    }
    hook(LegacyMigrationStage::Staged)?;
    adopt_staged_directory(project_dir, &stage, &backup, hook)?;
    load_project_package(project_dir)
}

fn non_empty_migration_name(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback.into()
    } else {
        trimmed.into()
    }
}

fn validate_package(package: &ProjectPackage) -> Result<()> {
    package.manifest.validate()?;
    validate_project_file_contract(&package.manifest)?;
    if package.manifest.designs.len() != package.designs.len() {
        bail!("project design index does not match supplied design packages");
    }
    for entry in &package.manifest.designs {
        let design = package
            .designs
            .iter()
            .find(|design| design.manifest.id == entry.id)
            .ok_or_else(|| anyhow!("missing design package `{}`", entry.id))?;
        design.manifest.validate()?;
        validate_design_file_contract(&design.manifest)?;
        if design.manifest.name != entry.name {
            bail!("design index name does not match design `{}`", entry.id);
        }
    }
    Ok(())
}

fn validate_project_file_contract(manifest: &ProjectManifest) -> Result<()> {
    if manifest.files.planning != PLANNING_FILE
        || manifest.files.sources != "sources"
        || manifest.files.designs != "designs"
    {
        bail!(
            "project `{}` uses unsupported package file paths",
            manifest.id
        );
    }
    Ok(())
}

fn validate_design_file_contract(manifest: &DesignManifest) -> Result<()> {
    if manifest.files.state != DESIGN_STATE_FILE
        || manifest.files.planning != PLANNING_FILE
        || manifest.files.shelf != "shelf.json"
        || manifest.files.workspace != "workspace.sqlite"
        || manifest.files.runs != "runs"
    {
        bail!(
            "design `{}` uses unsupported package file paths",
            manifest.id
        );
    }
    if let Some(relative) = &manifest.files.legacy_project
        && relative != LEGACY_PROJECT_ARCHIVE
    {
        bail!(
            "design `{}` uses unsupported legacy archive path",
            manifest.id
        );
    }
    match (&manifest.files.legacy_project, &manifest.legacy_migration) {
        (Some(archive), Some(migration)) if archive == &migration.archive => {}
        (None, None) => {}
        _ => bail!(
            "design `{}` has inconsistent legacy migration provenance",
            manifest.id
        ),
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(bytes))
}

fn safe_design_relative_path(design_dir: &Path, relative: &str) -> Result<PathBuf> {
    let relative = Path::new(relative);
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        bail!(
            "unsafe design-relative package path `{}`",
            relative.display()
        );
    }
    Ok(design_dir.join(relative))
}

fn ensure_project_package_directories(
    project_dir: &Path,
    project: &ProjectManifest,
    designs: &[DesignManifest],
) -> Result<()> {
    let paths = project_package_paths(project_dir);
    ensure_dir(&paths.source_originals_dir)?;
    ensure_dir(&paths.source_derived_dir)?;
    ensure_dir(&paths.designs_dir)?;
    if !paths.planning_file.exists() {
        atomic_write_bytes(&paths.planning_file, b"")?;
    }
    if !paths.source_index.exists() {
        atomic_write_json(
            &paths.source_index,
            &serde_json::json!({ "schema_version": "fraia.sources.v1", "items": [] }),
        )?;
    }
    for design in designs {
        let design_paths = design_package_paths(project_dir, &design.id)?;
        ensure_dir(&design_paths.design_dir)?;
        ensure_dir(&design_paths.runs_dir)?;
        if !design_paths.planning_file.exists() {
            atomic_write_bytes(&design_paths.planning_file, b"")?;
        }
        if !design_paths.shelf_file.exists() {
            atomic_write_json(
                &design_paths.shelf_file,
                &serde_json::json!({ "schema_version": "fraia.shelf.v1", "items": [] }),
            )?;
        }
    }
    if project.designs.len() != designs.len() {
        bail!("cannot prepare package directories for an incomplete design set");
    }
    Ok(())
}

fn write_package_tree(
    project_dir: &Path,
    package: &ProjectPackage,
    preserve_legacy_archives: bool,
) -> Result<()> {
    validate_package(package)?;
    let manifests = package
        .designs
        .iter()
        .map(|design| design.manifest.clone())
        .collect::<Vec<_>>();
    ensure_project_package_directories(project_dir, &package.manifest, &manifests)?;
    for design in &package.designs {
        let paths = design_package_paths(project_dir, &design.manifest.id)?;
        atomic_write_json(&paths.design_manifest, &design.manifest)?;
        atomic_write_json(&paths.project_state, &design.project)?;
        if !preserve_legacy_archives {
            if let Some(legacy) = &design.legacy_project {
                atomic_write_json(&paths.legacy_project_archive, legacy)?;
            }
        } else if design.manifest.files.legacy_project.is_some()
            && !paths.legacy_project_archive.is_file()
        {
            bail!(
                "cannot save migrated design `{}` without its legacy archive",
                design.manifest.id
            );
        }
    }
    atomic_write_json(
        &project_package_paths(project_dir).project_manifest,
        &package.manifest,
    )?;
    Ok(())
}

fn sibling_transaction_paths(project_dir: &Path) -> Result<(PathBuf, PathBuf)> {
    let parent = project_dir.parent().ok_or_else(|| {
        anyhow!(
            "project directory `{}` has no parent for atomic staging",
            project_dir.display()
        )
    })?;
    let name = project_dir
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow!("project directory name is not valid UTF-8"))?;
    Ok((
        parent.join(format!(".{name}.fraia-stage")),
        parent.join(format!(".{name}.fraia-backup")),
    ))
}

fn prepare_transaction_paths(project_dir: &Path, stage: &Path, backup: &Path) -> Result<()> {
    if !project_dir.is_dir() {
        bail!(
            "project directory `{}` does not exist",
            project_dir.display()
        );
    }
    remove_known_transaction_dir(stage)?;
    if backup.exists() {
        bail!(
            "recovery backup `{}` exists; recover it before starting another transaction",
            backup.display()
        );
    }
    initialize_transaction_dir(stage)
}

fn recover_interrupted_transaction(project_dir: &Path, stage: &Path, backup: &Path) -> Result<()> {
    if !project_dir.exists() && backup.is_dir() {
        fs::rename(backup, project_dir).with_context(|| {
            format!(
                "restore interrupted migration backup `{}`",
                backup.display()
            )
        })?;
        remove_transaction_marker_if_present(project_dir)?;
        sync_parent(project_dir)?;
    }
    if project_dir.exists() && backup.exists() {
        if load_project_package(project_dir).is_ok() {
            remove_known_transaction_dir(backup)?;
        } else {
            bail!(
                "both project `{}` and recovery backup `{}` exist, but the project is not a valid package",
                project_dir.display(),
                backup.display()
            );
        }
    }
    remove_known_transaction_dir(stage)
}

fn adopt_staged_directory<F>(
    project_dir: &Path,
    stage: &Path,
    backup: &Path,
    mut hook: F,
) -> Result<()>
where
    F: FnMut(LegacyMigrationStage) -> Result<()>,
{
    fs::rename(project_dir, backup).with_context(|| {
        format!(
            "back up project `{}` as `{}`",
            project_dir.display(),
            backup.display()
        )
    })?;
    if let Err(error) = mark_transaction_dir(backup) {
        fs::rename(backup, project_dir)
            .context("restore project after transaction-marker failure")?;
        let _ = sync_parent(project_dir);
        return Err(error);
    }
    if let Err(error) = sync_parent(project_dir) {
        fs::rename(backup, project_dir).context("restore project after backup-sync failure")?;
        let _ = remove_transaction_marker_if_present(project_dir);
        let _ = sync_parent(project_dir);
        return Err(error);
    }
    if let Err(error) = hook(LegacyMigrationStage::OriginalBackedUp) {
        fs::rename(backup, project_dir).context("restore project after injected failure")?;
        remove_transaction_marker_if_present(project_dir)?;
        sync_parent(project_dir)?;
        return Err(error);
    }

    if let Err(error) = fs::rename(stage, project_dir) {
        fs::rename(backup, project_dir).context("restore project after adoption failure")?;
        remove_transaction_marker_if_present(project_dir)?;
        sync_parent(project_dir)?;
        return Err(error).context("adopt staged project package");
    }
    if let Err(error) = remove_transaction_marker_if_present(project_dir) {
        rollback_adoption(project_dir, stage, backup)?;
        return Err(error);
    }
    sync_parent(project_dir)?;
    if let Err(error) = hook(LegacyMigrationStage::PackageAdopted) {
        rollback_adoption(project_dir, stage, backup)?;
        return Err(error);
    }
    if let Err(error) = load_project_package(project_dir).context("verify adopted project package")
    {
        rollback_adoption(project_dir, stage, backup)?;
        return Err(error);
    }
    if let Err(error) = hook(LegacyMigrationStage::PackageVerified) {
        rollback_adoption(project_dir, stage, backup)?;
        return Err(error);
    }
    remove_known_transaction_dir(backup)?;
    sync_parent(project_dir)
}

fn rollback_adoption(project_dir: &Path, stage: &Path, backup: &Path) -> Result<()> {
    if stage.exists() {
        remove_known_transaction_dir(stage)?;
    }
    fs::rename(project_dir, stage).context("retain failed adopted package as staging data")?;
    let stage_marker_result = mark_transaction_dir(stage);
    fs::rename(backup, project_dir).context("restore original project after failed adoption")?;
    remove_transaction_marker_if_present(project_dir)?;
    sync_parent(project_dir)?;
    stage_marker_result
}

fn remove_known_transaction_dir(path: &Path) -> Result<()> {
    if path.exists() {
        if !path.join(TRANSACTION_MARKER).is_file() {
            bail!(
                "refusing to remove unmarked transaction directory `{}`",
                path.display()
            );
        }
        fs::remove_dir_all(path)
            .with_context(|| format!("remove transaction directory `{}`", path.display()))?;
    }
    Ok(())
}

fn initialize_transaction_dir(path: &Path) -> Result<()> {
    ensure_dir(path)?;
    mark_transaction_dir(path)
}

fn mark_transaction_dir(path: &Path) -> Result<()> {
    atomic_write_bytes(
        &path.join(TRANSACTION_MARKER),
        b"Fraia package transaction\n",
    )
}

fn remove_transaction_marker_if_present(path: &Path) -> Result<()> {
    let marker = path.join(TRANSACTION_MARKER);
    if marker.exists() {
        fs::remove_file(&marker)
            .with_context(|| format!("remove transaction marker `{}`", marker.display()))?;
        sync_directory(path)?;
    }
    Ok(())
}

fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    ensure_dir(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_tree(&source_path, &destination_path)?;
        } else if file_type.is_file() {
            fs::copy(&source_path, &destination_path)
                .with_context(|| format!("copy package file `{}`", source_path.display()))?;
        } else if file_type.is_symlink() {
            copy_symlink(&source_path, &destination_path)?;
        } else {
            bail!("unsupported package entry `{}`", source_path.display());
        }
    }
    Ok(())
}

#[cfg(unix)]
fn copy_symlink(source: &Path, destination: &Path) -> Result<()> {
    let target = fs::read_link(source)?;
    std::os::unix::fs::symlink(target, destination)?;
    Ok(())
}

#[cfg(windows)]
fn copy_symlink(source: &Path, destination: &Path) -> Result<()> {
    let target = fs::read_link(source)?;
    if source.is_dir() {
        std::os::windows::fs::symlink_dir(target, destination)?;
    } else {
        std::os::windows::fs::symlink_file(target, destination)?;
    }
    Ok(())
}

fn atomic_write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(value)?;
    let mut terminated = bytes;
    terminated.push(b'\n');
    atomic_write_bytes(path, &terminated)
}

fn atomic_write_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("file `{}` has no parent", path.display()))?;
    ensure_dir(parent)?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow!("file name is not valid UTF-8"))?;
    let temporary = parent.join(format!(".{file_name}.fraia-tmp"));
    let backup = parent.join(format!(".{file_name}.fraia-bak"));
    recover_owned_file(path)?;
    if backup.exists() {
        bail!(
            "recovery backup `{}` must be resolved before writing `{}`",
            backup.display(),
            path.display()
        );
    }
    let mut file = File::create(&temporary)
        .with_context(|| format!("create temporary file `{}`", temporary.display()))?;
    file.write_all(bytes)?;
    file.sync_all()?;
    let had_original = path.exists();
    if had_original {
        fs::rename(path, &backup).with_context(|| {
            format!(
                "back up package file `{}` as `{}`",
                path.display(),
                backup.display()
            )
        })?;
    }
    if let Err(error) = fs::rename(&temporary, path) {
        if had_original {
            fs::rename(&backup, path).with_context(|| {
                format!(
                    "restore package file `{}` after write failure",
                    path.display()
                )
            })?;
        }
        return Err(error)
            .with_context(|| format!("replace `{}` with synced temporary file", path.display()));
    }
    sync_directory(parent)?;
    if had_original {
        fs::remove_file(&backup)
            .with_context(|| format!("remove package-file backup `{}`", backup.display()))?;
        sync_directory(parent)?;
    }
    Ok(())
}

fn recover_owned_file(path: &Path) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("file `{}` has no parent", path.display()))?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow!("file name is not valid UTF-8"))?;
    let backup = parent.join(format!(".{file_name}.fraia-bak"));
    if !backup.exists() {
        return Ok(());
    }
    if path.exists() {
        fs::remove_file(&backup).with_context(|| {
            format!(
                "remove completed package-file backup `{}`",
                backup.display()
            )
        })?;
    } else {
        fs::rename(&backup, path).with_context(|| {
            format!(
                "restore interrupted package-file backup `{}`",
                backup.display()
            )
        })?;
    }
    sync_directory(parent)
}

fn sync_parent(path: &Path) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("path `{}` has no parent", path.display()))?;
    sync_directory(parent)
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<()> {
    File::open(path)?.sync_all()?;
    Ok(())
}

#[cfg(windows)]
fn sync_directory(_path: &Path) -> Result<()> {
    // Windows rename durability is provided by the synced file handle. Rust's
    // standard library does not expose a portable directory sync operation.
    Ok(())
}

impl ProjectManifest {
    pub fn validate(&self) -> Result<(), ManifestValidationError> {
        if self.schema_version != PROJECT_MANIFEST_SCHEMA_VERSION {
            return Err(ManifestValidationError::UnsupportedProjectSchema {
                schema_version: self.schema_version.clone(),
            });
        }
        if !valid_opaque_id(self.id.as_str()) {
            return Err(ManifestValidationError::InvalidProjectId {
                project_id: self.id.clone(),
            });
        }
        if self.name.trim().is_empty() {
            return Err(ManifestValidationError::EmptyProjectName);
        }
        let mut ids = std::collections::BTreeSet::new();
        let mut names = std::collections::BTreeSet::new();
        for design in &self.designs {
            if !valid_opaque_id(design.id.as_str()) {
                return Err(ManifestValidationError::InvalidDesignId {
                    design_id: design.id.clone(),
                });
            }
            if design.name.trim().is_empty() {
                return Err(ManifestValidationError::EmptyDesignName {
                    design_id: design.id.clone(),
                });
            }
            if !ids.insert(design.id.clone()) {
                return Err(ManifestValidationError::DuplicateDesignId {
                    design_id: design.id.clone(),
                });
            }
            let normalized_name = design.name.trim().to_lowercase();
            if !names.insert(normalized_name) {
                return Err(ManifestValidationError::DuplicateDesignName {
                    name: design.name.clone(),
                });
            }
        }
        Ok(())
    }
}

impl DesignManifest {
    pub fn validate(&self) -> Result<(), ManifestValidationError> {
        if self.schema_version != DESIGN_MANIFEST_SCHEMA_VERSION {
            return Err(ManifestValidationError::UnsupportedDesignSchema {
                schema_version: self.schema_version.clone(),
            });
        }
        if !valid_opaque_id(self.id.as_str()) {
            return Err(ManifestValidationError::InvalidDesignId {
                design_id: self.id.clone(),
            });
        }
        if self.name.trim().is_empty() {
            return Err(ManifestValidationError::EmptyDesignName {
                design_id: self.id.clone(),
            });
        }
        Ok(())
    }
}

fn generate_opaque_id(kind: &str) -> String {
    let sequence = ID_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let material = format!(
        "{kind}:{}:{}:{sequence}",
        std::process::id(),
        crate::utils::timestamp_id()
    );
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(material.as_bytes());
    let encoded = format!("{digest:x}");
    format!("{kind}-{}", &encoded[..32])
}

fn valid_opaque_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

#[derive(Debug, Clone)]
pub struct ProjectPaths {
    pub project_dir: PathBuf,
    pub project_file: PathBuf,
    pub planning_file: PathBuf,
    pub generated_dir: PathBuf,
    pub runs_dir: PathBuf,
}

pub fn project_paths(project_dir: &Path) -> ProjectPaths {
    ProjectPaths {
        project_dir: project_dir.to_path_buf(),
        project_file: project_dir.join(PROJECT_FILE),
        planning_file: project_dir.join(PLANNING_FILE),
        generated_dir: project_dir.join("generated"),
        runs_dir: project_dir.join("runs"),
    }
}

pub fn create_project(project_dir: &Path, name: &str) -> Result<(ProjectFile, ProjectPaths)> {
    let paths = project_paths(project_dir);
    ensure_dir(&paths.project_dir)?;
    ensure_dir(&paths.generated_dir)?;
    ensure_dir(&paths.runs_dir)?;

    let project = new_project_file(name);

    write_json(&paths.project_file, &project)?;
    fs::write(&paths.planning_file, default_planning_markdown(&project))?;
    Ok((project, paths))
}

fn new_project_file(name: &str) -> ProjectFile {
    let mut project = ProjectFile {
        schema_version: "0.2.0".into(),
        name: name.into(),
        created_at: iso_now(),
        updated_at: None,
        intent: Intent {
            building_type: "warehouse".into(),
            design_stage: "concept".into(),
            objective_priority: "balanced".into(),
            option_count: 5,
            hard_constraints: vec![],
            soft_preferences: vec!["balanced tradeoff exploration".into()],
            search_permissions: SearchPermissions {
                resize_sections: true,
                add_internal_columns: true,
                change_topology: true,
            },
            approval_triggers: vec![
                "change material system".into(),
                "change overall building envelope".into(),
            ],
        },
        requirements: Requirements {
            span_m: 20.0,
            height_m: 6.0,
            gravity_load_kn_per_m: 20.0,
            lateral_load_kn: 80.0,
            max_deflection_ratio: 250.0,
            max_drift_ratio: 300.0,
            max_utilization: 0.67,
            max_internal_columns: 2,
        },
        unit_profile: metric_structural_unit_profile(),
        planning_draft: None,
        files: ProjectFiles {
            planning: PLANNING_FILE.into(),
        },
        builder_graph: None,
        legacy_builder_instance: None,
        agent_state: AgentState::default(),
        base_model_brief: None,
        structural_model: Some(StructuralModel::empty()),
        design_option_decisions: Default::default(),
    };
    project.planning_draft = Some(default_planning_draft(&project));
    project
}

fn migrate_project_file(mut project: ProjectFile) -> ProjectFile {
    project.schema_version = "0.2.0".into();
    if project.builder_graph.is_none()
        && let Some(legacy) = project.legacy_builder_instance.take()
    {
        project.builder_graph = Some(builder_graph_from_legacy_builder(&legacy));
    }
    if project.planning_draft.is_none() {
        project.planning_draft = Some(default_planning_draft(&project));
    }
    project
}

pub fn load_project(project_dir: &Path) -> Result<(ProjectFile, ProjectPaths)> {
    let paths = project_paths(project_dir);
    let project = migrate_project_file(read_json::<ProjectFile>(&paths.project_file)?);
    Ok((project, paths))
}

pub fn save_project(project_dir: &Path, project: &ProjectFile) -> Result<()> {
    let paths = project_paths(project_dir);
    let mut migrated = project.clone();
    migrated.schema_version = "0.2.0".into();
    write_json(&paths.project_file, &migrated)
}

pub fn materialize_project_structural_model(project: &ProjectFile) -> Option<StructuralModel> {
    if let Some(structural) = project
        .structural_model
        .as_ref()
        .filter(|model| !model.is_empty())
    {
        return Some(structural.clone());
    }
    let graph = if let Some(graph) = &project.builder_graph {
        Some(graph.clone())
    } else {
        project
            .legacy_builder_instance
            .as_ref()
            .map(builder_graph_from_legacy_builder)
    }?;
    materialize_structural_model_from_builder_graph(&graph)
}

pub fn update_planning_markdown(project_dir: &Path, markdown: &str) -> Result<()> {
    let paths = project_paths(project_dir);
    fs::write(paths.planning_file, markdown)?;
    Ok(())
}

pub fn planning_draft(project: &ProjectFile) -> PlanningDraft {
    project
        .planning_draft
        .clone()
        .unwrap_or_else(|| default_planning_draft(project))
}

pub fn default_planning_draft(project: &ProjectFile) -> PlanningDraft {
    let family_hint = infer_system_family_hint(project);
    let form_hint = match family_hint.as_str() {
        "beam.simply_supported" => "simply supported beam",
        "portal_frame" => "clear-span portal frame",
        _ => "concept system",
    };

    PlanningDraft {
        project_intent: PlanningProjectIntent {
            name: project.name.clone(),
            building_type: project.intent.building_type.clone(),
            design_stage: project.intent.design_stage.clone(),
            objective_priority: project.intent.objective_priority.clone(),
        },
        system_brief: PlanningSystemBrief {
            system_family_hint: family_hint,
            structural_form_hint: form_hint.into(),
            notes: String::new(),
        },
        geometry_and_loads: PlanningGeometryAndLoads {
            span_m: project.requirements.span_m,
            height_m: project.requirements.height_m,
            gravity_line_load_kn_per_m: project.requirements.gravity_load_kn_per_m,
            lateral_load_kn: project.requirements.lateral_load_kn,
        },
        design_constraints: PlanningDesignConstraints {
            max_deflection_ratio: project.requirements.max_deflection_ratio,
            max_drift_ratio: project.requirements.max_drift_ratio,
            max_utilization: project.requirements.max_utilization,
            allow_internal_columns: project.requirements.max_internal_columns > 0,
            max_internal_columns: project.requirements.max_internal_columns,
        },
        analysis_brief: PlanningAnalysisBrief {
            requested_analysis_intent: "size-and-check".into(),
            preferred_backend: None,
            summary_goals: "Establish a conservative concept model, run the supported analysis path, and report governing values first.".into(),
        },
        system_parameters: Default::default(),
    }
}

pub fn apply_planning_draft(project: &mut ProjectFile, draft: PlanningDraft) {
    project.name = draft.project_intent.name.clone();
    project.intent.building_type = draft.project_intent.building_type.clone();
    project.intent.design_stage = draft.project_intent.design_stage.clone();
    project.intent.objective_priority = draft.project_intent.objective_priority.clone();
    project.requirements.span_m = draft.geometry_and_loads.span_m;
    project.requirements.height_m = draft.geometry_and_loads.height_m;
    project.requirements.gravity_load_kn_per_m =
        draft.geometry_and_loads.gravity_line_load_kn_per_m;
    project.requirements.lateral_load_kn = draft.geometry_and_loads.lateral_load_kn;
    project.requirements.max_deflection_ratio = draft.design_constraints.max_deflection_ratio;
    project.requirements.max_drift_ratio = draft.design_constraints.max_drift_ratio;
    project.requirements.max_utilization = draft.design_constraints.max_utilization;
    project.requirements.max_internal_columns = if draft.design_constraints.allow_internal_columns {
        draft.design_constraints.max_internal_columns
    } else {
        0
    };
    project.planning_draft = Some(draft);
    project.updated_at = Some(iso_now());
}

pub fn default_planning_markdown(project: &ProjectFile) -> String {
    let draft = planning_draft(project);
    format!(
        "# Fraia Planning\n\n## Project summary\n- Name: {}\n- Building type: {}\n- Design stage: {}\n- Objective priority: {}\n- System family hint: {}\n- Structural form hint: {}\n\n## Requirements\n- Span: {} m\n- Height: {} m\n- Gravity line load: {} kN/m\n- Lateral load: {} kN\n- Max deflection ratio: L/{}\n- Max drift ratio: H/{}\n- Max utilisation: {}\n- Internal columns allowed: {}\n- Max internal columns: {}\n\n## Analysis brief\n- Requested intent: {}\n- Preferred backend: {}\n- Summary goals: {}\n\n## System notes\n{}\n\n## Hard constraints\n{}\n\n## Soft preferences\n{}\n\n## Search permissions\n- Resize sections: {}\n- Add internal columns: {}\n- Change topology: {}\n\n## Approval triggers\n{}\n\n## Open questions\n- Add site/wind/seismic context\n- Add material/system alternatives beyond the MVP frame demo\n- Add code/jurisdiction if required\n\n## Next Fraia actions\n- Use `fraia optimize <projectDir>` to generate concept options\n- Review candidate tradeoffs before selecting a preferred direction\n",
        draft.project_intent.name,
        draft.project_intent.building_type,
        draft.project_intent.design_stage,
        draft.project_intent.objective_priority,
        draft.system_brief.system_family_hint,
        draft.system_brief.structural_form_hint,
        draft.geometry_and_loads.span_m,
        draft.geometry_and_loads.height_m,
        draft.geometry_and_loads.gravity_line_load_kn_per_m,
        draft.geometry_and_loads.lateral_load_kn,
        draft.design_constraints.max_deflection_ratio,
        draft.design_constraints.max_drift_ratio,
        draft.design_constraints.max_utilization,
        yes_no(draft.design_constraints.allow_internal_columns),
        draft.design_constraints.max_internal_columns,
        draft.analysis_brief.requested_analysis_intent,
        draft
            .analysis_brief
            .preferred_backend
            .as_deref()
            .unwrap_or("auto"),
        draft.analysis_brief.summary_goals,
        if draft.system_brief.notes.trim().is_empty() {
            "- None recorded yet".into()
        } else {
            format!("- {}", draft.system_brief.notes.trim())
        },
        render_bullets(&project.intent.hard_constraints),
        render_bullets(&project.intent.soft_preferences),
        yes_no(project.intent.search_permissions.resize_sections),
        yes_no(project.intent.search_permissions.add_internal_columns),
        yes_no(project.intent.search_permissions.change_topology),
        render_bullets(&project.intent.approval_triggers),
    )
}

fn infer_system_family_hint(project: &ProjectFile) -> String {
    if let Some(graph) = &project.builder_graph {
        for node in &graph.nodes {
            match node.parameters {
                BuilderNodeParameters::SimplySupportedBeam2D(_) => {
                    return "beam.simply_supported".into();
                }
                BuilderNodeParameters::PortalFrame2D(_) => return "portal_frame".into(),
                BuilderNodeParameters::ConceptRoot => {}
            }
        }
    }

    match project.intent.building_type.as_str() {
        "beam" | "beam.simply_supported" => "beam.simply_supported".into(),
        "portal_frame" | "frame.portal_2d" => "portal_frame".into(),
        other => other.to_owned(),
    }
}

fn render_bullets(items: &[String]) -> String {
    if items.is_empty() {
        "- None recorded yet".into()
    } else {
        items
            .iter()
            .map(|item| format!("- {item}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

#[cfg(test)]
mod tests {
    use super::{
        ID_SEQUENCE, LegacyMigrationStage, ManifestValidationError, apply_planning_draft,
        create_project, create_project_package, design_package_paths, load_project,
        load_project_package, materialize_project_structural_model, migrate_legacy_project_package,
        migrate_legacy_project_package_with_hook, new_blank_project_manifests, planning_draft,
        project_package_paths, project_paths, save_project, save_project_package,
        sibling_transaction_paths,
    };
    use crate::archetypes::portal_frame_builder_graph;
    use crate::structural_app::{
        AssignmentTargetRef, LoadAssignment, LoadKind, LoadVector, MemberEnd, MemberEndTarget,
        ReleaseAssignment, StructuralMember, StructuralNode,
    };
    use crate::types::{DesignId, DesignManifest, ProjectDesignEntry, ProjectManifest};
    use crate::utils::timestamp_id;
    use serde_json::json;
    use std::fs;
    use std::path::Path;
    use std::sync::atomic::Ordering;

    #[test]
    fn blank_manifests_have_stable_identity_and_typed_paths() {
        let (mut project, mut design) = new_blank_project_manifests();
        project.validate().expect("valid project manifest");
        design.validate().expect("valid design manifest");

        assert_eq!(project.name, "Untitled Project");
        assert_eq!(design.name, "Design 1");
        assert_eq!(project.designs[0].id, design.id);

        let project_id = project.id.clone();
        let design_id = design.id.clone();
        let before = design_package_paths(Path::new("/tmp/fraia-package"), &design.id)
            .expect("valid design paths");
        project.name = "House Structure".into();
        project.designs[0].name = "Complete Steelwork".into();
        design.name = "Complete Steelwork".into();
        let after = design_package_paths(Path::new("/tmp/fraia-package"), &design.id)
            .expect("valid design paths");

        assert_eq!(project.id, project_id);
        assert_eq!(design.id, design_id);
        assert_eq!(before, after);
        assert_eq!(
            before.design_manifest,
            before.design_dir.join("fraia.design.json")
        );
        assert_eq!(before.shelf_file, before.design_dir.join("shelf.json"));
        assert_eq!(
            before.workspace_database,
            before.design_dir.join("workspace.sqlite")
        );
        assert_eq!(before.runs_dir, before.design_dir.join("runs"));

        let paths = project_package_paths(Path::new("/tmp/fraia-package"));
        assert_eq!(
            paths.project_manifest,
            paths.project_dir.join("fraia.project.json")
        );
        assert_eq!(
            paths.source_index,
            paths.sources_dir.join("source-index.json")
        );
        assert_eq!(paths.designs_dir, paths.project_dir.join("designs"));
    }

    #[test]
    fn two_independent_designs_round_trip_with_versioned_manifests() {
        let (mut project, first_design) = new_blank_project_manifests();
        let second_design = DesignManifest {
            id: DesignId::new("design-second"),
            name: "Canopy".into(),
            ..first_design.clone()
        };
        project.designs.push(ProjectDesignEntry {
            id: second_design.id.clone(),
            name: second_design.name.clone(),
        });

        project.validate().expect("valid project manifest");
        first_design.validate().expect("valid first design");
        second_design.validate().expect("valid second design");

        let project_json = serde_json::to_string(&project).expect("serialize project manifest");
        let first_json = serde_json::to_string(&first_design).expect("serialize first design");
        let second_json = serde_json::to_string(&second_design).expect("serialize second design");
        let loaded_project: ProjectManifest =
            serde_json::from_str(&project_json).expect("deserialize project manifest");
        let loaded_first: DesignManifest =
            serde_json::from_str(&first_json).expect("deserialize first design");
        let loaded_second: DesignManifest =
            serde_json::from_str(&second_json).expect("deserialize second design");

        assert_eq!(loaded_project, project);
        assert_eq!(loaded_first, first_design);
        assert_eq!(loaded_second, second_design);
        assert_ne!(loaded_first.id, loaded_second.id);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&project_json).expect("project JSON")["schema_version"],
            "fraia.project.v1"
        );
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&first_json).expect("design JSON")["schema_version"],
            "fraia.design.v1"
        );
    }

    #[test]
    fn manifest_validation_returns_typed_name_and_identity_diagnostics() {
        let (mut project, mut design) = new_blank_project_manifests();
        project.name = " ".into();
        assert_eq!(
            project.validate(),
            Err(ManifestValidationError::EmptyProjectName)
        );

        let (mut project, _) = new_blank_project_manifests();
        project.designs.push(ProjectDesignEntry {
            id: DesignId::new("design-second"),
            name: " design 1 ".into(),
        });
        assert!(matches!(
            project.validate(),
            Err(ManifestValidationError::DuplicateDesignName { .. })
        ));

        design.name.clear();
        assert!(matches!(
            design.validate(),
            Err(ManifestValidationError::EmptyDesignName { .. })
        ));

        let unsafe_id = DesignId::new("../outside");
        assert!(matches!(
            design_package_paths(Path::new("/tmp/fraia-package"), &unsafe_id),
            Err(ManifestValidationError::InvalidDesignId { .. })
        ));
    }

    #[test]
    fn package_create_load_and_save_preserve_unknown_files_and_stable_paths() {
        let temp_dir =
            std::env::temp_dir().join(format!("fraia-package-create-{}", timestamp_id()));
        let mut package = create_project_package(&temp_dir).expect("create project package");
        let first_id = package.designs[0].manifest.id.clone();
        let paths_before = design_package_paths(&temp_dir, &first_id).expect("design paths");
        fs::write(temp_dir.join("unknown-client-data.bin"), b"do not rewrite")
            .expect("write unknown file");

        package.manifest.name = "House Structure".into();
        package.manifest.designs[0].name = "Complete Steelwork".into();
        package.designs[0].manifest.name = "Complete Steelwork".into();
        save_project_package(&temp_dir, &package).expect("save project package");

        let loaded = load_project_package(&temp_dir).expect("reload project package");
        let paths_after =
            design_package_paths(&temp_dir, &loaded.designs[0].manifest.id).expect("design paths");
        assert_eq!(loaded.manifest.id, package.manifest.id);
        assert_eq!(loaded.designs[0].manifest.id, first_id);
        assert_eq!(paths_before, paths_after);
        assert_eq!(
            fs::read(temp_dir.join("unknown-client-data.bin")).expect("read unknown file"),
            b"do not rewrite"
        );
        assert!(project_package_paths(&temp_dir).source_index.is_file());
        assert!(paths_after.shelf_file.is_file());
        assert!(paths_after.runs_dir.is_dir());

        let state_name = paths_after
            .project_state
            .file_name()
            .and_then(|name| name.to_str())
            .expect("state file name");
        let interrupted_backup = paths_after
            .design_dir
            .join(format!(".{state_name}.fraia-bak"));
        fs::rename(&paths_after.project_state, &interrupted_backup)
            .expect("simulate interrupted state replacement");
        let recovered = load_project_package(&temp_dir).expect("recover state-file backup");
        assert_eq!(recovered.designs[0].project.requirements.span_m, 20.0);
        assert!(paths_after.project_state.is_file());
        assert!(!interrupted_backup.exists());

        cleanup_package_test(&temp_dir);
    }

    #[test]
    fn legacy_fixture_variants_migrate_to_one_design_without_changing_input_bytes() {
        for kind in ["empty", "authored", "builder", "conversation", "runs"] {
            let (temp_dir, raw_legacy) = legacy_fixture(kind);
            let unknown = temp_dir.join("consultant-notes.dat");
            fs::write(&unknown, format!("unknown-{kind}")).expect("write unknown fixture");
            fs::create_dir_all(temp_dir.join(".fraia")).expect("create legacy metadata dir");
            fs::write(
                temp_dir.join(".fraia").join("workspace.sqlite"),
                format!("workspace-{kind}"),
            )
            .expect("write legacy workspace fixture");

            let first = migrate_legacy_project_package(&temp_dir).expect("migrate legacy fixture");
            assert_eq!(first.designs.len(), 1);
            assert_eq!(first.manifest.designs[0].id, first.designs[0].manifest.id);
            let design_paths =
                design_package_paths(&temp_dir, &first.designs[0].manifest.id).expect("paths");
            assert_eq!(
                fs::read(&design_paths.legacy_project_archive).expect("legacy archive"),
                raw_legacy,
                "legacy bytes changed for fixture {kind}"
            );
            assert!(!design_paths.workspace_database.exists());
            assert_eq!(
                fs::read(temp_dir.join(".fraia/workspace.sqlite"))
                    .expect("preserved legacy workspace"),
                format!("workspace-{kind}").as_bytes()
            );
            assert_eq!(
                fs::read_to_string(&unknown).expect("unknown file"),
                format!("unknown-{kind}")
            );
            let legacy = first.designs[0]
                .legacy_project
                .as_ref()
                .expect("legacy compatibility view");
            match kind {
                "authored" => assert_eq!(
                    legacy
                        .structural_model
                        .as_ref()
                        .expect("authored model")
                        .nodes
                        .len(),
                    1
                ),
                "builder" => assert!(legacy.builder_graph.is_some()),
                "conversation" => assert_eq!(legacy.agent_state.sessions.len(), 1),
                "runs" => {
                    assert!(temp_dir.join("runs/run-fixture/results.json").is_file());
                    assert!(
                        design_paths
                            .runs_dir
                            .join("run-fixture/results.json")
                            .is_file()
                    );
                }
                _ => {}
            }
            let provenance = first.designs[0]
                .manifest
                .legacy_migration
                .as_ref()
                .expect("migration provenance");
            assert_eq!(provenance.archive, super::LEGACY_PROJECT_ARCHIVE);
            assert_eq!(provenance.source_sha256, super::sha256_hex(&raw_legacy));

            let second =
                migrate_legacy_project_package(&temp_dir).expect("idempotent migration reopen");
            assert_eq!(second.manifest.id, first.manifest.id);
            assert_eq!(second.designs[0].manifest.id, first.designs[0].manifest.id);
            cleanup_package_test(&temp_dir);
        }
    }

    #[test]
    fn failure_at_each_migration_stage_restores_an_openable_legacy_project() {
        for failure_stage in [
            LegacyMigrationStage::Staged,
            LegacyMigrationStage::OriginalBackedUp,
            LegacyMigrationStage::PackageAdopted,
            LegacyMigrationStage::PackageVerified,
        ] {
            let (temp_dir, raw_legacy) = legacy_fixture("authored");
            let result = migrate_legacy_project_package_with_hook(&temp_dir, |stage| {
                if stage == failure_stage {
                    Err(anyhow::anyhow!("injected failure at {stage:?}"))
                } else {
                    Ok(())
                }
            });
            assert!(
                result.is_err(),
                "failure was not injected at {failure_stage:?}"
            );
            assert_eq!(
                fs::read(project_paths(&temp_dir).project_file).expect("restored legacy file"),
                raw_legacy
            );
            load_project(&temp_dir).expect("restored legacy project remains openable");

            let recovered =
                migrate_legacy_project_package(&temp_dir).expect("retry migration succeeds");
            load_project_package(&temp_dir).expect("recovered package opens");
            assert_eq!(recovered.designs.len(), 1);
            cleanup_package_test(&temp_dir);
        }
    }

    #[test]
    fn migration_recovers_a_backup_after_interruption_before_adoption() {
        let (temp_dir, raw_legacy) = legacy_fixture("conversation");
        let (_stage, backup) = sibling_transaction_paths(&temp_dir).expect("transaction paths");
        fs::rename(&temp_dir, &backup).expect("simulate interrupted backup rename");

        let migrated = migrate_legacy_project_package(&temp_dir).expect("recover and migrate");
        assert_eq!(migrated.designs.len(), 1);
        let design_paths =
            design_package_paths(&temp_dir, &migrated.designs[0].manifest.id).expect("paths");
        assert_eq!(
            fs::read(design_paths.legacy_project_archive).expect("archive"),
            raw_legacy
        );
        assert!(!backup.exists());
        cleanup_package_test(&temp_dir);
    }

    #[test]
    fn migration_refuses_to_delete_an_unmarked_sibling_stage_directory() {
        let (temp_dir, _raw_legacy) = legacy_fixture("empty");
        let (stage, _backup) = sibling_transaction_paths(&temp_dir).expect("transaction paths");
        fs::create_dir_all(&stage).expect("create unrelated sibling");
        fs::write(stage.join("owned-by-user.txt"), b"keep").expect("write unrelated data");

        let error = migrate_legacy_project_package(&temp_dir)
            .expect_err("unmarked sibling must block migration");
        assert!(error.to_string().contains("unmarked transaction directory"));
        assert_eq!(
            fs::read(stage.join("owned-by-user.txt")).expect("unrelated data survives"),
            b"keep"
        );
        load_project(&temp_dir).expect("legacy project remains openable");
        cleanup_package_test(&temp_dir);
    }

    #[test]
    fn migrated_package_rejects_a_changed_legacy_archive() {
        let (temp_dir, _raw_legacy) = legacy_fixture("builder");
        let migrated = migrate_legacy_project_package(&temp_dir).expect("migrate fixture");
        let paths =
            design_package_paths(&temp_dir, &migrated.designs[0].manifest.id).expect("paths");
        fs::write(&paths.legacy_project_archive, b"{}\n").expect("damage archive fixture");

        let error = load_project_package(&temp_dir).expect_err("hash mismatch must be rejected");
        assert!(error.to_string().contains("hash does not match"));
        cleanup_package_test(&temp_dir);
    }

    fn legacy_fixture(kind: &str) -> (std::path::PathBuf, Vec<u8>) {
        let sequence = ID_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let temp_dir = std::env::temp_dir().join(format!(
            "fraia-legacy-package-{kind}-{}-{}-{sequence}",
            std::process::id(),
            timestamp_id()
        ));
        let (mut project, paths) =
            create_project(&temp_dir, &format!("{kind} fixture")).expect("create legacy fixture");
        match kind {
            "authored" => {
                project
                    .structural_model
                    .as_mut()
                    .expect("structural model")
                    .nodes
                    .push(StructuralNode {
                        id: "fixture-node".into(),
                        x: 1.0,
                        y: 2.0,
                        z: 3.0,
                    });
            }
            "builder" => {
                project.builder_graph = Some(portal_frame_builder_graph(
                    "fixture-builder",
                    "clear_span",
                    "310UB",
                    "310UB",
                    12.0,
                    5.0,
                    15.0,
                    20.0,
                    None,
                    None,
                ));
                project.structural_model = None;
            }
            "conversation" => {
                let session = serde_json::from_value(json!({
                    "id": "fixture-session",
                    "surface": "default",
                    "title": "Design conversation",
                    "status": "active",
                    "messages": [{
                        "author": "user",
                        "text": "Design the frame",
                        "createdAt": "2026-01-01T00:00:00Z"
                    }],
                    "planItems": [],
                    "currentQuestion": null,
                    "createdAt": "2026-01-01T00:00:00Z",
                    "updatedAt": "2026-01-01T00:00:00Z"
                }))
                .expect("conversation fixture");
                project.agent_state.sessions.push(session);
            }
            "runs" => {
                fs::create_dir_all(paths.runs_dir.join("run-fixture")).expect("create run fixture");
                fs::write(
                    paths.runs_dir.join("run-fixture/results.json"),
                    b"{\"status\":\"complete\"}\n",
                )
                .expect("write run fixture");
            }
            "empty" => {}
            other => panic!("unknown legacy fixture kind {other}"),
        }
        save_project(&temp_dir, &project).expect("save legacy fixture");
        fs::write(&paths.planning_file, format!("# {kind} planning\n"))
            .expect("write planning fixture");
        let raw = fs::read(&paths.project_file).expect("read legacy fixture bytes");
        (temp_dir, raw)
    }

    fn cleanup_package_test(project_dir: &Path) {
        let _ = fs::remove_dir_all(project_dir);
        if let Ok((stage, backup)) = sibling_transaction_paths(project_dir) {
            let _ = fs::remove_dir_all(stage);
            let _ = fs::remove_dir_all(backup);
        }
    }

    #[test]
    fn project_round_trips_structural_model() {
        let temp_dir = std::env::temp_dir().join(format!("fraia-project-test-{}", timestamp_id()));
        let (mut project, _) = create_project(&temp_dir, "test-project").expect("create project");

        project.builder_graph = Some(portal_frame_builder_graph(
            "builder-1",
            "clear_span",
            "310UB",
            "310UB",
            6.0,
            4.0,
            20.0,
            10.0,
            Some("run-1".into()),
            Some(1),
        ));

        let structural = project.structural_model.as_mut().expect("structural model");
        structural.nodes.push(StructuralNode {
            id: "n1".into(),
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });
        structural.nodes.push(StructuralNode {
            id: "n2".into(),
            x: 6.0,
            y: 0.0,
            z: 0.0,
        });
        structural.members.push(StructuralMember {
            id: "m1".into(),
            start_node: "n1".into(),
            end_node: "n2".into(),
            role: "beam".into(),
            semantic_tags: Vec::new(),
            section_id: "W310x39".into(),
            material_id: "steel".into(),
        });
        structural.loads.push(LoadAssignment {
            id: "load-1".into(),
            target: AssignmentTargetRef::Member("m1".into()),
            load_case_id: "LC1".into(),
            kind: LoadKind::UniformLine,
            direction: LoadVector {
                x: 0.0,
                y: -1.0,
                z: 0.0,
            },
            magnitude: 20_000.0,
        });
        structural.releases.push(ReleaseAssignment {
            id: "release-1".into(),
            target: MemberEndTarget {
                member_id: "m1".into(),
                end: MemberEnd::Start,
            },
            ux: false,
            uy: false,
            uz: false,
            rx: false,
            ry: false,
            rz: true,
        });

        save_project(&temp_dir, &project).expect("save project");
        let (loaded, _) = load_project(&temp_dir).expect("load project");
        let graph = loaded.builder_graph.expect("loaded builder graph");
        let structural = loaded.structural_model.expect("loaded structural model");

        assert_eq!(graph.root_node_ids.len(), 1);
        assert_eq!(
            graph.nodes[0].archetype_id,
            "frame.portal_2d_steel_concept.v1"
        );
        assert_eq!(structural.nodes.len(), 2);
        assert_eq!(structural.members.len(), 1);
        assert_eq!(structural.loads.len(), 1);
        assert_eq!(structural.releases.len(), 1);
        assert_eq!(structural.loads[0].load_case_id, "LC1");
        assert!(matches!(
            structural.releases[0].target.end,
            MemberEnd::Start
        ));

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn materialize_project_structural_model_can_fall_back_to_builder() {
        let temp_dir =
            std::env::temp_dir().join(format!("fraia-project-builder-test-{}", timestamp_id()));
        let (mut project, _) =
            create_project(&temp_dir, "builder-project").expect("create project");
        project.builder_graph = Some(portal_frame_builder_graph(
            "builder-portal",
            "one_internal",
            "310UB",
            "360UB",
            24.0,
            7.0,
            18.0,
            90.0,
            None,
            None,
        ));
        project.structural_model = None;

        let materialized =
            materialize_project_structural_model(&project).expect("materialize structural model");
        assert!(!materialized.members.is_empty());
        assert!(!materialized.nodes.is_empty());

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn load_project_migrates_legacy_builder_instance_to_builder_graph() {
        let temp_dir =
            std::env::temp_dir().join(format!("fraia-project-legacy-builder-{}", timestamp_id()));
        let (_project, paths) =
            create_project(&temp_dir, "legacy-builder-project").expect("create project");
        let legacy_project = json!({
            "schema_version": "0.1.0",
            "name": "legacy-builder-project",
            "created_at": "2026-04-14T00:00:00Z",
            "intent": {
                "building_type": "warehouse",
                "design_stage": "concept",
                "objective_priority": "balanced",
                "option_count": 5,
                "hard_constraints": [],
                "soft_preferences": [],
                "search_permissions": {
                    "resize_sections": true,
                    "add_internal_columns": true,
                    "change_topology": true
                },
                "approval_triggers": []
            },
            "requirements": {
                "span_m": 24.0,
                "height_m": 7.0,
                "gravity_load_kn_per_m": 18.0,
                "lateral_load_kn": 90.0,
                "max_deflection_ratio": 250.0,
                "max_drift_ratio": 300.0,
                "max_utilization": 0.67,
                "max_internal_columns": 2
            },
            "files": { "planning": "planning.md" },
            "builder_instance": {
                "id": "legacy-builder",
                "archetype_id": "frame.portal_2d_steel_concept",
                "topology_id": "one_internal",
                "beam_section": "310UB",
                "column_section": "360UB",
                "span_m": 24.0,
                "height_m": 7.0,
                "gravity_load_kn_per_m": 18.0,
                "lateral_load_kn": 90.0
            },
            "structural_model": null
        });
        fs::write(
            &paths.project_file,
            serde_json::to_string_pretty(&legacy_project).expect("serialize legacy project"),
        )
        .expect("write legacy project");

        let (loaded, _) = load_project(&temp_dir).expect("load migrated project");
        let graph = loaded.builder_graph.expect("migrated builder graph");
        assert!(loaded.legacy_builder_instance.is_none());
        assert_eq!(graph.root_node_ids.len(), 1);
        assert_eq!(
            graph.nodes[0].archetype_id,
            "frame.portal_2d_steel_concept.v1"
        );

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn load_project_migrates_legacy_quantity_fields_on_save() {
        let temp_dir = std::env::temp_dir().join(format!(
            "fraia-project-quantity-migration-{}",
            timestamp_id()
        ));
        let (_project, paths) =
            create_project(&temp_dir, "quantity-migration").expect("create project");
        let legacy_project = json!({
            "schema_version": "0.1.0",
            "name": "quantity-migration",
            "created_at": "2026-04-14T00:00:00Z",
            "intent": {
                "building_type": "warehouse",
                "design_stage": "concept",
                "objective_priority": "balanced",
                "option_count": 5,
                "hard_constraints": [],
                "soft_preferences": [],
                "search_permissions": {
                    "resize_sections": true,
                    "add_internal_columns": true,
                    "change_topology": true
                },
                "approval_triggers": []
            },
            "requirements": {
                "span_m": 24.0,
                "height_m": 7.0,
                "gravity_load_kn_per_m": 18.0,
                "lateral_load_kn": 90.0,
                "max_deflection_ratio": 250.0,
                "max_drift_ratio": 300.0,
                "max_utilization": 0.67,
                "max_internal_columns": 2
            },
            "files": { "planning": "planning.md" },
            "structural_model": {
                "dimension": "2d-in-3d",
                "nodes": [
                    { "id": "n1", "x": 0.0, "y": 0.0, "z": 0.0 },
                    { "id": "n2", "x": 6.0, "y": 0.0, "z": 0.0 }
                ],
                "members": [
                    {
                        "id": "m1",
                        "start_node": "n1",
                        "end_node": "n2",
                        "role": "beam",
                        "section_id": "310UB",
                        "material_id": "steel"
                    }
                ],
                "plates": [
                    {
                        "id": "p1",
                        "boundary_nodes": ["n1", "n2"],
                        "role": "slab",
                        "thickness_m": 0.2,
                        "material_id": "steel",
                        "generated_from": "legacy"
                    }
                ],
                "supports": [],
                "loads": [
                    {
                        "id": "load-1",
                        "target": { "Member": "m1" },
                        "load_case_id": "LC1",
                        "family": "distributed",
                        "direction": { "x": 0.0, "y": -1.0, "z": 0.0 },
                        "magnitude": 18000.0
                    }
                ],
                "releases": [],
                "load_cases": [],
                "builder_node_materializations": []
            }
        });
        fs::write(
            &paths.project_file,
            serde_json::to_string_pretty(&legacy_project).expect("serialize legacy project"),
        )
        .expect("write legacy project");

        let (loaded, _) = load_project(&temp_dir).expect("load migrated project");
        assert_eq!(loaded.schema_version, "0.2.0");
        assert_eq!(loaded.requirements.span_m, 24.0);
        assert_eq!(loaded.requirements.gravity_load_kn_per_m, 18.0);
        let structural = loaded.structural_model.as_ref().expect("structural model");
        assert_eq!(structural.nodes[1].x, 6.0);
        assert_eq!(structural.plates[0].thickness_m, 0.2);
        assert_eq!(structural.loads[0].magnitude, 18_000.0);

        save_project(&temp_dir, &loaded).expect("save migrated project");
        let raw = fs::read_to_string(&paths.project_file).expect("read migrated project");
        assert!(!raw.contains("span_m"));
        assert!(!raw.contains("height_m"));
        assert!(!raw.contains("gravity_load_kn_per_m"));
        assert!(!raw.contains("lateral_load_kn"));
        assert!(!raw.contains("thickness_m"));

        let saved: serde_json::Value = serde_json::from_str(&raw).expect("parse saved project");
        assert_eq!(saved["schema_version"], "0.2.0");
        assert_eq!(saved["requirements"]["span"]["quantityKind"], "length");
        assert_eq!(saved["requirements"]["span"]["canonicalUnit"], "m");
        assert_eq!(saved["requirements"]["gravityLoad"]["value"], 18_000.0);
        assert_eq!(
            saved["structural_model"]["nodes"][1]["position"]["quantityKind"],
            "length"
        );
        assert_eq!(
            saved["structural_model"]["loads"][0]["magnitude"]["canonicalUnit"],
            "N/m"
        );

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn project_round_trips_planning_draft_and_backfills_missing_value() {
        let temp_dir =
            std::env::temp_dir().join(format!("fraia-project-planning-test-{}", timestamp_id()));
        let (mut project, paths) = create_project(&temp_dir, "planning-project").expect("create");
        let mut draft = planning_draft(&project);
        draft.project_intent.name = "Workbench Project".into();
        draft.project_intent.building_type = "portal_frame".into();
        draft.system_brief.system_family_hint = "portal_frame".into();
        draft.system_brief.notes = "Portal frame concept for workbench testing".into();
        draft.geometry_and_loads.span_m = 28.0;
        draft.design_constraints.allow_internal_columns = true;
        draft.design_constraints.max_internal_columns = 1;
        apply_planning_draft(&mut project, draft.clone());
        save_project(&temp_dir, &project).expect("save");

        let (loaded, _) = load_project(&temp_dir).expect("load");
        let loaded_draft = loaded.planning_draft.expect("planning draft");
        assert_eq!(loaded_draft.project_intent.name, "Workbench Project");
        assert_eq!(loaded_draft.system_brief.system_family_hint, "portal_frame");
        assert_eq!(loaded.requirements.span_m, 28.0);
        assert_eq!(loaded.requirements.max_internal_columns, 1);

        let raw = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(&paths.project_file).expect("read project"),
        )
        .expect("parse project json");
        let mut no_planning = raw;
        no_planning
            .as_object_mut()
            .expect("project object")
            .remove("planning_draft");
        fs::write(
            &paths.project_file,
            serde_json::to_string_pretty(&no_planning).expect("serialise without draft"),
        )
        .expect("write legacy-ish project");

        let (backfilled, _) = load_project(&temp_dir).expect("load without planning draft");
        assert!(backfilled.planning_draft.is_some());
        assert_eq!(
            backfilled
                .planning_draft
                .expect("backfilled draft")
                .project_intent
                .name,
            "Workbench Project"
        );

        let _ = fs::remove_dir_all(temp_dir);
    }
}
