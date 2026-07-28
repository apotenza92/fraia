use crate::catalog::steel_material;
use crate::realization::realize_structural_model_to_frame2d;
use crate::structural_app::{
    AssignmentTargetRef, BuilderNodeMaterialization, LoadAssignment, LoadKind, LoadVector,
    StructuralMember, StructuralModel, StructuralNode, StructuralObjectRef, SupportAssignment,
};
use crate::types::{
    ArchetypeDefinition, BuilderArchetypeInstance, BuilderGraph, BuilderNode,
    BuilderNodeParameters, BuilderNodeStatus, Combo2D, FrameElement2D, FrameModel2D, LoadCase2D,
    NodalLoad2D, Node2D, PortalFrame2DBuilderParams, Section, SimplySupportedBeam2DBuilderParams,
    Support2D, Topology,
};
use std::collections::{BTreeMap, HashMap, HashSet};

pub const BUILDING_CONCEPT_ROOT_ARCHETYPE_ID: &str = "building.concept_root.v1";
pub const PORTAL_FRAME_2D_ARCHETYPE_ID: &str = "frame.portal_2d_steel_concept.v1";
pub const SIMPLY_SUPPORTED_BEAM_2D_ARCHETYPE_ID: &str = "beam.simply_supported_2d_steel_concept.v1";

pub fn archetype_definitions() -> Vec<ArchetypeDefinition> {
    vec![
        ArchetypeDefinition {
            id: BUILDING_CONCEPT_ROOT_ARCHETYPE_ID.into(),
            name: "Building concept root".into(),
            description: "Composition-only builder node used as the root of a Fraia builder graph. It does not materialize direct structural objects by itself.".into(),
            category: "building_root".into(),
            scale: "building".into(),
        },
        ArchetypeDefinition {
            id: PORTAL_FRAME_2D_ARCHETYPE_ID.into(),
            name: "2D steel portal frame concept".into(),
            description: "Single builder-node archetype that materializes a conceptual 2D steel portal frame into Fraia structural primitives.".into(),
            category: "frame_system".into(),
            scale: "subsystem".into(),
        },
        ArchetypeDefinition {
            id: SIMPLY_SUPPORTED_BEAM_2D_ARCHETYPE_ID.into(),
            name: "2D simply supported steel beam concept".into(),
            description: "Single builder-node archetype that materializes a conceptual simply supported steel beam with optional point and distributed loads into Fraia structural primitives.".into(),
            category: "beam_system".into(),
            scale: "component".into(),
        },
    ]
}

pub fn topologies() -> Vec<Topology> {
    vec![
        Topology {
            id: "clear_span".into(),
            name: "Clear-span rigid frame".into(),
            internal_columns: 0,
        },
        Topology {
            id: "one_internal".into(),
            name: "Rigid frame with one internal column".into(),
            internal_columns: 1,
        },
        Topology {
            id: "two_internal".into(),
            name: "Rigid frame with two internal columns".into(),
            internal_columns: 2,
        },
    ]
}

pub fn concept_root_builder_graph(root_id: &str, child_nodes: Vec<BuilderNode>) -> BuilderGraph {
    let child_node_ids = child_nodes.iter().map(|node| node.id.clone()).collect();
    let mut nodes = vec![BuilderNode {
        id: root_id.into(),
        label: Some("Project concept root".into()),
        archetype_id: BUILDING_CONCEPT_ROOT_ARCHETYPE_ID.into(),
        parameters: BuilderNodeParameters::ConceptRoot,
        child_node_ids,
        source_run_id: None,
        source_option_index: None,
        status: BuilderNodeStatus::Materialized,
    }];
    nodes.extend(child_nodes);
    BuilderGraph {
        root_node_ids: vec![root_id.into()],
        nodes,
    }
}

pub fn portal_frame_builder_node(
    node_id: &str,
    topology_id: &str,
    beam_section: &str,
    column_section: &str,
    span_m: f64,
    height_m: f64,
    gravity_load_kn_per_m: f64,
    lateral_load_kn: f64,
    origin_x_m: f64,
    origin_y_m: f64,
    origin_z_m: f64,
    source_run_id: Option<String>,
    source_option_index: Option<usize>,
) -> BuilderNode {
    BuilderNode {
        id: node_id.into(),
        label: Some("Primary frame concept".into()),
        archetype_id: PORTAL_FRAME_2D_ARCHETYPE_ID.into(),
        parameters: BuilderNodeParameters::PortalFrame2D(PortalFrame2DBuilderParams {
            topology_id: topology_id.into(),
            beam_section: beam_section.into(),
            column_section: column_section.into(),
            span_m,
            height_m,
            gravity_load_kn_per_m,
            lateral_load_kn,
            origin_x_m,
            origin_y_m,
            origin_z_m,
        }),
        child_node_ids: vec![],
        source_run_id,
        source_option_index,
        status: BuilderNodeStatus::Materialized,
    }
}

