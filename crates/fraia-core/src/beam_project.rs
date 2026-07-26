use crate::{
    BuilderGraph, BuilderNodeParameters, BuilderNodeStatus, ProjectFile,
    SimplySupportedBeam2DBuilderParams, SimplySupportedBeamSizingRequest,
    SimplySupportedBeamSizingResult, materialize_structural_model_from_builder_graph,
    simply_supported_beam_builder_node, size_simply_supported_beam,
};
use anyhow::{Context, Result};

pub fn current_simply_supported_beam_builder_node_id(project: &ProjectFile) -> Option<&str> {
    project
        .builder_graph
        .as_ref()?
        .nodes
        .iter()
        .find_map(|node| match &node.parameters {
            BuilderNodeParameters::SimplySupportedBeam2D(_) => Some(node.id.as_str()),
            _ => None,
        })
}

pub fn current_simply_supported_beam_builder_params(
    project: &ProjectFile,
) -> Option<SimplySupportedBeam2DBuilderParams> {
    project
        .builder_graph
        .as_ref()?
        .nodes
        .iter()
        .find_map(|node| match &node.parameters {
            BuilderNodeParameters::SimplySupportedBeam2D(params) => Some(params.clone()),
            _ => None,
        })
}

pub fn sync_project_requirements_from_current_beam(project: &mut ProjectFile) -> bool {
    let Some(params) = current_simply_supported_beam_builder_params(project) else {
        return false;
    };

    let mut changed = false;
    if project.intent.building_type != "beam" {
        project.intent.building_type = "beam".into();
        changed = true;
    }
    if (project.requirements.span_m - params.span_m).abs() > 1e-9 {
        project.requirements.span_m = params.span_m;
        changed = true;
    }
    if project.requirements.height_m.abs() > 1e-9 {
        project.requirements.height_m = 0.0;
        changed = true;
    }
    if (project.requirements.gravity_load_kn_per_m - params.distributed_load_kn_per_m).abs() > 1e-9
    {
        project.requirements.gravity_load_kn_per_m = params.distributed_load_kn_per_m;
        changed = true;
    }
    if project.requirements.lateral_load_kn.abs() > 1e-9 {
        project.requirements.lateral_load_kn = 0.0;
        changed = true;
    }
    if project.requirements.max_internal_columns != 0 {
        project.requirements.max_internal_columns = 0;
        changed = true;
    }
    changed
}

pub fn seed_simply_supported_beam_in_project(
    project: &mut ProjectFile,
    node_id_hint: Option<&str>,
) -> Result<String> {
    let existing_params = current_simply_supported_beam_builder_params(project);
    let node_id = node_id_hint
        .map(str::to_owned)
        .or_else(|| current_simply_supported_beam_builder_node_id(project).map(str::to_owned))
        .unwrap_or_else(|| "builder.beam.authored".into());
    let preserved_params = existing_params.unwrap_or_else(|| SimplySupportedBeam2DBuilderParams {
        section: "200UB".into(),
        span_m: project.requirements.span_m,
        distributed_load_kn_per_m: project.requirements.gravity_load_kn_per_m,
        point_load_kn: None,
        point_load_x_m: None,
        origin_x_m: 0.0,
        origin_y_m: 0.0,
        origin_z_m: 0.0,
    });

    let graph = BuilderGraph {
        root_node_ids: vec![node_id.clone()],
        nodes: vec![simply_supported_beam_builder_node(
            &node_id,
            &preserved_params.section,
            project.requirements.span_m,
            project.requirements.gravity_load_kn_per_m,
            preserved_params.point_load_kn,
            preserved_params.point_load_x_m,
            preserved_params.origin_x_m,
            preserved_params.origin_y_m,
            preserved_params.origin_z_m,
            None,
            None,
        )],
    };
    let structural = materialize_structural_model_from_builder_graph(&graph)
        .context("failed to materialize simply supported beam from project requirements")?;

    project.intent.building_type = "beam".into();
    project.builder_graph = Some(graph);
    project.legacy_builder_instance = None;
    project.structural_model = Some(structural);
    let _ = sync_project_requirements_from_current_beam(project);
    project.updated_at = Some(crate::utils::iso_now());
    Ok(node_id)
}

