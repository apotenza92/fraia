use crate::{
    BeamValueComparison, CalculixCompiledInput, CalculixExecutionArtifacts,
    CalculixExecutionOutcome, ProjectFile, SimplySupportedBeamAnalysisArtifacts,
    SimplySupportedBeamSizingRequest, analyze_current_simply_supported_beam_project,
    compile_frame_model_to_calculix_input, deflection_at_x_mm,
    execute_calculix_compiled_input_with_runtime, require_calculix_runtime, section_by_id,
};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplySupportedBeamCalculixRunManifest {
    pub run_kind: String,
    pub generated_at: String,
    pub project_name: String,
    pub section_id: String,
    pub combo_id: String,
    pub adapter: String,
    pub runtime_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplySupportedBeamCalculixArtifacts {
    pub run: SimplySupportedBeamCalculixRunManifest,
    pub baseline: SimplySupportedBeamAnalysisArtifacts,
    pub compiled_input: CalculixCompiledInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculixBeamResponseSummary {
    pub left_reaction_kn: f64,
    pub right_reaction_kn: f64,
    pub max_deflection_mm: f64,
    pub max_deflection_x_m: f64,
    pub max_abs_sxx_mpa: f64,
    pub max_abs_sxx_x_m: f64,
    pub max_abs_sxx_element_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculixBeamDeflectionPoint {
    pub node_id: String,
    pub x_m: f64,
    pub deflection_mm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculixBeamDeflectionPointComparison {
    pub node_id: String,
    pub x_m: f64,
    pub exact_deflection_mm: f64,
    pub calculix_deflection_mm: f64,
    pub abs_diff_mm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculixBeamDeflectionProfileComparison {
    pub point_count: usize,
    pub max_abs_diff_mm: f64,
    pub max_abs_diff_x_m: f64,
    pub points: Vec<CalculixBeamDeflectionPointComparison>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculixBeamResponseComparison {
    pub left_reaction_kn: BeamValueComparison,
    pub right_reaction_kn: BeamValueComparison,
    pub max_deflection_mm: BeamValueComparison,
    pub max_deflection_x_m: BeamValueComparison,
    pub max_abs_sxx_mpa: BeamValueComparison,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculixBeamElementStressSummary {
    pub element_id: String,
    pub point_count: usize,
    pub max_abs_sxx_pa: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplySupportedBeamCalculixExecutionArtifacts {
    pub run: SimplySupportedBeamCalculixRunManifest,
    pub baseline: SimplySupportedBeamAnalysisArtifacts,
    pub compiled_input: CalculixCompiledInput,
    pub execution: CalculixExecutionArtifacts,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extracted_response: Option<CalculixBeamResponseSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extracted_comparison: Option<CalculixBeamResponseComparison>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extracted_deflection_profile: Option<Vec<CalculixBeamDeflectionPoint>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extracted_element_stresses: Option<Vec<CalculixBeamElementStressSummary>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extracted_deflection_profile_comparison: Option<CalculixBeamDeflectionProfileComparison>,
}

pub fn compile_current_simply_supported_beam_project_to_calculix(
    project: &ProjectFile,
) -> Result<SimplySupportedBeamCalculixArtifacts> {
    let baseline = analyze_current_simply_supported_beam_project(project).context(
        "failed to prepare baseline beam analysis artifacts before CalculiX compilation",
    )?;
    let combo = baseline
        .realization
        .model
        .combos
        .iter()
        .find(|combo| combo.id == baseline.run.combo_id)
        .or_else(|| baseline.realization.model.combos.first())
        .context("no combo available for CalculiX compilation")?;
    let compiled_input = compile_frame_model_to_calculix_input(
        &baseline.realization.model,
        combo,
        &format!("beam-{}", crate::utils::timestamp_id()),
    )
    .context("failed to compile beam realization to CalculiX input")?;

    Ok(SimplySupportedBeamCalculixArtifacts {
        run: SimplySupportedBeamCalculixRunManifest {
            run_kind: "beam-calculix-compile".into(),
            generated_at: crate::utils::iso_now(),
            project_name: project.name.clone(),
            section_id: baseline.run.section_id.clone(),
            combo_id: combo.id.clone(),
            adapter: compiled_input.adapter.clone(),
            runtime_available: compiled_input.runtime.ccx_available,
        },
        baseline,
        compiled_input,
    })
}

pub fn execute_current_simply_supported_beam_project_in_calculix(
    project: &ProjectFile,
    working_dir: &std::path::Path,
) -> Result<SimplySupportedBeamCalculixExecutionArtifacts> {
    let compiled = compile_current_simply_supported_beam_project_to_calculix(project)
        .context("failed to compile current beam project to CalculiX before execution")?;
    let runtime = require_calculix_runtime()?;
    let execution = execute_calculix_compiled_input_with_runtime(
        &compiled.compiled_input,
        working_dir,
        runtime,
    )
    .context("failed to produce CalculiX execution artifacts")?;

    let (extracted_response, extracted_deflection_profile, extracted_element_stresses) =
        if matches!(execution.outcome, CalculixExecutionOutcome::Completed) {
            let dat_path = working_dir.join(format!("{}.dat", compiled.compiled_input.job_name));
            let dat_text = fs::read_to_string(&dat_path)
                .with_context(|| format!("failed to read {}", dat_path.display()))?;
            let extracted =
                extract_calculix_beam_dat_response(&dat_text, &compiled.baseline.realization.model)
                    .context("failed to extract CalculiX beam response from .dat output")?;
            (
                Some(extracted.summary),
                Some(extracted.deflection_profile),
                Some(extracted.element_stresses),
            )
        } else {
            (None, None, None)
        };
    let extracted_comparison =
        extracted_response
            .as_ref()
            .map(|response| CalculixBeamResponseComparison {
                left_reaction_kn: compare_values(
                    compiled.baseline.exact_response.left_reaction_kn,
                    response.left_reaction_kn,
                ),
                right_reaction_kn: compare_values(
                    compiled.baseline.exact_response.right_reaction_kn,
                    response.right_reaction_kn,
                ),
                max_deflection_mm: compare_values(
                    compiled.baseline.exact_response.max_deflection_mm,
                    response.max_deflection_mm,
                ),
                max_deflection_x_m: compare_values(
                    compiled.baseline.exact_response.max_deflection_x_m,
                    response.max_deflection_x_m,
                ),
                max_abs_sxx_mpa: compare_values(
                    compiled.baseline.exact_response.max_bending_stress_mpa,
                    response.max_abs_sxx_mpa,
                ),
            });
    let extracted_deflection_profile_comparison = extracted_deflection_profile
        .as_ref()
        .map(|profile| {
            compare_deflection_profile(
                &compiled.baseline.run.section_id,
                &compiled.baseline.run.request,
                profile,
            )
        })
        .transpose()?;

    Ok(SimplySupportedBeamCalculixExecutionArtifacts {
        run: SimplySupportedBeamCalculixRunManifest {
            run_kind: "beam-calculix-run".into(),
            generated_at: crate::utils::iso_now(),
            project_name: compiled.run.project_name.clone(),
            section_id: compiled.run.section_id.clone(),
            combo_id: compiled.run.combo_id.clone(),
            adapter: execution.adapter.clone(),
            runtime_available: execution.runtime.ccx_available,
        },
        baseline: compiled.baseline,
        compiled_input: compiled.compiled_input,
        execution,
        extracted_response,
        extracted_comparison,
        extracted_deflection_profile,
        extracted_element_stresses,
        extracted_deflection_profile_comparison,
    })
}

fn compare_values(exact: f64, calculix: f64) -> BeamValueComparison {
    BeamValueComparison {
        exact,
        internal: calculix,
        abs_diff: (exact - calculix).abs(),
    }
}

struct CalculixBeamDatExtraction {
    summary: CalculixBeamResponseSummary,
    deflection_profile: Vec<CalculixBeamDeflectionPoint>,
    element_stresses: Vec<CalculixBeamElementStressSummary>,
}

fn extract_calculix_beam_dat_response(
    dat_text: &str,
    model: &crate::FrameModel2D,
) -> Result<CalculixBeamDatExtraction> {
    enum Mode {
        None,
        Displacements,
        LeftReaction,
        RightReaction,
        ElementStress,
    }

    let mut mode = Mode::None;
    let mut left_reaction_kn = None;
    let mut right_reaction_kn = None;
    let mut max_deflection_mm = 0.0;
    let mut max_deflection_x_m = 0.0;
    let mut deflection_profile = Vec::new();
    let mut element_stresses =
        std::collections::HashMap::<String, CalculixBeamElementStressSummary>::new();
    let origin_x_m = model
        .nodes
        .iter()
        .map(|node| node.x)
        .min_by(|a, b| a.total_cmp(b))
        .unwrap_or(0.0);

    for line in dat_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("displacements (vx,vy,vz) for set NALL") {
            mode = Mode::Displacements;
            continue;
        }
        if trimmed.starts_with("forces (fx,fy,fz) for set SUPPORT_LEFT") {
            mode = Mode::LeftReaction;
            continue;
        }
        if trimmed.starts_with("forces (fx,fy,fz) for set SUPPORT_RIGHT") {
            mode = Mode::RightReaction;
            continue;
        }
        if trimmed.starts_with("stresses (elem, integ.pnt.,sxx,syy,szz,sxy,sxz,syz) for set EALL") {
            mode = Mode::ElementStress;
            continue;
        }

        let fields: Vec<&str> = trimmed.split_whitespace().collect();
        match mode {
            Mode::Displacements | Mode::LeftReaction | Mode::RightReaction => {
                if fields.len() < 4 || fields[0].parse::<usize>().is_err() {
                    continue;
                }
                let node_id = fields[0].parse::<usize>()?;
                let fy = fields[2].parse::<f64>()?;
                match mode {
                    Mode::Displacements => {
                        let uy_mm = fy.abs() * 1000.0;
                        let node = model
                            .nodes
                            .get(node_id.saturating_sub(1))
                            .with_context(|| format!("missing model node index {}", node_id))?;
                        let x_m = node.x - origin_x_m;
                        deflection_profile.push(CalculixBeamDeflectionPoint {
                            node_id: node.id.clone(),
                            x_m,
                            deflection_mm: uy_mm,
                        });
                        if uy_mm > max_deflection_mm {
                            max_deflection_mm = uy_mm;
                            max_deflection_x_m = x_m;
                        }
                    }
                    Mode::LeftReaction => {
                        left_reaction_kn = Some(fy / 1000.0);
                    }
                    Mode::RightReaction => {
                        right_reaction_kn = Some(fy / 1000.0);
                    }
                    _ => {}
                }
            }
            Mode::ElementStress => {
                if fields.len() < 8 || fields[0].parse::<usize>().is_err() {
                    continue;
                }
                let element_index = fields[0].parse::<usize>()?;
                let element = model
                    .elements
                    .get(element_index.saturating_sub(1))
                    .with_context(|| format!("missing model element index {}", element_index))?;
                let sxx_pa = fields[2].parse::<f64>()?;
                let entry = element_stresses
                    .entry(element.id.clone())
                    .or_insert_with(|| CalculixBeamElementStressSummary {
                        element_id: element.id.clone(),
                        point_count: 0,
                        max_abs_sxx_pa: 0.0,
                    });
                entry.point_count += 1;
                entry.max_abs_sxx_pa = entry.max_abs_sxx_pa.max(sxx_pa.abs());
            }
            Mode::None => {}
        }
    }

    let left_reaction_kn =
        left_reaction_kn.context("missing SUPPORT_LEFT RF data in CalculiX .dat")?;
    let right_reaction_kn =
        right_reaction_kn.context("missing SUPPORT_RIGHT RF data in CalculiX .dat")?;
    if max_deflection_mm <= 0.0 || deflection_profile.is_empty() {
        bail!("missing displacement data in CalculiX .dat");
    }
    let max_stress = element_stresses
        .values()
        .max_by(|a, b| a.max_abs_sxx_pa.total_cmp(&b.max_abs_sxx_pa))
        .context("missing element stress data in CalculiX .dat")?;
    let max_abs_sxx_element = model
        .elements
        .iter()
        .find(|element| element.id == max_stress.element_id)
        .context("missing beam element for extracted stress summary")?;
    let i_node = model
        .nodes
        .iter()
        .find(|node| node.id == max_abs_sxx_element.i)
        .context("missing beam element start node for extracted stress summary")?;
    let j_node = model
        .nodes
        .iter()
        .find(|node| node.id == max_abs_sxx_element.j)
        .context("missing beam element end node for extracted stress summary")?;
    let max_abs_sxx_x_m = ((i_node.x + j_node.x) * 0.5) - origin_x_m;

    Ok(CalculixBeamDatExtraction {
        summary: CalculixBeamResponseSummary {
            left_reaction_kn,
            right_reaction_kn,
            max_deflection_mm,
            max_deflection_x_m,
            max_abs_sxx_mpa: max_stress.max_abs_sxx_pa / 1e6,
            max_abs_sxx_x_m,
            max_abs_sxx_element_id: max_stress.element_id.clone(),
        },
        deflection_profile,
        element_stresses: element_stresses.into_values().collect(),
    })
}

fn compare_deflection_profile(
    section_id: &str,
    request: &SimplySupportedBeamSizingRequest,
    profile: &[CalculixBeamDeflectionPoint],
) -> Result<CalculixBeamDeflectionProfileComparison> {
    let section = section_by_id(section_id).with_context(|| {
        format!(
            "unknown section {} for exact profile comparison",
            section_id
        )
    })?;
    let mut max_abs_diff_mm = 0.0;
    let mut max_abs_diff_x_m = 0.0;
    let mut points = Vec::new();

    for point in profile {
        let exact_deflection_mm = deflection_at_x_mm(&section, request, point.x_m)?;
        let abs_diff_mm = (exact_deflection_mm - point.deflection_mm).abs();
        if abs_diff_mm > max_abs_diff_mm {
            max_abs_diff_mm = abs_diff_mm;
            max_abs_diff_x_m = point.x_m;
        }
        points.push(CalculixBeamDeflectionPointComparison {
            node_id: point.node_id.clone(),
            x_m: point.x_m,
            exact_deflection_mm,
            calculix_deflection_mm: point.deflection_mm,
            abs_diff_mm,
        });
    }

    Ok(CalculixBeamDeflectionProfileComparison {
        point_count: points.len(),
        max_abs_diff_mm,
        max_abs_diff_x_m,
        points,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        CalculixBeamDeflectionPoint, compare_deflection_profile,
        compile_current_simply_supported_beam_project_to_calculix,
        execute_current_simply_supported_beam_project_in_calculix,
        extract_calculix_beam_dat_response,
    };
    use crate::{
        BuilderNodeParameters, create_project, materialize_structural_model_from_builder_graph,
        seed_simply_supported_beam_in_project,
    };
    use std::fs;

    #[test]
    fn compiles_current_beam_project_to_calculix_with_baseline_artifacts() {
        let temp_dir = std::env::temp_dir().join(format!(
            "fraia-beam-calculix-{}",
            crate::utils::timestamp_id()
        ));
        let (mut project, _) = create_project(&temp_dir, "beam-calculix").expect("create");
        project.requirements.span_m = 6.0;
        project.requirements.gravity_load_kn_per_m = 8.0;
        let node_id = seed_simply_supported_beam_in_project(&mut project, Some("builder.beam.ccx"))
            .expect("seed beam");
        if let Some(graph) = &mut project.builder_graph
            && let Some(node) = graph.nodes.iter_mut().find(|node| node.id == node_id)
            && let BuilderNodeParameters::SimplySupportedBeam2D(params) = &mut node.parameters
        {
            params.section = "250UB".into();
            params.point_load_kn = Some(20.0);
            params.point_load_x_m = Some(3.0);
        }
        if let Some(graph) = &project.builder_graph {
            project.structural_model = materialize_structural_model_from_builder_graph(graph);
        }

        let compiled = compile_current_simply_supported_beam_project_to_calculix(&project)
            .expect("compile calculix artifacts");

        assert_eq!(compiled.run.section_id, "250UB");
        assert_eq!(compiled.run.combo_id, "SLS");
        assert!(
            compiled
                .compiled_input
                .input_deck
                .contains("*ELEMENT,TYPE=B31")
        );
        assert!(
            compiled
                .compiled_input
                .input_deck
                .contains("*NODE,NSET=NALL")
        );
        assert!(compiled.compiled_input.input_deck.contains("*CLOAD"));

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn extracts_beam_response_from_dat_text() {
        let graph = crate::simply_supported_beam_builder_graph(
            "builder.beam.ccx",
            "250UB",
            6.0,
            8.0,
            Some(20.0),
            Some(3.0),
            None,
            None,
        );
        let model = crate::build_frame_model_from_builder_graph(&graph).expect("frame model");
        let dat = r#"
 displacements (vx,vy,vz) for set NALL and time  0.1000000E+01

         1  0.000000E+00  0.000000E+00  0.000000E+00
         2 -9.259087E-17 -6.863864E-03  0.000000E+00
         6 -2.666996E-15 -2.238772E-02  0.000000E+00
        11 -5.557187E-15 -2.465190E-32  0.000000E+00

 forces (fx,fy,fz) for set SUPPORT_LEFT and time  0.1000000E+01

         1  4.615480E-07  3.160000E+04  0.000000E+00

 forces (fx,fy,fz) for set SUPPORT_RIGHT and time  0.1000000E+01

        11 -3.712793E-07  3.160000E+04  0.000000E+00

 stresses (elem, integ.pnt.,sxx,syy,szz,sxy,sxz,syz) for set EALL and time  0.1000000E+01

         5   1  1.520000E+08  0.000000E+00  0.000000E+00  2.500000E+05  0.000000E+00  0.000000E+00
         5   2  1.650000E+08  0.000000E+00  0.000000E+00  2.600000E+05  0.000000E+00  0.000000E+00
"#;
        let extracted = extract_calculix_beam_dat_response(dat, &model).expect("extract");
        assert_eq!(extracted.summary.left_reaction_kn, 31.6);
        assert_eq!(extracted.summary.right_reaction_kn, 31.6);
        assert!((extracted.summary.max_deflection_mm - 22.38772).abs() < 1e-5);
        assert_eq!(extracted.summary.max_deflection_x_m, 3.0);
        assert!((extracted.summary.max_abs_sxx_mpa - 165.0).abs() < 1e-6);
        assert_eq!(extracted.summary.max_abs_sxx_element_id, "m5");
        assert_eq!(extracted.deflection_profile.len(), 4);
        assert_eq!(extracted.element_stresses.len(), 1);
        assert_eq!(extracted.deflection_profile[0].x_m, 0.0);
        assert_eq!(extracted.deflection_profile[2].x_m, 3.0);
    }

    #[test]
    fn compares_deflection_profile_against_exact_reference() {
        let request = crate::SimplySupportedBeamSizingRequest {
            span_m: 6.0,
            distributed_load_kn_per_m: 8.0,
            point_load_kn: Some(20.0),
            point_load_x_m: Some(3.0),
            target_max_utilization: 0.67,
            target_deflection_ratio: 250.0,
        };
        let profile = vec![
            CalculixBeamDeflectionPoint {
                node_id: "n1".into(),
                x_m: 0.0,
                deflection_mm: 0.0,
            },
            CalculixBeamDeflectionPoint {
                node_id: "n6".into(),
                x_m: 3.0,
                deflection_mm: 22.38772,
            },
            CalculixBeamDeflectionPoint {
                node_id: "n11".into(),
                x_m: 6.0,
                deflection_mm: 0.0,
            },
        ];

        let comparison = compare_deflection_profile("250UB", &request, &profile).expect("compare");

        assert_eq!(comparison.point_count, 3);
        assert_eq!(comparison.max_abs_diff_x_m, 3.0);
        assert!((comparison.max_abs_diff_mm - 0.11228).abs() < 1e-3);
    }

    #[test]
    fn execution_fails_clearly_when_ccx_missing() {
        let _env_guard = crate::calculix::CALCULIX_ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let temp_dir = std::env::temp_dir().join(format!(
            "fraia-beam-calculix-run-{}",
            crate::utils::timestamp_id()
        ));
        let (mut project, _) = create_project(&temp_dir, "beam-calculix-run").expect("create");
        project.requirements.span_m = 6.0;
        project.requirements.gravity_load_kn_per_m = 8.0;
        let node_id = seed_simply_supported_beam_in_project(&mut project, Some("builder.beam.ccx"))
            .expect("seed beam");
        if let Some(graph) = &mut project.builder_graph
            && let Some(node) = graph.nodes.iter_mut().find(|node| node.id == node_id)
            && let BuilderNodeParameters::SimplySupportedBeam2D(params) = &mut node.parameters
        {
            params.section = "250UB".into();
        }
        if let Some(graph) = &project.builder_graph {
            project.structural_model = materialize_structural_model_from_builder_graph(graph);
        }

        let original = std::env::var_os("FRAIA_CCX_PATH");
        unsafe {
            std::env::set_var("FRAIA_CCX_PATH", "/definitely/missing/ccx");
        }
        let err = execute_current_simply_supported_beam_project_in_calculix(&project, &temp_dir)
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
                entry.file_name().to_string_lossy().starts_with("beam-")
                    && entry.path().extension().and_then(|ext| ext.to_str()) == Some("inp")
            })
            .count();
        assert_eq!(generated_inp_count, 0);

        let _ = fs::remove_dir_all(temp_dir);
    }
}