pub fn portal_frame_builder_graph(
    node_id: &str,
    topology_id: &str,
    beam_section: &str,
    column_section: &str,
    span_m: f64,
    height_m: f64,
    gravity_load_kn_per_m: f64,
    lateral_load_kn: f64,
    source_run_id: Option<String>,
    source_option_index: Option<usize>,
) -> BuilderGraph {
    BuilderGraph {
        root_node_ids: vec![node_id.into()],
        nodes: vec![portal_frame_builder_node(
            node_id,
            topology_id,
            beam_section,
            column_section,
            span_m,
            height_m,
            gravity_load_kn_per_m,
            lateral_load_kn,
            0.0,
            0.0,
            0.0,
            source_run_id,
            source_option_index,
        )],
    }
}

pub fn simply_supported_beam_builder_node(
    node_id: &str,
    section: &str,
    span_m: f64,
    distributed_load_kn_per_m: f64,
    point_load_kn: Option<f64>,
    point_load_x_m: Option<f64>,
    origin_x_m: f64,
    origin_y_m: f64,
    origin_z_m: f64,
    source_run_id: Option<String>,
    source_option_index: Option<usize>,
) -> BuilderNode {
    BuilderNode {
        id: node_id.into(),
        label: Some("Simply supported beam".into()),
        archetype_id: SIMPLY_SUPPORTED_BEAM_2D_ARCHETYPE_ID.into(),
        parameters: BuilderNodeParameters::SimplySupportedBeam2D(
            SimplySupportedBeam2DBuilderParams {
                section: section.into(),
                span_m,
                distributed_load_kn_per_m,
                point_load_kn,
                point_load_x_m,
                origin_x_m,
                origin_y_m,
                origin_z_m,
            },
        ),
        child_node_ids: vec![],
        source_run_id,
        source_option_index,
        status: BuilderNodeStatus::Materialized,
    }
}

pub fn simply_supported_beam_builder_graph(
    node_id: &str,
    section: &str,
    span_m: f64,
    distributed_load_kn_per_m: f64,
    point_load_kn: Option<f64>,
    point_load_x_m: Option<f64>,
    source_run_id: Option<String>,
    source_option_index: Option<usize>,
) -> BuilderGraph {
    BuilderGraph {
        root_node_ids: vec![node_id.into()],
        nodes: vec![simply_supported_beam_builder_node(
            node_id,
            section,
            span_m,
            distributed_load_kn_per_m,
            point_load_kn,
            point_load_x_m,
            0.0,
            0.0,
            0.0,
            source_run_id,
            source_option_index,
        )],
    }
}

pub fn ensure_concept_root_builder_graph(graph: &mut BuilderGraph, root_id: &str) -> String {
    if let Some(existing_root_id) = graph.root_node_ids.first()
        && let Some(existing_root) = graph.nodes.iter().find(|node| node.id == *existing_root_id)
        && matches!(existing_root.parameters, BuilderNodeParameters::ConceptRoot)
    {
        return existing_root.id.clone();
    }

    let previous_roots = graph.root_node_ids.clone();
    graph.nodes.insert(
        0,
        BuilderNode {
            id: root_id.into(),
            label: Some("Project concept root".into()),
            archetype_id: BUILDING_CONCEPT_ROOT_ARCHETYPE_ID.into(),
            parameters: BuilderNodeParameters::ConceptRoot,
            child_node_ids: previous_roots,
            source_run_id: None,
            source_option_index: None,
            status: BuilderNodeStatus::Proposed,
        },
    );
    graph.root_node_ids = vec![root_id.into()];
    root_id.into()
}

pub fn append_child_builder_node(
    graph: &mut BuilderGraph,
    parent_id: &str,
    child: BuilderNode,
) -> Option<()> {
    if graph.nodes.iter().any(|node| node.id == child.id) {
        return None;
    }

    let parent_index = graph.nodes.iter().position(|node| node.id == parent_id)?;
    graph.nodes[parent_index]
        .child_node_ids
        .push(child.id.clone());
    graph.nodes[parent_index].status = BuilderNodeStatus::Proposed;
    graph.nodes.push(child);
    Some(())
}

