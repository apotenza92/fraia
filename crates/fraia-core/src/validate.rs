use crate::catalog::section_by_id;
use crate::structural_app::{AssignmentTargetRef, StructuralModel, StructuralObjectRef};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticCategory {
    Parameters,
    TopologyConnectivity,
    SupportsReleasesConnections,
    SolverRealization,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationDiagnostic {
    pub severity: DiagnosticSeverity,
    pub category: DiagnosticCategory,
    pub code: String,
    pub message: String,
    pub object_refs: Vec<String>,
    pub suggested_actions: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationReport {
    pub diagnostics: Vec<ValidationDiagnostic>,
}

impl ValidationReport {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
    }
}

pub fn validate_structural_model(model: &StructuralModel) -> ValidationReport {
    let mut report = ValidationReport::default();

    if model.nodes.is_empty() {
        push(
            &mut report,
            DiagnosticSeverity::Error,
            DiagnosticCategory::TopologyConnectivity,
            "structural-model.no-nodes",
            "Structural model contains no nodes.",
            vec![],
            vec!["Add at least one node or generate a structural option.".into()],
        );
    }

    if model.members.is_empty() {
        push(
            &mut report,
            DiagnosticSeverity::Warning,
            DiagnosticCategory::TopologyConnectivity,
            "structural-model.no-members",
            "Structural model contains no members.",
            vec![],
            vec!["Add members or generate a structural frame option.".into()],
        );
    }

    if model.supports.is_empty() {
        push(
            &mut report,
            DiagnosticSeverity::Warning,
            DiagnosticCategory::SupportsReleasesConnections,
            "structural-model.no-supports",
            "Structural model contains no supports and may be unstable for analysis.",
            vec![],
            vec!["Add at least one support assignment before analysis.".into()],
        );
    }

    check_unique_ids(
        &mut report,
        "node",
        model.nodes.iter().map(|node| node.id.as_str()),
    );
    check_unique_ids(
        &mut report,
        "member",
        model.members.iter().map(|member| member.id.as_str()),
    );
    check_unique_ids(
        &mut report,
        "plate",
        model.plates.iter().map(|plate| plate.id.as_str()),
    );
    check_unique_ids(
        &mut report,
        "support",
        model.supports.iter().map(|support| support.id.as_str()),
    );
    check_unique_ids(
        &mut report,
        "load",
        model.loads.iter().map(|load| load.id.as_str()),
    );
    check_unique_ids(
        &mut report,
        "release",
        model.releases.iter().map(|release| release.id.as_str()),
    );

    let node_ids: HashSet<&str> = model.nodes.iter().map(|node| node.id.as_str()).collect();
    let member_ids: HashSet<&str> = model
        .members
        .iter()
        .map(|member| member.id.as_str())
        .collect();
    let plate_ids: HashSet<&str> = model.plates.iter().map(|plate| plate.id.as_str()).collect();

    for member in &model.members {
        if !node_ids.contains(member.start_node.as_str()) {
            push(
                &mut report,
                DiagnosticSeverity::Error,
                DiagnosticCategory::TopologyConnectivity,
                "member.start-node.missing",
                &format!(
                    "Member {} references missing start node {}.",
                    member.id, member.start_node
                ),
                vec![format!("member:{}", member.id)],
                vec!["Fix the member start node reference.".into()],
            );
        }
        if !node_ids.contains(member.end_node.as_str()) {
            push(
                &mut report,
                DiagnosticSeverity::Error,
                DiagnosticCategory::TopologyConnectivity,
                "member.end-node.missing",
                &format!(
                    "Member {} references missing end node {}.",
                    member.id, member.end_node
                ),
                vec![format!("member:{}", member.id)],
                vec!["Fix the member end node reference.".into()],
            );
        }
        if section_by_id(&member.section_id).is_none() {
            push(
                &mut report,
                DiagnosticSeverity::Error,
                DiagnosticCategory::Parameters,
                "member.section.unknown",
                &format!(
                    "Member {} references unknown section {}.",
                    member.id, member.section_id
                ),
                vec![format!("member:{}", member.id)],
                vec!["Assign a section from the Fraia section catalog.".into()],
            );
        }
    }

    for plate in &model.plates {
        if plate.boundary_nodes.len() < 3 {
            push(
                &mut report,
                DiagnosticSeverity::Error,
                DiagnosticCategory::TopologyConnectivity,
                "plate.boundary.too-few-nodes",
                &format!("Plate {} has fewer than 3 boundary nodes.", plate.id),
                vec![format!("plate:{}", plate.id)],
                vec!["Use at least 3 boundary nodes for a plate region.".into()],
            );
        }
        for node_id in &plate.boundary_nodes {
            if !node_ids.contains(node_id.as_str()) {
                push(
                    &mut report,
                    DiagnosticSeverity::Error,
                    DiagnosticCategory::TopologyConnectivity,
                    "plate.boundary-node.missing",
                    &format!(
                        "Plate {} references missing boundary node {}.",
                        plate.id, node_id
                    ),
                    vec![format!("plate:{}", plate.id)],
                    vec!["Fix the plate boundary node references.".into()],
                );
            }
        }
    }

    for support in &model.supports {
        if !node_ids.contains(support.target_node.as_str()) {
            push(
                &mut report,
                DiagnosticSeverity::Error,
                DiagnosticCategory::SupportsReleasesConnections,
                "support.target-node.missing",
                &format!(
                    "Support {} references missing target node {}.",
                    support.id, support.target_node
                ),
                vec![format!("support:{}", support.id)],
                vec!["Fix the support target node reference.".into()],
            );
        }
    }

    for load in &model.loads {
        if load.magnitude.abs() <= 1e-9 {
            push(
                &mut report,
                DiagnosticSeverity::Warning,
                DiagnosticCategory::Parameters,
                "load.magnitude.zero",
                &format!("Load {} has zero magnitude.", load.id),
                vec![format!("load:{}", load.id)],
                vec!["Remove the load or give it a non-zero magnitude.".into()],
            );
        }
        if load.direction.x.abs() <= 1e-9
            && load.direction.y.abs() <= 1e-9
            && load.direction.z.abs() <= 1e-9
        {
            push(
                &mut report,
                DiagnosticSeverity::Warning,
                DiagnosticCategory::Parameters,
                "load.direction.zero",
                &format!("Load {} has a zero direction vector.", load.id),
                vec![format!("load:{}", load.id)],
                vec!["Set a non-zero load direction vector.".into()],
            );
        }
        match &load.target {
            AssignmentTargetRef::Node(node_id) => {
                if !node_ids.contains(node_id.as_str()) {
                    push(
                        &mut report,
                        DiagnosticSeverity::Error,
                        DiagnosticCategory::TopologyConnectivity,
                        "load.target-node.missing",
                        &format!("Load {} references missing node {}.", load.id, node_id),
                        vec![format!("load:{}", load.id)],
                        vec!["Fix the load target node reference.".into()],
                    );
                }
                if !load.kind_matches_target() {
                    push(
                        &mut report,
                        DiagnosticSeverity::Error,
                        DiagnosticCategory::Parameters,
                        "load.kind.node-target-mismatch",
                        &format!(
                            "Load {} targets a {} but uses kind {}.",
                            load.id,
                            load.target.kind_label(),
                            load.kind.as_str()
                        ),
                        vec![format!("load:{}", load.id)],
                        vec![format!(
                            "Use the {} load kind for node-targeted authored loads.",
                            load.expected_kind().as_str()
                        )],
                    );
                }
            }
            AssignmentTargetRef::Member(member_id) => {
                if !member_ids.contains(member_id.as_str()) {
                    push(
                        &mut report,
                        DiagnosticSeverity::Error,
                        DiagnosticCategory::TopologyConnectivity,
                        "load.target-member.missing",
                        &format!("Load {} references missing member {}.", load.id, member_id),
                        vec![format!("load:{}", load.id)],
                        vec!["Fix the load target member reference.".into()],
                    );
                }
                if !load.kind_matches_target() {
                    push(
                        &mut report,
                        DiagnosticSeverity::Error,
                        DiagnosticCategory::Parameters,
                        "load.kind.member-target-mismatch",
                        &format!(
                            "Load {} targets a {} but uses kind {}.",
                            load.id,
                            load.target.kind_label(),
                            load.kind.as_str()
                        ),
                        vec![format!("load:{}", load.id)],
                        vec![format!(
                            "Use the {} load kind for current member-targeted authored loads.",
                            load.expected_kind().as_str()
                        )],
                    );
                }
            }
            AssignmentTargetRef::Plate(plate_id) => {
                if !plate_ids.contains(plate_id.as_str()) {
                    push(
                        &mut report,
                        DiagnosticSeverity::Error,
                        DiagnosticCategory::TopologyConnectivity,
                        "load.target-plate.missing",
                        &format!("Load {} references missing plate {}.", load.id, plate_id),
                        vec![format!("load:{}", load.id)],
                        vec!["Fix the load target plate reference.".into()],
                    );
                }
                if !load.kind_matches_target() {
                    push(
                        &mut report,
                        DiagnosticSeverity::Error,
                        DiagnosticCategory::Parameters,
                        "load.kind.plate-target-mismatch",
                        &format!(
                            "Load {} targets a {} but uses kind {}.",
                            load.id,
                            load.target.kind_label(),
                            load.kind.as_str()
                        ),
                        vec![format!("load:{}", load.id)],
                        vec![format!(
                            "Use the {} load kind for current plate-targeted authored loads.",
                            load.expected_kind().as_str()
                        )],
                    );
                }
            }
        }
    }

    for materialization in &model.builder_node_materializations {
        for object_ref in &materialization.object_refs {
            if !structural_object_ref_exists(model, object_ref) {
                push(
                    &mut report,
                    DiagnosticSeverity::Warning,
                    DiagnosticCategory::TopologyConnectivity,
                    "builder-materialization.object-ref.missing",
                    &format!(
                        "Builder node {} references a missing structural object {:?} in its materialization map.",
                        materialization.builder_node_id, object_ref
                    ),
                    vec![format!("builder-node:{}", materialization.builder_node_id)],
                    vec![
                        "Rebuild the authored model from the builder graph or remove stale materialization references."
                            .into(),
                    ],
                );
            }
        }
    }

    for release in &model.releases {
        if !member_ids.contains(release.target.member_id.as_str()) {
            push(
                &mut report,
                DiagnosticSeverity::Error,
                DiagnosticCategory::SupportsReleasesConnections,
                "release.target-member.missing",
                &format!(
                    "Release {} references missing member {}.",
                    release.id, release.target.member_id
                ),
                vec![format!("release:{}", release.id)],
                vec!["Fix the release target member reference.".into()],
            );
        }
    }

    report
}

fn check_unique_ids<'a>(
    report: &mut ValidationReport,
    kind: &str,
    ids: impl Iterator<Item = &'a str>,
) {
    let mut seen = HashSet::new();
    for id in ids {
        if !seen.insert(id.to_string()) {
            push(
                report,
                DiagnosticSeverity::Error,
                DiagnosticCategory::TopologyConnectivity,
                &format!("{}.duplicate-id", kind),
                &format!("Duplicate {} id detected: {}.", kind, id),
                vec![format!("{}:{}", kind, id)],
                vec!["Ensure each authored object id is unique.".into()],
            );
        }
    }
}

