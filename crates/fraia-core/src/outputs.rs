use crate::{
    BeamValueComparison, BuilderNodeStatus, CalculixExecutionOutcome, CheckReport, CheckResult,
    CurrentFrameCalculixExecutionArtifacts, DesignActionReport, Frame2DRealization,
    ImportedStickFrameArtifacts, ProjectFile, QuantityKind, SimplySupportedBeamAnalysisArtifacts,
    SimplySupportedBeamCalculixArtifacts, SimplySupportedBeamCalculixExecutionArtifacts,
    SimplySupportedBeamSizingResult, UnitProfile, ValidationReport, format_quantity_from_unit,
};

pub fn render_validation_summary(
    project: &ProjectFile,
    validation: &ValidationReport,
    realization: Option<&Frame2DRealization>,
    design_actions: Option<&DesignActionReport>,
    checks: Option<&CheckReport>,
) -> String {
    let mut lines = vec![
        format!("# Fraia validation: {}", project.name),
        String::new(),
        format!("- Building type: {}", project.intent.building_type),
        format!("- Validation diagnostics: {}", validation.diagnostics.len()),
        format!(
            "- Realization diagnostics: {}",
            realization.map(|r| r.diagnostics.len()).unwrap_or(0)
        ),
        format!(
            "- Design action packets: {}",
            design_actions
                .map(|r| r.member_actions.len() + r.support_reactions.len())
                .unwrap_or(0)
        ),
        format!(
            "- Check results: {}",
            checks.map(|r| r.results.len()).unwrap_or(0)
        ),
    ];
    if let Some(graph) = &project.builder_graph {
        let (materialized_count, proposed_count, diverged_count) =
            graph
                .nodes
                .iter()
                .fold((0usize, 0usize, 0usize), |counts, node| match node.status {
                    BuilderNodeStatus::Materialized => (counts.0 + 1, counts.1, counts.2),
                    BuilderNodeStatus::Proposed => (counts.0, counts.1 + 1, counts.2),
                    BuilderNodeStatus::DivergedFromMaterialization => {
                        (counts.0, counts.1, counts.2 + 1)
                    }
                });
        lines.push(format!(
            "- Builder graph root nodes: {}",
            graph.root_node_ids.len()
        ));
        lines.push(format!("- Builder graph nodes: {}", graph.nodes.len()));
        lines.push(format!(
            "- Builder node statuses: {} materialized / {} proposed / {} diverged",
            materialized_count, proposed_count, diverged_count
        ));
        if let Some(structural) = project.structural_model.as_ref() {
            let total_generated_objects: usize = structural
                .builder_node_materializations
                .iter()
                .map(|entry| entry.object_refs.len())
                .sum();
            lines.push(format!(
                "- Total builder-generated objects: {}",
                total_generated_objects
            ));
        }
        if let Some(root_id) = graph.root_node_ids.first()
            && let Some(root) = graph.nodes.iter().find(|node| &node.id == root_id)
        {
            lines.push(format!("- Root archetype: {}", root.archetype_id));
            lines.push(format!("- Root status: {:?}", root.status));
            if let Some(structural) = project.structural_model.as_ref()
                && let Some(materialization) = structural.materialization_for_builder_node(&root.id)
            {
                lines.push(format!(
                    "- Root generated objects: {}",
                    materialization.object_refs.len()
                ));
            }
        }
    }

    lines.extend([
        String::new(),
        "## Validation diagnostics".into(),
        String::new(),
    ]);
    if validation.diagnostics.is_empty() {
        lines.push("- No validation diagnostics.".into());
    } else {
        for diagnostic in &validation.diagnostics {
            lines.push(format!(
                "- [{:#?}/{:#?}] {}: {}",
                diagnostic.severity, diagnostic.category, diagnostic.code, diagnostic.message
            ));
            for action in &diagnostic.suggested_actions {
                lines.push(format!("  - Suggestion: {}", action));
            }
        }
    }

    lines.push(String::new());
    lines.push("## Engineering outputs".into());
    lines.push(String::new());
    if let Some(actions) = design_actions {
        lines.push(format!(
            "- Member action summaries: {}",
            actions.member_actions.len()
        ));
        lines.push(format!(
            "- Support reaction summaries: {}",
            actions.support_reactions.len()
        ));
        if let Some(serviceability) = &actions.global_serviceability {
            if let Some(drift_ratio) = serviceability.drift_ratio {
                lines.push(format!("- Governing drift ratio: H/{:.0}", drift_ratio));
            }
            if let Some(deflection_ratio) = serviceability.deflection_ratio {
                lines.push(format!(
                    "- Governing deflection ratio: L/{:.0}",
                    deflection_ratio
                ));
            }
        }
    } else {
        lines.push("- Engineering outputs unavailable because realization was unavailable.".into());
    }
    if let Some(checks) = checks {
        let fail_count = checks
            .results
            .iter()
            .filter(|result| matches!(result.severity, crate::CheckSeverity::Fail))
            .count();
        let warning_count = checks
            .results
            .iter()
            .filter(|result| matches!(result.severity, crate::CheckSeverity::Warning))
            .count();
        lines.push(format!("- Check fails: {}", fail_count));
        lines.push(format!("- Check warnings: {}", warning_count));
    }

    lines.push(String::new());
    lines.push("## Realization diagnostics".into());
    lines.push(String::new());
    if let Some(realization) = realization {
        if realization.diagnostics.is_empty() {
            lines.push("- No realization diagnostics.".into());
        } else {
            for diagnostic in &realization.diagnostics {
                lines.push(format!(
                    "- [{:#?}] {}: {}",
                    diagnostic.severity, diagnostic.code, diagnostic.message
                ));
            }
        }
    } else {
        lines.push("- Realization failed or was unavailable.".into());
    }

    lines.join("\n")
}

