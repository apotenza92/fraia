use crate::types::{Combo2D, FrameModel2D};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculixRuntimeStatus {
    pub ccx_available: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ccx_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculixCompiledInput {
    pub adapter: String,
    pub job_name: String,
    pub combo_id: String,
    pub node_count: usize,
    pub element_count: usize,
    pub runtime: CalculixRuntimeStatus,
    pub input_deck: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CalculixExecutionOutcome {
    SkippedRuntimeUnavailable,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculixExecutionArtifacts {
    pub adapter: String,
    pub job_name: String,
    pub working_dir: String,
    pub command: Vec<String>,
    pub runtime: CalculixRuntimeStatus,
    pub outcome: CalculixExecutionOutcome,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub produced_files: Vec<String>,
}

#[cfg(test)]
pub(crate) static CALCULIX_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub fn calculix_runtime_status() -> CalculixRuntimeStatus {
    if std::env::var("FRAIA_DISABLE_CALCULIX_RUNTIME").as_deref() == Ok("1") {
        return CalculixRuntimeStatus {
            ccx_available: false,
            ccx_path: None,
        };
    }
    if let Some(path) = std::env::var_os("FRAIA_CCX_PATH") {
        return valid_ccx_file(path).map_or(
            CalculixRuntimeStatus {
                ccx_available: false,
                ccx_path: None,
            },
            available_runtime,
        );
    }

    for dir in calculix_runtime_dirs() {
        if let Some(path) = find_ccx_in_dir(&dir) {
            return available_runtime(path);
        }
    }

    CalculixRuntimeStatus {
        ccx_available: false,
        ccx_path: None,
    }
}

pub fn require_calculix_runtime() -> Result<CalculixRuntimeStatus> {
    let runtime = calculix_runtime_status();
    if !runtime.ccx_available {
        bail!(
            "CalculiX runtime unavailable. Fraia could not find a bundled, managed, or system `ccx` executable. Set FRAIA_CCX_PATH to a valid executable or provide a packaged runtime under the Fraia runtimes/calculix directory."
        );
    }
    Ok(runtime)
}

fn available_runtime(path: PathBuf) -> CalculixRuntimeStatus {
    CalculixRuntimeStatus {
        ccx_available: true,
        ccx_path: Some(path.display().to_string()),
    }
}

fn valid_ccx_file(path: std::ffi::OsString) -> Option<PathBuf> {
    let path = PathBuf::from(path);
    path.is_file().then_some(path)
}

fn calculix_runtime_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(dir) = std::env::var_os("FRAIA_CALCULIX_DIR") {
        push_calculix_dir_candidates(&mut dirs, PathBuf::from(dir));
    }
    if let Some(dir) = std::env::var_os("FRAIA_RUNTIME_DIR") {
        push_calculix_dir_candidates(&mut dirs, PathBuf::from(dir).join("calculix"));
    }
    if let Some(dir) = std::env::var_os("FRAIA_APP_RESOURCE_DIR") {
        push_calculix_dir_candidates(
            &mut dirs,
            PathBuf::from(dir).join("runtimes").join("calculix"),
        );
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            push_calculix_dir_candidates(&mut dirs, exe_dir.join("runtimes").join("calculix"));
            push_calculix_dir_candidates(
                &mut dirs,
                exe_dir
                    .join("..")
                    .join("Resources")
                    .join("runtimes")
                    .join("calculix"),
            );
        }
    }
    if let Some(paths) = std::env::var_os("PATH") {
        dirs.extend(std::env::split_paths(&paths));
    }
    dirs.extend(common_calculix_runtime_dirs());
    dedupe_existing_dirs(dirs)
}

fn push_calculix_dir_candidates(dirs: &mut Vec<PathBuf>, base: PathBuf) {
    dirs.push(base.clone());
    dirs.push(base.join("bin"));
    dirs.push(base.join(platform_arch()).join("bin"));
    dirs.push(base.join(platform_arch()));
}

fn common_calculix_runtime_dirs() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/usr/local/bin"),
        PathBuf::from("/opt/local/bin"),
    ]
}

fn dedupe_existing_dirs(dirs: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    for dir in dirs {
        if !dir.is_dir() {
            continue;
        }
        let key = dir.display().to_string();
        if seen.insert(key) {
            out.push(dir);
        }
    }
    out
}

fn find_ccx_in_dir(dir: &Path) -> Option<PathBuf> {
    let direct = dir.join(if cfg!(windows) { "ccx.exe" } else { "ccx" });
    if direct.is_file() {
        return Some(direct);
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return None;
    };
    let mut versioned: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.is_file()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.starts_with("ccx_") || name.starts_with("ccx-"))
                    .unwrap_or(false)
        })
        .collect();
    versioned.sort();
    versioned.pop()
}