fn structural_object_ref_exists(model: &StructuralModel, object_ref: &StructuralObjectRef) -> bool {
    match object_ref {
        StructuralObjectRef::Node(id) => model.nodes.iter().any(|node| &node.id == id),
        StructuralObjectRef::Member(id) => model.members.iter().any(|member| &member.id == id),
        StructuralObjectRef::Plate(id) => model.plates.iter().any(|plate| &plate.id == id),
        StructuralObjectRef::Support(id) => model.supports.iter().any(|support| &support.id == id),
        StructuralObjectRef::Load(id) => model.loads.iter().any(|load| &load.id == id),
        StructuralObjectRef::Release(id) => model.releases.iter().any(|release| &release.id == id),
    }
}

fn push(
    report: &mut ValidationReport,
    severity: DiagnosticSeverity,
    category: DiagnosticCategory,
    code: &str,
    message: &str,
    object_refs: Vec<String>,
    suggested_actions: Vec<String>,
) {
    report.diagnostics.push(ValidationDiagnostic {
        severity,
        category,
        code: code.into(),
        message: message.into(),
        object_refs,
        suggested_actions,
    });
}

#[cfg(test)]
mod tests {
    use super::{DiagnosticSeverity, validate_structural_model};
    use crate::structural_app::{
        AssignmentTargetRef, BuilderNodeMaterialization, LoadAssignment, LoadKind, LoadVector,
        StructuralMember, StructuralModel, StructuralNode, StructuralObjectRef, SupportAssignment,
    };

