//! Closed, validated typed operations over authored structural models.

use crate::diff::{SemanticDiff, semantic_diff};
use fraia_core::{
    AssignmentTargetRef, LoadAssignment, LoadKind, LoadVector, ReleaseAssignment, StructuralMember,
    StructuralModel, StructuralNode, StructuralObjectRef, StructuralPlate, SupportAssignment,
    catalog::section_by_id, validate_structural_model,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

/// Length units accepted at the revision-engine boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LengthUnit {
    Meters,
    Millimeters,
    Feet,
}

/// A length supplied by an agent or client before it is admitted to the SI model.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Length {
    pub value: f64,
    pub unit: LengthUnit,
}

impl Length {
    pub fn meters(value: f64) -> Self {
        Self {
            value,
            unit: LengthUnit::Meters,
        }
    }
}

/// A typed model-space position. The first spike accepts metres only because
/// `StructuralModel` stores canonical SI coordinates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: Length,
    pub y: Length,
    pub z: Length,
}

/// Typed node input that admits coordinates only in the canonical model unit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeInput {
    pub id: String,
    pub position: Position,
}

/// Force units accepted for a node-targeted point load.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ForceUnit {
    Newtons,
    KiloNewtons,
}

/// Force-per-length units accepted for a member-targeted line load.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineLoadUnit {
    NewtonsPerMeter,
    KiloNewtonsPerMeter,
}

/// Pressure units accepted for a plate-targeted area load.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PressureUnit {
    Pascals,
    KiloPascals,
}

/// Unit-safe magnitude supplied before the patch boundary converts it to the
/// canonical SI `LoadAssignment::magnitude` representation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LoadMagnitude {
    Force { value: f64, unit: ForceUnit },
    LineLoad { value: f64, unit: LineLoadUnit },
    Pressure { value: f64, unit: PressureUnit },
}

/// Typed authored load input. Target and magnitude family are checked before
/// the model is mutated; generic raw `LoadAssignment` values do not cross this
/// construction boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadInput {
    pub id: String,
    pub target: AssignmentTargetRef,
    pub load_case_id: String,
    pub direction: LoadVector,
    pub magnitude: LoadMagnitude,
}

/// The deliberately small role vocabulary that can be changed by a patch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberRole {
    Beam,
    Column,
    Rafter,
    Brace,
    Joist,
    Purlin,
}

impl MemberRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Beam => "beam",
            Self::Column => "column",
            Self::Rafter => "rafter",
            Self::Brace => "brace",
            Self::Joist => "joist",
            Self::Purlin => "purlin",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "beam" => Some(Self::Beam),
            "column" => Some(Self::Column),
            "rafter" => Some(Self::Rafter),
            "brace" => Some(Self::Brace),
            "joist" => Some(Self::Joist),
            "purlin" => Some(Self::Purlin),
            _ => None,
        }
    }
}

/// A closed patch applied atomically to one authored model.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StructuralPatch {
    pub operations: Vec<StructuralOperation>,
}

/// Supported primitive and role operations. There is intentionally no generic
/// JSON merge or pointer mutation operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StructuralOperation {
    AddNode(NodeInput),
    MoveNode {
        node_id: String,
        position: Position,
    },
    DeleteNode {
        node_id: String,
    },
    AddMember(StructuralMember),
    UpdateMember(StructuralMember),
    DeleteMember {
        member_id: String,
    },
    SetMemberRole {
        member_id: String,
        role: MemberRole,
    },
    SetSection {
        member_id: String,
        section_id: String,
    },
    AddPlate(StructuralPlate),
    UpdatePlate(StructuralPlate),
    DeletePlate {
        plate_id: String,
    },
    AddSupport(SupportAssignment),
    UpdateSupport(SupportAssignment),
    DeleteSupport {
        support_id: String,
    },
    AddLoad(LoadInput),
    UpdateLoad(LoadInput),
    DeleteLoad {
        load_id: String,
    },
    AddRelease(ReleaseAssignment),
    SetRelease(ReleaseAssignment),
    UpdateRelease(ReleaseAssignment),
    DeleteRelease {
        release_id: String,
    },
}

/// Accepted patch output, including the revision-engine semantic diff.
#[derive(Debug, Clone)]
pub struct AppliedPatch {
    pub model: StructuralModel,
    pub diff: SemanticDiff,
}