fn platform_arch() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => "darwin-arm64",
        ("macos", "x86_64") => "darwin-x64",
        ("linux", "aarch64") => "linux-arm64",
        ("linux", "x86_64") => "linux-x64",
        ("windows", "x86_64") => "win32-x64",
        _ => "unknown",
    }
}

pub fn execute_calculix_compiled_input(
    compiled: &CalculixCompiledInput,
    working_dir: &Path,
) -> Result<CalculixExecutionArtifacts> {
    execute_calculix_compiled_input_with_runtime(compiled, working_dir, compiled.runtime.clone())
}

pub fn execute_calculix_compiled_input_with_runtime(
    compiled: &CalculixCompiledInput,
    working_dir: &Path,
    runtime: CalculixRuntimeStatus,
) -> Result<CalculixExecutionArtifacts> {
    fs::create_dir_all(working_dir).with_context(|| {
        format!(
            "failed to create CalculiX working dir {}",
            working_dir.display()
        )
    })?;
    let inp_path = working_dir.join(format!("{}.inp", compiled.job_name));
    fs::write(&inp_path, &compiled.input_deck)
        .with_context(|| format!("failed to write {}", inp_path.display()))?;

    let command = runtime
        .ccx_path
        .as_ref()
        .map(|ccx| vec![ccx.clone(), "-i".into(), compiled.job_name.clone()])
        .unwrap_or_else(|| vec!["ccx".into(), "-i".into(), compiled.job_name.clone()]);

    if !runtime.ccx_available {
        return Ok(CalculixExecutionArtifacts {
            adapter: "calculix.ccx.execute.v1".into(),
            job_name: compiled.job_name.clone(),
            working_dir: working_dir.display().to_string(),
            command,
            runtime,
            outcome: CalculixExecutionOutcome::SkippedRuntimeUnavailable,
            exit_code: None,
            stdout: String::new(),
            stderr: "CalculiX runtime unavailable; execution skipped".into(),
            produced_files: collect_calculix_output_files(working_dir, &compiled.job_name),
        });
    }

    let output = Command::new(
        runtime
            .ccx_path
            .as_ref()
            .context("ccx path missing despite runtime claiming availability")?,
    )
    .arg("-i")
    .arg(&compiled.job_name)
    .current_dir(working_dir)
    .output()
    .with_context(|| format!("failed to invoke CalculiX in {}", working_dir.display()))?;

    let outcome = if output.status.success() {
        CalculixExecutionOutcome::Completed
    } else {
        CalculixExecutionOutcome::Failed
    };

    Ok(CalculixExecutionArtifacts {
        adapter: "calculix.ccx.execute.v1".into(),
        job_name: compiled.job_name.clone(),
        working_dir: working_dir.display().to_string(),
        command,
        runtime,
        outcome,
        exit_code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        produced_files: collect_calculix_output_files(working_dir, &compiled.job_name),
    })
}

