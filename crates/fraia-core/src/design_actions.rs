use crate::frame2d::solve_frame_2d;
use crate::types::{FrameModel2D, ProjectFile};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberDesignActions {
    pub member_id: String,
    pub role: String,
    pub section_id: String,
    pub length_m: f64,
    pub max_axial_n: f64,
    pub governing_axial_combo_id: String,
    pub max_shear_n: f64,
    pub governing_shear_combo_id: String,
    pub max_moment_nm: f64,
    pub governing_moment_combo_id: String,
    pub max_utilization: f64,
    pub governing_utilization_combo_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportReactionSummary {
    pub support_node_id: String,
    pub max_fx_n: f64,
    pub governing_fx_combo_id: String,
    pub max_fy_n: f64,
    pub governing_fy_combo_id: String,
    pub max_mz_nm: f64,
    pub governing_mz_combo_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalServiceabilitySummary {
    pub reference_height_m: f64,
    pub max_drift_m: f64,
    pub drift_ratio: Option<f64>,
    pub governing_drift_combo_id: Option<String>,
    pub reference_span_m: f64,
    pub max_deflection_m: f64,
    pub deflection_ratio: Option<f64>,
    pub governing_deflection_combo_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DesignActionReport {
    pub member_actions: Vec<MemberDesignActions>,
    pub support_reactions: Vec<SupportReactionSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub global_serviceability: Option<GlobalServiceabilitySummary>,
}

pub fn derive_design_action_report(
    _project: &ProjectFile,
    model: &FrameModel2D,
) -> Result<DesignActionReport> {
    let combo_results: Vec<_> = model
        .combos
        .iter()
        .map(|combo| solve_frame_2d(model, combo))
        .collect::<Result<Vec<_>>>()?;

    let mut member_actions: HashMap<String, MemberDesignActions> = HashMap::new();
    for result in &combo_results {
        for element in &result.element_results {
            let local = &element.local_end_forces;
            let shear = if local.len() >= 5 {
                local[1].abs().max(local[4].abs())
            } else {
                0.0
            };
            let entry =
                member_actions
                    .entry(element.id.clone())
                    .or_insert_with(|| MemberDesignActions {
                        member_id: element.id.clone(),
                        role: element.role.clone(),
                        section_id: model
                            .elements
                            .iter()
                            .find(|candidate| candidate.id == element.id)
                            .map(|candidate| candidate.section.id.clone())
                            .unwrap_or_default(),
                        length_m: element.length_m,
                        max_axial_n: 0.0,
                        governing_axial_combo_id: result.combo.id.clone(),
                        max_shear_n: 0.0,
                        governing_shear_combo_id: result.combo.id.clone(),
                        max_moment_nm: 0.0,
                        governing_moment_combo_id: result.combo.id.clone(),
                        max_utilization: 0.0,
                        governing_utilization_combo_id: result.combo.id.clone(),
                    });
            if element.axial_n > entry.max_axial_n {
                entry.max_axial_n = element.axial_n;
                entry.governing_axial_combo_id = result.combo.id.clone();
            }
            if shear > entry.max_shear_n {
                entry.max_shear_n = shear;
                entry.governing_shear_combo_id = result.combo.id.clone();
            }
            if element.moment_nm > entry.max_moment_nm {
                entry.max_moment_nm = element.moment_nm;
                entry.governing_moment_combo_id = result.combo.id.clone();
            }
            if element.utilization > entry.max_utilization {
                entry.max_utilization = element.utilization;
                entry.governing_utilization_combo_id = result.combo.id.clone();
            }
        }
    }

    let mut support_reactions: HashMap<String, SupportReactionSummary> = HashMap::new();
    for result in &combo_results {
        for support in &model.supports {
            if let Some(node) = result
                .node_results
                .iter()
                .find(|node| node.id == support.node)
            {
                let entry = support_reactions
                    .entry(support.node.clone())
                    .or_insert_with(|| SupportReactionSummary {
                        support_node_id: support.node.clone(),
                        max_fx_n: 0.0,
                        governing_fx_combo_id: result.combo.id.clone(),
                        max_fy_n: 0.0,
                        governing_fy_combo_id: result.combo.id.clone(),
                        max_mz_nm: 0.0,
                        governing_mz_combo_id: result.combo.id.clone(),
                    });
                if node.rxn_fx_n.abs() > entry.max_fx_n.abs() {
                    entry.max_fx_n = node.rxn_fx_n;
                    entry.governing_fx_combo_id = result.combo.id.clone();
                }
                if node.rxn_fy_n.abs() > entry.max_fy_n.abs() {
                    entry.max_fy_n = node.rxn_fy_n;
                    entry.governing_fy_combo_id = result.combo.id.clone();
                }
                if node.rxn_mz_nm.abs() > entry.max_mz_nm.abs() {
                    entry.max_mz_nm = node.rxn_mz_nm;
                    entry.governing_mz_combo_id = result.combo.id.clone();
                }
            }
        }
    }

    let global_serviceability = derive_global_serviceability_summary(model, &combo_results);

    Ok(DesignActionReport {
        member_actions: member_actions.into_values().collect(),
        support_reactions: support_reactions.into_values().collect(),
        global_serviceability,
    })
}

fn derive_global_serviceability_summary(
    model: &FrameModel2D,
    combo_results: &[crate::types::SolveResult2D],
) -> Option<GlobalServiceabilitySummary> {
    if combo_results.is_empty() || model.nodes.is_empty() {
        return None;
    }

    let min_y = model
        .nodes
        .iter()
        .map(|node| node.y)
        .fold(f64::INFINITY, f64::min);
    let max_y = model
        .nodes
        .iter()
        .map(|node| node.y)
        .fold(f64::NEG_INFINITY, f64::max);
    let top_node_ids: Vec<&str> = model
        .nodes
        .iter()
        .filter(|node| (node.y - max_y).abs() < 1e-9)
        .map(|node| node.id.as_str())
        .collect();
    let beam_node_ids: Vec<&str> = model
        .elements
        .iter()
        .filter(|element| element.role == "beam")
        .flat_map(|element| [element.i.as_str(), element.j.as_str()])
        .collect();

    let mut max_drift_m = 0.0;
    let mut governing_drift_combo_id = None;
    let mut max_deflection_m = 0.0;
    let mut governing_deflection_combo_id = None;

    for result in combo_results {
        let drift = result
            .node_results
            .iter()
            .filter(|node| top_node_ids.contains(&node.id.as_str()))
            .map(|node| node.ux_m.abs())
            .fold(0.0, f64::max);
        if drift > max_drift_m {
            max_drift_m = drift;
            governing_drift_combo_id = Some(result.combo.id.clone());
        }

        let deflection = result
            .node_results
            .iter()
            .filter(|node| beam_node_ids.contains(&node.id.as_str()))
            .map(|node| node.uy_m.abs())
            .fold(0.0, f64::max);
        if deflection > max_deflection_m {
            max_deflection_m = deflection;
            governing_deflection_combo_id = Some(result.combo.id.clone());
        }
    }

    let reference_height_m = (max_y - min_y).abs().max(1e-9);
    let beam_points: Vec<(f64, f64)> = model
        .nodes
        .iter()
        .filter(|node| beam_node_ids.contains(&node.id.as_str()))
        .map(|node| (node.x, node.y))
        .collect();
    let reference_span_m = beam_points
        .iter()
        .enumerate()
        .flat_map(|(i, start)| {
            beam_points
                .iter()
                .skip(i + 1)
                .map(move |end| ((end.0 - start.0).powi(2) + (end.1 - start.1).powi(2)).sqrt())
        })
        .fold(0.0, f64::max)
        .max(1e-9);

    Some(GlobalServiceabilitySummary {
        reference_height_m,
        max_drift_m,
        drift_ratio: if max_drift_m > 0.0 {
            Some(reference_height_m / max_drift_m)
        } else {
            None
        },
        governing_drift_combo_id,
        reference_span_m,
        max_deflection_m,
        deflection_ratio: if max_deflection_m > 0.0 {
            Some(reference_span_m / max_deflection_m)
        } else {
            None
        },
        governing_deflection_combo_id,
    })
}

#[cfg(test)]
mod tests {
    use super::derive_design_action_report;
    use crate::archetypes::{
        build_frame_model, build_frame_model_from_builder_graph,
        simply_supported_beam_builder_graph,
    };
    use crate::catalog::section_by_id;
    use crate::project::create_project;
    use crate::utils::timestamp_id;
    use std::fs;

    #[test]
    fn design_action_report_extracts_member_and_support_summaries() {
        let temp_dir =
            std::env::temp_dir().join(format!("fraia-design-actions-test-{}", timestamp_id()));
        let (project, _) =
            create_project(&temp_dir, "design-actions-test").expect("create project");
        let beam = section_by_id("310UB").unwrap();
        let column = section_by_id("310UB").unwrap();
        let model = build_frame_model("clear_span", 20.0, 6.0, &beam, &column, 20.0, 80.0);
        let report = derive_design_action_report(&project, &model).expect("derive design actions");

        assert!(!report.member_actions.is_empty());
        assert!(!report.support_reactions.is_empty());
        assert!(report.global_serviceability.is_some());

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn global_serviceability_uses_model_geometry_not_project_requirement_lengths() {
        let temp_dir =
            std::env::temp_dir().join(format!("fraia-design-actions-geom-{}", timestamp_id()));
        let (mut project, _) =
            create_project(&temp_dir, "design-actions-geom").expect("create project");
        project.requirements.span_m = 999.0;
        project.requirements.height_m = 999.0;
        let beam = section_by_id("310UB").unwrap();
        let column = section_by_id("310UB").unwrap();
        let model = build_frame_model("clear_span", 20.0, 6.0, &beam, &column, 20.0, 80.0);
        let report = derive_design_action_report(&project, &model).expect("derive design actions");
        let serviceability = report
            .global_serviceability
            .expect("serviceability summary");

        assert!(serviceability.reference_height_m < 100.0);
        assert!(serviceability.reference_span_m < 100.0);

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn segmented_beam_serviceability_uses_full_beam_span_not_one_element_length() {
        let temp_dir =
            std::env::temp_dir().join(format!("fraia-design-actions-beam-{}", timestamp_id()));
        let (project, _) =
            create_project(&temp_dir, "design-actions-beam").expect("create project");
        let graph = simply_supported_beam_builder_graph(
            "beam-1",
            "250UB",
            6.0,
            8.0,
            Some(20.0),
            Some(3.0),
            None,
            None,
        );
        let model = build_frame_model_from_builder_graph(&graph).expect("build beam model");
        let report = derive_design_action_report(&project, &model).expect("derive design actions");
        let serviceability = report
            .global_serviceability
            .expect("serviceability summary");

        assert!(serviceability.reference_span_m > 5.9);
        assert!(serviceability.reference_span_m < 6.1);

        let _ = fs::remove_dir_all(temp_dir);
    }
}