pub fn builder_graph_from_legacy_builder(builder: &BuilderArchetypeInstance) -> BuilderGraph {
    portal_frame_builder_graph(
        &builder.id,
        &builder.topology_id,
        &builder.beam_section,
        &builder.column_section,
        builder.span_m,
        builder.height_m,
        builder.gravity_load_kn_per_m,
        builder.lateral_load_kn,
        builder.source_run_id.clone(),
        builder.source_option_index,
    )
}

pub fn build_frame_model_from_builder_graph(graph: &BuilderGraph) -> Option<FrameModel2D> {
    for root_id in &graph.root_node_ids {
        let root = graph.nodes.iter().find(|node| &node.id == root_id)?;
        if let Some(model) = build_frame_model_from_builder_node(graph, root) {
            return Some(model);
        }
    }
    None
}

pub fn build_frame_model_from_builder_node(
    graph: &BuilderGraph,
    node: &BuilderNode,
) -> Option<FrameModel2D> {
    match &node.parameters {
        BuilderNodeParameters::ConceptRoot => {
            for child_id in &node.child_node_ids {
                let child = graph
                    .nodes
                    .iter()
                    .find(|candidate| &candidate.id == child_id)?;
                if let Some(model) = build_frame_model_from_builder_node(graph, child) {
                    return Some(model);
                }
            }
            None
        }
        BuilderNodeParameters::PortalFrame2D(params) => {
            let beam = crate::catalog::section_by_id(&params.beam_section)?;
            let column = crate::catalog::section_by_id(&params.column_section)?;
            let mut model = build_frame_model(
                &params.topology_id,
                params.span_m,
                params.height_m,
                &beam,
                &column,
                params.gravity_load_kn_per_m,
                params.lateral_load_kn,
            );
            translate_frame_model(
                &mut model,
                params.origin_x_m,
                params.origin_y_m,
                params.origin_z_m,
            );
            Some(model)
        }
        BuilderNodeParameters::SimplySupportedBeam2D(params) => {
            let structural = build_simply_supported_beam_structural_model(params)?;
            realize_structural_model_to_frame2d(&structural)
                .ok()
                .map(|realization| realization.model)
        }
    }
}

pub fn materialize_structural_model_from_builder_graph(
    graph: &BuilderGraph,
) -> Option<StructuralModel> {
    let mut combined = StructuralModel::empty();
    let mut visited = HashSet::new();

    for root_id in &graph.root_node_ids {
        materialize_builder_node_into(graph, root_id, &mut combined, &mut visited)?;
    }

    if !combined.nodes.is_empty() {
        combined.dimension = "2d-in-3d".into();
    }
    Some(combined)
}

fn materialize_builder_node_into(
    graph: &BuilderGraph,
    node_id: &str,
    combined: &mut StructuralModel,
    visited: &mut HashSet<String>,
) -> Option<()> {
    if !visited.insert(node_id.to_string()) {
        return Some(());
    }

    let node = graph
        .nodes
        .iter()
        .find(|candidate| candidate.id == node_id)?;
    match &node.parameters {
        BuilderNodeParameters::ConceptRoot => {}
        BuilderNodeParameters::PortalFrame2D(_) => {
            let frame = build_frame_model_from_builder_node(graph, node)?;
            let mut structural = StructuralModel::from_frame2d(&frame);
            namespace_structural_model(&mut structural, &node.id);
            let object_refs = structural_object_refs(&structural);
            combined
                .builder_node_materializations
                .push(BuilderNodeMaterialization {
                    builder_node_id: node.id.clone(),
                    object_refs,
                });
            append_structural_model(combined, structural);
        }
        BuilderNodeParameters::SimplySupportedBeam2D(params) => {
            let mut structural = build_simply_supported_beam_structural_model(params)?;
            namespace_structural_model(&mut structural, &node.id);
            let object_refs = structural_object_refs(&structural);
            combined
                .builder_node_materializations
                .push(BuilderNodeMaterialization {
                    builder_node_id: node.id.clone(),
                    object_refs,
                });
            append_structural_model(combined, structural);
        }
    }

    for child_id in &node.child_node_ids {
        materialize_builder_node_into(graph, child_id, combined, visited)?;
    }

    Some(())
}