    #[test]
    fn flags_missing_member_nodes() {
        let mut model = StructuralModel::empty();
        model.nodes.push(StructuralNode {
            id: "n1".into(),
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });
        model.members.push(StructuralMember {
            id: "m1".into(),
            start_node: "n1".into(),
            end_node: "n2".into(),
            role: "beam".into(),
            semantic_tags: Vec::new(),
            section_id: "310UB".into(),
            material_id: "steel".into(),
        });
        let report = validate_structural_model(&model);
        assert!(report.has_errors());
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.code == "member.end-node.missing")
        );
    }

    #[test]
    fn warns_when_supports_are_missing() {
        let mut model = StructuralModel::empty();
        model.nodes.push(StructuralNode {
            id: "n1".into(),
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });
        model.members.push(StructuralMember {
            id: "m1".into(),
            start_node: "n1".into(),
            end_node: "n1".into(),
            role: "beam".into(),
            semantic_tags: Vec::new(),
            section_id: "310UB".into(),
            material_id: "steel".into(),
        });
        let report = validate_structural_model(&model);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.code == "structural-model.no-supports"
                    && d.severity == DiagnosticSeverity::Warning)
        );
    }

    #[test]
    fn valid_minimal_model_has_no_errors() {
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
        ];
        model.members.push(StructuralMember {
            id: "m1".into(),
            start_node: "n1".into(),
            end_node: "n2".into(),
            role: "beam".into(),
            semantic_tags: Vec::new(),
            section_id: "310UB".into(),
            material_id: "steel".into(),
        });
        model.supports.push(SupportAssignment {
            id: "s1".into(),
            target_node: "n1".into(),
            ux: true,
            uy: true,
            uz: false,
            rx: false,
            ry: false,
            rz: true,
        });
        let report = validate_structural_model(&model);
        assert!(!report.has_errors());
    }

    #[test]
    fn warns_when_builder_materialization_refs_are_stale() {
        let mut model = StructuralModel::empty();
        model.nodes.push(StructuralNode {
            id: "n1".into(),
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });
        model
            .builder_node_materializations
            .push(BuilderNodeMaterialization {
                builder_node_id: "builder-root".into(),
                object_refs: vec![
                    StructuralObjectRef::Node("n1".into()),
                    StructuralObjectRef::Member("missing-member".into()),
                ],
            });

        let report = validate_structural_model(&model);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.code == "builder-materialization.object-ref.missing")
        );
    }

    #[test]
    fn flags_load_kind_target_mismatches() {
        let mut model = StructuralModel::empty();
        model.nodes.push(StructuralNode {
            id: "n1".into(),
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });
        model.nodes.push(StructuralNode {
            id: "n2".into(),
            x: 4.0,
            y: 0.0,
            z: 0.0,
        });
        model.members.push(StructuralMember {
            id: "m1".into(),
            start_node: "n1".into(),
            end_node: "n2".into(),
            role: "beam".into(),
            semantic_tags: Vec::new(),
            section_id: "310UB".into(),
            material_id: "steel".into(),
        });
        model.supports.push(SupportAssignment {
            id: "s1".into(),
            target_node: "n1".into(),
            ux: true,
            uy: true,
            uz: false,
            rx: false,
            ry: false,
            rz: true,
        });
        model.loads.push(LoadAssignment {
            id: "load-1".into(),
            target: AssignmentTargetRef::Node("n1".into()),
            load_case_id: "gravity".into(),
            kind: LoadKind::UniformLine,
            direction: LoadVector {
                x: 0.0,
                y: -1.0,
                z: 0.0,
            },
            magnitude: 10_000.0,
        });

        let report = validate_structural_model(&model);
        assert!(report.has_errors());
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.code == "load.kind.node-target-mismatch")
        );
    }

    #[test]
    fn accepts_area_loads_on_plate_targets() {
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
        model.plates.push(crate::structural_app::StructuralPlate {
            id: "p1".into(),
            boundary_nodes: vec!["n1".into(), "n2".into(), "n3".into(), "n4".into()],
            role: "slab".into(),
            semantic_tags: Vec::new(),
            thickness_m: 0.2,
            material_id: "steel".into(),
            generated_from: "test".into(),
        });
        model.loads.push(LoadAssignment {
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
        });

        let report = validate_structural_model(&model);
        assert!(
            !report
                .diagnostics
                .iter()
                .any(|d| d.code == "load.kind.plate-target-mismatch")
        );
        assert!(!report.has_errors());
    }
}