pub fn render_beam_calculix_summary(
    project: &ProjectFile,
    analysis: &SimplySupportedBeamCalculixArtifacts,
) -> String {
    let profile = &project.unit_profile;
    let lines = vec![
        format!("# Fraia beam CalculiX compile: {}", project.name),
        String::new(),
        format!("- Section: {}", analysis.run.section_id),
        format!("- Combo: {}", analysis.run.combo_id),
        format!("- Adapter: {}", analysis.run.adapter),
        format!(
            "- CalculiX runtime available: {}",
            if analysis.run.runtime_available { "yes" } else { "no" }
        ),
        format!(
            "- Compiled nodes/elements: {} / {}",
            analysis.compiled_input.node_count, analysis.compiled_input.element_count
        ),
        String::new(),
        "## Baseline comparison retained".into(),
        String::new(),
        render_beam_value_comparison(
            "Max deflection",
            "mm",
            &analysis.baseline.comparison.max_deflection_mm,
            profile,
        ),
        render_beam_value_comparison(
            "Max moment",
            "kN·m",
            &analysis.baseline.comparison.max_moment_knm,
            profile,
        ),
        render_beam_value_comparison(
            "Max shear",
            "kN",
            &analysis.baseline.comparison.max_shear_kn,
            profile,
        ),
        String::new(),
        "## Notes".into(),
        String::new(),
        "- This run compiles the current beam realization into a CalculiX input deck while retaining the exact/internal Fraia baseline artifacts.".into(),
        if analysis.run.runtime_available {
            "- A local `ccx` runtime is available; execution/extraction can be added on top of this seam.".into()
        } else {
            "- Local `ccx` runtime is not available, so this slice stops at compiled solver input artifacts and does not pretend the external backend executed.".into()
        },
    ];
    lines.join("\n")
}

