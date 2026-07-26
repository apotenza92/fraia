use crate::archetypes::{
    builder_graph_from_legacy_builder, materialize_structural_model_from_builder_graph,
};
use crate::structural_app::StructuralModel;
use crate::types::{
    AgentState, BuilderNodeParameters, Intent, PlanningAnalysisBrief, PlanningDesignConstraints,
    PlanningDraft, PlanningGeometryAndLoads, PlanningProjectIntent, PlanningSystemBrief,
    ProjectFile, ProjectFiles, Requirements, SearchPermissions,
};
use crate::units::metric_structural_unit_profile;
use crate::utils::{ensure_dir, iso_now, read_json, write_json};
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

pub const PROJECT_FILE: &str = "fraia.project.json";
pub const PLANNING_FILE: &str = "planning.md";

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

    write_json(&paths.project_file, &project)?;
    fs::write(&paths.planning_file, default_planning_markdown(&project))?;
    Ok((project, paths))
}

pub fn load_project(project_dir: &Path) -> Result<(ProjectFile, ProjectPaths)> {
    let paths = project_paths(project_dir);
    let mut project = read_json::<ProjectFile>(&paths.project_file)?;
    project.schema_version = "0.2.0".into();
    if project.builder_graph.is_none()
        && let Some(legacy) = project.legacy_builder_instance.take()
    {
        project.builder_graph = Some(builder_graph_from_legacy_builder(&legacy));
    }
    if project.planning_draft.is_none() {
        project.planning_draft = Some(default_planning_draft(&project));
    }
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
        apply_planning_draft, create_project, load_project, materialize_project_structural_model,
        planning_draft, save_project,
    };
    use crate::archetypes::portal_frame_builder_graph;
    use crate::structural_app::{
        AssignmentTargetRef, LoadAssignment, LoadKind, LoadVector, MemberEnd, MemberEndTarget,
        ReleaseAssignment, StructuralMember, StructuralNode,
    };
    use crate::utils::timestamp_id;
    use serde_json::json;
    use std::fs;

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