fn structural_object_refs(structural: &StructuralModel) -> Vec<StructuralObjectRef> {
    let mut object_refs = Vec::new();
    object_refs.extend(
        structural
            .nodes
            .iter()
            .map(|node| StructuralObjectRef::Node(node.id.clone())),
    );
    object_refs.extend(
        structural
            .members
            .iter()
            .map(|member| StructuralObjectRef::Member(member.id.clone())),
    );
    object_refs.extend(
        structural
            .plates
            .iter()
            .map(|plate| StructuralObjectRef::Plate(plate.id.clone())),
    );
    object_refs.extend(
        structural
            .supports
            .iter()
            .map(|support| StructuralObjectRef::Support(support.id.clone())),
    );
    object_refs.extend(
        structural
            .loads
            .iter()
            .map(|load| StructuralObjectRef::Load(load.id.clone())),
    );
    object_refs.extend(
        structural
            .releases
            .iter()
            .map(|release| StructuralObjectRef::Release(release.id.clone())),
    );
    object_refs
}

fn build_simply_supported_beam_structural_model(
    params: &SimplySupportedBeam2DBuilderParams,
) -> Option<StructuralModel> {
    if params.span_m <= 0.0 {
        return None;
    }
    let section = crate::catalog::section_by_id(&params.section)?;
    let steel = steel_material();
    let tolerance = 1e-9;

    let mut xs: Vec<f64> = (0..=10)
        .map(|index| params.span_m * (index as f64) / 10.0)
        .collect();
    if params.point_load_kn.unwrap_or(0.0).abs() > tolerance {
        xs.push(
            params
                .point_load_x_m
                .unwrap_or(params.span_m * 0.5)
                .clamp(0.0, params.span_m),
        );
    }
    xs.sort_by(|a, b| a.total_cmp(b));
    xs.dedup_by(|a, b| (*a - *b).abs() < tolerance);

    let nodes: Vec<StructuralNode> = xs
        .iter()
        .enumerate()
        .map(|(index, x)| StructuralNode {
            id: format!("n{}", index + 1),
            x: params.origin_x_m + *x,
            y: params.origin_y_m,
            z: params.origin_z_m,
        })
        .collect();

    let members: Vec<StructuralMember> = xs
        .windows(2)
        .enumerate()
        .map(|(index, _)| StructuralMember {
            id: format!("m{}", index + 1),
            start_node: format!("n{}", index + 1),
            end_node: format!("n{}", index + 2),
            role: "beam".into(),
            semantic_tags: vec!["floor".into(), "primary".into(), "gravity".into()],
            section_id: section.id.clone(),
            material_id: steel.id.clone(),
        })
        .collect();

    let supports = vec![
        SupportAssignment {
            id: "support-left".into(),
            target_node: "n1".into(),
            ux: true,
            uy: true,
            uz: false,
            rx: false,
            ry: false,
            rz: false,
        },
        SupportAssignment {
            id: format!("support-right"),
            target_node: format!("n{}", nodes.len()),
            ux: false,
            uy: true,
            uz: false,
            rx: false,
            ry: false,
            rz: false,
        },
    ];

    let mut loads = Vec::new();
    if params.distributed_load_kn_per_m.abs() > tolerance {
        for member in &members {
            loads.push(LoadAssignment {
                id: format!("load-udl-{}", member.id),
                target: AssignmentTargetRef::Member(member.id.clone()),
                load_case_id: "gravity".into(),
                kind: LoadKind::UniformLine,
                direction: LoadVector {
                    x: 0.0,
                    y: -1.0,
                    z: 0.0,
                },
                magnitude: params.distributed_load_kn_per_m * 1000.0,
            });
        }
    }
    if let Some(point_load_kn) = params.point_load_kn.filter(|value| value.abs() > tolerance) {
        let point_x = params
            .point_load_x_m
            .unwrap_or(params.span_m * 0.5)
            .clamp(0.0, params.span_m);
        let load_node = nodes
            .iter()
            .find(|node| ((node.x - params.origin_x_m) - point_x).abs() < 1e-6)
            .map(|node| node.id.clone())
            .unwrap_or_else(|| "n1".into());
        loads.push(LoadAssignment {
            id: "load-point-1".into(),
            target: AssignmentTargetRef::Node(load_node),
            load_case_id: "gravity".into(),
            kind: LoadKind::Point,
            direction: LoadVector {
                x: 0.0,
                y: -1.0,
                z: 0.0,
            },
            magnitude: point_load_kn * 1000.0,
        });
    }

    Some(StructuralModel {
        dimension: "2d-in-3d".into(),
        nodes,
        members,
        plates: Vec::new(),
        supports,
        loads,
        releases: Vec::new(),
        load_cases: vec![LoadCase2D {
            id: "gravity".into(),
            nodal_loads: Vec::new(),
        }],
        builder_node_materializations: Vec::new(),
    })
}