/// A rejection leaves the source model untouched because application occurs on
/// a private clone and returns it only after validation succeeds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatchError {
    EmptyPatch,
    DuplicateId {
        kind: &'static str,
        id: String,
    },
    MissingTarget {
        kind: &'static str,
        id: String,
    },
    UnsupportedRole {
        role: String,
    },
    IncompatibleUnit {
        unit: LengthUnit,
    },
    IncompatibleLoadMagnitude {
        target_kind: &'static str,
    },
    NonFiniteLength,
    NonFiniteLoadMagnitude,
    InvalidLoadTarget {
        target_kind: &'static str,
        id: String,
    },
    InvalidModel {
        diagnostics: Vec<String>,
    },
    InvalidId {
        kind: &'static str,
        id: String,
    },
    InvalidReference {
        kind: &'static str,
        id: String,
        target_kind: &'static str,
        target_id: String,
    },
    InvalidTopology {
        kind: &'static str,
        id: String,
        reason: String,
    },
    UnknownSection {
        member_id: String,
        section_id: String,
    },
    ReferencedObject {
        kind: &'static str,
        id: String,
        referenced_by: Vec<String>,
    },
}

impl fmt::Display for PatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPatch => f.write_str("structural patch contains no operations"),
            Self::DuplicateId { kind, id } => write!(f, "duplicate {kind} id: {id}"),
            Self::MissingTarget { kind, id } => write!(f, "missing {kind} target: {id}"),
            Self::UnsupportedRole { role } => write!(f, "unsupported member role: {role}"),
            Self::IncompatibleUnit { unit } => {
                write!(f, "coordinates must be supplied in metres, not {unit:?}")
            }
            Self::NonFiniteLength => f.write_str("length must be finite"),
            Self::IncompatibleLoadMagnitude { target_kind } => write!(
                f,
                "load magnitude family does not match {target_kind} target"
            ),
            Self::NonFiniteLoadMagnitude => f.write_str("load magnitude must be finite"),
            Self::InvalidLoadTarget { target_kind, id } => {
                write!(f, "load targets missing {target_kind} `{id}`")
            }
            Self::InvalidModel { diagnostics } => write!(
                f,
                "patch produced an invalid structural model: {}",
                diagnostics.join("; ")
            ),
            Self::InvalidId { kind, id } => write!(f, "{kind} id must not be empty: `{id}`"),
            Self::InvalidReference {
                kind,
                id,
                target_kind,
                target_id,
            } => write!(
                f,
                "{kind} `{id}` references missing {target_kind} `{target_id}`"
            ),
            Self::InvalidTopology { kind, id, reason } => {
                write!(f, "invalid {kind} topology for `{id}`: {reason}")
            }
            Self::UnknownSection {
                member_id,
                section_id,
            } => write!(
                f,
                "member `{member_id}` references unknown section `{section_id}`"
            ),
            Self::ReferencedObject {
                kind,
                id,
                referenced_by,
            } => write!(
                f,
                "cannot delete {kind} `{id}`; referenced by {}",
                referenced_by.join(", ")
            ),
        }
    }
}

impl std::error::Error for PatchError {}

/// Applies every operation to a clone, validates it, then emits a stable
/// engineering diff. Any error rejects the entire patch.
pub fn apply_patch(
    parent: &StructuralModel,
    patch: &StructuralPatch,
) -> Result<AppliedPatch, PatchError> {
    if patch.operations.is_empty() {
        return Err(PatchError::EmptyPatch);
    }
    // A patch must not introduce validation failures. Existing imported or
    // legacy diagnostics remain visible to later validation/review work, but
    // do not make an otherwise safe targeted patch impossible to record.
    let parent_errors: BTreeSet<_> = validate_structural_model(parent)
        .diagnostics
        .into_iter()
        .filter(|diagnostic| matches!(diagnostic.severity, fraia_core::DiagnosticSeverity::Error))
        .map(|diagnostic| (diagnostic.code, diagnostic.object_refs))
        .collect();
    let mut candidate = parent.clone();
    for operation in &patch.operations {
        apply_operation(&mut candidate, operation)?;
    }
    let diagnostics: Vec<_> = validate_structural_model(&candidate)
        .diagnostics
        .into_iter()
        .filter(|diagnostic| matches!(diagnostic.severity, fraia_core::DiagnosticSeverity::Error))
        .filter(|diagnostic| {
            !parent_errors.contains(&(diagnostic.code.clone(), diagnostic.object_refs.clone()))
        })
        .map(|diagnostic| diagnostic.message)
        .collect();
    if !diagnostics.is_empty() {
        return Err(PatchError::InvalidModel { diagnostics });
    }
    let diff = semantic_diff(parent, &candidate);
    Ok(AppliedPatch {
        model: candidate,
        diff,
    })
}