pub fn size_current_simply_supported_beam_in_project(
    project: &mut ProjectFile,
) -> Result<SimplySupportedBeamSizingResult> {
    let beam_params = current_simply_supported_beam_builder_params(project)
        .context("no simply supported beam builder node was found in this project")?;

    let sizing = size_simply_supported_beam(&SimplySupportedBeamSizingRequest {
        span_m: beam_params.span_m,
        distributed_load_kn_per_m: beam_params.distributed_load_kn_per_m,
        point_load_kn: beam_params.point_load_kn,
        point_load_x_m: beam_params.point_load_x_m,
        target_max_utilization: project.requirements.max_utilization,
        target_deflection_ratio: project.requirements.max_deflection_ratio,
    })?;
    let chosen = sizing
        .chosen
        .as_ref()
        .context("no feasible beam section was found for the current request")?;

    let graph = project
        .builder_graph
        .as_mut()
        .context("no builder graph saved in the project")?;
    for node in &mut graph.nodes {
        if let BuilderNodeParameters::SimplySupportedBeam2D(params) = &mut node.parameters {
            params.section = chosen.section_id.clone();
            node.status = BuilderNodeStatus::Materialized;
        }
    }
    project.structural_model = materialize_structural_model_from_builder_graph(graph);
    let _ = sync_project_requirements_from_current_beam(project);
    project.updated_at = Some(crate::utils::iso_now());
    Ok(sizing)
}

#[cfg(test)]
mod tests {
    use super::{
        current_simply_supported_beam_builder_node_id,
        current_simply_supported_beam_builder_params, seed_simply_supported_beam_in_project,
        size_current_simply_supported_beam_in_project, sync_project_requirements_from_current_beam,
    };
    use crate::create_project;
    use std::fs;

    #[test]
    fn seeds_beam_in_project_and_syncs_requirements() {
        let temp_dir = std::env::temp_dir().join(format!(
            "fraia-beam-project-ops-seed-{}",
            crate::utils::timestamp_id()
        ));
        let (mut project, _) = create_project(&temp_dir, "beam-project-ops").expect("create");
        project.requirements.span_m = 7.5;
        project.requirements.gravity_load_kn_per_m = 6.0;

        let node_id = seed_simply_supported_beam_in_project(&mut project, Some("builder.beam.ops"))
            .expect("seed beam");

        assert_eq!(node_id, "builder.beam.ops");
        assert_eq!(
            current_simply_supported_beam_builder_node_id(&project),
            Some("builder.beam.ops")
        );
        let params = current_simply_supported_beam_builder_params(&project).expect("params");
        assert_eq!(params.span_m, 7.5);
        assert_eq!(project.intent.building_type, "beam");
        assert_eq!(project.requirements.height_m, 0.0);
        assert!(project.structural_model.is_some());

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn sizes_current_beam_in_project_updates_section_and_syncs_requirements() {
        let temp_dir = std::env::temp_dir().join(format!(
            "fraia-beam-project-ops-size-{}",
            crate::utils::timestamp_id()
        ));
        let (mut project, _) = create_project(&temp_dir, "beam-project-ops").expect("create");
        project.requirements.span_m = 6.0;
        project.requirements.gravity_load_kn_per_m = 8.0;
        seed_simply_supported_beam_in_project(&mut project, None).expect("seed beam");

        let sizing = size_current_simply_supported_beam_in_project(&mut project).expect("size");
        let chosen = sizing.chosen.expect("chosen");
        let params = current_simply_supported_beam_builder_params(&project).expect("params");

        assert_eq!(params.section, chosen.section_id);
        assert_eq!(project.requirements.span_m, params.span_m);
        assert_eq!(
            project.requirements.gravity_load_kn_per_m,
            params.distributed_load_kn_per_m
        );
        assert!(project.structural_model.is_some());

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn syncs_project_requirements_from_existing_beam() {
        let temp_dir = std::env::temp_dir().join(format!(
            "fraia-beam-project-ops-sync-{}",
            crate::utils::timestamp_id()
        ));
        let (mut project, _) = create_project(&temp_dir, "beam-project-ops").expect("create");
        project.requirements.span_m = 9.0;
        project.requirements.gravity_load_kn_per_m = 4.0;
        seed_simply_supported_beam_in_project(&mut project, None).expect("seed beam");
        project.requirements.span_m = 100.0;
        project.requirements.gravity_load_kn_per_m = 100.0;
        project.requirements.height_m = 5.0;
        project.requirements.lateral_load_kn = 50.0;
        project.requirements.max_internal_columns = 3;

        let changed = sync_project_requirements_from_current_beam(&mut project);

        assert!(changed);
        assert_eq!(project.requirements.span_m, 9.0);
        assert_eq!(project.requirements.gravity_load_kn_per_m, 4.0);
        assert_eq!(project.requirements.height_m, 0.0);
        assert_eq!(project.requirements.lateral_load_kn, 0.0);
        assert_eq!(project.requirements.max_internal_columns, 0);

        let _ = fs::remove_dir_all(temp_dir);
    }
}