fn translate_frame_model(
    model: &mut FrameModel2D,
    origin_x_m: f64,
    origin_y_m: f64,
    _origin_z_m: f64,
) {
    for node in &mut model.nodes {
        node.x += origin_x_m;
        node.y += origin_y_m;
    }
}

fn namespace_structural_model(structural: &mut StructuralModel, builder_node_id: &str) {
    let mut node_map = HashMap::new();
    for node in &mut structural.nodes {
        let old_id = node.id.clone();
        node.id = namespaced_id(builder_node_id, &old_id);
        node_map.insert(old_id, node.id.clone());
    }

    let mut member_map = HashMap::new();
    for member in &mut structural.members {
        let old_id = member.id.clone();
        member.id = namespaced_id(builder_node_id, &old_id);
        member.start_node = node_map
            .get(&member.start_node)
            .cloned()
            .unwrap_or_else(|| namespaced_id(builder_node_id, &member.start_node));
        member.end_node = node_map
            .get(&member.end_node)
            .cloned()
            .unwrap_or_else(|| namespaced_id(builder_node_id, &member.end_node));
        member_map.insert(old_id, member.id.clone());
    }

    let mut plate_map = HashMap::new();
    for plate in &mut structural.plates {
        let old_id = plate.id.clone();
        plate.id = namespaced_id(builder_node_id, &old_id);
        plate.boundary_nodes = plate
            .boundary_nodes
            .iter()
            .map(|node_id| {
                node_map
                    .get(node_id)
                    .cloned()
                    .unwrap_or_else(|| namespaced_id(builder_node_id, node_id))
            })
            .collect();
        plate_map.insert(old_id, plate.id.clone());
    }

    for support in &mut structural.supports {
        support.id = namespaced_id(builder_node_id, &support.id);
        support.target_node = node_map
            .get(&support.target_node)
            .cloned()
            .unwrap_or_else(|| namespaced_id(builder_node_id, &support.target_node));
    }

    for load in &mut structural.loads {
        load.id = namespaced_id(builder_node_id, &load.id);
        load.load_case_id = namespaced_id(builder_node_id, &load.load_case_id);
        load.target = match &load.target {
            crate::structural_app::AssignmentTargetRef::Node(id) => {
                crate::structural_app::AssignmentTargetRef::Node(
                    node_map
                        .get(id)
                        .cloned()
                        .unwrap_or_else(|| namespaced_id(builder_node_id, id)),
                )
            }
            crate::structural_app::AssignmentTargetRef::Member(id) => {
                crate::structural_app::AssignmentTargetRef::Member(
                    member_map
                        .get(id)
                        .cloned()
                        .unwrap_or_else(|| namespaced_id(builder_node_id, id)),
                )
            }
            crate::structural_app::AssignmentTargetRef::Plate(id) => {
                crate::structural_app::AssignmentTargetRef::Plate(
                    plate_map
                        .get(id)
                        .cloned()
                        .unwrap_or_else(|| namespaced_id(builder_node_id, id)),
                )
            }
        };
    }

    for release in &mut structural.releases {
        release.id = namespaced_id(builder_node_id, &release.id);
        release.target.member_id = member_map
            .get(&release.target.member_id)
            .cloned()
            .unwrap_or_else(|| namespaced_id(builder_node_id, &release.target.member_id));
    }

    for load_case in &mut structural.load_cases {
        load_case.id = namespaced_id(builder_node_id, &load_case.id);
        for nodal_load in &mut load_case.nodal_loads {
            nodal_load.node = node_map
                .get(&nodal_load.node)
                .cloned()
                .unwrap_or_else(|| namespaced_id(builder_node_id, &nodal_load.node));
        }
    }

    structural.builder_node_materializations.clear();
}

fn append_structural_model(target: &mut StructuralModel, mut source: StructuralModel) {
    target.nodes.append(&mut source.nodes);
    target.members.append(&mut source.members);
    target.plates.append(&mut source.plates);
    target.supports.append(&mut source.supports);
    target.loads.append(&mut source.loads);
    target.releases.append(&mut source.releases);
    target.load_cases.append(&mut source.load_cases);
    target
        .builder_node_materializations
        .append(&mut source.builder_node_materializations);
}

