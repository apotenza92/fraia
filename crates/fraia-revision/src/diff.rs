//! Engineering-language comparisons between authored structural models.

use fraia_core::{AssignmentTargetRef, StructuralModel};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};

/// The engineering concerns affected by a model change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DiffCategory {
    Geometry,
    Topology,
    Member,
    Plate,
    Support,
    Load,
    Release,
    Role,
}

/// The lifecycle action represented by a semantic change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DiffAction {
    Added,
    Updated,
    Removed,
}

/// One deterministic, human-readable engineering change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticChange {
    pub object_kind: String,
    pub object_id: String,
    pub action: DiffAction,
    pub categories: BTreeSet<DiffCategory>,
    pub description: String,
}

/// A stable engineering summary of the difference between two model snapshots.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticDiff {
    pub changes: Vec<SemanticChange>,
}

impl SemanticDiff {
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    pub fn affects(&self, category: DiffCategory) -> bool {
        self.changes
            .iter()
            .any(|change| change.categories.contains(&category))
    }
}

/// Produces a deterministic diff expressed in structural-engineering terms.
pub fn semantic_diff(before: &StructuralModel, after: &StructuralModel) -> SemanticDiff {
    let mut changes = Vec::new();

    diff_nodes(before, after, &mut changes);
    diff_members(before, after, &mut changes);
    diff_plates(before, after, &mut changes);
    diff_supports(before, after, &mut changes);
    diff_loads(before, after, &mut changes);
    diff_releases(before, after, &mut changes);

    changes.sort_by(|left, right| {
        (
            &left.object_kind,
            &left.object_id,
            left.action,
            &left.description,
        )
            .cmp(&(
                &right.object_kind,
                &right.object_id,
                right.action,
                &right.description,
            ))
    });
    SemanticDiff { changes }
}

fn diff_nodes(
    before: &StructuralModel,
    after: &StructuralModel,
    changes: &mut Vec<SemanticChange>,
) {
    let before_by_id: HashMap<_, _> = before
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect();
    let after_by_id: HashMap<_, _> = after
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect();
    let ids: BTreeSet<_> = before_by_id
        .keys()
        .chain(after_by_id.keys())
        .copied()
        .collect();

    for id in ids {
        match (before_by_id.get(id), after_by_id.get(id)) {
            (None, Some(_)) => push(
                changes,
                "node",
                id,
                DiffAction::Added,
                [DiffCategory::Geometry, DiffCategory::Topology],
                format!(
                    "Added node {id} at ({:.3} m, {:.3} m, {:.3} m).",
                    after_by_id[id].x, after_by_id[id].y, after_by_id[id].z
                ),
            ),
            (Some(_), None) => push(
                changes,
                "node",
                id,
                DiffAction::Removed,
                [DiffCategory::Geometry, DiffCategory::Topology],
                format!("Removed node {id}."),
            ),
            (Some(old), Some(new)) if old.x != new.x || old.y != new.y || old.z != new.z => {
                push(
                    changes,
                    "node",
                    id,
                    DiffAction::Updated,
                    [DiffCategory::Geometry],
                    format!("Moved node {id}."),
                );
                for member in after
                    .members
                    .iter()
                    .filter(|member| member.start_node == id || member.end_node == id)
                {
                    push(
                        changes,
                        "member",
                        &member.id,
                        DiffAction::Updated,
                        [DiffCategory::Geometry, DiffCategory::Member],
                        format!(
                            "Member {} geometry changed because node {id} moved.",
                            member.id
                        ),
                    );
                }
            }
            _ => {}
        }
    }
}

fn diff_members(
    before: &StructuralModel,
    after: &StructuralModel,
    changes: &mut Vec<SemanticChange>,
) {
    let before_by_id: HashMap<_, _> = before
        .members
        .iter()
        .map(|member| (member.id.as_str(), member))
        .collect();
    let after_by_id: HashMap<_, _> = after
        .members
        .iter()
        .map(|member| (member.id.as_str(), member))
        .collect();
    let ids: BTreeSet<_> = before_by_id
        .keys()
        .chain(after_by_id.keys())
        .copied()
        .collect();
    for id in ids {
        match (before_by_id.get(id), after_by_id.get(id)) {
            (None, Some(_)) => push(
                changes,
                "member",
                id,
                DiffAction::Added,
                [DiffCategory::Member, DiffCategory::Topology],
                format!(
                    "Added {} member {id} between nodes {} and {} using section {}.",
                    after_by_id[id].role,
                    after_by_id[id].start_node,
                    after_by_id[id].end_node,
                    after_by_id[id].section_id
                ),
            ),
            (Some(_), None) => push(
                changes,
                "member",
                id,
                DiffAction::Removed,
                [DiffCategory::Member, DiffCategory::Topology],
                format!("Removed member {id}."),
            ),
            (Some(old), Some(new)) => {
                if old.start_node != new.start_node || old.end_node != new.end_node {
                    push(
                        changes,
                        "member",
                        id,
                        DiffAction::Updated,
                        [
                            DiffCategory::Member,
                            DiffCategory::Topology,
                            DiffCategory::Geometry,
                        ],
                        format!("Changed member {id} connectivity."),
                    );
                }
                if old.role != new.role {
                    push(
                        changes,
                        "member",
                        id,
                        DiffAction::Updated,
                        [DiffCategory::Member, DiffCategory::Role],
                        format!(
                            "Changed member {id} role from {} to {}.",
                            old.role, new.role
                        ),
                    );
                }
                if old.section_id != new.section_id {
                    push(
                        changes,
                        "member",
                        id,
                        DiffAction::Updated,
                        [DiffCategory::Member],
                        format!(
                            "Changed member {id} section from {} to {}.",
                            old.section_id, new.section_id
                        ),
                    );
                }
                if old.material_id != new.material_id || old.semantic_tags != new.semantic_tags {
                    push(
                        changes,
                        "member",
                        id,
                        DiffAction::Updated,
                        [DiffCategory::Member],
                        format!("Changed member {id} material or semantic tags."),
                    );
                }
            }
            _ => {}
        }
    }
}

