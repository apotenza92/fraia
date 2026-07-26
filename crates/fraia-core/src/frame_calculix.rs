use crate::frame2d::solve_frame_2d;
use crate::{
    CalculixCompiledInput, CalculixExecutionArtifacts, CalculixExecutionOutcome,
    Frame2DRealization, ProjectFile, SolveResult2D, StructuralModel, ValidationReport,
    compile_frame_model_to_calculix_input, execute_calculix_compiled_input_with_runtime,
    materialize_project_structural_model, realize_structural_model_to_frame2d,
    require_calculix_runtime, validate_structural_model,
};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentFrameCalculixRunManifest {
    pub run_kind: String,
    pub generated_at: String,
    pub project_name: String,
    pub combo_id: String,
    pub adapter: String,
    pub runtime_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameNodeDisplacementPoint {
    pub node_id: String,
    pub x_m: f64,
    pub y_m: f64,
    pub ux_m: f64,
    pub uy_m: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameSupportReactionPoint {
    pub node_id: String,
    pub x_m: f64,
    pub y_m: f64,
    pub fx_n: f64,
    pub fy_n: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameNodeDisplacementComparisonPoint {
    pub node_id: String,
    pub x_m: f64,
    pub y_m: f64,
    pub internal_ux_m: f64,
    pub calculix_ux_m: f64,
    pub abs_ux_diff_m: f64,
    pub internal_uy_m: f64,
    pub calculix_uy_m: f64,
    pub abs_uy_diff_m: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameSupportReactionComparisonPoint {
    pub node_id: String,
    pub x_m: f64,
    pub y_m: f64,
    pub internal_fx_n: f64,
    pub calculix_fx_n: f64,
    pub abs_fx_diff_n: f64,
    pub internal_fy_n: f64,
    pub calculix_fy_n: f64,
    pub abs_fy_diff_n: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameNodeDisplacementComparisonSummary {
    pub node_count: usize,
    pub max_abs_ux_diff_m: f64,
    pub max_abs_ux_diff_node_id: String,
    pub max_abs_uy_diff_m: f64,
    pub max_abs_uy_diff_node_id: String,
    pub points: Vec<FrameNodeDisplacementComparisonPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameSupportReactionComparisonSummary {
    pub support_count: usize,
    pub max_abs_fx_diff_n: f64,
    pub max_abs_fx_diff_node_id: String,
    pub max_abs_fy_diff_n: f64,
    pub max_abs_fy_diff_node_id: String,
    pub points: Vec<FrameSupportReactionComparisonPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameElementStressSummary {
    pub element_id: String,
    pub role: String,
    pub point_count: usize,
    pub max_abs_sxx_pa: f64,
    pub max_abs_sxy_pa: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameElementStressComparisonPoint {
    pub element_id: String,
    pub role: String,
    pub internal_stress_pa: f64,
    pub calculix_max_abs_sxx_pa: f64,
    pub abs_diff_pa: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameElementStressComparisonSummary {
    pub element_count: usize,
    pub max_abs_sxx_diff_pa: f64,
    pub max_abs_sxx_diff_element_id: String,
    pub points: Vec<FrameElementStressComparisonPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentFrameCalculixExecutionArtifacts {
    pub run: CurrentFrameCalculixRunManifest,
    pub structural_model: StructuralModel,
    pub validation: ValidationReport,
    pub realization: Frame2DRealization,
    pub internal_solve: SolveResult2D,
    pub compiled_input: CalculixCompiledInput,
    pub execution: CalculixExecutionArtifacts,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extracted_node_displacements: Option<Vec<FrameNodeDisplacementPoint>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extracted_support_reactions: Option<Vec<FrameSupportReactionPoint>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extracted_element_stresses: Option<Vec<FrameElementStressSummary>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub displacement_comparison: Option<FrameNodeDisplacementComparisonSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_reaction_comparison: Option<FrameSupportReactionComparisonSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_stress_comparison: Option<FrameElementStressComparisonSummary>,
}

pub fn execute_current_frame_project_in_calculix(
    project: &ProjectFile,
    working_dir: &std::path::Path,
) -> Result<CurrentFrameCalculixExecutionArtifacts> {
    let structural_model = materialize_project_structural_model(project).context(
        "no authored structural model or builder-derived structural model was available",
    )?;
    let validation = validate_structural_model(&structural_model);
    let realization = realize_structural_model_to_frame2d(&structural_model)
        .context("failed to realize current project structural model to frame2d")?;
    let combo = realization
        .model
        .combos
        .iter()
        .find(|combo| combo.id == "SLS")
        .or_else(|| realization.model.combos.first())
        .context("no frame2d load combination was available")?;
    let internal_solve = solve_frame_2d(&realization.model, combo).with_context(|| {
        format!(
            "failed to solve combo {} with internal frame2d solver",
            combo.id
        )
    })?;
    let compiled_input = compile_frame_model_to_calculix_input(
        &realization.model,
        combo,
        &format!("frame-{}", crate::utils::timestamp_id()),
    )
    .context("failed to compile current frame realization to CalculiX input")?;
    let runtime = require_calculix_runtime()?;
    let execution =
        execute_calculix_compiled_input_with_runtime(&compiled_input, working_dir, runtime)
            .context("failed to produce CalculiX execution artifacts for current frame project")?;

    let (extracted_node_displacements, extracted_support_reactions, extracted_element_stresses) =
        if matches!(execution.outcome, CalculixExecutionOutcome::Completed) {
            let dat_path = working_dir.join(format!("{}.dat", compiled_input.job_name));
            let dat_text = fs::read_to_string(&dat_path)
                .with_context(|| format!("failed to read {}", dat_path.display()))?;
            extract_frame_calculix_dat(&dat_text, &realization.model)
                .context("failed to extract generic frame CalculiX response from .dat output")?
        } else {
            (None, None, None)
        };

    let displacement_comparison = extracted_node_displacements
        .as_ref()
        .map(|points| compare_node_displacements(&internal_solve, points));
    let support_reaction_comparison = extracted_support_reactions
        .as_ref()
        .map(|points| compare_support_reactions(&internal_solve, points));
    let element_stress_comparison = extracted_element_stresses
        .as_ref()
        .map(|points| compare_element_stresses(&internal_solve, points));

    Ok(CurrentFrameCalculixExecutionArtifacts {
        run: CurrentFrameCalculixRunManifest {
            run_kind: "frame-calculix-run".into(),
            generated_at: crate::utils::iso_now(),
            project_name: project.name.clone(),
            combo_id: combo.id.clone(),
            adapter: execution.adapter.clone(),
            runtime_available: execution.runtime.ccx_available,
        },
        structural_model,
        validation,
        realization,
        internal_solve,
        compiled_input,
        execution,
        extracted_node_displacements,
        extracted_support_reactions,
        extracted_element_stresses,
        displacement_comparison,
        support_reaction_comparison,
        element_stress_comparison,
    })
}

enum Mode {
    None,
    Displacements,
    SupportAll,
    ElementStress,
}

pub fn extract_frame_calculix_dat(
    dat_text: &str,
    model: &crate::FrameModel2D,
) -> Result<(
    Option<Vec<FrameNodeDisplacementPoint>>,
    Option<Vec<FrameSupportReactionPoint>>,
    Option<Vec<FrameElementStressSummary>>,
)> {
    let mut mode = Mode::None;
    let mut node_displacements = Vec::new();
    let mut support_reactions = Vec::new();
    let mut element_stresses: HashMap<String, FrameElementStressSummary> = HashMap::new();

    for line in dat_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("displacements (vx,vy,vz) for set NALL") {
            mode = Mode::Displacements;
            continue;
        }
        if trimmed.starts_with("forces (fx,fy,fz) for set SUPPORT_ALL") {
            mode = Mode::SupportAll;
            continue;
        }
        if trimmed.starts_with("stresses (elem, integ.pnt.,sxx,syy,szz,sxy,sxz,syz) for set EALL") {
            mode = Mode::ElementStress;
            continue;
        }

        let fields: Vec<&str> = trimmed.split_whitespace().collect();
        match mode {
            Mode::Displacements | Mode::SupportAll => {
                if fields.len() < 4 || fields[0].parse::<usize>().is_err() {
                    continue;
                }
                let numeric_node_id = fields[0].parse::<usize>()?;
                let node = model
                    .nodes
                    .get(numeric_node_id.saturating_sub(1))
                    .with_context(|| format!("missing model node index {}", numeric_node_id))?;
                let v1 = fields[1].parse::<f64>()?;
                let v2 = fields[2].parse::<f64>()?;
                match mode {
                    Mode::Displacements => node_displacements.push(FrameNodeDisplacementPoint {
                        node_id: node.id.clone(),
                        x_m: node.x,
                        y_m: node.y,
                        ux_m: v1,
                        uy_m: v2,
                    }),
                    Mode::SupportAll => support_reactions.push(FrameSupportReactionPoint {
                        node_id: node.id.clone(),
                        x_m: node.x,
                        y_m: node.y,
                        fx_n: v1,
                        fy_n: v2,
                    }),
                    _ => {}
                }
            }
            Mode::ElementStress => {
                if fields.len() < 8 || fields[0].parse::<usize>().is_err() {
                    continue;
                }
                let numeric_element_id = fields[0].parse::<usize>()?;
                let element = model
                    .elements
                    .get(numeric_element_id.saturating_sub(1))
                    .with_context(|| {
                        format!("missing model element index {}", numeric_element_id)
                    })?;
                let sxx_pa = fields[2].parse::<f64>()?;
                let sxy_pa = fields[5].parse::<f64>()?;
                let entry = element_stresses
                    .entry(element.id.clone())
                    .or_insert_with(|| FrameElementStressSummary {
                        element_id: element.id.clone(),
                        role: element.role.clone(),
                        point_count: 0,
                        max_abs_sxx_pa: 0.0,
                        max_abs_sxy_pa: 0.0,
                    });
                entry.point_count += 1;
                entry.max_abs_sxx_pa = entry.max_abs_sxx_pa.max(sxx_pa.abs());
                entry.max_abs_sxy_pa = entry.max_abs_sxy_pa.max(sxy_pa.abs());
            }
            Mode::None => {}
        }
    }

    if node_displacements.is_empty() {
        bail!("missing displacement data in CalculiX .dat");
    }
    Ok((
        Some(node_displacements),
        if support_reactions.is_empty() {
            None
        } else {
            Some(support_reactions)
        },
        if element_stresses.is_empty() {
            None
        } else {
            Some(element_stresses.into_values().collect())
        },
    ))
}

fn compare_node_displacements(
    internal_solve: &SolveResult2D,
    calculix_points: &[FrameNodeDisplacementPoint],
) -> FrameNodeDisplacementComparisonSummary {
    let internal_by_id: HashMap<&str, _> = internal_solve
        .node_results
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect();
    let mut max_abs_ux_diff_m = 0.0;
    let mut max_abs_ux_diff_node_id = String::new();
    let mut max_abs_uy_diff_m = 0.0;
    let mut max_abs_uy_diff_node_id = String::new();
    let mut points = Vec::new();

    for point in calculix_points {
        if let Some(internal) = internal_by_id.get(point.node_id.as_str()) {
            let abs_ux_diff_m = (internal.ux_m - point.ux_m).abs();
            let abs_uy_diff_m = (internal.uy_m - point.uy_m).abs();
            if abs_ux_diff_m > max_abs_ux_diff_m {
                max_abs_ux_diff_m = abs_ux_diff_m;
                max_abs_ux_diff_node_id = point.node_id.clone();
            }
            if abs_uy_diff_m > max_abs_uy_diff_m {
                max_abs_uy_diff_m = abs_uy_diff_m;
                max_abs_uy_diff_node_id = point.node_id.clone();
            }
            points.push(FrameNodeDisplacementComparisonPoint {
                node_id: point.node_id.clone(),
                x_m: point.x_m,
                y_m: point.y_m,
                internal_ux_m: internal.ux_m,
                calculix_ux_m: point.ux_m,
                abs_ux_diff_m,
                internal_uy_m: internal.uy_m,
                calculix_uy_m: point.uy_m,
                abs_uy_diff_m,
            });
        }
    }

    FrameNodeDisplacementComparisonSummary {
        node_count: points.len(),
        max_abs_ux_diff_m,
        max_abs_ux_diff_node_id,
        max_abs_uy_diff_m,
        max_abs_uy_diff_node_id,
        points,
    }
}

fn compare_support_reactions(
    internal_solve: &SolveResult2D,
    calculix_points: &[FrameSupportReactionPoint],
) -> FrameSupportReactionComparisonSummary {
    let internal_by_id: HashMap<&str, _> = internal_solve
        .node_results
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect();
    let mut max_abs_fx_diff_n = 0.0;
    let mut max_abs_fx_diff_node_id = String::new();
    let mut max_abs_fy_diff_n = 0.0;
    let mut max_abs_fy_diff_node_id = String::new();
    let mut points = Vec::new();

    for point in calculix_points {
        if let Some(internal) = internal_by_id.get(point.node_id.as_str()) {
            let abs_fx_diff_n = (internal.rxn_fx_n - point.fx_n).abs();
            let abs_fy_diff_n = (internal.rxn_fy_n - point.fy_n).abs();
            if abs_fx_diff_n > max_abs_fx_diff_n {
                max_abs_fx_diff_n = abs_fx_diff_n;
                max_abs_fx_diff_node_id = point.node_id.clone();
            }
            if abs_fy_diff_n > max_abs_fy_diff_n {
                max_abs_fy_diff_n = abs_fy_diff_n;
                max_abs_fy_diff_node_id = point.node_id.clone();
            }
            points.push(FrameSupportReactionComparisonPoint {
                node_id: point.node_id.clone(),
                x_m: point.x_m,
                y_m: point.y_m,
                internal_fx_n: internal.rxn_fx_n,
                calculix_fx_n: point.fx_n,
                abs_fx_diff_n,
                internal_fy_n: internal.rxn_fy_n,
                calculix_fy_n: point.fy_n,
                abs_fy_diff_n,
            });
        }
    }

    FrameSupportReactionComparisonSummary {
        support_count: points.len(),
        max_abs_fx_diff_n,
        max_abs_fx_diff_node_id,
        max_abs_fy_diff_n,
        max_abs_fy_diff_node_id,
        points,
    }
}

fn compare_element_stresses(
    internal_solve: &SolveResult2D,
    calculix_points: &[FrameElementStressSummary],
) -> FrameElementStressComparisonSummary {
    let internal_by_id: HashMap<&str, _> = internal_solve
        .element_results
        .iter()
        .map(|element| (element.id.as_str(), element))
        .collect();
    let mut max_abs_sxx_diff_pa = 0.0;
    let mut max_abs_sxx_diff_element_id = String::new();
    let mut points = Vec::new();

    for point in calculix_points {
        if let Some(internal) = internal_by_id.get(point.element_id.as_str()) {
            let abs_diff_pa = (internal.stress_pa.abs() - point.max_abs_sxx_pa).abs();
            if abs_diff_pa > max_abs_sxx_diff_pa {
                max_abs_sxx_diff_pa = abs_diff_pa;
                max_abs_sxx_diff_element_id = point.element_id.clone();
            }
            points.push(FrameElementStressComparisonPoint {
                element_id: point.element_id.clone(),
                role: point.role.clone(),
                internal_stress_pa: internal.stress_pa.abs(),
                calculix_max_abs_sxx_pa: point.max_abs_sxx_pa,
                abs_diff_pa,
            });
        }
    }

    FrameElementStressComparisonSummary {
        element_count: points.len(),
        max_abs_sxx_diff_pa,
        max_abs_sxx_diff_element_id,
        points,
    }
}

#[cfg(test)]
mod tests {
    use super::{execute_current_frame_project_in_calculix, extract_frame_calculix_dat};
    use crate::{
        create_project, materialize_structural_model_from_builder_graph, portal_frame_builder_graph,
    };
    use std::fs;

    #[test]
    fn extracts_generic_frame_dat_response() {
        let graph = portal_frame_builder_graph(
            "builder.frame.ccx",
            "clear_span",
            "310UB",
            "360UB",
            20.0,
            6.0,
            20.0,
            80.0,
            None,
            None,
        );
        let model = crate::build_frame_model_from_builder_graph(&graph).expect("frame model");
        let dat = r#"
 displacements (vx,vy,vz) for set NALL and time  0.1000000E+01

         1  0.000000E+00  0.000000E+00  0.000000E+00
         2  1.000000E-04 -2.500000E-03  0.000000E+00

 forces (fx,fy,fz) for set SUPPORT_ALL and time  0.1000000E+01

         1  0.000000E+00  1.100000E+05  0.000000E+00
         4  0.000000E+00  1.200000E+05  0.000000E+00

 stresses (elem, integ.pnt.,sxx,syy,szz,sxy,sxz,syz) for set EALL and time  0.1000000E+01

         1   1  2.500000E+07  0.000000E+00  0.000000E+00  1.000000E+05  0.000000E+00  0.000000E+00
         1   2  3.100000E+07  0.000000E+00  0.000000E+00  2.000000E+05  0.000000E+00  0.000000E+00
         2   1  4.200000E+07  0.000000E+00  0.000000E+00  5.000000E+04  0.000000E+00  0.000000E+00
"#;
        let (disp, supports, stresses) = extract_frame_calculix_dat(dat, &model).expect("extract");
        let disp = disp.expect("disp");
        let supports = supports.expect("supports");
        let stresses = stresses.expect("stresses");
        assert_eq!(disp.len(), 2);
        assert_eq!(supports.len(), 2);
        assert_eq!(stresses.len(), 2);
        assert_eq!(disp[1].node_id, "n2");
        assert_eq!(supports[0].fy_n, 110000.0);
        let first_member = stresses
            .iter()
            .find(|stress| stress.point_count == 2)
            .expect("stress summary with two points");
        assert!((first_member.max_abs_sxx_pa - 3.1e7).abs() < 1.0);
    }

    #[test]
    fn execution_fails_clearly_when_ccx_missing() {
        let _env_guard = crate::calculix::CALCULIX_ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let temp_dir = std::env::temp_dir().join(format!(
            "fraia-frame-calculix-run-{}",
            crate::utils::timestamp_id()
        ));
        let (mut project, _) = create_project(&temp_dir, "frame-calculix-run").expect("create");
        let graph = portal_frame_builder_graph(
            "builder.frame.ccx",
            "clear_span",
            "310UB",
            "360UB",
            20.0,
            6.0,
            20.0,
            80.0,
            None,
            None,
        );
        project.builder_graph = Some(graph.clone());
        project.structural_model = materialize_structural_model_from_builder_graph(&graph);

        let original = std::env::var_os("FRAIA_CCX_PATH");
        unsafe {
            std::env::set_var("FRAIA_CCX_PATH", "/definitely/missing/ccx");
        }
        let err = execute_current_frame_project_in_calculix(&project, &temp_dir)
            .expect_err("runtime-unavailable error");
        match original {
            Some(value) => unsafe {
                std::env::set_var("FRAIA_CCX_PATH", value);
            },
            None => unsafe {
                std::env::remove_var("FRAIA_CCX_PATH");
            },
        }
        assert!(
            err.to_string().contains("CalculiX runtime unavailable"),
            "unexpected error: {err:#}"
        );
        let generated_inp_count = fs::read_dir(&temp_dir)
            .expect("read temp dir")
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.file_name().to_string_lossy().starts_with("frame-")
                    && entry.path().extension().and_then(|ext| ext.to_str()) == Some("inp")
            })
            .count();
        assert_eq!(generated_inp_count, 0);

        let _ = fs::remove_dir_all(temp_dir);
    }
}