pub fn render_beam_calculix_execution_summary(
    project: &ProjectFile,
    analysis: &SimplySupportedBeamCalculixExecutionArtifacts,
) -> String {
    let profile = &project.unit_profile;
    let mut lines = vec![
        format!("# Fraia beam CalculiX run: {}", project.name),
        String::new(),
        format!("- Section: {}", analysis.run.section_id),
        format!("- Combo: {}", analysis.run.combo_id),
        format!("- Adapter: {}", analysis.run.adapter),
        format!(
            "- CalculiX runtime available: {}",
            if analysis.run.runtime_available {
                "yes"
            } else {
                "no"
            }
        ),
        format!("- Working dir: {}", analysis.execution.working_dir),
        format!(
            "- Execution outcome: {}",
            match analysis.execution.outcome {
                CalculixExecutionOutcome::SkippedRuntimeUnavailable =>
                    "skipped-runtime-unavailable",
                CalculixExecutionOutcome::Completed => "completed",
                CalculixExecutionOutcome::Failed => "failed",
            }
        ),
    ];
    if let Some(exit_code) = analysis.execution.exit_code {
        lines.push(format!("- Exit code: {}", exit_code));
    }
    lines.push(format!(
        "- Produced files: {}",
        if analysis.execution.produced_files.is_empty() {
            "none".into()
        } else {
            analysis.execution.produced_files.join(", ")
        }
    ));
    if let Some(extracted) = &analysis.extracted_response {
        lines.push(String::new());
        lines.push("## CalculiX extracted response".into());
        lines.push(String::new());
        lines.push(format!(
            "- Left reaction: {}",
            format_quantity_from_unit(
                extracted.left_reaction_kn,
                QuantityKind::Force,
                "kN",
                profile
            )
        ));
        lines.push(format!(
            "- Right reaction: {}",
            format_quantity_from_unit(
                extracted.right_reaction_kn,
                QuantityKind::Force,
                "kN",
                profile
            )
        ));
        lines.push(format!(
            "- Max deflection: {} at x = {}",
            format_quantity_from_unit(
                extracted.max_deflection_mm,
                QuantityKind::Displacement,
                "mm",
                profile
            ),
            format_quantity_from_unit(
                extracted.max_deflection_x_m,
                QuantityKind::Length,
                "m",
                profile
            )
        ));
        lines.push(format!(
            "- Max |SXX|: {} near x = {} (element {})",
            format_quantity_from_unit(
                extracted.max_abs_sxx_mpa,
                QuantityKind::Stress,
                "MPa",
                profile
            ),
            format_quantity_from_unit(
                extracted.max_abs_sxx_x_m,
                QuantityKind::Length,
                "m",
                profile
            ),
            extracted.max_abs_sxx_element_id
        ));
    }
    lines.push(String::new());
    lines.push("## Retained verification context".into());
    lines.push(String::new());
    lines.push(render_beam_value_comparison(
        "Max deflection",
        "mm",
        &analysis.baseline.comparison.max_deflection_mm,
        profile,
    ));
    lines.push(render_beam_value_comparison(
        "Max moment",
        "kN·m",
        &analysis.baseline.comparison.max_moment_knm,
        profile,
    ));
    lines.push(render_beam_value_comparison(
        "Max shear",
        "kN",
        &analysis.baseline.comparison.max_shear_kn,
        profile,
    ));
    if let Some(comparison) = &analysis.extracted_comparison {
        lines.push(String::new());
        lines.push("## Exact/internal vs CalculiX comparison".into());
        lines.push(String::new());
        lines.push(render_exact_vs_external_comparison(
            "Left reaction",
            "kN",
            &comparison.left_reaction_kn,
            "calculix",
            profile,
        ));
        lines.push(render_exact_vs_external_comparison(
            "Right reaction",
            "kN",
            &comparison.right_reaction_kn,
            "calculix",
            profile,
        ));
        lines.push(render_exact_vs_external_comparison(
            "Max deflection",
            "mm",
            &comparison.max_deflection_mm,
            "calculix",
            profile,
        ));
        lines.push(render_exact_vs_external_comparison(
            "Max deflection location",
            "m",
            &comparison.max_deflection_x_m,
            "calculix",
            profile,
        ));
        lines.push(render_exact_vs_external_comparison(
            "Max |SXX|",
            "MPa",
            &comparison.max_abs_sxx_mpa,
            "calculix",
            profile,
        ));
    }
    if let Some(profile_comparison) = &analysis.extracted_deflection_profile_comparison {
        lines.push(String::new());
        lines.push("## Exact beam reference vs CalculiX deflection profile".into());
        lines.push(String::new());
        lines.push(format!(
            "- Compared points: {}",
            profile_comparison.point_count
        ));
        lines.push(format!(
            "- Max pointwise deflection delta: {} at x = {}",
            format_quantity_from_unit(
                profile_comparison.max_abs_diff_mm,
                QuantityKind::Displacement,
                "mm",
                profile
            ),
            format_quantity_from_unit(
                profile_comparison.max_abs_diff_x_m,
                QuantityKind::Length,
                "m",
                profile
            )
        ));
    }
    lines.push(String::new());
    lines.push("## Notes".into());
    lines.push(String::new());
    lines.push(
        "- CalculiX is the runtime analysis engine for this run; Fraia retains exact/internal comparisons only as explicit verification context under `verification/`.".into(),
    );
    lines.join("\n")
}

