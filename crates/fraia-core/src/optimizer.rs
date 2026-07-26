use crate::archetypes::{build_frame_model, topologies};
use crate::catalog::{section_by_id, section_catalog, steel_material};
use crate::engineering::{
    ActionableEngineeringWarning, ConnectionDemand, derive_engineering_summary,
};
use crate::frame2d::solve_frame_2d;
use crate::project::{ProjectPaths, load_project};
use crate::realization::{Frame2DRealization, realize_structural_model_to_frame2d};
use crate::structural_app::StructuralModel;
use crate::types::{
    CandidateOption, FrameElement2D, FrameModel2D, OptimizationRun, ProjectFile, Section, Topology,
};
use crate::utils::{ensure_dir, format_number, iso_now, round3, sum, timestamp_id, write_json};
use crate::validate::{ValidationReport, validate_structural_model};
use anyhow::Result;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub run_id: String,
    pub run_dir: PathBuf,
    pub ranked: Vec<CandidateOption>,
    pub selected: Vec<CandidateOption>,
    pub infeasible_count: usize,
    pub project: ProjectFile,
    pub paths: ProjectPaths,
}

pub fn run_optimization(project_dir: &Path) -> Result<OptimizationResult> {
    let (project, paths) = load_project(project_dir)?;
    let req = &project.requirements;
    let intent = &project.intent;
    let sections = section_catalog();
    let topologies: Vec<Topology> = topologies()
        .into_iter()
        .filter(|topology| {
            (intent.search_permissions.change_topology || topology.id == "clear_span")
                && (intent.search_permissions.add_internal_columns
                    || topology.internal_columns == 0)
                && topology.internal_columns <= req.max_internal_columns
        })
        .collect();

    let mut candidates = Vec::new();

    for topology in &topologies {
        for beam_section in &sections {
            for column_section in &sections {
                let model = build_frame_model(
                    &topology.id,
                    req.span_m,
                    req.height_m,
                    beam_section,
                    column_section,
                    req.gravity_load_kn_per_m,
                    req.lateral_load_kn,
                );

                match evaluate_candidate(&project, &model, beam_section, column_section) {
                    Ok(candidate) => candidates.push(candidate),
                    Err(error) => candidates.push(CandidateOption {
                        feasible: false,
                        topology: topology.name.clone(),
                        topology_id: topology.id.clone(),
                        internal_columns: topology.internal_columns,
                        beam_section: beam_section.id.clone(),
                        column_section: column_section.id.clone(),
                        mass_kg: 0.0,
                        cost: 0.0,
                        carbon: 0.0,
                        max_utilization: f64::INFINITY,
                        max_deflection_mm: f64::INFINITY,
                        max_drift_mm: f64::INFINITY,
                        deflection_ratio: None,
                        drift_ratio: None,
                        score: f64::INFINITY,
                        summary: error.to_string(),
                        tradeoffs: vec![
                            "Model was unstable or unsupported for this configuration.".into(),
                        ],
                        combo_metrics: BTreeMap::new(),
                    }),
                }
            }
        }
    }

    let mut feasible: Vec<CandidateOption> =
        candidates.into_iter().filter(|c| c.feasible).collect();
    feasible.sort_by(|a, b| {
        a.score
            .partial_cmp(&b.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let selected = diversify(&feasible, intent.option_count);

    let run_id = timestamp_id();
    let run_dir = paths.runs_dir.join(&run_id);
    ensure_dir(&run_dir)?;
    let generated_at = iso_now();
    let run = OptimizationRun {
        run_id: run_id.clone(),
        generated_at: generated_at.clone(),
        project_name: project.name.clone(),
        project_intent: project.intent.clone(),
        requirements: project.requirements.clone(),
        option_count: selected.len(),
        options: selected.clone(),
    };
    write_json(&run_dir.join("options.json"), &run)?;
    persist_run_artifacts(&run_dir, &run_id, &generated_at, &project, &selected)?;

    Ok(OptimizationResult {
        run_id,
        run_dir,
        ranked: feasible.clone(),
        selected,
        infeasible_count: topologies.len() * sections.len() * sections.len() - feasible.len(),
        project,
        paths,
    })
}

fn evaluate_candidate(
    project: &ProjectFile,
    model: &FrameModel2D,
    beam_section: &Section,
    column_section: &Section,
) -> Result<CandidateOption> {
    let sls_combo = model
        .combos
        .iter()
        .find(|c| c.id == "SLS")
        .expect("SLS combo missing");
    let uls_combo = model
        .combos
        .iter()
        .find(|c| c.id == "ULS")
        .expect("ULS combo missing");
    let sls = solve_frame_2d(model, sls_combo)?;
    let uls = solve_frame_2d(model, uls_combo)?;

    let top_node_ids: Vec<&str> = model
        .nodes
        .iter()
        .filter(|n| (n.y - project.requirements.height_m).abs() < 1e-9)
        .map(|n| n.id.as_str())
        .collect();

    let beam_node_ids: Vec<&str> = model
        .elements
        .iter()
        .filter(|e| e.role == "beam")
        .flat_map(|e| [e.i.as_str(), e.j.as_str()])
        .collect();

    let max_drift = sls
        .node_results
        .iter()
        .filter(|n| top_node_ids.contains(&n.id.as_str()))
        .map(|n| n.ux_m.abs())
        .fold(0.0, f64::max);

    let max_deflection = sls
        .node_results
        .iter()
        .filter(|n| beam_node_ids.contains(&n.id.as_str()))
        .map(|n| n.uy_m.abs())
        .fold(0.0, f64::max);

    let max_utilization = uls
        .element_results
        .iter()
        .map(|e| e.utilization)
        .fold(0.0, f64::max);

    let mass_kg = sum(&model
        .elements
        .iter()
        .map(|e| e.section.mass_kg_per_m * element_length(model, e))
        .collect::<Vec<_>>());
    let steel = steel_material();
    let cost = mass_kg * steel.cost_per_kg;
    let carbon = mass_kg * steel.carbon_per_kg;
    let internal_columns = model.topology.internal_columns;
    let deflection_ratio = if max_deflection > 0.0 {
        Some((project.requirements.span_m / max_deflection).round() as u64)
    } else {
        None
    };
    let drift_ratio = if max_drift > 0.0 {
        Some((project.requirements.height_m / max_drift).round() as u64)
    } else {
        None
    };

    let feasible = max_utilization <= project.requirements.max_utilization
        && deflection_ratio.unwrap_or(u64::MAX) as f64 >= project.requirements.max_deflection_ratio
        && drift_ratio.unwrap_or(u64::MAX) as f64 >= project.requirements.max_drift_ratio;

    let score = objective_score(
        &project.intent.objective_priority,
        cost,
        carbon,
        max_utilization,
        internal_columns,
    );

    Ok(CandidateOption {
        feasible,
        topology: model.topology.name.clone(),
        topology_id: model.topology.id.clone(),
        internal_columns,
        beam_section: beam_section.id.clone(),
        column_section: column_section.id.clone(),
        mass_kg: round3(mass_kg),
        cost: round3(cost),
        carbon: round3(carbon),
        max_utilization: round3(max_utilization),
        max_deflection_mm: round3(max_deflection * 1000.0),
        max_drift_mm: round3(max_drift * 1000.0),
        deflection_ratio,
        drift_ratio,
        score,
        summary: format!(
            "{}; beam {}, columns {}; cost proxy {}; max utilization {}.",
            model.topology.name,
            beam_section.id,
            column_section.id,
            format_number(cost, 0),
            format_number(max_utilization, 2)
        ),
        tradeoffs: tradeoffs(&model.topology, cost, carbon),
        combo_metrics: BTreeMap::from([
            ("SLS".into(), sls.metrics.clone()),
            ("ULS".into(), uls.metrics.clone()),
        ]),
    })
}

fn objective_score(
    priority: &str,
    cost: f64,
    carbon: f64,
    max_utilization: f64,
    internal_columns: usize,
) -> f64 {
    let normalized_cost = cost / 1000.0;
    let normalized_carbon = carbon / 1000.0;
    let normalized_util = max_utilization * 100.0;
    let normalized_internal = internal_columns as f64 * 20.0;
    match priority {
        "minimize_cost" => normalized_cost + normalized_internal * 0.2 + normalized_util,
        "low_carbon" => normalized_carbon + normalized_internal * 0.2 + normalized_util,
        "clear_span" => normalized_internal * 5.0 + normalized_cost * 0.2 + normalized_util,
        _ => {
            normalized_cost * 0.6
                + normalized_carbon * 0.3
                + normalized_internal * 0.6
                + normalized_util
        }
    }
}

fn tradeoffs(topology: &Topology, cost: f64, carbon: f64) -> Vec<String> {
    let mut out = Vec::new();
    if topology.id == "clear_span" {
        out.push("Preserves open internal space but generally demands heavier members.".into());
    }
    if topology.internal_columns > 0 {
        out.push("Introduces internal supports to reduce spans and steel demand.".into());
    }
    if cost < 15_000.0 {
        out.push("Lower mass/cost proxy in this demo search.".into());
    }
    if carbon < 8_000.0 {
        out.push("Lower carbon proxy in this demo search.".into());
    }
    out
}

fn element_length(model: &FrameModel2D, element: &FrameElement2D) -> f64 {
    let i = model.nodes.iter().find(|n| n.id == element.i).unwrap();
    let j = model.nodes.iter().find(|n| n.id == element.j).unwrap();
    ((j.x - i.x).powi(2) + (j.y - i.y).powi(2)).sqrt()
}

#[derive(Debug, Clone, Serialize)]
struct RunManifest {
    run_id: String,
    generated_at: String,
    kind: String,
    project_name: String,
    artifact_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct OptionSnapshot {
    option: CandidateOption,
    authored_structural_model: StructuralModel,
    realized_frame_model: FrameModel2D,
}

#[derive(Debug, Clone, Serialize)]
struct OptionDiagnosticsSnapshot {
    option_topology_id: String,
    option_beam_section: String,
    option_column_section: String,
    validation: ValidationReport,
    realization: Vec<crate::realization::RealizationDiagnostic>,
    engineering_warnings: Vec<ActionableEngineeringWarning>,
    connection_demands: Vec<ConnectionDemand>,
}

fn persist_run_artifacts(
    run_dir: &Path,
    run_id: &str,
    generated_at: &str,
    project: &ProjectFile,
    selected: &[CandidateOption],
) -> Result<()> {
    let manifest = RunManifest {
        run_id: run_id.into(),
        generated_at: generated_at.into(),
        kind: "optimization".into(),
        project_name: project.name.clone(),
        artifact_files: vec![
            "run.json".into(),
            "options.json".into(),
            "snapshot.json".into(),
            "diagnostics.json".into(),
        ],
    };
    write_json(&run_dir.join("run.json"), &manifest)?;

    let mut snapshots = Vec::new();
    let mut diagnostics = Vec::new();

    for option in selected {
        let Some(beam) = section_by_id(&option.beam_section) else {
            continue;
        };
        let Some(column) = section_by_id(&option.column_section) else {
            continue;
        };
        let frame_model = build_frame_model(
            &option.topology_id,
            project.requirements.span_m,
            project.requirements.height_m,
            &beam,
            &column,
            project.requirements.gravity_load_kn_per_m,
            project.requirements.lateral_load_kn,
        );
        let structural_model = StructuralModel::from_frame2d(&frame_model);
        let validation = validate_structural_model(&structural_model);
        let realization: Frame2DRealization =
            realize_structural_model_to_frame2d(&structural_model)?;
        let engineering = derive_engineering_summary(project, &frame_model)?;

        snapshots.push(OptionSnapshot {
            option: option.clone(),
            authored_structural_model: structural_model.clone(),
            realized_frame_model: realization.model,
        });
        diagnostics.push(OptionDiagnosticsSnapshot {
            option_topology_id: option.topology_id.clone(),
            option_beam_section: option.beam_section.clone(),
            option_column_section: option.column_section.clone(),
            validation,
            realization: realization.diagnostics,
            engineering_warnings: engineering.warnings,
            connection_demands: engineering.connection_demands,
        });
    }

    write_json(&run_dir.join("snapshot.json"), &snapshots)?;
    write_json(&run_dir.join("diagnostics.json"), &diagnostics)?;
    Ok(())
}

fn diversify(ranked: &[CandidateOption], count: usize) -> Vec<CandidateOption> {
    let mut selected = Vec::new();
    let mut seen_topology = Vec::<String>::new();

    for candidate in ranked {
        if !seen_topology.contains(&candidate.topology_id) {
            selected.push(candidate.clone());
            seen_topology.push(candidate.topology_id.clone());
            if selected.len() >= count {
                return selected;
            }
        }
    }

    for candidate in ranked {
        if selected.iter().any(|existing| {
            existing.topology_id == candidate.topology_id
                && existing.beam_section == candidate.beam_section
                && existing.column_section == candidate.column_section
        }) {
            continue;
        }
        selected.push(candidate.clone());
        if selected.len() >= count {
            return selected;
        }
    }

    selected
}

#[cfg(test)]
mod tests {
    use super::run_optimization;
    use crate::project::create_project;
    use crate::utils::timestamp_id;
    use std::fs;

    #[test]
    fn optimization_writes_run_artifacts() {
        let temp_dir = std::env::temp_dir().join(format!("fraia-opt-test-{}", timestamp_id()));
        create_project(&temp_dir, "optimizer-test").expect("create project");

        let result = run_optimization(&temp_dir).expect("run optimization");
        assert!(result.run_dir.join("run.json").exists());
        assert!(result.run_dir.join("options.json").exists());
        assert!(result.run_dir.join("snapshot.json").exists());
        assert!(result.run_dir.join("diagnostics.json").exists());

        let snapshot =
            fs::read_to_string(result.run_dir.join("snapshot.json")).expect("read snapshot");
        let diagnostics =
            fs::read_to_string(result.run_dir.join("diagnostics.json")).expect("read diagnostics");
        assert!(snapshot.contains("authored_structural_model"));
        assert!(diagnostics.contains("option_topology_id"));
        assert!(diagnostics.contains("validation"));

        let _ = fs::remove_dir_all(temp_dir);
    }
}
