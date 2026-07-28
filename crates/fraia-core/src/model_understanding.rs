use crate::structural_app::{
    AssignmentTargetRef, LoadVector, StructuralModel, StructuralObjectRef,
};
use crate::validate::{ValidationDiagnostic, validate_structural_model};
use crate::{section_catalog, section_family};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUnderstandingReport {
    pub dimension: String,
    pub counts: ModelObjectCounts,
    pub bounds: Option<ModelBounds>,
    pub member_roles: Vec<MemberRoleSummary>,
    pub member_groups: Vec<MemberGroupUnderstanding>,
    pub members: Vec<MemberUnderstanding>,
    pub plates: Vec<PlateUnderstanding>,
    pub supports: Vec<SupportUnderstanding>,
    pub loads: Vec<LoadUnderstanding>,
    pub unresolved_objects: Vec<UnresolvedObjectUnderstanding>,
    pub builder_materializations: Vec<BuilderMaterializationSummary>,
    pub validation: ModelValidationUnderstanding,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelObjectCounts {
    pub nodes: usize,
    pub members: usize,
    pub plates: usize,
    pub supports: usize,
    pub loads: usize,
    pub releases: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelBounds {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
    pub min_z: f64,
    pub max_z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberRoleSummary {
    pub role: String,
    pub count: usize,
    pub total_length_m: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberUnderstanding {
    pub id: String,
    pub display_label: String,
    pub start_node: String,
    pub end_node: String,
    pub role: String,
    pub semantic_tags: Vec<String>,
    pub section_id: String,
    pub material_id: String,
    pub length_m: f64,
    pub generated_by: Option<String>,
    pub recommended_section_families: Vec<String>,
    pub derived_tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberGroupUnderstanding {
    pub id: String,
    pub display_label: String,
    pub member_ids: Vec<String>,
    pub start_node: String,
    pub end_node: String,
    pub role: String,
    pub semantic_tags: Vec<String>,
    pub section_id: String,
    pub material_id: String,
    pub length_m: f64,
    pub generated_by: Option<String>,
    pub recommended_section_families: Vec<String>,
    pub derived_tags: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
struct MemberDirection {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Debug, Clone, Copy)]
enum ChainEnd {
    Start,
    End,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlateUnderstanding {
    pub id: String,
    pub display_label: String,
    pub boundary_nodes: Vec<String>,
    pub role: String,
    pub semantic_tags: Vec<String>,
    pub thickness_m: f64,
    pub material_id: String,
    pub generated_from: String,
    pub generated_by: Option<String>,
    pub derived_tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnresolvedObjectUnderstanding {
    pub object_kind: String,
    pub object_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportUnderstanding {
    pub id: String,
    pub target_node: String,
    pub restrained_dofs: Vec<String>,
    pub generated_by: Option<String>,
    pub derived_tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadUnderstanding {
    pub id: String,
    pub target_kind: String,
    pub target_id: String,
    pub load_case_id: String,
    pub kind: String,
    pub magnitude: f64,
    pub direction: LoadVector,
    pub generated_by: Option<String>,
    pub derived_tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuilderMaterializationSummary {
    pub builder_node_id: String,
    pub object_count: usize,
    pub derived_tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelValidationUnderstanding {
    pub error_count: usize,
    pub warning_count: usize,
    pub diagnostics: Vec<ValidationDiagnostic>,
}

pub fn understand_structural_model(model: &StructuralModel) -> ModelUnderstandingReport {
    let validation = validate_structural_model(model);
    let members = model
        .members
        .iter()
        .map(|member| {
            let length_m = member_length_m(model, &member.start_node, &member.end_node);
            let generated_by = model
                .generated_by_builder_node_for_object(&StructuralObjectRef::Member(
                    member.id.clone(),
                ))
                .map(str::to_owned);
            let recommended_section_families =
                recommended_section_families_for_member(&member.section_id);
            let mut derived_tags = vec![
                "object:member".into(),
                format!("role:{}", member.role),
                format!("section:{}", member.section_id),
                format!("material:{}", member.material_id),
            ];
            append_semantic_tags(&mut derived_tags, &member.semantic_tags);
            if let Some(builder_node_id) = &generated_by {
                derived_tags.push(format!("builder:{}", builder_node_id));
            }
            MemberUnderstanding {
                id: member.id.clone(),
                display_label: format_structural_object_label(
                    None,
                    &member.role,
                    "member",
                    &member.id,
                ),
                start_node: member.start_node.clone(),
                end_node: member.end_node.clone(),
                role: member.role.clone(),
                semantic_tags: member.semantic_tags.clone(),
                section_id: member.section_id.clone(),
                material_id: member.material_id.clone(),
                length_m,
                generated_by,
                recommended_section_families,
                derived_tags,
            }
        })
        .collect();

    let plates = model
        .plates
        .iter()
        .map(|plate| {
            let generated_by = model
                .generated_by_builder_node_for_object(&StructuralObjectRef::Plate(plate.id.clone()))
                .map(str::to_owned);
            let mut derived_tags = vec![
                "object:plate".into(),
                format!("role:{}", plate.role),
                format!("material:{}", plate.material_id),
                format!("generated_from:{}", plate.generated_from),
            ];
            append_semantic_tags(&mut derived_tags, &plate.semantic_tags);
            if let Some(builder_node_id) = &generated_by {
                derived_tags.push(format!("builder:{}", builder_node_id));
            }
            PlateUnderstanding {
                id: plate.id.clone(),
                display_label: format_structural_object_label(
                    None,
                    &plate.role,
                    "plate",
                    &plate.id,
                ),
                boundary_nodes: plate.boundary_nodes.clone(),
                role: plate.role.clone(),
                semantic_tags: plate.semantic_tags.clone(),
                thickness_m: plate.thickness_m,
                material_id: plate.material_id.clone(),
                generated_from: plate.generated_from.clone(),
                generated_by,
                derived_tags,
            }
        })
        .collect();

    let supports = model
        .supports
        .iter()
        .map(|support| {
            let restrained_dofs = support_restrained_dofs(support);
            let generated_by = model
                .generated_by_builder_node_for_object(&StructuralObjectRef::Support(
                    support.id.clone(),
                ))
                .map(str::to_owned);
            let mut derived_tags = vec![
                "object:support".into(),
                format!("target_node:{}", support.target_node),
            ];
            for dof in &restrained_dofs {
                derived_tags.push(format!("restrained:{}", dof));
            }
            if let Some(builder_node_id) = &generated_by {
                derived_tags.push(format!("builder:{}", builder_node_id));
            }
            SupportUnderstanding {
                id: support.id.clone(),
                target_node: support.target_node.clone(),
                restrained_dofs,
                generated_by,
                derived_tags,
            }
        })
        .collect();

    let loads = model
        .loads
        .iter()
        .map(|load| {
            let (target_kind, target_id) = target_parts(&load.target);
            let generated_by = model
                .generated_by_builder_node_for_object(&StructuralObjectRef::Load(load.id.clone()))
                .map(str::to_owned);
            let mut derived_tags = vec![
                "object:load".into(),
                format!("kind:{}", load.kind.as_str()),
                format!("target:{}:{}", target_kind, target_id),
                format!("load_case:{}", load.load_case_id),
            ];
            if let Some(builder_node_id) = &generated_by {
                derived_tags.push(format!("builder:{}", builder_node_id));
            }
            LoadUnderstanding {
                id: load.id.clone(),
                target_kind,
                target_id,
                load_case_id: load.load_case_id.clone(),
                kind: load.kind.as_str().into(),
                magnitude: load.magnitude,
                direction: load.direction.clone(),
                generated_by,
                derived_tags,
            }
        })
        .collect();

    let member_groups = continuous_member_groups(model);
    let member_roles = member_groups
        .iter()
        .fold(
            BTreeMap::<String, (usize, f64)>::new(),
            |mut counts, group| {
                let entry = counts.entry(group.role.clone()).or_default();
                entry.0 += 1;
                entry.1 += group.length_m;
                counts
            },
        )
        .into_iter()
        .map(|(role, (count, total_length_m))| MemberRoleSummary {
            role,
            count,
            total_length_m,
        })
        .collect();
    let builder_materializations = model
        .builder_node_materializations
        .iter()
        .map(|entry| BuilderMaterializationSummary {
            builder_node_id: entry.builder_node_id.clone(),
            object_count: entry.object_refs.len(),
            derived_tags: vec![
                "object:builder_materialization".into(),
                format!("builder:{}", entry.builder_node_id),
            ],
        })
        .collect();
    let error_count = validation
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == crate::validate::DiagnosticSeverity::Error)
        .count();
    let warning_count = validation
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == crate::validate::DiagnosticSeverity::Warning)
        .count();
    let unresolved_objects = unresolved_objects(model);

    ModelUnderstandingReport {
        dimension: model.dimension.clone(),
        counts: ModelObjectCounts {
            nodes: model.nodes.len(),
            members: model.members.len(),
            plates: model.plates.len(),
            supports: model.supports.len(),
            loads: model.loads.len(),
            releases: model.releases.len(),
        },
        bounds: model_bounds(model),
        member_roles,
        member_groups,
        members,
        plates,
        supports,
        loads,
        unresolved_objects,
        builder_materializations,
        validation: ModelValidationUnderstanding {
            error_count,
            warning_count,
            diagnostics: validation.diagnostics,
        },
    }
}

fn append_semantic_tags(derived_tags: &mut Vec<String>, semantic_tags: &[String]) {
    for tag in semantic_tags {
        let trimmed = tag.trim();
        if !trimmed.is_empty() {
            derived_tags.push(format!("tag:{}", trimmed));
        }
    }
}

pub fn format_semantic_role(value: &str) -> String {
    let text = value
        .trim()
        .replace(['_', '-'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase();
    if text.is_empty() {
        return String::new();
    }
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    format!("{}{}", first.to_uppercase(), chars.collect::<String>())
}

pub fn format_structural_object_label(
    display_name: Option<&str>,
    role: &str,
    object_type: &str,
    id: &str,
) -> String {
    let display_name = display_name.unwrap_or_default().trim();
    if !display_name.is_empty() {
        return display_name.into();
    }
    let role = format_semantic_role(role);
    if !role.is_empty() {
        return format!("{role} {id}");
    }
    let object_type = format_semantic_role(object_type);
    if object_type.is_empty() {
        id.into()
    } else {
        format!("{object_type} {id}")
    }
}

fn recommended_section_families_for_member(section_id: &str) -> Vec<String> {
    if let Some(family) = section_family(section_id) {
        return vec![family.to_owned()];
    }
    available_catalog_section_families()
}

fn available_catalog_section_families() -> Vec<String> {
    let mut families = BTreeSet::new();
    for section in section_catalog() {
        if let Some(family) = section_family(&section.id) {
            families.insert(family.to_owned());
        }
    }
    families.into_iter().collect()
}

fn unresolved_objects(model: &StructuralModel) -> Vec<UnresolvedObjectUnderstanding> {
    let mut unresolved = Vec::new();
    for member in &model.members {
        if is_unresolved_role(&member.role) {
            unresolved.push(UnresolvedObjectUnderstanding {
                object_kind: "member".into(),
                object_id: member.id.clone(),
                reason: "primary role is unclassified".into(),
            });
        }
    }
    for plate in &model.plates {
        if is_unresolved_role(&plate.role) {
            unresolved.push(UnresolvedObjectUnderstanding {
                object_kind: "plate".into(),
                object_id: plate.id.clone(),
                reason: "primary role is unclassified".into(),
            });
        }
    }
    unresolved
}

fn is_unresolved_role(role: &str) -> bool {
    matches!(
        role.trim().to_ascii_lowercase().as_str(),
        "" | "unclassified"
    )
}

fn continuous_member_groups(model: &StructuralModel) -> Vec<MemberGroupUnderstanding> {
    let mut used = vec![false; model.members.len()];
    let mut groups = Vec::new();

    for index in 0..model.members.len() {
        if used[index] {
            continue;
        }

        used[index] = true;
        let seed = &model.members[index];
        let mut member_indices = vec![index];
        let mut start_node = seed.start_node.clone();
        let mut end_node = seed.end_node.clone();
        let Some(direction) = member_direction(model, seed) else {
            continue;
        };

        extend_member_chain(
            model,
            &mut used,
            &mut member_indices,
            &mut start_node,
            &mut end_node,
            direction,
            ChainEnd::Start,
        );
        extend_member_chain(
            model,
            &mut used,
            &mut member_indices,
            &mut start_node,
            &mut end_node,
            direction,
            ChainEnd::End,
        );

        member_indices.sort_unstable();
        let member_ids: Vec<String> = member_indices
            .iter()
            .map(|member_index| model.members[*member_index].id.clone())
            .collect();
        let length_m = member_indices
            .iter()
            .map(|member_index| {
                let member = &model.members[*member_index];
                member_length_m(model, &member.start_node, &member.end_node)
            })
            .sum();
        let generated_by = model
            .generated_by_builder_node_for_object(&StructuralObjectRef::Member(seed.id.clone()))
            .map(str::to_owned);
        let recommended_section_families =
            recommended_section_families_for_member(&seed.section_id);
        let mut derived_tags = vec![
            "object:member_group".into(),
            format!("role:{}", seed.role),
            format!("section:{}", seed.section_id),
            format!("material:{}", seed.material_id),
        ];
        append_semantic_tags(&mut derived_tags, &seed.semantic_tags);
        if let Some(builder_node_id) = &generated_by {
            derived_tags.push(format!("builder:{}", builder_node_id));
        }

        groups.push(MemberGroupUnderstanding {
            id: format!("member-group-{}", seed.id),
            display_label: format_structural_object_label(
                None,
                &seed.role,
                "member",
                &member_ids
                    .first()
                    .cloned()
                    .unwrap_or_else(|| seed.id.clone()),
            ),
            member_ids,
            start_node,
            end_node,
            role: seed.role.clone(),
            semantic_tags: seed.semantic_tags.clone(),
            section_id: seed.section_id.clone(),
            material_id: seed.material_id.clone(),
            length_m,
            generated_by,
            recommended_section_families,
            derived_tags,
        });
    }

    groups
}

fn extend_member_chain(
    model: &StructuralModel,
    used: &mut [bool],
    member_indices: &mut Vec<usize>,
    start_node: &mut String,
    end_node: &mut String,
    direction: MemberDirection,
    chain_end: ChainEnd,
) {
    loop {
        let node_id = match chain_end {
            ChainEnd::Start => start_node.clone(),
            ChainEnd::End => end_node.clone(),
        };
        let seed = &model.members[member_indices[0]];
        let candidates: Vec<usize> = model
            .members
            .iter()
            .enumerate()
            .filter(|(index, member)| {
                !used[*index]
                    && member_compatible(seed, member)
                    && (member.start_node == node_id || member.end_node == node_id)
                    && member_collinear(model, member, direction)
            })
            .map(|(index, _)| index)
            .collect();

        if candidates.len() != 1 {
            break;
        }

        let index = candidates[0];
        used[index] = true;
        let member = &model.members[index];
        match chain_end {
            ChainEnd::Start => {
                *start_node = if member.start_node == node_id {
                    member.end_node.clone()
                } else {
                    member.start_node.clone()
                };
            }
            ChainEnd::End => {
                *end_node = if member.start_node == node_id {
                    member.end_node.clone()
                } else {
                    member.start_node.clone()
                };
            }
        }
        member_indices.push(index);
    }
}

fn member_compatible(
    a: &crate::structural_app::StructuralMember,
    b: &crate::structural_app::StructuralMember,
) -> bool {
    a.role == b.role
        && a.semantic_tags == b.semantic_tags
        && a.section_id == b.section_id
        && a.material_id == b.material_id
}

fn member_direction(
    model: &StructuralModel,
    member: &crate::structural_app::StructuralMember,
) -> Option<MemberDirection> {
    let start = model.node_by_id(&member.start_node)?;
    let end = model.node_by_id(&member.end_node)?;
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let dz = end.z - start.z;
    let len = (dx * dx + dy * dy + dz * dz).sqrt();
    (len > 1e-9).then_some(MemberDirection {
        x: dx / len,
        y: dy / len,
        z: dz / len,
    })
}

fn member_collinear(
    model: &StructuralModel,
    member: &crate::structural_app::StructuralMember,
    direction: MemberDirection,
) -> bool {
    let Some(candidate) = member_direction(model, member) else {
        return false;
    };
    let cross_x = direction.y * candidate.z - direction.z * candidate.y;
    let cross_y = direction.z * candidate.x - direction.x * candidate.z;
    let cross_z = direction.x * candidate.y - direction.y * candidate.x;
    (cross_x * cross_x + cross_y * cross_y + cross_z * cross_z).sqrt() < 1e-6
}

fn member_length_m(model: &StructuralModel, start_node: &str, end_node: &str) -> f64 {
    let Some(start) = model.node_by_id(start_node) else {
        return 0.0;
    };
    let Some(end) = model.node_by_id(end_node) else {
        return 0.0;
    };
    ((end.x - start.x).powi(2) + (end.y - start.y).powi(2) + (end.z - start.z).powi(2)).sqrt()
}

fn model_bounds(model: &StructuralModel) -> Option<ModelBounds> {
    let first = model.nodes.first()?;
    let mut bounds = ModelBounds {
        min_x: first.x,
        max_x: first.x,
        min_y: first.y,
        max_y: first.y,
        min_z: first.z,
        max_z: first.z,
    };
    for node in &model.nodes {
        bounds.min_x = bounds.min_x.min(node.x);
        bounds.max_x = bounds.max_x.max(node.x);
        bounds.min_y = bounds.min_y.min(node.y);
        bounds.max_y = bounds.max_y.max(node.y);
        bounds.min_z = bounds.min_z.min(node.z);
        bounds.max_z = bounds.max_z.max(node.z);
    }
    Some(bounds)
}

fn support_restrained_dofs(support: &crate::structural_app::SupportAssignment) -> Vec<String> {
    [
        ("ux", support.ux),
        ("uy", support.uy),
        ("uz", support.uz),
        ("rx", support.rx),
        ("ry", support.ry),
        ("rz", support.rz),
    ]
    .into_iter()
    .filter_map(|(label, restrained)| restrained.then_some(label.to_owned()))
    .collect()
}

fn target_parts(target: &AssignmentTargetRef) -> (String, String) {
    match target {
        AssignmentTargetRef::Node(id) => ("node".into(), id.clone()),
        AssignmentTargetRef::Member(id) => ("member".into(), id.clone()),
        AssignmentTargetRef::Plate(id) => ("plate".into(), id.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        format_semantic_role, format_structural_object_label, understand_structural_model,
    };
    use crate::{create_project, seed_simply_supported_beam_in_project};

    #[test]
    fn understanding_report_derives_roles_tags_and_validation_counts() {
        let temp_dir = std::env::temp_dir().join(format!(
            "fraia-understand-test-{}",
            crate::utils::timestamp_id()
        ));
        let (mut project, _) = create_project(&temp_dir, "understand").expect("create");
        project.requirements.span_m = 6.0;
        project.requirements.gravity_load_kn_per_m = 8.0;
        seed_simply_supported_beam_in_project(&mut project, Some("builder.beam.test"))
            .expect("seed");
        let model = project.structural_model.as_ref().expect("structural model");

        let report = understand_structural_model(model);

        assert_eq!(report.counts.supports, 2);
        assert!(report.member_roles.iter().any(|role| role.role == "beam"));
        assert_eq!(report.member_groups.len(), 1);
        assert_eq!(
            report.member_groups[0].member_ids.len(),
            report.members.len()
        );
        assert_eq!(report.member_roles[0].count, 1);
        assert!(
            report
                .members
                .iter()
                .all(|member| member.derived_tags.iter().any(|tag| tag == "role:beam"))
        );
        assert!(
            report
                .members
                .iter()
                .all(|member| member.derived_tags.iter().any(|tag| tag == "tag:floor"))
        );
        assert!(
            report
                .members
                .iter()
                .all(|member| member.recommended_section_families.contains(&"UB".into()))
        );
        assert!(
            report
                .members
                .iter()
                .all(|member| member.display_label.starts_with("Beam "))
        );
        assert!(report.member_groups[0].display_label.starts_with("Beam "));
        assert_eq!(report.validation.error_count, 0);

        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn semantic_role_formatter_is_generic_sentence_case() {
        assert_eq!(format_semantic_role("beam"), "Beam");
        assert_eq!(format_semantic_role("wall_panel"), "Wall panel");
        assert_eq!(format_semantic_role("roof-panel"), "Roof panel");
        assert_eq!(
            format_semantic_role("primary_roof_beam"),
            "Primary roof beam"
        );
        assert_eq!(format_semantic_role(""), "");
        assert_eq!(
            format_structural_object_label(None, "", "member", "m1"),
            "Member m1"
        );
        assert_eq!(
            format_structural_object_label(Some("Main roof beam"), "beam", "member", "B1"),
            "Main roof beam"
        );
    }
}