fn diff_plates(
    before: &StructuralModel,
    after: &StructuralModel,
    changes: &mut Vec<SemanticChange>,
) {
    let before_by_id: HashMap<_, _> = before
        .plates
        .iter()
        .map(|plate| (plate.id.as_str(), plate))
        .collect();
    let after_by_id: HashMap<_, _> = after
        .plates
        .iter()
        .map(|plate| (plate.id.as_str(), plate))
        .collect();
    let ids: BTreeSet<_> = before_by_id
        .keys()
        .chain(after_by_id.keys())
        .copied()
        .collect();
    for id in ids {
        match (before_by_id.get(id), after_by_id.get(id)) {
            (None, Some(_)) => push(
                changes,
                "plate",
                id,
                DiffAction::Added,
                [DiffCategory::Plate, DiffCategory::Topology],
                format!(
                    "Added {} plate {id} with {} boundary nodes and {:.3} m thickness.",
                    after_by_id[id].role,
                    after_by_id[id].boundary_nodes.len(),
                    after_by_id[id].thickness_m
                ),
            ),
            (Some(_), None) => push(
                changes,
                "plate",
                id,
                DiffAction::Removed,
                [DiffCategory::Plate, DiffCategory::Topology],
                format!("Removed plate {id}."),
            ),
            (Some(old), Some(new)) => {
                if old.boundary_nodes != new.boundary_nodes {
                    push(
                        changes,
                        "plate",
                        id,
                        DiffAction::Updated,
                        [DiffCategory::Plate, DiffCategory::Topology],
                        format!("Changed plate {id} boundary nodes."),
                    );
                }
                if old.role != new.role {
                    push(
                        changes,
                        "plate",
                        id,
                        DiffAction::Updated,
                        [DiffCategory::Plate, DiffCategory::Role],
                        format!("Changed plate {id} role from {} to {}.", old.role, new.role),
                    );
                }
                if old.thickness_m != new.thickness_m
                    || old.material_id != new.material_id
                    || old.semantic_tags != new.semantic_tags
                {
                    push(
                        changes,
                        "plate",
                        id,
                        DiffAction::Updated,
                        [DiffCategory::Plate],
                        format!("Changed plate {id} thickness, material, or semantic tags."),
                    );
                }
            }
            _ => {}
        }
    }
}

fn diff_supports(
    before: &StructuralModel,
    after: &StructuralModel,
    changes: &mut Vec<SemanticChange>,
) {
    let old: HashMap<_, _> = before
        .supports
        .iter()
        .map(|value| (value.id.as_str(), value))
        .collect();
    let new: HashMap<_, _> = after
        .supports
        .iter()
        .map(|value| (value.id.as_str(), value))
        .collect();
    for id in old
        .keys()
        .chain(new.keys())
        .copied()
        .collect::<BTreeSet<_>>()
    {
        match (old.get(id), new.get(id)) {
            (None, Some(_)) => push(
                changes,
                "support",
                id,
                DiffAction::Added,
                [DiffCategory::Support],
                format!("Added support {id} at node {}.", new[id].target_node),
            ),
            (Some(_), None) => push(
                changes,
                "support",
                id,
                DiffAction::Removed,
                [DiffCategory::Support],
                format!("Removed support {id}."),
            ),
            (Some(old), Some(new)) if old.target_node != new.target_node => push(
                changes,
                "support",
                id,
                DiffAction::Updated,
                [DiffCategory::Support, DiffCategory::Topology],
                format!("Moved support {id} to node {}.", new.target_node),
            ),
            (Some(old), Some(new))
                if old.ux != new.ux
                    || old.uy != new.uy
                    || old.uz != new.uz
                    || old.rx != new.rx
                    || old.ry != new.ry
                    || old.rz != new.rz =>
            {
                push(
                    changes,
                    "support",
                    id,
                    DiffAction::Updated,
                    [DiffCategory::Support],
                    format!("Changed restraint conditions for support {id}."),
                )
            }
            _ => {}
        }
    }
}