fn namespaced_id(builder_node_id: &str, raw_id: &str) -> String {
    format!("{}::{}", builder_node_id, raw_id)
}

pub fn build_frame_model(
    topology_id: &str,
    span_m: f64,
    height_m: f64,
    beam_section: &Section,
    column_section: &Section,
    gravity_load_kn_per_m: f64,
    lateral_load_kn: f64,
) -> FrameModel2D {
    let topology = topologies()
        .into_iter()
        .find(|t| t.id == topology_id)
        .expect("unknown topology");
    let column_positions = column_xs(topology_id, span_m);
    let beam_divisions_per_bay = 2;
    let column_divisions = 2;

    let mut nodes: Vec<Node2D> = vec![];
    let mut elements: Vec<FrameElement2D> = vec![];
    let mut supports: Vec<Support2D> = vec![];
    let mut node_counter = 1usize;
    let mut element_counter = 1usize;
    let mut node_at: HashMap<String, Node2D> = HashMap::new();

    let add_node = |x: f64,
                    y: f64,
                    nodes: &mut Vec<Node2D>,
                    node_at: &mut HashMap<String, Node2D>,
                    node_counter: &mut usize|
     -> Node2D {
        let key = format!("{x:.6}:{y:.6}");
        if let Some(existing) = node_at.get(&key) {
            return existing.clone();
        }
        let node = Node2D {
            id: format!("n{}", *node_counter),
            x,
            y,
        };
        *node_counter += 1;
        nodes.push(node.clone());
        node_at.insert(key, node.clone());
        node
    };

    let steel = steel_material();

    for x in column_positions.iter().copied() {
        let mut prev = add_node(x, 0.0, &mut nodes, &mut node_at, &mut node_counter);
        supports.push(Support2D {
            node: prev.id.clone(),
            ux: true,
            uy: true,
            rz: true,
        });
        for k in 1..=column_divisions {
            let y = height_m * (k as f64) / (column_divisions as f64);
            let next = add_node(x, y, &mut nodes, &mut node_at, &mut node_counter);
            elements.push(FrameElement2D {
                id: format!("e{}", element_counter),
                i: prev.id.clone(),
                j: next.id.clone(),
                role: "column".into(),
                section: column_section.clone(),
                material: steel.clone(),
            });
            element_counter += 1;
            prev = next;
        }
    }

    for bay in 0..(column_positions.len() - 1) {
        let x0 = column_positions[bay];
        let x1 = column_positions[bay + 1];
        let mut prev = add_node(x0, height_m, &mut nodes, &mut node_at, &mut node_counter);
        for k in 1..=beam_divisions_per_bay {
            let x = x0 + (x1 - x0) * (k as f64) / (beam_divisions_per_bay as f64);
            let next = add_node(x, height_m, &mut nodes, &mut node_at, &mut node_counter);
            elements.push(FrameElement2D {
                id: format!("e{}", element_counter),
                i: prev.id.clone(),
                j: next.id.clone(),
                role: "beam".into(),
                section: beam_section.clone(),
                material: steel.clone(),
            });
            element_counter += 1;
            prev = next;
        }
    }

    let top_node_ids: Vec<String> = nodes
        .iter()
        .filter(|n| (n.y - height_m).abs() < 1e-9)
        .map(|n| n.id.clone())
        .collect();

    FrameModel2D {
        model_type: "frame2d".into(),
        topology,
        nodes: nodes.clone(),
        elements: elements.clone(),
        supports,
        load_cases: vec![
            crate::types::LoadCase2D {
                id: "gravity".into(),
                nodal_loads: gravity_nodal_loads(&nodes, &elements, gravity_load_kn_per_m),
            },
            crate::types::LoadCase2D {
                id: "wind".into(),
                nodal_loads: top_node_ids
                    .into_iter()
                    .map(|node| NodalLoad2D {
                        node,
                        fx: lateral_load_kn * 1000.0
                            / (nodes
                                .iter()
                                .filter(|n| (n.y - height_m).abs() < 1e-9)
                                .count() as f64),
                        fy: 0.0,
                        mz: 0.0,
                    })
                    .collect(),
            },
        ],
        combos: vec![
            Combo2D {
                id: "SLS".into(),
                factors: BTreeMap::from([("gravity".into(), 1.0), ("wind".into(), 1.0)]),
            },
            Combo2D {
                id: "ULS".into(),
                factors: BTreeMap::from([("gravity".into(), 1.2), ("wind".into(), 1.5)]),
            },
        ],
    }
}

