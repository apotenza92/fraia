use crate::structural_app::{
    AssignmentTargetRef, LoadAssignment, LoadKind, LoadVector, StructuralMember, StructuralModel,
    StructuralNode, SupportAssignment,
};
use crate::types::LoadCase2D;
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedStickFrameInput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub section_id: String,
    pub material_id: String,
    #[serde(default)]
    pub segments: Vec<ImportedStickSegment>,
    #[serde(default)]
    pub supports: Vec<ImportedStickSupport>,
    #[serde(default)]
    pub cleanup: ImportedStickCleanupSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedStickSegment {
    pub id: String,
    pub start: ImportedPoint2D,
    pub end: ImportedPoint2D,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role_hint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uniform_line_load_kn_per_m: Option<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ImportedPoint2D {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedStickSupport {
    pub id: String,
    pub point: ImportedPoint2D,
    #[serde(default)]
    pub ux: bool,
    #[serde(default)]
    pub uy: bool,
    #[serde(default)]
    pub rz: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedStickCleanupSettings {
    #[serde(default = "default_merge_tolerance_m")]
    pub merge_tolerance_m: f64,
    #[serde(default = "default_intersection_tolerance_m")]
    pub intersection_tolerance_m: f64,
}

impl Default for ImportedStickCleanupSettings {
    fn default() -> Self {
        Self {
            merge_tolerance_m: default_merge_tolerance_m(),
            intersection_tolerance_m: default_intersection_tolerance_m(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedStickFrameArtifacts {
    pub structural_model: StructuralModel,
    pub imported_segment_count: usize,
    pub derived_member_count: usize,
    pub cleaned_node_count: usize,
    pub split_intersection_count: usize,
    pub merged_node_hit_count: usize,
}

#[derive(Debug, Clone)]
struct DerivedSegment {
    start: ImportedPoint2D,
    end: ImportedPoint2D,
    role_hint: Option<String>,
    uniform_line_load_kn_per_m: Option<f64>,
}

pub fn import_stick_frame_to_structural_model(
    input: &ImportedStickFrameInput,
) -> Result<ImportedStickFrameArtifacts> {
    if input.segments.is_empty() {
        bail!("imported stick frame must contain at least one segment");
    }
    if input.cleanup.merge_tolerance_m <= 0.0 || input.cleanup.intersection_tolerance_m <= 0.0 {
        bail!("cleanup tolerances must be positive");
    }

    let mut split_intersection_count = 0usize;
    let mut derived_segments = Vec::new();

    for (index, segment) in input.segments.iter().enumerate() {
        let mut params = vec![0.0, 1.0];
        for (other_index, other) in input.segments.iter().enumerate() {
            if index == other_index {
                continue;
            }
            if let Some((t, _u)) = segment_intersection_parameters(
                segment.start,
                segment.end,
                other.start,
                other.end,
                input.cleanup.intersection_tolerance_m,
            ) {
                if t > input.cleanup.intersection_tolerance_m
                    && t < 1.0 - input.cleanup.intersection_tolerance_m
                {
                    params.push(t);
                }
            }
        }
        params.sort_by(|a, b| a.total_cmp(b));
        params.dedup_by(|a, b| (*a - *b).abs() < input.cleanup.intersection_tolerance_m);
        if params.len() > 2 {
            split_intersection_count += params.len() - 2;
        }
        for window in params.windows(2) {
            let t0 = window[0];
            let t1 = window[1];
            let start = interpolate(segment.start, segment.end, t0);
            let end = interpolate(segment.start, segment.end, t1);
            if point_distance(start, end) <= input.cleanup.intersection_tolerance_m {
                continue;
            }
            derived_segments.push(DerivedSegment {
                start,
                end,
                role_hint: segment.role_hint.clone(),
                uniform_line_load_kn_per_m: segment.uniform_line_load_kn_per_m,
            });
        }
    }

    let mut canonical_nodes: Vec<ImportedPoint2D> = Vec::new();
    let mut merged_node_hit_count = 0usize;
    let node_id_for_point = |point: ImportedPoint2D,
                             nodes: &mut Vec<ImportedPoint2D>,
                             merged_hit_count: &mut usize|
     -> String {
        if let Some(index) = nodes.iter().position(|candidate| {
            point_distance(*candidate, point) <= input.cleanup.merge_tolerance_m
        }) {
            *merged_hit_count += 1;
            return format!("n{}", index + 1);
        }
        nodes.push(point);
        format!("n{}", nodes.len())
    };

    let mut members = Vec::new();
    let mut loads = Vec::new();
    for (index, segment) in derived_segments.iter().enumerate() {
        let start_id = node_id_for_point(
            segment.start,
            &mut canonical_nodes,
            &mut merged_node_hit_count,
        );
        let end_id = node_id_for_point(
            segment.end,
            &mut canonical_nodes,
            &mut merged_node_hit_count,
        );
        if start_id == end_id {
            continue;
        }
        let member_id = format!("m{}", index + 1);
        let role = segment
            .role_hint
            .clone()
            .unwrap_or_else(|| infer_member_role(segment.start, segment.end));
        members.push(StructuralMember {
            id: member_id.clone(),
            start_node: start_id,
            end_node: end_id,
            role,
            semantic_tags: Vec::new(),
            section_id: input.section_id.clone(),
            material_id: input.material_id.clone(),
        });
        if let Some(magnitude) = segment
            .uniform_line_load_kn_per_m
            .filter(|value| value.abs() > 1e-9)
        {
            loads.push(LoadAssignment {
                id: format!("load-{}", member_id),
                target: AssignmentTargetRef::Member(member_id),
                load_case_id: "gravity".into(),
                kind: LoadKind::UniformLine,
                direction: LoadVector {
                    x: 0.0,
                    y: -1.0,
                    z: 0.0,
                },
                magnitude: magnitude * 1000.0,
            });
        }
    }

    if members.is_empty() {
        bail!("cleanup produced no valid members");
    }

    let nodes: Vec<StructuralNode> = canonical_nodes
        .iter()
        .enumerate()
        .map(|(index, point)| StructuralNode {
            id: format!("n{}", index + 1),
            x: point.x,
            y: point.y,
            z: 0.0,
        })
        .collect();

    let supports: Vec<SupportAssignment> = input
        .supports
        .iter()
        .map(|support| {
            let node = nodes
                .iter()
                .min_by(|a, b| {
                    point_distance(
                        ImportedPoint2D { x: a.x, y: a.y },
                        support.point,
                    )
                    .total_cmp(&point_distance(
                        ImportedPoint2D { x: b.x, y: b.y },
                        support.point,
                    ))
                })
                .context("no cleaned nodes available for support snapping")?;
            let distance = point_distance(ImportedPoint2D { x: node.x, y: node.y }, support.point);
            if distance > input.cleanup.merge_tolerance_m * 2.0 {
                bail!(
                    "support {} could not be snapped within tolerance; nearest node distance = {} m",
                    support.id,
                    distance
                );
            }
            Ok(SupportAssignment {
                id: support.id.clone(),
                target_node: node.id.clone(),
                ux: support.ux,
                uy: support.uy,
                uz: false,
                rx: false,
                ry: false,
                rz: support.rz,
            })
        })
        .collect::<Result<_>>()?;

    Ok(ImportedStickFrameArtifacts {
        structural_model: StructuralModel {
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
        },
        imported_segment_count: input.segments.len(),
        derived_member_count: derived_segments.len(),
        cleaned_node_count: canonical_nodes.len(),
        split_intersection_count,
        merged_node_hit_count,
    })
}

fn default_merge_tolerance_m() -> f64 {
    0.01
}

fn default_intersection_tolerance_m() -> f64 {
    1e-6
}

fn point_distance(a: ImportedPoint2D, b: ImportedPoint2D) -> f64 {
    ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt()
}

fn interpolate(a: ImportedPoint2D, b: ImportedPoint2D, t: f64) -> ImportedPoint2D {
    ImportedPoint2D {
        x: a.x + (b.x - a.x) * t,
        y: a.y + (b.y - a.y) * t,
    }
}

fn infer_member_role(start: ImportedPoint2D, end: ImportedPoint2D) -> String {
    let dx = (end.x - start.x).abs();
    let dy = (end.y - start.y).abs();
    if dx <= 1e-9 && dy > 1e-9 {
        "column".into()
    } else if dy <= 1e-9 && dx > 1e-9 {
        "beam".into()
    } else if dy > dx * 5.0 {
        "column".into()
    } else if dx > dy * 5.0 {
        "beam".into()
    } else {
        "brace".into()
    }
}

fn segment_intersection_parameters(
    p1: ImportedPoint2D,
    p2: ImportedPoint2D,
    q1: ImportedPoint2D,
    q2: ImportedPoint2D,
    tolerance: f64,
) -> Option<(f64, f64)> {
    let r = (p2.x - p1.x, p2.y - p1.y);
    let s = (q2.x - q1.x, q2.y - q1.y);
    let denom = cross(r, s);
    let qp = (q1.x - p1.x, q1.y - p1.y);
    if denom.abs() <= tolerance {
        return None;
    }
    let t = cross(qp, s) / denom;
    let u = cross(qp, r) / denom;
    if t >= -tolerance && t <= 1.0 + tolerance && u >= -tolerance && u <= 1.0 + tolerance {
        Some((t.clamp(0.0, 1.0), u.clamp(0.0, 1.0)))
    } else {
        None
    }
}

fn cross(a: (f64, f64), b: (f64, f64)) -> f64 {
    a.0 * b.1 - a.1 * b.0
}

#[cfg(test)]
mod tests {
    use super::{
        ImportedPoint2D, ImportedStickCleanupSettings, ImportedStickFrameInput,
        ImportedStickSegment, ImportedStickSupport, import_stick_frame_to_structural_model,
    };

    #[test]
    fn splits_t_junctions_and_infers_roles() {
        let imported = ImportedStickFrameInput {
            name: Some("imported frame".into()),
            section_id: "250UB".into(),
            material_id: "steel".into(),
            segments: vec![
                ImportedStickSegment {
                    id: "beam".into(),
                    start: ImportedPoint2D { x: 0.0, y: 3.0 },
                    end: ImportedPoint2D { x: 6.0, y: 3.0 },
                    role_hint: None,
                    uniform_line_load_kn_per_m: Some(8.0),
                },
                ImportedStickSegment {
                    id: "left-col".into(),
                    start: ImportedPoint2D { x: 0.0, y: 0.0 },
                    end: ImportedPoint2D { x: 0.0, y: 3.0 },
                    role_hint: None,
                    uniform_line_load_kn_per_m: None,
                },
                ImportedStickSegment {
                    id: "mid-col".into(),
                    start: ImportedPoint2D { x: 3.0, y: 0.0 },
                    end: ImportedPoint2D { x: 3.0, y: 3.0 },
                    role_hint: None,
                    uniform_line_load_kn_per_m: None,
                },
                ImportedStickSegment {
                    id: "right-col".into(),
                    start: ImportedPoint2D { x: 6.0, y: 0.0 },
                    end: ImportedPoint2D { x: 6.0, y: 3.0 },
                    role_hint: None,
                    uniform_line_load_kn_per_m: None,
                },
            ],
            supports: vec![
                ImportedStickSupport {
                    id: "s1".into(),
                    point: ImportedPoint2D { x: 0.0, y: 0.0 },
                    ux: true,
                    uy: true,
                    rz: true,
                },
                ImportedStickSupport {
                    id: "s2".into(),
                    point: ImportedPoint2D { x: 3.0, y: 0.0 },
                    ux: true,
                    uy: true,
                    rz: true,
                },
                ImportedStickSupport {
                    id: "s3".into(),
                    point: ImportedPoint2D { x: 6.0, y: 0.0 },
                    ux: true,
                    uy: true,
                    rz: true,
                },
            ],
            cleanup: ImportedStickCleanupSettings::default(),
        };

        let artifacts = import_stick_frame_to_structural_model(&imported).expect("import");

        assert_eq!(artifacts.imported_segment_count, 4);
        assert!(artifacts.split_intersection_count >= 1);
        assert_eq!(artifacts.structural_model.supports.len(), 3);
        assert_eq!(artifacts.structural_model.loads.len(), 2);
        assert!(artifacts.structural_model.members.len() >= 5);
        assert!(
            artifacts
                .structural_model
                .members
                .iter()
                .any(|m| m.role == "beam")
        );
        assert!(
            artifacts
                .structural_model
                .members
                .iter()
                .any(|m| m.role == "column")
        );
        assert!(
            artifacts
                .structural_model
                .nodes
                .iter()
                .any(|n| (n.x - 3.0).abs() < 1e-9 && (n.y - 3.0).abs() < 1e-9)
        );
    }

    #[test]
    fn merges_near_coincident_points_within_tolerance() {
        let imported = ImportedStickFrameInput {
            name: None,
            section_id: "250UB".into(),
            material_id: "steel".into(),
            segments: vec![
                ImportedStickSegment {
                    id: "a".into(),
                    start: ImportedPoint2D { x: 0.0, y: 0.0 },
                    end: ImportedPoint2D { x: 1.0, y: 0.0 },
                    role_hint: None,
                    uniform_line_load_kn_per_m: None,
                },
                ImportedStickSegment {
                    id: "b".into(),
                    start: ImportedPoint2D { x: 1.005, y: 0.0 },
                    end: ImportedPoint2D { x: 2.0, y: 0.0 },
                    role_hint: None,
                    uniform_line_load_kn_per_m: None,
                },
            ],
            supports: vec![],
            cleanup: ImportedStickCleanupSettings {
                merge_tolerance_m: 0.01,
                intersection_tolerance_m: 1e-6,
            },
        };
        let artifacts = import_stick_frame_to_structural_model(&imported).expect("import");
        assert_eq!(artifacts.cleaned_node_count, 3);
        assert!(artifacts.merged_node_hit_count > 0);
    }
}