fn apply_operation(
    model: &mut StructuralModel,
    operation: &StructuralOperation,
) -> Result<(), PatchError> {
    match operation {
        StructuralOperation::AddNode(node) => {
            validate_id("node", &node.id)?;
            let (x, y, z) = (
                meters(node.position.x)?,
                meters(node.position.y)?,
                meters(node.position.z)?,
            );
            let node = StructuralNode {
                id: node.id.clone(),
                x,
                y,
                z,
            };
            add(&mut model.nodes, node, "node", |value| &value.id)
        }
        StructuralOperation::MoveNode { node_id, position } => {
            let (x, y, z) = (
                meters(position.x)?,
                meters(position.y)?,
                meters(position.z)?,
            );
            let node = find_mut(&mut model.nodes, node_id, "node", |value| &value.id)?;
            node.x = x;
            node.y = y;
            node.z = z;
            Ok(())
        }
        StructuralOperation::DeleteNode { node_id } => delete_node(model, node_id),
        StructuralOperation::AddMember(member) => {
            validate_member(model, member)?;
            add(&mut model.members, member.clone(), "member", |value| {
                &value.id
            })
        }
        StructuralOperation::UpdateMember(member) => {
            validate_member(model, member)?;
            update(&mut model.members, member.clone(), "member", |value| {
                &value.id
            })
        }
        StructuralOperation::DeleteMember { member_id } => delete_member(model, member_id),
        StructuralOperation::SetMemberRole { member_id, role } => {
            let member = find_mut(&mut model.members, member_id, "member", |value| &value.id)?;
            member.role = role.as_str().into();
            Ok(())
        }
        StructuralOperation::SetSection {
            member_id,
            section_id,
        } => {
            validate_section(member_id, section_id)?;
            let member = find_mut(&mut model.members, member_id, "member", |value| &value.id)?;
            member.section_id = section_id.clone();
            Ok(())
        }
        StructuralOperation::AddPlate(plate) => {
            validate_plate(model, plate)?;
            add(&mut model.plates, plate.clone(), "plate", |value| &value.id)
        }
        StructuralOperation::UpdatePlate(plate) => {
            validate_plate(model, plate)?;
            update(&mut model.plates, plate.clone(), "plate", |value| &value.id)
        }
        StructuralOperation::DeletePlate { plate_id } => delete_plate(model, plate_id),
        StructuralOperation::AddSupport(support) => {
            validate_support(model, support)?;
            add(&mut model.supports, support.clone(), "support", |value| {
                &value.id
            })
        }
        StructuralOperation::UpdateSupport(support) => {
            validate_support(model, support)?;
            update(&mut model.supports, support.clone(), "support", |value| {
                &value.id
            })
        }
        StructuralOperation::DeleteSupport { support_id } => {
            delete(&mut model.supports, support_id, "support", |value| {
                &value.id
            })
        }
        StructuralOperation::AddLoad(load) => {
            let load = authored_load(model, load)?;
            add(&mut model.loads, load, "load", |value| &value.id)
        }
        StructuralOperation::UpdateLoad(load) => {
            let load = authored_load(model, load)?;
            update(&mut model.loads, load, "load", |value| &value.id)
        }
        StructuralOperation::DeleteLoad { load_id } => {
            delete(&mut model.loads, load_id, "load", |value| &value.id)
        }
        StructuralOperation::AddRelease(release) => {
            validate_release(model, release)?;
            add(&mut model.releases, release.clone(), "release", |value| {
                &value.id
            })
        }
        StructuralOperation::SetRelease(release) => {
            validate_release(model, release)?;
            update(&mut model.releases, release.clone(), "release", |value| {
                &value.id
            })
        }
        StructuralOperation::UpdateRelease(release) => {
            validate_release(model, release)?;
            update(&mut model.releases, release.clone(), "release", |value| {
                &value.id
            })
        }
        StructuralOperation::DeleteRelease { release_id } => {
            delete(&mut model.releases, release_id, "release", |value| {
                &value.id
            })
        }
    }
}