fn column_xs(topology_id: &str, span_m: f64) -> Vec<f64> {
    match topology_id {
        "clear_span" => vec![0.0, span_m],
        "one_internal" => vec![0.0, span_m / 2.0, span_m],
        "two_internal" => vec![0.0, span_m / 3.0, 2.0 * span_m / 3.0, span_m],
        _ => panic!("unknown topology"),
    }
}

fn gravity_nodal_loads(
    nodes: &[Node2D],
    elements: &[FrameElement2D],
    gravity_load_kn_per_m: f64,
) -> Vec<NodalLoad2D> {
    let mut map: HashMap<String, NodalLoad2D> = HashMap::new();
    for element in elements {
        if element.role != "beam" {
            continue;
        }
        let i = nodes.iter().find(|n| n.id == element.i).unwrap();
        let j = nodes.iter().find(|n| n.id == element.j).unwrap();
        let l = ((j.x - i.x).powi(2) + (j.y - i.y).powi(2)).sqrt();
        let total = gravity_load_kn_per_m * 1000.0 * l;
        add_load(&mut map, &element.i, -total / 2.0);
        add_load(&mut map, &element.j, -total / 2.0);
    }
    map.into_values().collect()
}

fn add_load(map: &mut HashMap<String, NodalLoad2D>, node_id: &str, fy: f64) {
    let current = map.entry(node_id.to_string()).or_insert(NodalLoad2D {
        node: node_id.into(),
        fx: 0.0,
        fy: 0.0,
        mz: 0.0,
    });
    current.fy += fy;
}

#[cfg(test)]
mod tests {
    use super::{
        BUILDING_CONCEPT_ROOT_ARCHETYPE_ID, PORTAL_FRAME_2D_ARCHETYPE_ID,
        SIMPLY_SUPPORTED_BEAM_2D_ARCHETYPE_ID, append_child_builder_node,
        build_frame_model_from_builder_graph, concept_root_builder_graph,
        ensure_concept_root_builder_graph, materialize_structural_model_from_builder_graph,
        portal_frame_builder_graph, portal_frame_builder_node, simply_supported_beam_builder_graph,
    };
    use crate::{LoadKind, types::BuilderNodeStatus};

    #[test]
    fn builder_graph_builds_frame_model() {
        let graph = portal_frame_builder_graph(
            "builder-1",
            "one_internal",
            "310UB",
            "360UB",
            24.0,
            7.0,
            18.0,
            90.0,
            None,
            None,
        );

        let root = &graph.nodes[0];
        assert_eq!(root.archetype_id, PORTAL_FRAME_2D_ARCHETYPE_ID);
        assert_eq!(root.status, BuilderNodeStatus::Materialized);

        let model = build_frame_model_from_builder_graph(&graph).expect("build from graph");
        assert_eq!(model.topology.id, "one_internal");
        assert!(!model.nodes.is_empty());
        assert!(!model.elements.is_empty());
    }

    #[test]
    fn builder_graph_materializes_structural_model() {
        let graph = portal_frame_builder_graph(
            "builder-1",
            "clear_span",
            "310UB",
            "360UB",
            18.0,
            6.0,
            15.0,
            60.0,
            Some("run-1".into()),
            Some(1),
        );

        let structural = materialize_structural_model_from_builder_graph(&graph)
            .expect("materialize structural model");
        assert!(!structural.members.is_empty());
        assert!(!structural.nodes.is_empty());
        let mapping = structural
            .materialization_for_builder_node("builder-1")
            .expect("builder mapping");
        assert!(!mapping.object_refs.is_empty());
    }

