use crate::design_actions::{DesignActionReport, GlobalServiceabilitySummary, MemberDesignActions};
use crate::types::ProjectFile;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckSeverity {
    Pass,
    Warning,
    Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CheckSubject {
    Member { member_id: String },
    GlobalDrift,
    GlobalDeflection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberStrengthCheckInput {
    pub member_id: String,
    pub section_id: String,
    pub role: String,
    pub max_utilization: f64,
    pub target_max_utilization: f64,
    pub governing_combo_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalDriftCheckInput {
    pub reference_height_m: f64,
    pub actual_ratio: f64,
    pub target_ratio: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governing_combo_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalDeflectionCheckInput {
    pub reference_span_m: f64,
    pub actual_ratio: f64,
    pub target_ratio: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governing_combo_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum CheckInput {
    MemberStrength(MemberStrengthCheckInput),
    GlobalDrift(GlobalDriftCheckInput),
    GlobalDeflection(GlobalDeflectionCheckInput),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub id: String,
    pub subject: CheckSubject,
    pub check_type: String,
    pub severity: CheckSeverity,
    pub unity_ratio: f64,
    pub actual_value: f64,
    pub limit_value: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governing_combo_id: Option<String>,
    pub message: String,
    pub assumptions: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CheckReport {
    pub inputs: Vec<CheckInput>,
    pub results: Vec<CheckResult>,
}

pub fn derive_conservative_check_report(
    project: &ProjectFile,
    actions: &DesignActionReport,
) -> CheckReport {
    let mut report = CheckReport::default();

    for action in &actions.member_actions {
        let input = member_strength_input(project, action);
        report
            .inputs
            .push(CheckInput::MemberStrength(input.clone()));
        let unity_ratio = if input.target_max_utilization > 0.0 {
            input.max_utilization / input.target_max_utilization
        } else {
            f64::INFINITY
        };
        report.results.push(CheckResult {
            id: format!("member-strength-{}", input.member_id),
            subject: CheckSubject::Member {
                member_id: input.member_id.clone(),
            },
            check_type: "member_strength_utilization".into(),
            severity: severity_from_unity_ratio(unity_ratio),
            unity_ratio,
            actual_value: input.max_utilization,
            limit_value: input.target_max_utilization,
            governing_combo_id: Some(input.governing_combo_id.clone()),
            message: format!(
                "Member {} utilization {:.3} versus target limit {:.3}.",
                input.member_id, input.max_utilization, input.target_max_utilization
            ),
            assumptions: vec![
                "Uses current solver-side utilization as a conservative early strength indicator."
                    .into(),
            ],
        });
    }

    if let Some(serviceability) = &actions.global_serviceability {
        let drift_input = global_drift_input(project, serviceability);
        report
            .inputs
            .push(CheckInput::GlobalDrift(drift_input.clone()));
        let drift_unity_ratio = if drift_input.actual_ratio > 0.0 {
            drift_input.target_ratio / drift_input.actual_ratio
        } else {
            0.0
        };
        report.results.push(CheckResult {
            id: "global-drift".into(),
            subject: CheckSubject::GlobalDrift,
            check_type: "global_drift_ratio".into(),
            severity: severity_from_unity_ratio(drift_unity_ratio),
            unity_ratio: drift_unity_ratio,
            actual_value: drift_input.actual_ratio,
            limit_value: drift_input.target_ratio,
            governing_combo_id: drift_input.governing_combo_id.clone(),
            message: format!(
                "Global drift ratio H/{:.0} versus target H/{:.0}.",
                drift_input.actual_ratio, drift_input.target_ratio
            ),
            assumptions: vec![
                "Uses current frame-model lateral displacement envelope at the top node set."
                    .into(),
            ],
        });

        let deflection_input = global_deflection_input(project, serviceability);
        report
            .inputs
            .push(CheckInput::GlobalDeflection(deflection_input.clone()));
        let deflection_unity_ratio = if deflection_input.actual_ratio > 0.0 {
            deflection_input.target_ratio / deflection_input.actual_ratio
        } else {
            0.0
        };
        report.results.push(CheckResult {
            id: "global-deflection".into(),
            subject: CheckSubject::GlobalDeflection,
            check_type: "global_deflection_ratio".into(),
            severity: severity_from_unity_ratio(deflection_unity_ratio),
            unity_ratio: deflection_unity_ratio,
            actual_value: deflection_input.actual_ratio,
            limit_value: deflection_input.target_ratio,
            governing_combo_id: deflection_input.governing_combo_id.clone(),
            message: format!(
                "Global beam deflection ratio L/{:.0} versus target L/{:.0}.",
                deflection_input.actual_ratio, deflection_input.target_ratio
            ),
            assumptions: vec![
                "Uses current beam-node vertical displacement envelope with project span as the reference length."
                    .into(),
            ],
        });
    }

    report
}

fn member_strength_input(
    project: &ProjectFile,
    action: &MemberDesignActions,
) -> MemberStrengthCheckInput {
    MemberStrengthCheckInput {
        member_id: action.member_id.clone(),
        section_id: action.section_id.clone(),
        role: action.role.clone(),
        max_utilization: action.max_utilization,
        target_max_utilization: project.requirements.max_utilization,
        governing_combo_id: action.governing_utilization_combo_id.clone(),
    }
}

fn global_drift_input(
    project: &ProjectFile,
    serviceability: &GlobalServiceabilitySummary,
) -> GlobalDriftCheckInput {
    GlobalDriftCheckInput {
        reference_height_m: serviceability.reference_height_m,
        actual_ratio: serviceability.drift_ratio.unwrap_or(f64::INFINITY),
        target_ratio: project.requirements.max_drift_ratio,
        governing_combo_id: serviceability.governing_drift_combo_id.clone(),
    }
}

fn global_deflection_input(
    project: &ProjectFile,
    serviceability: &GlobalServiceabilitySummary,
) -> GlobalDeflectionCheckInput {
    GlobalDeflectionCheckInput {
        reference_span_m: serviceability.reference_span_m,
        actual_ratio: serviceability.deflection_ratio.unwrap_or(f64::INFINITY),
        target_ratio: project.requirements.max_deflection_ratio,
        governing_combo_id: serviceability.governing_deflection_combo_id.clone(),
    }
}

fn severity_from_unity_ratio(unity_ratio: f64) -> CheckSeverity {
    if unity_ratio > 1.0 {
        CheckSeverity::Fail
    } else if unity_ratio > 0.9 {
        CheckSeverity::Warning
    } else {
        CheckSeverity::Pass
    }
}

#[cfg(test)]
mod tests {
    use super::{CheckSeverity, derive_conservative_check_report};
    use crate::archetypes::build_frame_model;
    use crate::catalog::section_by_id;
    use crate::design_actions::derive_design_action_report;
    use crate::project::create_project;
    use crate::utils::timestamp_id;
    use std::fs;

    #[test]
    fn conservative_check_report_produces_inputs_and_results() {
        let temp_dir = std::env::temp_dir().join(format!("fraia-checks-test-{}", timestamp_id()));
        let (project, _) = create_project(&temp_dir, "checks-test").expect("create project");
        let beam = section_by_id("310UB").unwrap();
        let column = section_by_id("310UB").unwrap();
        let model = build_frame_model("clear_span", 20.0, 6.0, &beam, &column, 20.0, 80.0);
        let actions = derive_design_action_report(&project, &model).expect("design actions");
        let report = derive_conservative_check_report(&project, &actions);

        assert!(!report.inputs.is_empty());
        assert!(!report.results.is_empty());
        assert!(report.results.iter().any(|result| matches!(
            result.severity,
            CheckSeverity::Pass | CheckSeverity::Warning | CheckSeverity::Fail
        )));

        let _ = fs::remove_dir_all(temp_dir);
    }
}
