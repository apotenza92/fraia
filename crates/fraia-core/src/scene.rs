use crate::structural_app::{AssignmentTargetRef, MemberEnd, StructuralModel, StructuralObjectRef};
use fraia_geometry::Point3;

#[derive(Debug, Clone)]
pub struct ScenePoint {
    pub object: StructuralObjectRef,
    pub position: Point3,
}

#[derive(Debug, Clone)]
pub struct SceneLine {
    pub object: StructuralObjectRef,
    pub role: String,
    pub start: Point3,
    pub end: Point3,
}

#[derive(Debug, Clone)]
pub struct ScenePlate {
    pub object: StructuralObjectRef,
    pub role: String,
    pub vertices: Vec<Point3>,
}

#[derive(Debug, Clone)]
pub struct SupportGlyph {
    pub object: StructuralObjectRef,
    pub at: Point3,
}

#[derive(Debug, Clone)]
pub struct LoadGlyph {
    pub object: StructuralObjectRef,
    pub start: Point3,
    pub end: Point3,
}

#[derive(Debug, Clone)]
pub struct ReleaseGlyph {
    pub object: StructuralObjectRef,
    pub at: Point3,
}

#[derive(Debug, Clone, Default)]
pub struct StructuralScene {
    pub points: Vec<ScenePoint>,
    pub lines: Vec<SceneLine>,
    pub plates: Vec<ScenePlate>,
    pub supports: Vec<SupportGlyph>,
    pub loads: Vec<LoadGlyph>,
    pub releases: Vec<ReleaseGlyph>,
}

impl StructuralScene {
    pub fn from_structural_model(model: &StructuralModel) -> Self {
        let mut scene = StructuralScene::default();

        for node in &model.nodes {
            scene.points.push(ScenePoint {
                object: StructuralObjectRef::Node(node.id.clone()),
                position: node.point(),
            });
        }

        for member in &model.members {
            let Some(start) = model.node_by_id(&member.start_node) else {
                continue;
            };
            let Some(end) = model.node_by_id(&member.end_node) else {
                continue;
            };
            scene.lines.push(SceneLine {
                object: StructuralObjectRef::Member(member.id.clone()),
                role: member.role.clone(),
                start: start.point(),
                end: end.point(),
            });
        }

        for plate in &model.plates {
            let mut vertices = Vec::new();
            for node_id in &plate.boundary_nodes {
                let Some(node) = model.node_by_id(node_id) else {
                    vertices.clear();
                    break;
                };
                vertices.push(node.point());
            }
            if vertices.len() >= 3 {
                scene.plates.push(ScenePlate {
                    object: StructuralObjectRef::Plate(plate.id.clone()),
                    role: plate.role.clone(),
                    vertices,
                });
            }
        }

        for support in &model.supports {
            let Some(node) = model.node_by_id(&support.target_node) else {
                continue;
            };
            scene.supports.push(SupportGlyph {
                object: StructuralObjectRef::Support(support.id.clone()),
                at: node.point(),
            });
        }

        for load in &model.loads {
            if let Some(start) = glyph_anchor_for_target(model, &load.target) {
                let length = 1.2;
                let norm = (load.direction.x.powi(2)
                    + load.direction.y.powi(2)
                    + load.direction.z.powi(2))
                .sqrt();
                let (dx, dy, dz) = if norm > 1e-9 {
                    (
                        load.direction.x / norm,
                        load.direction.y / norm,
                        load.direction.z / norm,
                    )
                } else {
                    (0.0, -1.0, 0.0)
                };
                scene.loads.push(LoadGlyph {
                    object: StructuralObjectRef::Load(load.id.clone()),
                    start: start.clone(),
                    end: Point3::new(
                        start.x() + dx * length,
                        start.y() + dy * length,
                        start.z() + dz * length,
                    ),
                });
            }
        }

        for release in &model.releases {
            let Some(member) = model
                .members
                .iter()
                .find(|member| member.id == release.target.member_id)
            else {
                continue;
            };
            let node_id = match release.target.end {
                MemberEnd::Start => &member.start_node,
                MemberEnd::End => &member.end_node,
            };
            let Some(node) = model.node_by_id(node_id) else {
                continue;
            };
            scene.releases.push(ReleaseGlyph {
                object: StructuralObjectRef::Release(release.id.clone()),
                at: node.point(),
            });
        }

        scene
    }
}

fn glyph_anchor_for_target(
    model: &StructuralModel,
    target: &AssignmentTargetRef,
) -> Option<Point3> {
    match target {
        AssignmentTargetRef::Node(node_id) => model.node_by_id(node_id).map(|node| node.point()),
        AssignmentTargetRef::Member(member_id) => {
            let member = model
                .members
                .iter()
                .find(|member| &member.id == member_id)?;
            let start = model.node_by_id(&member.start_node)?;
            let end = model.node_by_id(&member.end_node)?;
            Some(Point3::new(
                (start.x + end.x) * 0.5,
                (start.y + end.y) * 0.5,
                (start.z + end.z) * 0.5,
            ))
        }
        AssignmentTargetRef::Plate(plate_id) => {
            let plate = model.plates.iter().find(|plate| &plate.id == plate_id)?;
            let mut points = Vec::new();
            for node_id in &plate.boundary_nodes {
                points.push(model.node_by_id(node_id)?.point());
            }
            let count = points.len() as f64;
            if count <= 0.0 {
                return None;
            }
            Some(Point3::new(
                points.iter().map(|point| point.x()).sum::<f64>() / count,
                points.iter().map(|point| point.y()).sum::<f64>() / count,
                points.iter().map(|point| point.z()).sum::<f64>() / count,
            ))
        }
    }
}
