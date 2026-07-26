use crate::catalog::{section_by_id, steel_material};
use crate::structural_app::{AssignmentTargetRef, LoadKind, StructuralModel};
use crate::types::{
    Combo2D, FrameElement2D, FrameModel2D, LoadCase2D, NodalLoad2D, Node2D, Support2D, Topology,
};
use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealizationDiagnostic {
    pub severity: RealizationSeverity,
    pub code: String,
    pub message: String,
    pub object_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RealizationSeverity {
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frame2DRealization {
    pub model: FrameModel2D,
    pub diagnostics: Vec<RealizationDiagnostic>,
}

pub fn realize_structural_model_to_frame2d(
    structural: &StructuralModel,
) -> Result<Frame2DRealization> {
    let mut diagnostics = Vec::new();

    for node in &structural.nodes {
        if node.z.abs() > 1e-9 {
            bail!(
                "node {} is not in the XY plane; current frame2d realization only supports z = 0",
                node.id
            );
        }
    }

    let nodes: Vec<Node2D> = structural
        .nodes
        .iter()
        .map(|node| Node2D {
            id: node.id.clone(),
            x: node.x,
            y: node.y,
        })
        .collect();

    let node_map: HashMap<&str, &Node2D> =
        nodes.iter().map(|node| (node.id.as_str(), node)).collect();
    let default_material = steel_material();

    let mut elements = Vec::new();
    for member in &structural.members {
        if !node_map.contains_key(member.start_node.as_str()) {
            bail!(
                "member {} references missing start node {}",
                member.id,
                member.start_node
            );
        }
        if !node_map.contains_key(member.end_node.as_str()) {
            bail!(
                "member {} references missing end node {}",
                member.id,
                member.end_node
            );
        }
        let section = section_by_id(&member.section_id).ok_or_else(|| {
            anyhow::anyhow!(
                "member {} references unknown section {}",
                member.id,
                member.section_id
            )
        })?;
        if member.material_id != default_material.id && member.material_id != "steel" {
            diagnostics.push(RealizationDiagnostic {
                severity: RealizationSeverity::Warning,
                code: "material-fallback".into(),
                message: format!(
                    "member {} material {} is not yet realized explicitly; using demo steel material",
                    member.id, member.material_id
                ),
                object_id: Some(member.id.clone()),
            });
        }
        elements.push(FrameElement2D {
            id: member.id.clone(),
            i: member.start_node.clone(),
            j: member.end_node.clone(),
            role: member.role.clone(),
            section,
            material: default_material.clone(),
        });
    }

    let supports: Vec<Support2D> = structural
        .supports
        .iter()
        .map(|support| {
            if !node_map.contains_key(support.target_node.as_str()) {
                bail!(
                    "support {} references missing node {}",
                    support.id,
                    support.target_node
                );
            }
            Ok(Support2D {
                node: support.target_node.clone(),
                ux: support.ux,
                uy: support.uy,
                rz: support.rz,
            })
        })
        .collect::<Result<_>>()?;

    let mut load_case_order: Vec<String> = structural
        .load_cases
        .iter()
        .map(|lc| lc.id.clone())
        .collect();
    let mut seen_cases: BTreeSet<String> = load_case_order.iter().cloned().collect();
    for load in &structural.loads {
        if seen_cases.insert(load.load_case_id.clone()) {
            load_case_order.push(load.load_case_id.clone());
            diagnostics.push(RealizationDiagnostic {
                severity: RealizationSeverity::Warning,
                code: "implicit-load-case".into(),
                message: format!(
                    "load {} referenced missing load case {}; creating it implicitly",
                    load.id, load.load_case_id
                ),
                object_id: Some(load.id.clone()),
            });
        }
    }
    if load_case_order.is_empty() {
        load_case_order.push("default".into());
    }

    let mut nodal_loads_by_case: HashMap<String, Vec<NodalLoad2D>> = HashMap::new();
    for case_id in &load_case_order {
        nodal_loads_by_case.insert(case_id.clone(), Vec::new());
    }

    for load in &structural.loads {
        let entries = nodal_loads_by_case
            .get_mut(&load.load_case_id)
            .expect("load case should exist");
        match &load.target {
            AssignmentTargetRef::Node(node_id) => {
                if !node_map.contains_key(node_id.as_str()) {
                    bail!("load {} references missing node {}", load.id, node_id);
                }
                if !matches!(load.kind, LoadKind::Point) {
                    diagnostics.push(RealizationDiagnostic {
                        severity: RealizationSeverity::Warning,
                        code: "unsupported-node-load-kind".into(),
                        message: format!(
                            "node load {} uses unsupported kind {}; it was ignored",
                            load.id,
                            load.kind.as_str()
                        ),
                        object_id: Some(load.id.clone()),
                    });
                    continue;
                }
                entries.push(NodalLoad2D {
                    node: node_id.clone(),
                    fx: load.direction.x * load.magnitude,
                    fy: load.direction.y * load.magnitude,
                    mz: 0.0,
                });
                if load.direction.z.abs() > 1e-9 {
                    diagnostics.push(RealizationDiagnostic {
                        severity: RealizationSeverity::Warning,
                        code: "out-of-plane-load-ignored".into(),
                        message: format!(
                            "load {} has a z component that cannot be realized in frame2d and was ignored",
                            load.id
                        ),
                        object_id: Some(load.id.clone()),
                    });
                }
            }
            AssignmentTargetRef::Member(member_id) => {
                let member = structural
                    .members
                    .iter()
                    .find(|member| member.id == *member_id)
                    .ok_or_else(|| {
                        anyhow::anyhow!("load {} references missing member {}", load.id, member_id)
                    })?;
                let start = node_map.get(member.start_node.as_str()).ok_or_else(|| {
                    anyhow::anyhow!(
                        "member {} start node missing during load realization",
                        member.id
                    )
                })?;
                let end = node_map.get(member.end_node.as_str()).ok_or_else(|| {
                    anyhow::anyhow!(
                        "member {} end node missing during load realization",
                        member.id
                    )
                })?;
                let length = ((end.x - start.x).powi(2) + (end.y - start.y).powi(2)).sqrt();
                match load.kind {
                    LoadKind::UniformLine => {
                        let total_fx = load.direction.x * load.magnitude * length;
                        let total_fy = load.direction.y * load.magnitude * length;
                        entries.push(NodalLoad2D {
                            node: member.start_node.clone(),
                            fx: total_fx * 0.5,
                            fy: total_fy * 0.5,
                            mz: 0.0,
                        });
                        entries.push(NodalLoad2D {
                            node: member.end_node.clone(),
                            fx: total_fx * 0.5,
                            fy: total_fy * 0.5,
                            mz: 0.0,
                        });
                        if load.direction.z.abs() > 1e-9 {
                            diagnostics.push(RealizationDiagnostic {
                                severity: RealizationSeverity::Warning,
                                code: "out-of-plane-load-ignored".into(),
                                message: format!(
                                    "member load {} has a z component that cannot be realized in frame2d and was ignored",
                                    load.id
                                ),
                                object_id: Some(load.id.clone()),
                            });
                        }
                    }
                    other => {
                        diagnostics.push(RealizationDiagnostic {
                            severity: RealizationSeverity::Warning,
                            code: "unsupported-member-load-kind".into(),
                            message: format!(
                                "member load {} uses unsupported kind {}; it was ignored",
                                load.id,
                                other.as_str()
                            ),
                            object_id: Some(load.id.clone()),
                        });
                    }
                }
            }
            AssignmentTargetRef::Plate(plate_id) => {
                diagnostics.push(RealizationDiagnostic {
                    severity: RealizationSeverity::Warning,
                    code: "plate-load-ignored".into(),
                    message: format!(
                        "plate load {} on {} is not realized in the current frame2d adapter",
                        load.id, plate_id
                    ),
                    object_id: Some(load.id.clone()),
                });
            }
        }
    }

    let mut load_cases: Vec<LoadCase2D> = Vec::new();
    for case_id in load_case_order {
        let mut nodal_loads = structural
            .load_cases
            .iter()
            .find(|load_case| load_case.id == case_id)
            .map(|lc| lc.nodal_loads.clone())
            .unwrap_or_default();
        if let Some(extra) = nodal_loads_by_case.remove(&case_id) {
            nodal_loads.extend(extra);
        }
        load_cases.push(LoadCase2D {
            id: case_id,
            nodal_loads,
        });
    }

    for plate in &structural.plates {
        diagnostics.push(RealizationDiagnostic {
            severity: RealizationSeverity::Warning,
            code: "plate-placeholder".into(),
            message: format!(
                "plate {} is preserved as authored state but not directly realized in the current frame2d adapter",
                plate.id
            ),
            object_id: Some(plate.id.clone()),
        });
    }

    for release in &structural.releases {
        diagnostics.push(RealizationDiagnostic {
            severity: RealizationSeverity::Warning,
            code: "release-placeholder".into(),
            message: format!(
                "release {} on member {} is not yet realized in the current frame2d adapter",
                release.id, release.target.member_id
            ),
            object_id: Some(release.id.clone()),
        });
    }

    let combos = default_combos(&load_cases);

    Ok(Frame2DRealization {
        model: FrameModel2D {
            model_type: "frame2d-from-authored-structural-model".into(),
            topology: Topology {
                id: "authored_structural_model".into(),
                name: "Authored structural model".into(),
                internal_columns: 0,
            },
            nodes,
            elements,
            supports,
            load_cases,
            combos,
        },
        diagnostics,
    })
}

fn default_combos(load_cases: &[LoadCase2D]) -> Vec<Combo2D> {
    if load_cases.is_empty() {
        return vec![Combo2D {
            id: "SLS".into(),
            factors: BTreeMap::new(),
        }];
    }

    let factors = load_cases
        .iter()
        .map(|load_case| (load_case.id.clone(), 1.0))
        .collect();
    vec![Combo2D {
        id: "SLS".into(),
        factors,
    }]
}

#[cfg(test)]
mod tests {
    use super::realize_structural_model_to_frame2d;
    use crate::structural_app::{
        AssignmentTargetRef, LoadAssignment, LoadKind, LoadVector, MemberEnd, MemberEndTarget,
        ReleaseAssignment, StructuralMember, StructuralModel, StructuralNode, StructuralPlate,
        SupportAssignment,
    };

    #[test]
    fn realizes_basic_structural_model_to_frame2d() {
        let mut model = StructuralModel::empty();
        model.dimension = "2d-in-3d".into();
        model.nodes = vec![
            StructuralNode {
                id: "n1".into(),
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            StructuralNode {
                id: "n2".into(),
                x: 0.0,
                y: 6.0,
                z: 0.0,
            },
            StructuralNode {
                id: "n3".into(),
                x: 8.0,
                y: 6.0,
                z: 0.0,
            },
        ];
        model.members = vec![
            StructuralMember {
                id: "c1".into(),
                start_node: "n1".into(),
                end_node: "n2".into(),
                role: "column".into(),
                semantic_tags: Vec::new(),
                section_id: "310UB".into(),
                material_id: "steel".into(),
            },
            StructuralMember {
                id: "b1".into(),
                start_node: "n2".into(),
                end_node: "n3".into(),
                role: "beam".into(),
                semantic_tags: Vec::new(),
                section_id: "250UB".into(),
                material_id: "steel".into(),
            },
        ];
        model.supports = vec![SupportAssignment {
            id: "s1".into(),
            target_node: "n1".into(),
            ux: true,
            uy: true,
            uz: false,
            rx: false,
            ry: false,
            rz: true,
        }];
        model.loads = vec![LoadAssignment {
            id: "load-1".into(),
            target: AssignmentTargetRef::Member("b1".into()),
            load_case_id: "gravity".into(),
            kind: LoadKind::UniformLine,
            direction: LoadVector {
                x: 0.0,
                y: -1.0,
                z: 0.0,
            },
            magnitude: 10_000.0,
        }];
        model.releases = vec![ReleaseAssignment {
            id: "r1".into(),
            target: MemberEndTarget {
                member_id: "b1".into(),
                end: MemberEnd::Start,
            },
            ux: false,
            uy: false,
            uz: false,
            rx: false,
            ry: false,
            rz: true,
        }];
        model.plates = vec![StructuralPlate {
            id: "p1".into(),
            boundary_nodes: vec!["n1".into(), "n2".into(), "n3".into()],
            role: "conceptual_panel".into(),
            semantic_tags: Vec::new(),
            thickness_m: 0.2,
            material_id: "steel".into(),
            generated_from: "test".into(),
        }];

        let realization = realize_structural_model_to_frame2d(&model).expect("realize model");
        assert_eq!(realization.model.nodes.len(), 3);
        assert_eq!(realization.model.elements.len(), 2);
        assert_eq!(realization.model.supports.len(), 1);
        assert_eq!(realization.model.load_cases.len(), 1);
        assert_eq!(realization.model.load_cases[0].nodal_loads.len(), 2);
        assert_eq!(realization.model.combos.len(), 1);
        assert!(
            realization
                .diagnostics
                .iter()
                .any(|diag| diag.code == "release-placeholder")
        );
        assert!(
            realization
                .diagnostics
                .iter()
                .any(|diag| diag.code == "plate-placeholder")
        );
    }

    #[test]
    fn rejects_out_of_plane_nodes() {
        let mut model = StructuralModel::empty();
        model.nodes.push(StructuralNode {
            id: "n1".into(),
            x: 0.0,
            y: 0.0,
            z: 1.0,
        });

        assert!(realize_structural_model_to_frame2d(&model).is_err());
    }

    #[test]
    fn plate_area_loads_are_retained_authored_but_warn_in_frame2d_realization() {
        let mut model = StructuralModel::empty();
        model.nodes = vec![
            StructuralNode {
                id: "n1".into(),
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            StructuralNode {
                id: "n2".into(),
                x: 4.0,
                y: 0.0,
                z: 0.0,
            },
            StructuralNode {
                id: "n3".into(),
                x: 4.0,
                y: 4.0,
                z: 0.0,
            },
            StructuralNode {
                id: "n4".into(),
                x: 0.0,
                y: 4.0,
                z: 0.0,
            },
        ];
        model.plates = vec![StructuralPlate {
            id: "p1".into(),
            boundary_nodes: vec!["n1".into(), "n2".into(), "n3".into(), "n4".into()],
            role: "slab".into(),
            semantic_tags: Vec::new(),
            thickness_m: 0.2,
            material_id: "steel".into(),
            generated_from: "test".into(),
        }];
        model.loads = vec![LoadAssignment {
            id: "load-area-1".into(),
            target: AssignmentTargetRef::Plate("p1".into()),
            load_case_id: "gravity".into(),
            kind: LoadKind::Area,
            direction: LoadVector {
                x: 0.0,
                y: -1.0,
                z: 0.0,
            },
            magnitude: 3.0,
        }];

        let realization = realize_structural_model_to_frame2d(&model).expect("realize model");
        assert!(
            realization
                .diagnostics
                .iter()
                .any(|diag| diag.code == "plate-load-ignored")
        );
    }
}