fn authored_load(model: &StructuralModel, input: &LoadInput) -> Result<LoadAssignment, PatchError> {
    validate_id("load", &input.id)?;
    if input.load_case_id.trim().is_empty() {
        return Err(PatchError::InvalidId {
            kind: "load case",
            id: input.load_case_id.clone(),
        });
    }
    let target_kind = input.target.kind_label();
    let target_exists = match &input.target {
        AssignmentTargetRef::Node(id) => model.nodes.iter().any(|node| node.id == *id),
        AssignmentTargetRef::Member(id) => model.members.iter().any(|member| member.id == *id),
        AssignmentTargetRef::Plate(id) => model.plates.iter().any(|plate| plate.id == *id),
    };
    if !target_exists {
        let id = match &input.target {
            AssignmentTargetRef::Node(id)
            | AssignmentTargetRef::Member(id)
            | AssignmentTargetRef::Plate(id) => id.clone(),
        };
        return Err(PatchError::InvalidLoadTarget { target_kind, id });
    }
    let (kind, magnitude) = match input.magnitude {
        LoadMagnitude::Force { value, unit }
            if matches!(input.target, AssignmentTargetRef::Node(_)) =>
        {
            (
                LoadKind::Point,
                finite_load(value)?
                    * match unit {
                        ForceUnit::Newtons => 1.0,
                        ForceUnit::KiloNewtons => 1_000.0,
                    },
            )
        }
        LoadMagnitude::LineLoad { value, unit }
            if matches!(input.target, AssignmentTargetRef::Member(_)) =>
        {
            (
                LoadKind::UniformLine,
                finite_load(value)?
                    * match unit {
                        LineLoadUnit::NewtonsPerMeter => 1.0,
                        LineLoadUnit::KiloNewtonsPerMeter => 1_000.0,
                    },
            )
        }
        LoadMagnitude::Pressure { value, unit }
            if matches!(input.target, AssignmentTargetRef::Plate(_)) =>
        {
            (
                LoadKind::Area,
                finite_load(value)?
                    * match unit {
                        PressureUnit::Pascals => 1.0,
                        PressureUnit::KiloPascals => 1_000.0,
                    },
            )
        }
        _ => return Err(PatchError::IncompatibleLoadMagnitude { target_kind }),
    };
    if !input.direction.x.is_finite()
        || !input.direction.y.is_finite()
        || !input.direction.z.is_finite()
    {
        return Err(PatchError::NonFiniteLoadMagnitude);
    }
    let magnitude = if magnitude.is_finite() {
        magnitude
    } else {
        return Err(PatchError::NonFiniteLoadMagnitude);
    };
    Ok(LoadAssignment {
        id: input.id.clone(),
        target: input.target.clone(),
        load_case_id: input.load_case_id.clone(),
        kind,
        direction: input.direction.clone(),
        magnitude,
    })
}

fn finite_load(value: f64) -> Result<f64, PatchError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(PatchError::NonFiniteLoadMagnitude)
    }
}

fn meters(length: Length) -> Result<f64, PatchError> {
    if !length.value.is_finite() {
        return Err(PatchError::NonFiniteLength);
    }
    if length.unit != LengthUnit::Meters {
        return Err(PatchError::IncompatibleUnit { unit: length.unit });
    }
    Ok(length.value)
}

fn validate_member_role(member: &StructuralMember) -> Result<(), PatchError> {
    MemberRole::parse(&member.role)
        .ok_or_else(|| PatchError::UnsupportedRole {
            role: member.role.clone(),
        })
        .map(|_| ())
}

fn validate_id(kind: &'static str, id: &str) -> Result<(), PatchError> {
    if id.trim().is_empty() {
        return Err(PatchError::InvalidId {
            kind,
            id: id.into(),
        });
    }
    Ok(())
}

fn validate_section(member_id: &str, section_id: &str) -> Result<(), PatchError> {
    validate_id("member", member_id)?;
    validate_id("section", section_id)?;
    if section_by_id(section_id).is_none() {
        return Err(PatchError::UnknownSection {
            member_id: member_id.into(),
            section_id: section_id.into(),
        });
    }
    Ok(())
}

fn validate_member(model: &StructuralModel, member: &StructuralMember) -> Result<(), PatchError> {
    validate_id("member", &member.id)?;
    validate_member_role(member)?;
    validate_section(&member.id, &member.section_id)?;
    if member.start_node == member.end_node {
        return Err(PatchError::InvalidTopology {
            kind: "member",
            id: member.id.clone(),
            reason: "start and end nodes must differ".into(),
        });
    }
    for (target_kind, target_id) in [("node", &member.start_node), ("node", &member.end_node)] {
        if model.node_by_id(target_id).is_none() {
            return Err(PatchError::InvalidReference {
                kind: "member",
                id: member.id.clone(),
                target_kind,
                target_id: target_id.clone(),
            });
        }
    }
    Ok(())
}

fn validate_plate(model: &StructuralModel, plate: &StructuralPlate) -> Result<(), PatchError> {
    validate_id("plate", &plate.id)?;
    if plate.boundary_nodes.len() < 3 {
        return Err(PatchError::InvalidTopology {
            kind: "plate",
            id: plate.id.clone(),
            reason: "a plate needs at least three boundary nodes".into(),
        });
    }
    let mut boundary = BTreeSet::new();
    for node_id in &plate.boundary_nodes {
        if !boundary.insert(node_id) {
            return Err(PatchError::InvalidTopology {
                kind: "plate",
                id: plate.id.clone(),
                reason: "boundary nodes must be unique".into(),
            });
        }
        if model.node_by_id(node_id).is_none() {
            return Err(PatchError::InvalidReference {
                kind: "plate",
                id: plate.id.clone(),
                target_kind: "node",
                target_id: node_id.clone(),
            });
        }
    }
    if !plate.thickness_m.is_finite() || plate.thickness_m <= 0.0 {
        return Err(PatchError::InvalidTopology {
            kind: "plate",
            id: plate.id.clone(),
            reason: "thickness must be finite and positive metres".into(),
        });
    }
    Ok(())
}