fn diff_loads(
    before: &StructuralModel,
    after: &StructuralModel,
    changes: &mut Vec<SemanticChange>,
) {
    let old: HashMap<_, _> = before
        .loads
        .iter()
        .map(|value| (value.id.as_str(), value))
        .collect();
    let new: HashMap<_, _> = after
        .loads
        .iter()
        .map(|value| (value.id.as_str(), value))
        .collect();
    for id in old
        .keys()
        .chain(new.keys())
        .copied()
        .collect::<BTreeSet<_>>()
    {
        match (old.get(id), new.get(id)) {
            (None, Some(_)) => push(
                changes,
                "load",
                id,
                DiffAction::Added,
                [DiffCategory::Load],
                format!(
                    "Added {} load {id} on {} with magnitude {:.6} in load case {}.",
                    new[id].kind.as_str(),
                    assignment_target_label(&new[id].target),
                    new[id].magnitude,
                    new[id].load_case_id
                ),
            ),
            (Some(_), None) => push(
                changes,
                "load",
                id,
                DiffAction::Removed,
                [DiffCategory::Load],
                format!("Removed load {id}."),
            ),
            (Some(old), Some(new))
                if old.target.kind_label() != new.target.kind_label()
                    || format!("{:?}", old.target) != format!("{:?}", new.target) =>
            {
                push(
                    changes,
                    "load",
                    id,
                    DiffAction::Updated,
                    [DiffCategory::Load, DiffCategory::Topology],
                    format!("Changed load {id} target."),
                )
            }
            (Some(old), Some(new))
                if old.load_case_id != new.load_case_id
                    || old.kind != new.kind
                    || old.direction.x != new.direction.x
                    || old.direction.y != new.direction.y
                    || old.direction.z != new.direction.z
                    || old.magnitude != new.magnitude =>
            {
                push(
                    changes,
                    "load",
                    id,
                    DiffAction::Updated,
                    [DiffCategory::Load],
                    format!("Changed load {id} magnitude, direction, or case."),
                )
            }
            _ => {}
        }
    }
}

fn diff_releases(
    before: &StructuralModel,
    after: &StructuralModel,
    changes: &mut Vec<SemanticChange>,
) {
    let old: HashMap<_, _> = before
        .releases
        .iter()
        .map(|value| (value.id.as_str(), value))
        .collect();
    let new: HashMap<_, _> = after
        .releases
        .iter()
        .map(|value| (value.id.as_str(), value))
        .collect();
    for id in old
        .keys()
        .chain(new.keys())
        .copied()
        .collect::<BTreeSet<_>>()
    {
        match (old.get(id), new.get(id)) {
            (None, Some(_)) => push(
                changes,
                "release",
                id,
                DiffAction::Added,
                [DiffCategory::Release],
                format!(
                    "Added member-end release {id} on member {} ({} end).",
                    new[id].target.member_id,
                    release_end_label(&new[id].target.end)
                ),
            ),
            (Some(_), None) => push(
                changes,
                "release",
                id,
                DiffAction::Removed,
                [DiffCategory::Release],
                format!("Removed member-end release {id}."),
            ),
            (Some(old), Some(new))
                if old.target.member_id != new.target.member_id
                    || std::mem::discriminant(&old.target.end)
                        != std::mem::discriminant(&new.target.end) =>
            {
                push(
                    changes,
                    "release",
                    id,
                    DiffAction::Updated,
                    [DiffCategory::Release, DiffCategory::Topology],
                    format!("Changed release {id} target."),
                )
            }
            (Some(old), Some(new))
                if old.ux != new.ux
                    || old.uy != new.uy
                    || old.uz != new.uz
                    || old.rx != new.rx
                    || old.ry != new.ry
                    || old.rz != new.rz =>
            {
                push(
                    changes,
                    "release",
                    id,
                    DiffAction::Updated,
                    [DiffCategory::Release],
                    format!("Changed release conditions for {id}."),
                )
            }
            _ => {}
        }
    }
}

fn push<I>(
    changes: &mut Vec<SemanticChange>,
    kind: &str,
    id: &str,
    action: DiffAction,
    categories: I,
    description: String,
) where
    I: IntoIterator<Item = DiffCategory>,
{
    changes.push(SemanticChange {
        object_kind: kind.into(),
        object_id: id.into(),
        action,
        categories: categories.into_iter().collect(),
        description,
    });
}

fn assignment_target_label(target: &AssignmentTargetRef) -> String {
    match target {
        AssignmentTargetRef::Node(id) => format!("node {id}"),
        AssignmentTargetRef::Member(id) => format!("member {id}"),
        AssignmentTargetRef::Plate(id) => format!("plate {id}"),
    }
}

fn release_end_label(end: &fraia_core::MemberEnd) -> &'static str {
    match end {
        fraia_core::MemberEnd::Start => "start",
        fraia_core::MemberEnd::End => "end",
    }
}