    #[test]
    fn concept_root_with_child_portal_frames_materializes_multiple_mapped_subsystems() {
        let child_a = portal_frame_builder_node(
            "frame-a",
            "clear_span",
            "310UB",
            "360UB",
            18.0,
            6.0,
            15.0,
            60.0,
            0.0,
            0.0,
            0.0,
            None,
            None,
        );
        let child_b = portal_frame_builder_node(
            "frame-b",
            "one_internal",
            "310UB",
            "360UB",
            18.0,
            6.0,
            15.0,
            60.0,
            30.0,
            0.0,
            0.0,
            None,
            None,
        );
        let graph = concept_root_builder_graph("building-root", vec![child_a, child_b]);

        assert_eq!(
            graph.nodes[0].archetype_id,
            BUILDING_CONCEPT_ROOT_ARCHETYPE_ID
        );
        let model = build_frame_model_from_builder_graph(&graph).expect("build first descendant");
        assert!(model.nodes.iter().any(|node| node.x >= 18.0));

        let structural = materialize_structural_model_from_builder_graph(&graph)
            .expect("materialize structural model");
        assert!(
            structural
                .materialization_for_builder_node("frame-a")
                .is_some()
        );
        assert!(
            structural
                .materialization_for_builder_node("frame-b")
                .is_some()
        );
        assert!(
            structural
                .nodes
                .iter()
                .any(|node| node.id.starts_with("frame-a::"))
        );
        assert!(
            structural
                .nodes
                .iter()
                .any(|node| node.id.starts_with("frame-b::"))
        );
    }

    #[test]
    fn ensure_concept_root_wraps_single_root_builder_graph() {
        let mut graph = portal_frame_builder_graph(
            "builder-1",
            "clear_span",
            "310UB",
            "360UB",
            18.0,
            6.0,
            15.0,
            60.0,
            None,
            None,
        );

        let root_id = ensure_concept_root_builder_graph(&mut graph, "building-root");

        assert_eq!(root_id, "building-root");
        assert_eq!(graph.root_node_ids, vec!["building-root"]);
        assert!(matches!(
            graph.nodes[0].parameters,
            crate::types::BuilderNodeParameters::ConceptRoot
        ));
        assert_eq!(graph.nodes[0].child_node_ids, vec!["builder-1"]);
    }

    #[test]
    fn append_child_builder_node_adds_child_relationship() {
        let child_a = portal_frame_builder_node(
            "frame-a",
            "clear_span",
            "310UB",
            "360UB",
            18.0,
            6.0,
            15.0,
            60.0,
            0.0,
            0.0,
            0.0,
            None,
            None,
        );
        let mut graph = concept_root_builder_graph("building-root", vec![child_a]);
        let child_b = portal_frame_builder_node(
            "frame-b",
            "one_internal",
            "310UB",
            "360UB",
            18.0,
            6.0,
            15.0,
            60.0,
            24.0,
            0.0,
            0.0,
            None,
            None,
        );

        append_child_builder_node(&mut graph, "building-root", child_b).expect("append child");

        let root = graph
            .nodes
            .iter()
            .find(|node| node.id == "building-root")
            .expect("concept root");
        assert!(root.child_node_ids.iter().any(|id| id == "frame-b"));
        assert!(graph.nodes.iter().any(|node| node.id == "frame-b"));
    }

    #[test]
    fn simply_supported_beam_builder_graph_builds_frame_model() {
        let graph = simply_supported_beam_builder_graph(
            "beam-1",
            "310UB",
            6.0,
            8.0,
            Some(20.0),
            Some(3.0),
            None,
            None,
        );

        let root = &graph.nodes[0];
        assert_eq!(root.archetype_id, SIMPLY_SUPPORTED_BEAM_2D_ARCHETYPE_ID);

        let model = build_frame_model_from_builder_graph(&graph).expect("build beam frame model");
        assert!(!model.nodes.is_empty());
        assert!(!model.elements.is_empty());
        assert_eq!(model.supports.len(), 2);
        assert!(model.load_cases.iter().any(|case| case.id == "gravity"));
    }

    #[test]
    fn simply_supported_beam_builder_graph_materializes_structural_model_with_loads() {
        let graph = simply_supported_beam_builder_graph(
            "beam-1",
            "250UB",
            5.0,
            12.0,
            Some(15.0),
            None,
            Some("run-1".into()),
            Some(1),
        );

        let structural = materialize_structural_model_from_builder_graph(&graph)
            .expect("materialize beam structural model");
        assert!(structural.nodes.len() >= 3);
        assert!(!structural.members.is_empty());
        assert_eq!(structural.supports.len(), 2);
        assert!(!structural.loads.is_empty());
        assert!(
            structural
                .loads
                .iter()
                .any(|load| load.kind == LoadKind::UniformLine
                    && (load.magnitude - 12_000.0).abs() < 1e-9)
        );
        assert!(
            structural.loads.iter().any(
                |load| load.kind == LoadKind::Point && (load.magnitude - 15_000.0).abs() < 1e-9
            )
        );
        assert!(
            structural
                .materialization_for_builder_node("beam-1")
                .is_some()
        );
    }
}