pub fn render_frame_calculix_execution_summary(
    project: &ProjectFile,
    analysis: &CurrentFrameCalculixExecutionArtifacts,
) -> String {
    let profile = &project.unit_profile;
    let mut lines = vec![
        format!("# Fraia frame CalculiX run: {}", project.name),
        String::new(),
        format!("- Combo: {}", analysis.run.combo_id),
        format!("- Adapter: {}", analysis.run.adapter),
        format!(
            "- CalculiX runtime available: {}",
            if analysis.run.runtime_available {
                "yes"
            } else {
                "no"
            }
        ),
        format!("- Working dir: {}", analysis.execution.working_dir),
        format!(
            "- Realized node count: {}",
            analysis.realization.model.nodes.len()
        ),
        format!(
            "- Realized support count: {}",
            analysis.realization.model.supports.len()
        ),
        format!(
            "- Execution outcome: {}",
            match analysis.execution.outcome {
                CalculixExecutionOutcome::SkippedRuntimeUnavailable =>
                    "skipped-runtime-unavailable",
                CalculixExecutionOutcome::Completed => "completed",
                CalculixExecutionOutcome::Failed => "failed",
            }
        ),
    ];
    if let Some(exit_code) = analysis.execution.exit_code {
        lines.push(format!("- Exit code: {}", exit_code));
    }
    lines.push(format!(
        "- Produced files: {}",
        if analysis.execution.produced_files.is_empty() {
            "none".into()
        } else {
            analysis.execution.produced_files.join(", ")
        }
    ));

    if let Some(points) = &analysis.extracted_node_displacements {
        let max_abs_ux = points
            .iter()
            .max_by(|a, b| a.ux_m.abs().total_cmp(&b.ux_m.abs()));
        let max_abs_uy = points
            .iter()
            .max_by(|a, b| a.uy_m.abs().total_cmp(&b.uy_m.abs()));
        lines.push(String::new());
        lines.push("## CalculiX extracted node displacement response".into());
        lines.push(String::new());
        lines.push(format!("- Extracted nodes: {}", points.len()));
        if let Some(point) = max_abs_ux {
            lines.push(format!(
                "- Max |UX|: {} at node {}",
                format_quantity_from_unit(
                    point.ux_m.abs(),
                    QuantityKind::Displacement,
                    "m",
                    profile
                ),
                point.node_id
            ));
        }
        if let Some(point) = max_abs_uy {
            lines.push(format!(
                "- Max |UY|: {} at node {}",
                format_quantity_from_unit(
                    point.uy_m.abs(),
                    QuantityKind::Displacement,
                    "m",
                    profile
                ),
                point.node_id
            ));
        }
    }
    if let Some(points) = &analysis.extracted_support_reactions {
        let max_abs_fx = points
            .iter()
            .max_by(|a, b| a.fx_n.abs().total_cmp(&b.fx_n.abs()));
        let max_abs_fy = points
            .iter()
            .max_by(|a, b| a.fy_n.abs().total_cmp(&b.fy_n.abs()));
        lines.push(String::new());
        lines.push("## CalculiX extracted support reaction response".into());
        lines.push(String::new());
        lines.push(format!("- Extracted supports: {}", points.len()));
        if let Some(point) = max_abs_fx {
            lines.push(format!(
                "- Max |FX|: {} at node {}",
                format_quantity_from_unit(point.fx_n.abs(), QuantityKind::Force, "N", profile),
                point.node_id
            ));
        }
        if let Some(point) = max_abs_fy {
            lines.push(format!(
                "- Max |FY|: {} at node {}",
                format_quantity_from_unit(point.fy_n.abs(), QuantityKind::Force, "N", profile),
                point.node_id
            ));
        }
    }
    if let Some(points) = &analysis.extracted_element_stresses {
        let max_abs_sxx = points
            .iter()
            .max_by(|a, b| a.max_abs_sxx_pa.total_cmp(&b.max_abs_sxx_pa));
        let max_abs_sxy = points
            .iter()
            .max_by(|a, b| a.max_abs_sxy_pa.total_cmp(&b.max_abs_sxy_pa));
        lines.push(String::new());
        lines.push("## CalculiX extracted element stress response".into());
        lines.push(String::new());
        lines.push(format!("- Extracted elements: {}", points.len()));
        if let Some(point) = max_abs_sxx {
            lines.push(format!(
                "- Max |SXX|: {} in element {}",
                format_quantity_from_unit(
                    point.max_abs_sxx_pa,
                    QuantityKind::Stress,
                    "Pa",
                    profile
                ),
                point.element_id
            ));
        }
        if let Some(point) = max_abs_sxy {
            lines.push(format!(
                "- Max |SXY|: {} in element {}",
                format_quantity_from_unit(
                    point.max_abs_sxy_pa,
                    QuantityKind::Stress,
                    "Pa",
                    profile
                ),
                point.element_id
            ));
        }
    }

    if let Some(comparison) = &analysis.displacement_comparison {
        lines.push(String::new());
        lines.push(
            "## Retained verification: internal vs CalculiX node displacement comparison".into(),
        );
        lines.push(String::new());
        lines.push(format!("- Compared nodes: {}", comparison.node_count));
        lines.push(format!(
            "- Max |UX diff|: {} at node {}",
            format_quantity_from_unit(
                comparison.max_abs_ux_diff_m,
                QuantityKind::Displacement,
                "m",
                profile
            ),
            comparison.max_abs_ux_diff_node_id
        ));
        lines.push(format!(
            "- Max |UY diff|: {} at node {}",
            format_quantity_from_unit(
                comparison.max_abs_uy_diff_m,
                QuantityKind::Displacement,
                "m",
                profile
            ),
            comparison.max_abs_uy_diff_node_id
        ));
    }
    if let Some(comparison) = &analysis.support_reaction_comparison {
        lines.push(String::new());
        lines.push(
            "## Retained verification: internal vs CalculiX support reaction comparison".into(),
        );
        lines.push(String::new());
        lines.push(format!("- Compared supports: {}", comparison.support_count));
        lines.push(format!(
            "- Max |FX diff|: {} at node {}",
            format_quantity_from_unit(
                comparison.max_abs_fx_diff_n,
                QuantityKind::Force,
                "N",
                profile
            ),
            comparison.max_abs_fx_diff_node_id
        ));
        lines.push(format!(
            "- Max |FY diff|: {} at node {}",
            format_quantity_from_unit(
                comparison.max_abs_fy_diff_n,
                QuantityKind::Force,
                "N",
                profile
            ),
            comparison.max_abs_fy_diff_node_id
        ));
    }
    if let Some(comparison) = &analysis.element_stress_comparison {
        lines.push(String::new());
        lines.push(
            "## Retained verification: internal vs CalculiX element stress comparison".into(),
        );
        lines.push(String::new());
        lines.push(format!("- Compared elements: {}", comparison.element_count));
        lines.push(format!(
            "- Max |SXX diff|: {} in element {}",
            format_quantity_from_unit(
                comparison.max_abs_sxx_diff_pa,
                QuantityKind::Stress,
                "Pa",
                profile
            ),
            comparison.max_abs_sxx_diff_element_id
        ));
    }

    lines.push(String::new());
    lines.push("## Notes".into());
    lines.push(String::new());
    lines.push(
        "- CalculiX is the runtime analysis engine for this run; Fraia retains internal comparisons only as explicit verification context under `verification/`.".into(),
    );
    lines.join("\n")
}