pub fn compile_frame_model_to_calculix_input(
    model: &FrameModel2D,
    combo: &Combo2D,
    job_name: &str,
) -> Result<CalculixCompiledInput> {
    if model.nodes.is_empty() {
        bail!("cannot compile empty frame model to CalculiX");
    }
    if model.elements.is_empty() {
        bail!("cannot compile frame model without elements to CalculiX");
    }

    let mut node_ids = HashMap::new();
    for (index, node) in model.nodes.iter().enumerate() {
        node_ids.insert(node.id.clone(), index + 1);
    }

    let mut element_ids = HashMap::new();
    for (index, element) in model.elements.iter().enumerate() {
        element_ids.insert(element.id.clone(), index + 1);
    }

    let mut section_groups: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for element in &model.elements {
        let numeric_id = *element_ids
            .get(&element.id)
            .with_context(|| format!("missing numeric id for element {}", element.id))?;
        section_groups
            .entry(element.section.id.clone())
            .or_default()
            .push(numeric_id);
    }

    let combo_loads = combo_nodal_loads(model, combo)?;
    let support_sets = support_node_sets(model, &node_ids)?;

    let mut lines = vec![
        "*HEADING".into(),
        format!("Fraia CalculiX compilation for {} ({})", job_name, combo.id),
        "*NODE,NSET=NALL".into(),
    ];
    for node in &model.nodes {
        let node_id = *node_ids
            .get(&node.id)
            .with_context(|| format!("missing numeric id for node {}", node.id))?;
        lines.push(format!("{}, {}, {}, 0.0", node_id, node.x, node.y));
    }
    if !support_sets.all.is_empty() {
        lines.push(format!("*NSET,NSET=SUPPORT_ALL"));
        lines.push(join_numeric_list(&support_sets.all));
    }
    if let Some(left) = support_sets.left {
        lines.push(format!("*NSET,NSET=SUPPORT_LEFT"));
        lines.push(left.to_string());
    }
    if let Some(right) = support_sets.right {
        lines.push(format!("*NSET,NSET=SUPPORT_RIGHT"));
        lines.push(right.to_string());
    }

    for (section_id, elements) in &section_groups {
        let elset_name = sanitize_name(&format!("ESET_{}", section_id));
        lines.push(format!("*ELEMENT,TYPE=B31,ELSET={}", elset_name));
        for numeric_element_id in elements {
            let element = model
                .elements
                .iter()
                .find(|element| element_ids.get(&element.id) == Some(numeric_element_id))
                .with_context(|| {
                    format!("missing element for numeric id {}", numeric_element_id)
                })?;
            let ni = node_ids
                .get(&element.i)
                .with_context(|| format!("missing start node {}", element.i))?;
            let nj = node_ids
                .get(&element.j)
                .with_context(|| format!("missing end node {}", element.j))?;
            lines.push(format!("{}, {}, {}", numeric_element_id, ni, nj));
        }
    }
    lines.push("*ELSET,ELSET=EALL".into());
    lines.push(join_numeric_list(
        &(1..=model.elements.len()).collect::<Vec<_>>(),
    ));

    let mut written_materials = HashMap::new();
    for element in &model.elements {
        if written_materials.contains_key(&element.section.id) {
            continue;
        }
        let material_name = sanitize_name(&format!("MAT_{}", element.section.id));
        let elset_name = sanitize_name(&format!("ESET_{}", element.section.id));
        let (rect_b, rect_h) =
            equivalent_rectangular_section(element.section.area, element.section.i).with_context(
                || {
                    format!(
                        "failed to derive equivalent rectangular section for {}",
                        element.section.id
                    )
                },
            )?;
        lines.push(format!("*MATERIAL,NAME={}", material_name));
        lines.push("*ELASTIC".into());
        lines.push(format!("{}, 0.3", element.material.e));
        lines.push(format!(
            "*BEAM SECTION,ELSET={},MATERIAL={},SECTION=RECT",
            elset_name, material_name
        ));
        lines.push(format!("{}, {}", rect_b, rect_h));
        lines.push("0., 0., 1.".into());
        written_materials.insert(element.section.id.clone(), true);
    }

    lines.push("*BOUNDARY".into());
    lines.push("NALL,3,5".into());
    for support in &model.supports {
        let node_id = *node_ids
            .get(&support.node)
            .with_context(|| format!("missing support node {}", support.node))?;
        if support.ux {
            lines.push(format!("{},1", node_id));
        }
        if support.uy {
            lines.push(format!("{},2", node_id));
        }
        if support.rz {
            lines.push(format!("{},6", node_id));
        }
    }

    lines.push("*STEP".into());
    lines.push("*STATIC".into());
    if !combo_loads.is_empty() {
        lines.push("*CLOAD".into());
        for (node_id, fx_n, fy_n, mz_nm) in combo_loads {
            if fx_n.abs() > 1e-9 {
                lines.push(format!("{},1,{}", node_id, fx_n));
            }
            if fy_n.abs() > 1e-9 {
                lines.push(format!("{},2,{}", node_id, fy_n));
            }
            if mz_nm.abs() > 1e-9 {
                lines.push(format!("{},6,{}", node_id, mz_nm));
            }
        }
    }
    lines.push("*NODE FILE".into());
    lines.push("U".into());
    lines.push("*EL FILE".into());
    lines.push("S".into());
    lines.push("*NODE PRINT,NSET=NALL".into());
    lines.push("U".into());
    lines.push("*EL PRINT,ELSET=EALL,GLOBAL=NO".into());
    lines.push("S".into());
    if !support_sets.all.is_empty() {
        lines.push("*NODE PRINT,NSET=SUPPORT_ALL".into());
        lines.push("RF".into());
    }
    if support_sets.left.is_some() {
        lines.push("*NODE PRINT,NSET=SUPPORT_LEFT".into());
        lines.push("RF".into());
    }
    if support_sets.right.is_some() {
        lines.push("*NODE PRINT,NSET=SUPPORT_RIGHT".into());
        lines.push("RF".into());
    }
    lines.push("*END STEP".into());

    Ok(CalculixCompiledInput {
        adapter: "calculix.ccx.compile.v1".into(),
        job_name: job_name.into(),
        combo_id: combo.id.clone(),
        node_count: model.nodes.len(),
        element_count: model.elements.len(),
        runtime: calculix_runtime_status(),
        input_deck: lines.join("\n") + "\n",
    })
}

