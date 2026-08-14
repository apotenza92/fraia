use anyhow::{Context, Result};
use fraia_core::{
    DesignId, DesignRunList, ImportedStickFrameInput, InspectedDesignRun, OptimizationRun,
    ProjectFile, analyze_current_simply_supported_beam_project,
    compile_current_simply_supported_beam_project_to_calculix, create_project,
    current_simply_supported_beam_builder_params, default_planning_markdown,
    derive_conservative_check_report, derive_design_action_report,
    execute_current_frame_project_in_calculix,
    execute_current_simply_supported_beam_project_in_calculix,
    import_stick_frame_to_structural_model, inspect_design_run, list_design_runs, load_project,
    load_project_package, materialize_project_structural_model,
    materialize_structural_model_from_builder_graph, portal_frame_builder_graph,
    realize_structural_model_to_frame2d, require_calculix_runtime, run_optimization, save_project,
    seed_simply_supported_beam_in_project, size_current_simply_supported_beam_in_project,
    understand_structural_model, update_planning_markdown, validate_structural_model,
};
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use fraia_revision::operations::{
    DesignRunOperationContext, OPERATION_CONTRACT_VERSION, Operation, OperationOutcome,
    OperationRequest, execute_sqlite_operation, execute_sqlite_operation_with_design_runs,
};
use fraia_revision::sqlite::SqliteRevisionRepository;
use serde::Serialize;

const EXIT_SUCCESS: i32 = 0;
const EXIT_OPERATION_ERROR: i32 = 2;
const EXIT_HEAD_CONFLICT: i32 = 3;
const EXIT_RUNTIME_UNAVAILABLE: i32 = 4;
const EXIT_USAGE: i32 = 64;
const EXIT_INPUT: i32 = 65;
const EXIT_REPOSITORY: i32 = 70;

fn main() -> Result<()> {
    let raw_args = env::args().skip(1).collect::<Vec<_>>();
    if matches!(
        raw_args.first().map(String::as_str),
        Some("operation" | "operation-capabilities" | "operation-schema")
    ) {
        let exit = run_machine_command(&raw_args, &mut io::stdin(), &mut io::stdout());
        std::process::exit(exit);
    }
    let mut args = raw_args.into_iter();
    match args.next().as_deref() {
        Some("init") => cmd_init(args.next()),
        Some("plan") => cmd_plan(args.next()),
        Some("optimize") => cmd_optimize(args.next()),
        Some("validate") => cmd_validate(args.next()),
        Some("inspect-model") => cmd_inspect_model(args.next(), args.next()),
        Some("adopt") => cmd_adopt(args.next(), args.next()),
        Some("demo") => cmd_demo(args.next()),
        Some("frame-demo") => cmd_frame_demo(args.next()),
        Some("import-stick-frame") => cmd_import_stick_frame(args.next(), args.next()),
        Some("frame-run-calculix") => cmd_frame_run_calculix(args.next()),
        Some("beam-demo") => cmd_beam_demo(args.next()),
        Some("beam-init") => cmd_beam_init(
            args.next(),
            args.next(),
            args.next(),
            args.next(),
            args.next(),
        ),
        Some("beam-size") => cmd_beam_size(args.next()),
        Some("beam-analyze") => cmd_beam_analyze(args.next()),
        Some("beam-compile-calculix") => cmd_beam_compile_calculix(args.next()),
        Some("beam-run-calculix") => cmd_beam_run_calculix(args.next()),
        Some("design-runs-list") => cmd_design_runs_list(args.next(), args.next()),
        Some("design-run-inspect") => cmd_design_run_inspect(args.next(), args.next(), args.next()),
        Some("design-runs-status") => {
            cmd_design_runs_status(args.next(), args.next(), args.next(), args.next())
        }
        _ => {
            print_help();
            Ok(())
        }
    }
}

fn cmd_design_runs_list(project_dir: Option<String>, design_id: Option<String>) -> Result<()> {
    let project_dir =
        project_dir.context("usage: fraia design-runs-list <project-directory> <design-id>")?;
    let design_id =
        design_id.context("usage: fraia design-runs-list <project-directory> <design-id>")?;
    let runs = cli_design_runs_list(&PathBuf::from(project_dir), &DesignId::new(design_id))?;
    println!("{}", serde_json::to_string_pretty(&runs)?);
    Ok(())
}

fn cmd_design_run_inspect(
    project_dir: Option<String>,
    design_id: Option<String>,
    run_id: Option<String>,
) -> Result<()> {
    let project_dir = project_dir
        .context("usage: fraia design-run-inspect <project-directory> <design-id> <run-id>")?;
    let design_id = design_id
        .context("usage: fraia design-run-inspect <project-directory> <design-id> <run-id>")?;
    let run_id = run_id
        .context("usage: fraia design-run-inspect <project-directory> <design-id> <run-id>")?;
    let run = cli_design_run_inspect(
        &PathBuf::from(project_dir),
        &DesignId::new(design_id),
        &run_id,
    )?;
    println!("{}", serde_json::to_string_pretty(&run)?);
    Ok(())
}

fn cli_design_runs_list(
    project_dir: &std::path::Path,
    design_id: &DesignId,
) -> Result<DesignRunList> {
    list_design_runs(project_dir, design_id).map_err(Into::into)
}

