use crate::frame2d::solve_frame_2d;
use crate::types::{FrameModel2D, ProjectFile, SolveResult2D};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionableEngineeringWarning {
    pub id: String,
    pub severity: String,
    pub object_refs: Vec<String>,
    pub message: String,
    pub suggested_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionDemand {
    pub id: String,
    pub member_id: String,
    pub end: String,
    pub axial_n: f64,
    pub shear_n: f64,
    pub moment_nm: f64,
    pub likely_connection_family: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EngineeringSummary {
    pub warnings: Vec<ActionableEngineeringWarning>,
    pub connection_demands: Vec<ConnectionDemand>,
}

pub fn derive_engineering_summary(
    project: &ProjectFile,
    model: &FrameModel2D,
) -> anyhow::Result<EngineeringSummary> {
    let sls_combo = model
        .combos
        .iter()
        .find(|combo| combo.id == "SLS")
        .or_else(|| model.combos.first())
        .ok_or_else(|| anyhow::anyhow!("model has no load combinations"))?;
    let uls_combo = model
        .combos
        .iter()
        .find(|combo| combo.id == "ULS")
        .or_else(|| model.combos.first())
        .ok_or_else(|| anyhow::anyhow!("model has no load combinations"))?;

    let sls = solve_frame_2d(model, sls_combo)?;
    let uls = solve_frame_2d(model, uls_combo)?;
    Ok(derive_engineering_summary_from_results(
        project, model, &sls, &uls,
    ))
}

pub fn derive_engineering_summary_from_results(
    project: &ProjectFile,
    model: &FrameModel2D,
    sls: &SolveResult2D,
    uls: &SolveResult2D,
) -> EngineeringSummary {
    let mut summary = EngineeringSummary::default();

    for element in &uls.element_results {
        if element.utilization > project.requirements.max_utilization * 0.9 {
            summary.warnings.push(ActionableEngineeringWarning {
                id: format!("warning-utilization-{}", element.id),
                severity: if element.utilization > project.requirements.max_utilization {
                    "warning".into()
                } else {
                    "info".into()
                },
                object_refs: vec![format!("member:{}", element.id)],
                message: format!(
                    "Member {} utilization {:.3} is close to or above the current target limit {:.3}.",
                    element.id, element.utilization, project.requirements.max_utilization
                ),
                suggested_actions: vec![
                    "Increase the member section size.".into(),
                    "Reduce span/demand or add supports if appropriate.".into(),
                ],
            });
        }

        let local = &element.local_end_forces;
        if local.len() >= 6 {
            summary.connection_demands.push(ConnectionDemand {
                id: format!("connection-demand-{}-start", element.id),
                member_id: element.id.clone(),
                end: "start".into(),
                axial_n: local[0],
                shear_n: local[1],
                moment_nm: local[2],
                likely_connection_family: likely_connection_family(local[1], local[2]),
            });
            summary.connection_demands.push(ConnectionDemand {
                id: format!("connection-demand-{}-end", element.id),
                member_id: element.id.clone(),
                end: "end".into(),
                axial_n: local[3],
                shear_n: local[4],
                moment_nm: local[5],
                likely_connection_family: likely_connection_family(local[4], local[5]),
            });
        }
    }

    let max_y = model
        .nodes
        .iter()
        .map(|node| node.y)
        .fold(f64::NEG_INFINITY, f64::max);
    let min_y = model
        .nodes
        .iter()
        .map(|node| node.y)
        .fold(f64::INFINITY, f64::min);
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

    let max_drift = sls
        .node_results
        .iter()
        .filter(|node| top_node_ids.contains(&node.id.as_str()))
        .map(|node| node.ux_m.abs())
        .fold(0.0, f64::max);
    let max_deflection = sls
        .node_results
        .iter()
        .filter(|node| beam_node_ids.contains(&node.id.as_str()))
        .map(|node| node.uy_m.abs())
        .fold(0.0, f64::max);

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

    let drift_ratio = if max_drift > 0.0 {
        reference_height_m / max_drift
    } else {
        f64::INFINITY
    };
    let deflection_ratio = if max_deflection > 0.0 {
        reference_span_m / max_deflection
    } else {
        f64::INFINITY
    };

    if drift_ratio < project.requirements.max_drift_ratio * 1.1 {
        summary.warnings.push(ActionableEngineeringWarning {
            id: "warning-drift".into(),
            severity: if drift_ratio < project.requirements.max_drift_ratio {
                "warning".into()
            } else {
                "info".into()
            },
            object_refs: top_node_ids
                .iter()
                .map(|id| format!("node:{}", id))
                .collect(),
            message: format!(
                "Global drift ratio H/{:.0} is close to or beyond the target limit H/{:.0}.",
                drift_ratio, project.requirements.max_drift_ratio
            ),
            suggested_actions: vec![
                "Increase lateral stiffness or adjust topology.".into(),
                "Review frame action, bracing, or internal support strategy.".into(),
            ],
        });
    }

    if deflection_ratio < project.requirements.max_deflection_ratio * 1.1 {
        summary.warnings.push(ActionableEngineeringWarning {
            id: "warning-deflection".into(),
            severity: if deflection_ratio < project.requirements.max_deflection_ratio {
                "warning".into()
            } else {
                "info".into()
            },
            object_refs: model
                .elements
                .iter()
                .filter(|element| element.role == "beam")
                .map(|element| format!("member:{}", element.id))
                .collect(),
            message: format!(
                "Beam deflection ratio L/{:.0} is close to or beyond the target limit L/{:.0}.",
                deflection_ratio, project.requirements.max_deflection_ratio
            ),
            suggested_actions: vec![
                "Increase beam stiffness or reduce span/demand.".into(),
                "Consider alternate framing topology if appropriate.".into(),
            ],
        });
    }

    summary
}

fn likely_connection_family(shear_n: f64, moment_nm: f64) -> String {
    if moment_nm.abs() > shear_n.abs() * 0.2 {
        "moment-capable".into()
    } else {
        "simple-shear".into()
    }
}

#[cfg(test)]
mod tests {
    use super::derive_engineering_summary;
    use crate::archetypes::build_frame_model;
    use crate::catalog::section_by_id;
    use crate::project::create_project;
    use crate::utils::timestamp_id;
    use std::fs;

    #[test]
    fn engineering_summary_extracts_connection_demands() {
        let temp_dir = std::env::temp_dir().join(format!("fraia-eng-test-{}", timestamp_id()));
        let (project, _) = create_project(&temp_dir, "eng-test").expect("create project");
        let beam = section_by_id("310UB").unwrap();
        let column = section_by_id("310UB").unwrap();
        let model = build_frame_model("clear_span", 20.0, 6.0, &beam, &column, 20.0, 80.0);
        let summary =
            derive_engineering_summary(&project, &model).expect("derive engineering summary");
        assert!(!summary.connection_demands.is_empty());
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn engineering_summary_uses_model_geometry_for_serviceability_warnings() {
        let temp_dir = std::env::temp_dir().join(format!("fraia-eng-geom-test-{}", timestamp_id()));
        let (mut project, _) = create_project(&temp_dir, "eng-geom-test").expect("create project");
        project.requirements.height_m = 999.0;
        project.requirements.span_m = 999.0;
        project.requirements.max_drift_ratio = 2000.0;
        project.requirements.max_deflection_ratio = 5000.0;
        let beam = section_by_id("310UB").unwrap();
        let column = section_by_id("310UB").unwrap();
        let model = build_frame_model("clear_span", 20.0, 6.0, &beam, &column, 20.0, 80.0);
        let summary =
            derive_engineering_summary(&project, &model).expect("derive engineering summary");

        assert!(
            summary
                .warnings
                .iter()
                .any(|warning| warning.id == "warning-drift")
        );
        assert!(
            summary
                .warnings
                .iter()
                .any(|warning| warning.id == "warning-deflection")
        );

        let _ = fs::remove_dir_all(temp_dir);
    }
}