#[derive(Debug, Clone)]
struct SupportNodeSets {
    all: Vec<usize>,
    left: Option<usize>,
    right: Option<usize>,
}

fn combo_nodal_loads(model: &FrameModel2D, combo: &Combo2D) -> Result<Vec<(usize, f64, f64, f64)>> {
    let mut node_ids = HashMap::new();
    for (index, node) in model.nodes.iter().enumerate() {
        node_ids.insert(node.id.as_str(), index + 1);
    }
    let mut loads: BTreeMap<usize, (f64, f64, f64)> = BTreeMap::new();
    for (case_id, factor) in &combo.factors {
        let load_case = model
            .load_cases
            .iter()
            .find(|case| case.id == *case_id)
            .with_context(|| {
                format!(
                    "combo {} referenced missing load case {}",
                    combo.id, case_id
                )
            })?;
        for load in &load_case.nodal_loads {
            let numeric_id = *node_ids
                .get(load.node.as_str())
                .with_context(|| format!("missing load node {}", load.node))?;
            let entry = loads.entry(numeric_id).or_insert((0.0, 0.0, 0.0));
            entry.0 += load.fx * factor;
            entry.1 += load.fy * factor;
            entry.2 += load.mz * factor;
        }
    }
    Ok(loads
        .into_iter()
        .map(|(node_id, (fx, fy, mz))| (node_id, fx, fy, mz))
        .collect())
}

fn support_node_sets(
    model: &FrameModel2D,
    node_ids: &HashMap<String, usize>,
) -> Result<SupportNodeSets> {
    let mut supports: Vec<(usize, f64)> = model
        .supports
        .iter()
        .filter_map(|support| {
            let node_id = node_ids.get(&support.node).copied()?;
            let x = model.nodes.iter().find(|node| node.id == support.node)?.x;
            Some((node_id, x))
        })
        .collect();
    supports.sort_by(|a, b| a.1.total_cmp(&b.1));
    let all = supports
        .iter()
        .map(|(node_id, _)| *node_id)
        .collect::<Vec<_>>();
    let left = supports.first().map(|(node_id, _)| *node_id);
    let right = supports.last().map(|(node_id, _)| *node_id);
    Ok(SupportNodeSets { all, left, right })
}