fn validate_support(
    model: &StructuralModel,
    support: &SupportAssignment,
) -> Result<(), PatchError> {
    validate_id("support", &support.id)?;
    if model.node_by_id(&support.target_node).is_none() {
        return Err(PatchError::InvalidReference {
            kind: "support",
            id: support.id.clone(),
            target_kind: "node",
            target_id: support.target_node.clone(),
        });
    }
    Ok(())
}

fn validate_release(
    model: &StructuralModel,
    release: &ReleaseAssignment,
) -> Result<(), PatchError> {
    validate_id("release", &release.id)?;
    if model
        .members
        .iter()
        .all(|member| member.id != release.target.member_id)
    {
        return Err(PatchError::InvalidReference {
            kind: "release",
            id: release.id.clone(),
            target_kind: "member",
            target_id: release.target.member_id.clone(),
        });
    }
    Ok(())
}

fn add<T, F>(items: &mut Vec<T>, value: T, kind: &'static str, id: F) -> Result<(), PatchError>
where
    F: Fn(&T) -> &String,
{
    let value_id = id(&value);
    if items.iter().any(|item| id(item) == value_id) {
        return Err(PatchError::DuplicateId {
            kind,
            id: value_id.clone(),
        });
    }
    items.push(value);
    Ok(())
}

fn update<T, F>(items: &mut [T], value: T, kind: &'static str, id: F) -> Result<(), PatchError>
where
    F: Fn(&T) -> &String,
{
    let value_id = id(&value).clone();
    let item = items
        .iter_mut()
        .find(|item| id(item) == &value_id)
        .ok_or_else(|| PatchError::MissingTarget { kind, id: value_id })?;
    *item = value;
    Ok(())
}

fn find_mut<'a, T, F>(
    items: &'a mut [T],
    value_id: &str,
    kind: &'static str,
    id: F,
) -> Result<&'a mut T, PatchError>
where
    F: Fn(&T) -> &String,
{
    items
        .iter_mut()
        .find(|item| id(item) == value_id)
        .ok_or_else(|| PatchError::MissingTarget {
            kind,
            id: value_id.into(),
        })
}

fn delete<T, F>(
    items: &mut Vec<T>,
    value_id: &str,
    kind: &'static str,
    id: F,
) -> Result<(), PatchError>
where
    F: Fn(&T) -> &String,
{
    let index = items
        .iter()
        .position(|item| id(item) == value_id)
        .ok_or_else(|| PatchError::MissingTarget {
            kind,
            id: value_id.into(),
        })?;
    items.remove(index);
    Ok(())
}

fn delete_node(model: &mut StructuralModel, node_id: &str) -> Result<(), PatchError> {
    if model.node_by_id(node_id).is_none() {
        return Err(PatchError::MissingTarget {
            kind: "node",
            id: node_id.into(),
        });
    }
    let mut references = Vec::new();
    for member in &model.members {
        if member.start_node == node_id || member.end_node == node_id {
            references.push(format!("member:{}", member.id));
        }
    }
    for plate in &model.plates {
        if plate.boundary_nodes.iter().any(|id| id == node_id) {
            references.push(format!("plate:{}", plate.id));
        }
    }
    for support in &model.supports {
        if support.target_node == node_id {
            references.push(format!("support:{}", support.id));
        }
    }
    for load in &model.loads {
        if matches!(&load.target, AssignmentTargetRef::Node(id) if id == node_id) {
            references.push(format!("load:{}", load.id));
        }
    }
    if builder_references(
        model,
        &StructuralObjectRef::Node(node_id.into()),
        &mut references,
    ) {
        references.sort();
        references.dedup();
    }
    if !references.is_empty() {
        return Err(PatchError::ReferencedObject {
            kind: "node",
            id: node_id.into(),
            referenced_by: references,
        });
    }
    delete(&mut model.nodes, node_id, "node", |value| &value.id)
}

fn delete_member(model: &mut StructuralModel, member_id: &str) -> Result<(), PatchError> {
    if model.members.iter().all(|member| member.id != member_id) {
        return Err(PatchError::MissingTarget {
            kind: "member",
            id: member_id.into(),
        });
    }
    let mut references = Vec::new();
    for load in &model.loads {
        if matches!(&load.target, AssignmentTargetRef::Member(id) if id == member_id) {
            references.push(format!("load:{}", load.id));
        }
    }
    for release in &model.releases {
        if release.target.member_id == member_id {
            references.push(format!("release:{}", release.id));
        }
    }
    builder_references(
        model,
        &StructuralObjectRef::Member(member_id.into()),
        &mut references,
    );
    references.sort();
    references.dedup();
    if !references.is_empty() {
        return Err(PatchError::ReferencedObject {
            kind: "member",
            id: member_id.into(),
            referenced_by: references,
        });
    }
    delete(&mut model.members, member_id, "member", |value| &value.id)
}

fn delete_plate(model: &mut StructuralModel, plate_id: &str) -> Result<(), PatchError> {
    if model.plates.iter().all(|plate| plate.id != plate_id) {
        return Err(PatchError::MissingTarget {
            kind: "plate",
            id: plate_id.into(),
        });
    }
    let mut references = Vec::new();
    for load in &model.loads {
        if matches!(&load.target, AssignmentTargetRef::Plate(id) if id == plate_id) {
            references.push(format!("load:{}", load.id));
        }
    }
    builder_references(
        model,
        &StructuralObjectRef::Plate(plate_id.into()),
        &mut references,
    );
    references.sort();
    references.dedup();
    if !references.is_empty() {
        return Err(PatchError::ReferencedObject {
            kind: "plate",
            id: plate_id.into(),
            referenced_by: references,
        });
    }
    delete(&mut model.plates, plate_id, "plate", |value| &value.id)
}

fn builder_references(
    model: &StructuralModel,
    object: &StructuralObjectRef,
    references: &mut Vec<String>,
) -> bool {
    let mut found = false;
    for materialization in &model.builder_node_materializations {
        if materialization
            .object_refs
            .iter()
            .any(|reference| reference == object)
        {
            references.push(format!("builder-node:{}", materialization.builder_node_id));
            found = true;
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::DiffCategory;
    use crate::root_fixture;
    use crate::snapshot::ModelSnapshot;
    use fraia_core::{
        AssignmentTargetRef, LoadVector, MemberEnd, MemberEndTarget, ReleaseAssignment,
    };

    #[test]
    fn move_node_is_atomic_and_reports_geometry_and_member_change() {
        let fixture = root_fixture();
        let patch = StructuralPatch {
            operations: vec![StructuralOperation::MoveNode {
                node_id: "left-eave".into(),
                position: Position {
                    x: Length::meters(0.0),
                    y: Length::meters(7.0),
                    z: Length::meters(0.0),
                },
            }],
        };
        let result = apply_patch(&fixture.model, &patch).unwrap();
        assert_eq!(fixture.model.nodes[1].y, 6.0);
        assert_eq!(result.model.nodes[1].y, 7.0);
        assert!(result.diff.affects(DiffCategory::Geometry));
        assert!(result.diff.affects(DiffCategory::Member));
        assert!(
            result
                .diff
                .changes
                .iter()
                .any(|change| change.object_id == "left-column"
                    && change.categories.contains(&DiffCategory::Geometry))
        );
    }

    #[test]
    fn support_change_is_classified_as_support_and_deterministic() {
        let fixture = root_fixture();
        let mut support = fixture.model.supports[1].clone();
        support.ux = true;
        let patch = StructuralPatch {
            operations: vec![StructuralOperation::UpdateSupport(support)],
        };
        let one = apply_patch(&fixture.model, &patch).unwrap();
        let two = apply_patch(&fixture.model, &patch).unwrap();
        assert_eq!(one.diff, two.diff);
        assert!(one.diff.affects(DiffCategory::Support));
    }

    #[test]
    fn invalid_operations_leave_the_parent_unchanged() {
        let fixture = root_fixture();
        let original = serde_json::to_vec(&fixture.model).unwrap();
        let patch = StructuralPatch {
            operations: vec![
                StructuralOperation::MoveNode {
                    node_id: "left-base".into(),
                    position: Position {
                        x: Length::meters(1.0),
                        y: Length::meters(0.0),
                        z: Length::meters(0.0),
                    },
                },
                StructuralOperation::DeleteNode {
                    node_id: "missing".into(),
                },
            ],
        };
        assert!(matches!(
            apply_patch(&fixture.model, &patch),
            Err(PatchError::MissingTarget { .. })
        ));
        assert_eq!(serde_json::to_vec(&fixture.model).unwrap(), original);
    }

    #[test]
    fn rejects_duplicate_ids_bad_units_and_unsupported_roles() {
        let fixture = root_fixture();
        let duplicate = StructuralPatch {
            operations: vec![StructuralOperation::AddNode(NodeInput {
                id: fixture.model.nodes[0].id.clone(),
                position: Position {
                    x: Length::meters(0.0),
                    y: Length::meters(0.0),
                    z: Length::meters(0.0),
                },
            })],
        };
        assert!(matches!(
            apply_patch(&fixture.model, &duplicate),
            Err(PatchError::DuplicateId { .. })
        ));
        let bad_unit = StructuralPatch {
            operations: vec![StructuralOperation::MoveNode {
                node_id: "left-base".into(),
                position: Position {
                    x: Length {
                        value: 1.0,
                        unit: LengthUnit::Millimeters,
                    },
                    y: Length::meters(0.0),
                    z: Length::meters(0.0),
                },
            }],
        };
        assert!(matches!(
            apply_patch(&fixture.model, &bad_unit),
            Err(PatchError::IncompatibleUnit { .. })
        ));
        let mut member = fixture.model.members[0].clone();
        member.role = "magic-frame".into();
        let bad_role = StructuralPatch {
            operations: vec![StructuralOperation::UpdateMember(member)],
        };
        assert!(matches!(
            apply_patch(&fixture.model, &bad_role),
            Err(PatchError::UnsupportedRole { .. })
        ));
    }

    #[test]
    fn supports_valid_add_delete_and_closed_role_operations() {
        let fixture = root_fixture();
        let patch = StructuralPatch {
            operations: vec![
                StructuralOperation::AddNode(NodeInput {
                    id: "new-roof-node".into(),
                    position: Position {
                        x: Length::meters(10.0),
                        y: Length::meters(8.0),
                        z: Length::meters(0.0),
                    },
                }),
                StructuralOperation::SetMemberRole {
                    member_id: "rafter".into(),
                    role: MemberRole::Beam,
                },
                StructuralOperation::DeleteSupport {
                    support_id: "right-base-support".into(),
                },
            ],
        };
        let result = apply_patch(&fixture.model, &patch).unwrap();
        assert!(result.model.node_by_id("new-roof-node").is_some());
        assert_eq!(result.model.members[1].role, "beam");
        assert_eq!(result.model.supports.len(), 1);
        assert!(result.diff.affects(DiffCategory::Topology));
        assert!(result.diff.affects(DiffCategory::Role));
        assert!(result.diff.affects(DiffCategory::Support));
    }

    #[test]
    fn closed_construction_patch_builds_a_valid_model_from_empty_state() {
        let empty = StructuralModel {
            dimension: "2d-in-3d".into(),
            nodes: vec![],
            members: vec![],
            plates: vec![],
            supports: vec![],
            loads: vec![],
            releases: vec![],
            load_cases: vec![],
            builder_node_materializations: vec![],
        };
        let patch = StructuralPatch {
            operations: vec![
                StructuralOperation::AddNode(NodeInput {
                    id: "left".into(),
                    position: Position {
                        x: Length::meters(0.0),
                        y: Length::meters(0.0),
                        z: Length::meters(0.0),
                    },
                }),
                StructuralOperation::AddNode(NodeInput {
                    id: "right".into(),
                    position: Position {
                        x: Length::meters(6.0),
                        y: Length::meters(0.0),
                        z: Length::meters(0.0),
                    },
                }),
                StructuralOperation::AddMember(StructuralMember {
                    id: "beam".into(),
                    start_node: "left".into(),
                    end_node: "right".into(),
                    role: "beam".into(),
                    semantic_tags: vec![],
                    section_id: "250UB".into(),
                    material_id: "steel".into(),
                }),
                StructuralOperation::AddSupport(SupportAssignment {
                    id: "left-support".into(),
                    target_node: "left".into(),
                    ux: true,
                    uy: true,
                    uz: true,
                    rx: false,
                    ry: false,
                    rz: true,
                }),
                StructuralOperation::AddSupport(SupportAssignment {
                    id: "right-support".into(),
                    target_node: "right".into(),
                    ux: false,
                    uy: true,
                    uz: true,
                    rx: false,
                    ry: false,
                    rz: true,
                }),
                StructuralOperation::AddLoad(LoadInput {
                    id: "gravity".into(),
                    target: AssignmentTargetRef::Member("beam".into()),
                    load_case_id: "dead".into(),
                    direction: LoadVector {
                        x: 0.0,
                        y: -1.0,
                        z: 0.0,
                    },
                    magnitude: LoadMagnitude::LineLoad {
                        value: 10.0,
                        unit: LineLoadUnit::KiloNewtonsPerMeter,
                    },
                }),
            ],
        };
        let result = apply_patch(&empty, &patch).unwrap();
        assert_eq!(result.model.nodes.len(), 2);
        assert_eq!(result.model.members.len(), 1);
        assert_eq!(result.model.supports.len(), 2);
        assert_eq!(result.model.loads[0].magnitude, 10_000.0);
        assert!(result.diff.affects(DiffCategory::Topology));
        assert!(result.diff.affects(DiffCategory::Support));
        assert!(result.diff.affects(DiffCategory::Load));
    }

    #[test]
    fn invalid_construction_reference_or_units_reject_atomically_and_preserve_parent_snapshot() {
        let fixture = root_fixture();
        let snapshot_before = ModelSnapshot::capture(fixture.model.clone()).unwrap();
        let invalid_target = StructuralPatch {
            operations: vec![StructuralOperation::AddLoad(LoadInput {
                id: "bad-load".into(),
                target: AssignmentTargetRef::Member("missing".into()),
                load_case_id: "dead".into(),
                direction: LoadVector {
                    x: 0.0,
                    y: -1.0,
                    z: 0.0,
                },
                magnitude: LoadMagnitude::LineLoad {
                    value: 1.0,
                    unit: LineLoadUnit::KiloNewtonsPerMeter,
                },
            })],
        };
        assert!(matches!(
            apply_patch(&fixture.model, &invalid_target),
            Err(PatchError::InvalidLoadTarget { .. })
        ));
        let bad_units = StructuralPatch {
            operations: vec![StructuralOperation::AddNode(NodeInput {
                id: "millimetre-node".into(),
                position: Position {
                    x: Length {
                        value: 1.0,
                        unit: LengthUnit::Millimeters,
                    },
                    y: Length::meters(0.0),
                    z: Length::meters(0.0),
                },
            })],
        };
        assert!(matches!(
            apply_patch(&fixture.model, &bad_units),
            Err(PatchError::IncompatibleUnit { .. })
        ));
        let snapshot_after = ModelSnapshot::capture(fixture.model.clone()).unwrap();
        assert_eq!(snapshot_before.id(), snapshot_after.id());
        assert_eq!(
            snapshot_before.canonical_bytes(),
            snapshot_after.canonical_bytes()
        );
    }

    #[test]
    fn explicit_section_release_plate_and_reference_deletion_contracts_are_safe() {
        let fixture = root_fixture();
        let patch = StructuralPatch {
            operations: vec![
                StructuralOperation::SetSection {
                    member_id: "rafter".into(),
                    section_id: "250UB".into(),
                },
                StructuralOperation::AddPlate(StructuralPlate {
                    id: "roof-plate".into(),
                    boundary_nodes: vec![
                        "left-base".into(),
                        "left-eave".into(),
                        "right-eave".into(),
                    ],
                    role: "roof_panel".into(),
                    semantic_tags: vec![],
                    thickness_m: 0.12,
                    material_id: "steel".into(),
                    generated_from: "authored".into(),
                }),
                StructuralOperation::AddRelease(ReleaseAssignment {
                    id: "rafter-start-release".into(),
                    target: MemberEndTarget {
                        member_id: "rafter".into(),
                        end: MemberEnd::Start,
                    },
                    ux: true,
                    uy: false,
                    uz: false,
                    rx: false,
                    ry: false,
                    rz: false,
                }),
                StructuralOperation::SetRelease(ReleaseAssignment {
                    id: "rafter-start-release".into(),
                    target: MemberEndTarget {
                        member_id: "rafter".into(),
                        end: MemberEnd::End,
                    },
                    ux: true,
                    uy: true,
                    uz: false,
                    rx: false,
                    ry: false,
                    rz: false,
                }),
                StructuralOperation::AddLoad(LoadInput {
                    id: "roof-pressure".into(),
                    target: AssignmentTargetRef::Plate("roof-plate".into()),
                    load_case_id: "dead".into(),
                    direction: LoadVector {
                        x: 0.0,
                        y: -1.0,
                        z: 0.0,
                    },
                    magnitude: LoadMagnitude::Pressure {
                        value: 1.5,
                        unit: PressureUnit::KiloPascals,
                    },
                }),
            ],
        };
        let result = apply_patch(&fixture.model, &patch).unwrap();
        assert_eq!(result.model.members[1].section_id, "250UB");
        assert!(matches!(
            &result.model.releases[0].target.end,
            &MemberEnd::End
        ));
        assert_eq!(result.model.loads[0].magnitude, 1_500.0);
        assert!(
            result
                .diff
                .changes
                .iter()
                .any(|change| { change.description.contains("section from 410UB54 to 250UB") })
        );

        let delete_referenced_node = StructuralPatch {
            operations: vec![StructuralOperation::DeleteNode {
                node_id: "left-base".into(),
            }],
        };
        assert!(matches!(
            apply_patch(&fixture.model, &delete_referenced_node),
            Err(PatchError::ReferencedObject { kind: "node", .. })
        ));
    }
}