pub fn render_imported_stick_frame_summary(artifacts: &ImportedStickFrameArtifacts) -> String {
    let lines = vec![
        "# Fraia imported stick frame".into(),
        String::new(),
        format!("- Imported segments: {}", artifacts.imported_segment_count),
        format!("- Derived members: {}", artifacts.derived_member_count),
        format!("- Cleaned nodes: {}", artifacts.cleaned_node_count),
        format!("- Split intersections: {}", artifacts.split_intersection_count),
        format!("- Merge hits: {}", artifacts.merged_node_hit_count),
        format!("- Supports: {}", artifacts.structural_model.supports.len()),
        format!("- Uniform line loads: {}", artifacts.structural_model.loads.len()),
        String::new(),
        "## Notes".into(),
        String::new(),
        "- This is a minimal import/cleanup/semantic-prep path for dirty 2D stick geometry feeding the Fraia structural model.".into(),
        "- Current cleanup supports near-node merging, intersection/T-junction splitting, role inference, support snapping, and derived member load inheritance.".into(),
    ];
    lines.join("\n")
}

pub fn render_beam_analysis_summary(
    project: &ProjectFile,
    analysis: &SimplySupportedBeamAnalysisArtifacts,
) -> String {
    let profile = &project.unit_profile;
    let mut lines = vec![
        format!("# Fraia beam analysis: {}", project.name),
        String::new(),
        format!("- Section: {}", analysis.run.section_id),
        format!(
            "- Span: {}",
            format_quantity_from_unit(
                analysis.run.request.span_m,
                QuantityKind::Length,
                "m",
                profile
            )
        ),
        format!(
            "- Distributed load: {}",
            format_quantity_from_unit(
                analysis.run.request.distributed_load_kn_per_m,
                QuantityKind::LineLoad,
                "kN/m",
                profile
            )
        ),
    ];
    if let Some(point_load_kn) = analysis.run.request.point_load_kn {
        lines.push(format!(
            "- Point load: {}",
            format_quantity_from_unit(point_load_kn, QuantityKind::Force, "kN", profile)
        ));
        lines.push(format!(
            "- Point load position: {}",
            format_quantity_from_unit(
                analysis
                    .run
                    .request
                    .point_load_x_m
                    .unwrap_or(analysis.run.request.span_m * 0.5),
                QuantityKind::Length,
                "m",
                profile
            )
        ));
    }
    lines.push(format!(
        "- Internal solver: {}",
        analysis.run.internal_solver
    ));
    lines.push(format!("- Exact baseline: {}", analysis.run.exact_baseline));
    lines.push(format!("- Combo: {}", analysis.run.combo_id));
    lines.push(format!(
        "- Validation diagnostics: {}",
        analysis.validation.diagnostics.len()
    ));
    lines.push(format!(
        "- Realization diagnostics: {}",
        analysis.realization.diagnostics.len()
    ));

    lines.push(String::new());
    lines.push("## Exact vs internal comparison".into());
    lines.push(String::new());
    lines.push(render_beam_value_comparison(
        "Left reaction",
        "kN",
        &analysis.comparison.left_reaction_kn,
        profile,
    ));
    lines.push(render_beam_value_comparison(
        "Right reaction",
        "kN",
        &analysis.comparison.right_reaction_kn,
        profile,
    ));
    lines.push(render_beam_value_comparison(
        "Max shear",
        "kN",
        &analysis.comparison.max_shear_kn,
        profile,
    ));
    lines.push(render_beam_value_comparison(
        "Max moment",
        "kN·m",
        &analysis.comparison.max_moment_knm,
        profile,
    ));
    lines.push(render_beam_value_comparison(
        "Max moment location",
        "m",
        &analysis.comparison.max_moment_x_m,
        profile,
    ));
    lines.push(render_beam_value_comparison(
        "Max deflection",
        "mm",
        &analysis.comparison.max_deflection_mm,
        profile,
    ));
    lines.push(render_beam_value_comparison(
        "Max deflection location",
        "m",
        &analysis.comparison.max_deflection_x_m,
        profile,
    ));
    lines.push(render_beam_value_comparison(
        "Max bending stress",
        "MPa",
        &analysis.comparison.max_bending_stress_mpa,
        profile,
    ));
    lines.push(render_beam_value_comparison(
        "Max utilization",
        "",
        &analysis.comparison.max_utilization,
        profile,
    ));

    lines.push(String::new());
    lines.push("## Notes".into());
    lines.push(String::new());
    lines.push(
        "- This run preserves both the exact simply-supported beam baseline and the current internal frame2d solver response.".into(),
    );
    lines.push(
        "- It is intended to be the explicit beam-analysis seam that future external solver adapters can plug into.".into(),
    );

    lines.join("\n")
}