fn join_numeric_list(values: &[usize]) -> String {
    values
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn equivalent_rectangular_section(area_m2: f64, inertia_m4: f64) -> Result<(f64, f64)> {
    if area_m2 <= 0.0 || inertia_m4 <= 0.0 {
        bail!("section area and inertia must be positive");
    }
    let h = ((12.0 * inertia_m4) / area_m2).sqrt();
    if h <= 0.0 {
        bail!("failed to derive positive section depth");
    }
    let b = area_m2 / h;
    if b <= 0.0 {
        bail!("failed to derive positive section width");
    }
    Ok((b, h))
}

fn collect_calculix_output_files(working_dir: &Path, job_name: &str) -> Vec<String> {
    ["dat", "frd", "sta", "cvg", "12d", "inp"]
        .iter()
        .filter_map(|ext| {
            let path = working_dir.join(format!("{}.{}", job_name, ext));
            if !path.exists() {
                return None;
            }
            path.file_name()
                .map(|name| name.to_string_lossy().to_string())
        })
        .collect()
}

fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        CalculixExecutionOutcome, CalculixRuntimeStatus, compile_frame_model_to_calculix_input,
        execute_calculix_compiled_input, execute_calculix_compiled_input_with_runtime,
        require_calculix_runtime,
    };
    use crate::{build_frame_model_from_builder_graph, simply_supported_beam_builder_graph};
    use std::fs;

    #[test]
    fn compiles_simple_beam_frame_model_to_calculix_input() {
        let graph = simply_supported_beam_builder_graph(
            "builder.beam.calculix",
            "250UB",
            6.0,
            8.0,
            Some(20.0),
            Some(3.0),
            None,
            None,
        );
        let model = build_frame_model_from_builder_graph(&graph).expect("frame model");
        let combo = model
            .combos
            .iter()
            .find(|combo| combo.id == "SLS")
            .expect("SLS combo");

        let compiled =
            compile_frame_model_to_calculix_input(&model, combo, "beam-test").expect("compile");

        assert_eq!(compiled.combo_id, "SLS");
        assert!(compiled.input_deck.contains("*ELEMENT,TYPE=B31"));
        assert!(compiled.input_deck.contains("*BEAM SECTION"));
        assert!(compiled.input_deck.contains("*BOUNDARY"));
        assert!(compiled.input_deck.contains("NALL,3,5"));
        assert!(compiled.input_deck.contains("*CLOAD"));
        assert!(compiled.input_deck.contains("*STATIC"));
        assert!(compiled.input_deck.contains("*ELSET,ELSET=EALL"));
        assert!(
            compiled
                .input_deck
                .contains("*EL PRINT,ELSET=EALL,GLOBAL=NO")
        );
    }

    #[test]
    fn execution_artifacts_report_runtime_unavailable_honestly() {
        let graph = simply_supported_beam_builder_graph(
            "builder.beam.calculix",
            "250UB",
            6.0,
            8.0,
            Some(20.0),
            Some(3.0),
            None,
            None,
        );
        let model = build_frame_model_from_builder_graph(&graph).expect("frame model");
        let combo = model
            .combos
            .iter()
            .find(|combo| combo.id == "SLS")
            .expect("SLS combo");
        let compiled = compile_frame_model_to_calculix_input(&model, combo, "beam-runtime-test")
            .expect("compile");
        let temp_dir = std::env::temp_dir().join(format!(
            "fraia-calculix-runtime-{}",
            crate::utils::timestamp_id()
        ));

        let execution = execute_calculix_compiled_input_with_runtime(
            &compiled,
            &temp_dir,
            CalculixRuntimeStatus {
                ccx_available: false,
                ccx_path: None,
            },
        )
        .expect("execution artifacts");

        assert!(matches!(
            execution.outcome,
            CalculixExecutionOutcome::SkippedRuntimeUnavailable
        ));
        assert!(temp_dir.join("beam-runtime-test.inp").exists());
        assert!(
            execution
                .produced_files
                .iter()
                .any(|file| file == "beam-runtime-test.inp")
        );

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn finds_versioned_ccx_in_managed_runtime_layout() {
        let temp_dir = std::env::temp_dir().join(format!(
            "fraia-managed-calculix-runtime-{}",
            crate::utils::timestamp_id()
        ));
        let bin_dir = temp_dir.join(super::platform_arch()).join("bin");
        fs::create_dir_all(&bin_dir).expect("runtime bin dir");
        let ccx_path = bin_dir.join("ccx_2.23");
        fs::write(&ccx_path, "#!/bin/sh\nexit 0\n").expect("fake ccx");

        let mut candidates = Vec::new();
        super::push_calculix_dir_candidates(&mut candidates, temp_dir.clone());
        let found = candidates
            .iter()
            .find_map(|candidate| super::find_ccx_in_dir(candidate));

        assert_eq!(found.as_deref(), Some(ccx_path.as_path()));
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn managed_or_system_calculix_runtime_is_available_when_present() {
        let _env_guard = super::CALCULIX_ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let runtime = match require_calculix_runtime() {
            Ok(runtime) => runtime,
            Err(error) => {
                eprintln!("skipping real CalculiX runtime check: {error:#}");
                return;
            }
        };

        let path = runtime.ccx_path.expect("ccx path");
        assert!(
            path.contains("ccx"),
            "expected ccx-like executable path, got {path}"
        );
    }

    #[test]
    fn executes_simple_beam_with_managed_or_system_calculix_when_available() {
        let _env_guard = super::CALCULIX_ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Err(error) = require_calculix_runtime() {
            eprintln!("skipping real CalculiX execution check: {error:#}");
            return;
        }
        let graph = simply_supported_beam_builder_graph(
            "builder.beam.calculix",
            "250UB",
            6.0,
            8.0,
            Some(20.0),
            Some(3.0),
            None,
            None,
        );
        let model = build_frame_model_from_builder_graph(&graph).expect("frame model");
        let combo = model
            .combos
            .iter()
            .find(|combo| combo.id == "SLS")
            .expect("SLS combo");
        let compiled = compile_frame_model_to_calculix_input(&model, combo, "beam-real-ccx-test")
            .expect("compile");
        let temp_dir = std::env::temp_dir().join(format!(
            "fraia-real-calculix-runtime-{}",
            crate::utils::timestamp_id()
        ));

        let execution =
            execute_calculix_compiled_input(&compiled, &temp_dir).expect("execute CalculiX");

        assert!(matches!(
            execution.outcome,
            CalculixExecutionOutcome::Completed
        ));
        assert!(temp_dir.join("beam-real-ccx-test.dat").exists());
        assert!(
            execution
                .produced_files
                .iter()
                .any(|file| file == "beam-real-ccx-test.dat")
        );

        let _ = fs::remove_dir_all(temp_dir);
    }
}
