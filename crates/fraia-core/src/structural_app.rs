use crate::types::{FrameModel2D, LoadCase2D};
use crate::units::{Force, LengthPoint3, LineLoad, Stress};
use fraia_geometry::Point3;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct StructuralNode {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Serialize for StructuralNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("StructuralNode", 2)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("position", &LengthPoint3::new(self.x, self.y, self.z))?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for StructuralNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawStructuralNode {
            id: String,
            #[serde(default)]
            position: Option<LengthPoint3>,
            #[serde(default)]
            x: Option<f64>,
            #[serde(default)]
            y: Option<f64>,
            #[serde(default)]
            z: Option<f64>,
        }

        let raw = RawStructuralNode::deserialize(deserializer)?;
        if let Some(position) = raw.position {
            Ok(Self {
                id: raw.id,
                x: position.x,
                y: position.y,
                z: position.z,
            })
        } else {
            Ok(Self {
                id: raw.id,
                x: raw.x.unwrap_or_default(),
                y: raw.y.unwrap_or_default(),
                z: raw.z.unwrap_or_default(),
            })
        }
    }
}

impl StructuralNode {
    pub fn point(&self) -> Point3 {
        Point3::new(self.x, self.y, self.z)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralMember {
    pub id: String,
    pub start_node: String,
    pub end_node: String,
    pub role: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub semantic_tags: Vec<String>,
    pub section_id: String,
    pub material_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralPlate {
    pub id: String,
    pub boundary_nodes: Vec<String>,
    pub role: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub semantic_tags: Vec<String>,
    #[serde(
        rename = "thickness",
        alias = "thickness_m",
        serialize_with = "crate::units::serde_f64::serialize_length",
        deserialize_with = "crate::units::serde_f64::deserialize_length"
    )]
    pub thickness_m: f64,
    pub material_id: String,
    pub generated_from: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportAssignment {
    pub id: String,
    pub target_node: String,
    pub ux: bool,
    pub uy: bool,
    pub uz: bool,
    pub rx: bool,
    pub ry: bool,
    pub rz: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssignmentTargetRef {
    Node(String),
    Member(String),
    Plate(String),
}

impl AssignmentTargetRef {
    pub fn expected_load_kind(&self) -> LoadKind {
        match self {
            Self::Node(_) => LoadKind::Point,
            Self::Member(_) => LoadKind::UniformLine,
            Self::Plate(_) => LoadKind::Area,
        }
    }

    pub fn kind_label(&self) -> &'static str {
        match self {
            Self::Node(_) => "node",
            Self::Member(_) => "member",
            Self::Plate(_) => "plate",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemberEnd {
    Start,
    End,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadVector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadKind {
    #[serde(rename = "point", alias = "nodal_force")]
    Point,
    #[serde(rename = "uniform_line", alias = "distributed")]
    UniformLine,
    #[serde(rename = "area", alias = "plate_pressure")]
    Area,
}

impl LoadKind {
    pub const ALL: [Self; 3] = [Self::Point, Self::UniformLine, Self::Area];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Point => "point",
            Self::UniformLine => "uniform_line",
            Self::Area => "area",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoadAssignment {
    pub id: String,
    pub target: AssignmentTargetRef,
    pub load_case_id: String,
    pub kind: LoadKind,
    pub direction: LoadVector,
    /// Canonical backend magnitude in SI units: N for point loads, N/m for
    /// uniform line loads. Area load conversion is intentionally not realised
    /// by the current frame2d path.
    pub magnitude: f64,
}

impl Serialize for LoadAssignment {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("LoadAssignment", 6)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("target", &self.target)?;
        state.serialize_field("load_case_id", &self.load_case_id)?;
        state.serialize_field("family", &self.kind)?;
        state.serialize_field("direction", &self.direction)?;
        match self.kind {
            LoadKind::Point => {
                state.serialize_field("magnitude", &Force::canonical(self.magnitude))?;
            }
            LoadKind::UniformLine => {
                state.serialize_field("magnitude", &LineLoad::canonical(self.magnitude))?;
            }
            LoadKind::Area => {
                state.serialize_field("magnitude", &Stress::canonical(self.magnitude))?;
            }
        }
        state.end()
    }
}

impl<'de> Deserialize<'de> for LoadAssignment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawLoadAssignment {
            id: String,
            target: AssignmentTargetRef,
            load_case_id: String,
            #[serde(rename = "family")]
            kind: LoadKind,
            direction: LoadVector,
            magnitude: Value,
        }

        let raw = RawLoadAssignment::deserialize(deserializer)?;
        let magnitude = match raw.magnitude {
            Value::Number(number) => number.as_f64().unwrap_or_default(),
            value => match raw.kind {
                LoadKind::Point => serde_json::from_value::<Force>(value)
                    .map(|quantity| quantity.newtons())
                    .map_err(serde::de::Error::custom)?,
                LoadKind::UniformLine => serde_json::from_value::<LineLoad>(value)
                    .map(|quantity| quantity.newtons_per_meter())
                    .map_err(serde::de::Error::custom)?,
                LoadKind::Area => serde_json::from_value::<Stress>(value)
                    .map(|quantity| quantity.pascals())
                    .map_err(serde::de::Error::custom)?,
            },
        };

        Ok(Self {
            id: raw.id,
            target: raw.target,
            load_case_id: raw.load_case_id,
            kind: raw.kind,
            direction: raw.direction,
            magnitude,
        })
    }
}

impl LoadAssignment {
    pub fn expected_kind(&self) -> LoadKind {
        self.target.expected_load_kind()
    }

    pub fn kind_matches_target(&self) -> bool {
        self.kind == self.expected_kind()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberEndTarget {
    pub member_id: String,
    pub end: MemberEnd,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseAssignment {
    pub id: String,
    pub target: MemberEndTarget,
    pub ux: bool,
    pub uy: bool,
    pub uz: bool,
    pub rx: bool,
    pub ry: bool,
    pub rz: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralModel {
    pub dimension: String,
    pub nodes: Vec<StructuralNode>,
    pub members: Vec<StructuralMember>,
    pub plates: Vec<StructuralPlate>,
    pub supports: Vec<SupportAssignment>,
    pub loads: Vec<LoadAssignment>,
    pub releases: Vec<ReleaseAssignment>,
    pub load_cases: Vec<LoadCase2D>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub builder_node_materializations: Vec<BuilderNodeMaterialization>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StructuralObjectRef {
    Node(String),
    Member(String),
    Plate(String),
    Support(String),
    Load(String),
    Release(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuilderNodeMaterialization {
    pub builder_node_id: String,
    pub object_refs: Vec<StructuralObjectRef>,
}

impl StructuralModel {
    pub fn empty() -> Self {
        Self {
            dimension: "3d".into(),
            nodes: Vec::new(),
            members: Vec::new(),
            plates: Vec::new(),
            supports: Vec::new(),
            loads: Vec::new(),
            releases: Vec::new(),
            load_cases: Vec::new(),
            builder_node_materializations: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
            && self.members.is_empty()
            && self.plates.is_empty()
            && self.supports.is_empty()
            && self.loads.is_empty()
            && self.releases.is_empty()
    }

    pub fn node_by_id(&self, id: &str) -> Option<&StructuralNode> {
        self.nodes.iter().find(|node| node.id == id)
    }

    pub fn materialization_for_builder_node(
        &self,
        builder_node_id: &str,
    ) -> Option<&BuilderNodeMaterialization> {
        self.builder_node_materializations
            .iter()
            .find(|entry| entry.builder_node_id == builder_node_id)
    }

    pub fn generated_by_builder_node_for_object(
        &self,
        object: &StructuralObjectRef,
    ) -> Option<&str> {
        self.builder_node_materializations
            .iter()
            .find(|entry| {
                entry
                    .object_refs
                    .iter()
                    .any(|candidate| candidate == object)
            })
            .map(|entry| entry.builder_node_id.as_str())
    }

    pub fn from_frame2d(model: &FrameModel2D) -> Self {
        let nodes: Vec<StructuralNode> = model
            .nodes
            .iter()
            .map(|node| StructuralNode {
                id: node.id.clone(),
                x: node.x,
                y: node.y,
                z: 0.0,
            })
            .collect();

        Self {
            dimension: "2d-in-3d".into(),
            members: model
                .elements
                .iter()
                .map(|element| StructuralMember {
                    id: element.id.clone(),
                    start_node: element.i.clone(),
                    end_node: element.j.clone(),
                    role: element.role.clone(),
                    semantic_tags: Vec::new(),
                    section_id: element.section.id.clone(),
                    material_id: element.material.id.clone(),
                })
                .collect(),
            plates: infer_conceptual_plates(&nodes),
            supports: model
                .supports
                .iter()
                .enumerate()
                .map(|(i, support)| SupportAssignment {
                    id: format!("support-{}", i + 1),
                    target_node: support.node.clone(),
                    ux: support.ux,
                    uy: support.uy,
                    uz: false,
                    rx: false,
                    ry: false,
                    rz: support.rz,
                })
                .collect(),
            loads: Vec::new(),
            releases: Vec::new(),
            load_cases: model.load_cases.clone(),
            builder_node_materializations: Vec::new(),
            nodes,
        }
    }
}

fn infer_conceptual_plates(nodes: &[StructuralNode]) -> Vec<StructuralPlate> {
    if nodes.len() < 4 {
        return Vec::new();
    }

    let min_y = nodes
        .iter()
        .map(|node| node.y)
        .fold(f64::INFINITY, f64::min);
    let max_y = nodes
        .iter()
        .map(|node| node.y)
        .fold(f64::NEG_INFINITY, f64::max);
    let tolerance = 1e-6;

    let mut base_nodes: Vec<&StructuralNode> = nodes
        .iter()
        .filter(|node| (node.y - min_y).abs() <= tolerance)
        .collect();
    let mut top_nodes: Vec<&StructuralNode> = nodes
        .iter()
        .filter(|node| (node.y - max_y).abs() <= tolerance)
        .collect();

    base_nodes.sort_by(|a, b| a.x.total_cmp(&b.x));
    top_nodes.sort_by(|a, b| a.x.total_cmp(&b.x));

    if base_nodes.len() < 2 || top_nodes.len() < 2 || base_nodes.len() != top_nodes.len() {
        return Vec::new();
    }

    let mut plates = Vec::new();
    for bay in 0..(base_nodes.len() - 1) {
        let boundary_nodes = vec![
            base_nodes[bay].id.clone(),
            top_nodes[bay].id.clone(),
            top_nodes[bay + 1].id.clone(),
            base_nodes[bay + 1].id.clone(),
        ];

        plates.push(StructuralPlate {
            id: format!("plate-bay-{}", bay + 1),
            boundary_nodes,
            role: "conceptual_panel".into(),
            semantic_tags: Vec::new(),
            thickness_m: 0.2,
            material_id: "steel".into(),
            generated_from: "frame2d-bay-inference".into(),
        });
    }

    plates
}

#[cfg(test)]
mod tests {
    use super::{
        AssignmentTargetRef, BuilderNodeMaterialization, LoadAssignment, LoadKind, LoadVector,
        MemberEnd, MemberEndTarget, ReleaseAssignment, StructuralMember, StructuralModel,
        StructuralNode, StructuralObjectRef,
    };

    #[test]
    fn structural_model_round_trips_through_json() {
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
                x: 8.0,
                y: 0.0,
                z: 0.0,
            },
        ];
        model.members = vec![StructuralMember {
            id: "m1".into(),
            start_node: "n1".into(),
            end_node: "n2".into(),
            role: "beam".into(),
            semantic_tags: vec!["primary".into()],
            section_id: "310UB".into(),
            material_id: "steel".into(),
        }];
        model.loads = vec![LoadAssignment {
            id: "l1".into(),
            target: AssignmentTargetRef::Member("m1".into()),
            load_case_id: "gravity".into(),
            kind: LoadKind::UniformLine,
            direction: LoadVector {
                x: 0.0,
                y: -1.0,
                z: 0.0,
            },
            magnitude: 18_000.0,
        }];
        model.releases = vec![ReleaseAssignment {
            id: "r1".into(),
            target: MemberEndTarget {
                member_id: "m1".into(),
                end: MemberEnd::End,
            },
            ux: false,
            uy: false,
            uz: false,
            rx: false,
            ry: false,
            rz: true,
        }];
        model.builder_node_materializations = vec![BuilderNodeMaterialization {
            builder_node_id: "builder-root".into(),
            object_refs: vec![
                StructuralObjectRef::Node("n1".into()),
                StructuralObjectRef::Node("n2".into()),
                StructuralObjectRef::Member("m1".into()),
                StructuralObjectRef::Load("l1".into()),
            ],
        }];

        let json = serde_json::to_string_pretty(&model).expect("serialize structural model");
        let decoded: StructuralModel =
            serde_json::from_str(&json).expect("deserialize structural model");

        assert_eq!(decoded.members.len(), 1);
        assert_eq!(decoded.members[0].semantic_tags, vec!["primary".to_owned()]);
        assert_eq!(decoded.loads.len(), 1);
        assert_eq!(decoded.releases.len(), 1);
        assert!(matches!(
            decoded.loads[0].target,
            AssignmentTargetRef::Member(ref id) if id == "m1"
        ));
        assert!(matches!(decoded.releases[0].target.end, MemberEnd::End));
        assert_eq!(decoded.builder_node_materializations.len(), 1);
        assert!(matches!(decoded.loads[0].kind, LoadKind::UniformLine));
        assert_eq!(
            decoded.generated_by_builder_node_for_object(&StructuralObjectRef::Member("m1".into())),
            Some("builder-root")
        );
    }

    #[test]
    fn structural_model_loads_accept_legacy_family_strings() {
        let json = r#"{
          "dimension": "2d-in-3d",
          "nodes": [],
          "members": [],
          "plates": [],
          "supports": [],
          "loads": [
            {
              "id": "l1",
              "target": { "Member": "m1" },
              "load_case_id": "gravity",
              "family": "distributed",
              "direction": { "x": 0.0, "y": -1.0, "z": 0.0 },
              "magnitude": 18.0
            }
          ],
          "releases": [],
          "load_cases": [],
          "builder_node_materializations": []
        }"#;

        let decoded: StructuralModel =
            serde_json::from_str(json).expect("deserialize legacy load family");

        assert!(matches!(decoded.loads[0].kind, LoadKind::UniformLine));
    }

    #[test]
    fn structural_model_accepts_legacy_objects_without_semantic_tags() {
        let json = r#"{
          "dimension": "2d-in-3d",
          "nodes": [
            { "id": "n1", "x": 0.0, "y": 0.0, "z": 0.0 },
            { "id": "n2", "x": 4.0, "y": 0.0, "z": 0.0 },
            { "id": "n3", "x": 4.0, "y": 3.0, "z": 0.0 }
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
              "boundary_nodes": ["n1", "n2", "n3"],
              "role": "slab",
              "thickness_m": 0.2,
              "material_id": "steel",
              "generated_from": "legacy"
            }
          ],
          "supports": [],
          "loads": [],
          "releases": [],
          "load_cases": [],
          "builder_node_materializations": []
        }"#;

        let decoded: StructuralModel =
            serde_json::from_str(json).expect("deserialize legacy semantic objects");

        assert!(decoded.members[0].semantic_tags.is_empty());
        assert!(decoded.plates[0].semantic_tags.is_empty());
    }

    #[test]
    fn load_assignment_reports_expected_kind_for_target() {
        let load = LoadAssignment {
            id: "l1".into(),
            target: AssignmentTargetRef::Plate("p1".into()),
            load_case_id: "wind".into(),
            kind: LoadKind::Area,
            direction: LoadVector {
                x: 0.0,
                y: -1.0,
                z: 0.0,
            },
            magnitude: 2.5,
        };

        assert!(load.kind_matches_target());
        assert!(matches!(load.expected_kind(), LoadKind::Area));
        assert_eq!(load.target.kind_label(), "plate");
    }
}