pub fn render_beam_sizing_summary(
    project: &ProjectFile,
    sizing: &SimplySupportedBeamSizingResult,
) -> String {
    let profile = &project.unit_profile;
    let mut lines = vec![
        format!("# Fraia beam sizing: {}", project.name),
        String::new(),
        format!(
            "- Span: {}",
            format_quantity_from_unit(sizing.request.span_m, QuantityKind::Length, "m", profile)
        ),
        format!(
            "- Distributed load: {}",
            format_quantity_from_unit(
                sizing.request.distributed_load_kn_per_m,
                QuantityKind::LineLoad,
                "kN/m",
                profile
            )
        ),
    ];
    if let Some(point_load_kn) = sizing.request.point_load_kn {
        lines.push(format!(
            "- Point load: {}",
            format_quantity_from_unit(point_load_kn, QuantityKind::Force, "kN", profile)
        ));
        lines.push(format!(
            "- Point load position: {}",
            format_quantity_from_unit(
                sizing
                    .request
                    .point_load_x_m
                    .unwrap_or(sizing.request.span_m * 0.5),
                QuantityKind::Length,
                "m",
                profile
            )
        ));
    }
    lines.push(format!(
        "- Target max utilization: {:.3}",
        sizing.request.target_max_utilization
    ));
    lines.push(format!(
        "- Target deflection ratio: L/{:.0}",
        sizing.request.target_deflection_ratio
    ));

    if let Some(chosen) = &sizing.chosen {
        lines.push(String::new());
        lines.push("## Chosen section".into());
        lines.push(String::new());
        lines.push(format!("- Section: {}", chosen.section_id));
        lines.push(format!("- Mass: {:.1} kg", chosen.mass_kg));
        lines.push(format!("- Max utilization: {:.3}", chosen.max_utilization));
        lines.push(format!(
            "- Max deflection: {} at x = {}",
            format_quantity_from_unit(
                chosen.max_deflection_mm,
                QuantityKind::Displacement,
                "mm",
                profile
            ),
            format_quantity_from_unit(
                chosen.max_deflection_x_m,
                QuantityKind::Length,
                "m",
                profile
            )
        ));
        if let Some(deflection_ratio) = chosen.deflection_ratio {
            lines.push(format!("- Deflection ratio: L/{:.0}", deflection_ratio));
        }
        lines.push(format!(
            "- Left reaction: {}",
            format_quantity_from_unit(chosen.left_reaction_kn, QuantityKind::Force, "kN", profile)
        ));
        lines.push(format!(
            "- Right reaction: {}",
            format_quantity_from_unit(chosen.right_reaction_kn, QuantityKind::Force, "kN", profile)
        ));
        lines.push(format!(
            "- Max shear: {}",
            format_quantity_from_unit(chosen.max_shear_kn, QuantityKind::Force, "kN", profile)
        ));
        lines.push(format!(
            "- Max moment: {} at x = {}",
            format_quantity_from_unit(chosen.max_moment_knm, QuantityKind::Moment, "kN·m", profile),
            format_quantity_from_unit(chosen.max_moment_x_m, QuantityKind::Length, "m", profile)
        ));
        lines.push(format!(
            "- Section modulus: {:.6} m^3",
            chosen.section_modulus_m3
        ));
        lines.push(format!(
            "- Max bending stress: {}",
            format_quantity_from_unit(
                chosen.max_bending_stress_mpa,
                QuantityKind::Stress,
                "MPa",
                profile
            )
        ));
    }

    lines.push(String::new());
    lines.push("## Candidate sections".into());
    lines.push(String::new());
    for candidate in &sizing.candidates {
        lines.push(format!(
            "- {} | {} | util {:.3} | defl {} | M {} | V {} | stress {} | {}",
            candidate.section_id,
            if candidate.feasible {
                "feasible"
            } else {
                "not feasible"
            },
            candidate.max_utilization,
            format_quantity_from_unit(
                candidate.max_deflection_mm,
                QuantityKind::Displacement,
                "mm",
                profile
            ),
            format_quantity_from_unit(
                candidate.max_moment_knm,
                QuantityKind::Moment,
                "kN·m",
                profile
            ),
            format_quantity_from_unit(candidate.max_shear_kn, QuantityKind::Force, "kN", profile),
            format_quantity_from_unit(
                candidate.max_bending_stress_mpa,
                QuantityKind::Stress,
                "MPa",
                profile
            ),
            candidate
                .deflection_ratio
                .map(|ratio| format!("L/{:.0}", ratio))
                .unwrap_or_else(|| "L/n/a".into()),
        ));
    }

    lines.join("\n")
}