fn cmd_design_runs_status(
    project_dir: Option<String>,
    design_id: Option<String>,
    snapshot_id: Option<String>,
    ancestor_snapshot_ids: Option<String>,
) -> Result<()> {
    let usage = "usage: fraia design-runs-status <project-directory> <design-id> <snapshot-id> [ancestor-snapshot-ids-comma-separated]";
    let project_dir = project_dir.context(usage)?;
    let design_id = design_id.context(usage)?;
    let snapshot_id = snapshot_id.context(usage)?;
    let ancestors = ancestor_snapshot_ids
        .map(|ids| {
            ids.split(',')
                .filter(|id| !id.trim().is_empty())
                .map(|id| id.trim().to_owned())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let statuses = cli_design_run_statuses(
        &PathBuf::from(project_dir),
        &DesignId::new(design_id),
        &snapshot_id,
        &ancestors,
    )?;
    println!("{}", serde_json::to_string_pretty(&statuses)?);
    Ok(())
}

fn cli_design_run_statuses(
    project_dir: &std::path::Path,
    design_id: &DesignId,
    snapshot_id: &str,
    ancestor_snapshot_ids: &[String],
) -> Result<Vec<fraia_core::DesignRunStatusProjection>> {
    fraia_core::list_design_run_statuses(project_dir, design_id, snapshot_id, ancestor_snapshot_ids)
        .map_err(Into::into)
}

fn cli_design_run_inspect(
    project_dir: &std::path::Path,
    design_id: &DesignId,
    run_id: &str,
) -> Result<InspectedDesignRun> {
    inspect_design_run(project_dir, design_id, run_id).map_err(Into::into)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CliError<'a> {
    schema: &'static str,
    code: &'a str,
    message: String,
}

fn run_machine_command(args: &[String], input: &mut impl Read, output: &mut impl Write) -> i32 {
    match args.first().map(String::as_str) {
        Some("operation-capabilities") if args.len() == 1 => {
            let request = OperationRequest {
                contract_version: OPERATION_CONTRACT_VERSION.into(),
                request_id: "capabilities".into(),
                operation: Operation::Capabilities,
            };
            let mut repository = match SqliteRevisionRepository::open_in_memory() {
                Ok(repository) => repository,
                Err(error) => {
                    return write_cli_error(output, EXIT_REPOSITORY, "repository_error", error);
                }
            };
            write_operation_response(output, execute_sqlite_operation(&mut repository, request))
        }
        Some("operation-schema") if args.len() == 1 => {
            let schema = serde_json::json!({
                "schema": "fraia.operation-schema.v1",
                "contractVersion": OPERATION_CONTRACT_VERSION,
                "transport": {
                    "input": "one JSON OperationRequest on stdin",
                    "output": "one JSON OperationResponse on stdout"
                },
                "commands": {
                    "execute": "fraia operation --database <sqlite-path>",
                    "capabilities": "fraia operation-capabilities"
                },
                "compatibilityCommands": [
                    "init", "plan", "optimize", "validate", "inspect-model", "adopt",
                    "demo", "frame-demo", "import-stick-frame", "frame-run-calculix",
                    "beam-demo", "beam-init", "beam-size", "beam-analyze",
                    "beam-compile-calculix", "beam-run-calculix",
                    "design-runs-list", "design-run-inspect", "design-runs-status"
                ],
                "exitCodes": {
                    "success": EXIT_SUCCESS,
                    "operationError": EXIT_OPERATION_ERROR,
                    "headConflict": EXIT_HEAD_CONFLICT,
                    "snapshotConflict": EXIT_HEAD_CONFLICT,
                    "runtimeUnavailable": EXIT_RUNTIME_UNAVAILABLE,
                    "usage": EXIT_USAGE,
                    "invalidInput": EXIT_INPUT,
                    "repositoryError": EXIT_REPOSITORY
                }
            });
            write_json_line(output, &schema, EXIT_REPOSITORY)
        }
        Some("operation") => {
            if args.len() < 3 || args.get(1).map(String::as_str) != Some("--database") {
                return write_cli_error(
                    output,
                    EXIT_USAGE,
                    "usage",
                    "usage: fraia operation --database <sqlite-path> [--input <json-path>] [--batch]",
                );
            }
            let mut input_path = None;
            let mut batch = false;
            let mut index = 3;
            while index < args.len() {
                match args[index].as_str() {
                    "--input" if index + 1 < args.len() => {
                        input_path = Some(args[index + 1].as_str());
                        index += 2;
                    }
                    "--batch" => {
                        batch = true;
                        index += 1;
                    }
                    _ => {
                        return write_cli_error(
                            output,
                            EXIT_USAGE,
                            "usage",
                            "usage: fraia operation --database <sqlite-path> [--input <json-path>] [--batch]",
                        );
                    }
                }
            }
            let bytes = if let Some(path) = input_path {
                match fs::read(path) {
                    Ok(bytes) => bytes,
                    Err(error) => return write_cli_error(output, EXIT_INPUT, "input_error", error),
                }
            } else {
                let mut bytes = Vec::new();
                if let Err(error) = input.read_to_end(&mut bytes) {
                    return write_cli_error(output, EXIT_INPUT, "input_error", error);
                }
                bytes
            };
            let database_path = PathBuf::from(&args[2]);
            let mut repository = match SqliteRevisionRepository::open(&database_path) {
                Ok(repository) => repository,
                Err(error) => {
                    return write_cli_error(output, EXIT_REPOSITORY, "repository_error", error);
                }
            };
            if batch {
                return run_operation_batch(
                    &mut repository,
                    &bytes,
                    output,
                    operation_design_run_context(&database_path).as_ref(),
                );
            }
            let request = match serde_json::from_slice::<OperationRequest>(&bytes) {
                Ok(request) => request,
                Err(error) => {
                    return write_cli_error(output, EXIT_INPUT, "invalid_json", error);
                }
            };
            write_operation_response(
                output,
                execute_cli_operation(
                    &mut repository,
                    request,
                    operation_design_run_context(&database_path).as_ref(),
                ),
            )
        }
        _ => write_cli_error(
            output,
            EXIT_USAGE,
            "usage",
            "usage: fraia operation --database <sqlite-path> [--input <json-path>] [--batch] | operation-capabilities | operation-schema",
        ),
    }
}

fn run_operation_batch(
    repository: &mut SqliteRevisionRepository,
    bytes: &[u8],
    output: &mut impl Write,
    run_context: Option<&DesignRunOperationContext>,
) -> i32 {
    let input = match std::str::from_utf8(bytes) {
        Ok(input) => input,
        Err(error) => return write_cli_error(output, EXIT_INPUT, "invalid_utf8", error),
    };
    let mut exit = EXIT_SUCCESS;
    let mut saw_request = false;
    for (line_index, line) in input.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        saw_request = true;
        let request = match serde_json::from_str::<OperationRequest>(line) {
            Ok(request) => request,
            Err(error) => {
                let code = write_cli_error(
                    output,
                    EXIT_INPUT,
                    "invalid_json",
                    format!("line {}: {error}", line_index + 1),
                );
                exit = exit.max(code);
                continue;
            }
        };
        let code = write_operation_response(
            output,
            execute_cli_operation(repository, request, run_context),
        );
        exit = exit.max(code);
    }
    if !saw_request {
        return write_cli_error(
            output,
            EXIT_INPUT,
            "empty_batch",
            "batch input contains no operation requests",
        );
    }
    exit
}

fn execute_cli_operation(
    repository: &mut SqliteRevisionRepository,
    request: OperationRequest,
    run_context: Option<&DesignRunOperationContext>,
) -> fraia_revision::operations::OperationResponse {
    match run_context {
        Some(context) => execute_sqlite_operation_with_design_runs(repository, request, context),
        None => execute_sqlite_operation(repository, request),
    }
}

fn operation_design_run_context(
    database_path: &std::path::Path,
) -> Option<DesignRunOperationContext> {
    if database_path.file_name()?.to_str()? != "workspace.sqlite" {
        return None;
    }
    let design_dir = database_path.parent()?;
    let designs_dir = design_dir.parent()?;
    if designs_dir.file_name()?.to_str()? != "designs" {
        return None;
    }
    let project_dir = designs_dir.parent()?;
    let package = load_project_package(project_dir).ok()?;
    let design_id = DesignId::new(design_dir.file_name()?.to_str()?);
    if !package
        .manifest
        .designs
        .iter()
        .any(|entry| entry.id == design_id)
    {
        return None;
    }
    Some(DesignRunOperationContext::new(
        project_dir,
        package.manifest.id,
        design_id,
        fraia_core::DesignRunActor {
            actor_type: "cli".into(),
            actor_id: "fraia.operations.v1".into(),
        },
        fraia_core::utils::timestamp_id(),
    ))
}

fn operation_exit(response: &fraia_revision::operations::OperationResponse) -> i32 {
    match &response.outcome {
        OperationOutcome::Success { result }
            if matches!(
                result.as_ref(),
                fraia_revision::operations::OperationResult::SnapshotAnalysed { run }
                    if matches!(run.outcome, fraia_revision::analysis_service::SnapshotAnalysisOutcome::Unsupported { .. })
            ) =>
        {
            EXIT_RUNTIME_UNAVAILABLE
        }
        OperationOutcome::Success { .. } => EXIT_SUCCESS,
        OperationOutcome::Error { error }
            if matches!(
                error.code,
                fraia_revision::operations::OperationErrorCode::ExpectedHeadMismatch
                    | fraia_revision::operations::OperationErrorCode::ExpectedSnapshotMismatch
            ) =>
        {
            EXIT_HEAD_CONFLICT
        }
        OperationOutcome::Error { error }
            if error.code == fraia_revision::operations::OperationErrorCode::RepositoryError =>
        {
            EXIT_REPOSITORY
        }
        OperationOutcome::Error { .. } => EXIT_OPERATION_ERROR,
    }
}

fn write_operation_response(
    output: &mut impl Write,
    response: fraia_revision::operations::OperationResponse,
) -> i32 {
    let exit = operation_exit(&response);
    if write_json_line(output, &response, EXIT_REPOSITORY) == EXIT_SUCCESS {
        exit
    } else {
        EXIT_REPOSITORY
    }
}

fn write_json_line(output: &mut impl Write, value: &impl Serialize, failure_exit: i32) -> i32 {
    if serde_json::to_writer(&mut *output, value).is_ok() && output.write_all(b"\n").is_ok() {
        EXIT_SUCCESS
    } else {
        failure_exit
    }
}

fn write_cli_error(
    output: &mut impl Write,
    exit: i32,
    code: &str,
    message: impl std::fmt::Display,
) -> i32 {
    let error = CliError {
        schema: "fraia.cli.error.v1",
        code,
        message: message.to_string(),
    };
    if write_json_line(output, &error, EXIT_REPOSITORY) == EXIT_SUCCESS {
        exit
    } else {
        EXIT_REPOSITORY
    }
}

fn cmd_init(project_dir: Option<String>) -> Result<()> {
    let dir = PathBuf::from(project_dir.unwrap_or_else(|| "fraia-project".into()));
    let name = dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("Fraia Project");
    let (_project, paths) = create_project(&dir, name)?;
    println!("Created Fraia project at {}", paths.project_dir.display());
    println!("- {}", paths.project_file.display());
    println!("- {}", paths.planning_file.display());
    Ok(())
}

fn cmd_plan(project_dir: Option<String>) -> Result<()> {
    let dir = require_dir(project_dir, "Usage: fraia plan <projectDir>")?;
    let (mut project, _paths) = load_project(&dir)?;
    println!("Fraia planning mode\nPress Enter to keep current values.\n");
    project.name = ask("Project name", &project.name)?;
    project.intent.building_type = ask("What are you building?", &project.intent.building_type)?;
    project.intent.design_stage = ask("Design stage", &project.intent.design_stage)?;
    project.intent.objective_priority = ask(
        "Objective priority (balanced/minimize_cost/low_carbon/clear_span)",
        &project.intent.objective_priority,
    )?;
    project.requirements.span_m = ask_f64("Main span in metres", project.requirements.span_m)?;
    project.requirements.height_m = ask_f64("Height in metres", project.requirements.height_m)?;
    project.requirements.gravity_load_kn_per_m = ask_f64(
        "Gravity line load in kN/m",
        project.requirements.gravity_load_kn_per_m,
    )?;
    project.requirements.lateral_load_kn =
        ask_f64("Lateral load in kN", project.requirements.lateral_load_kn)?;
    project.requirements.max_internal_columns = ask_usize(
        "Maximum internal columns Fraia may introduce",
        project.requirements.max_internal_columns,
    )?;
    project.intent.option_count = ask_usize(
        "How many options should Fraia return?",
        project.intent.option_count,
    )?;
    project.intent.search_permissions.add_internal_columns = ask_bool(
        "Allow Fraia to add internal columns/supports?",
        project.intent.search_permissions.add_internal_columns,
    )?;
    project.intent.search_permissions.change_topology = ask_bool(
        "Allow Fraia to explore different topologies?",
        project.intent.search_permissions.change_topology,
    )?;
    project.intent.hard_constraints = ask_list(
        "Hard constraints (comma-separated)",
        &project.intent.hard_constraints,
    )?;
    project.intent.soft_preferences = ask_list(
        "Soft preferences (comma-separated)",
        &project.intent.soft_preferences,
    )?;
    project.intent.approval_triggers = ask_list(
        "Approval triggers (comma-separated)",
        &project.intent.approval_triggers,
    )?;
    project.updated_at = Some(fraia_core::utils::iso_now());
    save_project(&dir, &project)?;
    update_planning_markdown(&dir, &default_planning_markdown(&project))?;
    println!("\nPlanning updated for {}", project.name);
    Ok(())
}

fn cmd_optimize(project_dir: Option<String>) -> Result<()> {
    let dir = require_dir(project_dir, "Usage: fraia optimize <projectDir>")?;
    let result = run_optimization(&dir)?;
    let summary = render_option_summary(&result.project, &result.selected, result.infeasible_count);
    fs::write(result.run_dir.join("summary.md"), &summary)?;
    println!("{}", summary);
    println!("\nSaved run artifacts to {}", result.run_dir.display());
    Ok(())
}

fn cmd_validate(project_dir: Option<String>) -> Result<()> {
    let dir = require_dir(project_dir, "Usage: fraia validate <projectDir>")?;
    let (project, paths) = load_project(&dir)?;
    let structural = materialize_project_structural_model(&project)
        .context("No authored structural model or builder-derived structural model saved in the project yet.")?;

    let validation = validate_structural_model(&structural);
    let realization = realize_structural_model_to_frame2d(&structural).ok();
    let design_actions = realization
        .as_ref()
        .and_then(|realization| derive_design_action_report(&project, &realization.model).ok());
    let checks = design_actions
        .as_ref()
        .map(|actions| derive_conservative_check_report(&project, actions));
    let run_id = format!("validate-{}", fraia_core::utils::timestamp_id());
    let run_dir = paths.runs_dir.join(&run_id);
    fs::create_dir_all(&run_dir)?;
    fraia_core::utils::write_json(&run_dir.join("validation.json"), &validation)?;
    if let Some(realization) = &realization {
        fraia_core::utils::write_json(&run_dir.join("realization.json"), realization)?;
    }
    if let Some(actions) = &design_actions {
        fraia_core::utils::write_json(&run_dir.join("design-actions.json"), actions)?;
    }
    if let Some(checks) = &checks {
        fraia_core::utils::write_json(&run_dir.join("checks.json"), checks)?;
        fs::write(
            run_dir.join("member-actions.csv"),
            render_member_actions_csv(design_actions.as_ref()),
        )?;
        fs::write(
            run_dir.join("check-results.csv"),
            render_check_results_csv(&checks.results),
        )?;
        fs::write(
            run_dir.join("support-reactions.csv"),
            render_support_reactions_csv(design_actions.as_ref()),
        )?;
    }
    let summary = render_validation_summary(
        &project,
        &validation,
        realization.as_ref(),
        design_actions.as_ref(),
        checks.as_ref(),
    );
    fs::write(run_dir.join("summary.md"), &summary)?;
    println!("{}", summary);
    println!("\nSaved validation artifacts to {}", run_dir.display());
    Ok(())
}

fn cmd_inspect_model(project_dir: Option<String>, format: Option<String>) -> Result<()> {
    let dir = require_dir(
        project_dir,
        "Usage: fraia inspect-model <projectDir> [--json]",
    )?;
    let (project, _paths) = load_project(&dir)?;
    let structural = materialize_project_structural_model(&project)
        .context("No authored structural model or builder-derived structural model saved in the project yet.")?;
    let report = understand_structural_model(&structural);

    match format.as_deref() {
        Some("--json") => {
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Some(other) => {
            anyhow::bail!(
                "Unknown inspect-model option `{other}`. Usage: fraia inspect-model <projectDir> [--json]"
            );
        }
        None => println!("{}", render_model_understanding_summary(&project, &report)),
    }
    Ok(())
}

fn cmd_adopt(project_dir: Option<String>, option_index: Option<String>) -> Result<()> {
    let dir = require_dir(project_dir, "Usage: fraia adopt <projectDir> <optionIndex>")?;
    let option_index = option_index
        .context("Usage: fraia adopt <projectDir> <optionIndex>")?
        .parse::<usize>()
        .context("optionIndex must be a 1-based integer")?;
    let (mut project, paths) = load_project(&dir)?;
    let run = load_latest_optimization_run(&paths.runs_dir)?;
    let option = run
        .options
        .get(option_index.saturating_sub(1))
        .with_context(|| format!("No option {} in latest run {}", option_index, run.run_id))?;
    let graph = portal_frame_builder_graph(
        &format!("builder.portal-frame.option-{}", option_index),
        &option.topology_id,
        &option.beam_section,
        &option.column_section,
        project.requirements.span_m,
        project.requirements.height_m,
        project.requirements.gravity_load_kn_per_m,
        project.requirements.lateral_load_kn,
        Some(run.run_id.clone()),
        Some(option_index),
    );
    let structural = materialize_structural_model_from_builder_graph(&graph)
        .context("Failed to materialize structural model from adopted builder graph")?;
    project.builder_graph = Some(graph);
    project.legacy_builder_instance = None;
    project.structural_model = Some(structural);
    project.updated_at = Some(fraia_core::utils::iso_now());
    save_project(&dir, &project)?;
    println!(
        "Adopted option {} from run {} into {}",
        option_index,
        run.run_id,
        paths.project_file.display()
    );
    Ok(())
}

fn cmd_demo(project_dir: Option<String>) -> Result<()> {
    let dir = PathBuf::from(project_dir.unwrap_or_else(|| "demo-project".into()));
    let _ = create_project(&dir, "Demo Warehouse")?;
    let result = run_optimization(&dir)?;
    let summary = render_option_summary(&result.project, &result.selected, result.infeasible_count);
    fs::write(result.run_dir.join("summary.md"), &summary)?;
    println!("{}", summary);
    println!("\nDemo project ready at {}", dir.display());
    Ok(())
}

fn create_portal_frame_demo_project(dir: &std::path::Path, name: &str) -> Result<()> {
    let (mut project, _) = create_project(dir, name)?;
    project.name = name.into();
    project.intent.building_type = "portal_frame".into();
    let graph = portal_frame_builder_graph(
        "builder.frame.demo",
        "clear_span",
        "310UB",
        "360UB",
        project.requirements.span_m,
        project.requirements.height_m,
        project.requirements.gravity_load_kn_per_m,
        project.requirements.lateral_load_kn,
        None,
        None,
    );
    project.builder_graph = Some(graph.clone());
    project.structural_model = materialize_structural_model_from_builder_graph(&graph);
    project.updated_at = Some(fraia_core::utils::iso_now());
    save_project(dir, &project)?;
    update_planning_markdown(dir, &default_planning_markdown(&project))?;
    Ok(())
}

fn cmd_frame_demo(project_dir: Option<String>) -> Result<()> {
    let dir = PathBuf::from(project_dir.unwrap_or_else(|| "frame-demo-project".into()));
    create_portal_frame_demo_project(&dir, "Demo Portal Frame")?;
    println!("Created frame demo project at {}", dir.display());
    println!(
        "- Use `fraia frame-run-calculix {}` to run the portal frame through the CalculiX seam",
        dir.display()
    );
    println!(
        "- Use `fraia validate {}` to inspect the resulting internal Fraia validation/check artifacts",
        dir.display()
    );
    Ok(())
}

fn cmd_import_stick_frame(project_dir: Option<String>, input_json: Option<String>) -> Result<()> {
    let dir = require_dir(
        project_dir,
        "Usage: fraia import-stick-frame <projectDir> <inputJson>",
    )?;
    let input_path = require_dir(
        input_json,
        "Usage: fraia import-stick-frame <projectDir> <inputJson>",
    )?;
    let imported = fraia_core::utils::read_json::<ImportedStickFrameInput>(&input_path)
        .with_context(|| format!("failed to read {}", input_path.display()))?;
    let artifacts = import_stick_frame_to_structural_model(&imported)?;

    let (mut project, paths) = if dir.join(fraia_core::project::PROJECT_FILE).exists() {
        load_project(&dir)?
    } else {
        let name = imported
            .name
            .clone()
            .or_else(|| {
                dir.file_name()
                    .and_then(|name| name.to_str())
                    .map(str::to_owned)
            })
            .unwrap_or_else(|| "Imported Stick Frame".into());
        create_project(&dir, &name)?
    };
    if let Some(name) = &imported.name {
        project.name = name.clone();
    }
    project.intent.building_type = "imported_stick_frame".into();
    project.builder_graph = None;
    project.legacy_builder_instance = None;
    project.structural_model = Some(artifacts.structural_model.clone());
    if let (Some(min_x), Some(max_x)) = (
        artifacts
            .structural_model
            .nodes
            .iter()
            .map(|node| node.x)
            .min_by(|a, b| a.total_cmp(b)),
        artifacts
            .structural_model
            .nodes
            .iter()
            .map(|node| node.x)
            .max_by(|a, b| a.total_cmp(b)),
    ) {
        project.requirements.span_m = max_x - min_x;
    }
    if let (Some(min_y), Some(max_y)) = (
        artifacts
            .structural_model
            .nodes
            .iter()
            .map(|node| node.y)
            .min_by(|a, b| a.total_cmp(b)),
        artifacts
            .structural_model
            .nodes
            .iter()
            .map(|node| node.y)
            .max_by(|a, b| a.total_cmp(b)),
    ) {
        project.requirements.height_m = max_y - min_y;
    }
    project.updated_at = Some(fraia_core::utils::iso_now());
    save_project(&dir, &project)?;
    update_planning_markdown(&dir, &default_planning_markdown(&project))?;

    let run_id = format!("import-stick-{}", fraia_core::utils::timestamp_id());
    let run_dir = paths.runs_dir.join(&run_id);
    fs::create_dir_all(&run_dir)?;
    fraia_core::utils::write_json(&run_dir.join("input.json"), &imported)?;
    fraia_core::utils::write_json(
        &run_dir.join("structural-model.json"),
        &artifacts.structural_model,
    )?;
    fraia_core::utils::write_json(&run_dir.join("cleanup-summary.json"), &artifacts)?;
    let summary = render_imported_stick_frame_summary(&artifacts);
    fs::write(run_dir.join("summary.md"), &summary)?;
    println!("{}", summary);
    println!("\nSaved import artifacts to {}", run_dir.display());
    Ok(())
}

fn create_beam_project(
    dir: &std::path::Path,
    name: &str,
    span_m: f64,
    distributed_load_kn_per_m: f64,
    point_load_kn: Option<f64>,
    point_load_x_m: Option<f64>,
) -> Result<()> {
    let (mut project, _) = create_project(dir, name)?;
    project.name = name.into();
    project.intent.objective_priority = "minimize_cost".into();
    project.requirements.span_m = span_m;
    project.requirements.height_m = 0.0;
    project.requirements.gravity_load_kn_per_m = distributed_load_kn_per_m;
    project.requirements.lateral_load_kn = 0.0;
    project.requirements.max_internal_columns = 0;
    let node_id = seed_simply_supported_beam_in_project(&mut project, Some("builder.beam.demo"))?;
    if let Some(graph) = &mut project.builder_graph
        && let Some(node) = graph.nodes.iter_mut().find(|node| node.id == node_id)
        && let fraia_core::BuilderNodeParameters::SimplySupportedBeam2D(params) =
            &mut node.parameters
    {
        params.point_load_kn = point_load_kn;
        params.point_load_x_m = point_load_x_m;
    }
    if let Some(graph) = &project.builder_graph {
        project.structural_model = materialize_structural_model_from_builder_graph(graph);
    }
    project.updated_at = Some(fraia_core::utils::iso_now());
    save_project(dir, &project)?;
    update_planning_markdown(dir, &default_planning_markdown(&project))?;
    Ok(())
}

fn cmd_beam_demo(project_dir: Option<String>) -> Result<()> {
    let dir = PathBuf::from(project_dir.unwrap_or_else(|| "beam-demo-project".into()));
    create_beam_project(
        &dir,
        "Demo Simply Supported Beam",
        6.0,
        8.0,
        Some(20.0),
        Some(3.0),
    )?;
    println!("Created beam demo project at {}", dir.display());
    println!("- Use `fraia beam-size {}` to size the beam", dir.display());
    println!(
        "- Use `fraia validate {}` to inspect the resulting analysis/check artifacts",
        dir.display()
    );
    Ok(())
}

fn cmd_beam_init(
    project_dir: Option<String>,
    span_m: Option<String>,
    distributed_load_kn_per_m: Option<String>,
    point_load_kn: Option<String>,
    point_load_x_m: Option<String>,
) -> Result<()> {
    let dir = require_dir(
        project_dir,
        "Usage: fraia beam-init <projectDir> <spanM> <udlKnPerM> [pointLoadKn] [pointLoadXM]",
    )?;
    let span_m = span_m
        .context(
            "Usage: fraia beam-init <projectDir> <spanM> <udlKnPerM> [pointLoadKn] [pointLoadXM]",
        )?
        .parse::<f64>()
        .context("spanM must be a number")?;
    let distributed_load_kn_per_m = distributed_load_kn_per_m
        .context(
            "Usage: fraia beam-init <projectDir> <spanM> <udlKnPerM> [pointLoadKn] [pointLoadXM]",
        )?
        .parse::<f64>()
        .context("udlKnPerM must be a number")?;
    let point_load_kn = point_load_kn
        .map(|value| value.parse::<f64>().context("pointLoadKn must be a number"))
        .transpose()?;
    let point_load_x_m = point_load_x_m
        .map(|value| value.parse::<f64>().context("pointLoadXM must be a number"))
        .transpose()?;

    create_beam_project(
        &dir,
        "Simply Supported Beam",
        span_m,
        distributed_load_kn_per_m,
        point_load_kn,
        point_load_x_m,
    )?;
    println!("Created beam project at {}", dir.display());
    println!("- Use `fraia beam-size {}` to size the beam", dir.display());
    println!(
        "- Use `fraia validate {}` to inspect the resulting analysis/check artifacts",
        dir.display()
    );
    Ok(())
}

fn cmd_beam_analyze(project_dir: Option<String>) -> Result<()> {
    let dir = require_dir(project_dir, "Usage: fraia beam-analyze <projectDir>")?;
    let (project, paths) = load_project(&dir)?;
    project.builder_graph.as_ref().context(
        "No builder graph saved in the project. Run `fraia beam-demo <projectDir>` or `fraia beam-init <projectDir> ...` first.",
    )?;
    current_simply_supported_beam_builder_params(&project)
        .context("No simply supported beam builder node was found in this project.")?;

    let analysis = analyze_current_simply_supported_beam_project(&project)?;
    let run_id = format!("beam-analysis-{}", fraia_core::utils::timestamp_id());
    let run_dir = paths.runs_dir.join(&run_id);
    fs::create_dir_all(&run_dir)?;
    fraia_core::utils::write_json(&run_dir.join("run.json"), &analysis.run)?;
    fraia_core::utils::write_json(&run_dir.join("snapshot.json"), &analysis.structural_model)?;
    fraia_core::utils::write_json(&run_dir.join("validation.json"), &analysis.validation)?;
    fraia_core::utils::write_json(&run_dir.join("realization.json"), &analysis.realization)?;
    fraia_core::utils::write_json(
        &run_dir.join("solver-input.json"),
        &analysis.realization.model,
    )?;
    fraia_core::utils::write_json(&run_dir.join("exact.json"), &analysis.exact_response)?;
    fraia_core::utils::write_json(
        &run_dir.join("internal-solve.json"),
        &analysis.internal_solve,
    )?;
    fraia_core::utils::write_json(
        &run_dir.join("internal-response.json"),
        &analysis.internal_response,
    )?;
    fraia_core::utils::write_json(&run_dir.join("comparison.json"), &analysis.comparison)?;
    let summary = render_beam_analysis_summary(&project, &analysis);
    fs::write(run_dir.join("summary.md"), &summary)?;
    println!("{}", summary);
    println!("\nSaved beam analysis artifacts to {}", run_dir.display());
    Ok(())
}

fn cmd_beam_compile_calculix(project_dir: Option<String>) -> Result<()> {
    let dir = require_dir(
        project_dir,
        "Usage: fraia beam-compile-calculix <projectDir>",
    )?;
    let (project, paths) = load_project(&dir)?;
    project.builder_graph.as_ref().context(
        "No builder graph saved in the project. Run `fraia beam-demo <projectDir>` or `fraia beam-init <projectDir> ...` first.",
    )?;
    current_simply_supported_beam_builder_params(&project)
        .context("No simply supported beam builder node was found in this project.")?;

    let analysis = compile_current_simply_supported_beam_project_to_calculix(&project)?;
    let run_id = format!("beam-calculix-{}", fraia_core::utils::timestamp_id());
    let run_dir = paths.runs_dir.join(&run_id);
    fs::create_dir_all(&run_dir)?;
    fraia_core::utils::write_json(&run_dir.join("run.json"), &analysis.run)?;
    fraia_core::utils::write_json(
        &run_dir.join("snapshot.json"),
        &analysis.baseline.structural_model,
    )?;
    fraia_core::utils::write_json(
        &run_dir.join("validation.json"),
        &analysis.baseline.validation,
    )?;
    fraia_core::utils::write_json(
        &run_dir.join("realization.json"),
        &analysis.baseline.realization,
    )?;
    fraia_core::utils::write_json(
        &run_dir.join("exact.json"),
        &analysis.baseline.exact_response,
    )?;
    fraia_core::utils::write_json(
        &run_dir.join("internal-solve.json"),
        &analysis.baseline.internal_solve,
    )?;
    fraia_core::utils::write_json(
        &run_dir.join("internal-response.json"),
        &analysis.baseline.internal_response,
    )?;
    fraia_core::utils::write_json(
        &run_dir.join("comparison.json"),
        &analysis.baseline.comparison,
    )?;
    fraia_core::utils::write_json(
        &run_dir.join("calculix-compiled.json"),
        &analysis.compiled_input,
    )?;
    fs::write(
        run_dir.join("calculix.inp"),
        &analysis.compiled_input.input_deck,
    )?;
    let summary = render_beam_calculix_summary(&project, &analysis);
    fs::write(run_dir.join("summary.md"), &summary)?;
    println!("{}", summary);
    println!(
        "\nSaved CalculiX compile artifacts to {}",
        run_dir.display()
    );
    Ok(())
}

fn cmd_frame_run_calculix(project_dir: Option<String>) -> Result<()> {
    let dir = require_dir(project_dir, "Usage: fraia frame-run-calculix <projectDir>")?;
    let (project, paths) = load_project(&dir)?;
    materialize_project_structural_model(&project)
        .context("No authored structural model or builder-derived structural model saved in the project yet.")?;

    require_calculix_runtime()?;

    let run_id = format!("frame-calculix-run-{}", fraia_core::utils::timestamp_id());
    let run_dir = paths.runs_dir.join(&run_id);
    fs::create_dir_all(&run_dir)?;
    let analysis = execute_current_frame_project_in_calculix(&project, &run_dir)?;
    fraia_core::utils::write_json(&run_dir.join("run.json"), &analysis.run)?;
    fraia_core::utils::write_json(&run_dir.join("snapshot.json"), &analysis.structural_model)?;
    fraia_core::utils::write_json(&run_dir.join("validation.json"), &analysis.validation)?;
    fraia_core::utils::write_json(&run_dir.join("realization.json"), &analysis.realization)?;
    fraia_core::utils::write_json(
        &run_dir.join("calculix-compiled.json"),
        &analysis.compiled_input,
    )?;
    fraia_core::utils::write_json(
        &run_dir.join("calculix-execution.json"),
        &analysis.execution,
    )?;
    fs::write(
        run_dir.join("calculix.inp"),
        &analysis.compiled_input.input_deck,
    )?;
    if let Some(points) = &analysis.extracted_node_displacements {
        fraia_core::utils::write_json(&run_dir.join("calculix-node-displacements.json"), points)?;
    }
    if let Some(points) = &analysis.extracted_support_reactions {
        fraia_core::utils::write_json(&run_dir.join("calculix-support-reactions.json"), points)?;
    }
    if let Some(points) = &analysis.extracted_element_stresses {
        fraia_core::utils::write_json(&run_dir.join("calculix-element-stresses.json"), points)?;
    }
    let verification_dir = run_dir.join("verification");
    fs::create_dir_all(&verification_dir)?;
    fraia_core::utils::write_json(
        &verification_dir.join("internal-solve.json"),
        &analysis.internal_solve,
    )?;
    if let Some(comparison) = &analysis.displacement_comparison {
        fraia_core::utils::write_json(
            &verification_dir.join("calculix-node-displacement-comparison.json"),
            comparison,
        )?;
    }
    if let Some(comparison) = &analysis.support_reaction_comparison {
        fraia_core::utils::write_json(
            &verification_dir.join("calculix-support-reaction-comparison.json"),
            comparison,
        )?;
    }
    if let Some(comparison) = &analysis.element_stress_comparison {
        fraia_core::utils::write_json(
            &verification_dir.join("calculix-element-stress-comparison.json"),
            comparison,
        )?;
    }
    let summary = render_frame_calculix_execution_summary(&project, &analysis);
    fs::write(run_dir.join("summary.md"), &summary)?;
    println!("{}", summary);
    println!(
        "\nSaved frame CalculiX run artifacts to {}",
        run_dir.display()
    );
    Ok(())
}

fn cmd_beam_run_calculix(project_dir: Option<String>) -> Result<()> {
    let dir = require_dir(project_dir, "Usage: fraia beam-run-calculix <projectDir>")?;
    let (project, paths) = load_project(&dir)?;
    project.builder_graph.as_ref().context(
        "No builder graph saved in the project. Run `fraia beam-demo <projectDir>` or `fraia beam-init <projectDir> ...` first.",
    )?;
    current_simply_supported_beam_builder_params(&project)
        .context("No simply supported beam builder node was found in this project.")?;

    require_calculix_runtime()?;

    let run_id = format!("beam-calculix-run-{}", fraia_core::utils::timestamp_id());
    let run_dir = paths.runs_dir.join(&run_id);
    fs::create_dir_all(&run_dir)?;
    let analysis = execute_current_simply_supported_beam_project_in_calculix(&project, &run_dir)?;
    fraia_core::utils::write_json(&run_dir.join("run.json"), &analysis.run)?;
    fraia_core::utils::write_json(
        &run_dir.join("snapshot.json"),
        &analysis.baseline.structural_model,
    )?;
    fraia_core::utils::write_json(
        &run_dir.join("validation.json"),
        &analysis.baseline.validation,
    )?;
    fraia_core::utils::write_json(
        &run_dir.join("realization.json"),
        &analysis.baseline.realization,
    )?;
    fraia_core::utils::write_json(
        &run_dir.join("calculix-compiled.json"),
        &analysis.compiled_input,
    )?;
    fraia_core::utils::write_json(
        &run_dir.join("calculix-execution.json"),
        &analysis.execution,
    )?;
    if let Some(extracted) = &analysis.extracted_response {
        fraia_core::utils::write_json(&run_dir.join("calculix-extracted.json"), extracted)?;
    }
    if let Some(profile) = &analysis.extracted_deflection_profile {
        fraia_core::utils::write_json(&run_dir.join("calculix-deflection-profile.json"), profile)?;
    }
    if let Some(stresses) = &analysis.extracted_element_stresses {
        fraia_core::utils::write_json(&run_dir.join("calculix-element-stresses.json"), stresses)?;
    }
    let verification_dir = run_dir.join("verification");
    fs::create_dir_all(&verification_dir)?;
    fraia_core::utils::write_json(
        &verification_dir.join("exact.json"),
        &analysis.baseline.exact_response,
    )?;
    fraia_core::utils::write_json(
        &verification_dir.join("internal-solve.json"),
        &analysis.baseline.internal_solve,
    )?;
    fraia_core::utils::write_json(
        &verification_dir.join("internal-response.json"),
        &analysis.baseline.internal_response,
    )?;
    fraia_core::utils::write_json(
        &verification_dir.join("comparison.json"),
        &analysis.baseline.comparison,
    )?;
    if let Some(extracted_comparison) = &analysis.extracted_comparison {
        fraia_core::utils::write_json(
            &verification_dir.join("calculix-comparison.json"),
            extracted_comparison,
        )?;
    }
    if let Some(profile_comparison) = &analysis.extracted_deflection_profile_comparison {
        fraia_core::utils::write_json(
            &verification_dir.join("calculix-deflection-profile-comparison.json"),
            profile_comparison,
        )?;
    }
    fs::write(
        run_dir.join("calculix.inp"),
        &analysis.compiled_input.input_deck,
    )?;
    let summary = render_beam_calculix_execution_summary(&project, &analysis);
    fs::write(run_dir.join("summary.md"), &summary)?;
    println!("{}", summary);
    println!("\nSaved CalculiX run artifacts to {}", run_dir.display());
    Ok(())
}

fn cmd_beam_size(project_dir: Option<String>) -> Result<()> {
    let dir = require_dir(project_dir, "Usage: fraia beam-size <projectDir>")?;
    let (mut project, paths) = load_project(&dir)?;
    project.builder_graph.as_ref().context(
        "No builder graph saved in the project. Run `fraia beam-demo <projectDir>` first.",
    )?;
    current_simply_supported_beam_builder_params(&project)
        .context("No simply supported beam builder node was found in this project.")?;

    let sizing = size_current_simply_supported_beam_in_project(&mut project)?;
    save_project(&dir, &project)?;
    update_planning_markdown(&dir, &default_planning_markdown(&project))?;

    let run_id = format!("beam-size-{}", fraia_core::utils::timestamp_id());
    let run_dir = paths.runs_dir.join(&run_id);
    fs::create_dir_all(&run_dir)?;
    fraia_core::utils::write_json(&run_dir.join("sizing.json"), &sizing)?;
    let summary = render_beam_sizing_summary(&project, &sizing);
    fs::write(run_dir.join("summary.md"), &summary)?;

    println!("{}", summary);
    println!("\nSaved beam sizing artifacts to {}", run_dir.display());
    Ok(())
}

fn render_validation_summary(
    project: &ProjectFile,
    validation: &fraia_core::ValidationReport,
    realization: Option<&fraia_core::Frame2DRealization>,
    design_actions: Option<&fraia_core::DesignActionReport>,
    checks: Option<&fraia_core::CheckReport>,
) -> String {
    fraia_core::render_validation_summary(project, validation, realization, design_actions, checks)
}

fn render_model_understanding_summary(
    project: &ProjectFile,
    report: &fraia_core::ModelUnderstandingReport,
) -> String {
    let mut lines = vec![
        format!("# Fraia model inspection: {}", project.name),
        String::new(),
        format!("- Dimension: {}", report.dimension),
        format!(
            "- Objects: {} nodes, {} members, {} plates, {} supports, {} loads, {} releases",
            report.counts.nodes,
            report.counts.members,
            report.counts.plates,
            report.counts.supports,
            report.counts.loads,
            report.counts.releases
        ),
        format!(
            "- Validation: {} errors, {} warnings",
            report.validation.error_count, report.validation.warning_count
        ),
    ];

    if let Some(bounds) = &report.bounds {
        lines.push(format!(
            "- Bounds: x {:.3}..{:.3} m, y {:.3}..{:.3} m, z {:.3}..{:.3} m",
            bounds.min_x, bounds.max_x, bounds.min_y, bounds.max_y, bounds.min_z, bounds.max_z
        ));
    }

    lines.push(String::new());
    lines.push("## Member roles".into());
    if report.member_roles.is_empty() {
        lines.push("- none".into());
    } else {
        for role in &report.member_roles {
            lines.push(format!(
                "- {}: {} members, {:.3} m total",
                role.role, role.count, role.total_length_m
            ));
        }
    }

    lines.push(String::new());
    lines.push("## Engineering member groups".into());
    if report.member_groups.is_empty() {
        lines.push("- none".into());
    } else {
        for group in &report.member_groups {
            lines.push(format!(
                "- {}: {} -> {}, role {}, section {}, material {}, length {:.3} m, analysis members {}",
                group.id,
                group.start_node,
                group.end_node,
                group.role,
                group.section_id,
                group.material_id,
                group.length_m,
                group.member_ids.join(", ")
            ));
            if !group.semantic_tags.is_empty() {
                lines.push(format!(
                    "  semantic tags: {}",
                    group.semantic_tags.join(", ")
                ));
            }
            if !group.recommended_section_families.is_empty() {
                lines.push(format!(
                    "  section families: {}",
                    group.recommended_section_families.join(", ")
                ));
            }
        }
    }

    lines.push(String::new());
    lines.push("## Analysis members".into());
    if report.members.is_empty() {
        lines.push("- none".into());
    } else {
        for member in &report.members {
            lines.push(format!(
                "- {}: {} -> {}, role {}, section {}, material {}, length {:.3} m",
                member.id,
                member.start_node,
                member.end_node,
                member.role,
                member.section_id,
                member.material_id,
                member.length_m
            ));
            if !member.semantic_tags.is_empty() {
                lines.push(format!(
                    "  semantic tags: {}",
                    member.semantic_tags.join(", ")
                ));
            }
            if !member.recommended_section_families.is_empty() {
                lines.push(format!(
                    "  section families: {}",
                    member.recommended_section_families.join(", ")
                ));
            }
            lines.push(format!(
                "  derived tags: {}",
                member.derived_tags.join(", ")
            ));
        }
    }

    lines.push(String::new());
    lines.push("## Plates".into());
    if report.plates.is_empty() {
        lines.push("- none".into());
    } else {
        for plate in &report.plates {
            lines.push(format!(
                "- {}: role {}, material {}, thickness {:.3} m, boundary nodes {}",
                plate.id,
                plate.role,
                plate.material_id,
                plate.thickness_m,
                plate.boundary_nodes.join(", ")
            ));
            if !plate.semantic_tags.is_empty() {
                lines.push(format!(
                    "  semantic tags: {}",
                    plate.semantic_tags.join(", ")
                ));
            }
            lines.push(format!("  generated from: {}", plate.generated_from));
            lines.push(format!("  derived tags: {}", plate.derived_tags.join(", ")));
        }
    }

    lines.push(String::new());
    lines.push("## Supports".into());
    if report.supports.is_empty() {
        lines.push("- none".into());
    } else {
        for support in &report.supports {
            lines.push(format!(
                "- {}: node {}, restrained {}",
                support.id,
                support.target_node,
                support.restrained_dofs.join(", ")
            ));
            lines.push(format!("  tags: {}", support.derived_tags.join(", ")));
        }
    }

    lines.push(String::new());
    lines.push("## Loads".into());
    if report.loads.is_empty() {
        lines.push("- none".into());
    } else {
        for load in &report.loads {
            lines.push(format!(
                "- {}: {} {}, {} {:.3} in case {}",
                load.id,
                load.target_kind,
                load.target_id,
                load.kind,
                load.magnitude,
                load.load_case_id
            ));
            lines.push(format!(
                "  direction: ({:.3}, {:.3}, {:.3})",
                load.direction.x, load.direction.y, load.direction.z
            ));
            lines.push(format!("  tags: {}", load.derived_tags.join(", ")));
        }
    }

    lines.push(String::new());
    lines.push("## Unresolved semantic objects".into());
    if report.unresolved_objects.is_empty() {
        lines.push("- none".into());
    } else {
        for object in &report.unresolved_objects {
            lines.push(format!(
                "- {} {}: {}",
                object.object_kind, object.object_id, object.reason
            ));
        }
    }

    lines.push(String::new());
    lines.push("## Builder provenance".into());
    if report.builder_materializations.is_empty() {
        lines.push("- none".into());
    } else {
        for materialization in &report.builder_materializations {
            lines.push(format!(
                "- {}: {} objects",
                materialization.builder_node_id, materialization.object_count
            ));
            lines.push(format!(
                "  tags: {}",
                materialization.derived_tags.join(", ")
            ));
        }
    }

    if !report.validation.diagnostics.is_empty() {
        lines.push(String::new());
        lines.push("## Diagnostics".into());
        for diagnostic in &report.validation.diagnostics {
            lines.push(format!(
                "- {:?} {}: {}",
                diagnostic.severity, diagnostic.code, diagnostic.message
            ));
        }
    }

    lines.join("\n")
}

fn render_option_summary(
    project: &ProjectFile,
    options: &[fraia_core::CandidateOption],
    infeasible_count: usize,
) -> String {
    let mut lines = vec![
        format!("# Fraia option study: {}", project.name),
        String::new(),
        format!("- Building type: {}", project.intent.building_type),
        format!(
            "- Objective priority: {}",
            project.intent.objective_priority
        ),
        format!("- Span: {} m", project.requirements.span_m),
        format!("- Height: {} m", project.requirements.height_m),
        format!(
            "- Gravity load: {} kN/m",
            project.requirements.gravity_load_kn_per_m
        ),
        format!(
            "- Lateral load: {} kN",
            project.requirements.lateral_load_kn
        ),
        format!("- Infeasible candidates rejected: {}", infeasible_count),
        String::new(),
        "## Recommended options".into(),
        String::new(),
    ];

    for (idx, option) in options.iter().enumerate() {
        lines.push(format!("### Option {}: {}", idx + 1, option.topology));
        lines.push(format!("- Beam section: {}", option.beam_section));
        lines.push(format!("- Column section: {}", option.column_section));
        lines.push(format!("- Internal columns: {}", option.internal_columns));
        lines.push(format!("- Cost proxy: {}", option.cost));
        lines.push(format!("- Carbon proxy: {}", option.carbon));
        lines.push(format!("- Mass: {} kg", option.mass_kg));
        lines.push(format!("- Max utilization: {}", option.max_utilization));
        lines.push(format!(
            "- Max vertical deflection: {} mm (ratio ~ L/{})",
            option.max_deflection_mm,
            option
                .deflection_ratio
                .map(|v| v.to_string())
                .unwrap_or_else(|| "n/a".into())
        ));
        lines.push(format!(
            "- Max drift: {} mm (ratio ~ H/{})",
            option.max_drift_mm,
            option
                .drift_ratio
                .map(|v| v.to_string())
                .unwrap_or_else(|| "n/a".into())
        ));
        lines.push(format!("- Summary: {}", option.summary));
        if !option.tradeoffs.is_empty() {
            lines.push("- Tradeoffs:".into());
            for tradeoff in &option.tradeoffs {
                lines.push(format!("  - {}", tradeoff));
            }
        }
        lines.push(String::new());
    }

    lines.push("## Notes".into());
    lines.push(String::new());
    lines.push("- This is a planning-first Rust MVP, not a code-compliant design system.".into());
    lines.push("- Analysis is linear-elastic 2D frame analysis only.".into());
    lines.push("- Results are intended for concept exploration and option comparison.".into());
    lines.join("\n")
}

fn render_frame_calculix_execution_summary(
    project: &ProjectFile,
    analysis: &fraia_core::CurrentFrameCalculixExecutionArtifacts,
) -> String {
    fraia_core::render_frame_calculix_execution_summary(project, analysis)
}

fn render_beam_calculix_execution_summary(
    project: &ProjectFile,
    analysis: &fraia_core::SimplySupportedBeamCalculixExecutionArtifacts,
) -> String {
    fraia_core::render_beam_calculix_execution_summary(project, analysis)
}

fn render_beam_calculix_summary(
    project: &ProjectFile,
    analysis: &fraia_core::SimplySupportedBeamCalculixArtifacts,
) -> String {
    fraia_core::render_beam_calculix_summary(project, analysis)
}

fn render_beam_analysis_summary(
    project: &ProjectFile,
    analysis: &fraia_core::SimplySupportedBeamAnalysisArtifacts,
) -> String {
    fraia_core::render_beam_analysis_summary(project, analysis)
}

fn render_imported_stick_frame_summary(
    artifacts: &fraia_core::ImportedStickFrameArtifacts,
) -> String {
    fraia_core::render_imported_stick_frame_summary(artifacts)
}

fn render_beam_sizing_summary(
    project: &ProjectFile,
    sizing: &fraia_core::SimplySupportedBeamSizingResult,
) -> String {
    fraia_core::render_beam_sizing_summary(project, sizing)
}

fn render_member_actions_csv(design_actions: Option<&fraia_core::DesignActionReport>) -> String {
    fraia_core::render_member_actions_csv(design_actions)
}

fn render_support_reactions_csv(design_actions: Option<&fraia_core::DesignActionReport>) -> String {
    fraia_core::render_support_reactions_csv(design_actions)
}

fn render_check_results_csv(results: &[fraia_core::CheckResult]) -> String {
    fraia_core::render_check_results_csv(results)
}

fn ask(prompt: &str, current: &str) -> Result<String> {
    print!("{} [{}]: ", prompt, current);
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    let value = buf.trim();
    Ok(if value.is_empty() {
        current.to_string()
    } else {
        value.to_string()
    })
}

fn ask_f64(prompt: &str, current: f64) -> Result<f64> {
    let text = ask(prompt, &current.to_string())?;
    text.parse::<f64>()
        .with_context(|| format!("Invalid number for {prompt}"))
}

fn ask_usize(prompt: &str, current: usize) -> Result<usize> {
    let text = ask(prompt, &current.to_string())?;
    text.parse::<usize>()
        .with_context(|| format!("Invalid integer for {prompt}"))
}

fn ask_bool(prompt: &str, current: bool) -> Result<bool> {
    let current_text = if current { "y" } else { "n" };
    let text = ask(prompt, current_text)?;
    Ok(matches!(
        text.to_ascii_lowercase().as_str(),
        "y" | "yes" | "true" | "1"
    ))
}

fn ask_list(prompt: &str, current: &[String]) -> Result<Vec<String>> {
    let text = ask(prompt, &current.join(", "))?;
    Ok(text
        .split(',')
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect())
}

fn load_latest_optimization_run(runs_dir: &std::path::Path) -> Result<OptimizationRun> {
    let latest_run_dir = fs::read_dir(runs_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map(|ty| ty.is_dir()).unwrap_or(false))
        .filter(|entry| entry.path().join("options.json").exists())
        .max_by_key(|entry| entry.file_name())
        .context("No optimization runs found. Run `fraia optimize <projectDir>` first.")?;
    let options_file = latest_run_dir.path().join("options.json");
    let text = fs::read_to_string(&options_file)
        .with_context(|| format!("Failed to read {}", options_file.display()))?;
    let run = serde_json::from_str::<OptimizationRun>(&text)
        .with_context(|| format!("Failed to parse {}", options_file.display()))?;
    Ok(run)
}

fn require_dir(value: Option<String>, usage: &str) -> Result<PathBuf> {
    value.map(PathBuf::from).with_context(|| usage.to_string())
}

fn print_help() {
    println!(
        "Fraia Rust MVP\n\nMachine interface:\n  fraia operation --database <sqlitePath>\n  fraia operation-capabilities\n  fraia operation-schema\n\nCompatibility-only human commands:\n  fraia init <projectDir>\n  fraia plan <projectDir>\n  fraia optimize <projectDir>\n  fraia validate <projectDir>\n  fraia inspect-model <projectDir> [--json]\n  fraia adopt <projectDir> <optionIndex>\n  fraia demo [projectDir]\n  fraia frame-demo [projectDir]\n  fraia import-stick-frame <projectDir> <inputJson>\n  fraia frame-run-calculix <projectDir>\n  fraia beam-demo [projectDir]\n  fraia beam-init <projectDir> <spanM> <udlKnPerM> [pointLoadKn] [pointLoadXM]\n  fraia beam-size <projectDir>\n  fraia beam-analyze <projectDir>\n  fraia beam-compile-calculix <projectDir>\n  fraia beam-run-calculix <projectDir>\n  fraia design-runs-list <projectDir> <designId>\n  fraia design-run-inspect <projectDir> <designId> <runId>\n  fraia design-runs-status <projectDir> <designId> <snapshotId> [ancestorSnapshotIdsCommaSeparated]\n"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use fraia_core::{BuilderNodeParameters, StructuralModel};
    use fraia_revision::{
        ConversationId, ProjectId, RevisionId, SnapshotId,
        analysis_service::AnalysisSettings,
        snapshot::ModelSnapshot,
        sqlite::{StoredConversation, StoredProjectRoot, StoredRevision, StoredSnapshot},
    };

    static CALCULIX_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn seed_operation_database(path: &std::path::Path, model: StructuralModel) -> SnapshotId {
        seed_operation_database_for_project(path, model, "cli-project")
    }

    fn seed_operation_database_for_project(
        path: &std::path::Path,
        model: StructuralModel,
        project_id: &str,
    ) -> SnapshotId {
        let snapshot = ModelSnapshot::capture(model).unwrap();
        let snapshot_id = snapshot.id().clone();
        let mut repository = SqliteRevisionRepository::open(path).unwrap();
        repository
            .create_project(StoredProjectRoot {
                project_id: ProjectId::new(project_id),
                root_conversation: StoredConversation {
                    id: ConversationId::from("overall"),
                    project_id: ProjectId::new(project_id),
                    purpose: "CLI adapter test".into(),
                    origin_json: "{\"kind\":\"root\"}".into(),
                    head_revision_id: RevisionId::from("root"),
                },
                root_revision: StoredRevision {
                    id: RevisionId::from("root"),
                    snapshot_id: snapshot_id.clone(),
                    parent_revision_id: None,
                    conversation_id: ConversationId::from("overall"),
                    metadata_json: "{\"operation\":\"root\"}".into(),
                },
                root_snapshot: StoredSnapshot {
                    id: snapshot_id.clone(),
                    format_version: snapshot.canonical_format_version().as_str().into(),
                    canonical_bytes: snapshot.canonical_bytes().to_vec(),
                },
            })
            .unwrap();
        snapshot_id
    }

    fn run_json_operation(
        database: &std::path::Path,
        request: &OperationRequest,
    ) -> (i32, Vec<u8>, serde_json::Value) {
        let input = serde_json::to_vec(request).unwrap();
        let mut output = Vec::new();
        let exit = run_machine_command(
            &[
                "operation".into(),
                "--database".into(),
                database.to_string_lossy().into_owned(),
            ],
            &mut input.as_slice(),
            &mut output,
        );
        let value = serde_json::from_slice(&output).unwrap();
        (exit, output, value)
    }

    #[test]
    fn cmd_validate_writes_validation_and_engineering_artifacts() {
        let temp_dir = std::env::temp_dir().join(format!(
            "fraia-cli-validate-{}",
            fraia_core::utils::timestamp_id()
        ));
        let (mut project, paths) =
            create_project(&temp_dir, "cli-validate").expect("create project");
        let graph = portal_frame_builder_graph(
            "builder-1",
            "clear_span",
            "310UB",
            "360UB",
            20.0,
            6.0,
            20.0,
            80.0,
            None,
            None,
        );
        let structural = materialize_structural_model_from_builder_graph(&graph)
            .expect("materialize structural model");
        project.builder_graph = Some(graph);
        project.structural_model = Some(structural);
        save_project(&temp_dir, &project).expect("save project");

        cmd_validate(Some(temp_dir.to_string_lossy().to_string())).expect("validate command");

        let latest_validate_run = fs::read_dir(&paths.runs_dir)
            .expect("read runs dir")
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_name().to_string_lossy().starts_with("validate-"))
            .max_by_key(|entry| entry.file_name())
            .expect("latest validate run");
        let run_dir = latest_validate_run.path();
        for artifact in [
            "validation.json",
            "realization.json",
            "design-actions.json",
            "checks.json",
            "member-actions.csv",
            "support-reactions.csv",
            "check-results.csv",
            "summary.md",
        ] {
            assert!(
                run_dir.join(artifact).exists(),
                "missing artifact {artifact}"
            );
        }
        let summary = fs::read_to_string(run_dir.join("summary.md")).expect("read summary");
        assert!(summary.contains("## Engineering outputs"));
        assert!(summary.contains("Governing drift ratio"));
        assert!(summary.contains("Builder node statuses:"));
        assert!(summary.contains("Total builder-generated objects:"));

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn cmd_adopt_persists_builder_graph_from_latest_optimization_run() {
        let temp_dir = std::env::temp_dir().join(format!(
            "fraia-cli-adopt-{}",
            fraia_core::utils::timestamp_id()
        ));
        let _ = create_project(&temp_dir, "cli-adopt").expect("create project");
        let optimization = run_optimization(&temp_dir).expect("run optimization");
        assert!(!optimization.selected.is_empty());

        cmd_adopt(
            Some(temp_dir.to_string_lossy().to_string()),
            Some("1".into()),
        )
        .expect("adopt command");

        let (project, _) = load_project(&temp_dir).expect("load project");
        let graph = project.builder_graph.expect("builder graph");
        assert_eq!(graph.root_node_ids.len(), 1);
        let root = graph
            .nodes
            .iter()
            .find(|node| node.id == graph.root_node_ids[0])
            .expect("root node");
        assert_eq!(
            root.source_run_id.as_deref(),
            Some(optimization.run_id.as_str())
        );
        assert_eq!(root.source_option_index, Some(1));
        assert!(project.structural_model.is_some());

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn load_latest_optimization_run_ignores_validation_run_directories() {
        let temp_dir = std::env::temp_dir().join(format!(
            "fraia-cli-load-run-{}",
            fraia_core::utils::timestamp_id()
        ));
        let (_, paths) = create_project(&temp_dir, "cli-load-run").expect("create project");
        let optimization = run_optimization(&temp_dir).expect("run optimization");
        let validation_dir = paths
            .runs_dir
            .join(format!("validate-{}", fraia_core::utils::timestamp_id()));
        fs::create_dir_all(&validation_dir).expect("create validate dir");
        fs::write(validation_dir.join("summary.md"), "validation summary").expect("write summary");

        let loaded =
            load_latest_optimization_run(&paths.runs_dir).expect("load latest optimization run");

        assert_eq!(loaded.run_id, optimization.run_id);
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn cmd_frame_demo_creates_a_portal_frame_project() {
        let temp_dir = std::env::temp_dir().join(format!(
            "fraia-cli-frame-demo-{}",
            fraia_core::utils::timestamp_id()
        ));

        cmd_frame_demo(Some(temp_dir.to_string_lossy().to_string())).expect("frame demo command");

        let (project, _) = load_project(&temp_dir).expect("load frame project");
        let graph = project.builder_graph.expect("frame builder graph");
        assert!(
            graph
                .nodes
                .iter()
                .any(|node| node.archetype_id == fraia_core::PORTAL_FRAME_2D_ARCHETYPE_ID)
        );
        assert!(project.structural_model.is_some());

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn cmd_import_stick_frame_cleans_and_persists_structural_model() {
        let temp_dir = std::env::temp_dir().join(format!(
            "fraia-cli-import-stick-{}",
            fraia_core::utils::timestamp_id()
        ));
        let input_path = temp_dir.join("input.json");
        fs::create_dir_all(&temp_dir).expect("temp dir");
        fs::write(
            &input_path,
            r#"{
  "name": "Imported Stick Portal",
  "section_id": "250UB",
  "material_id": "steel",
  "segments": [
    {"id":"beam","start":{"x":0.0,"y":3.0},"end":{"x":6.0,"y":3.0},"uniform_line_load_kn_per_m":8.0},
    {"id":"left","start":{"x":0.0,"y":0.0},"end":{"x":0.0,"y":3.0}},
    {"id":"mid","start":{"x":3.0,"y":0.0},"end":{"x":3.0,"y":3.0}},
    {"id":"right","start":{"x":6.0,"y":0.0},"end":{"x":6.0,"y":3.0}}
  ],
  "supports": [
    {"id":"s1","point":{"x":0.0,"y":0.0},"ux":true,"uy":true,"rz":true},
    {"id":"s2","point":{"x":3.0,"y":0.0},"ux":true,"uy":true,"rz":true},
    {"id":"s3","point":{"x":6.0,"y":0.0},"ux":true,"uy":true,"rz":true}
  ]
}"#,
        )
        .expect("write input");
        let project_dir = temp_dir.join("project");

        cmd_import_stick_frame(
            Some(project_dir.to_string_lossy().to_string()),
            Some(input_path.to_string_lossy().to_string()),
        )
        .expect("import stick frame command");

        let (project, paths) = load_project(&project_dir).expect("load imported project");
        let structural = project.structural_model.expect("structural model");
        assert_eq!(project.intent.building_type, "imported_stick_frame");
        assert!(structural.members.len() >= 5);
        assert_eq!(structural.supports.len(), 3);
        let latest_run_dir = fs::read_dir(paths.runs_dir)
            .expect("read runs dir")
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("import-stick-")
            })
            .max_by_key(|entry| entry.file_name())
            .expect("import run dir")
            .path();
        assert!(latest_run_dir.join("cleanup-summary.json").exists());
        assert!(latest_run_dir.join("summary.md").exists());

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn cmd_frame_run_calculix_fails_clearly_when_runtime_missing() {
        let _environment = CALCULIX_ENV_LOCK.lock().unwrap();
        let temp_dir = std::env::temp_dir().join(format!(
            "fraia-cli-frame-calculix-run-{}",
            fraia_core::utils::timestamp_id()
        ));
        cmd_frame_demo(Some(temp_dir.to_string_lossy().to_string())).expect("frame demo command");
        let original = std::env::var_os("FRAIA_DISABLE_CALCULIX_RUNTIME");
        unsafe {
            std::env::set_var("FRAIA_DISABLE_CALCULIX_RUNTIME", "1");
        }
        let err = cmd_frame_run_calculix(Some(temp_dir.to_string_lossy().to_string()))
            .expect_err("runtime-unavailable error");
        match original {
            Some(value) => unsafe {
                std::env::set_var("FRAIA_DISABLE_CALCULIX_RUNTIME", value);
            },
            None => unsafe {
                std::env::remove_var("FRAIA_DISABLE_CALCULIX_RUNTIME");
            },
        }

        assert!(
            err.to_string().contains("CalculiX runtime unavailable"),
            "unexpected error: {err:#}"
        );
        let (_, paths) = load_project(&temp_dir).expect("load frame project");
        let run_count = fs::read_dir(paths.runs_dir)
            .expect("read runs dir")
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("frame-calculix-run-")
            })
            .count();
        assert_eq!(run_count, 0);

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn cmd_beam_demo_creates_a_beam_project() {
        let temp_dir = std::env::temp_dir().join(format!(
            "fraia-cli-beam-demo-{}",
            fraia_core::utils::timestamp_id()
        ));

        cmd_beam_demo(Some(temp_dir.to_string_lossy().to_string())).expect("beam demo command");

        let (project, _) = load_project(&temp_dir).expect("load beam project");
        let graph = project.builder_graph.expect("beam builder graph");
        assert!(graph.nodes.iter().any(|node| matches!(
            node.parameters,
            BuilderNodeParameters::SimplySupportedBeam2D(_)
        )));
        assert!(project.structural_model.is_some());

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn cmd_beam_init_accepts_custom_span_and_loads() {
        let temp_dir = std::env::temp_dir().join(format!(
            "fraia-cli-beam-init-{}",
            fraia_core::utils::timestamp_id()
        ));

        cmd_beam_init(
            Some(temp_dir.to_string_lossy().to_string()),
            Some("8.0".into()),
            Some("5.0".into()),
            Some("12.0".into()),
            Some("4.0".into()),
        )
        .expect("beam init command");

        let (project, _) = load_project(&temp_dir).expect("load beam project");
        let graph = project.builder_graph.expect("beam builder graph");
        let beam_params = graph
            .nodes
            .iter()
            .find_map(|node| match &node.parameters {
                BuilderNodeParameters::SimplySupportedBeam2D(params) => Some(params),
                _ => None,
            })
            .expect("beam params");
        assert_eq!(beam_params.span_m, 8.0);
        assert_eq!(beam_params.distributed_load_kn_per_m, 5.0);
        assert_eq!(beam_params.point_load_kn, Some(12.0));
        assert_eq!(beam_params.point_load_x_m, Some(4.0));

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn cmd_beam_size_updates_project_and_writes_artifacts() {
        let temp_dir = std::env::temp_dir().join(format!(
            "fraia-cli-beam-size-{}",
            fraia_core::utils::timestamp_id()
        ));
        cmd_beam_demo(Some(temp_dir.to_string_lossy().to_string())).expect("beam demo command");

        cmd_beam_size(Some(temp_dir.to_string_lossy().to_string())).expect("beam size command");

        let (project, paths) = load_project(&temp_dir).expect("load beam project");
        let graph = project.builder_graph.expect("beam builder graph");
        let beam_params = graph
            .nodes
            .iter()
            .find_map(|node| match &node.parameters {
                BuilderNodeParameters::SimplySupportedBeam2D(params) => Some(params),
                _ => None,
            })
            .expect("beam params");
        assert_ne!(beam_params.section, "200UB");
        assert!(project.structural_model.is_some());
        let latest_run_dir = fs::read_dir(paths.runs_dir)
            .expect("read runs dir")
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("beam-size-")
            })
            .max_by_key(|entry| entry.file_name())
            .expect("beam sizing run dir")
            .path();
        assert!(latest_run_dir.join("sizing.json").exists());
        assert!(latest_run_dir.join("summary.md").exists());

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn cmd_beam_analyze_writes_explicit_analysis_artifacts() {
        let temp_dir = std::env::temp_dir().join(format!(
            "fraia-cli-beam-analyze-{}",
            fraia_core::utils::timestamp_id()
        ));
        cmd_beam_demo(Some(temp_dir.to_string_lossy().to_string())).expect("beam demo command");
        cmd_beam_size(Some(temp_dir.to_string_lossy().to_string())).expect("beam size command");

        cmd_beam_analyze(Some(temp_dir.to_string_lossy().to_string()))
            .expect("beam analyze command");

        let (_, paths) = load_project(&temp_dir).expect("load beam project");
        let latest_run_dir = fs::read_dir(paths.runs_dir)
            .expect("read runs dir")
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("beam-analysis-")
            })
            .max_by_key(|entry| entry.file_name())
            .expect("beam analysis run dir")
            .path();
        for artifact in [
            "run.json",
            "snapshot.json",
            "validation.json",
            "realization.json",
            "solver-input.json",
            "exact.json",
            "internal-solve.json",
            "internal-response.json",
            "comparison.json",
            "summary.md",
        ] {
            assert!(
                latest_run_dir.join(artifact).exists(),
                "missing artifact {artifact}"
            );
        }
        let summary = fs::read_to_string(latest_run_dir.join("summary.md")).expect("summary");
        assert!(summary.contains("## Exact vs internal comparison"));
        assert!(summary.contains("Internal solver"));

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn cmd_beam_compile_calculix_writes_compiled_input_artifacts() {
        let temp_dir = std::env::temp_dir().join(format!(
            "fraia-cli-beam-calculix-{}",
            fraia_core::utils::timestamp_id()
        ));
        cmd_beam_demo(Some(temp_dir.to_string_lossy().to_string())).expect("beam demo command");
        cmd_beam_size(Some(temp_dir.to_string_lossy().to_string())).expect("beam size command");

        cmd_beam_compile_calculix(Some(temp_dir.to_string_lossy().to_string()))
            .expect("beam calculix compile command");

        let (_, paths) = load_project(&temp_dir).expect("load beam project");
        let latest_run_dir = fs::read_dir(paths.runs_dir)
            .expect("read runs dir")
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("beam-calculix-")
            })
            .max_by_key(|entry| entry.file_name())
            .expect("beam calculix run dir")
            .path();
        for artifact in [
            "run.json",
            "snapshot.json",
            "validation.json",
            "realization.json",
            "exact.json",
            "internal-solve.json",
            "internal-response.json",
            "comparison.json",
            "calculix-compiled.json",
            "calculix.inp",
            "summary.md",
        ] {
            assert!(
                latest_run_dir.join(artifact).exists(),
                "missing artifact {artifact}"
            );
        }
        let deck = fs::read_to_string(latest_run_dir.join("calculix.inp")).expect("deck");
        assert!(deck.contains("*ELEMENT,TYPE=B31"));
        let summary = fs::read_to_string(latest_run_dir.join("summary.md")).expect("summary");
        assert!(summary.contains("CalculiX runtime available"));

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn cmd_beam_run_calculix_fails_clearly_when_runtime_missing() {
        let _environment = CALCULIX_ENV_LOCK.lock().unwrap();
        let temp_dir = std::env::temp_dir().join(format!(
            "fraia-cli-beam-calculix-run-{}",
            fraia_core::utils::timestamp_id()
        ));
        cmd_beam_demo(Some(temp_dir.to_string_lossy().to_string())).expect("beam demo command");
        cmd_beam_size(Some(temp_dir.to_string_lossy().to_string())).expect("beam size command");

        let original = std::env::var_os("FRAIA_DISABLE_CALCULIX_RUNTIME");
        unsafe {
            std::env::set_var("FRAIA_DISABLE_CALCULIX_RUNTIME", "1");
        }
        let err = cmd_beam_run_calculix(Some(temp_dir.to_string_lossy().to_string()))
            .expect_err("runtime-unavailable error");
        match original {
            Some(value) => unsafe {
                std::env::set_var("FRAIA_DISABLE_CALCULIX_RUNTIME", value);
            },
            None => unsafe {
                std::env::remove_var("FRAIA_DISABLE_CALCULIX_RUNTIME");
            },
        }

        assert!(
            err.to_string().contains("CalculiX runtime unavailable"),
            "unexpected error: {err:#}"
        );
        let (_, paths) = load_project(&temp_dir).expect("load beam project");
        let run_count = fs::read_dir(paths.runs_dir)
            .expect("read runs dir")
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("beam-calculix-run-")
            })
            .count();
        assert_eq!(run_count, 0);

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn operation_schema_is_stable_single_line_json() {
        let mut output = Vec::new();
        let exit = run_machine_command(
            &["operation-schema".into()],
            &mut std::io::empty(),
            &mut output,
        );
        assert_eq!(exit, EXIT_SUCCESS);
        assert_eq!(
            String::from_utf8(output).unwrap(),
            concat!(
                "{\"commands\":{\"capabilities\":\"fraia operation-capabilities\",",
                "\"execute\":\"fraia operation --database <sqlite-path>\"},",
                "\"compatibilityCommands\":[\"init\",\"plan\",\"optimize\",\"validate\",",
                "\"inspect-model\",\"adopt\",\"demo\",\"frame-demo\",\"import-stick-frame\",",
                "\"frame-run-calculix\",\"beam-demo\",\"beam-init\",\"beam-size\",",
                "\"beam-analyze\",\"beam-compile-calculix\",\"beam-run-calculix\",",
                "\"design-runs-list\",\"design-run-inspect\",\"design-runs-status\"],",
                "\"contractVersion\":\"fraia.operations.v1\",",
                "\"exitCodes\":{\"headConflict\":3,\"invalidInput\":65,\"operationError\":2,",
                "\"repositoryError\":70,\"runtimeUnavailable\":4,\"snapshotConflict\":3,",
                "\"success\":0,\"usage\":64},",
                "\"schema\":\"fraia.operation-schema.v1\",",
                "\"transport\":{\"input\":\"one JSON OperationRequest on stdin\",",
                "\"output\":\"one JSON OperationResponse on stdout\"}}\n"
            )
        );
    }

    #[test]
    fn capability_adapter_matches_shared_operation_contract() {
        let mut output = Vec::new();
        let exit = run_machine_command(
            &["operation-capabilities".into()],
            &mut std::io::empty(),
            &mut output,
        );
        assert_eq!(exit, EXIT_SUCCESS);
        let cli: serde_json::Value = serde_json::from_slice(&output).unwrap();

        let mut repository = SqliteRevisionRepository::open_in_memory().unwrap();
        let direct = execute_sqlite_operation(
            &mut repository,
            OperationRequest {
                contract_version: OPERATION_CONTRACT_VERSION.into(),
                request_id: "capabilities".into(),
                operation: Operation::Capabilities,
            },
        );
        assert_eq!(cli, serde_json::to_value(direct).unwrap());
        let operations = cli["result"]["operations"].as_array().unwrap();
        for operation in [
            "reject_structural_patch",
            "validate_snapshot",
            "analyse_snapshot",
            "inspect_analysis_evidence",
        ] {
            assert!(operations.iter().any(|value| value == operation));
        }
        let features = cli["result"]["features"].as_array().unwrap();
        for primitive in ["node", "member", "plate", "support", "load", "release"] {
            assert!(
                features
                    .iter()
                    .any(|value| value == &format!("patch_{primitive}"))
            );
        }
    }

    #[test]
    fn malformed_machine_input_has_stable_json_and_exit_code() {
        let mut output = Vec::new();
        let exit = run_machine_command(
            &["operation".into(), "--database".into(), ":memory:".into()],
            &mut "not-json".as_bytes(),
            &mut output,
        );
        assert_eq!(exit, EXIT_INPUT);
        let value: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(value["schema"], "fraia.cli.error.v1");
        assert_eq!(value["code"], "invalid_json");
    }

    #[test]
    fn repository_operation_error_uses_repository_exit_code() {
        let request = serde_json::json!({
            "contractVersion": "fraia.operations.v1",
            "requestId": "inspect-missing",
            "operation": "inspect",
            "parameters": { "conversation_id": "missing" }
        })
        .to_string();
        let mut output = Vec::new();
        let exit = run_machine_command(
            &["operation".into(), "--database".into(), ":memory:".into()],
            &mut request.as_bytes(),
            &mut output,
        );
        assert_eq!(exit, EXIT_REPOSITORY);
        let value: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(value["status"], "error");
        assert_eq!(value["error"]["code"], "repository_error");
    }

    #[test]
    fn snapshot_conflict_has_stable_json_and_conflict_exit() {
        let directory = tempfile::tempdir().unwrap();
        let database = directory.path().join("snapshot-conflict.sqlite");
        seed_operation_database(&database, StructuralModel::empty());
        let (exit, _, value) = run_json_operation(
            &database,
            &OperationRequest {
                contract_version: OPERATION_CONTRACT_VERSION.into(),
                request_id: "snapshot-conflict".into(),
                operation: Operation::ValidateSnapshot {
                    revision_id: RevisionId::from("root"),
                    expected_snapshot_id: SnapshotId::from("wrong"),
                },
            },
        );
        assert_eq!(exit, EXIT_HEAD_CONFLICT);
        assert_eq!(value["status"], "error");
        assert_eq!(value["error"]["code"], "expected_snapshot_mismatch");
        assert_eq!(value["error"]["snapshotConflict"]["revisionId"], "root");
    }

    #[test]
    fn analysis_and_evidence_outputs_match_shared_executor_across_restart() {
        let directory = tempfile::tempdir().unwrap();
        let database = directory.path().join("analysis.sqlite");
        let snapshot_id = seed_operation_database(&database, StructuralModel::empty());
        let request = OperationRequest {
            contract_version: OPERATION_CONTRACT_VERSION.into(),
            request_id: "analyse".into(),
            operation: Operation::AnalyseSnapshot {
                revision_id: RevisionId::from("root"),
                expected_snapshot_id: snapshot_id,
                evidence_id: fraia_revision::EvidenceId::from("evidence"),
                settings: AnalysisSettings::frame3d(),
            },
        };
        let (exit, first_bytes, first) = run_json_operation(&database, &request);
        assert_eq!(exit, EXIT_RUNTIME_UNAVAILABLE);
        assert_eq!(first["result"]["type"], "snapshot_analysed");
        assert_eq!(first["result"]["run"]["outcome"]["status"], "unsupported");
        assert!(first["result"]["run"]["evidence"]["analysis_manifest"]["metrics"].is_null());

        let (replay_exit, replay_bytes, replay) = run_json_operation(&database, &request);
        assert_eq!(replay_exit, EXIT_RUNTIME_UNAVAILABLE);
        assert_eq!(replay_bytes, first_bytes);
        assert_eq!(replay, first);

        let (inspect_exit, _, inspected) = run_json_operation(
            &database,
            &OperationRequest {
                contract_version: OPERATION_CONTRACT_VERSION.into(),
                request_id: "inspect-evidence".into(),
                operation: Operation::InspectAnalysisEvidence {
                    evidence_id: fraia_revision::EvidenceId::from("evidence"),
                    against_revision_id: RevisionId::from("root"),
                },
            },
        );
        assert_eq!(inspect_exit, EXIT_SUCCESS);
        assert_eq!(inspected["result"]["type"], "analysis_evidence_inspection");
        assert_eq!(inspected["result"]["staleness"]["status"], "current");
    }

    #[test]
    fn managed_design_operation_publishes_and_returns_the_canonical_run_id() {
        let directory = tempfile::tempdir().unwrap();
        let project = directory.path().join("project");
        let package = fraia_core::create_named_project_package(&project, "CLI run").unwrap();
        let design_id = package.designs[0].manifest.id.clone();
        let database = fraia_core::design_package_paths(&project, &design_id)
            .unwrap()
            .workspace_database;
        let snapshot_id = seed_operation_database_for_project(
            &database,
            StructuralModel::empty(),
            design_id.as_str(),
        );
        let (exit, _, value) = run_json_operation(
            &database,
            &OperationRequest {
                contract_version: OPERATION_CONTRACT_VERSION.into(),
                request_id: "managed-analysis".into(),
                operation: Operation::AnalyseSnapshot {
                    revision_id: RevisionId::from("root"),
                    expected_snapshot_id: snapshot_id,
                    evidence_id: fraia_revision::EvidenceId::from("managed-evidence"),
                    settings: AnalysisSettings::frame3d(),
                },
            },
        );
        assert_eq!(exit, EXIT_RUNTIME_UNAVAILABLE);
        let run_id = value["result"]["run"]["canonical_run_id"]
            .as_str()
            .expect("canonical run id");
        assert_eq!(
            value["result"]["run"]["evidence"]["analysis_manifest"]["canonical_run_id"],
            run_id
        );
        assert_eq!(
            list_design_runs(&project, &design_id).unwrap().runs[0].run_id,
            run_id
        );
    }

    #[test]
    fn failed_analysis_is_a_truthful_successful_adapter_response() {
        let directory = tempfile::tempdir().unwrap();
        let database = directory.path().join("failed-analysis.sqlite");
        let snapshot_id = seed_operation_database(&database, StructuralModel::empty());
        let (exit, _, value) = run_json_operation(
            &database,
            &OperationRequest {
                contract_version: OPERATION_CONTRACT_VERSION.into(),
                request_id: "failed-analysis".into(),
                operation: Operation::AnalyseSnapshot {
                    revision_id: RevisionId::from("root"),
                    expected_snapshot_id: snapshot_id,
                    evidence_id: fraia_revision::EvidenceId::from("failed-evidence"),
                    settings: AnalysisSettings::frame2d(),
                },
            },
        );
        assert_eq!(exit, EXIT_SUCCESS);
        assert_eq!(value["result"]["run"]["outcome"]["status"], "failed");
        let manifest = &value["result"]["run"]["evidence"]["analysis_manifest"];
        assert!(manifest["metrics"].is_null());
        assert!(manifest["result_hash"].is_null());
        assert!(!manifest["diagnostics"].as_array().unwrap().is_empty());
    }

    #[test]
    fn newline_delimited_batch_emits_one_stable_response_per_request() {
        let directory = std::env::temp_dir().join(format!(
            "fraia-cli-operation-batch-{}",
            fraia_core::utils::timestamp_id()
        ));
        fs::create_dir_all(&directory).unwrap();
        let database = directory.join("batch.sqlite");
        let first = serde_json::json!({
            "contractVersion": "fraia.operations.v1",
            "requestId": "cap-1",
            "operation": "capabilities"
        });
        let second = serde_json::json!({
            "contractVersion": "fraia.operations.v1",
            "requestId": "cap-2",
            "operation": "capabilities"
        });
        let input = format!("{}\n{}\n", first, second);
        let mut output = Vec::new();
        let exit = run_machine_command(
            &[
                "operation".into(),
                "--database".into(),
                database.to_string_lossy().into_owned(),
                "--batch".into(),
            ],
            &mut input.as_bytes(),
            &mut output,
        );
        assert_eq!(exit, EXIT_SUCCESS);
        let lines = String::from_utf8(output).unwrap();
        let values = lines
            .lines()
            .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(values.len(), 2);
        assert_eq!(values[0]["requestId"], "cap-1");
        assert_eq!(values[1]["requestId"], "cap-2");
        assert_eq!(values[0]["result"], values[1]["result"]);

        let input_path = directory.join("requests.ndjson");
        fs::write(&input_path, input).unwrap();
        let mut file_output = Vec::new();
        let file_exit = run_machine_command(
            &[
                "operation".into(),
                "--database".into(),
                database.to_string_lossy().into_owned(),
                "--input".into(),
                input_path.to_string_lossy().into_owned(),
                "--batch".into(),
            ],
            &mut std::io::empty(),
            &mut file_output,
        );
        assert_eq!(file_exit, EXIT_SUCCESS);
        assert_eq!(file_output, lines.as_bytes());
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn design_run_commands_read_the_canonical_core_index_without_adapter_state() {
        let directory = tempfile::tempdir().unwrap();
        let project = directory.path().join("project");
        let package = fraia_core::create_named_project_package(&project, "CLI runs").unwrap();
        let design_id = package.designs[0].manifest.id.clone();
        let run = fraia_core::publish_design_run(
            &project,
            fraia_core::PublishDesignRunRequest {
                project_id: package.manifest.id,
                design_id: design_id.clone(),
                parent_run_id: None,
                created_at: "2026-08-13T04:00:00Z".into(),
                actor: fraia_core::DesignRunActor {
                    actor_type: "cli_test".into(),
                    actor_id: "fixture".into(),
                },
                run_kind: "frame3d_analysis".into(),
                authored_revision_id: "revision-1".into(),
                authored_snapshot_id: "snapshot-1".into(),
                resolved_snapshot_id: None,
                request: serde_json::json!({"analysis":"frame3d"}),
                settings: serde_json::json!({"version":1}),
                solver_identity: "fraia.frame3d.unavailable.v1".into(),
                runtime_identity: "fraia.runtime.v1".into(),
                input_identity: None,
                result_identity: None,
                status: fraia_core::DesignRunStatus::Unsupported,
                diagnostics: vec![fraia_core::DesignRunDiagnostic {
                    severity: fraia_core::DesignRunDiagnosticSeverity::Warning,
                    code: "solver.unsupported".into(),
                    message: "No reviewed solver supports this request.".into(),
                }],
                metrics: None,
                attachments: Vec::new(),
            },
        )
        .unwrap();
        let cli_list = cli_design_runs_list(&project, &design_id).unwrap();
        assert_eq!(cli_list, list_design_runs(&project, &design_id).unwrap());
        assert_eq!(
            cli_design_run_inspect(&project, &design_id, &run.run_id).unwrap(),
            inspect_design_run(&project, &design_id, &run.run_id).unwrap()
        );
        assert_eq!(
            cli_design_run_statuses(&project, &design_id, "snapshot-1", &[]).unwrap(),
            fraia_core::list_design_run_statuses(&project, &design_id, "snapshot-1", &[]).unwrap()
        );
    }
}
