use crate::catalog::{section_catalog, steel_material};
use crate::types::Section;
use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

const ANALYSIS_TOLERANCE: f64 = 1e-9;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplySupportedBeamSizingRequest {
    pub span_m: f64,
    pub distributed_load_kn_per_m: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub point_load_kn: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub point_load_x_m: Option<f64>,
    pub target_max_utilization: f64,
    pub target_deflection_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplySupportedBeamResponse {
    pub left_reaction_kn: f64,
    pub right_reaction_kn: f64,
    pub max_shear_kn: f64,
    pub max_moment_knm: f64,
    pub max_moment_x_m: f64,
    pub max_deflection_mm: f64,
    pub max_deflection_x_m: f64,
    pub section_modulus_m3: f64,
    pub max_bending_stress_mpa: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplySupportedBeamSizingCandidate {
    pub section_id: String,
    pub mass_kg: f64,
    pub max_utilization: f64,
    pub max_deflection_mm: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deflection_ratio: Option<f64>,
    pub left_reaction_kn: f64,
    pub right_reaction_kn: f64,
    pub max_shear_kn: f64,
    pub max_moment_knm: f64,
    pub max_moment_x_m: f64,
    pub max_deflection_x_m: f64,
    pub section_modulus_m3: f64,
    pub max_bending_stress_mpa: f64,
    pub feasible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplySupportedBeamSizingResult {
    pub request: SimplySupportedBeamSizingRequest,
    pub candidates: Vec<SimplySupportedBeamSizingCandidate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chosen: Option<SimplySupportedBeamSizingCandidate>,
}

pub fn analyze_simply_supported_beam(
    section: &Section,
    request: &SimplySupportedBeamSizingRequest,
) -> Result<SimplySupportedBeamResponse> {
    if request.span_m <= 0.0 {
        bail!("beam span must be greater than zero");
    }
    if section.i <= 0.0 || section.depth <= 0.0 {
        bail!(
            "section {} must have positive inertia and depth",
            section.id
        );
    }

    let span_m = request.span_m;
    let elastic_modulus_pa = steel_material().e;
    let udl_n_per_m = request.distributed_load_kn_per_m * 1000.0;
    let point_load = normalized_point_load(request);

    let left_reaction_n = support_reaction_left_n(span_m, udl_n_per_m, point_load);
    let right_reaction_n = support_reaction_right_n(span_m, udl_n_per_m, point_load);
    let max_shear_n = left_reaction_n.abs().max(right_reaction_n.abs());

    let mut moment_positions = vec![0.0, span_m];
    if let Some((_, point_x_m)) = point_load {
        moment_positions.push(point_x_m);
    }
    moment_positions.extend(moment_station_candidates_m(
        span_m,
        udl_n_per_m,
        point_load,
        left_reaction_n,
    ));
    dedup_positions(&mut moment_positions);

    let (max_moment_x_m, max_moment_nm) = moment_positions
        .iter()
        .copied()
        .map(|x_m| {
            (
                x_m,
                bending_moment_nm(x_m, udl_n_per_m, point_load, left_reaction_n).abs(),
            )
        })
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .unwrap_or((0.0, 0.0));

    let mut deflection_positions = vec![0.0, span_m, max_moment_x_m];
    if let Some((_, point_x_m)) = point_load {
        deflection_positions.push(point_x_m);
    }
    deflection_positions.extend(deflection_station_candidates_m(
        span_m,
        udl_n_per_m,
        point_load,
        elastic_modulus_pa,
        section.i,
    ));
    dedup_positions(&mut deflection_positions);

    let (max_deflection_x_m, max_deflection_m) = deflection_positions
        .iter()
        .copied()
        .map(|x_m| {
            (
                x_m,
                beam_deflection_m(
                    x_m,
                    span_m,
                    udl_n_per_m,
                    point_load,
                    elastic_modulus_pa,
                    section.i,
                )
                .abs(),
            )
        })
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .unwrap_or((0.0, 0.0));

    let section_modulus_m3 = section.i / (section.depth * 0.5);
    let max_bending_stress_pa = if section_modulus_m3 > ANALYSIS_TOLERANCE {
        max_moment_nm / section_modulus_m3
    } else {
        0.0
    };

    Ok(SimplySupportedBeamResponse {
        left_reaction_kn: left_reaction_n / 1000.0,
        right_reaction_kn: right_reaction_n / 1000.0,
        max_shear_kn: max_shear_n / 1000.0,
        max_moment_knm: max_moment_nm / 1000.0,
        max_moment_x_m,
        max_deflection_mm: max_deflection_m * 1000.0,
        max_deflection_x_m,
        section_modulus_m3,
        max_bending_stress_mpa: max_bending_stress_pa / 1e6,
    })
}

pub fn deflection_at_x_mm(
    section: &Section,
    request: &SimplySupportedBeamSizingRequest,
    x_m: f64,
) -> Result<f64> {
    if request.span_m <= 0.0 {
        bail!("beam span must be greater than zero");
    }
    if section.i <= 0.0 || section.depth <= 0.0 {
        bail!(
            "section {} must have positive inertia and depth",
            section.id
        );
    }

    let x_m = x_m.clamp(0.0, request.span_m);
    let deflection_m = beam_deflection_m(
        x_m,
        request.span_m,
        request.distributed_load_kn_per_m * 1000.0,
        normalized_point_load(request),
        steel_material().e,
        section.i,
    );
    Ok(deflection_m.abs() * 1000.0)
}

pub fn size_simply_supported_beam(
    request: &SimplySupportedBeamSizingRequest,
) -> Result<SimplySupportedBeamSizingResult> {
    let steel = steel_material();
    let mut candidates = Vec::new();

    for section in section_catalog() {
        let response = analyze_simply_supported_beam(&section, request)?;
        let mass_kg = section.mass_kg_per_m * request.span_m;
        let max_utilization = (response.max_bending_stress_mpa * 1e6) / steel.fy;
        let deflection_ratio = if response.max_deflection_mm > ANALYSIS_TOLERANCE {
            Some(request.span_m / (response.max_deflection_mm / 1000.0))
        } else {
            None
        };
        let feasible = max_utilization <= request.target_max_utilization
            && deflection_ratio.unwrap_or(f64::INFINITY) >= request.target_deflection_ratio;

        candidates.push(SimplySupportedBeamSizingCandidate {
            section_id: section.id,
            mass_kg,
            max_utilization,
            max_deflection_mm: response.max_deflection_mm,
            deflection_ratio,
            left_reaction_kn: response.left_reaction_kn,
            right_reaction_kn: response.right_reaction_kn,
            max_shear_kn: response.max_shear_kn,
            max_moment_knm: response.max_moment_knm,
            max_moment_x_m: response.max_moment_x_m,
            max_deflection_x_m: response.max_deflection_x_m,
            section_modulus_m3: response.section_modulus_m3,
            max_bending_stress_mpa: response.max_bending_stress_mpa,
            feasible,
        });
    }

    candidates.sort_by(|a, b| a.mass_kg.total_cmp(&b.mass_kg));
    let chosen = candidates
        .iter()
        .find(|candidate| candidate.feasible)
        .cloned();

    Ok(SimplySupportedBeamSizingResult {
        request: request.clone(),
        candidates,
        chosen,
    })
}

fn normalized_point_load(request: &SimplySupportedBeamSizingRequest) -> Option<(f64, f64)> {
    let point_load_kn = request
        .point_load_kn
        .filter(|value| value.abs() > ANALYSIS_TOLERANCE)?;
    Some((
        point_load_kn * 1000.0,
        request
            .point_load_x_m
            .unwrap_or(request.span_m * 0.5)
            .clamp(0.0, request.span_m),
    ))
}

fn support_reaction_left_n(span_m: f64, udl_n_per_m: f64, point_load: Option<(f64, f64)>) -> f64 {
    let point_component = point_load
        .map(|(point_load_n, point_x_m)| point_load_n * (span_m - point_x_m) / span_m)
        .unwrap_or(0.0);
    (udl_n_per_m * span_m * 0.5) + point_component
}

fn support_reaction_right_n(span_m: f64, udl_n_per_m: f64, point_load: Option<(f64, f64)>) -> f64 {
    let point_component = point_load
        .map(|(point_load_n, point_x_m)| point_load_n * point_x_m / span_m)
        .unwrap_or(0.0);
    (udl_n_per_m * span_m * 0.5) + point_component
}

fn bending_moment_nm(
    x_m: f64,
    udl_n_per_m: f64,
    point_load: Option<(f64, f64)>,
    left_reaction_n: f64,
) -> f64 {
    let mut moment_nm = left_reaction_n * x_m - (udl_n_per_m * x_m.powi(2) * 0.5);
    if let Some((point_load_n, point_x_m)) = point_load
        && x_m >= point_x_m
    {
        moment_nm -= point_load_n * (x_m - point_x_m);
    }
    moment_nm
}

fn beam_deflection_m(
    x_m: f64,
    span_m: f64,
    udl_n_per_m: f64,
    point_load: Option<(f64, f64)>,
    elastic_modulus_pa: f64,
    inertia_m4: f64,
) -> f64 {
    let udl_deflection_m = if udl_n_per_m.abs() > ANALYSIS_TOLERANCE {
        (udl_n_per_m * x_m * (span_m.powi(3) - (2.0 * span_m * x_m.powi(2)) + x_m.powi(3)))
            / (24.0 * elastic_modulus_pa * inertia_m4)
    } else {
        0.0
    };

    let point_deflection_m = if let Some((point_load_n, point_x_m)) = point_load {
        if x_m <= point_x_m {
            let b_m = span_m - point_x_m;
            point_load_n * b_m * x_m * (span_m.powi(2) - b_m.powi(2) - x_m.powi(2))
                / (6.0 * span_m * elastic_modulus_pa * inertia_m4)
        } else {
            let remaining_m = span_m - x_m;
            point_load_n
                * point_x_m
                * remaining_m
                * (span_m.powi(2) - point_x_m.powi(2) - remaining_m.powi(2))
                / (6.0 * span_m * elastic_modulus_pa * inertia_m4)
        }
    } else {
        0.0
    };

    udl_deflection_m + point_deflection_m
}

fn beam_slope_rad(
    x_m: f64,
    span_m: f64,
    udl_n_per_m: f64,
    point_load: Option<(f64, f64)>,
    elastic_modulus_pa: f64,
    inertia_m4: f64,
) -> f64 {
    let udl_slope_rad = if udl_n_per_m.abs() > ANALYSIS_TOLERANCE {
        udl_n_per_m * (span_m.powi(3) - (6.0 * span_m * x_m.powi(2)) + (4.0 * x_m.powi(3)))
            / (24.0 * elastic_modulus_pa * inertia_m4)
    } else {
        0.0
    };

    let point_slope_rad = if let Some((point_load_n, point_x_m)) = point_load {
        if x_m <= point_x_m {
            let b_m = span_m - point_x_m;
            point_load_n * b_m * (span_m.powi(2) - b_m.powi(2) - (3.0 * x_m.powi(2)))
                / (6.0 * span_m * elastic_modulus_pa * inertia_m4)
        } else {
            let remaining_m = span_m - x_m;
            let h_m2 = span_m.powi(2) - point_x_m.powi(2);
            point_load_n * point_x_m * ((3.0 * remaining_m.powi(2)) - h_m2)
                / (6.0 * span_m * elastic_modulus_pa * inertia_m4)
        }
    } else {
        0.0
    };

    udl_slope_rad + point_slope_rad
}

fn moment_station_candidates_m(
    span_m: f64,
    udl_n_per_m: f64,
    point_load: Option<(f64, f64)>,
    left_reaction_n: f64,
) -> Vec<f64> {
    let mut candidates = Vec::new();
    if udl_n_per_m.abs() > ANALYSIS_TOLERANCE {
        let before_point_x_m = left_reaction_n / udl_n_per_m;
        if let Some((point_load_n, point_x_m)) = point_load {
            if before_point_x_m >= 0.0 && before_point_x_m <= point_x_m {
                candidates.push(before_point_x_m);
            }
            let after_point_x_m = (left_reaction_n - point_load_n) / udl_n_per_m;
            if after_point_x_m >= point_x_m && after_point_x_m <= span_m {
                candidates.push(after_point_x_m);
            }
        } else if before_point_x_m >= 0.0 && before_point_x_m <= span_m {
            candidates.push(before_point_x_m);
        }
    }
    candidates
}

fn deflection_station_candidates_m(
    span_m: f64,
    udl_n_per_m: f64,
    point_load: Option<(f64, f64)>,
    elastic_modulus_pa: f64,
    inertia_m4: f64,
) -> Vec<f64> {
    let interior_point_x_m = point_load.and_then(|(_, point_x_m)| {
        if point_x_m > ANALYSIS_TOLERANCE && point_x_m < (span_m - ANALYSIS_TOLERANCE) {
            Some(point_x_m)
        } else {
            None
        }
    });
    let intervals = if let Some(point_x_m) = interior_point_x_m {
        vec![(0.0, point_x_m), (point_x_m, span_m)]
    } else {
        vec![(0.0, span_m)]
    };

    let mut roots = Vec::new();
    for (start_x_m, end_x_m) in intervals {
        let segment_length_m = (end_x_m - start_x_m).abs();
        if segment_length_m <= ANALYSIS_TOLERANCE {
            continue;
        }
        let samples = 256usize;
        let mut previous_x_m = start_x_m;
        let mut previous_slope = beam_slope_rad(
            previous_x_m,
            span_m,
            udl_n_per_m,
            point_load,
            elastic_modulus_pa,
            inertia_m4,
        );
        if previous_slope.abs() <= 1e-12 {
            roots.push(previous_x_m);
        }
        for sample_index in 1..=samples {
            let x_m = start_x_m + segment_length_m * (sample_index as f64) / (samples as f64);
            let slope = beam_slope_rad(
                x_m,
                span_m,
                udl_n_per_m,
                point_load,
                elastic_modulus_pa,
                inertia_m4,
            );
            if slope.abs() <= 1e-12 {
                roots.push(x_m);
            } else if previous_slope.abs() > 1e-12 && previous_slope.signum() != slope.signum() {
                roots.push(bisect_slope_root_m(
                    previous_x_m,
                    x_m,
                    span_m,
                    udl_n_per_m,
                    point_load,
                    elastic_modulus_pa,
                    inertia_m4,
                ));
            }
            previous_x_m = x_m;
            previous_slope = slope;
        }
    }

    roots
}

fn bisect_slope_root_m(
    mut left_x_m: f64,
    mut right_x_m: f64,
    span_m: f64,
    udl_n_per_m: f64,
    point_load: Option<(f64, f64)>,
    elastic_modulus_pa: f64,
    inertia_m4: f64,
) -> f64 {
    let mut left_slope = beam_slope_rad(
        left_x_m,
        span_m,
        udl_n_per_m,
        point_load,
        elastic_modulus_pa,
        inertia_m4,
    );
    for _ in 0..80 {
        let mid_x_m = 0.5 * (left_x_m + right_x_m);
        let mid_slope = beam_slope_rad(
            mid_x_m,
            span_m,
            udl_n_per_m,
            point_load,
            elastic_modulus_pa,
            inertia_m4,
        );
        if mid_slope.abs() <= 1e-12 || (right_x_m - left_x_m).abs() <= 1e-12 {
            return mid_x_m;
        }
        if left_slope.signum() == mid_slope.signum() {
            left_x_m = mid_x_m;
            left_slope = mid_slope;
        } else {
            right_x_m = mid_x_m;
        }
    }
    0.5 * (left_x_m + right_x_m)
}

fn dedup_positions(positions: &mut Vec<f64>) {
    positions.retain(|position| position.is_finite());
    positions.sort_by(|a, b| a.total_cmp(b));
    positions.dedup_by(|a, b| (*a - *b).abs() <= 1e-8);
}

#[cfg(test)]
mod tests {
    use super::{
        SimplySupportedBeamSizingRequest, analyze_simply_supported_beam, deflection_at_x_mm,
        size_simply_supported_beam,
    };
    use crate::archetypes::{
        build_frame_model_from_builder_graph, simply_supported_beam_builder_graph,
    };
    use crate::catalog::section_by_id;
    use crate::frame2d::solve_frame_2d;

    fn approx_eq(actual: f64, expected: f64, tolerance: f64) {
        assert!(
            (actual - expected).abs() <= tolerance,
            "expected {expected} ± {tolerance}, got {actual}"
        );
    }

    #[test]
    fn analyzes_standard_uniform_load_case_against_closed_form_reference_values() {
        let section = section_by_id("250UB").expect("section");
        let result = analyze_simply_supported_beam(
            &section,
            &SimplySupportedBeamSizingRequest {
                span_m: 6.0,
                distributed_load_kn_per_m: 8.0,
                point_load_kn: None,
                point_load_x_m: None,
                target_max_utilization: 1.0,
                target_deflection_ratio: 0.0,
            },
        )
        .expect("analyze");

        approx_eq(result.left_reaction_kn, 24.0, 1e-6);
        approx_eq(result.right_reaction_kn, 24.0, 1e-6);
        approx_eq(result.max_shear_kn, 24.0, 1e-6);
        approx_eq(result.max_moment_knm, 36.0, 1e-6);
        approx_eq(result.max_moment_x_m, 3.0, 1e-6);
        approx_eq(result.max_deflection_mm, 13.5, 1e-3);
        approx_eq(result.max_deflection_x_m, 3.0, 1e-6);
        approx_eq(result.max_bending_stress_mpa, 90.0, 1e-3);
    }

    #[test]
    fn analyzes_standard_midspan_point_load_case_against_closed_form_reference_values() {
        let section = section_by_id("250UB").expect("section");
        let result = analyze_simply_supported_beam(
            &section,
            &SimplySupportedBeamSizingRequest {
                span_m: 6.0,
                distributed_load_kn_per_m: 0.0,
                point_load_kn: Some(20.0),
                point_load_x_m: Some(3.0),
                target_max_utilization: 1.0,
                target_deflection_ratio: 0.0,
            },
        )
        .expect("analyze");

        approx_eq(result.left_reaction_kn, 10.0, 1e-6);
        approx_eq(result.right_reaction_kn, 10.0, 1e-6);
        approx_eq(result.max_shear_kn, 10.0, 1e-6);
        approx_eq(result.max_moment_knm, 30.0, 1e-6);
        approx_eq(result.max_moment_x_m, 3.0, 1e-6);
        approx_eq(result.max_deflection_mm, 9.0, 1e-3);
        approx_eq(result.max_deflection_x_m, 3.0, 1e-6);
        approx_eq(result.max_bending_stress_mpa, 75.0, 1e-3);
    }

    #[test]
    fn analyzes_standard_combined_case_and_matches_section_modulus_stress() {
        let section = section_by_id("250UB").expect("section");
        let result = analyze_simply_supported_beam(
            &section,
            &SimplySupportedBeamSizingRequest {
                span_m: 6.0,
                distributed_load_kn_per_m: 8.0,
                point_load_kn: Some(20.0),
                point_load_x_m: Some(3.0),
                target_max_utilization: 1.0,
                target_deflection_ratio: 0.0,
            },
        )
        .expect("analyze");

        approx_eq(result.left_reaction_kn, 34.0, 1e-6);
        approx_eq(result.right_reaction_kn, 34.0, 1e-6);
        approx_eq(result.max_shear_kn, 34.0, 1e-6);
        approx_eq(result.max_moment_knm, 66.0, 1e-6);
        approx_eq(result.max_moment_x_m, 3.0, 1e-6);
        approx_eq(result.max_deflection_mm, 22.5, 1e-3);
        approx_eq(result.max_deflection_x_m, 3.0, 1e-6);
        approx_eq(result.section_modulus_m3, 4.0e-4, 1e-12);
        approx_eq(result.max_bending_stress_mpa, 165.0, 1e-3);
    }

    #[test]
    fn sizes_a_simple_beam_to_a_feasible_section() {
        let result = size_simply_supported_beam(&SimplySupportedBeamSizingRequest {
            span_m: 6.0,
            distributed_load_kn_per_m: 8.0,
            point_load_kn: Some(20.0),
            point_load_x_m: Some(3.0),
            target_max_utilization: 0.67,
            target_deflection_ratio: 250.0,
        })
        .expect("size beam");

        assert!(!result.candidates.is_empty());
        let chosen = result.chosen.expect("chosen section");
        assert!(chosen.feasible);
        assert!(chosen.max_utilization <= 0.67);
        assert!(chosen.deflection_ratio.unwrap_or(0.0) >= 250.0);
        approx_eq(chosen.max_moment_knm, 66.0, 1e-6);
        approx_eq(chosen.max_shear_kn, 34.0, 1e-6);
        approx_eq(chosen.max_bending_stress_mpa, 165.0, 1e-3);
    }

    #[test]
    fn deflection_at_x_matches_midspan_reference_value() {
        let section = section_by_id("250UB").expect("section");
        let request = SimplySupportedBeamSizingRequest {
            span_m: 6.0,
            distributed_load_kn_per_m: 8.0,
            point_load_kn: Some(20.0),
            point_load_x_m: Some(3.0),
            target_max_utilization: 0.67,
            target_deflection_ratio: 250.0,
        };

        let deflection_mm = deflection_at_x_mm(&section, &request, 3.0).expect("deflection");
        approx_eq(deflection_mm, 22.5, 1e-3);
    }

    #[test]
    fn current_frame_model_tracks_exact_demo_beam_response_closely() {
        let request = SimplySupportedBeamSizingRequest {
            span_m: 6.0,
            distributed_load_kn_per_m: 8.0,
            point_load_kn: Some(20.0),
            point_load_x_m: Some(3.0),
            target_max_utilization: 0.67,
            target_deflection_ratio: 250.0,
        };
        let section = section_by_id("250UB").expect("section");
        let exact = analyze_simply_supported_beam(&section, &request).expect("exact analysis");

        let graph = simply_supported_beam_builder_graph(
            "builder.beam.reference",
            "250UB",
            request.span_m,
            request.distributed_load_kn_per_m,
            request.point_load_kn,
            request.point_load_x_m,
            None,
            None,
        );
        let model = build_frame_model_from_builder_graph(&graph).expect("beam frame model");
        let combo = model.combos.first().expect("combo");
        let result = solve_frame_2d(&model, combo).expect("solve frame model");

        let left_support_id = &model.supports[0].node;
        let right_support_id = &model.supports[1].node;
        let left_reaction_kn = result
            .node_results
            .iter()
            .find(|node| &node.id == left_support_id)
            .map(|node| node.rxn_fy_n / 1000.0)
            .expect("left reaction");
        let right_reaction_kn = result
            .node_results
            .iter()
            .find(|node| &node.id == right_support_id)
            .map(|node| node.rxn_fy_n / 1000.0)
            .expect("right reaction");
        let max_moment_knm = result
            .element_results
            .iter()
            .map(|element| element.moment_nm.abs() / 1000.0)
            .fold(0.0, f64::max);
        let max_utilization = result
            .element_results
            .iter()
            .map(|element| element.utilization)
            .fold(0.0, f64::max);

        approx_eq(left_reaction_kn, exact.left_reaction_kn, 1e-6);
        approx_eq(right_reaction_kn, exact.right_reaction_kn, 1e-6);
        approx_eq(max_moment_knm, exact.max_moment_knm, 1e-6);
        approx_eq(max_utilization, exact.max_bending_stress_mpa / 300.0, 1e-6);
        approx_eq(
            result.metrics.max_uy_m * 1000.0,
            exact.max_deflection_mm,
            0.2,
        );
    }
}