fn render_exact_vs_external_comparison(
    label: &str,
    unit: &str,
    comparison: &BeamValueComparison,
    external_label: &str,
    profile: &UnitProfile,
) -> String {
    let exact = format_comparison_value(comparison.exact, unit, profile);
    let external = format_comparison_value(comparison.internal, unit, profile);
    let abs_diff = format_comparison_value(comparison.abs_diff, unit, profile);
    format!(
        "- {}: exact {} | {} {} | |Δ| {}",
        label, exact, external_label, external, abs_diff,
    )
}

fn render_beam_value_comparison(
    label: &str,
    unit: &str,
    comparison: &BeamValueComparison,
    profile: &UnitProfile,
) -> String {
    let exact = format_comparison_value(comparison.exact, unit, profile);
    let internal = format_comparison_value(comparison.internal, unit, profile);
    let abs_diff = format_comparison_value(comparison.abs_diff, unit, profile);
    format!(
        "- {}: exact {} | internal {} | |Δ| {}",
        label, exact, internal, abs_diff,
    )
}

fn format_comparison_value(value: f64, unit: &str, profile: &UnitProfile) -> String {
    if unit.is_empty() {
        format!("{value:.3}")
    } else {
        let kind = quantity_kind_for_unit(unit);
        format_quantity_from_unit(value, kind, unit, profile)
    }
}

fn quantity_kind_for_unit(unit: &str) -> QuantityKind {
    match unit {
        "m" => QuantityKind::Length,
        "mm" => QuantityKind::Displacement,
        "kN" | "N" => QuantityKind::Force,
        "kN/m" | "N/m" => QuantityKind::LineLoad,
        "kN·m" | "kN*m" | "N*m" => QuantityKind::Moment,
        "MPa" | "Pa" => QuantityKind::Stress,
        _ => QuantityKind::Length,
    }
}

