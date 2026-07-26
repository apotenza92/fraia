use crate::{
    Frame2DRealization, ProjectFile, SimplySupportedBeamSizingRequest, SolveResult2D,
    StructuralModel, ValidationReport, analyze_simply_supported_beam,
    current_simply_supported_beam_builder_params, frame2d::solve_frame_2d,
    materialize_project_structural_model, realize_structural_model_to_frame2d, section_by_id,
    steel_material, validate_structural_model,
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplySupportedBeamAnalysisRunManifest {
    pub run_kind: String,
    pub generated_at: String,
    pub project_name: String,
    pub section_id: String,
    pub combo_id: String,
    pub internal_solver: String,
    pub exact_baseline: String,
    pub request: SimplySupportedBeamSizingRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeamResponseSummary {
    pub left_reaction_kn: f64,
    pub right_reaction_kn: f64,
    pub max_shear_kn: f64,
    pub max_moment_knm: f64,
    pub max_moment_x_m: f64,
    pub max_deflection_mm: f64,
    pub max_deflection_x_m: f64,
    pub section_modulus_m3: f64,
    pub max_bending_stress_mpa: f64,
    pub max_utilization: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeamValueComparison {
    pub exact: f64,
    pub internal: f64,
    pub abs_diff: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeamResponseComparison {
    pub left_reaction_kn: BeamValueComparison,
    pub right_reaction_kn: BeamValueComparison,
    pub max_shear_kn: BeamValueComparison,
    pub max_moment_knm: BeamValueComparison,
    pub max_moment_x_m: BeamValueComparison,
    pub max_deflection_mm: BeamValueComparison,
    pub max_deflection_x_m: BeamValueComparison,
    pub max_bending_stress_mpa: BeamValueComparison,
    pub max_utilization: BeamValueComparison,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplySupportedBeamAnalysisArtifacts {
    pub run: SimplySupportedBeamAnalysisRunManifest,
    pub structural_model: StructuralModel,
    pub validation: ValidationReport,
    pub realization: Frame2DRealization,
    pub exact_response: BeamResponseSummary,
    pub internal_solve: SolveResult2D,
    pub internal_response: BeamResponseSummary,
    pub comparison: BeamResponseComparison,
}

pub fn analyze_current_simply_supported_beam_project(
    project: &ProjectFile,
) -> Result<SimplySupportedBeamAnalysisArtifacts> {
    let beam_params = current_simply_supported_beam_builder_params(project)
        .context("no simply supported beam builder node was found in this project")?;
    let section = section_by_id(&beam_params.section)
        .with_context(|| format!("unknown beam section {}", beam_params.section))?;
    let structural_model = materialize_project_structural_model(project).context(
        "no authored structural model or builder-derived structural model was available",
    )?;
    let validation = validate_structural_model(&structural_model);
    let realization = realize_structural_model_to_frame2d(&structural_model)
        .context("failed to realize the current beam structural model to frame2d")?;
    let combo = realization
        .model
        .combos
        .iter()
        .find(|combo| combo.id == "SLS")
        .or_else(|| realization.model.combos.first())
        .context("no frame2d load combination was available for beam analysis")?;
    let internal_solve = solve_frame_2d(&realization.model, combo).with_context(|| {
        format!(
            "failed to solve combo {} with internal frame2d solver",
            combo.id
        )
    })?;

    let request = SimplySupportedBeamSizingRequest {
        span_m: beam_params.span_m,
        distributed_load_kn_per_m: beam_params.distributed_load_kn_per_m,
        point_load_kn: beam_params.point_load_kn,
        point_load_x_m: beam_params.point_load_x_m,
        target_max_utilization: project.requirements.max_utilization,
        target_deflection_ratio: project.requirements.max_deflection_ratio,
    };
    let exact = analyze_simply_supported_beam(&section, &request)
        .context("failed to compute exact simply supported beam baseline")?;

    let exact_response = BeamResponseSummary {
        left_reaction_kn: exact.left_reaction_kn,
        right_reaction_kn: exact.right_reaction_kn,
        max_shear_kn: exact.max_shear_kn,
        max_moment_knm: exact.max_moment_knm,
        max_moment_x_m: exact.max_moment_x_m,
        max_deflection_mm: exact.max_deflection_mm,
        max_deflection_x_m: exact.max_deflection_x_m,
        section_modulus_m3: exact.section_modulus_m3,
        max_bending_stress_mpa: exact.max_bending_stress_mpa,
        max_utilization: (exact.max_bending_stress_mpa * 1e6) / steel_material().fy,
    };
    let internal_response = internal_frame_beam_response(
        &realization,
        &internal_solve,
        exact.section_modulus_m3,
        beam_params.origin_x_m,
    )?;

    Ok(SimplySupportedBeamAnalysisArtifacts {
        run: SimplySupportedBeamAnalysisRunManifest {
            run_kind: "beam-analysis".into(),
            generated_at: crate::utils::iso_now(),
            project_name: project.name.clone(),
            section_id: section.id.clone(),
            combo_id: combo.id.clone(),
            internal_solver: "fraia.frame2d.internal.v1".into(),
            exact_baseline: "fraia.beam.closed_form.v1".into(),
            request,
        },
        structural_model,
        validation,
        realization,
        exact_response: exact_response.clone(),
        internal_solve,
        comparison: BeamResponseComparison {
            left_reaction_kn: compare_values(
                exact_response.left_reaction_kn,
                internal_response.left_reaction_kn,
            ),
            right_reaction_kn: compare_values(
                exact_response.right_reaction_kn,
                internal_response.right_reaction_kn,
            ),
            max_shear_kn: compare_values(
                exact_response.max_shear_kn,
                internal_response.max_shear_kn,
            ),
            max_moment_knm: compare_values(
                exact_response.max_moment_knm,
                internal_response.max_moment_knm,
            ),
            max_moment_x_m: compare_values(
                exact_response.max_moment_x_m,
                internal_response.max_moment_x_m,
            ),
            max_deflection_mm: compare_values(
                exact_response.max_deflection_mm,
                internal_response.max_deflection_mm,
            ),
            max_deflection_x_m: compare_values(
                exact_response.max_deflection_x_m,
                internal_response.max_deflection_x_m,
            ),
            max_bending_stress_mpa: compare_values(
                exact_response.max_bending_stress_mpa,
                internal_response.max_bending_stress_mpa,
            ),
            max_utilization: compare_values(
                exact_response.max_utilization,
                internal_response.max_utilization,
            ),
        },
        internal_response,
    })
}

fn internal_frame_beam_response(
    realization: &Frame2DRealization,
    solve: &SolveResult2D,
    section_modulus_m3: f64,
    origin_x_m: f64,
) -> Result<BeamResponseSummary> {
    let node_by_id: HashMap<&str, _> = realization
        .model
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect();

    let left_support_id = realization
        .model
        .supports
        .iter()
        .min_by(|a, b| {
            let ax = node_by_id
                .get(a.node.as_str())
                .map(|node| node.x)
                .unwrap_or(f64::INFINITY);
            let bx = node_by_id
                .get(b.node.as_str())
                .map(|node| node.x)
                .unwrap_or(f64::INFINITY);
            ax.total_cmp(&bx)
        })
        .map(|support| support.node.as_str())
        .context("beam realization has no left support")?;
    let right_support_id = realization
        .model
        .supports
        .iter()
        .max_by(|a, b| {
            let ax = node_by_id
                .get(a.node.as_str())
                .map(|node| node.x)
                .unwrap_or(f64::NEG_INFINITY);
            let bx = node_by_id
                .get(b.node.as_str())
                .map(|node| node.x)
                .unwrap_or(f64::NEG_INFINITY);
            ax.total_cmp(&bx)
        })
        .map(|support| support.node.as_str())
        .context("beam realization has no right support")?;

    let left_reaction_kn = solve
        .node_results
        .iter()
        .find(|node| node.id == left_support_id)
        .map(|node| node.rxn_fy_n / 1000.0)
        .context("left support result missing from internal solve")?;
    let right_reaction_kn = solve
        .node_results
        .iter()
        .find(|node| node.id == right_support_id)
        .map(|node| node.rxn_fy_n / 1000.0)
        .context("right support result missing from internal solve")?;

    let max_shear_kn = solve
        .element_results
        .iter()
        .flat_map(|element| [element.local_end_forces[1], element.local_end_forces[4]])
        .map(|value| value.abs() / 1000.0)
        .fold(0.0, f64::max);

    let mut max_moment_knm = 0.0;
    let mut max_moment_x_m = 0.0;
    for element in &solve.element_results {
        let frame_element = realization
            .model
            .elements
            .iter()
            .find(|candidate| candidate.id == element.id)
            .with_context(|| format!("element {} missing from realization model", element.id))?;
        let start_x_m = node_by_id
            .get(frame_element.i.as_str())
            .map(|node| node.x - origin_x_m)
            .context("beam element start node missing from realization")?;
        let end_x_m = node_by_id
            .get(frame_element.j.as_str())
            .map(|node| node.x - origin_x_m)
            .context("beam element end node missing from realization")?;
        let start_moment_knm = element.local_end_forces[2].abs() / 1000.0;
        if start_moment_knm > max_moment_knm {
            max_moment_knm = start_moment_knm;
            max_moment_x_m = start_x_m;
        }
        let end_moment_knm = element.local_end_forces[5].abs() / 1000.0;
        if end_moment_knm > max_moment_knm {
            max_moment_knm = end_moment_knm;
            max_moment_x_m = end_x_m;
        }
    }

    let (max_deflection_x_m, max_deflection_mm) = solve
        .node_results
        .iter()
        .map(|node| (node.x - origin_x_m, node.uy_m.abs() * 1000.0))
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .context("internal solve contained no node results")?;

    let max_bending_stress_mpa = solve
        .element_results
        .iter()
        .map(|element| element.stress_pa.abs() / 1e6)
        .fold(0.0, f64::max);
    let max_utilization = solve
        .element_results
        .iter()
        .map(|element| element.utilization)
        .fold(0.0, f64::max);

    Ok(BeamResponseSummary {
        left_reaction_kn,
        right_reaction_kn,
        max_shear_kn,
        max_moment_knm,
        max_moment_x_m,
        max_deflection_mm,
        max_deflection_x_m,
        section_modulus_m3,
        max_bending_stress_mpa,
        max_utilization,
    })
}

fn compare_values(exact: f64, internal: f64) -> BeamValueComparison {
    BeamValueComparison {
        exact,
        internal,
        abs_diff: (exact - internal).abs(),
    }
}

#[cfg(test)]
mod tests {
    use super::analyze_current_simply_supported_beam_project;
    use crate::{
        BuilderNodeParameters, create_project, materialize_structural_model_from_builder_graph,
        seed_simply_supported_beam_in_project,
    };
    use std::fs;

    #[test]
    fn standard_demo_beam_analysis_artifacts_track_exact_and_internal_values() {
        let temp_dir = std::env::temp_dir().join(format!(
            "fraia-beam-analysis-{}",
            crate::utils::timestamp_id()
        ));
        let (mut project, _) = create_project(&temp_dir, "beam-analysis").expect("create");
        project.requirements.span_m = 6.0;
        project.requirements.gravity_load_kn_per_m = 8.0;
        project.requirements.max_utilization = 0.67;
        project.requirements.max_deflection_ratio = 250.0;
        let node_id =
            seed_simply_supported_beam_in_project(&mut project, Some("builder.beam.analysis"))
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

        let run = analyze_current_simply_supported_beam_project(&project).expect("analyze");

        assert_eq!(run.run.section_id, "250UB");
        assert_eq!(run.run.combo_id, "SLS");
        assert!(run.validation.diagnostics.is_empty());
        assert!(run.realization.diagnostics.is_empty());
        assert!(run.comparison.left_reaction_kn.abs_diff < 1e-6);
        assert!(run.comparison.right_reaction_kn.abs_diff < 1e-6);
        assert!(run.comparison.max_moment_knm.abs_diff < 1e-6);
        assert!(run.comparison.max_bending_stress_mpa.abs_diff < 1e-3);
        assert!(run.comparison.max_utilization.abs_diff < 1e-6);
        assert!(run.comparison.max_deflection_mm.abs_diff < 0.2);
        assert_eq!(run.exact_response.max_deflection_x_m, 3.0);
        assert_eq!(run.internal_response.max_deflection_x_m, 3.0);

        let _ = fs::remove_dir_all(temp_dir);
    }
}