pub fn render_member_actions_csv(design_actions: Option<&DesignActionReport>) -> String {
    let mut lines = vec![
        "member_id,role,section_id,length_m,max_axial_n,axial_combo,max_shear_n,shear_combo,max_moment_nm,moment_combo,max_utilization,utilization_combo".into(),
    ];
    if let Some(actions) = design_actions {
        for action in &actions.member_actions {
            lines.push(format!(
                "{},{},{},{:.6},{:.6},{},{:.6},{},{:.6},{},{:.6},{}",
                action.member_id,
                action.role,
                action.section_id,
                action.length_m,
                action.max_axial_n,
                action.governing_axial_combo_id,
                action.max_shear_n,
                action.governing_shear_combo_id,
                action.max_moment_nm,
                action.governing_moment_combo_id,
                action.max_utilization,
                action.governing_utilization_combo_id,
            ));
        }
    }
    lines.join("\n")
}

pub fn render_support_reactions_csv(design_actions: Option<&DesignActionReport>) -> String {
    let mut lines =
        vec!["support_node_id,max_fx_n,fx_combo,max_fy_n,fy_combo,max_mz_nm,mz_combo".into()];
    if let Some(actions) = design_actions {
        for reaction in &actions.support_reactions {
            lines.push(format!(
                "{},{:.6},{},{:.6},{},{:.6},{}",
                reaction.support_node_id,
                reaction.max_fx_n,
                reaction.governing_fx_combo_id,
                reaction.max_fy_n,
                reaction.governing_fy_combo_id,
                reaction.max_mz_nm,
                reaction.governing_mz_combo_id,
            ));
        }
    }
    lines.join("\n")
}

pub fn render_check_results_csv(results: &[CheckResult]) -> String {
    let mut lines = vec![
        "check_id,check_type,severity,unity_ratio,actual_value,limit_value,governing_combo_id,message".into(),
    ];
    for result in results {
        let message = result.message.replace('"', "'");
        lines.push(format!(
            "{},{},{:?},{:.6},{:.6},{:.6},\"{}\",\"{}\"",
            result.id,
            result.check_type,
            result.severity,
            result.unity_ratio,
            result.actual_value,
            result.limit_value,
            result.governing_combo_id.clone().unwrap_or_default(),
            message,
        ));
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::{render_beam_sizing_summary, render_check_results_csv};
    use crate::{
        CheckResult, CheckSeverity, CheckSubject, SimplySupportedBeamSizingCandidate,
        SimplySupportedBeamSizingRequest, SimplySupportedBeamSizingResult, create_project,
    };

    #[test]
    fn beam_sizing_summary_includes_chosen_section() {
        let temp_dir = std::env::temp_dir().join(format!(
            "fraia-core-output-summary-{}",
            crate::utils::timestamp_id()
        ));
        let (mut project, _) = create_project(&temp_dir, "beam-output").expect("create project");
        project.name = "Beam Output".into();
        let sizing = SimplySupportedBeamSizingResult {
            request: SimplySupportedBeamSizingRequest {
                span_m: 6.0,
                distributed_load_kn_per_m: 8.0,
                point_load_kn: Some(20.0),
                point_load_x_m: Some(3.0),
                target_max_utilization: 0.67,
                target_deflection_ratio: 250.0,
            },
            candidates: vec![],
            chosen: Some(SimplySupportedBeamSizingCandidate {
                section_id: "250UB".into(),
                mass_kg: 188.4,
                max_utilization: 0.55,
                max_deflection_mm: 22.392,
                deflection_ratio: Some(268.0),
                left_reaction_kn: 34.0,
                right_reaction_kn: 34.0,
                max_shear_kn: 34.0,
                max_moment_knm: 66.0,
                max_moment_x_m: 3.0,
                max_deflection_x_m: 3.0,
                section_modulus_m3: 4.0e-4,
                max_bending_stress_mpa: 165.0,
                feasible: true,
            }),
        };

        let summary = render_beam_sizing_summary(&project, &sizing);
        assert!(summary.contains("# Fraia beam sizing: Beam Output"));
        assert!(summary.contains("- Section: 250UB"));

        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn check_results_csv_quotes_messages() {
        let csv = render_check_results_csv(&[CheckResult {
            id: "check.1".into(),
            subject: CheckSubject::GlobalDeflection,
            check_type: "deflection".into(),
            severity: CheckSeverity::Warning,
            unity_ratio: 1.2,
            actual_value: 12.0,
            limit_value: 10.0,
            governing_combo_id: Some("SLS".into()),
            message: "quoted \"message\"".into(),
            assumptions: vec![],
        }]);

        assert!(csv.contains("quoted 'message'"));
    }
}
