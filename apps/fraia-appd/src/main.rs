use anyhow::{Context, Result, anyhow};
use axum::{
    Json, Router,
    extract::{Query, Request, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
#[cfg(test)]
use fraia_app_api::AgentReasoningOption;
use fraia_app_api::{
    AgentApplyReviewRequest, AgentCatalogueFreshness, AgentCoordinatorProposal,
    AgentCoordinatorRequest, AgentCoordinatorResponse, AgentCoordinatorTarget, AgentModelOption,
    AgentProposedAction, AgentProviderDescriptor, AgentProviderStatusRequest,
    AgentProviderStatusResponse, AgentReviewReplyRequest, AgentReviewReplyResponse,
    AgentSessionCancelRequest, AgentSessionRespondRequest, AgentSessionStartRequest,
    AgentSettingsUpdateRequest, AnalysisReadiness, AnalysisRunSummary, AppHealthResponse,
    CoordinationGroup, CoordinationReport, CreateProjectRequest, DesignOptionAnalysisRequest,
    DesignOptionDecisionUpdateRequest, DesignOptionIntent, DesignScheme, DesignSchemeGroupChoice,
    DesignSchemeSectionCandidate, ErrorResponse, PlanningAnalysisBrief as ApiPlanningAnalysisBrief,
    PlanningDesignConstraints as ApiPlanningDesignConstraints, PlanningDraft as ApiPlanningDraft,
    PlanningDraftRequest, PlanningGeometryAndLoads as ApiPlanningGeometryAndLoads,
    PlanningProjectIntent as ApiPlanningProjectIntent,
    PlanningSystemBrief as ApiPlanningSystemBrief, ProjectPathRequest, SceneBounds, SceneLoad,
    SceneMember, SceneNode, ScenePlate, SceneRelease, SceneSectionCoordination,
    SceneSizeCoordination, SceneSupport, SummaryArtifactRef, WorkbenchDiagnostic,
    WorkbenchOperationResponse, WorkbenchProjectOverview, WorkbenchProjectState, WorkbenchScene,
};
use fraia_core::{
    AgentMessage, AgentModelSettings, AgentProposedActionState, AgentSession, AgentState,
    AgentSuggestedReplyGroup, AiProvenance, AssignmentTargetRef, BaseModelBrief,
    BaseModelBriefLoadDirection, BaseModelBriefLoadTarget, BaseModelBriefReadiness,
    BaseModelBriefVisualIntent, CalculixCompiledInput, CalculixExecutionArtifacts,
    CalculixExecutionOutcome, DesignOptionBatch, DesignOptionComparisonEvidenceReference,
    DesignOptionComparisonRun, DesignOptionRevision, DevelopmentPath, Force,
    FrameElementStressSummary, FrameModel2D, FrameNodeDisplacementPoint, FrameSupportReactionPoint,
    LineLoad, LoadAssignment, LoadCase2D, LoadKind, LoadVector,
    PlanningAnalysisBrief as CorePlanningAnalysisBrief,
    PlanningDesignConstraints as CorePlanningDesignConstraints, PlanningDraft as CorePlanningDraft,
    PlanningGeometryAndLoads as CorePlanningGeometryAndLoads,
    PlanningProjectIntent as CorePlanningProjectIntent,
    PlanningSystemBrief as CorePlanningSystemBrief, ProjectFile, QuantityKind, Stress,
    StructuralMember, StructuralModel, StructuralNode, SupportAssignment,
    analyze_current_simply_supported_beam_project, apply_planning_draft, calculix_runtime_status,
    canonical_value_from_unit, compile_frame_model_to_calculix_input, create_project,
    current_simply_supported_beam_builder_params, default_planning_markdown,
    derive_conservative_check_report, derive_design_action_report,
    execute_calculix_compiled_input_with_runtime, execute_current_frame_project_in_calculix,
    extract_frame_calculix_dat, format_quantity, frame2d::solve_frame_2d, load_project,
    materialize_project_structural_model, materialize_structural_model_from_builder_graph,
    metric_structural_unit_profile, parse_quantity, planning_draft, portal_frame_builder_graph,
    realize_structural_model_to_frame2d, render_beam_analysis_summary, render_beam_sizing_summary,
    render_frame_calculix_execution_summary, render_validation_summary, require_calculix_runtime,
    save_project, section_by_id, section_catalog, section_family,
    seed_simply_supported_beam_in_project, simply_supported_beam_builder_graph,
    size_current_simply_supported_beam_in_project, steel_material, understand_structural_model,
    update_planning_markdown, validate_structural_model,
};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

const AGENT_JUSTIFIED_SECTION_SELECTION_POLICY: &str = "agent_justified";

const DEFAULT_AI_TURN_TIMEOUT_SECONDS: u64 = 125;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PiCatalogueResponse {
    #[serde(default)]
    providers: Vec<AgentProviderDescriptor>,
    #[serde(default)]
    models: Vec<AgentModelOption>,
    #[serde(default)]
    catalogue: AgentCatalogueFreshness,
    #[serde(default)]
    secure_credential_storage_available: bool,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct PiTurnRequest<'a> {
    request_id: &'a str,
    provider_id: &'a str,
    model_id: &'a str,
    reasoning_effort: &'a str,
    prompt: &'a str,
    response_schema: &'a Value,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PiTurnResponse {
    output: Value,
    provider_id: String,
    model_id: String,
    reasoning_effort: String,
    #[serde(default)]
    catalogue_refreshed_at: Option<String>,
}

fn ai_runtime_url() -> Result<String> {
    std::env::var("FRAIA_AI_URL")
        .context("Fraia's Pi AI runtime is unavailable; restart the desktop application")
}

fn ai_runtime_token() -> Result<String> {
    std::env::var("FRAIA_AI_TOKEN")
        .context("Fraia's Pi AI runtime authentication token is unavailable")
}

fn ai_runtime_client() -> Result<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(DEFAULT_AI_TURN_TIMEOUT_SECONDS))
        .build()
        .context("failed to create the Fraia AI runtime client")
}

fn pi_catalogue() -> Result<PiCatalogueResponse> {
    run_on_blocking_thread(pi_catalogue_blocking)
}

fn run_on_blocking_thread<T: Send>(operation: impl FnOnce() -> Result<T> + Send) -> Result<T> {
    std::thread::scope(|scope| {
        scope
            .spawn(operation)
            .join()
            .map_err(|_| anyhow!("Fraia's Pi runtime bridge thread panicked"))?
    })
}

fn pi_catalogue_blocking() -> Result<PiCatalogueResponse> {
    let response = ai_runtime_client()?
        .get(format!(
            "{}/v1/catalog",
            ai_runtime_url()?.trim_end_matches('/')
        ))
        .bearer_auth(ai_runtime_token()?)
        .send()
        .context("failed to contact Fraia's Pi model runtime")?;
    let status = response.status();
    let body = response
        .text()
        .context("failed to read Pi catalogue response")?;
    if !status.is_success() {
        let detail = serde_json::from_str::<Value>(&body)
            .ok()
            .and_then(|value| {
                value
                    .get("error")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            })
            .unwrap_or(body);
        return Err(anyhow!("Pi catalogue request failed: {detail}"));
    }
    serde_json::from_str(&body).context("Pi returned an invalid catalogue response")
}

fn run_pi_turn<T: DeserializeOwned + Send>(
    request_id: &str,
    settings: &AgentModelSettings,
    prompt: &str,
    response_schema: &Value,
) -> Result<(T, PiTurnResponse)> {
    run_on_blocking_thread(|| run_pi_turn_blocking(request_id, settings, prompt, response_schema))
}

fn run_pi_turn_blocking<T: DeserializeOwned>(
    request_id: &str,
    settings: &AgentModelSettings,
    prompt: &str,
    response_schema: &Value,
) -> Result<(T, PiTurnResponse)> {
    let response = ai_runtime_client()?
        .post(format!(
            "{}/v1/turns",
            ai_runtime_url()?.trim_end_matches('/')
        ))
        .bearer_auth(ai_runtime_token()?)
        .json(&PiTurnRequest {
            request_id,
            provider_id: &settings.provider_id,
            model_id: &settings.model,
            reasoning_effort: &settings.reasoning_effort,
            prompt,
            response_schema,
        })
        .send()
        .context("failed to contact Fraia's Pi inference runtime")?;
    let status = response.status();
    let body = response.text().context("failed to read Pi turn response")?;
    if !status.is_success() {
        let detail = serde_json::from_str::<Value>(&body)
            .ok()
            .and_then(|value| {
                value
                    .get("error")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            })
            .unwrap_or(body);
        return Err(anyhow!("Pi turn failed: {detail}"));
    }
    let envelope: PiTurnResponse =
        serde_json::from_str(&body).context("Pi returned an invalid turn envelope")?;
    let typed = serde_json::from_value(envelope.output.clone())
        .context("Pi structured output failed Fraia's Rust type validation")?;
    Ok((typed, envelope))
}

fn cancel_pi_turn(request_id: &str) -> Result<bool> {
    run_on_blocking_thread(|| cancel_pi_turn_blocking(request_id))
}

fn cancel_pi_turn_blocking(request_id: &str) -> Result<bool> {
    let encoded = request_id.replace('%', "%25").replace('/', "%2F");
    let response = ai_runtime_client()?
        .delete(format!(
            "{}/v1/turns/{encoded}",
            ai_runtime_url()?.trim_end_matches('/')
        ))
        .bearer_auth(ai_runtime_token()?)
        .send()
        .context("failed to cancel the Pi turn")?;
    if !response.status().is_success() {
        return Err(anyhow!(
            "Pi cancellation request failed with {}",
            response.status()
        ));
    }
    let body: Value = response
        .json()
        .context("Pi returned an invalid cancellation response")?;
    Ok(body
        .get("cancelled")
        .and_then(Value::as_bool)
        .unwrap_or(false))
}

#[tokio::main]
async fn main() -> Result<()> {
    let port = parse_port(std::env::args().skip(1))?;
    let appd_token = std::env::var("FRAIA_APPD_TOKEN")
        .context("FRAIA_APPD_TOKEN is required to authenticate the desktop application")?;
    if appd_token.len() < 32 {
        return Err(anyhow!(
            "FRAIA_APPD_TOKEN must contain at least 32 characters"
        ));
    }
    let appd_token: Arc<str> = Arc::from(appd_token);
    let app = Router::new()
        .route("/health", get(health))
        .route("/projects/create", post(create_project_handler))
        .route("/projects/open", post(open_project_handler))
        .route("/projects/state", get(project_state_handler))
        .route(
            "/projects/planning-draft",
            post(save_planning_draft_handler),
        )
        .route(
            "/projects/materialize-planning",
            post(materialize_planning_handler),
        )
        .route("/projects/analyse", post(analyse_planning_handler))
        .route(
            "/projects/design-option-analysis",
            post(design_option_analysis_handler),
        )
        .route(
            "/projects/design-option-analysis/raw",
            get(raw_design_option_analysis_handler),
        )
        .route(
            "/projects/design-options/decision",
            post(design_option_decision_handler),
        )
        .route(
            "/schemas/base-model-handoff",
            post(schema_base_model_handoff_handler),
        )
        .route("/agent/review-reply", post(agent_review_reply_handler))
        .route(
            "/agent/design-options/generate",
            post(agent_design_options_generate_handler),
        )
        .route(
            "/agent/provider-status",
            post(agent_provider_status_handler),
        )
        .route("/agent/settings", post(agent_settings_handler))
        .route(
            "/agent/base-model-guide/reset",
            post(agent_base_model_guide_reset_handler),
        )
        .route("/agent/sessions/start", post(agent_session_start_handler))
        .route(
            "/agent/sessions/respond",
            post(agent_session_respond_handler),
        )
        .route("/agent/sessions/cancel", post(agent_session_cancel_handler))
        .route(
            "/agent/pre-solve-coordinator",
            post(agent_pre_solve_coordinator_handler),
        )
        .route("/agent/apply-review", post(agent_apply_review_handler))
        .route("/projects/base-model/edit", post(base_model_edit_handler))
        .route("/projects/seed-frame-demo", post(seed_frame_demo_handler))
        .route(
            "/projects/seed-frame-review-demo",
            post(seed_frame_review_demo_handler),
        )
        .route("/projects/seed-beam-demo", post(seed_beam_demo_handler))
        .route("/projects/beam-size", post(beam_size_handler))
        .route("/projects/validate", post(validate_handler))
        .route(
            "/projects/frame-run-calculix",
            post(frame_run_calculix_handler),
        )
        .layer(middleware::from_fn_with_state(
            appd_token,
            require_appd_auth,
        ));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind Fraia app service on {addr}"))?;
    println!("Fraia app service listening on http://{addr}");
    axum::serve(listener, app)
        .await
        .context("Fraia app service exited unexpectedly")?;
    Ok(())
}

async fn require_appd_auth(
    State(expected_token): State<Arc<str>>,
    request: Request,
    next: Next,
) -> Response {
    let authorised = request_is_authorised(request.headers(), &expected_token);
    if !authorised {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    next.run(request).await
}

fn request_is_authorised(headers: &HeaderMap, expected_token: &str) -> bool {
    headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .is_some_and(|token| constant_time_equal(token.as_bytes(), expected_token.as_bytes()))
}

fn constant_time_equal(left: &[u8], right: &[u8]) -> bool {
    let mut difference = left.len() ^ right.len();
    let length = left.len().max(right.len());
    for index in 0..length {
        let left_byte = left.get(index).copied().unwrap_or(0);
        let right_byte = right.get(index).copied().unwrap_or(0);
        difference |= usize::from(left_byte ^ right_byte);
    }
    difference == 0
}

async fn health() -> Json<AppHealthResponse> {
    Json(AppHealthResponse {
        status: "ok".into(),
        api_version: "v0".into(),
        calculix_runtime: calculix_runtime_status(),
    })
}

async fn create_project_handler(
    Json(request): Json<CreateProjectRequest>,
) -> Result<Json<WorkbenchOperationResponse>, ApiError> {
    let dir = PathBuf::from(request.project_dir);
    let name = request.name.unwrap_or_else(|| {
        dir.file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("Fraia Project")
            .to_owned()
    });
    let (project, paths) = create_project(&dir, &name)
        .with_context(|| format!("failed to create project at {}", dir.display()))?;
    update_planning_markdown(&dir, &default_planning_markdown(&project)).with_context(|| {
        format!(
            "failed to initialise planning markdown at {}",
            paths.planning_file.display()
        )
    })?;
    let state = build_workbench_state(&dir, &project)?;
    Ok(Json(WorkbenchOperationResponse {
        message: format!("Created project at {}", dir.display()),
        state,
    }))
}

async fn open_project_handler(
    Json(request): Json<ProjectPathRequest>,
) -> Result<Json<WorkbenchOperationResponse>, ApiError> {
    let dir = PathBuf::from(request.project_dir);
    let (project, _) = load_project(&dir)
        .with_context(|| format!("failed to load project from {}", dir.display()))?;
    let state = build_workbench_state(&dir, &project)?;
    Ok(Json(WorkbenchOperationResponse {
        message: format!("Loaded project from {}", dir.display()),
        state,
    }))
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectStateQuery {
    project_dir: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawDesignOptionAnalysisQuery {
    project_dir: String,
    #[serde(default)]
    run_id: Option<String>,
}

async fn project_state_handler(
    Query(query): Query<ProjectStateQuery>,
) -> Result<Json<WorkbenchProjectState>, ApiError> {
    let dir = PathBuf::from(query.project_dir);
    let (project, _) = load_project(&dir)
        .with_context(|| format!("failed to load project from {}", dir.display()))?;
    Ok(Json(build_workbench_state(&dir, &project)?))
}

async fn raw_design_option_analysis_handler(
    Query(query): Query<RawDesignOptionAnalysisQuery>,
) -> Result<Json<Value>, ApiError> {
    let dir = PathBuf::from(query.project_dir);
    let run_dir = raw_design_option_analysis_run_dir(&dir, query.run_id.as_deref())?;
    Ok(Json(load_raw_design_option_analysis(&run_dir)?))
}

async fn design_option_decision_handler(
    Json(request): Json<DesignOptionDecisionUpdateRequest>,
) -> Result<Json<WorkbenchOperationResponse>, ApiError> {
    let dir = PathBuf::from(&request.project_dir);
    let (mut project, _) = load_project(&dir)
        .with_context(|| format!("failed to load project from {}", dir.display()))?;
    ensure_active_design_option_batch(&mut project);
    refresh_design_option_batch_freshness(&mut project);
    sync_active_design_option_revisions(&mut project);
    let evidence_state = build_workbench_state(&dir, &project)?;
    sync_decision_analysis_evidence(
        &mut project.design_option_decisions,
        &evidence_state.design_schemes,
        evidence_state
            .latest_design_option_analysis
            .as_ref()
            .map(|artifact| artifact.run_id.as_str()),
    );

    let active_batch_id = project
        .design_option_decisions
        .active_batch_id
        .clone()
        .ok_or_else(|| anyhow!("no active design-option batch is available"))?;
    let active_batch_index = project
        .design_option_decisions
        .batches
        .iter()
        .position(|batch| batch.id == active_batch_id)
        .ok_or_else(|| anyhow!("active design-option batch `{active_batch_id}` does not exist"))?;
    if project.design_option_decisions.batches[active_batch_index].status != "active" {
        return Err(anyhow!(
            "the current design-option batch is outdated; regenerate options from the current Base Model"
        )
        .into());
    }

    let option_id = request.option_id.as_deref().unwrap_or_default();
    let message = match request.action.as_str() {
        "set_included" => {
            let included = request
                .included
                .ok_or_else(|| anyhow!("set_included requires an included value"))?;
            let revision = project.design_option_decisions.batches[active_batch_index]
                .option_revisions
                .iter_mut()
                .find(|revision| revision.option_id == option_id)
                .ok_or_else(|| anyhow!("design option `{option_id}` is not in the active batch"))?;
            revision.included = included;
            format!(
                "{} {} in comparison.",
                if included { "Included" } else { "Excluded" },
                revision.label
            )
        }
        "develop" => {
            let revision = project.design_option_decisions.batches[active_batch_index]
                .option_revisions
                .iter()
                .find(|revision| revision.option_id == option_id)
                .cloned()
                .ok_or_else(|| anyhow!("design option `{option_id}` is not in the active batch"))?;
            if !revision.included {
                return Err(anyhow!("include this design option before developing it").into());
            }
            if revision.analysis_status != "current" {
                return Err(anyhow!(
                    "run a successful current preliminary analysis before developing this option"
                )
                .into());
            }
            if revision.latest_analysis_run_id.is_none() {
                return Err(anyhow!(
                    "current analysis evidence is missing its immutable run reference; refresh the comparison or rerun this option"
                )
                .into());
            }
            let now = fraia_core::utils::iso_now();
            let existing_index = project
                .design_option_decisions
                .development_paths
                .iter()
                .position(|path| path.option_revision_id == revision.revision_id);
            let path_id = if let Some(index) = existing_index {
                let path = &mut project.design_option_decisions.development_paths[index];
                path.status = "active".into();
                path.updated_at = now;
                path.source_analysis_run_id = revision.latest_analysis_run_id.clone();
                path.id.clone()
            } else {
                let id = format!(
                    "development-{}-{}",
                    safe_identifier(&revision.option_id),
                    fraia_core::utils::timestamp_id()
                );
                project
                    .design_option_decisions
                    .development_paths
                    .push(DevelopmentPath {
                        id: id.clone(),
                        option_id: revision.option_id.clone(),
                        option_revision_id: revision.revision_id.clone(),
                        status: "active".into(),
                        created_at: now.clone(),
                        updated_at: now,
                        source_analysis_run_id: revision.latest_analysis_run_id.clone(),
                    });
                id
            };
            for path in &mut project.design_option_decisions.development_paths {
                if path.id != path_id && path.status == "active" {
                    path.status = "available".into();
                }
            }
            project.design_option_decisions.active_development_path_id = Some(path_id);
            format!("Opened {} for development.", revision.label)
        }
        "refresh_comparison" => {
            let included = project.design_option_decisions.batches[active_batch_index]
                .option_revisions
                .iter()
                .filter(|revision| revision.included)
                .collect::<Vec<_>>();
            if included.is_empty() {
                return Err(anyhow!("include at least one design option before comparison").into());
            }
            if included.iter().any(|revision| {
                revision.analysis_status != "current" || revision.latest_analysis_run_id.is_none()
            }) {
                return Err(anyhow!(
                    "analyse every included design option before refreshing the comparison"
                )
                .into());
            }
            "Refreshed the comparison from current immutable analysis evidence.".into()
        }
        other => return Err(anyhow!("unsupported design-option decision action `{other}`").into()),
    };

    if request.action == "set_included" || request.action == "refresh_comparison" {
        let state = build_workbench_state(&dir, &project)?;
        let comparison_id = format!(
            "design-option-comparison-{}",
            fraia_core::utils::timestamp_id()
        );
        record_design_option_comparison_run(
            &mut project,
            &comparison_id,
            &[],
            &state.design_schemes,
        );
    }

    project.updated_at = Some(fraia_core::utils::iso_now());
    persist_project_and_markdown(&dir, &project)?;
    Ok(Json(WorkbenchOperationResponse {
        message,
        state: build_workbench_state(&dir, &project)?,
    }))
}

async fn save_planning_draft_handler(
    Json(request): Json<PlanningDraftRequest>,
) -> Result<Json<WorkbenchOperationResponse>, ApiError> {
    let dir = PathBuf::from(&request.project_dir);
    let (mut project, _) = load_project(&dir)
        .with_context(|| format!("failed to load project from {}", dir.display()))?;
    let draft = api_planning_to_core(request.draft);
    apply_planning_draft(&mut project, draft);
    persist_project_and_markdown(&dir, &project)?;
    let state = build_workbench_state(&dir, &project)?;
    Ok(Json(WorkbenchOperationResponse {
        message: "Saved planning draft.".into(),
        state,
    }))
}

async fn materialize_planning_handler(
    Json(request): Json<PlanningDraftRequest>,
) -> Result<Json<WorkbenchOperationResponse>, ApiError> {
    let dir = PathBuf::from(&request.project_dir);
    let (mut project, _) = load_project(&dir)
        .with_context(|| format!("failed to load project from {}", dir.display()))?;
    let draft = api_planning_to_core(request.draft);
    apply_planning_draft(&mut project, draft);

    if let Some(structural) = project.structural_model.as_ref() {
        let structural_readiness = evaluate_structural_solve_readiness(structural);
        if structural_readiness.status != "ready" {
            persist_project_and_markdown(&dir, &project)?;
            let mut state = build_workbench_state(&dir, &project)?;
            state.analysis_readiness = Some(structural_readiness.clone());
            state.latest_run_summary = Some(AnalysisRunSummary {
                status: "not_run".into(),
                analysis_kind: "analysis".into(),
                message: structural_readiness.summary.clone(),
                run_id: None,
            });
            return Ok(Json(WorkbenchOperationResponse {
                message: structural_readiness.summary,
                state,
            }));
        }
    }

    let materialize = materialize_current_planning(&mut project)?;
    persist_project_and_markdown(&dir, &project)?;

    let mut state = build_workbench_state(&dir, &project)?;
    state.latest_run_summary = Some(materialize.run_summary.clone());
    Ok(Json(WorkbenchOperationResponse {
        message: materialize.message,
        state,
    }))
}

async fn analyse_planning_handler(
    Json(request): Json<PlanningDraftRequest>,
) -> Result<Json<WorkbenchOperationResponse>, ApiError> {
    let dir = PathBuf::from(&request.project_dir);
    let (mut project, _) = load_project(&dir)
        .with_context(|| format!("failed to load project from {}", dir.display()))?;
    let draft = api_planning_to_core(request.draft);
    apply_planning_draft(&mut project, draft);

    let pending_decisions = pending_scheme_analysis_decisions(&project);
    if !pending_decisions.is_empty() {
        persist_project_and_markdown(&dir, &project)?;
        let mut state = build_workbench_state(&dir, &project)?;
        let detail = pending_decisions
            .iter()
            .map(|decision| format!("{}: {}", decision.scheme_id, decision.prompt))
            .collect::<Vec<_>>()
            .join("\n");
        merge_diagnostic(
            &mut state,
            WorkbenchDiagnostic {
                severity: "warning".into(),
                code: "analysis.design_option_decision_required".into(),
                message: "Resolve pending design-option decisions before running analysis.".into(),
                detail: Some(detail),
            },
        );
        state.latest_run_summary = Some(AnalysisRunSummary {
            status: "not_run".into(),
            analysis_kind: "analysis".into(),
            message: "Resolve pending design-option decisions before running analysis.".into(),
            run_id: None,
        });
        return Ok(Json(WorkbenchOperationResponse {
            message: "Resolve pending design-option decisions before running analysis.".into(),
            state,
        }));
    }

    let materialize = materialize_current_planning(&mut project)?;
    if !materialize.can_analyse {
        persist_project_and_markdown(&dir, &project)?;
        let mut state = build_workbench_state(&dir, &project)?;
        state.latest_run_summary = Some(materialize.run_summary);
        return Ok(Json(WorkbenchOperationResponse {
            message: materialize.message,
            state,
        }));
    }

    let analysis = match dispatch_analysis(&dir, &mut project) {
        Ok(analysis) => analysis,
        Err(error) => {
            persist_project_and_markdown(&dir, &project)?;
            let mut state = build_workbench_state(&dir, &project)?;
            let failure = WorkbenchDiagnostic {
                severity: "error".into(),
                code: "analysis.run_failed".into(),
                message: "Analysis run failed.".into(),
                detail: Some(format!("{error:#}")),
            };
            merge_diagnostic(&mut state, failure);
            state.latest_run_summary = Some(AnalysisRunSummary {
                status: "failed".into(),
                analysis_kind: "analysis".into(),
                message: "Analysis run failed.".into(),
                run_id: None,
            });
            return Ok(Json(WorkbenchOperationResponse {
                message: "Analysis run failed.".into(),
                state,
            }));
        }
    };

    let (reloaded, _) =
        load_project(&dir).with_context(|| format!("failed to reload {}", dir.display()))?;
    let mut state = build_workbench_state(&dir, &reloaded)?;
    state.latest_run_summary = Some(analysis.run_summary.clone());
    Ok(Json(WorkbenchOperationResponse {
        message: analysis.message,
        state,
    }))
}

async fn design_option_analysis_handler(
    Json(mut request): Json<DesignOptionAnalysisRequest>,
) -> Result<Json<WorkbenchOperationResponse>, ApiError> {
    let dir = PathBuf::from(&request.project_dir);
    let (mut project, _) = load_project(&dir)
        .with_context(|| format!("failed to load project from {}", dir.display()))?;
    ensure_active_design_option_batch(&mut project);
    refresh_design_option_batch_freshness(&mut project);
    sync_active_design_option_revisions(&mut project);
    let active_batch = project
        .design_option_decisions
        .active_batch_id
        .as_ref()
        .and_then(|id| {
            project
                .design_option_decisions
                .batches
                .iter()
                .find(|batch| &batch.id == id)
        })
        .ok_or_else(|| anyhow!("no active design-option batch is available"))?;
    if active_batch.status != "active" {
        return Err(anyhow!(
            "the current design-option batch is outdated; regenerate options before analysis"
        )
        .into());
    }
    let included_ids = active_batch
        .option_revisions
        .iter()
        .filter(|revision| revision.included)
        .map(|revision| revision.option_id.clone())
        .collect::<Vec<_>>();
    if included_ids.is_empty() {
        return Err(anyhow!("include at least one design option before analysis").into());
    }
    let requested_ids = request
        .scope
        .as_ref()
        .map(|scope| scope.option_ids.clone())
        .unwrap_or_default();
    if requested_ids.is_empty() {
        request.scope = Some(fraia_app_api::DesignOptionAnalysisScope {
            kind: "selected_design_options".into(),
            option_ids: included_ids.clone(),
        });
    } else if requested_ids
        .iter()
        .any(|option_id| !included_ids.contains(option_id))
    {
        return Err(anyhow!("analysis scope contains an excluded design option").into());
    }
    let run_dir = persist_design_option_analysis_run(&dir, &project, &request)?;
    let run_id = run_dir
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_owned);
    let analysed_ids = request
        .scope
        .as_ref()
        .map(|scope| scope.option_ids.clone())
        .unwrap_or_else(|| included_ids.clone());
    if let Some(run_id) = run_id.as_deref()
        && let Some(batch) = project
            .design_option_decisions
            .batches
            .iter_mut()
            .find(|batch| {
                Some(&batch.id) == project.design_option_decisions.active_batch_id.as_ref()
            })
    {
        for revision in &mut batch.option_revisions {
            if analysed_ids.contains(&revision.option_id) {
                revision.latest_analysis_run_id = Some(run_id.to_owned());
            }
        }
    }
    let mut state = build_workbench_state(&dir, &project)?;
    if let Some(run_id) = run_id.as_deref() {
        record_design_option_comparison_run(
            &mut project,
            run_id,
            &analysed_ids,
            &state.design_schemes,
        );
        persist_project_and_markdown(&dir, &project)?;
        state = build_workbench_state(&dir, &project)?;
    }
    state.latest_run_summary = Some(AnalysisRunSummary {
        status: "completed".into(),
        analysis_kind: "design_option_analysis".into(),
        message: "Analysed design-option candidate sections with CalculiX.".into(),
        run_id: run_id.clone(),
    });
    Ok(Json(WorkbenchOperationResponse {
        message: format!(
            "Analysed design options. Artefacts saved to {}",
            run_dir.display()
        ),
        state,
    }))
}

async fn agent_review_reply_handler(
    Json(request): Json<AgentReviewReplyRequest>,
) -> Result<Json<AgentReviewReplyResponse>, ApiError> {
    let dir = PathBuf::from(&request.project_dir);
    let (project, _) = load_project(&dir)
        .with_context(|| format!("failed to load project from {}", dir.display()))?;
    let response = interpret_agent_review_reply(&project, &request);
    Ok(Json(response))
}

async fn schema_base_model_handoff_handler(
    Json(request): Json<ProjectPathRequest>,
) -> Result<Json<WorkbenchOperationResponse>, ApiError> {
    let dir = PathBuf::from(&request.project_dir);
    let (mut project, _) = load_project(&dir)
        .with_context(|| format!("failed to load project from {}", dir.display()))?;
    ensure_base_model_brief(&mut project);
    let run_dir = persist_schema_handoff_snapshot(&dir, &project)?;
    persist_project_and_markdown(&dir, &project)?;
    Ok(Json(WorkbenchOperationResponse {
        message: format!(
            "Prepared design-option handoff snapshot at {}.",
            run_dir.display()
        ),
        state: build_workbench_state(&dir, &project)?,
    }))
}

async fn agent_design_options_generate_handler(
    Json(request): Json<AgentApplyReviewRequest>,
) -> Result<Json<WorkbenchOperationResponse>, ApiError> {
    let dir = PathBuf::from(&request.project_dir);
    let (mut project, _) = load_project(&dir)
        .with_context(|| format!("failed to load project from {}", dir.display()))?;
    ensure_base_model_brief(&mut project);

    let mut draft = planning_draft(&project);
    let mut proposed_actions = request.proposed_actions.clone();
    let overwriting_existing_options = !authored_design_option_intents(&draft).is_empty()
        || project
            .agent_state
            .sessions
            .iter()
            .any(|session| session.surface.starts_with("scheme:"));
    if overwriting_existing_options {
        draft.system_parameters.remove("designOptionIntents");
        apply_planning_draft(&mut project, draft.clone());
        proposed_actions.clear();
    }
    if proposed_actions.is_empty() && authored_design_option_intents(&draft).is_empty() {
        if let Some(message) = append_pi_agent_turn(
            &mut project,
            "pre_solve",
            if overwriting_existing_options {
                "Regenerate design-option intents now from the ready Base Model Brief. This replaces the existing design options. Return the reviewed coordination.designOptionIntents action for Fraia to realise; do not ask another briefing question unless the brief is genuinely not ready."
            } else {
                "Generate design-option intents now from the ready Base Model Brief. Return the reviewed coordination.designOptionIntents action for Fraia to realise; do not ask another briefing question unless the brief is genuinely not ready."
            },
            None,
        ) {
            proposed_actions = message
                .proposed_actions
                .iter()
                .map(agent_action_state_to_action)
                .collect();
            let session = ensure_agent_session(&mut project, "pre_solve");
            session.messages.push(message);
            session.updated_at = fraia_core::utils::iso_now();
        }
        draft = planning_draft(&project);
    }

    let (mut applied, mut diagnostics) =
        apply_design_option_intent_actions(&project, &mut draft, &proposed_actions);
    if !diagnostics.is_empty() && !proposed_actions.is_empty() {
        let feedback = diagnostics
            .iter()
            .map(|diagnostic| {
                diagnostic
                    .detail
                    .as_deref()
                    .unwrap_or(diagnostic.message.as_str())
            })
            .collect::<Vec<_>>()
            .join(" ");
        if let Some(message) = append_pi_agent_turn(
            &mut project,
            "pre_solve",
            &format!(
                "Regenerate design-option intents now. The previous coordination.designOptionIntents action failed deterministic validation: {feedback}. Return one corrected reviewed coordination.designOptionIntents action for Fraia to realise. Every supportStrategy must explicitly choose pinned restraint, fixed restraint, or existing authored SupportAssignment objects at confirmed support-location nodes, and provenance must explicitly cover support/restraint choice, load path or stability, section-family policy, coordination/standardisation, and connection/detailing. Do not ask another briefing question unless the brief is genuinely not ready."
            ),
            None,
        ) {
            proposed_actions = message
                .proposed_actions
                .iter()
                .map(agent_action_state_to_action)
                .collect();
            let session = ensure_agent_session(&mut project, "pre_solve");
            session.messages.push(message);
            session.updated_at = fraia_core::utils::iso_now();
            draft = planning_draft(&project);
            (applied, diagnostics) =
                apply_design_option_intent_actions(&project, &mut draft, &proposed_actions);
        }
    }

    if diagnostics.is_empty() && !applied.is_empty() {
        apply_planning_draft(&mut project, draft);
    }

    if diagnostics.is_empty()
        && authored_design_option_intents(&planning_draft(&project)).is_empty()
    {
        diagnostics.push(WorkbenchDiagnostic {
            severity: "error".into(),
            code: "agent.design_options.no_agent_intents".into(),
            message: "No agent-authored design-option intents are available yet.".into(),
            detail: Some(
                "Ask the Base Model Guide to propose design options before generating them.".into(),
            ),
        });
    }

    if !diagnostics.is_empty() {
        let mut state = build_workbench_state(&dir, &project)?;
        for diagnostic in diagnostics {
            merge_diagnostic(&mut state, diagnostic);
        }
        project.updated_at = Some(fraia_core::utils::iso_now());
        persist_project_and_markdown(&dir, &project)?;
        return Ok(Json(WorkbenchOperationResponse {
            message: "Design options were not generated.".into(),
            state,
        }));
    }

    persist_schema_handoff_snapshot(&dir, &project)?;
    project.updated_at = Some(fraia_core::utils::iso_now());
    persist_project_and_markdown(&dir, &project)?;

    let state = build_workbench_state(&dir, &project)?;
    let scheme_ids: Vec<_> = state
        .design_schemes
        .iter()
        .map(|scheme| scheme.id.clone())
        .collect();
    if scheme_ids.is_empty() {
        let mut state = state;
        merge_diagnostic(
            &mut state,
            WorkbenchDiagnostic {
                severity: "error".into(),
                code: "agent.design_options.no_realised_schemes".into(),
                message: "The agent-authored intents did not realise into design-option scenes."
                    .into(),
                detail: Some(
                    "Check that each DesignOptionIntent has a realizable support strategy and valid section-family policy."
                        .into(),
                ),
            },
        );
        return Ok(Json(WorkbenchOperationResponse {
            message: "Design options were not generated.".into(),
            state,
        }));
    }

    if overwriting_existing_options {
        archive_active_design_option_batch(&mut project);
    }
    create_active_design_option_batch(&mut project);

    let mut pi_failures = Vec::new();
    for scheme_id in scheme_ids {
        let surface = format!("scheme:{scheme_id}");
        if project
            .agent_state
            .sessions
            .iter()
            .find(|session| session.surface == surface)
            .map(|session| {
                session.messages.iter().any(|message| {
                    message.author == "assistant"
                        && message.mode.as_deref() == Some("pi")
                        && !message.text.trim().is_empty()
                })
            })
            .unwrap_or(false)
        {
            continue;
        }
        let assistant = append_pi_agent_turn(&mut project, &surface, "", None);
        if let Some(message) = assistant {
            if message.mode.as_deref() == Some("pi_unavailable") {
                pi_failures.push(scheme_id.clone());
            }
            let session = ensure_agent_session(&mut project, &surface);
            session.messages.push(message);
            session.updated_at = fraia_core::utils::iso_now();
        }
    }

    project.updated_at = Some(fraia_core::utils::iso_now());
    persist_project_and_markdown(&dir, &project)?;
    let message = if pi_failures.is_empty() && overwriting_existing_options {
        "Regenerated design options with a first-pass AI option analysis.".into()
    } else if pi_failures.is_empty() {
        "Generated design options with a first-pass AI option analysis.".into()
    } else if overwriting_existing_options {
        format!(
            "Regenerated design options, but AI option analysis failed for {}.",
            pi_failures.join(", ")
        )
    } else {
        format!(
            "Generated design options, but AI option analysis failed for {}.",
            pi_failures.join(", ")
        )
    };
    Ok(Json(WorkbenchOperationResponse {
        message,
        state: build_workbench_state(&dir, &project)?,
    }))
}

fn apply_design_option_intent_actions(
    project: &ProjectFile,
    draft: &mut CorePlanningDraft,
    proposed_actions: &[AgentProposedAction],
) -> (Vec<String>, Vec<WorkbenchDiagnostic>) {
    let mut applied = Vec::new();
    let mut diagnostics = Vec::new();
    for action in proposed_actions {
        if action.action_kind != "update_planning_draft"
            || action.field != "coordination.designOptionIntents"
        {
            diagnostics.push(WorkbenchDiagnostic {
                severity: "error".into(),
                code: "agent.design_options.unsupported_action".into(),
                message:
                    "Only agent-authored design-option intent actions can generate design options."
                        .into(),
                detail: Some(format!(
                    "Unsupported action `{}` on `{}`.",
                    action.action_kind, action.field
                )),
            });
            continue;
        }
        match apply_agent_action_to_draft(&project, &mut *draft, action) {
            Ok(summary) => applied.push(summary),
            Err(error) => diagnostics.push(WorkbenchDiagnostic {
                severity: "error".into(),
                code: "agent.design_options.intent_apply_failed".into(),
                message: "Could not apply the agent-authored design-option intents.".into(),
                detail: Some(format!("{error:#}")),
            }),
        }
    }
    (applied, diagnostics)
}

async fn agent_pre_solve_coordinator_handler(
    Json(request): Json<AgentCoordinatorRequest>,
) -> Result<Json<AgentCoordinatorResponse>, ApiError> {
    let dir = PathBuf::from(&request.project_dir);
    let (project, _) = load_project(&dir)
        .with_context(|| format!("failed to load project from {}", dir.display()))?;
    let response = interpret_agent_coordinator(&project, &request);
    Ok(Json(response))
}

async fn agent_provider_status_handler(
    Json(request): Json<AgentProviderStatusRequest>,
) -> Result<Json<AgentProviderStatusResponse>, ApiError> {
    let dir = PathBuf::from(&request.project_dir);
    let (mut project, _) = load_project(&dir)
        .with_context(|| format!("failed to load project from {}", dir.display()))?;
    if ensure_agent_settings(&mut project) {
        project.updated_at = Some(fraia_core::utils::iso_now());
        persist_project_and_markdown(&dir, &project)?;
    }
    Ok(Json(build_agent_provider_status(
        &project,
        &request.surface,
    )))
}

async fn agent_settings_handler(
    Json(request): Json<AgentSettingsUpdateRequest>,
) -> Result<Json<WorkbenchOperationResponse>, ApiError> {
    let dir = PathBuf::from(&request.project_dir);
    let (mut project, _) = load_project(&dir)
        .with_context(|| format!("failed to load project from {}", dir.display()))?;
    let catalogue = pi_catalogue().context(
        "failed to validate agent settings because the Pi model catalogue is unavailable",
    )?;
    let models = catalogue.models;
    if models.is_empty() {
        return Err(anyhow!(
            "failed to validate agent settings because the Pi model catalogue returned no models"
        )
        .into());
    }
    let requested_provider = request.provider_id.trim();
    let requested_model = request.model.trim();
    let Some(model_option) = models.iter().find(|candidate| {
        candidate.provider_id == requested_provider && candidate.slug == requested_model
    }) else {
        return Err(anyhow!(
            "requested Pi model `{}/{}` is not present in the current catalogue",
            request.provider_id,
            request.model,
        )
        .into());
    };
    if !model_option.available {
        return Err(anyhow!(
            "requested Pi model `{}/{}` is unavailable; authenticate its provider or choose another model",
            request.provider_id,
            request.model,
        )
        .into());
    }
    let requested_reasoning = request.reasoning_effort.trim().to_ascii_lowercase();
    if !model_option
        .supported_reasoning_levels
        .iter()
        .any(|candidate| candidate.effort == requested_reasoning)
    {
        return Err(anyhow!(
            "reasoning effort `{}` is not supported by Pi model `{}/{}`",
            request.reasoning_effort,
            model_option.provider_id,
            model_option.slug
        )
        .into());
    }
    let settings = AgentModelSettings {
        provider_id: model_option.provider_id.clone(),
        model: model_option.slug.clone(),
        reasoning_effort: requested_reasoning,
    };
    ensure_agent_settings(&mut project);
    project
        .agent_state
        .settings_by_surface
        .insert("default".into(), settings.clone());
    project
        .agent_state
        .settings_by_surface
        .insert(request.surface.clone(), settings);
    project.updated_at = Some(fraia_core::utils::iso_now());
    persist_project_and_markdown(&dir, &project)?;
    Ok(Json(WorkbenchOperationResponse {
        message: format!("Updated agent settings for {}.", request.surface),
        state: build_workbench_state(&dir, &project)?,
    }))
}

async fn agent_base_model_guide_reset_handler(
    Json(request): Json<ProjectPathRequest>,
) -> Result<Json<WorkbenchOperationResponse>, ApiError> {
    let dir = PathBuf::from(&request.project_dir);
    let (mut project, _) = load_project(&dir)
        .with_context(|| format!("failed to load project from {}", dir.display()))?;
    project
        .agent_state
        .sessions
        .retain(|session| session.surface != "pre_solve");
    project.base_model_brief = None;
    remove_base_model_brief_artifacts(&dir)?;
    project.updated_at = Some(fraia_core::utils::iso_now());
    persist_project_and_markdown(&dir, &project)?;
    Ok(Json(WorkbenchOperationResponse {
        message: "Reset the Base Model Guide.".into(),
        state: build_workbench_state(&dir, &project)?,
    }))
}

async fn agent_session_start_handler(
    Json(request): Json<AgentSessionStartRequest>,
) -> Result<Json<WorkbenchOperationResponse>, ApiError> {
    let dir = PathBuf::from(&request.project_dir);
    let (mut project, _) = load_project(&dir)
        .with_context(|| format!("failed to load project from {}", dir.display()))?;
    ensure_agent_session(&mut project, &request.surface);
    if request.surface == "pre_solve" {
        ensure_base_model_brief(&mut project);
    }
    let assistant = append_pi_agent_turn(&mut project, &request.surface, "", None);
    if let Some(message) = assistant {
        let session = ensure_agent_session(&mut project, &request.surface);
        session.messages.push(message);
        session.updated_at = fraia_core::utils::iso_now();
    }
    project.updated_at = Some(fraia_core::utils::iso_now());
    persist_project_and_markdown(&dir, &project)?;
    Ok(Json(WorkbenchOperationResponse {
        message: format!("Started {} agent session.", request.surface),
        state: build_workbench_state(&dir, &project)?,
    }))
}

async fn agent_session_respond_handler(
    Json(request): Json<AgentSessionRespondRequest>,
) -> Result<Json<WorkbenchOperationResponse>, ApiError> {
    let dir = PathBuf::from(&request.project_dir);
    let (mut project, _) = load_project(&dir)
        .with_context(|| format!("failed to load project from {}", dir.display()))?;
    let mut user_text = request.text.trim().to_string();
    if user_text.is_empty() && !request.selected_option_ids.is_empty() {
        user_text = selected_reply_text(
            &project,
            &request.surface,
            request.session_id.as_deref(),
            &request.selected_option_ids,
        )
        .unwrap_or_else(|| request.selected_option_ids.join(" "));
    }
    if request.surface == "pre_solve" {
        ensure_base_model_brief(&mut project);
    }
    let assistant = append_pi_agent_turn(
        &mut project,
        &request.surface,
        &user_text,
        request.request_id.as_deref(),
    );
    if let Some(message) = assistant {
        let session = ensure_agent_session(&mut project, &request.surface);
        session.messages.push(message);
        session.updated_at = fraia_core::utils::iso_now();
    }
    project.updated_at = Some(fraia_core::utils::iso_now());
    persist_project_and_markdown(&dir, &project)?;
    Ok(Json(WorkbenchOperationResponse {
        message: "Updated agent conversation.".into(),
        state: build_workbench_state(&dir, &project)?,
    }))
}

async fn agent_session_cancel_handler(
    Json(request): Json<AgentSessionCancelRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let cancelled = cancel_pi_turn(&request.request_id).unwrap_or(false);
    Ok(Json(json!({
        "status": if cancelled { "cancelled" } else { "not_found" },
        "requestId": request.request_id,
    })))
}

async fn agent_apply_review_handler(
    Json(request): Json<AgentApplyReviewRequest>,
) -> Result<Json<WorkbenchOperationResponse>, ApiError> {
    let dir = PathBuf::from(&request.project_dir);
    let (mut project, _) = load_project(&dir)
        .with_context(|| format!("failed to load project from {}", dir.display()))?;

    let mut draft = planning_draft(&project);
    let mut applied = Vec::new();
    let mut diagnostics = Vec::new();
    let mut draft_changed = false;
    let mut materialize_after_draft_change = false;
    let mut structural_changed = false;
    for action in &request.proposed_actions {
        let result = match action.action_kind.as_str() {
            "add_load" | "add_support" => {
                let model = project
                    .structural_model
                    .as_mut()
                    .ok_or_else(|| anyhow!("no authored structural model is available"))?;
                apply_agent_action_to_structural_model(model, action).map(|summary| {
                    structural_changed = true;
                    summary
                })
            }
            _ => apply_agent_action_to_draft(&project, &mut draft, action).map(|summary| {
                draft_changed = true;
                if !action.field.starts_with("coordinationGroup.")
                    && action.field != "coordination.designOptionIntents"
                {
                    materialize_after_draft_change = true;
                }
                summary
            }),
        };
        match result {
            Ok(summary) => applied.push(summary),
            Err(error) => diagnostics.push(WorkbenchDiagnostic {
                severity: "error".into(),
                code: "agent.apply_action_failed".into(),
                message: "Could not apply one agent review action.".into(),
                detail: Some(format!("{error:#}")),
            }),
        }
    }

    if diagnostics.is_empty() {
        let run_summary = if draft_changed {
            apply_planning_draft(&mut project, draft);
            if materialize_after_draft_change {
                Some(materialize_current_planning(&mut project)?.run_summary)
            } else {
                project.updated_at = Some(fraia_core::utils::iso_now());
                Some(AnalysisRunSummary {
                    status: "completed".into(),
                    analysis_kind: "coordination".into(),
                    message: "Updated coordination preferences.".into(),
                    run_id: None,
                })
            }
        } else if structural_changed {
            project.updated_at = Some(fraia_core::utils::iso_now());
            Some(AnalysisRunSummary {
                status: "completed".into(),
                analysis_kind: "review_apply".into(),
                message: "Applied agent review changes to the structural model.".into(),
                run_id: None,
            })
        } else {
            None
        };
        persist_project_and_markdown(&dir, &project)?;
        let mut state = build_workbench_state(&dir, &project)?;
        if let Some(run_summary) = run_summary {
            state.latest_run_summary = Some(run_summary);
        }
        Ok(Json(WorkbenchOperationResponse {
            message: if applied.is_empty() {
                "No agent review changes were applied.".into()
            } else {
                format!("Applied agent review change: {}", applied.join("; "))
            },
            state,
        }))
    } else {
        let mut state = build_workbench_state(&dir, &project)?;
        for diagnostic in diagnostics {
            merge_diagnostic(&mut state, diagnostic);
        }
        Ok(Json(WorkbenchOperationResponse {
            message: "Agent review change was not applied.".into(),
            state,
        }))
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct BaseModelEditRequest {
    project_dir: String,
    operations: Vec<BaseModelEditOperation>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum BaseModelEditOperation {
    CreateNode {
        id: Option<String>,
        x: f64,
        y: f64,
        z: f64,
    },
    UpdateNode {
        id: String,
        x: Option<f64>,
        y: Option<f64>,
        z: Option<f64>,
    },
    DeleteNode {
        id: String,
        cascade: Option<bool>,
    },
    CreateMember {
        id: Option<String>,
        start_node: String,
        end_node: String,
        role: Option<String>,
        section_id: Option<String>,
        material_id: Option<String>,
    },
    UpdateMember {
        id: String,
        role: Option<String>,
        start_node: Option<String>,
        end_node: Option<String>,
        section_id: Option<String>,
        material_id: Option<String>,
    },
    DeleteMember {
        id: String,
    },
    SplitMember {
        id: String,
        node_id: Option<String>,
        x: Option<f64>,
        y: Option<f64>,
        z: Option<f64>,
    },
    AddSupport {
        id: Option<String>,
        target_node: String,
        ux: Option<bool>,
        uy: Option<bool>,
        uz: Option<bool>,
        rx: Option<bool>,
        ry: Option<bool>,
        rz: Option<bool>,
    },
    RemoveSupport {
        id: String,
    },
    AddLoad {
        id: Option<String>,
        target_kind: String,
        target_id: String,
        load_case_id: Option<String>,
        family: Option<String>,
        magnitude: f64,
        direction_x: Option<f64>,
        direction_y: Option<f64>,
        direction_z: Option<f64>,
    },
    RemoveLoad {
        id: String,
    },
}

async fn base_model_edit_handler(
    Json(request): Json<BaseModelEditRequest>,
) -> Result<Json<WorkbenchOperationResponse>, ApiError> {
    let dir = PathBuf::from(&request.project_dir);
    let (mut project, _) = load_project(&dir)
        .with_context(|| format!("failed to load project from {}", dir.display()))?;
    let mut model = project
        .structural_model
        .take()
        .unwrap_or_else(StructuralModel::empty);
    let mut applied = Vec::new();

    for operation in request.operations {
        applied.push(apply_base_model_edit_operation(&mut model, operation)?);
    }

    project.structural_model = Some(model);
    refresh_design_option_batch_freshness(&mut project);
    project.updated_at = Some(fraia_core::utils::iso_now());
    persist_project_and_markdown(&dir, &project)?;
    let mut state = build_workbench_state(&dir, &project)?;
    state.latest_run_summary = Some(AnalysisRunSummary {
        status: "completed".into(),
        analysis_kind: "base_model_edit".into(),
        message: format!("Applied base model edit: {}", applied.join("; ")),
        run_id: None,
    });
    Ok(Json(WorkbenchOperationResponse {
        message: format!("Applied base model edit: {}", applied.join("; ")),
        state,
    }))
}

fn apply_base_model_edit_operation(
    model: &mut StructuralModel,
    operation: BaseModelEditOperation,
) -> Result<String> {
    match operation {
        BaseModelEditOperation::CreateNode { id, x, y, z } => {
            let id = id.unwrap_or_else(|| {
                next_model_id("node.N", model.nodes.iter().map(|node| node.id.as_str()))
            });
            ensure_unique_model_id(model, &id)?;
            model.nodes.push(StructuralNode {
                id: id.clone(),
                x,
                y,
                z,
            });
            Ok(format!("created node {id}"))
        }
        BaseModelEditOperation::UpdateNode { id, x, y, z } => {
            let node = model
                .nodes
                .iter_mut()
                .find(|node| node.id == id)
                .ok_or_else(|| anyhow!("node {id} does not exist"))?;
            if let Some(x) = x {
                node.x = x;
            }
            if let Some(y) = y {
                node.y = y;
            }
            if let Some(z) = z {
                node.z = z;
            }
            Ok(format!("updated node {id}"))
        }
        BaseModelEditOperation::DeleteNode { id, cascade } => {
            if let Some(message) = delete_unnecessary_colinear_node(model, &id)? {
                return Ok(message);
            }
            let referenced = node_is_referenced(model, &id);
            if referenced && !cascade.unwrap_or(false) {
                return Err(anyhow!(
                    "node {id} is referenced; set cascade to delete dependent objects"
                ));
            }
            model.nodes.retain(|node| node.id != id);
            model
                .members
                .retain(|member| member.start_node != id && member.end_node != id);
            model
                .plates
                .retain(|plate| !plate.boundary_nodes.iter().any(|node_id| node_id == &id));
            model.supports.retain(|support| support.target_node != id);
            model.loads.retain(|load| !matches!(&load.target, AssignmentTargetRef::Node(node_id) if node_id == &id));
            Ok(format!("deleted node {id}"))
        }
        BaseModelEditOperation::CreateMember {
            id,
            start_node,
            end_node,
            role,
            section_id,
            material_id,
        } => create_member_and_split_intersections(
            model,
            id,
            start_node,
            end_node,
            role,
            section_id,
            material_id,
        ),
        BaseModelEditOperation::UpdateMember {
            id,
            role,
            start_node,
            end_node,
            section_id,
            material_id,
        } => {
            let index = model
                .members
                .iter()
                .position(|member| member.id == id)
                .ok_or_else(|| anyhow!("member {id} does not exist"))?;
            let next_start = start_node.unwrap_or_else(|| model.members[index].start_node.clone());
            let next_end = end_node.unwrap_or_else(|| model.members[index].end_node.clone());
            ensure_node_exists(model, &next_start)?;
            ensure_node_exists(model, &next_end)?;
            ensure_member_has_length(model, &next_start, &next_end)?;
            let member = &mut model.members[index];
            member.start_node = next_start;
            member.end_node = next_end;
            if let Some(role) = role {
                member.role = clean_role(Some(role));
            }
            if let Some(section_id) = section_id {
                member.section_id = section_id;
            }
            if let Some(material_id) = material_id {
                member.material_id = material_id;
            }
            Ok(format!("updated member {id}"))
        }
        BaseModelEditOperation::DeleteMember { id } => {
            let index = model
                .members
                .iter()
                .position(|member| member.id == id)
                .ok_or_else(|| anyhow!("member {id} does not exist"))?;
            let removed = model.members.remove(index);
            let endpoint_ids = [removed.start_node, removed.end_node];
            model.loads.retain(|load| !matches!(&load.target, AssignmentTargetRef::Member(member_id) if member_id == &id));
            model
                .releases
                .retain(|release| release.target.member_id != id);
            let merged_split_nodes = endpoint_ids
                .iter()
                .map(|node_id| delete_unnecessary_colinear_node(model, node_id))
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .filter(Option::is_some)
                .count();
            let removed_nodes = delete_free_member_endpoint_nodes(model, &endpoint_ids);
            match (merged_split_nodes, removed_nodes) {
                (0, 0) => Ok(format!("deleted member {id}")),
                (0, _) => Ok(format!(
                    "deleted member {id} and removed {removed_nodes} free node(s)"
                )),
                (_, 0) => Ok(format!(
                    "deleted member {id} and merged {merged_split_nodes} split node(s)"
                )),
                (_, _) => Ok(format!(
                    "deleted member {id}, merged {merged_split_nodes} split node(s), and removed {removed_nodes} free node(s)"
                )),
            }
        }
        BaseModelEditOperation::SplitMember {
            id,
            node_id,
            x,
            y,
            z,
        } => split_member(model, &id, node_id, x, y, z),
        BaseModelEditOperation::AddSupport {
            id,
            target_node,
            ux,
            uy,
            uz,
            rx,
            ry,
            rz,
        } => {
            ensure_node_exists(model, &target_node)?;
            let id = id.unwrap_or_else(|| {
                next_model_id(
                    "support.S",
                    model.supports.iter().map(|support| support.id.as_str()),
                )
            });
            ensure_unique_model_id(model, &id)?;
            model.supports.push(SupportAssignment {
                id: id.clone(),
                target_node,
                ux: ux.unwrap_or(true),
                uy: uy.unwrap_or(true),
                uz: uz.unwrap_or(true),
                rx: rx.unwrap_or(false),
                ry: ry.unwrap_or(false),
                rz: rz.unwrap_or(false),
            });
            Ok(format!("added support {id}"))
        }
        BaseModelEditOperation::RemoveSupport { id } => {
            let before = model.supports.len();
            model.supports.retain(|support| support.id != id);
            if model.supports.len() == before {
                return Err(anyhow!("support {id} does not exist"));
            }
            Ok(format!("removed support {id}"))
        }
        BaseModelEditOperation::AddLoad {
            id,
            target_kind,
            target_id,
            load_case_id,
            family,
            magnitude,
            direction_x,
            direction_y,
            direction_z,
        } => {
            let target = match target_kind.as_str() {
                "node" => {
                    ensure_node_exists(model, &target_id)?;
                    AssignmentTargetRef::Node(target_id)
                }
                "member" => {
                    ensure_member_exists(model, &target_id)?;
                    AssignmentTargetRef::Member(target_id)
                }
                other => return Err(anyhow!("unsupported load target kind {other}")),
            };
            let kind = match family
                .as_deref()
                .unwrap_or(target.expected_load_kind().as_str())
            {
                "point" | "nodal_force" => LoadKind::Point,
                "uniform_line" | "distributed" => LoadKind::UniformLine,
                other => return Err(anyhow!("unsupported load family {other}")),
            };
            if kind != target.expected_load_kind() {
                return Err(anyhow!(
                    "load family {} is invalid for {} target",
                    kind.as_str(),
                    target.kind_label()
                ));
            }
            let id = id.unwrap_or_else(|| {
                next_model_id("load.L", model.loads.iter().map(|load| load.id.as_str()))
            });
            ensure_unique_model_id(model, &id)?;
            let direction = normalized_load_vector(
                direction_x.unwrap_or(0.0),
                direction_y.unwrap_or(-1.0),
                direction_z.unwrap_or(0.0),
            )?;
            model.loads.push(LoadAssignment {
                id: id.clone(),
                target,
                load_case_id: load_case_id.unwrap_or_else(|| "LC1".into()),
                kind,
                direction,
                magnitude,
            });
            Ok(format!("added load {id}"))
        }
        BaseModelEditOperation::RemoveLoad { id } => {
            let before = model.loads.len();
            model.loads.retain(|load| load.id != id);
            if model.loads.len() == before {
                return Err(anyhow!("load {id} does not exist"));
            }
            Ok(format!("removed load {id}"))
        }
    }
}

fn clean_role(role: Option<String>) -> String {
    let role = role
        .unwrap_or_else(|| "member".into())
        .trim()
        .replace([' ', '-'], "_");
    if role.is_empty() {
        "member".into()
    } else {
        role
    }
}

fn ensure_unique_model_id(model: &StructuralModel, id: &str) -> Result<()> {
    if model.nodes.iter().any(|node| node.id == id)
        || model.members.iter().any(|member| member.id == id)
        || model.plates.iter().any(|plate| plate.id == id)
        || model.supports.iter().any(|support| support.id == id)
        || model.loads.iter().any(|load| load.id == id)
        || model.releases.iter().any(|release| release.id == id)
    {
        return Err(anyhow!("id {id} already exists in the base model"));
    }
    Ok(())
}

fn ensure_node_exists(model: &StructuralModel, id: &str) -> Result<()> {
    model
        .node_by_id(id)
        .map(|_| ())
        .ok_or_else(|| anyhow!("node {id} does not exist"))
}

fn ensure_member_exists(model: &StructuralModel, id: &str) -> Result<()> {
    model
        .members
        .iter()
        .find(|member| member.id == id)
        .map(|_| ())
        .ok_or_else(|| anyhow!("member {id} does not exist"))
}

fn delete_free_member_endpoint_nodes(
    model: &mut StructuralModel,
    endpoint_ids: &[String; 2],
) -> usize {
    let candidates = endpoint_ids.iter().cloned().collect::<BTreeSet<_>>();
    let removable = candidates
        .into_iter()
        .filter(|node_id| {
            model.nodes.iter().any(|node| &node.id == node_id)
                && !node_is_referenced(model, node_id)
        })
        .collect::<BTreeSet<_>>();
    let removed = removable.len();
    model.nodes.retain(|node| !removable.contains(&node.id));
    removed
}

fn node_is_referenced(model: &StructuralModel, id: &str) -> bool {
    model
        .members
        .iter()
        .any(|member| member.start_node == id || member.end_node == id)
        || model
            .plates
            .iter()
            .any(|plate| plate.boundary_nodes.iter().any(|node_id| node_id == id))
        || model
            .supports
            .iter()
            .any(|support| support.target_node == id)
        || model
            .loads
            .iter()
            .any(|load| matches!(&load.target, AssignmentTargetRef::Node(node_id) if node_id == id))
}

fn delete_unnecessary_colinear_node(
    model: &mut StructuralModel,
    id: &str,
) -> Result<Option<String>> {
    if node_has_non_member_references(model, id) {
        return Ok(None);
    }
    let Some(node) = model.node_by_id(id) else {
        return Ok(None);
    };
    let node_point = ModelPoint {
        x: node.x,
        y: node.y,
        z: node.z,
    };
    let incident_members = model
        .members
        .iter()
        .enumerate()
        .filter(|(_, member)| member.start_node == id || member.end_node == id)
        .map(|(index, member)| (index, member.clone()))
        .collect::<Vec<_>>();
    if incident_members.len() != 2 {
        return Ok(None);
    }

    let (keep_index, keep_member) = &incident_members[0];
    let (remove_index, remove_member) = &incident_members[1];
    if !members_have_merge_compatible_metadata(keep_member, remove_member)
        || member_has_assigned_load_or_release(model, &keep_member.id)
        || member_has_assigned_load_or_release(model, &remove_member.id)
    {
        return Ok(None);
    }

    let keep_far_node = other_member_endpoint(keep_member, id)?;
    let remove_far_node = other_member_endpoint(remove_member, id)?;
    if keep_far_node == remove_far_node {
        return Ok(None);
    }
    let keep_far_point = model_node_point(model, keep_far_node)?;
    let remove_far_point = model_node_point(model, remove_far_node)?;
    if !point_between_colinear_endpoints(keep_far_point, node_point, remove_far_point) {
        return Ok(None);
    }

    let keep_member_id = keep_member.id.clone();
    let remove_member_id = remove_member.id.clone();
    if model.members[*keep_index].start_node == id {
        model.members[*keep_index].start_node = remove_far_node.to_string();
    } else {
        model.members[*keep_index].end_node = remove_far_node.to_string();
    }
    ensure_member_has_length(
        model,
        &model.members[*keep_index].start_node,
        &model.members[*keep_index].end_node,
    )?;
    model.members.remove(*remove_index);
    model.nodes.retain(|node| node.id != id);
    Ok(Some(format!(
        "deleted unnecessary node {id} and merged members {keep_member_id} and {remove_member_id}"
    )))
}

fn node_has_non_member_references(model: &StructuralModel, id: &str) -> bool {
    model
        .plates
        .iter()
        .any(|plate| plate.boundary_nodes.iter().any(|node_id| node_id == id))
        || model
            .supports
            .iter()
            .any(|support| support.target_node == id)
        || model
            .loads
            .iter()
            .any(|load| matches!(&load.target, AssignmentTargetRef::Node(node_id) if node_id == id))
}

fn members_have_merge_compatible_metadata(
    first: &StructuralMember,
    second: &StructuralMember,
) -> bool {
    first.role == second.role
        && first.semantic_tags == second.semantic_tags
        && first.section_id == second.section_id
        && first.material_id == second.material_id
}

fn member_has_assigned_load_or_release(model: &StructuralModel, id: &str) -> bool {
    model.loads.iter().any(
        |load| matches!(&load.target, AssignmentTargetRef::Member(member_id) if member_id == id),
    ) || model
        .releases
        .iter()
        .any(|release| release.target.member_id == id)
}

fn other_member_endpoint<'a>(member: &'a StructuralMember, node_id: &str) -> Result<&'a str> {
    if member.start_node == node_id {
        Ok(&member.end_node)
    } else if member.end_node == node_id {
        Ok(&member.start_node)
    } else {
        Err(anyhow!(
            "member {} is not incident to node {node_id}",
            member.id
        ))
    }
}

fn point_between_colinear_endpoints(
    first_endpoint: ModelPoint,
    middle: ModelPoint,
    second_endpoint: ModelPoint,
) -> bool {
    let ax = first_endpoint.x - middle.x;
    let ay = first_endpoint.y - middle.y;
    let az = first_endpoint.z - middle.z;
    let bx = second_endpoint.x - middle.x;
    let by = second_endpoint.y - middle.y;
    let bz = second_endpoint.z - middle.z;
    let a_length = (ax * ax + ay * ay + az * az).sqrt();
    let b_length = (bx * bx + by * by + bz * bz).sqrt();
    if a_length <= 1e-9 || b_length <= 1e-9 {
        return false;
    }
    let cross_x = ay * bz - az * by;
    let cross_y = az * bx - ax * bz;
    let cross_z = ax * by - ay * bx;
    let cross_length = (cross_x * cross_x + cross_y * cross_y + cross_z * cross_z).sqrt();
    let dot = ax * bx + ay * by + az * bz;
    cross_length <= MEMBER_INTERSECTION_EPS * a_length * b_length
        && dot < -MEMBER_INTERSECTION_EPS * a_length * b_length
}

fn ensure_member_has_length(
    model: &StructuralModel,
    start_node: &str,
    end_node: &str,
) -> Result<()> {
    let start = model
        .node_by_id(start_node)
        .ok_or_else(|| anyhow!("node {start_node} does not exist"))?;
    let end = model
        .node_by_id(end_node)
        .ok_or_else(|| anyhow!("node {end_node} does not exist"))?;
    let length =
        ((end.x - start.x).powi(2) + (end.y - start.y).powi(2) + (end.z - start.z).powi(2)).sqrt();
    if length <= 1e-9 {
        return Err(anyhow!("member endpoints must be distinct"));
    }
    Ok(())
}

fn next_model_id<'a>(prefix: &str, ids: impl Iterator<Item = &'a str>) -> String {
    let mut next = 1usize;
    for id in ids {
        if let Some(rest) = id.strip_prefix(prefix) {
            if let Ok(value) = rest.parse::<usize>() {
                next = next.max(value + 1);
            }
        }
    }
    format!("{prefix}{next}")
}

fn normalized_load_vector(x: f64, y: f64, z: f64) -> Result<LoadVector> {
    let length = (x * x + y * y + z * z).sqrt();
    if length <= 1e-9 {
        return Err(anyhow!("load direction must be non-zero"));
    }
    Ok(LoadVector {
        x: x / length,
        y: y / length,
        z: z / length,
    })
}

#[derive(Clone, Copy)]
struct ModelPoint {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Clone)]
struct MemberSplitPoint {
    t: f64,
    node_id: String,
}

struct SegmentIntersection {
    new_t: f64,
    existing_t: f64,
    point: ModelPoint,
}

const MEMBER_INTERSECTION_EPS: f64 = 1e-7;

fn create_member_and_split_intersections(
    model: &mut StructuralModel,
    id: Option<String>,
    start_node: String,
    end_node: String,
    role: Option<String>,
    section_id: Option<String>,
    material_id: Option<String>,
) -> Result<String> {
    ensure_node_exists(model, &start_node)?;
    ensure_node_exists(model, &end_node)?;
    ensure_member_has_length(model, &start_node, &end_node)?;
    let id = id.unwrap_or_else(|| {
        next_model_id(
            "member.M",
            model.members.iter().map(|member| member.id.as_str()),
        )
    });
    ensure_unique_model_id(model, &id)?;
    let start = model_node_point(model, &start_node)?;
    let end = model_node_point(model, &end_node)?;
    let existing_members = model.members.clone();
    let mut new_member_splits: Vec<MemberSplitPoint> = Vec::new();
    let mut existing_member_splits: BTreeMap<String, Vec<MemberSplitPoint>> = BTreeMap::new();

    for member in existing_members {
        let member_start = model_node_point(model, &member.start_node)?;
        let member_end = model_node_point(model, &member.end_node)?;
        let Some(intersection) = segment_intersection_xy(start, end, member_start, member_end)
        else {
            continue;
        };
        let node_id = find_or_create_node_at_point(model, intersection.point);
        if is_segment_interior(intersection.new_t) {
            push_member_split(&mut new_member_splits, intersection.new_t, node_id.clone());
        }
        if is_segment_interior(intersection.existing_t) {
            push_member_split(
                existing_member_splits.entry(member.id.clone()).or_default(),
                intersection.existing_t,
                node_id,
            );
        }
    }

    model.members.push(StructuralMember {
        id: id.clone(),
        start_node,
        end_node,
        role: clean_role(role),
        semantic_tags: Vec::new(),
        section_id: section_id.unwrap_or_else(|| "unassigned".into()),
        material_id: material_id.unwrap_or_else(|| "unassigned".into()),
    });

    let mut split_count = 0usize;
    for (member_id, splits) in existing_member_splits {
        split_count += split_member_at_nodes(model, &member_id, splits)?.len();
    }
    split_count += split_member_at_nodes(model, &id, new_member_splits)?.len();

    if split_count == 0 {
        Ok(format!("created member {id}"))
    } else {
        Ok(format!(
            "created member {id} and split {split_count} intersecting segment(s)"
        ))
    }
}

fn segment_intersection_xy(
    a: ModelPoint,
    b: ModelPoint,
    c: ModelPoint,
    d: ModelPoint,
) -> Option<SegmentIntersection> {
    if !same_z(a, b) || !same_z(c, d) || (a.z - c.z).abs() > MEMBER_INTERSECTION_EPS {
        return None;
    }
    let abx = b.x - a.x;
    let aby = b.y - a.y;
    let cdx = d.x - c.x;
    let cdy = d.y - c.y;
    let denom = cross2(abx, aby, cdx, cdy);
    if denom.abs() <= MEMBER_INTERSECTION_EPS {
        return None;
    }
    let acx = c.x - a.x;
    let acy = c.y - a.y;
    let new_t = cross2(acx, acy, cdx, cdy) / denom;
    let existing_t = cross2(acx, acy, abx, aby) / denom;
    if !is_segment_parameter(new_t) || !is_segment_parameter(existing_t) {
        return None;
    }
    let new_t = clamp_unit(new_t);
    let existing_t = clamp_unit(existing_t);
    Some(SegmentIntersection {
        new_t,
        existing_t,
        point: ModelPoint {
            x: a.x + abx * new_t,
            y: a.y + aby * new_t,
            z: a.z,
        },
    })
}

fn split_member_at_nodes(
    model: &mut StructuralModel,
    id: &str,
    splits: Vec<MemberSplitPoint>,
) -> Result<Vec<String>> {
    let index = model
        .members
        .iter()
        .position(|member| member.id == id)
        .ok_or_else(|| anyhow!("member {id} does not exist"))?;
    let original = model.members[index].clone();
    let mut sorted = splits
        .into_iter()
        .filter(|split| is_segment_interior(split.t))
        .collect::<Vec<_>>();
    sorted.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap_or(std::cmp::Ordering::Equal));
    sorted.dedup_by(|a, b| (a.t - b.t).abs() <= MEMBER_INTERSECTION_EPS || a.node_id == b.node_id);
    if sorted.is_empty() {
        return Ok(Vec::new());
    }

    let mut node_ids = Vec::with_capacity(sorted.len() + 2);
    node_ids.push(original.start_node.clone());
    node_ids.extend(sorted.into_iter().map(|split| split.node_id));
    node_ids.push(original.end_node.clone());
    for pair in node_ids.windows(2) {
        ensure_member_has_length(model, &pair[0], &pair[1])?;
    }

    let mut created_ids = Vec::new();
    model.members[index].end_node = node_ids[1].clone();
    for pair in node_ids.windows(2).skip(1) {
        let second_id = next_model_id(
            "member.M",
            model.members.iter().map(|member| member.id.as_str()),
        );
        ensure_unique_model_id(model, &second_id)?;
        model.members.push(StructuralMember {
            id: second_id.clone(),
            start_node: pair[0].clone(),
            end_node: pair[1].clone(),
            role: original.role.clone(),
            semantic_tags: original.semantic_tags.clone(),
            section_id: original.section_id.clone(),
            material_id: original.material_id.clone(),
        });
        created_ids.push(second_id);
    }
    Ok(created_ids)
}

fn model_node_point(model: &StructuralModel, id: &str) -> Result<ModelPoint> {
    let node = model
        .node_by_id(id)
        .ok_or_else(|| anyhow!("node {id} does not exist"))?;
    Ok(ModelPoint {
        x: node.x,
        y: node.y,
        z: node.z,
    })
}

fn find_or_create_node_at_point(model: &mut StructuralModel, point: ModelPoint) -> String {
    if let Some(node) = model
        .nodes
        .iter()
        .find(|node| point_matches_node(point, node))
    {
        return node.id.clone();
    }
    let id = next_model_id("node.N", model.nodes.iter().map(|node| node.id.as_str()));
    model.nodes.push(StructuralNode {
        id: id.clone(),
        x: clean_geometry_value(point.x),
        y: clean_geometry_value(point.y),
        z: clean_geometry_value(point.z),
    });
    id
}

fn push_member_split(splits: &mut Vec<MemberSplitPoint>, t: f64, node_id: String) {
    if splits
        .iter()
        .any(|split| (split.t - t).abs() <= MEMBER_INTERSECTION_EPS || split.node_id == node_id)
    {
        return;
    }
    splits.push(MemberSplitPoint { t, node_id });
}

fn point_matches_node(point: ModelPoint, node: &StructuralNode) -> bool {
    (point.x - node.x).abs() <= MEMBER_INTERSECTION_EPS
        && (point.y - node.y).abs() <= MEMBER_INTERSECTION_EPS
        && (point.z - node.z).abs() <= MEMBER_INTERSECTION_EPS
}

fn same_z(a: ModelPoint, b: ModelPoint) -> bool {
    (a.z - b.z).abs() <= MEMBER_INTERSECTION_EPS
}

fn cross2(ax: f64, ay: f64, bx: f64, by: f64) -> f64 {
    ax * by - ay * bx
}

fn is_segment_parameter(value: f64) -> bool {
    value >= -MEMBER_INTERSECTION_EPS && value <= 1.0 + MEMBER_INTERSECTION_EPS
}

fn is_segment_interior(value: f64) -> bool {
    value > MEMBER_INTERSECTION_EPS && value < 1.0 - MEMBER_INTERSECTION_EPS
}

fn clamp_unit(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

fn clean_geometry_value(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

fn split_member(
    model: &mut StructuralModel,
    id: &str,
    node_id: Option<String>,
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
) -> Result<String> {
    let index = model
        .members
        .iter()
        .position(|member| member.id == id)
        .ok_or_else(|| anyhow!("member {id} does not exist"))?;
    let original = model.members[index].clone();
    let split_node_id = if let Some(node_id) = node_id {
        ensure_node_exists(model, &node_id)?;
        node_id
    } else {
        let split_id = next_model_id("node.N", model.nodes.iter().map(|node| node.id.as_str()));
        model.nodes.push(StructuralNode {
            id: split_id.clone(),
            x: x.ok_or_else(|| anyhow!("split x is required when no node_id is provided"))?,
            y: y.ok_or_else(|| anyhow!("split y is required when no node_id is provided"))?,
            z: z.unwrap_or(0.0),
        });
        split_id
    };
    ensure_member_has_length(model, &original.start_node, &split_node_id)?;
    ensure_member_has_length(model, &split_node_id, &original.end_node)?;
    let second_id = next_model_id(
        "member.M",
        model.members.iter().map(|member| member.id.as_str()),
    );
    ensure_unique_model_id(model, &second_id)?;
    model.members[index].end_node = split_node_id.clone();
    model.members.push(StructuralMember {
        id: second_id.clone(),
        start_node: split_node_id,
        end_node: original.end_node,
        role: original.role,
        semantic_tags: original.semantic_tags,
        section_id: original.section_id,
        material_id: original.material_id,
    });
    Ok(format!("split member {id} into {id} and {second_id}"))
}

async fn seed_frame_demo_handler(
    Json(request): Json<ProjectPathRequest>,
) -> Result<Json<WorkbenchOperationResponse>, ApiError> {
    let dir = PathBuf::from(request.project_dir);
    let (mut project, _) = load_project(&dir)
        .with_context(|| format!("failed to load project from {}", dir.display()))?;
    let graph = portal_frame_builder_graph(
        "builder.frame.authored",
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
    let structural = materialize_structural_model_from_builder_graph(&graph)
        .context("failed to materialise portal frame demo from project requirements")?;
    project.intent.building_type = "portal_frame".into();
    project.builder_graph = Some(graph);
    project.legacy_builder_instance = None;
    project.structural_model = Some(structural);
    project.updated_at = Some(fraia_core::utils::iso_now());
    persist_project_and_markdown(&dir, &project)?;
    let mut state = build_workbench_state(&dir, &project)?;
    state.latest_run_summary = Some(AnalysisRunSummary {
        status: "completed".into(),
        analysis_kind: "materialise".into(),
        message: "Seeded a deterministic portal frame model.".into(),
        run_id: None,
    });
    Ok(Json(WorkbenchOperationResponse {
        message: "Seeded a deterministic portal-frame demo from current project requirements."
            .into(),
        state,
    }))
}

async fn seed_frame_review_demo_handler(
    Json(request): Json<ProjectPathRequest>,
) -> Result<Json<WorkbenchOperationResponse>, ApiError> {
    let dir = PathBuf::from(request.project_dir);
    let (mut project, _) = load_project(&dir)
        .with_context(|| format!("failed to load project from {}", dir.display()))?;
    let span_m = if project.requirements.span_m > 1.0 {
        project.requirements.span_m
    } else {
        20.0
    };
    let height_m = if project.requirements.height_m > 1.0 {
        project.requirements.height_m
    } else {
        6.0
    };
    let structural = StructuralModel {
        dimension: "2d".into(),
        nodes: vec![
            StructuralNode {
                id: "builder.frame.review::n1".into(),
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            StructuralNode {
                id: "builder.frame.review::n2".into(),
                x: 0.0,
                y: height_m,
                z: 0.0,
            },
            StructuralNode {
                id: "builder.frame.review::n3".into(),
                x: span_m,
                y: height_m,
                z: 0.0,
            },
            StructuralNode {
                id: "builder.frame.review::n4".into(),
                x: span_m,
                y: 0.0,
                z: 0.0,
            },
        ],
        members: vec![
            StructuralMember {
                id: "builder.frame.review::e1".into(),
                start_node: "builder.frame.review::n1".into(),
                end_node: "builder.frame.review::n2".into(),
                role: "member".into(),
                semantic_tags: Vec::new(),
                section_id: "unassigned".into(),
                material_id: "unassigned".into(),
            },
            StructuralMember {
                id: "builder.frame.review::e2".into(),
                start_node: "builder.frame.review::n2".into(),
                end_node: "builder.frame.review::n3".into(),
                role: "member".into(),
                semantic_tags: Vec::new(),
                section_id: "unassigned".into(),
                material_id: "unassigned".into(),
            },
            StructuralMember {
                id: "builder.frame.review::e3".into(),
                start_node: "builder.frame.review::n4".into(),
                end_node: "builder.frame.review::n3".into(),
                role: "member".into(),
                semantic_tags: Vec::new(),
                section_id: "unassigned".into(),
                material_id: "unassigned".into(),
            },
        ],
        plates: Vec::new(),
        supports: Vec::new(),
        loads: Vec::new(),
        releases: Vec::new(),
        load_cases: Vec::new(),
        builder_node_materializations: Vec::new(),
    };
    project.intent.building_type = "unspecified".into();
    project.requirements.span_m = span_m;
    project.requirements.height_m = height_m;
    project.requirements.gravity_load_kn_per_m = 0.0;
    project.requirements.lateral_load_kn = 0.0;
    let mut draft = planning_draft(&project);
    draft.project_intent.building_type = "unspecified".into();
    draft.system_brief.system_family_hint = "unknown".into();
    draft.system_brief.structural_form_hint = "Raw CAD-like frame geometry".into();
    draft.system_brief.notes =
        "Development review model: raw connected line geometry only. Roles, material, supports, and loads are intentionally unresolved."
            .into();
    draft.geometry_and_loads.span_m = span_m;
    draft.geometry_and_loads.height_m = height_m;
    draft.geometry_and_loads.gravity_line_load_kn_per_m = 0.0;
    draft.geometry_and_loads.lateral_load_kn = 0.0;
    project.planning_draft = Some(draft);
    project.builder_graph = None;
    project.legacy_builder_instance = None;
    project.structural_model = Some(structural);
    project.agent_state = AgentState::default();
    project.base_model_brief = None;
    project.updated_at = Some(fraia_core::utils::iso_now());
    persist_project_and_markdown(&dir, &project)?;
    let mut state = build_workbench_state(&dir, &project)?;
    state.latest_run_summary = Some(AnalysisRunSummary {
        status: "not_run".into(),
        analysis_kind: "review_seed".into(),
        message:
            "Seeded raw frame geometry only. Roles, supports, loads, and material are intentionally unresolved."
                .into(),
        run_id: None,
    });
    Ok(Json(WorkbenchOperationResponse {
        message: "Seeded raw review geometry without roles, supports, loads, or material.".into(),
        state,
    }))
}

async fn seed_beam_demo_handler(
    Json(request): Json<ProjectPathRequest>,
) -> Result<Json<WorkbenchOperationResponse>, ApiError> {
    let dir = PathBuf::from(request.project_dir);
    let (mut project, _) = load_project(&dir)
        .with_context(|| format!("failed to load project from {}", dir.display()))?;
    seed_simply_supported_beam_in_project(&mut project, Some("builder.beam.authored"))
        .context("failed to seed simply supported beam from project requirements")?;
    persist_project_and_markdown(&dir, &project)?;
    let mut state = build_workbench_state(&dir, &project)?;
    state.latest_run_summary = Some(AnalysisRunSummary {
        status: "completed".into(),
        analysis_kind: "materialise".into(),
        message: "Seeded a deterministic simply supported beam model.".into(),
        run_id: None,
    });
    Ok(Json(WorkbenchOperationResponse {
        message: "Seeded a deterministic simply supported beam from current project requirements."
            .into(),
        state,
    }))
}

async fn beam_size_handler(
    Json(request): Json<ProjectPathRequest>,
) -> Result<Json<WorkbenchOperationResponse>, ApiError> {
    let dir = PathBuf::from(request.project_dir);
    let (mut project, _) = load_project(&dir)
        .with_context(|| format!("failed to load project from {}", dir.display()))?;
    let run_dir = persist_beam_sizing_run(&dir, &mut project)?;
    let state = build_workbench_state(&dir, &project)?;
    Ok(Json(WorkbenchOperationResponse {
        message: format!("Saved beam sizing artefacts to {}", run_dir.display()),
        state,
    }))
}

async fn validate_handler(
    Json(request): Json<ProjectPathRequest>,
) -> Result<Json<WorkbenchOperationResponse>, ApiError> {
    let dir = PathBuf::from(request.project_dir);
    let (project, _) = load_project(&dir)
        .with_context(|| format!("failed to load project from {}", dir.display()))?;
    let run_dir = persist_validation_run(&dir, &project)?;
    let state = build_workbench_state(&dir, &project)?;
    Ok(Json(WorkbenchOperationResponse {
        message: format!("Saved validation artefacts to {}", run_dir.display()),
        state,
    }))
}

async fn frame_run_calculix_handler(
    Json(request): Json<ProjectPathRequest>,
) -> Result<Json<WorkbenchOperationResponse>, ApiError> {
    let dir = PathBuf::from(request.project_dir);
    let (project, _) = load_project(&dir)
        .with_context(|| format!("failed to load project from {}", dir.display()))?;
    let run_dir = persist_frame_calculix_run(&dir, &project)?;
    let state = build_workbench_state(&dir, &project)?;
    Ok(Json(WorkbenchOperationResponse {
        message: format!(
            "Saved frame CalculiX run artefacts to {}",
            run_dir.display()
        ),
        state,
    }))
}

#[derive(Debug, Default, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct BeamPlanningSystemParameters {
    point_load_kn: Option<f64>,
    point_load_x_m: Option<f64>,
    preferred_section: Option<String>,
    allowed_section_families: Option<Vec<String>>,
    preferred_section_family: Option<String>,
    excluded_section_families: Option<Vec<String>>,
    #[allow(dead_code)]
    section_selection_strategy: Option<String>,
}

#[derive(Debug, Default, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PortalFramePlanningSystemParameters {
    topology_id: Option<String>,
    beam_section: Option<String>,
    column_section: Option<String>,
}

enum SupportedFamily {
    BeamSimplySupported,
    PortalFrame,
    Unsupported(String),
}

struct MaterializeOutcome {
    can_analyse: bool,
    message: String,
    run_summary: AnalysisRunSummary,
}

struct AnalysisOutcome {
    message: String,
    run_summary: AnalysisRunSummary,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct DesignOptionAnalysisRunManifest {
    run_id: String,
    run_kind: String,
    generated_at: String,
    project_name: String,
    option_ids: Vec<String>,
    candidate_policy: String,
    check_profile: String,
    solver: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct DesignOptionCandidateAnalysisInput {
    option_id: String,
    option_label: String,
    coordination_group_id: String,
    section_id: String,
    selected_candidate: bool,
    member_ids: Vec<String>,
    standardisation_policy: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct DesignOptionCandidateAnalysisResult {
    option_id: String,
    option_label: String,
    coordination_group_id: String,
    section_id: String,
    status: String,
    passed: Option<bool>,
    selected_candidate: bool,
    approximate_mass_kg: Option<f64>,
    max_utilization: Option<f64>,
    max_stress_mpa: Option<f64>,
    max_moment_knm: Option<f64>,
    max_shear_kn: Option<f64>,
    max_deflection_mm: Option<f64>,
    max_drift_mm: Option<f64>,
    max_reaction_kn: Option<f64>,
    governing_member_id: Option<String>,
    governing_combo_id: Option<String>,
    diagnostic: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct DesignOptionAnalysisOptionResult {
    option_id: String,
    option_label: String,
    lifecycle_status: String,
    selected_result: Option<DesignOptionCandidateAnalysisResult>,
    candidate_results: Vec<DesignOptionCandidateAnalysisResult>,
    diagnostics: Vec<WorkbenchDiagnostic>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct DesignOptionAnalysisComparison {
    run_id: String,
    option_results: Vec<DesignOptionAnalysisOptionResult>,
}

#[derive(Debug, Clone, Default)]
struct DesignOptionAnalysisLookup {
    candidate_results: BTreeMap<String, DesignOptionCandidateAnalysisResult>,
    candidate_run_ids: BTreeMap<String, String>,
    selected_results: BTreeMap<String, DesignOptionCandidateAnalysisResult>,
    candidate_previews: BTreeMap<String, Value>,
    selected_previews: BTreeMap<String, Value>,
}

impl DesignOptionAnalysisLookup {
    fn candidate_key(option_id: &str, group_id: &str, section_id: &str) -> String {
        format!("{option_id}\u{1f}{group_id}\u{1f}{section_id}")
    }

    fn insert_candidate(&mut self, result: DesignOptionCandidateAnalysisResult, run_id: &str) {
        let key = Self::candidate_key(
            &result.option_id,
            &result.coordination_group_id,
            &result.section_id,
        );
        if result.selected_candidate {
            self.selected_results
                .entry(result.option_id.clone())
                .or_insert_with(|| result.clone());
        }
        self.candidate_run_ids.insert(key.clone(), run_id.into());
        self.candidate_results.insert(key, result);
    }

    fn candidate(
        &self,
        option_id: &str,
        group_id: &str,
        section_id: &str,
    ) -> Option<&DesignOptionCandidateAnalysisResult> {
        self.candidate_results
            .get(&Self::candidate_key(option_id, group_id, section_id))
    }

    fn candidate_run_id(&self, option_id: &str, group_id: &str, section_id: &str) -> Option<&str> {
        self.candidate_run_ids
            .get(&Self::candidate_key(option_id, group_id, section_id))
            .map(String::as_str)
    }

    fn insert_preview(&mut self, preview: Value) {
        let Some(option_id) = preview.get("optionId").and_then(Value::as_str) else {
            return;
        };
        let Some(group_id) = preview.get("coordinationGroupId").and_then(Value::as_str) else {
            return;
        };
        let Some(section_id) = preview.get("sectionId").and_then(Value::as_str) else {
            return;
        };
        let key = Self::candidate_key(option_id, group_id, section_id);
        self.candidate_previews.insert(key, preview);
    }

    fn preview(&self, option_id: &str, group_id: &str, section_id: &str) -> Option<&Value> {
        self.candidate_previews
            .get(&Self::candidate_key(option_id, group_id, section_id))
    }
}

fn interpret_agent_review_reply(
    project: &ProjectFile,
    request: &AgentReviewReplyRequest,
) -> AgentReviewReplyResponse {
    let mut settings = agent_settings_for_surface(project, "comment_review");
    settings.model = sanitize_agent_model(request.model.as_deref().or(Some(&settings.model)));
    settings.reasoning_effort = sanitize_reasoning_effort(
        request
            .reasoning_effort
            .as_deref()
            .or(Some(&settings.reasoning_effort)),
    );
    match run_pi_review_agent(project, request, &settings) {
        Ok(response) => response,
        Err(error) => review_agent_unavailable(
            &settings.model,
            &settings.reasoning_effort,
            "The AI provider could not complete this review. No local fallback was used.",
            Some(format!("{error:#}")),
        ),
    }
}

fn review_agent_unavailable(
    model: &str,
    reasoning_effort: &str,
    message: &str,
    detail: Option<String>,
) -> AgentReviewReplyResponse {
    AgentReviewReplyResponse {
        agent_mode: "pi_unavailable".into(),
        model: model.into(),
        reasoning_effort: reasoning_effort.into(),
        status: "needs_more_information".into(),
        message: message.into(),
        follow_up: Some(
            "Connect the selected provider or choose another available model, then ask again."
                .into(),
        ),
        suggested_chips: Vec::new(),
        resolution_summary: String::new(),
        interpretation: String::new(),
        proposed_actions: Vec::new(),
        diagnostics: vec![WorkbenchDiagnostic {
            severity: "error".into(),
            code: "agent.pi_unavailable".into(),
            message: message.into(),
            detail,
        }],
    }
}

fn interpret_agent_coordinator(
    _project: &ProjectFile,
    request: &AgentCoordinatorRequest,
) -> AgentCoordinatorResponse {
    let model = sanitize_agent_model(request.model.as_deref());
    let reasoning_effort = sanitize_reasoning_effort(request.reasoning_effort.as_deref());
    coordinator_agent_unavailable(
        &model,
        &reasoning_effort,
        "The legacy pre-solve coordinator local fallback is disabled. Use the main AI assistant instead.",
        None,
    )
}

fn coordinator_agent_unavailable(
    model: &str,
    reasoning_effort: &str,
    message: &str,
    detail: Option<String>,
) -> AgentCoordinatorResponse {
    AgentCoordinatorResponse {
        agent_mode: "pi_unavailable".into(),
        model: model.into(),
        reasoning_effort: reasoning_effort.into(),
        status: "needs_more_information".into(),
        message: message.into(),
        follow_up: Some("Use the main AI assistant for pre-solve planning.".into()),
        suggested_chips: Vec::new(),
        proposals: Vec::new(),
        proposed_actions: Vec::new(),
        affected_targets: Vec::new(),
        readiness_delta: String::new(),
        diagnostics: vec![WorkbenchDiagnostic {
            severity: "error".into(),
            code: "agent.legacy_local_fallback_disabled".into(),
            message: message.into(),
            detail,
        }],
    }
}

#[allow(dead_code)]
fn local_coordinator_agent(
    project: &ProjectFile,
    request: &AgentCoordinatorRequest,
    model: &str,
    reasoning_effort: &str,
) -> AgentCoordinatorResponse {
    let text = coordinator_user_text(request);
    let Some(structural) = project.structural_model.as_ref() else {
        return coordinator_needs_more(
            model,
            reasoning_effort,
            "I need an authored structural model before I can coordinate pre-solve actions.",
            vec![
                "author/import a base model".into(),
                "describe the structural system".into(),
            ],
        );
    };

    if should_coordinate_supports(&text, request) {
        let supports = proposed_support_actions(structural, &text, request);
        if supports.is_empty() {
            return coordinator_needs_more(
                model,
                reasoning_effort,
                "I need explicit support locations and restraint intent before applying support assumptions.",
                vec![
                    "record support locations only".into(),
                    "compare support fixity as design options".into(),
                    "state restrained DOFs".into(),
                ],
            );
        }
        let affected_targets = supports
            .iter()
            .map(|action| AgentCoordinatorTarget {
                kind: action.target_kind.clone(),
                id: action.target_id.clone(),
            })
            .collect::<Vec<_>>();
        let summaries = supports
            .iter()
            .map(|action| action.summary.clone())
            .collect::<Vec<_>>();
        return AgentCoordinatorResponse {
            agent_mode: "local".into(),
            model: model.into(),
            reasoning_effort: reasoning_effort.into(),
            status: "ready_to_apply".into(),
            message: format!(
                "I can make the model stable for preliminary analysis by applying these support assumptions: {}.",
                summaries.join("; ")
            ),
            follow_up: None,
            suggested_chips: Vec::new(),
            proposals: vec![AgentCoordinatorProposal {
                id: "pre-solve-support-option".into(),
                title: "Apply preliminary support option".into(),
                summary: "Add coordinated support assumptions at explicit support locations so the model can progress towards analysis readiness.".into(),
                actions: supports.clone(),
                affected_targets: affected_targets.clone(),
            }],
            proposed_actions: supports,
            affected_targets,
            readiness_delta: "Support readiness should improve after these assumptions are applied. Loads and section criteria may still need review.".into(),
            diagnostics: Vec::new(),
        };
    }

    if text.contains("section") || !parse_section_families(&text).is_empty() {
        if let Some(response) =
            coordinate_section_families(project, request, &text, model, reasoning_effort)
        {
            return response;
        }
    }

    if parse_line_load_n_per_m(&text).is_some() {
        if let Some(response) = coordinate_line_load(request, &text, model, reasoning_effort) {
            return response;
        }
    }

    coordinator_needs_more(
        model,
        reasoning_effort,
        "I can coordinate pre-solve setup, but I need a more specific instruction.",
        vec![
            "state support locations".into(),
            "select load target members".into(),
            "record hard constraints".into(),
        ],
    )
}

#[allow(dead_code)]
fn coordinator_needs_more(
    model: &str,
    reasoning_effort: &str,
    message: &str,
    suggested_chips: Vec<String>,
) -> AgentCoordinatorResponse {
    AgentCoordinatorResponse {
        agent_mode: "local".into(),
        model: model.into(),
        reasoning_effort: reasoning_effort.into(),
        status: "needs_more_information".into(),
        message: message.into(),
        follow_up: Some(
            "Tell the agent what assumption to make, or ask it to recommend a conservative preliminary setup."
                .into(),
        ),
        suggested_chips,
        proposals: Vec::new(),
        proposed_actions: Vec::new(),
        affected_targets: Vec::new(),
        readiness_delta: String::new(),
        diagnostics: Vec::new(),
    }
}

#[allow(dead_code)]
fn coordinator_user_text(request: &AgentCoordinatorRequest) -> String {
    let mut parts = Vec::new();
    parts.push(request.instruction.clone());
    if let Some(id) = &request.focus_comment_id {
        parts.push(id.clone());
    }
    for target in &request.focus_targets {
        parts.push(format!("{} {}", target.kind, target.id));
    }
    for message in &request.messages {
        if message.author == "user" {
            parts.push(message.text.clone());
        }
    }
    parts.join(" ").to_ascii_lowercase()
}

#[allow(dead_code)]
fn should_coordinate_supports(text: &str, request: &AgentCoordinatorRequest) -> bool {
    text.contains("support")
        || text.contains("stable")
        || text.contains("stability")
        || request
            .focus_comment_id
            .as_deref()
            .unwrap_or_default()
            .contains("support")
        || request
            .focus_targets
            .iter()
            .any(|target| target.kind == "node" || target.kind == "support")
}

#[allow(dead_code)]
fn proposed_support_actions(
    model: &StructuralModel,
    text: &str,
    request: &AgentCoordinatorRequest,
) -> Vec<AgentProposedAction> {
    let Some(support_type) = explicit_support_type_from_text(text) else {
        return Vec::new();
    };
    let target_nodes = explicit_support_target_nodes(model, text, request);
    if target_nodes.is_empty() {
        return Vec::new();
    }

    target_nodes
        .iter()
        .map(|node| {
            let dofs = support_dofs_for_type(support_type);
            AgentProposedAction {
                action_kind: "add_support".into(),
                target_kind: "node".into(),
                target_id: node.id.clone(),
                field: "structural_model.supports".into(),
                value: json!({
                    "supportType": support_type,
                    "ux": dofs.0,
                    "uy": dofs.1,
                    "uz": dofs.2,
                    "rx": dofs.3,
                    "ry": dofs.4,
                    "rz": dofs.5,
                }),
                summary: format!("Add {support_type} support at node {}.", node.id),
            }
        })
        .collect()
}

fn explicit_support_target_nodes<'a>(
    model: &'a StructuralModel,
    text: &str,
    request: &AgentCoordinatorRequest,
) -> Vec<&'a StructuralNode> {
    let mut ids = std::collections::BTreeSet::new();
    for target in &request.focus_targets {
        if target.kind == "node" {
            ids.insert(target.id.clone());
        }
    }
    for node in &model.nodes {
        let node_id = node.id.to_ascii_lowercase();
        if text.contains(&node_id) {
            ids.insert(node.id.clone());
        }
    }
    ids.into_iter()
        .filter_map(|id| model.node_by_id(&id))
        .collect()
}

fn explicit_support_type_from_text(text: &str) -> Option<&'static str> {
    if text.contains("fixed") {
        Some("fixed")
    } else if text.contains("roller") {
        Some("roller")
    } else if text.contains("pinned") || text.contains("pin ") || text.contains("pin-base") {
        Some("pinned")
    } else {
        None
    }
}

fn support_dofs_for_type(kind: &str) -> (bool, bool, bool, bool, bool, bool) {
    match kind {
        "fixed" => (true, true, true, true, true, true),
        "roller" => (false, true, false, false, false, false),
        _ => (true, true, false, false, false, false),
    }
}

#[allow(dead_code)]
fn coordinate_section_families(
    project: &ProjectFile,
    request: &AgentCoordinatorRequest,
    text: &str,
    model: &str,
    reasoning_effort: &str,
) -> Option<AgentCoordinatorResponse> {
    let allowed_families = parse_section_families(text);
    let wants_agent_choose = mentions_any(text, &["let agent choose", "agent choose", "choose"]);
    if allowed_families.is_empty() && !wants_agent_choose {
        return None;
    }
    let draft = planning_draft(project);
    let structural = project.structural_model.as_ref()?;
    let report = build_coordination_report(project, &draft, structural).ok()?;
    let requested_target = request
        .focus_targets
        .iter()
        .find(|target| target.kind == "coordination_group")
        .map(|target| target.id.clone());
    let group = requested_target
        .as_deref()
        .and_then(|id| resolve_coordination_group_reference(&report.groups, id))
        .or_else(|| resolve_coordination_group_reference(&report.groups, text))
        .or_else(|| report.groups.first())?;
    let allowed_families = if allowed_families.is_empty() {
        group.recommended_section_families.clone()
    } else {
        allowed_families
    };
    if allowed_families.is_empty() {
        return None;
    }
    let strategy = if wants_agent_choose {
        AGENT_JUSTIFIED_SECTION_SELECTION_POLICY
    } else {
        "user_allowed_families"
    };
    let action = AgentProposedAction {
        action_kind: "update_planning_draft".into(),
        target_kind: "coordination_group".into(),
        target_id: group.id.clone(),
        field: "coordinationGroup.allowedSectionFamilies".into(),
        value: json!({
            "allowedSectionFamilies": allowed_families,
            "sectionSelectionPolicy": strategy,
            "sectionSelectionStrategy": strategy,
        }),
        summary: format!("Allow {} for {}.", allowed_families.join(", "), group.label),
    };
    let affected = vec![AgentCoordinatorTarget {
        kind: "coordination_group".into(),
        id: group.id.clone(),
    }];
    Some(AgentCoordinatorResponse {
        agent_mode: "local".into(),
        model: model.into(),
        reasoning_effort: reasoning_effort.into(),
        status: "ready_to_apply".into(),
        message: format!(
            "I can record {} as allowed catalogue section families for {}. Exact section sizes stay for the solve/design step.",
            allowed_families.join(", "),
            group.label
        ),
        follow_up: None,
        suggested_chips: Vec::new(),
        proposals: vec![AgentCoordinatorProposal {
            id: format!("pre-solve-section-family-{}", group.id),
            title: format!("Set section families for {}", group.label),
            summary: "Store section-family criteria for later coordinated sizing.".into(),
            actions: vec![action.clone()],
            affected_targets: affected.clone(),
        }],
        proposed_actions: vec![action],
        affected_targets: affected,
        readiness_delta:
            "Section-family criteria will be recorded; final sizes are still chosen after analysis/design."
                .into(),
        diagnostics: Vec::new(),
    })
}

fn resolve_coordination_group_reference<'a>(
    groups: &'a [CoordinationGroup],
    text: &str,
) -> Option<&'a CoordinationGroup> {
    let lower = text.to_ascii_lowercase();
    groups
        .iter()
        .find(|group| group.id.eq_ignore_ascii_case(text))
        .or_else(|| {
            groups.iter().find(|group| {
                lower.contains(&group.role.to_ascii_lowercase())
                    || lower.contains(&group.label.to_ascii_lowercase())
                    || group
                        .member_group_ids
                        .iter()
                        .any(|id| lower.contains(&id.to_ascii_lowercase()))
            })
        })
}

#[allow(dead_code)]
fn coordinate_line_load(
    request: &AgentCoordinatorRequest,
    text: &str,
    model: &str,
    reasoning_effort: &str,
) -> Option<AgentCoordinatorResponse> {
    let load_n_per_m = parse_line_load_n_per_m(text)?;
    let load_label = format_metric_line_load(load_n_per_m);
    let target = request
        .focus_targets
        .iter()
        .find(|target| target.kind == "member_group" || target.kind == "member")
        .cloned()?;
    let action = AgentProposedAction {
        action_kind: "add_load".into(),
        target_kind: target.kind.clone(),
        target_id: target.id.clone(),
        field: "structural_model.loads".into(),
        value: json!({
            "kind": "uniform_line",
            "magnitude": {
                "value": load_n_per_m,
                "quantityKind": "line_load",
                "canonicalUnit": "N/m"
            },
            "loadCaseId": "gravity",
            "direction": { "x": 0.0, "y": -1.0, "z": 0.0 }
        }),
        summary: format!("Add {load_label} downward gravity line load."),
    };
    Some(AgentCoordinatorResponse {
        agent_mode: "local".into(),
        model: model.into(),
        reasoning_effort: reasoning_effort.into(),
        status: "ready_to_apply".into(),
        message: format!(
            "I can apply a preliminary {load_label} downward gravity line load to {} {}.",
            target.kind, target.id
        ),
        follow_up: None,
        suggested_chips: Vec::new(),
        proposals: vec![AgentCoordinatorProposal {
            id: "pre-solve-gravity-load".into(),
            title: "Apply preliminary gravity load".into(),
            summary: "Add the stated preliminary line load to the selected load-carrying member."
                .into(),
            actions: vec![action.clone()],
            affected_targets: vec![target.clone()],
        }],
        proposed_actions: vec![action],
        affected_targets: vec![target],
        readiness_delta: "Load readiness should improve after this load is applied.".into(),
        diagnostics: Vec::new(),
    })
}

fn selected_reply_text(
    project: &ProjectFile,
    surface: &str,
    session_id: Option<&str>,
    selected_ids: &[String],
) -> Option<String> {
    let session = project.agent_state.sessions.iter().find(|session| {
        session.surface == surface && session_id.map(|id| id == session.id).unwrap_or(true)
    })?;
    let last_assistant = session
        .messages
        .iter()
        .rev()
        .find(|message| message.author == "assistant")?;
    let replies = selected_ids
        .iter()
        .filter_map(|id| {
            id.strip_prefix("reply-")
                .and_then(|index| index.parse::<usize>().ok())
                .and_then(|index| last_assistant.suggested_replies.get(index))
                .or_else(|| {
                    id.strip_prefix("group-reply-").and_then(|rest| {
                        let (group_index, reply_index) = rest.split_once('-')?;
                        last_assistant
                            .suggested_reply_groups
                            .get(group_index.parse::<usize>().ok()?)?
                            .replies
                            .get(reply_index.parse::<usize>().ok()?)
                    })
                })
                .or_else(|| {
                    last_assistant
                        .suggested_replies
                        .iter()
                        .find(|reply| *reply == id)
                })
                .or_else(|| {
                    last_assistant
                        .suggested_reply_groups
                        .iter()
                        .flat_map(|group| group.replies.iter())
                        .find(|reply| *reply == id)
                })
        })
        .cloned()
        .collect::<Vec<_>>();
    (!replies.is_empty()).then(|| replies.join(" "))
}

fn ensure_agent_settings(project: &mut ProjectFile) -> bool {
    let mut changed = false;
    for surface in ["default", "pre_solve", "comment_review"] {
        let settings = project
            .agent_state
            .settings_by_surface
            .entry(surface.into())
            .or_insert_with(|| {
                changed = true;
                AgentModelSettings::default()
            });
        if settings.model == "gpt-5.5" && settings.reasoning_effort == "medium" {
            settings.reasoning_effort = "low".into();
            changed = true;
        }
    }
    changed
}

fn agent_settings_for_surface(project: &ProjectFile, surface: &str) -> AgentModelSettings {
    project
        .agent_state
        .settings_by_surface
        .get(surface)
        .or_else(|| project.agent_state.settings_by_surface.get("default"))
        .or_else(|| project.agent_state.settings_by_surface.get("pre_solve"))
        .cloned()
        .unwrap_or_default()
}

fn validated_agent_settings_for_surface_from_models(
    project: &mut ProjectFile,
    surface: &str,
    _models: &[AgentModelOption],
) -> AgentModelSettings {
    agent_settings_for_surface(project, surface)
}

fn validated_agent_settings_for_surface(
    project: &mut ProjectFile,
    surface: &str,
) -> AgentModelSettings {
    let current = agent_settings_for_surface(project, surface);
    let Ok(catalogue) = pi_catalogue() else {
        return current;
    };
    validated_agent_settings_for_surface_from_models(project, surface, &catalogue.models)
}

fn agent_session_title(surface: &str) -> String {
    if surface == "pre_solve" {
        return "Pre-solve agent".into();
    }
    if let Some(scheme_id) = scheme_surface_id(surface) {
        return format!("Design option chat: {}", humanize_scheme_id(scheme_id));
    }
    "Agent chat".into()
}

fn scheme_surface_id(surface: &str) -> Option<&str> {
    surface
        .strip_prefix("scheme:")
        .filter(|id| !id.trim().is_empty())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingSchemeDecision {
    scheme_id: String,
    prompt: String,
}

fn pending_scheme_analysis_decisions(project: &ProjectFile) -> Vec<PendingSchemeDecision> {
    project
        .agent_state
        .sessions
        .iter()
        .filter_map(pending_scheme_decision_from_session)
        .collect()
}

fn pending_scheme_decision_from_session(session: &AgentSession) -> Option<PendingSchemeDecision> {
    let scheme_id = scheme_surface_id(&session.surface)?.to_string();
    if let Some(question) = &session.current_question {
        if !question.options.is_empty() {
            return Some(PendingSchemeDecision {
                scheme_id,
                prompt: trim_decision_prompt(&question.prompt),
            });
        }
    }

    let last_user_index = session
        .messages
        .iter()
        .enumerate()
        .rev()
        .find(|(_, message)| message.author == "user")
        .map(|(index, _)| index);
    session
        .messages
        .iter()
        .enumerate()
        .rev()
        .find(|(index, message)| {
            message.author == "assistant"
                && !matches!(
                    message.mode.as_deref(),
                    Some("deterministic") | Some("local")
                )
                && last_user_index.map_or(true, |last_user| *index > last_user)
                && !message.suggested_replies.is_empty()
                && is_blocking_decision_prompt(&message.text)
        })
        .map(|(_, message)| PendingSchemeDecision {
            scheme_id,
            prompt: trim_decision_prompt(&message.text),
        })
}

fn is_blocking_decision_prompt(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    text.contains('?')
        || lower.contains("may this option")
        || lower.contains("should this option")
        || lower.contains("allow")
        || lower.contains("approve")
        || lower.contains("apply this")
        || lower.contains("refine")
}

fn trim_decision_prompt(text: &str) -> String {
    let trimmed = text
        .trim()
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .map(str::trim)
        .unwrap_or("This design option has an unresolved decision.");
    if trimmed.chars().count() > 180 {
        format!("{}...", trimmed.chars().take(177).collect::<String>())
    } else {
        trimmed.into()
    }
}

fn humanize_scheme_id(id: &str) -> String {
    id.trim()
        .trim_start_matches("scheme-")
        .split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn ensure_agent_session<'a>(project: &'a mut ProjectFile, surface: &str) -> &'a mut AgentSession {
    ensure_agent_settings(project);
    let now = fraia_core::utils::iso_now();
    let title = agent_session_title(surface);
    let index = project
        .agent_state
        .sessions
        .iter()
        .position(|session| session.surface == surface)
        .unwrap_or_else(|| {
            project.agent_state.sessions.push(AgentSession {
                id: format!("session-{surface}"),
                surface: surface.into(),
                title,
                status: "active".into(),
                messages: Vec::new(),
                plan_items: Vec::new(),
                current_question: None,
                created_at: now.clone(),
                updated_at: now.clone(),
            });
            project.agent_state.sessions.len() - 1
        });
    let session = &mut project.agent_state.sessions[index];
    session.current_question = None;
    session.plan_items.clear();
    session.messages.retain(|message| {
        !matches!(
            message.mode.as_deref(),
            Some("deterministic") | Some("local") | Some("pi_unavailable")
        )
    });
    session
}

fn initial_base_model_brief(project: &ProjectFile, session_id: &str) -> BaseModelBrief {
    let structural = materialize_project_structural_model(project);
    let now = fraia_core::utils::iso_now();
    let current_understanding = structural
        .as_ref()
        .map(|model| {
            let mut role_counts = BTreeMap::<String, usize>::new();
            for member in &model.members {
                *role_counts.entry(member.role.clone()).or_default() += 1;
            }
            let roles = role_counts
                .into_iter()
                .map(|(role, count)| format!("{count} {role}"))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "Opened base model contains {} nodes, {} members{}, {} supports, and {} loads. The structural meaning is not yet confirmed.",
                model.nodes.len(),
                model.members.len(),
                if roles.is_empty() { String::new() } else { format!(" ({roles})") },
                model.supports.len(),
                model.loads.len()
            )
        })
        .unwrap_or_else(|| "No authored structural model exists yet.".into());

    let mut open_questions: Vec<String> = vec![
        "Confirm what the opened geometry represents.".into(),
        "Confirm whether design options should treat the geometry as standalone, representative/repeated, or connected to existing structure.".into(),
        "Confirm where the model can be supported at concept level.".into(),
        "Confirm self weight is included by default and identify any additional concept-stage load assumptions.".into(),
        "Confirm any hard constraints or no-go zones, or state that there are none.".into(),
    ];
    if let Some(model) = structural.as_ref() {
        if !model.supports.is_empty() {
            open_questions.retain(|question| !question.contains("supported"));
        }
        if !model.loads.is_empty() {
            open_questions.retain(|question| !question.contains("load assumptions"));
        }
    }
    BaseModelBrief {
        version: 1,
        session_id: session_id.into(),
        current_understanding,
        confirmed_intent: Vec::new(),
        open_questions: open_questions.clone(),
        soft_assumptions: Vec::new(),
        schema_guidance: vec![
            "Briefing answers are fixed boundaries for design-option generation, not design-option choices.".into(),
            "Support locations, no-go zones, and hard constraints from the brief are fixed unless the user later changes them.".into(),
            "Design-option generation should vary support kind, section family, member grouping, base fixity, and stability strategy inside the confirmed briefing boundary unless explicitly fixed.".into(),
        ],
        do_not_decide_yet: vec![
            "Detailed member sizing belongs to solve/design after design options.".into(),
            "Support kind, base fixity, and stability strategy remain design-option alternatives unless the brief explicitly fixes them.".into(),
            "Section family and member grouping remain design-option alternatives unless the brief names them as hard constraints.".into(),
        ],
        visual_intent: BaseModelBriefVisualIntent::default(),
        readiness: BaseModelBriefReadiness {
            ready_for_schemas: open_questions.is_empty(),
            unresolved_topics: open_questions,
            manual_override_allowed: true,
        },
        updated_at: now,
    }
}

fn ensure_base_model_brief(project: &mut ProjectFile) {
    if project.base_model_brief.is_some() {
        return;
    }
    let session = ensure_agent_session(project, "pre_solve");
    let session_id = session.id.clone();
    project.base_model_brief = Some(initial_base_model_brief(project, &session_id));
}

fn push_unique_brief_question(brief: &mut BaseModelBrief, question: &str) {
    if !brief.open_questions.iter().any(|item| item == question) {
        brief.open_questions.push(question.into());
    }
    if !brief
        .readiness
        .unresolved_topics
        .iter()
        .any(|item| item == question)
    {
        brief.readiness.unresolved_topics.push(question.into());
    }
}

fn remove_brief_question(brief: &mut BaseModelBrief, question: &str) {
    brief.open_questions.retain(|item| item != question);
    brief
        .readiness
        .unresolved_topics
        .retain(|item| item != question);
}

const POINT_LOAD_MAGNITUDE_QUESTION: &str =
    "Confirm the point-load magnitude before moving to the next briefing topic.";
const LINE_LOAD_MAGNITUDE_QUESTION: &str =
    "Confirm the line-load magnitude before moving to the next briefing topic.";
const LEGACY_SUPPORT_VISUAL_INTENT_QUESTION: &str = "Confirm the exact node ids for any provisional support locations before showing them in the viewport.";
const LEGACY_LOAD_VISUAL_INTENT_QUESTION: &str = "Confirm exact typed load target, magnitude where needed, and direction before showing provisional loads in the viewport.";

fn brief_visual_diagnostic(
    message: impl Into<String>,
    detail: impl Into<String>,
) -> WorkbenchDiagnostic {
    WorkbenchDiagnostic {
        severity: "warning".into(),
        code: "base_model_brief_visual_intent".into(),
        message: message.into(),
        detail: Some(detail.into()),
    }
}

fn valid_brief_load_target(
    target: &BaseModelBriefLoadTarget,
    node_ids: &BTreeSet<String>,
    member_ids: &BTreeSet<String>,
) -> bool {
    match target.kind.as_str() {
        "all_members" => !member_ids.is_empty(),
        "member" => target
            .member_id
            .as_ref()
            .is_some_and(|id| member_ids.contains(id)),
        "node" => target
            .node_id
            .as_ref()
            .is_some_and(|id| node_ids.contains(id)),
        _ => false,
    }
}

fn valid_brief_load_direction(
    direction: &BaseModelBriefLoadDirection,
    node_ids: &BTreeSet<String>,
) -> bool {
    match direction.kind.as_str() {
        "toward_node" => {
            let from = direction.from_node.as_ref();
            let to = direction.to_node.as_ref();
            matches!((from, to), (Some(from), Some(to)) if from != to && node_ids.contains(from) && node_ids.contains(to))
        }
        "vector" => {
            let x = direction.x.unwrap_or(0.0);
            let y = direction.y.unwrap_or(0.0);
            let z = direction.z.unwrap_or(0.0);
            (x * x + y * y + z * z).sqrt() > 1e-9
        }
        _ => false,
    }
}

fn validate_base_model_brief_visual_intent(
    project: &ProjectFile,
    brief: &mut BaseModelBrief,
) -> Vec<WorkbenchDiagnostic> {
    remove_brief_question(brief, LEGACY_SUPPORT_VISUAL_INTENT_QUESTION);
    remove_brief_question(brief, LEGACY_LOAD_VISUAL_INTENT_QUESTION);

    let Some(model) = project.structural_model.as_ref() else {
        brief.visual_intent = BaseModelBriefVisualIntent::default();
        return Vec::new();
    };
    let node_ids: BTreeSet<String> = model.nodes.iter().map(|node| node.id.clone()).collect();
    let member_ids: BTreeSet<String> = model
        .members
        .iter()
        .map(|member| member.id.clone())
        .collect();
    let mut diagnostics = Vec::new();

    let mut valid_support_locations = Vec::new();
    for mut support in std::mem::take(&mut brief.visual_intent.support_locations) {
        if node_ids.contains(&support.target_node) {
            support.status = "location_only".into();
            valid_support_locations.push(support);
        } else {
            diagnostics.push(brief_visual_diagnostic(
                "Dropped Base Model support-location visual intent with an unknown node.",
                format!(
                    "Support visual intent `{}` referenced node `{}`.",
                    support.id, support.target_node
                ),
            ));
        }
    }
    brief.visual_intent.support_locations = valid_support_locations;

    let mut valid_loads = Vec::new();
    let mut valid_point_load_count = 0usize;
    let mut valid_line_load_count = 0usize;
    let mut dropped_point_load_missing_magnitude = false;
    let mut dropped_line_load_missing_magnitude = false;
    for load in std::mem::take(&mut brief.visual_intent.loads) {
        let target_valid = valid_brief_load_target(&load.target, &node_ids, &member_ids);
        let direction_valid = load
            .direction
            .as_ref()
            .is_some_and(|direction| valid_brief_load_direction(direction, &node_ids));
        let valid = match load.kind.as_str() {
            "self_weight" => target_valid,
            "point" => target_valid && load.magnitude_n.is_some() && direction_valid,
            "uniform_line" => {
                load.target.kind == "member"
                    && target_valid
                    && load.magnitude_n_per_m.is_some()
                    && direction_valid
            }
            _ => false,
        };
        if valid {
            if load.kind == "point" {
                valid_point_load_count += 1;
            }
            if load.kind == "uniform_line" {
                valid_line_load_count += 1;
            }
            valid_loads.push(load);
        } else {
            if load.kind == "point" && target_valid && direction_valid && load.magnitude_n.is_none()
            {
                dropped_point_load_missing_magnitude = true;
            }
            if load.kind == "uniform_line"
                && load.target.kind == "member"
                && target_valid
                && direction_valid
                && load.magnitude_n_per_m.is_none()
            {
                dropped_line_load_missing_magnitude = true;
            }
            diagnostics.push(brief_visual_diagnostic(
                "Dropped Base Model load visual intent because its typed reference was incomplete or invalid.",
                format!("Load visual intent `{}` was `{}`.", load.id, load.kind),
            ));
        }
    }
    brief.visual_intent.loads = valid_loads;
    if dropped_point_load_missing_magnitude {
        push_unique_brief_question(brief, POINT_LOAD_MAGNITUDE_QUESTION);
        brief.readiness.ready_for_schemas = false;
    } else if valid_point_load_count > 0 {
        remove_brief_question(brief, POINT_LOAD_MAGNITUDE_QUESTION);
    }
    if dropped_line_load_missing_magnitude {
        push_unique_brief_question(brief, LINE_LOAD_MAGNITUDE_QUESTION);
        brief.readiness.ready_for_schemas = false;
    } else if valid_line_load_count > 0 {
        remove_brief_question(brief, LINE_LOAD_MAGNITUDE_QUESTION);
    }

    diagnostics
}

fn pre_solve_blocking_reply(brief: &BaseModelBrief) -> Option<(String, Vec<String>)> {
    let missing_point_load_magnitude = brief
        .open_questions
        .iter()
        .chain(brief.readiness.unresolved_topics.iter())
        .any(|question| question == POINT_LOAD_MAGNITUDE_QUESTION);
    if !missing_point_load_magnitude {
        return None;
    }
    Some((
        "I can’t finish the load part of the Base Model Brief yet: the point-load target and direction are clear, but the magnitude is missing. What magnitude should Fraia use for that point load?".into(),
        vec![
            "Use 10 kN for the point load.".into(),
            "Use 20 kN for the point load.".into(),
            "Use 50 kN for the point load.".into(),
            "Leave the point load out for now.".into(),
            "I need to derive it from tributary loading.".into(),
        ],
    ))
}

fn build_agent_provider_status(
    project: &ProjectFile,
    surface: &str,
) -> AgentProviderStatusResponse {
    let mut diagnostics = Vec::new();
    let catalogue = match pi_catalogue() {
        Ok(catalogue) => catalogue,
        Err(error) => {
            diagnostics.push(WorkbenchDiagnostic {
                severity: "error".into(),
                code: "agent.pi_runtime_unavailable".into(),
                message: "Could not load the Pi provider and model catalogue.".into(),
                detail: Some(format!("{error:#}")),
            });
            PiCatalogueResponse {
                providers: Vec::new(),
                models: Vec::new(),
                catalogue: AgentCatalogueFreshness::default(),
                secure_credential_storage_available: false,
            }
        }
    };
    let current = project
        .agent_state
        .settings_by_surface
        .get(surface)
        .cloned()
        .unwrap_or_else(|| agent_settings_for_surface(project, surface));
    let selected_available = catalogue.models.iter().any(|model| {
        model.provider_id == current.provider_id && model.slug == current.model && model.available
    });
    if !selected_available {
        diagnostics.push(WorkbenchDiagnostic {
            severity: "warning".into(),
            code: "agent.selected_model_unavailable".into(),
            message: format!(
                "Selected AI model {}/{} is unavailable.",
                current.provider_id, current.model
            ),
            detail: Some(
                "Choose an available authenticated model before starting another AI turn.".into(),
            ),
        });
    }
    AgentProviderStatusResponse {
        providers: catalogue.providers,
        models: catalogue.models,
        selected_provider_id: current.provider_id,
        selected_model: current.model,
        selected_reasoning_effort: current.reasoning_effort,
        catalogue: catalogue.catalogue,
        secure_credential_storage_available: catalogue.secure_credential_storage_available,
        diagnostics,
    }
}

#[cfg(test)]
fn validate_agent_settings_from_models(
    models: &[AgentModelOption],
    provider_id: Option<&str>,
    model: Option<&str>,
    reasoning: Option<&str>,
) -> AgentModelSettings {
    let requested_provider = provider_id.unwrap_or("openai-codex").trim();
    let requested_model = model.unwrap_or_default().trim();
    let selected_model = models.iter().find(|candidate| {
        candidate.provider_id == requested_provider && candidate.slug == requested_model
    });
    let Some(selected_model) = selected_model else {
        return AgentModelSettings {
            provider_id: requested_provider.into(),
            model: requested_model.into(),
            reasoning_effort: reasoning.unwrap_or("low").into(),
        };
    };
    let requested_reasoning = reasoning.unwrap_or_default().trim().to_ascii_lowercase();
    let selected_reasoning = selected_model
        .supported_reasoning_levels
        .iter()
        .find(|candidate| candidate.effort == requested_reasoning)
        .map(|candidate| candidate.effort.clone())
        .or_else(|| {
            selected_model
                .supported_reasoning_levels
                .iter()
                .find(|candidate| candidate.effort == "low")
                .map(|candidate| candidate.effort.clone())
        })
        .unwrap_or_else(|| selected_model.default_reasoning_level.clone());
    AgentModelSettings {
        provider_id: selected_model.provider_id.clone(),
        model: selected_model.slug.clone(),
        reasoning_effort: selected_reasoning,
    }
}

fn append_pi_agent_turn(
    project: &mut ProjectFile,
    surface: &str,
    user_text: &str,
    request_id: Option<&str>,
) -> Option<AgentMessage> {
    let now = fraia_core::utils::iso_now();
    let settings = validated_agent_settings_for_surface(project, surface);
    let session_id = {
        let session = ensure_agent_session(project, surface);
        if !user_text.trim().is_empty() {
            session.messages.push(AgentMessage {
                author: "user".into(),
                text: user_text.trim().into(),
                created_at: now.clone(),
                mode: None,
                model: None,
                provider_id: None,
                reasoning_effort: None,
                catalogue_refreshed_at: None,
                suggested_replies: Vec::new(),
                suggested_reply_groups: Vec::new(),
                plan_summary: None,
                proposed_actions: Vec::new(),
            });
        }
        session.id.clone()
    };
    match run_pi_session_agent(project, surface, &settings, request_id) {
        Ok(mut response) => {
            let response_ai_provenance = AiProvenance {
                provider_id: response.provider_id.clone(),
                model_id: response.model_id.clone(),
                reasoning_effort: response.reasoning_effort.clone(),
                catalogue_refreshed_at: response.catalogue_refreshed_at.clone(),
            };
            let mut blocking_reply = None;
            let mut replacement_notes = Vec::new();
            let mut replacement_option_ids = Vec::new();
            if surface == "pre_solve" {
                if let Some(mut brief) = response.base_model_brief.clone() {
                    brief.version = 1;
                    brief.session_id = session_id.clone();
                    brief.updated_at = fraia_core::utils::iso_now();
                    brief.readiness.manual_override_allowed = true;
                    response
                        .diagnostics
                        .extend(validate_base_model_brief_visual_intent(project, &mut brief));
                    blocking_reply = pre_solve_blocking_reply(&brief);
                    project.base_model_brief = Some(brief);
                }
            }
            if scheme_surface_id(surface).is_some() {
                let mut draft = planning_draft(project);
                let mut draft_changed = false;
                for action in &response.proposed_actions {
                    if action.action_kind == "update_planning_draft"
                        && action.field == "coordination.designOptionReplacement"
                    {
                        match apply_agent_action_to_draft(project, &mut draft, action) {
                            Ok(summary) => {
                                draft_changed = true;
                                if let Some(replacement_id) = action
                                    .value
                                    .get("replacementDesignOptionIntent")
                                    .and_then(|intent| intent.get("id"))
                                    .and_then(Value::as_str)
                                {
                                    replacement_option_ids.push(replacement_id.to_string());
                                }
                                replacement_notes.push(format!(
                                    "Fraia recorded this as a replacement: {summary}."
                                ));
                                response.diagnostics.push(WorkbenchDiagnostic {
                                    severity: "info".into(),
                                    code: "agent.design_option.replacement_persisted".into(),
                                    message: summary,
                                    detail: None,
                                });
                            }
                            Err(error) => {
                                replacement_notes.push(
                                    "Fraia could not record the replacement option. The original option is unchanged.".into(),
                                );
                                response.diagnostics.push(WorkbenchDiagnostic {
                                    severity: "error".into(),
                                    code: "agent.design_option.replacement_failed".into(),
                                    message:
                                        "Could not persist the requested design-option replacement."
                                            .into(),
                                    detail: Some(format!("{error:#}")),
                                });
                            }
                        }
                    }
                }
                if draft_changed {
                    project.planning_draft = Some(draft);
                    sync_active_design_option_revisions(project);
                    let active_batch_id = project.design_option_decisions.active_batch_id.clone();
                    if let Some(batch) = project
                        .design_option_decisions
                        .batches
                        .iter_mut()
                        .find(|batch| Some(&batch.id) == active_batch_id.as_ref())
                    {
                        for revision in &mut batch.option_revisions {
                            if replacement_option_ids.contains(&revision.option_id) {
                                revision.ai_provenance = Some(response_ai_provenance.clone());
                            }
                        }
                    }
                    project.updated_at = Some(fraia_core::utils::iso_now());
                }
                response.proposed_actions.clear();
            }
            let _diagnostics_are_structured_only = response.diagnostics.len();
            let message_source = blocking_reply
                .as_ref()
                .map(|(message, _)| message.as_str())
                .unwrap_or(&response.message);
            let mut message = sanitize_agent_surface_text(message_source);
            if !replacement_notes.is_empty() {
                if !message.trim().is_empty() {
                    message.push_str("\n\n");
                }
                message.push_str(&replacement_notes.join("\n\n"));
            }
            let design_option_surface = scheme_surface_id(surface).is_some();
            let suggested_reply_source = blocking_reply
                .map(|(_, suggested_replies)| suggested_replies)
                .unwrap_or(response.suggested_replies);
            let suggested_replies = if design_option_surface {
                Vec::new()
            } else {
                suggested_reply_source
                    .into_iter()
                    .map(|reply| sanitize_agent_surface_text(&reply))
                    .collect()
            };
            let suggested_reply_groups = if design_option_surface {
                Vec::new()
            } else {
                response
                    .suggested_reply_groups
                    .into_iter()
                    .filter_map(sanitize_agent_suggested_reply_group)
                    .collect()
            };
            let plan_summary = sanitize_agent_surface_text(&response.plan_summary);
            Some(AgentMessage {
                author: "assistant".into(),
                text: message,
                created_at: now,
                mode: Some("pi".into()),
                model: Some(response.model_id),
                provider_id: Some(response.provider_id),
                reasoning_effort: Some(response.reasoning_effort),
                catalogue_refreshed_at: response.catalogue_refreshed_at,
                suggested_replies,
                suggested_reply_groups,
                plan_summary: (!plan_summary.trim().is_empty()).then_some(plan_summary),
                proposed_actions: response
                    .proposed_actions
                    .into_iter()
                    .map(agent_action_to_state)
                    .collect(),
            })
        }
        Err(error) => Some(AgentMessage {
            author: "assistant".into(),
            text: format_pi_agent_error(&error),
            created_at: now,
            mode: Some("pi_unavailable".into()),
            model: Some(settings.model),
            provider_id: Some(settings.provider_id),
            reasoning_effort: Some(settings.reasoning_effort),
            catalogue_refreshed_at: None,
            suggested_replies: Vec::new(),
            suggested_reply_groups: Vec::new(),
            plan_summary: None,
            proposed_actions: Vec::new(),
        }),
    }
}

fn format_pi_agent_error(error: &anyhow::Error) -> String {
    let detail = summarize_pi_error(&format!("{error:#}"));
    format!(
        "The selected AI provider could not complete this turn.\n\n{detail}\n\nCheck the provider connection and model selection, then try again."
    )
}

fn summarize_pi_error(raw: &str) -> String {
    let raw = raw.trim();
    let without_prompt = raw.split("\n--------\nuser\n").next().unwrap_or(raw);
    if let Some(index) = without_prompt.find("invalid_json_schema") {
        let window = &without_prompt[index..without_prompt.len().min(index + 900)];
        return format!("Schema error: {}", compact_error_text(window));
    }
    if let Some(index) = without_prompt.find("\"message\":") {
        let window = &without_prompt[index..without_prompt.len().min(index + 900)];
        return compact_error_text(window);
    }
    let compact = compact_error_text(without_prompt);
    if compact.chars().count() > 900 {
        format!("{}...", compact.chars().take(897).collect::<String>())
    } else {
        compact
    }
}

fn compact_error_text(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn sanitize_agent_surface_text(text: &str) -> String {
    let mut cleaned = text.replace("Diagnostics:", "");
    for prefix in [
        "builder.frame.review::",
        "builder.frame.authored::",
        "builder.frame.planning::",
        "builder.beam.authored::",
    ] {
        let mut search_from = 0;
        while let Some(relative_start) = cleaned[search_from..].find(prefix) {
            let start = search_from + relative_start;
            let mut end = start + prefix.len();
            for (offset, ch) in cleaned[end..].char_indices() {
                if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | ':') {
                    end = start + prefix.len() + offset + ch.len_utf8();
                } else {
                    break;
                }
            }
            let replacement = display_label_for_internal_model_id(&cleaned[start..end]);
            let replacement_len = replacement.len();
            cleaned.replace_range(start..end, &replacement);
            search_from = start + replacement_len;
        }
    }
    sanitize_bare_model_labels(&cleaned)
}

fn display_label_for_internal_model_id(id: &str) -> String {
    let suffix = id.rsplit("::").next().unwrap_or(id).trim();
    let lower = suffix.to_ascii_lowercase();
    if let Some(number) = lower.strip_prefix('n') {
        if !number.is_empty() && number.chars().all(|ch| ch.is_ascii_digit()) {
            return format!("N{number}");
        }
    }
    if let Some(number) = lower.strip_prefix('m').or_else(|| lower.strip_prefix('e')) {
        if !number.is_empty() && number.chars().all(|ch| ch.is_ascii_digit()) {
            return format!("M{number}");
        }
    }
    suffix.replace(['_', '-'], " ")
}

fn sanitize_bare_model_labels(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut token = String::new();
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            token.push(ch);
        } else {
            output.push_str(&display_label_for_bare_model_token(&token));
            token.clear();
            output.push(ch);
        }
    }
    output.push_str(&display_label_for_bare_model_token(&token));
    output
}

fn display_label_for_bare_model_token(token: &str) -> String {
    if token.len() < 2 {
        return token.to_string();
    }
    let lower = token.to_ascii_lowercase();
    if let Some(number) = lower.strip_prefix('n') {
        if !number.is_empty() && number.chars().all(|ch| ch.is_ascii_digit()) {
            return format!("N{number}");
        }
    }
    if let Some(number) = lower.strip_prefix('e') {
        if !number.is_empty() && number.chars().all(|ch| ch.is_ascii_digit()) {
            return format!("M{number}");
        }
    }
    token.to_string()
}

fn sanitize_agent_suggested_reply_group(
    group: AgentSuggestedReplyGroup,
) -> Option<AgentSuggestedReplyGroup> {
    let title = sanitize_agent_surface_text(&group.title);
    let prompt = sanitize_agent_surface_text(&group.prompt);
    let replies = group
        .replies
        .into_iter()
        .map(|reply| sanitize_agent_surface_text(&reply))
        .filter(|reply| !reply.trim().is_empty())
        .collect::<Vec<_>>();
    let default_replies = group
        .default_replies
        .into_iter()
        .map(|reply| sanitize_agent_surface_text(&reply))
        .filter(|reply| replies.contains(reply))
        .collect::<Vec<_>>();
    (!title.trim().is_empty() && (!prompt.trim().is_empty() || !replies.is_empty())).then_some(
        AgentSuggestedReplyGroup {
            title,
            prompt,
            replies,
            default_replies,
        },
    )
}

fn agent_action_to_state(action: AgentProposedAction) -> AgentProposedActionState {
    AgentProposedActionState {
        action_kind: action.action_kind,
        target_kind: action.target_kind,
        target_id: action.target_id,
        field: action.field,
        value: action.value,
        summary: action.summary,
    }
}

fn agent_action_state_to_action(action: &AgentProposedActionState) -> AgentProposedAction {
    AgentProposedAction {
        action_kind: action.action_kind.clone(),
        target_kind: action.target_kind.clone(),
        target_id: action.target_id.clone(),
        field: action.field.clone(),
        value: action.value.clone(),
        summary: action.summary.clone(),
    }
}

fn sanitize_agent_model(model: Option<&str>) -> String {
    model.unwrap_or("gpt-5.5").trim().to_owned()
}

fn sanitize_reasoning_effort(effort: Option<&str>) -> String {
    match effort.unwrap_or("low").trim().to_ascii_lowercase().as_str() {
        "off" => "off".into(),
        "low" => "low".into(),
        "medium" => "medium".into(),
        "high" => "high".into(),
        "xhigh" => "xhigh".into(),
        _ => "low".into(),
    }
}

#[allow(dead_code)]
fn local_review_agent(
    project: &ProjectFile,
    request: &AgentReviewReplyRequest,
    model: &str,
    reasoning_effort: &str,
) -> AgentReviewReplyResponse {
    let text = review_user_text(request);
    let comment_title = request
        .comment
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or(&request.comment_id);

    if request.comment_id.starts_with("missing-gravity-load-") {
        if let Some(load_n_per_m) = parse_line_load_n_per_m(&text) {
            let load_label =
                format_quantity(load_n_per_m, QuantityKind::LineLoad, &project.unit_profile);
            let (target_kind, target_id) = first_comment_target(request)
                .unwrap_or_else(|| ("member_group".into(), "unknown".into()));
            return AgentReviewReplyResponse {
                agent_mode: "local".into(),
                model: model.into(),
                reasoning_effort: reasoning_effort.into(),
                status: "ready_to_apply".into(),
                message: format!(
                    "Resolution: use a preliminary {load_label} downward gravity line load for this member group."
                ),
                follow_up: None,
                suggested_chips: Vec::new(),
                resolution_summary: format!(
                    "Use a preliminary {load_label} downward gravity line load for this member group."
                ),
                interpretation: format!(
                    "Apply preliminary gravity line load: {load_label} downward on the selected member group."
                ),
                proposed_actions: vec![AgentProposedAction {
                    action_kind: "add_load".into(),
                    target_kind,
                    target_id,
                    field: "structural_model.loads".into(),
                    value: json!({
                        "kind": "uniform_line",
                        "magnitude": {
                            "value": load_n_per_m,
                            "quantityKind": "line_load",
                            "canonicalUnit": "N/m"
                        },
                        "loadCaseId": "gravity",
                        "direction": { "x": 0.0, "y": -1.0, "z": 0.0 }
                    }),
                    summary: format!("Add {load_label} downward gravity line load."),
                }],
                diagnostics: Vec::new(),
            };
        }
        return AgentReviewReplyResponse {
            agent_mode: "local".into(),
            model: model.into(),
            reasoning_effort: reasoning_effort.into(),
            status: "needs_more_information".into(),
            message:
                "I can record a preliminary gravity line load, but I need a value or derivation inputs before this can be resolved."
                    .into(),
            follow_up: Some(
                format!(
                    "Please provide the combined gravity line load in {}, or give tributary width and load assumptions so the agent can derive it.",
                    project.unit_profile.line_load.symbol
                )
                    .into(),
            ),
            suggested_chips: vec![
                "provide line load value".into(),
                "derive from tributary width".into(),
                "state roof and ceiling load assumptions".into(),
                "not sure yet".into(),
            ],
            resolution_summary: String::new(),
            interpretation: String::new(),
            proposed_actions: Vec::new(),
            diagnostics: Vec::new(),
        };
    }

    if request.comment_id.starts_with("section-family-") {
        let allowed_families = parse_section_families(&text);
        let wants_agent_choose =
            mentions_any(&text, &["let agent choose", "agent choose", "choose"]);
        if !allowed_families.is_empty() || wants_agent_choose {
            let mut target_id = request
                .comment
                .get("targets")
                .and_then(Value::as_array)
                .and_then(|targets| targets.first())
                .and_then(|target| target.get("id"))
                .and_then(Value::as_str)
                .unwrap_or("beam.simply_supported")
                .to_owned();
            let target_kind = request
                .comment
                .get("targets")
                .and_then(Value::as_array)
                .and_then(|targets| targets.first())
                .and_then(|target| target.get("kind"))
                .and_then(Value::as_str)
                .unwrap_or("member_group");
            if target_kind == "coordination_group" {
                if let Some(structural) = project.structural_model.as_ref() {
                    let draft = planning_draft(project);
                    if let Ok(report) = build_coordination_report(project, &draft, structural) {
                        if let Some(group) =
                            resolve_coordination_group_reference(&report.groups, &target_id)
                        {
                            target_id = group.id.clone();
                        }
                    }
                }
            }
            let allowed_families = if allowed_families.is_empty() {
                available_section_families()
            } else {
                allowed_families
            };
            let strategy = if wants_agent_choose {
                AGENT_JUSTIFIED_SECTION_SELECTION_POLICY
            } else {
                "user_allowed_families"
            };
            return AgentReviewReplyResponse {
                agent_mode: "local".into(),
                model: model.into(),
                reasoning_effort: reasoning_effort.into(),
                status: "ready_to_apply".into(),
                message: format!(
                    "Understood. I will consider {} for this beam and leave exact section sizing to the analysis step.",
                    allowed_families.join(", ")
                ),
                follow_up: None,
                suggested_chips: Vec::new(),
                resolution_summary: format!(
                    "Store {} as allowed catalogue section families and leave exact sizing to analysis.",
                    allowed_families.join(", ")
                ),
                interpretation: format!(
                    "Allowed section families: {}. Selection strategy: {strategy}.",
                    allowed_families.join(", ")
                ),
                proposed_actions: vec![AgentProposedAction {
                    action_kind: "update_planning_draft".into(),
                    target_kind: target_kind.into(),
                    target_id,
                    field: if target_kind == "coordination_group" {
                        "coordinationGroup.allowedSectionFamilies".into()
                    } else {
                        "beam.simply_supported.sectionFamilyPreferences".into()
                    },
                    value: json!({
                        "allowedSectionFamilies": allowed_families,
                        "sectionSelectionStrategy": strategy,
                        "sectionSelectionPolicy": strategy,
                    }),
                    summary: "Store section-family options for later sizing.".into(),
                }],
                diagnostics: Vec::new(),
            };
        }
    }

    if text.trim().len() >= 4 {
        AgentReviewReplyResponse {
            agent_mode: "local".into(),
            model: model.into(),
            reasoning_effort: reasoning_effort.into(),
            status: "ready_to_apply".into(),
            message: "That is enough information for this review item.".into(),
            follow_up: None,
            suggested_chips: Vec::new(),
            resolution_summary: format!("Record this answer for {comment_title}: {}", text.trim()),
            interpretation: format!("User response for {comment_title}: {}", text.trim()),
            proposed_actions: Vec::new(),
            diagnostics: Vec::new(),
        }
    } else {
        let mut suggested_chips: Vec<String> = available_section_families()
            .into_iter()
            .take(2)
            .map(|family| format!("use {family}"))
            .collect();
        suggested_chips.push("let agent choose".into());
        let examples = suggested_chips.join(" or ");
        AgentReviewReplyResponse {
            agent_mode: "local".into(),
            model: model.into(),
            reasoning_effort: reasoning_effort.into(),
            status: "needs_more_information".into(),
            message: "I need one concrete engineering assumption before this can be resolved."
                .into(),
            follow_up: Some(format!(
                "Pick an option or write a short instruction, for example {examples}."
            )),
            suggested_chips,
            resolution_summary: String::new(),
            interpretation: String::new(),
            proposed_actions: Vec::new(),
            diagnostics: Vec::new(),
        }
    }
}

fn filter_supported_agent_actions(actions: Vec<AgentProposedAction>) -> Vec<AgentProposedAction> {
    actions
        .into_iter()
        .filter(|action| {
            matches!(
                action.action_kind.as_str(),
                "update_planning_draft" | "add_load" | "add_support"
            )
        })
        .collect()
}

fn agent_action_value_schema() -> serde_json::Value {
    let design_option_intent_schema = design_option_intent_action_schema();
    json!({
        "type": "object",
        "properties": {
            "supportType": { "type": ["string", "null"] },
            "ux": { "type": ["boolean", "null"] },
            "uy": { "type": ["boolean", "null"] },
            "uz": { "type": ["boolean", "null"] },
            "rx": { "type": ["boolean", "null"] },
            "ry": { "type": ["boolean", "null"] },
            "rz": { "type": ["boolean", "null"] },
            "kind": { "type": ["string", "null"] },
            "magnitude": {
                "type": ["object", "number", "null"],
                "properties": {
                    "value": { "type": "number" },
                    "quantityKind": { "type": "string", "enum": ["force", "line_load", "stress"] },
                    "canonicalUnit": { "type": "string", "enum": ["N", "N/m", "Pa"] }
                },
                "required": ["value", "quantityKind", "canonicalUnit"],
                "additionalProperties": false
            },
            "loadCaseId": { "type": ["string", "null"] },
            "direction": {
                "type": ["object", "null"],
                "properties": { "x": { "type": "number" }, "y": { "type": "number" }, "z": { "type": "number" } },
                "required": ["x", "y", "z"],
                "additionalProperties": false
            },
            "allowedSectionFamilies": { "type": ["array", "null"], "items": { "type": "string" } },
            "sectionSelectionPolicy": { "type": ["string", "null"] },
            "sectionSelectionStrategy": { "type": ["string", "null"] },
            "designOptionIntents": {
                "type": ["array", "null"],
                "items": design_option_intent_schema.clone()
            },
            "replacementDesignOptionIntent": design_option_intent_schema,
            "supersededOptionId": { "type": ["string", "null"] },
            "supersededReason": { "type": ["string", "null"] },
            "replacementReason": { "type": ["string", "null"] },
            "revisionOf": { "type": ["string", "null"] }
        },
        "required": ["supportType", "ux", "uy", "uz", "rx", "ry", "rz", "kind", "magnitude", "loadCaseId", "direction", "allowedSectionFamilies", "sectionSelectionPolicy", "sectionSelectionStrategy", "designOptionIntents", "replacementDesignOptionIntent", "supersededOptionId", "supersededReason", "replacementReason", "revisionOf"],
        "additionalProperties": false
    })
}

fn design_option_intent_action_schema() -> serde_json::Value {
    json!({
        "type": ["object", "null"],
        "properties": {
            "id": { "type": "string" },
            "label": { "type": "string" },
            "hypothesis": { "type": "string" },
            "explorationBand": { "type": "string" },
            "lifecycleStatus": { "type": ["string", "null"], "enum": ["active", "superseded", "rejected", null] },
            "supersededBy": { "type": ["string", "null"] },
            "supersededReason": { "type": ["string", "null"] },
            "revisionOf": { "type": ["string", "null"] },
            "objectiveTags": { "type": "array", "items": { "type": "string" } },
            "standardisationStrategy": { "type": "string" },
            "connectionStrategy": { "type": "string" },
            "supportStrategy": { "type": "string" },
            "sectionFamilyPolicy": { "type": "string" },
            "coordinationGroupPolicy": { "type": "string" },
            "coordinationOverrides": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "memberId": { "type": "string" },
                        "familyGroupLabel": { "type": ["string", "null"] },
                        "designationGroupLabel": { "type": ["string", "null"] },
                        "note": { "type": ["string", "null"] }
                    },
                    "required": ["memberId", "familyGroupLabel", "designationGroupLabel", "note"],
                    "additionalProperties": false
                }
            },
            "assumptions": { "type": "array", "items": { "type": "string" } },
            "provenance": { "type": "array", "items": { "type": "string" } }
        },
        "required": ["id", "label", "hypothesis", "explorationBand", "lifecycleStatus", "supersededBy", "supersededReason", "revisionOf", "objectiveTags", "standardisationStrategy", "connectionStrategy", "supportStrategy", "sectionFamilyPolicy", "coordinationGroupPolicy", "coordinationOverrides", "assumptions", "provenance"],
        "additionalProperties": false
    })
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PiSessionAgentResponse {
    status: String,
    message: String,
    #[serde(default)]
    suggested_replies: Vec<String>,
    #[serde(default)]
    suggested_reply_groups: Vec<AgentSuggestedReplyGroup>,
    #[serde(default)]
    plan_summary: String,
    #[serde(default)]
    proposed_actions: Vec<AgentProposedAction>,
    #[serde(default)]
    diagnostics: Vec<WorkbenchDiagnostic>,
    #[serde(default)]
    base_model_brief: Option<BaseModelBrief>,
    #[serde(skip)]
    provider_id: String,
    #[serde(skip)]
    model_id: String,
    #[serde(skip)]
    reasoning_effort: String,
    #[serde(skip)]
    catalogue_refreshed_at: Option<String>,
}

fn run_pi_session_agent(
    project: &ProjectFile,
    surface: &str,
    settings: &AgentModelSettings,
    request_id: Option<&str>,
) -> Result<PiSessionAgentResponse> {
    let schema = pi_session_schema();
    let prompt = build_pi_session_prompt(project, surface);
    let generated_request_id = format!(
        "fraia-{surface}-{}",
        fraia_core::utils::iso_now().replace([':', '-'], "")
    );
    let request_id = request_id
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .unwrap_or(&generated_request_id);
    let (mut response, envelope) =
        run_pi_turn::<PiSessionAgentResponse>(request_id, settings, &prompt, &schema)?;
    response.status = normalize_agent_status(&response.status);
    response.proposed_actions = filter_supported_agent_actions(response.proposed_actions);
    response.provider_id = envelope.provider_id;
    response.model_id = envelope.model_id;
    response.reasoning_effort = envelope.reasoning_effort;
    response.catalogue_refreshed_at = envelope.catalogue_refreshed_at;
    Ok(response)
}

fn run_pi_review_agent(
    project: &ProjectFile,
    request: &AgentReviewReplyRequest,
    settings: &AgentModelSettings,
) -> Result<AgentReviewReplyResponse> {
    let schema = pi_review_schema();
    let prompt = build_pi_review_prompt(project, request);
    let request_id = format!(
        "fraia-review-{}",
        fraia_core::utils::iso_now().replace([':', '-'], "")
    );
    let (mut response, envelope) =
        run_pi_turn::<AgentReviewReplyResponse>(&request_id, settings, &prompt, &schema)?;
    response.agent_mode = "pi".into();
    response.model = format!("{}/{}", envelope.provider_id, envelope.model_id);
    response.reasoning_effort = envelope.reasoning_effort;
    response.status = normalize_agent_status(&response.status);
    response.proposed_actions = response
        .proposed_actions
        .into_iter()
        .filter(|action| {
            matches!(
                action.action_kind.as_str(),
                "update_planning_draft" | "add_load" | "add_support"
            )
        })
        .collect();
    Ok(response)
}

fn pi_session_schema() -> Value {
    let action_value_schema = agent_action_value_schema();
    let action_schema = json!({
        "type": "object",
        "properties": {
            "actionKind": { "type": "string" },
            "targetKind": { "type": "string" },
            "targetId": { "type": "string" },
            "field": { "type": "string" },
            "value": action_value_schema,
            "summary": { "type": "string" }
        },
        "required": ["actionKind", "targetKind", "targetId", "field", "value", "summary"],
        "additionalProperties": false
    });
    let diagnostic_schema = json!({
        "type": "object",
        "properties": {
            "severity": { "type": "string" },
            "code": { "type": "string" },
            "message": { "type": "string" },
            "detail": { "type": ["string", "null"] }
        },
        "required": ["severity", "code", "message", "detail"],
        "additionalProperties": false
    });
    let readiness_schema = json!({
        "type": "object",
        "properties": {
            "readyForSchemas": { "type": "boolean" },
            "unresolvedTopics": { "type": "array", "items": { "type": "string" } },
            "manualOverrideAllowed": { "type": "boolean" }
        },
        "required": ["readyForSchemas", "unresolvedTopics", "manualOverrideAllowed"],
        "additionalProperties": false
    });
    let visual_intent_support_schema = json!({
        "type": "object",
        "properties": {
            "id": { "type": "string" },
            "targetNode": { "type": "string" },
            "label": { "type": ["string", "null"] },
            "status": { "type": "string", "enum": ["location_only"] }
        },
        "required": ["id", "targetNode", "label", "status"],
        "additionalProperties": false
    });
    let visual_intent_load_target_schema = json!({
        "type": "object",
        "properties": {
            "kind": { "type": "string", "enum": ["all_members", "member", "node"] },
            "memberId": { "type": ["string", "null"] },
            "nodeId": { "type": ["string", "null"] }
        },
        "required": ["kind", "memberId", "nodeId"],
        "additionalProperties": false
    });
    let visual_intent_load_direction_schema = json!({
        "type": ["object", "null"],
        "properties": {
            "kind": { "type": "string", "enum": ["toward_node", "vector"] },
            "fromNode": { "type": ["string", "null"] },
            "toNode": { "type": ["string", "null"] },
            "x": { "type": ["number", "null"] },
            "y": { "type": ["number", "null"] },
            "z": { "type": ["number", "null"] }
        },
        "required": ["kind", "fromNode", "toNode", "x", "y", "z"],
        "additionalProperties": false
    });
    let visual_intent_load_schema = json!({
        "type": "object",
        "properties": {
            "id": { "type": "string" },
            "kind": { "type": "string", "enum": ["self_weight", "point", "uniform_line"] },
            "target": visual_intent_load_target_schema,
            "magnitudeN": { "type": ["number", "null"] },
            "magnitudeNPerM": { "type": ["number", "null"] },
            "direction": visual_intent_load_direction_schema
        },
        "required": ["id", "kind", "target", "magnitudeN", "magnitudeNPerM", "direction"],
        "additionalProperties": false
    });
    let visual_intent_schema = json!({
        "type": "object",
        "properties": {
            "supportLocations": { "type": "array", "items": visual_intent_support_schema },
            "loads": { "type": "array", "items": visual_intent_load_schema }
        },
        "required": ["supportLocations", "loads"],
        "additionalProperties": false
    });
    let base_model_brief_schema = json!({
        "type": ["object", "null"],
        "properties": {
            "version": { "type": "integer" },
            "sessionId": { "type": "string" },
            "currentUnderstanding": { "type": "string" },
            "confirmedIntent": { "type": "array", "items": { "type": "string" } },
            "openQuestions": { "type": "array", "items": { "type": "string" } },
            "softAssumptions": { "type": "array", "items": { "type": "string" } },
            "schemaGuidance": { "type": "array", "items": { "type": "string" } },
            "doNotDecideYet": { "type": "array", "items": { "type": "string" } },
            "visualIntent": visual_intent_schema,
            "readiness": readiness_schema,
            "updatedAt": { "type": "string" }
        },
        "required": ["version", "sessionId", "currentUnderstanding", "confirmedIntent", "openQuestions", "softAssumptions", "schemaGuidance", "doNotDecideYet", "visualIntent", "readiness", "updatedAt"],
        "additionalProperties": false
    });
    let schema = json!({
        "type": "object",
        "properties": {
            "status": { "type": "string", "enum": ["needs_user_input", "needs_more_information", "ready_to_apply", "blocked"] },
            "message": { "type": "string" },
            "suggestedReplies": { "type": "array", "items": { "type": "string" } },
            "suggestedReplyGroups": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "title": { "type": "string" },
                        "prompt": { "type": "string" },
                        "replies": { "type": "array", "items": { "type": "string" } },
                        "defaultReplies": { "type": "array", "items": { "type": "string" } }
                    },
                    "required": ["title", "prompt", "replies", "defaultReplies"],
                    "additionalProperties": false
                }
            },
            "planSummary": { "type": "string" },
            "proposedActions": { "type": "array", "items": action_schema },
            "diagnostics": { "type": "array", "items": diagnostic_schema },
            "baseModelBrief": base_model_brief_schema
        },
        "required": ["status", "message", "suggestedReplies", "suggestedReplyGroups", "planSummary", "proposedActions", "diagnostics", "baseModelBrief"],
        "additionalProperties": false
    });
    schema
}

fn pi_review_schema() -> Value {
    let schema = json!({
        "type": "object",
        "properties": {
            "agentMode": { "type": "string" },
            "model": { "type": "string" },
            "reasoningEffort": { "type": "string", "enum": ["off", "low", "medium", "high", "xhigh"] },
            "status": { "type": "string", "enum": ["needs_more_information", "ready_to_apply"] },
            "message": { "type": "string" },
            "followUp": { "type": ["string", "null"] },
            "suggestedChips": {
                "type": "array",
                "items": { "type": "string" }
            },
            "resolutionSummary": { "type": "string" },
            "interpretation": { "type": "string" },
            "proposedActions": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "actionKind": { "type": "string" },
                        "targetKind": { "type": "string" },
                        "targetId": { "type": "string" },
                        "field": { "type": "string" },
                        "value": { "type": "object" },
                        "summary": { "type": "string" }
                    },
                    "required": ["actionKind", "targetKind", "targetId", "field", "value", "summary"],
                    "additionalProperties": false
                }
            },
            "diagnostics": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "severity": { "type": "string" },
                        "code": { "type": "string" },
                        "message": { "type": "string" },
                        "detail": { "type": ["string", "null"] }
                    },
                    "required": ["severity", "code", "message", "detail"],
                    "additionalProperties": false
                }
            }
        },
        "required": ["agentMode", "model", "reasoningEffort", "status", "message", "followUp", "suggestedChips", "resolutionSummary", "interpretation", "proposedActions", "diagnostics"],
        "additionalProperties": false
    });
    schema
}

fn build_pi_session_prompt(project: &ProjectFile, surface: &str) -> String {
    let draft = planning_draft(project);
    let structural = project.structural_model.clone();
    let session = project
        .agent_state
        .sessions
        .iter()
        .find(|session| session.surface == surface);
    let scheme_id = scheme_surface_id(surface).map(str::to_owned);
    let knowledge_context =
        build_llm_knowledge_context(project, &draft, structural.as_ref(), surface, "session");
    let model_shape_hints = structural
        .as_ref()
        .map(generic_model_context_hints)
        .unwrap_or_default();
    let selected_design_scheme = scheme_id.as_ref().and_then(|id| {
        structural.as_ref().and_then(|model| {
            build_coordination_report(project, &draft, model)
                .ok()
                .and_then(|report| {
                    report
                        .design_schemes
                        .into_iter()
                        .find(|scheme| &scheme.id == id)
                })
        })
    });
    let payload = json!({
        "surface": surface,
        "schemeSurface": scheme_id.as_ref().map(|id| json!({
            "schemeId": id,
            "displayName": humanize_scheme_id(id),
            "scope": "This is a design-option-specific chat surface. Use selectedDesignScheme plus the current structural model; do not rely on a properties sidebar payload.",
        })),
        "selectedDesignScheme": selected_design_scheme,
        "project": {
            "name": project.name.clone(),
            "planningDraft": draft,
            "structuralModel": structural,
            "baseModelBrief": project.base_model_brief.clone(),
            "analysisReadiness": structural
                .as_ref()
                .map(evaluate_structural_solve_readiness)
                .unwrap_or_else(|| AnalysisReadiness {
                    status: "not_ready".into(),
                    summary: "No authored structural model exists.".into(),
                    diagnostics: Vec::new(),
                }),
        },
        "session": session,
        "modelShapeHints": model_shape_hints,
        "knowledgeContext": knowledge_context,
        "supportedSectionFamilies": if scheme_id.is_some() {
            json!(available_section_families())
        } else {
            json!([])
        },
        "allowedActionKinds": if scheme_id.is_some() {
            json!(["update_planning_draft"])
        } else {
            json!([
                "update_planning_draft",
                "add_load",
                "add_support"
            ])
        },
    });
    let scheme_instruction = if scheme_id.is_some() {
        "This conversation is scoped to one engineering design option selected in the Fraia UI. In user-facing text, call it a design option, not a scheme; selectedDesignScheme is an internal API name only.\n\
Treat the selected DesignOptionIntent as already agent-authored option judgement. Do not ask the user to confirm assumptions already encoded in the option, and do not ask generic steering questions such as what Fraia should focus on next, whether this option should be compared, or whether there are hard constraints unless the user introduces that topic.\n\
If this design-option chat has no prior assistant message, write a concise opening review that already explains the option. Cover, using short Markdown headings or bullets where helpful: what this option tests, encoded assumptions, realised supports/loads/section-family choices visible in selectedDesignScheme, load path, support/restraint behaviour, connection/foundation implications, and main risks or limits. Do not say realised SupportAssignment, LoadAssignment, support type, load target, load magnitude, section-family policy, or group-choice data are still needed when selectedDesignScheme already contains them. Do not include a generic \"evidence still needed\" checklist. If analysis has not run yet, mention it only briefly when it affects interpretation, for example \"CalculiX results are not attached yet, so reactions, displacements, and stress values are not available in this view.\" Do not list obvious downstream design/check artefacts unless the user asks what remains. Do not use suggestedReplies or suggestedReplyGroups for design-option opening reviews.\n\
For later turns, answer the user's specific question directly and concisely. Use Markdown formatting naturally, including short headings, bullets, bold labels, and compact tables when they improve readability. Do not force a fixed template.\n\
Do not state or recommend exact member section sizes in design-option chat; exact sizes belong to later solve/design. If exact frontend-only design-option details are not present in Context JSON, say what can be inferred from the project context and ask for the missing design-option detail instead of inventing it.\n\
Design options are immutable comparison artefacts derived from the Base Model. Do not propose actions that mutate, overwrite, adopt, or apply the option back into the Base Model. Keep suggestedReplies and suggestedReplyGroups empty by default. Keep proposedActions empty unless the user explicitly asks to change, revise, replace, avoid, or alter this design option.\n\
For an explicit change request, return one update_planning_draft proposed action with field \"coordination.designOptionReplacement\", targetKind \"design_option\", targetId set to selectedDesignScheme.id, value.supersededOptionId set to selectedDesignScheme.id, value.supersededReason explaining why the original is superseded, and value.replacementDesignOptionIntent containing a complete replacement DesignOptionIntent. The replacement intent must have lifecycleStatus \"active\", revisionOf set to selectedDesignScheme.id, supersededBy null, supersededReason null, and provenance that explains the revised support/restraint, load path or stability concept, section-family policy, coordination/standardisation policy, and connection/detailing consequence using internal structural knowledge plus project evidence. If the user asks for a specific member to move to a different GF or GD group, encode that exact deterministic change in replacementDesignOptionIntent.coordinationOverrides with memberId and the requested familyGroupLabel and/or designationGroupLabel, for example familyGroupLabel \"Family Group 2\" for GF2 and designationGroupLabel \"Size Group 1\" for GD1. The original option will be marked superseded and the replacement realised as a new option; do not describe this as applying anything to the Base Model.\n"
    } else {
        ""
    };
    format!(
        "You are Fraia's structural planning assistant. Be conversational, helpful, and focused on getting the Rust-authored structural model useful to solve.\n\
{scheme_instruction}\
Return only JSON matching the provided schema.\n\
	For surface \"pre_solve\", maintain baseModelBrief as the persistent distilled planning state. Return the complete updated baseModelBrief every turn: preserve confirmed items, add newly confirmed intent, keep unresolved points explicit, and set readiness.readyForSchemas only when enough fixed-boundary intent exists to generate design options. For non-pre_solve surfaces, return baseModelBrief as null.\n\
For surface \"pre_solve\", put confirmed viewport-relevant support and load intent in baseModelBrief.visualIntent. Do not rely on prose alone for anything that should appear in the viewport. visualIntent.supportLocations is only for neutral support locations with status \"location_only\". Never encode pinned, fixed, roller, or base-fixity decisions there unless the user later applies actual SupportAssignment objects. visualIntent.loads may include self_weight on all_members or member targets, uniform_line loads on exact member targets when target, magnitudeNPerM, and direction are known, and point loads only when target, magnitudeN, and direction are known. Use canonical SI values: point magnitudeN in N and uniform_line magnitudeNPerM in N/m. Resolve natural references such as \"node 2 toward node 3\" or \"downward\" to exact structuralModel node ids or vector components in visualIntent.direction. Never use view-dependent location words such as upper-left, upper-right, lower-left, lower-right, top-left, or top-right in user-facing load choices because the model can rotate in 3D. Use stable Node and Member ids plus a direction, for example \"point load at Node N2, direction +X toward Node N3\" or \"uniform line load on Member M2, direction -Y global\".\n\
If support or load prose is confirmed but the exact typed node/member reference is unknown, leave the corresponding visualIntent item out and keep an open question or unresolved topic for the missing reference. Do not invent node or member ids. If a point load target and direction are known but magnitude is missing, do not advance to the next briefing stage; ask for the point-load magnitude as the next main question and include realistic suggestedReplies such as placeholder kN values, removing the point load, or deriving it from tributary loading. If a uniform line load target and direction are known but magnitude is missing, ask for the line-load magnitude before marking the brief ready.\n\
	Do not claim to mutate the project. You may only propose actions for explicit user review/apply.\n\
		Ask the fewest turns possible. Before asking, do a critical-path check: if multiple questions are independent, ask them together in one message instead of serialising them across separate LLM turns. Keep dependent questions sequenced only when an answer is genuinely needed before the next question makes sense. When you ask one question, use suggestedReplies for 2-5 realistic replies. When you ask multiple independent questions, do not make one flat list of combined mega-answers; instead put each independent issue in suggestedReplyGroups with a short title, one prompt, and 2-4 local preset replies for that issue. Use suggestedReplyGroups.defaultReplies only for choices Fraia should preselect because they are sane defaults from the current model and structural design judgement; every defaultReply must exactly match one entry in replies. The user should be able to discuss structure context, supports, loads, and hard constraints independently while still seeing everything in one assistant response. Leave suggestedReplies empty when suggestedReplyGroups are present.\n\
		Write compact, efficient chat text: friendly but straight to the point. Use Markdown naturally when it improves scanning: short headings, bullets, bold labels, numbered steps, and compact tables are all acceptable. Do not force a fixed template or pad responses. Keep most messages under 150 words unless the user asks for detail or a first-pass design-option review needs a complete but concise option summary. Prefer plain structural language throughout. Avoid specialist shorthand as the main wording, especially for load path, support restraint, connection fixity, frame action, member actions, drift, reactions, releases, haunches, knees, bases, and stability. Include the technical term in parentheses only when it adds useful precision or matches labels elsewhere in the app. Prefer \"the N2/N3 corner connections stay stiff enough to transfer bending\" over bare phrases like \"knee moment continuity\".\n\
	In user-facing Base Model Guide questions and suggestedReplyGroups, refer directly to the actual opened model whenever possible. Use exact display Node ids and Member ids with short positional labels, for example \"N1 bottom-left and N4 bottom-right\" rather than \"bottom-left and bottom-right nodes\". For load choices, name the target Member ids when known, for example \"self weight on all Members\" or \"roof gravity load on M2, the top/sloping Member\". Never show raw authored ids or bare source ids such as \"builder.frame.review::e2\", \"e2\", or \"n2\" to the user; use \"M2\" or \"N2\" display labels instead. If a natural label is useful, pair it with the display id. Do not invent ids, but do not stay purely qualitative when Context JSON gives the ids.\n\
	Do not include a raw Diagnostics section in the user-facing message. Translate readiness issues into one short plain-language point only when it directly helps the next question.\n\
	Use knowledgeContext as internal structural knowledge before giving structural-system guidance. Ground recommendations about load paths, restraint/stability, supports/releases, section-family tradeoffs, serviceability intent, buildability, and provenance in that internal knowledge when relevant. Do not mention the wiki, knowledgeContext, retrieved excerpts, sources, or knowledge-base mechanics in user-facing message, follow-up, suggestedReplies, or suggestedReplyGroups; speak directly as Fraia's structural design judgement. Treat internal knowledge as reusable guidance only, never as project-specific approval, code compliance, or a replacement for user confirmations, analysis runs, check inputs, or check results. Keep this prompt generic: infer the current structural system from Context JSON, use relevant internal knowledge, and ask for missing project intent rather than hard-coding assumptions for one system type. In the Base Model Guide opening message, do not teach structural concepts or explain generic support/load theory; use knowledge only to choose good concise prompts and defaults.\n\
When discussing or proposing design options, speak in DesignOptionIntent terms: hypothesis, explorationBand, objectiveTags, standardisationStrategy, connectionStrategy, supportStrategy, sectionFamilyPolicy, coordinationGroupPolicy, coordinationOverrides, assumptions, and provenance. Recommend only options that are worth exploring from the current Base Model evidence or user-confirmed intent; do not spam typical alternatives just because they are common.\n\
When design options are ready to be generated, propose a reviewed update_planning_draft action with field \"coordination.designOptionIntents\" and value.designOptionIntents containing the complete DesignOptionIntent records. Use knowledgeContext and project evidence to decide which intents are worth proposing; deterministic Fraia code will validate and realise them into design-option views. Every new DesignOptionIntent should set lifecycleStatus \"active\", supersededBy null, supersededReason null, and revisionOf null unless it is explicitly a replacement. Every DesignOptionIntent must justify all major decisions using internal structural knowledge plus project evidence. In provenance, include concise decision-justification entries that explicitly cover support/restraint choice, load path or stability concept, section-family policy, coordination/standardisation policy, and connection/detailing consequence. Each provenance entry should state the structural rationale without referencing the wiki, retrieved excerpts, sources, or knowledgeContext; do not use vague provenance such as \"agent chose this\" or \"typical option\". Every DesignOptionIntent must state a realizable buildable supportStrategy: either use existing SupportAssignment objects, or explicitly choose pinned or fixed restraint at confirmed support-location node ids. Do not propose pinned/roller, roller, sliding, or horizontally released support strategies as generated buildable design options; those are low-level support primitives or explicit analysis sensitivities, not default design-option assumptions. Location-only support intent is acceptable for the Base Model brief, but is not a complete design option. Do not change support fixity just to make an option solve. If a design option varies support fixity (fixed bases, added restraint, or released restraint), the supportStrategy and assumptions/provenance must explicitly justify why that restraint change belongs to this option, whether it is a baseline assumption or a sensitivity case, and what connection/foundation tradeoff it introduces. If the option hypothesis is haunching, bracing, or section standardisation and support fixity is not part of the hypothesis, inherit the Base Model support intent instead of inventing fixed or roller supports.\n\
Use object type for canonical Fraia primitives such as Member, Plate, Node, SupportAssignment, LoadAssignment, and ReleaseAssignment. Use role for semantic terms such as beam, column, rafter, brace, slab, or wall_panel. Reserve element for analysis/discretisation objects only.\n\
		If this session has no prior messages, open with at most two short sentences, under 60 words total, that only state what Fraia broadly thinks the visible Base Model is, using display Node and Member ids only if they help. Do not explain generic support theory, load theory, reactions, self weight, point loads, lateral loads, or design-option philosophy in the opening message. Then ask the first grouped briefing questions. Bundle independent first-pass briefing questions together, such as what the geometry represents, support permissions, loads, and approval boundaries, when they do not depend on each other. Avoid overlapping groups: do not ask separate \"model intent\" and \"geometry context\" questions if the available answer choices would mostly repeat each other. Prefer one combined group such as \"What this geometry represents\" with choices that distinguish standalone frame, repeated bay/slice, connected partial structure, or generic test geometry. Only split into a second context group if it asks a genuinely different question, such as whether this visible frame is one bay within a larger longitudinal system. For the initial Loads group, make replies additive checklist items, not combined bundles: include self weight on all Members as a defaultReply unless the project evidence says otherwise, and list likely applied-load components separately. Every non-self-weight load reply must include load shape, exact target id, and exact direction: for example \"uniform line load on Member M2, direction -Y global\", \"vertical point load at Node N3, direction -Y global\", or \"horizontal point load at Node N2, direction +X toward Node N3\". Do not offer vague replies such as \"lateral load at N2\" or \"point load at N3\" without direction. Do not use view-dependent labels such as upper-left, upper-right, top, bottom, left, or right unless they are paired with exact Node/Member ids and a stable global direction; prefer omitting those labels entirely. Do not write replies like \"self weight plus roof gravity load\" because users need to tick load components independently. Present grouped questions as separate short paragraphs in the message and return matching suggestedReplyGroups. For hard constraints/no-go boundaries, prefer a short prompt plus an empty replies array so the user can write free text; include preset checkboxes only for constraints strongly suggested by the current project evidence or prior user wording. Do not include generic presets like \"Steel only\", \"No fixed bases\", or \"No internal columns\" unless the user or project already made those likely. For each suggestedReplyGroup prompt, include only a concise reason if it is necessary to answer the group.\n\
	At the Base Model Guide stage, collect only fixed briefing boundaries: what the structure represents, whether design options should treat the geometry as standalone, representative/repeated, or connected to existing structure, where it can physically be supported or cannot be supported, broad load intent including self weight as included by default unless the user opts out, and any hard constraints or no-go zones. Do not ask design-option-choice questions. Do not ask the user to choose support type, base fixity, bracing/stability mechanism, pinned/fixed behaviour, member sizing, section-family choices, member grouping, or optimization tradeoffs such as economy versus fabrication simplicity versus stiffness/serviceability unless the user explicitly volunteers one as a hard constraint. Treat buildable pinned/fixed restraint choices as design-option alternatives by default.\n\
	Do not block design-option readiness just to ask the user to choose economy, lowest weight, fabrication simplicity, support type, base fixity, bracing/stability strategy, stiffness, balanced comparison, section family, or member grouping. If model intent, geometry context, support locations/constraints, load intent, and hard constraints/no-go zones are clear enough, mark readiness.readyForSchemas true and present the Base Model Brief for confirmation. Record support locations and no-go zones as fixed confirmedIntent. Record unspecified support kind, section family, member grouping, base fixity, and stability strategy as schemaGuidance/doNotDecideYet so design options compare alternatives within the confirmed boundary.\n\
For Base Model Guide section discussion, stay qualitative and only record material or section-family preferences as hard constraints when the user states them that way, such as steel only or no RHS. Do not mention demo section IDs and do not propose section-family actions from this chat surface.\n\
	For surface \"pre_solve\", keep proposedActions empty during ordinary Base Model Guide discussion. Persist confirmed assumptions and unresolved questions in baseModelBrief instead. Treat support locations/constraints, no-go zones, hard constraints, qualitative load intent, and design-option alternatives as design-option generator intent, not immediate model mutations. For design-option chat surfaces, keep proposedActions empty except for the explicit design-option replacement action described above; those conversations analyse, compare, and revise realised options rather than mutating the Base Model.\n\
Use only action kinds listed in allowedActionKinds.\n\n\
Context JSON:\n{}",
        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".into())
    )
}

fn build_llm_knowledge_context(
    project: &ProjectFile,
    draft: &CorePlanningDraft,
    structural: Option<&StructuralModel>,
    surface: &str,
    task: &str,
) -> Value {
    let retrieval_queries = knowledge_retrieval_queries(surface, task);
    let mut pages = retrieve_wiki_pages_for_llm_context(
        project,
        draft,
        structural,
        surface,
        task,
        &retrieval_queries,
        18,
    );
    pages.sort();
    pages.dedup();
    json!({
        "retrievalMode": "compiled-wiki-excerpts",
        "surface": surface,
        "task": task,
        "retrievalQueries": retrieval_queries,
        "instruction": "Use these excerpts as reusable engineering guidance. They are not project-specific approval, code compliance, final design checks, or a substitute for user-confirmed project artifacts, analysis runs, design actions, check inputs, or check results.",
        "requiredUse": [
            "Consult relevant internal engineering guidance before structural-system recommendations.",
            "Use project artifacts and user confirmations as project truth.",
            "Keep exact member sizing downstream of solve/design/check stages unless a run/check artifact already provides it.",
            "Keep authored, resolved, run, design-action, check-input, check-result, and export layers distinct."
        ],
        "pages": pages
            .into_iter()
            .filter_map(|relative_path| knowledge_page_excerpt(&relative_path))
            .collect::<Vec<_>>(),
    })
}

fn knowledge_retrieval_queries(surface: &str, task: &str) -> Vec<String> {
    let mut queries = vec![
        "authored resolved run artifact boundaries provenance assumptions project truth".to_owned(),
        "load paths supports restraints releases gravity lateral loads bracing diagnostics"
            .to_owned(),
    ];
    if surface == "pre_solve" || scheme_surface_id(surface).is_some() {
        queries.push(
            "scheme generation from knowledge structural design option intelligence concept options hypothesis standardisation connection strategy steel material section families serviceability robustness"
                .to_owned(),
        );
    }
    if surface == "review_reply" || task.to_ascii_lowercase().contains("review") {
        queries.push(
            "review load application equivalent nodal loads steel material section families coordination connection fixity partial restraint taxonomy design action check input"
                .to_owned(),
        );
    }
    queries
}

fn generic_model_context_hints(model: &StructuralModel) -> Vec<String> {
    let mut hints = Vec::new();
    if model
        .members
        .iter()
        .all(|member| member.role.trim().is_empty() || member.role == "member")
    {
        hints.push("Member roles are not authored yet; any system identification should be presented as an inference to confirm.".into());
    }
    if model.supports.is_empty() {
        hints.push("No authored SupportAssignment objects are present.".into());
    }
    if model.loads.is_empty() {
        hints.push("No authored LoadAssignment objects are present.".into());
    }
    hints
}

fn retrieve_wiki_pages_for_llm_context(
    project: &ProjectFile,
    draft: &CorePlanningDraft,
    structural: Option<&StructuralModel>,
    surface: &str,
    task: &str,
    retrieval_queries: &[String],
    limit: usize,
) -> Vec<String> {
    let context_terms =
        project_context_terms(project, draft, structural, surface, task, retrieval_queries);
    if context_terms.is_empty() {
        return Vec::new();
    }
    let mut scored: Vec<(String, usize)> = wiki_markdown_pages()
        .into_iter()
        .filter_map(|path| {
            let haystack = wiki_page_search_text(&path)?;
            let score = context_terms
                .iter()
                .map(|term| wiki_term_score(&haystack, &path, term))
                .sum();
            (score > 0).then_some((path, score))
        })
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    scored
        .into_iter()
        .take(limit)
        .map(|(path, _score)| path)
        .collect()
}

fn project_context_terms(
    project: &ProjectFile,
    draft: &CorePlanningDraft,
    structural: Option<&StructuralModel>,
    surface: &str,
    task: &str,
    retrieval_queries: &[String],
) -> Vec<String> {
    let mut text = format!(
        "{} {} {} {} {} {} {} {} {} {}",
        project.intent.building_type,
        project.intent.objective_priority,
        draft.project_intent.building_type,
        draft.project_intent.objective_priority,
        draft.system_brief.system_family_hint,
        draft.system_brief.structural_form_hint,
        draft.system_brief.notes,
        retrieval_queries.join(" "),
        surface,
        task,
    )
    .to_ascii_lowercase();
    if let Some(brief) = &project.base_model_brief {
        text.push(' ');
        text.push_str(&brief.current_understanding.to_ascii_lowercase());
        for item in brief
            .confirmed_intent
            .iter()
            .chain(brief.soft_assumptions.iter())
            .chain(brief.schema_guidance.iter())
            .chain(brief.do_not_decide_yet.iter())
            .chain(brief.open_questions.iter())
        {
            text.push(' ');
            text.push_str(&item.to_ascii_lowercase());
        }
    }
    if let Some(scheme_id) = scheme_surface_id(surface) {
        text.push(' ');
        text.push_str(&humanize_scheme_id(scheme_id).to_ascii_lowercase());
        text.push(' ');
        text.push_str(&scheme_id.replace('-', " "));
    }
    if let Some(model) = structural {
        for member in &model.members {
            text.push(' ');
            text.push_str(&member.role.to_ascii_lowercase());
            for tag in &member.semantic_tags {
                text.push(' ');
                text.push_str(&tag.to_ascii_lowercase());
            }
        }
    }
    let mut terms = Vec::new();
    for raw in text.split(|ch: char| !ch.is_ascii_alphanumeric()) {
        if raw.len() < 4 || is_generic_retrieval_term(raw) {
            continue;
        }
        let term = raw.to_owned();
        if !terms.iter().any(|existing| existing == &term) {
            push_retrieval_term_variants(&mut terms, &term);
        }
    }
    terms
}

fn push_retrieval_term_variants(terms: &mut Vec<String>, term: &str) {
    if !terms.iter().any(|existing| existing == term) {
        terms.push(term.to_owned());
    }
    let variant = if let Some(prefix) = term.strip_suffix('y') {
        Some(format!("{prefix}ies"))
    } else if let Some(prefix) = term.strip_suffix("ies") {
        Some(format!("{prefix}y"))
    } else if let Some(prefix) = term.strip_suffix('s') {
        Some(prefix.to_owned())
    } else {
        Some(format!("{term}s"))
    };
    if let Some(variant) = variant {
        if variant.len() >= 4 && !terms.iter().any(|existing| existing == &variant) {
            terms.push(variant);
        }
    }
}

fn wiki_term_score(haystack: &str, path: &str, term: &str) -> usize {
    let mut score = 0;
    if haystack.contains(term) {
        score += 1;
    }
    if path.to_ascii_lowercase().contains(term) {
        score += 2;
    }
    score
}

fn is_generic_retrieval_term(term: &str) -> bool {
    matches!(
        term,
        "none"
            | "unknown"
            | "member"
            | "members"
            | "model"
            | "review"
            | "geometry"
            | "unresolved"
            | "unspecified"
            | "raw"
            | "cad"
            | "like"
            | "line"
            | "connected"
            | "concept"
            | "frame"
            | "frames"
            | "structural"
            | "system"
            | "form"
            | "hint"
    )
}

fn wiki_markdown_pages() -> Vec<String> {
    let Some(root) = find_repo_relative_path("docs/knowledge/wiki") else {
        return Vec::new();
    };
    let mut pages = Vec::new();
    collect_markdown_pages(&root, "docs/knowledge/wiki", &mut pages);
    pages
}

fn collect_markdown_pages(dir: &Path, relative_dir: &str, pages: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();
        let relative_path = format!("{relative_dir}/{file_name}");
        if path.is_dir() {
            collect_markdown_pages(&path, &relative_path, pages);
        } else if path.extension().and_then(|value| value.to_str()) == Some("md") {
            pages.push(relative_path);
        }
    }
}

fn find_repo_relative_path(relative_path: &str) -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        let candidate = dir.join(relative_path);
        if candidate.exists() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn wiki_page_search_text(relative_path: &str) -> Option<String> {
    let text = read_repo_text_file(relative_path)?;
    Some(text.to_ascii_lowercase())
}

fn knowledge_page_excerpt(relative_path: &str) -> Option<Value> {
    let text = read_repo_text_file(relative_path)?;
    let excerpt = compact_wiki_excerpt(&text, 1800);
    if excerpt.trim().is_empty() {
        return None;
    }
    Some(json!({
        "path": relative_path,
        "excerpt": excerpt,
    }))
}

fn read_repo_text_file(relative_path: &str) -> Option<String> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        let candidate = dir.join(relative_path);
        if candidate.exists() {
            return fs::read_to_string(candidate).ok();
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn compact_wiki_excerpt(text: &str, max_chars: usize) -> String {
    let mut in_frontmatter = false;
    let mut saw_frontmatter_start = false;
    let mut in_sources = false;
    let mut excerpt = String::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed == "---" && !saw_frontmatter_start {
            saw_frontmatter_start = true;
            in_frontmatter = true;
            continue;
        }
        if trimmed == "---" && in_frontmatter {
            in_frontmatter = false;
            continue;
        }
        if in_frontmatter {
            continue;
        }
        if trimmed.starts_with("## Sources") || trimmed.starts_with("## Related pages") {
            in_sources = true;
        }
        if in_sources {
            continue;
        }
        if trimmed.starts_with("## Source-backed claims")
            || trimmed.starts_with("## Open questions")
        {
            continue;
        }
        if trimmed.is_empty() && excerpt.ends_with('\n') {
            continue;
        }
        if excerpt.len() + line.len() + 1 > max_chars {
            break;
        }
        excerpt.push_str(line);
        excerpt.push('\n');
    }
    excerpt.trim().to_owned()
}

fn build_pi_review_prompt(project: &ProjectFile, request: &AgentReviewReplyRequest) -> String {
    let draft = planning_draft(project);
    let structural = project.structural_model.clone();
    let review_task = format!(
        "review_reply comment={} selected_chips={} reply={}",
        request.comment,
        request.selected_chips.join(" "),
        request.reply
    );
    let knowledge_context = build_llm_knowledge_context(
        project,
        &draft,
        structural.as_ref(),
        "review_reply",
        &review_task,
    );
    let payload = json!({
        "project": {
            "name": project.name.clone(),
            "planningDraft": draft,
            "unitProfile": project.unit_profile.clone(),
            "structuralModel": structural,
        },
        "commentId": request.comment_id,
        "comment": request.comment.clone(),
        "selectedChips": request.selected_chips.clone(),
        "reply": request.reply.clone(),
        "messages": request.messages.clone(),
        "knowledgeContext": knowledge_context,
        "supportedSectionFamilies": available_section_families(),
        "coordinationGroups": project
            .structural_model
            .as_ref()
            .and_then(|model| build_coordination_report(project, &draft, model).ok())
            .map(|report| report.groups)
            .unwrap_or_default(),
    });
    let line_load_symbol = project.unit_profile.line_load.symbol.as_str();
    format!(
        "You are the structural review agent. Act like a conservative senior structural engineer reviewing a GUI question.\n\
Return only JSON matching the provided schema.\n\
Do not claim to mutate the project. You may only propose actions.\n\
Allowed statuses are needs_more_information and ready_to_apply.\n\
Use knowledgeContext as internal structural knowledge before giving structural recommendations. Ground comments about load application, section-family tradeoffs, connection fixity, coordination groups, restraint/stability, and provenance in that internal knowledge when relevant. Do not mention the wiki, knowledgeContext, retrieved excerpts, sources, or knowledge-base mechanics in user-facing message, followUp, suggestedChips, resolutionSummary, or interpretation; speak directly as Fraia's structural design judgement. Treat internal knowledge as reusable guidance only, never as project-specific approval, code compliance, or a substitute for user confirmations, analysis runs, design actions, check inputs, or check results.\n\
Prefer plain structural language throughout. Avoid specialist shorthand as the main wording, especially for load path, support restraint, connection fixity, frame action, member actions, drift, reactions, releases, haunches, knees, bases, and stability. Include the technical term in parentheses only when it adds useful precision or matches labels elsewhere in the app. Prefer \"the N2/N3 corner connections stay stiff enough to transfer bending\" over bare phrases like \"knee moment continuity\".\n\
If the answer gives enough information, set status ready_to_apply and include concise proposedActions.\n\
Always include suggestedChips. If you need more information, make them the best 2-5 next replies for the current follow-up. Before asking, do a critical-path check: if multiple missing answers are independent, ask them together and make suggestedChips complete combined answer drafts; only serialize dependent questions when one answer is genuinely needed before asking the next. If ready_to_apply, use an empty list.\n\
Always include resolutionSummary. If status is ready_to_apply, resolutionSummary must say exactly what the agent will record/apply when the user resolves the item. If more information is needed, use an empty string.\n\
For load questions, do not mark ready_to_apply unless the user provides a numeric line-load value in {line_load_symbol} or enough tributary-width/load information to derive one, and an exact member or member_group target is known from the comment target or project context. If the load target is ambiguous, ask for the target instead of proposing a load_case-wide action. If ready, state the display load assumption in message, interpretation, and resolutionSummary. Proposed add_load action values must use Fraia canonical SI quantity JSON: for kind \"uniform_line\", set magnitude to {{ \"value\": <N/m>, \"quantityKind\": \"line_load\", \"canonicalUnit\": \"N/m\" }}; include field structural_model.loads, targetKind/targetId from the exact member/member_group target, loadCaseId \"gravity\", and direction {{ x: 0, y: -1, z: 0 }}.\n\
For section-family questions, collect acceptable section families from Context JSON supportedSectionFamilies as options rather than choosing one exact section.\n\
For section-family questions targeting a coordination_group, use actionKind update_planning_draft, field coordinationGroup.allowedSectionFamilies, targetKind coordination_group, and value with allowedSectionFamilies and sectionSelectionPolicy.\n\
For legacy simply-supported beam section-family questions, use actionKind update_planning_draft, field beam.simply_supported.sectionFamilyPreferences, and value with allowedSectionFamilies, preferredSectionFamily, excludedSectionFamilies, and sectionSelectionStrategy.\n\
If the user asks the agent to choose, set sectionSelectionStrategy to a short evidence-backed policy name such as agent_justified and preserve any allowed families they gave. Do not silently turn agent choice into lowest mass unless the project evidence or user explicitly makes lowest mass the objective.\n\
Do not convert UB to 200UB. Exact section sizing is a later analysis step.\n\
Keep message short, friendly, and efficient. Use Markdown naturally when it improves scanning: short headings, bullets, bold labels, numbered steps, and compact tables are acceptable. Do not force a fixed template or pad responses.\n\n\
Context JSON:\n{}",
        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".into())
    )
}

#[allow(dead_code)]
fn review_user_text(request: &AgentReviewReplyRequest) -> String {
    let mut parts = Vec::new();
    parts.extend(request.selected_chips.iter().cloned());
    parts.push(request.reply.clone());
    for message in &request.messages {
        if message.author == "user" {
            parts.push(message.text.clone());
        }
    }
    parts.join(" ").to_ascii_lowercase()
}

#[allow(dead_code)]
fn mentions_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

#[allow(dead_code)]
fn parse_line_load_n_per_m(text: &str) -> Option<f64> {
    parse_quantity(text, QuantityKind::LineLoad).ok()
}

fn format_metric_line_load(load_n_per_m: f64) -> String {
    format_quantity(
        load_n_per_m,
        QuantityKind::LineLoad,
        &metric_structural_unit_profile(),
    )
}

#[allow(dead_code)]
fn parse_section_families(text: &str) -> Vec<String> {
    let text = text.to_ascii_lowercase();
    let mut matches = Vec::new();
    for (index, family) in available_section_families().into_iter().enumerate() {
        let needles = section_family_text_needles(&family);
        if let Some(position) = section_family_match_position(&text, &needles) {
            matches.push((position, index, family));
        }
    }
    matches.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    matches.into_iter().map(|(_, _, family)| family).collect()
}

fn section_family_text_needles(family: &str) -> Vec<String> {
    let mut needles = vec![family.to_ascii_lowercase()];
    match family {
        "UB" => needles.push("universal beam".into()),
        "UC" => needles.push("universal column".into()),
        "PFC" => needles.push("channel".into()),
        "EA" => needles.push("angle".into()),
        _ => {}
    }
    needles
}

fn section_family_match_position(text: &str, needles: &[String]) -> Option<usize> {
    needles
        .iter()
        .filter_map(|needle| text_needle_match_position(text, needle))
        .min()
}

fn text_needle_match_position(text: &str, needle: &str) -> Option<usize> {
    if needle.contains(' ') {
        return text.find(needle);
    }
    let mut token_start = None;
    for (index, character) in text.char_indices() {
        if character.is_ascii_alphanumeric() {
            token_start.get_or_insert(index);
            continue;
        }
        if let Some(start) = token_start.take() {
            if &text[start..index] == needle {
                return Some(start);
            }
        }
    }
    token_start.and_then(|start| (&text[start..] == needle).then_some(start))
}

#[allow(dead_code)]
fn first_comment_target(request: &AgentReviewReplyRequest) -> Option<(String, String)> {
    let target = request
        .comment
        .get("targets")
        .and_then(Value::as_array)
        .and_then(|targets| targets.first())?;
    let kind = target.get("kind").and_then(Value::as_str)?;
    let id = target.get("id").and_then(Value::as_str)?;
    Some((kind.to_owned(), id.to_owned()))
}

fn normalize_agent_status(status: &str) -> String {
    match status {
        "ready_to_apply" => "ready_to_apply".into(),
        _ => "needs_more_information".into(),
    }
}

fn apply_agent_action_to_structural_model(
    model: &mut StructuralModel,
    action: &AgentProposedAction,
) -> Result<String> {
    match action.action_kind.as_str() {
        "add_load" => add_load_from_agent_action(model, action),
        "add_support" => add_support_from_agent_action(model, action),
        _ => Err(anyhow!(
            "unsupported structural review action `{}`",
            action.action_kind
        )),
    }
}

fn add_support_from_agent_action(
    model: &mut StructuralModel,
    action: &AgentProposedAction,
) -> Result<String> {
    if action.field != "structural_model.supports" {
        return Err(anyhow!(
            "add_support action must target structural_model.supports, got `{}`",
            action.field
        ));
    }
    if action.target_kind != "node" {
        return Err(anyhow!(
            "add_support action must target a node, got `{}`",
            action.target_kind
        ));
    }
    if !model.nodes.iter().any(|node| node.id == action.target_id) {
        return Err(anyhow!(
            "support target node `{}` does not exist",
            action.target_id
        ));
    }
    if model
        .supports
        .iter()
        .any(|support| support.target_node == action.target_id)
    {
        return Ok(format!(
            "node {} already has a support assignment",
            action.target_id
        ));
    }
    let support_type = action
        .value
        .get("supportType")
        .and_then(Value::as_str)
        .unwrap_or("pinned");
    let default_dofs = support_dofs_for_type(support_type);
    let support = SupportAssignment {
        id: unique_support_id(model, &format!("agent-support-{}", action.target_id)),
        target_node: action.target_id.clone(),
        ux: action
            .value
            .get("ux")
            .and_then(Value::as_bool)
            .unwrap_or(default_dofs.0),
        uy: action
            .value
            .get("uy")
            .and_then(Value::as_bool)
            .unwrap_or(default_dofs.1),
        uz: action
            .value
            .get("uz")
            .and_then(Value::as_bool)
            .unwrap_or(default_dofs.2),
        rx: action
            .value
            .get("rx")
            .and_then(Value::as_bool)
            .unwrap_or(default_dofs.3),
        ry: action
            .value
            .get("ry")
            .and_then(Value::as_bool)
            .unwrap_or(default_dofs.4),
        rz: action
            .value
            .get("rz")
            .and_then(Value::as_bool)
            .unwrap_or(default_dofs.5),
    };
    let id = support.id.clone();
    model.supports.push(support);
    Ok(format!(
        "added {support_type} support `{id}` at node {}",
        action.target_id
    ))
}

fn add_load_from_agent_action(
    model: &mut StructuralModel,
    action: &AgentProposedAction,
) -> Result<String> {
    if action.field != "structural_model.loads" && action.field != "loads" {
        return Err(anyhow!(
            "add_load action must target structural_model.loads, got `{}`",
            action.field
        ));
    }
    let kind = action
        .value
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or("uniform_line");
    let kind = match kind {
        "uniform_line" | "member_line_load" | "line_load" => LoadKind::UniformLine,
        "point" => LoadKind::Point,
        "area" => LoadKind::Area,
        other => return Err(anyhow!("unsupported load kind `{other}`")),
    };
    let raw_magnitude = action
        .value
        .get("magnitude")
        .ok_or_else(|| anyhow!("add_load action requires magnitude"))?;
    let magnitude = canonical_load_magnitude(raw_magnitude, kind, action)?;
    if !magnitude.is_finite() || magnitude.abs() <= 1e-9 {
        return Err(anyhow!(
            "add_load action requires non-zero finite magnitude"
        ));
    }
    let load_case_id = action
        .value
        .get("loadCaseId")
        .and_then(Value::as_str)
        .unwrap_or("gravity")
        .to_owned();
    let direction_value = action.value.get("direction").unwrap_or(&Value::Null);
    let direction = LoadVector {
        x: direction_value
            .get("x")
            .and_then(Value::as_f64)
            .unwrap_or(0.0),
        y: direction_value
            .get("y")
            .and_then(Value::as_f64)
            .unwrap_or(-1.0),
        z: direction_value
            .get("z")
            .and_then(Value::as_f64)
            .unwrap_or(0.0),
    };
    if direction.x.abs() <= 1e-9 && direction.y.abs() <= 1e-9 && direction.z.abs() <= 1e-9 {
        return Err(anyhow!("add_load action requires a non-zero direction"));
    }

    let member_ids = target_member_ids_for_action(model, action)?;
    if kind != LoadKind::UniformLine {
        return Err(anyhow!(
            "only uniform line loads on members are supported by this review apply path"
        ));
    }
    ensure_load_case(model, &load_case_id);
    for member_id in &member_ids {
        if !model.members.iter().any(|member| &member.id == member_id) {
            return Err(anyhow!("load target member `{member_id}` does not exist"));
        }
    }
    let mut added = Vec::new();
    for member_id in member_ids {
        let id = unique_load_id(model, &format!("agent-load-{member_id}"));
        model.loads.push(LoadAssignment {
            id: id.clone(),
            target: AssignmentTargetRef::Member(member_id.clone()),
            load_case_id: load_case_id.clone(),
            kind: LoadKind::UniformLine,
            direction: direction.clone(),
            magnitude,
        });
        added.push(id);
    }
    Ok(format!(
        "added {} uniform line load(s): {}",
        added.len(),
        added.join(", ")
    ))
}

fn canonical_load_magnitude(
    magnitude: &Value,
    kind: LoadKind,
    action: &AgentProposedAction,
) -> Result<f64> {
    let quantity_kind = match kind {
        LoadKind::Point => QuantityKind::Force,
        LoadKind::UniformLine => QuantityKind::LineLoad,
        LoadKind::Area => {
            return Err(anyhow!(
                "canonical SI conversion for area loads is not supported in this apply path"
            ));
        }
    };
    if let Some(number) = magnitude.as_f64() {
        let Some(unit) = action
            .value
            .get("magnitudeUnit")
            .or_else(|| action.value.get("unit"))
            .and_then(Value::as_str)
        else {
            return Ok(number);
        };
        return canonical_value_from_unit(number, quantity_kind, unit).ok_or_else(|| {
            anyhow!(
                "unsupported magnitude unit `{unit}` for load kind `{}`",
                kind.as_str()
            )
        });
    }
    if magnitude.is_object() {
        return match kind {
            LoadKind::Point => serde_json::from_value::<Force>(magnitude.clone())
                .map(|quantity| quantity.newtons())
                .context("add_load action magnitude must be a force quantity"),
            LoadKind::UniformLine => serde_json::from_value::<LineLoad>(magnitude.clone())
                .map(|quantity| quantity.newtons_per_meter())
                .context("add_load action magnitude must be a line_load quantity"),
            LoadKind::Area => serde_json::from_value::<Stress>(magnitude.clone())
                .map(|quantity| quantity.pascals())
                .context("add_load action magnitude must be a stress quantity"),
        };
    }
    Err(anyhow!(
        "add_load action magnitude must be a number or canonical SI quantity object"
    ))
}

fn target_member_ids_for_action(
    model: &StructuralModel,
    action: &AgentProposedAction,
) -> Result<Vec<String>> {
    match action.target_kind.as_str() {
        "member" => Ok(vec![action.target_id.clone()]),
        "member_group" => {
            let report = understand_structural_model(model);
            let group = report
                .member_groups
                .iter()
                .find(|group| group.id == action.target_id)
                .ok_or_else(|| anyhow!("member group `{}` does not exist", action.target_id))?;
            Ok(group.member_ids.clone())
        }
        "LoadCase" | "load_case" => Err(anyhow!(
            "add_load does not infer member targets from load case `{}`; target an explicit member or member_group instead",
            action.target_id
        )),
        other => Err(anyhow!(
            "add_load currently supports member/member_group targets, got `{other}`"
        )),
    }
}

fn ensure_load_case(model: &mut StructuralModel, load_case_id: &str) {
    if !model
        .load_cases
        .iter()
        .any(|load_case| load_case.id == load_case_id)
    {
        model.load_cases.push(LoadCase2D {
            id: load_case_id.to_owned(),
            nodal_loads: Vec::new(),
        });
    }
}

fn unique_load_id(model: &StructuralModel, base: &str) -> String {
    let cleaned: String = base
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '-'
            }
        })
        .collect();
    if !model.loads.iter().any(|load| load.id == cleaned) {
        return cleaned;
    }
    for index in 2.. {
        let candidate = format!("{cleaned}-{index}");
        if !model.loads.iter().any(|load| load.id == candidate) {
            return candidate;
        }
    }
    unreachable!()
}

fn apply_agent_action_to_draft(
    project: &ProjectFile,
    draft: &mut CorePlanningDraft,
    action: &AgentProposedAction,
) -> Result<String> {
    match action.field.as_str() {
        "coordination.designOptionReplacement" => {
            apply_design_option_replacement_to_draft(project, draft, action)
        }
        "coordination.designOptionIntents" => {
            let intents_value = action
                .value
                .get("designOptionIntents")
                .cloned()
                .ok_or_else(|| anyhow!("designOptionIntents action value is required"))?;
            let intents: Vec<DesignOptionIntent> =
                serde_json::from_value(intents_value).context("invalid DesignOptionIntent list")?;
            if intents.is_empty() {
                return Err(anyhow!("at least one DesignOptionIntent is required"));
            }
            validate_design_option_intents(&intents)?;
            validate_design_option_supports_are_realizable(project, &intents)?;
            draft
                .system_parameters
                .insert("designOptionIntents".into(), json!(intents));
            Ok("design option intents updated".into())
        }
        "coordinationGroup.allowedSectionFamilies" => {
            let allowed = json_string_array(&action.value, "allowedSectionFamilies");
            if allowed.is_empty() {
                return Err(anyhow!("at least one allowed section family is required"));
            }
            let policy = action
                .value
                .get("sectionSelectionPolicy")
                .or_else(|| action.value.get("sectionSelectionStrategy"))
                .and_then(Value::as_str)
                .unwrap_or(AGENT_JUSTIFIED_SECTION_SELECTION_POLICY);
            let groups = draft
                .system_parameters
                .entry("coordinationGroups".into())
                .or_insert_with(|| json!({}));
            let Some(groups_object) = groups.as_object_mut() else {
                return Err(anyhow!(
                    "coordinationGroups system parameters are not an object"
                ));
            };
            let group = groups_object
                .entry(action.target_id.clone())
                .or_insert_with(|| json!({}));
            let Some(group_object) = group.as_object_mut() else {
                return Err(anyhow!("coordination group preferences are not an object"));
            };
            group_object.insert(
                "allowedSectionFamilies".into(),
                json!(normalise_section_families(&allowed)),
            );
            group_object.insert(
                "sectionSelectionPolicy".into(),
                Value::String(policy.into()),
            );
            Ok(format!(
                "coordination group `{}` section-family preferences updated",
                action.target_id
            ))
        }
        "coordinationGroup.sectionSelectionPolicy" => {
            let policy = action
                .value
                .get("sectionSelectionPolicy")
                .or_else(|| action.value.get("sectionSelectionStrategy"))
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow!("section selection policy is required"))?;
            let groups = draft
                .system_parameters
                .entry("coordinationGroups".into())
                .or_insert_with(|| json!({}));
            let Some(groups_object) = groups.as_object_mut() else {
                return Err(anyhow!(
                    "coordinationGroups system parameters are not an object"
                ));
            };
            let group = groups_object
                .entry(action.target_id.clone())
                .or_insert_with(|| json!({}));
            let Some(group_object) = group.as_object_mut() else {
                return Err(anyhow!("coordination group preferences are not an object"));
            };
            group_object.insert(
                "sectionSelectionPolicy".into(),
                Value::String(policy.into()),
            );
            Ok(format!(
                "coordination group `{}` section-selection policy updated",
                action.target_id
            ))
        }
        "coordinationGroup.connectionPreference" => {
            let preference = action
                .value
                .get("connectionPreference")
                .and_then(Value::as_str)
                .unwrap_or("review_required");
            let groups = draft
                .system_parameters
                .entry("coordinationGroups".into())
                .or_insert_with(|| json!({}));
            let Some(groups_object) = groups.as_object_mut() else {
                return Err(anyhow!(
                    "coordinationGroups system parameters are not an object"
                ));
            };
            let group = groups_object
                .entry(action.target_id.clone())
                .or_insert_with(|| json!({}));
            let Some(group_object) = group.as_object_mut() else {
                return Err(anyhow!("coordination group preferences are not an object"));
            };
            group_object.insert(
                "connectionPreference".into(),
                Value::String(preference.into()),
            );
            Ok(format!(
                "coordination group `{}` connection preference updated",
                action.target_id
            ))
        }
        "beam.simply_supported.sectionFamilyPreferences" => {
            let allowed = json_string_array(&action.value, "allowedSectionFamilies");
            if allowed.is_empty() {
                return Err(anyhow!("at least one allowed section family is required"));
            }
            let excluded = json_string_array(&action.value, "excludedSectionFamilies");
            let preferred = action
                .value
                .get("preferredSectionFamily")
                .and_then(Value::as_str)
                .map(str::to_owned);
            let strategy = action
                .value
                .get("sectionSelectionStrategy")
                .and_then(Value::as_str)
                .unwrap_or(AGENT_JUSTIFIED_SECTION_SELECTION_POLICY);
            let beam_parameters = draft
                .system_parameters
                .entry("beam.simply_supported".into())
                .or_insert_with(|| json!({}));
            let Some(object) = beam_parameters.as_object_mut() else {
                return Err(anyhow!(
                    "beam.simply_supported system parameters are not an object"
                ));
            };
            object.insert("allowedSectionFamilies".into(), json!(allowed));
            object.insert("excludedSectionFamilies".into(), json!(excluded));
            object.insert(
                "sectionSelectionStrategy".into(),
                Value::String(strategy.into()),
            );
            if let Some(preferred) = preferred {
                object.insert("preferredSectionFamily".into(), Value::String(preferred));
            } else {
                object.remove("preferredSectionFamily");
            }
            object.remove("preferredSection");
            Ok("beam section-family preferences updated".into())
        }
        "beam.simply_supported.preferredSection" => {
            let section = action
                .value
                .as_str()
                .ok_or_else(|| anyhow!("preferredSection action value must be a string"))?;
            if section_by_id(section).is_none() {
                return Err(anyhow!(
                    "preferredSection `{section}` is not available in the current catalogue"
                ));
            }
            let beam_parameters = draft
                .system_parameters
                .entry("beam.simply_supported".into())
                .or_insert_with(|| json!({}));
            let Some(object) = beam_parameters.as_object_mut() else {
                return Err(anyhow!(
                    "beam.simply_supported system parameters are not an object"
                ));
            };
            object.insert("preferredSection".into(), Value::String(section.into()));
            Ok(format!("beam preferred section set to {section}"))
        }
        other => Err(anyhow!("unsupported agent review action field `{other}`")),
    }
}

fn apply_design_option_replacement_to_draft(
    project: &ProjectFile,
    draft: &mut CorePlanningDraft,
    action: &AgentProposedAction,
) -> Result<String> {
    let superseded_id = action
        .value
        .get("supersededOptionId")
        .and_then(Value::as_str)
        .filter(|id| !id.trim().is_empty())
        .unwrap_or(&action.target_id)
        .trim()
        .to_string();
    if superseded_id.is_empty() {
        return Err(anyhow!(
            "design-option replacement requires supersededOptionId"
        ));
    }
    let replacement_value = action
        .value
        .get("replacementDesignOptionIntent")
        .cloned()
        .ok_or_else(|| anyhow!("replacementDesignOptionIntent action value is required"))?;
    let mut replacement: DesignOptionIntent = serde_json::from_value(replacement_value)
        .context("invalid replacementDesignOptionIntent")?;
    if replacement.id.trim().is_empty() {
        return Err(anyhow!("replacement DesignOptionIntent requires id"));
    }
    if replacement.id == superseded_id {
        return Err(anyhow!(
            "replacement DesignOptionIntent id must differ from superseded option id"
        ));
    }
    replacement.lifecycle_status = Some("active".into());
    replacement.revision_of = Some(superseded_id.clone());
    replacement.superseded_by = None;
    replacement.superseded_reason = None;

    validate_design_option_intents(&[replacement.clone()])?;
    validate_design_option_supports_are_realizable(project, &[replacement.clone()])?;

    let mut intents = authored_design_option_intents(draft);
    let original_index = intents
        .iter()
        .position(|intent| intent.id == superseded_id)
        .ok_or_else(|| {
            anyhow!("design option `{superseded_id}` does not exist and cannot be superseded")
        })?;
    if intents.iter().any(|intent| intent.id == replacement.id) {
        return Err(anyhow!(
            "replacement design option `{}` already exists",
            replacement.id
        ));
    }
    let original = &mut intents[original_index];
    if design_option_lifecycle_status(original) == "superseded" {
        return Err(anyhow!(
            "design option `{superseded_id}` is already superseded"
        ));
    }
    original.lifecycle_status = Some("superseded".into());
    original.superseded_by = Some(replacement.id.clone());
    original.superseded_reason = action
        .value
        .get("supersededReason")
        .or_else(|| action.value.get("replacementReason"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|reason| !reason.is_empty())
        .map(str::to_owned)
        .or_else(|| Some(action.summary.clone()));
    intents.push(replacement.clone());
    validate_design_option_intents(&intents)?;
    validate_design_option_supports_are_realizable(project, &intents)?;
    draft
        .system_parameters
        .insert("designOptionIntents".into(), json!(intents));
    Ok(format!(
        "superseded design option `{superseded_id}` with replacement `{}`",
        replacement.id
    ))
}

fn unique_support_id(model: &StructuralModel, base: &str) -> String {
    let cleaned: String = base
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '-'
            }
        })
        .collect();
    if !model.supports.iter().any(|support| support.id == cleaned) {
        return cleaned;
    }
    for index in 2.. {
        let candidate = format!("{cleaned}-{index}");
        if !model.supports.iter().any(|support| support.id == candidate) {
            return candidate;
        }
    }
    unreachable!("unbounded support id search should always return")
}

fn json_string_array(value: &Value, field: &str) -> Vec<String> {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn select_demo_section_from_family_preferences(
    params: &BeamPlanningSystemParameters,
) -> Option<String> {
    let allowed = normalise_section_families(params.allowed_section_families.as_ref()?);
    let excluded = normalise_section_families(
        params
            .excluded_section_families
            .as_deref()
            .unwrap_or_default(),
    );
    let mut family_order = Vec::new();
    if let Some(preferred) = &params.preferred_section_family {
        let preferred = preferred.trim().to_ascii_uppercase();
        if allowed.iter().any(|family| family == &preferred)
            && !excluded.iter().any(|family| family == &preferred)
        {
            family_order.push(preferred);
        }
    }
    family_order.extend(
        allowed
            .into_iter()
            .filter(|family| !excluded.iter().any(|excluded| excluded == family)),
    );
    unique_strings(family_order)
        .into_iter()
        .find_map(|family| first_catalog_section_for_family(&family))
}

fn first_catalog_section_for_family(family: &str) -> Option<String> {
    sections_for_family(family)
        .into_iter()
        .next()
        .map(|section| section.id)
}

fn first_catalog_section_id() -> Option<String> {
    section_catalog().first().map(|section| section.id.clone())
}

fn safe_identifier(value: &str) -> String {
    let cleaned = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '-'
            }
        })
        .collect::<String>();
    cleaned.trim_matches('-').to_owned()
}

fn base_model_fingerprint(project: &ProjectFile) -> String {
    let payload = serde_json::to_vec(&json!({
        "structuralModel": project.structural_model,
        "baseModelBrief": project.base_model_brief,
    }))
    .unwrap_or_default();
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in payload {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64-{hash:016x}")
}

fn design_option_revision_id(batch_id: &str, option_id: &str) -> String {
    format!("{batch_id}::revision::{option_id}")
}

fn next_design_option_batch_id(project: &ProjectFile) -> String {
    let base = format!("design-option-batch-{}", fraia_core::utils::timestamp_id());
    if !project
        .design_option_decisions
        .batches
        .iter()
        .any(|batch| batch.id == base)
    {
        return base;
    }
    let mut suffix = 2;
    loop {
        let candidate = format!("{base}-{suffix}");
        if !project
            .design_option_decisions
            .batches
            .iter()
            .any(|batch| batch.id == candidate)
        {
            return candidate;
        }
        suffix += 1;
    }
}

fn ensure_design_option_revision_identities(project: &mut ProjectFile) {
    let active_batch_id = project.design_option_decisions.active_batch_id.clone();
    for batch in &mut project.design_option_decisions.batches {
        for revision in &mut batch.option_revisions {
            if revision.revision_id.is_empty() {
                revision.revision_id = design_option_revision_id(&batch.id, &revision.option_id);
            }
        }
        for comparison in &mut batch.comparison_runs {
            for reference in &mut comparison.evidence_references {
                if batch
                    .option_revisions
                    .iter()
                    .any(|revision| revision.revision_id == reference.option_revision_id)
                {
                    continue;
                }
                if let Some(revision) = batch
                    .option_revisions
                    .iter()
                    .find(|revision| revision.option_id == reference.option_revision_id)
                {
                    reference.option_revision_id = revision.revision_id.clone();
                }
            }
        }
    }

    let revisions = project
        .design_option_decisions
        .batches
        .iter()
        .flat_map(|batch| {
            let active = Some(&batch.id) == active_batch_id.as_ref();
            batch.option_revisions.iter().map(move |revision| {
                (
                    active,
                    revision.option_id.clone(),
                    revision.revision_id.clone(),
                    revision.latest_analysis_run_id.clone(),
                )
            })
        })
        .collect::<Vec<_>>();
    for path in &mut project.design_option_decisions.development_paths {
        if revisions
            .iter()
            .any(|(_, _, revision_id, _)| revision_id == &path.option_revision_id)
        {
            continue;
        }
        let source_match = path
            .source_analysis_run_id
            .as_ref()
            .and_then(|source_run_id| {
                revisions.iter().rev().find(|(_, option_id, _, run_id)| {
                    option_id == &path.option_id && run_id.as_ref() == Some(source_run_id)
                })
            });
        let active_match = revisions
            .iter()
            .rev()
            .find(|(active, option_id, _, _)| *active && option_id == &path.option_id);
        let legacy_id_match = revisions
            .iter()
            .rev()
            .find(|(_, option_id, _, _)| option_id == &path.option_revision_id);
        let option_match = revisions
            .iter()
            .rev()
            .find(|(_, option_id, _, _)| option_id == &path.option_id);
        if let Some((_, _, revision_id, _)) = source_match
            .or(active_match)
            .or(legacy_id_match)
            .or(option_match)
        {
            path.option_revision_id = revision_id.clone();
        }
    }
}

fn refresh_design_option_batch_freshness(project: &mut ProjectFile) {
    let fingerprint = base_model_fingerprint(project);
    let active_id = project.design_option_decisions.active_batch_id.clone();
    let Some(batch) = project
        .design_option_decisions
        .batches
        .iter_mut()
        .find(|batch| Some(&batch.id) == active_id.as_ref())
    else {
        return;
    };
    if batch.status == "active" && batch.base_model_fingerprint != fingerprint {
        batch.status = "outdated".into();
        for revision in &mut batch.option_revisions {
            revision.analysis_status = "stale".into();
        }
    }
}

fn effective_design_option_decisions(
    project: &ProjectFile,
) -> fraia_core::DesignOptionDecisionState {
    let mut cloned = project.clone();
    ensure_active_design_option_batch(&mut cloned);
    refresh_design_option_batch_freshness(&mut cloned);
    sync_active_design_option_revisions(&mut cloned);
    cloned.design_option_decisions
}

fn archive_active_design_option_batch(project: &mut ProjectFile) {
    ensure_design_option_revision_identities(project);
    let active_id = project.design_option_decisions.active_batch_id.clone();
    let mut archive_id = active_id
        .clone()
        .unwrap_or_else(|| format!("legacy-{}", fraia_core::utils::timestamp_id()));
    if let Some(batch) = project
        .design_option_decisions
        .batches
        .iter_mut()
        .find(|batch| Some(&batch.id) == active_id.as_ref())
    {
        archive_id = batch.id.clone();
        if batch.status == "active" {
            batch.status = "superseded".into();
        }
    }
    for session in &mut project.agent_state.sessions {
        if let Some(option_id) = session.surface.strip_prefix("scheme:") {
            session.surface = format!("scheme-history:{archive_id}:{option_id}");
        }
    }
    project.design_option_decisions.active_batch_id = None;
}

fn latest_ai_provenance(project: &ProjectFile, surface: &str) -> Option<AiProvenance> {
    project
        .agent_state
        .sessions
        .iter()
        .find(|session| session.surface == surface)
        .and_then(|session| {
            session.messages.iter().rev().find_map(|message| {
                Some(AiProvenance {
                    provider_id: message.provider_id.clone()?,
                    model_id: message.model.clone()?,
                    reasoning_effort: message.reasoning_effort.clone()?,
                    catalogue_refreshed_at: message.catalogue_refreshed_at.clone(),
                })
            })
        })
}

fn create_active_design_option_batch(project: &mut ProjectFile) {
    ensure_design_option_revision_identities(project);
    let id = next_design_option_batch_id(project);
    let ai_provenance = latest_ai_provenance(project, "pre_solve");
    let intents = authored_design_option_intents(&planning_draft(project));
    let revisions = intents
        .into_iter()
        .filter(|intent| design_option_lifecycle_status(intent) == "active")
        .map(|intent| DesignOptionRevision {
            revision_id: design_option_revision_id(&id, &intent.id),
            option_id: intent.id.clone(),
            label: if intent.label.trim().is_empty() {
                intent.id.clone()
            } else {
                intent.label.clone()
            },
            revision_of: intent.revision_of.clone(),
            included: true,
            analysis_status: "not_run".into(),
            latest_analysis_run_id: None,
            ai_provenance: ai_provenance.clone(),
        })
        .collect::<Vec<_>>();
    project
        .design_option_decisions
        .batches
        .push(DesignOptionBatch {
            id: id.clone(),
            generated_at: fraia_core::utils::iso_now(),
            base_model_fingerprint: base_model_fingerprint(project),
            status: "active".into(),
            ai_provenance,
            option_revisions: revisions,
            comparison_runs: Vec::new(),
        });
    project.design_option_decisions.active_batch_id = Some(id);
}

fn ensure_active_design_option_batch(project: &mut ProjectFile) {
    ensure_design_option_revision_identities(project);
    if project.design_option_decisions.active_batch_id.is_some() {
        return;
    }
    if authored_design_option_intents(&planning_draft(project))
        .iter()
        .any(|intent| design_option_lifecycle_status(intent) == "active")
    {
        create_active_design_option_batch(project);
    }
}

fn sync_active_design_option_revisions(project: &mut ProjectFile) {
    let active_id = project.design_option_decisions.active_batch_id.clone();
    let intents = authored_design_option_intents(&planning_draft(project));
    let default_ai_provenance = latest_ai_provenance(project, "pre_solve");
    let Some(batch) = project
        .design_option_decisions
        .batches
        .iter_mut()
        .find(|batch| Some(&batch.id) == active_id.as_ref())
    else {
        return;
    };
    for intent in intents
        .iter()
        .filter(|intent| design_option_lifecycle_status(intent) == "active")
    {
        if batch
            .option_revisions
            .iter()
            .any(|revision| revision.option_id == intent.id)
        {
            continue;
        }
        let replaced_index = intent.revision_of.as_ref().and_then(|revision_of| {
            batch
                .option_revisions
                .iter()
                .position(|revision| &revision.option_id == revision_of)
        });
        let inherited_inclusion = replaced_index
            .map(|index| batch.option_revisions[index].included)
            .unwrap_or(true);
        if let Some(index) = replaced_index {
            batch.option_revisions[index].included = false;
            batch.option_revisions[index].analysis_status = "superseded".into();
        }
        batch.option_revisions.push(DesignOptionRevision {
            revision_id: design_option_revision_id(&batch.id, &intent.id),
            option_id: intent.id.clone(),
            label: if intent.label.trim().is_empty() {
                intent.id.clone()
            } else {
                intent.label.clone()
            },
            revision_of: intent.revision_of.clone(),
            included: inherited_inclusion,
            analysis_status: "not_run".into(),
            latest_analysis_run_id: None,
            ai_provenance: default_ai_provenance.clone(),
        });
    }
}

fn record_design_option_comparison_run(
    project: &mut ProjectFile,
    run_id: &str,
    analysed_ids: &[String],
    schemes: &[DesignScheme],
) {
    let active_id = project.design_option_decisions.active_batch_id.clone();
    let Some(batch) = project
        .design_option_decisions
        .batches
        .iter_mut()
        .find(|batch| Some(&batch.id) == active_id.as_ref())
    else {
        return;
    };
    for revision in &mut batch.option_revisions {
        if !analysed_ids.contains(&revision.option_id) {
            continue;
        }
        let successful = schemes
            .iter()
            .find(|scheme| scheme.id == revision.option_id)
            .and_then(|scheme| scheme.analysis_summary.as_ref())
            .is_some_and(|summary| summary.status != "not_checkable" && summary.status != "failed");
        revision.analysis_status = if successful { "current" } else { "failed" }.into();
        revision.latest_analysis_run_id = Some(run_id.into());
    }

    let included_ids = batch
        .option_revisions
        .iter()
        .filter(|revision| revision.included)
        .map(|revision| revision.option_id.clone())
        .collect::<Vec<_>>();
    let all_current = batch
        .option_revisions
        .iter()
        .filter(|revision| revision.included)
        .all(|revision| revision.analysis_status == "current");
    let evidence_references = batch
        .option_revisions
        .iter()
        .filter(|revision| revision.included)
        .filter_map(|revision| {
            revision.latest_analysis_run_id.as_ref().map(|run_id| {
                DesignOptionComparisonEvidenceReference {
                    option_revision_id: revision.revision_id.clone(),
                    analysis_run_id: run_id.clone(),
                }
            })
        })
        .collect();
    let recommendation = if all_current {
        schemes
            .iter()
            .filter(|scheme| included_ids.contains(&scheme.id))
            .filter(|scheme| {
                scheme
                    .analysis_summary
                    .as_ref()
                    .and_then(|summary| summary.max_utilization)
                    .is_some_and(|utilization| utilization <= 1.0)
            })
            .min_by(|a, b| {
                a.approximate_mass_kg
                    .unwrap_or(f64::INFINITY)
                    .total_cmp(&b.approximate_mass_kg.unwrap_or(f64::INFINITY))
                    .then_with(|| a.id.cmp(&b.id))
            })
            .or_else(|| {
                schemes
                    .iter()
                    .filter(|scheme| included_ids.contains(&scheme.id))
                    .filter(|scheme| scheme.analysis_summary.is_some())
                    .min_by(|a, b| {
                        a.analysis_summary
                            .as_ref()
                            .and_then(|summary| summary.max_utilization)
                            .unwrap_or(f64::INFINITY)
                            .total_cmp(
                                &b.analysis_summary
                                    .as_ref()
                                    .and_then(|summary| summary.max_utilization)
                                    .unwrap_or(f64::INFINITY),
                            )
                    })
            })
    } else {
        None
    };
    let explanation = recommendation
        .map(|scheme| {
            let mass = scheme
                .approximate_mass_kg
                .map(|value| format!("{value:.0} kg estimated mass"))
                .unwrap_or_else(|| "mass estimate unavailable".into());
            let utilization = scheme
                .analysis_summary
                .as_ref()
                .and_then(|summary| summary.max_utilization)
                .map(|value| format!("{value:.2} maximum preliminary utilisation"))
                .unwrap_or_else(|| "utilisation unavailable".into());
            format!(
                "{} is the lightest included option passing the current preliminary screen ({mass}, {utilization}).",
                scheme.label
            )
        })
        .unwrap_or_else(|| {
            if all_current {
                "No included option currently passes the preliminary screen; compare the governing issues before revising an option.".into()
            } else {
                "Fraia will recommend an option after every included option has current preliminary evidence.".into()
            }
        });
    batch.comparison_runs.push(DesignOptionComparisonRun {
        run_id: run_id.into(),
        created_at: fraia_core::utils::iso_now(),
        option_ids: included_ids,
        evidence_references,
        objective: "lowest estimated mass among options passing the preliminary conservative screen".into(),
        recommended_option_id: recommendation.map(|scheme| scheme.id.clone()),
        explanation,
        limitations: vec![
            "Preliminary linear-elastic analysis is not a full code-compliant design check.".into(),
            "Connection, foundation, buckling, and detailed stability checks remain outside this comparison.".into(),
        ],
    });
}

fn sync_decision_analysis_evidence(
    decisions: &mut fraia_core::DesignOptionDecisionState,
    schemes: &[DesignScheme],
    fallback_run_id: Option<&str>,
) {
    let active_id = decisions.active_batch_id.clone();
    let Some(batch) = decisions
        .batches
        .iter_mut()
        .find(|batch| Some(&batch.id) == active_id.as_ref())
    else {
        return;
    };
    if batch.status != "active" {
        return;
    }
    for revision in &mut batch.option_revisions {
        let Some(scheme) = schemes
            .iter()
            .find(|scheme| scheme.id == revision.option_id)
        else {
            continue;
        };
        let Some(summary) = scheme.analysis_summary.as_ref() else {
            continue;
        };
        revision.analysis_status =
            if summary.status == "failed" || summary.status == "not_checkable" {
                "failed"
            } else {
                "current"
            }
            .into();
        if revision.latest_analysis_run_id.is_none() {
            revision.latest_analysis_run_id = fallback_run_id.map(str::to_owned);
        }
    }
}

fn persist_project_and_markdown(project_dir: &Path, project: &ProjectFile) -> Result<()> {
    save_project(project_dir, project)?;
    update_planning_markdown(project_dir, &default_planning_markdown(project))?;
    update_base_model_brief_artifacts(project_dir, project)?;
    Ok(())
}

fn validate_design_option_supports_are_realizable(
    project: &ProjectFile,
    intents: &[DesignOptionIntent],
) -> Result<()> {
    let model = project.structural_model.as_ref().ok_or_else(|| {
        anyhow!("DesignOptionIntent records require an authored structural model")
    })?;
    let has_authored_supports = !model.supports.is_empty();
    let has_support_locations = !design_scheme_support_location_node_ids(project, model).is_empty();
    for intent in intents {
        let name = if intent.label.trim().is_empty() {
            intent.id.as_str()
        } else {
            intent.label.as_str()
        };
        let Some(mode) = support_mode_from_strategy(&intent.support_strategy) else {
            return Err(anyhow!(
                "DesignOptionIntent `{name}` must state a realizable buildable supportStrategy such as pinned, fixed, or existing authored SupportAssignment objects"
            ));
        };
        match mode {
            SchemeSupportMode::PinnedRoller => {
                return Err(anyhow!(
                    "DesignOptionIntent `{name}` uses a pinned/roller support strategy. Roller supports are allowed as low-level support primitives, but generated design options must use buildable support strategies such as pinned, fixed, or authored SupportAssignment objects."
                ));
            }
            SchemeSupportMode::Authored if !has_authored_supports => {
                return Err(anyhow!(
                    "DesignOptionIntent `{name}` references authored supports, but the Base Model has no SupportAssignment objects"
                ));
            }
            SchemeSupportMode::PinnedPinned | SchemeSupportMode::FixedFixed
                if !has_support_locations && !has_authored_supports =>
            {
                return Err(anyhow!(
                    "DesignOptionIntent `{name}` chooses a support type, but no confirmed support-location nodes are available"
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

fn update_base_model_brief_artifacts(project_dir: &Path, project: &ProjectFile) -> Result<()> {
    let Some(brief) = project.base_model_brief.as_ref() else {
        return Ok(());
    };
    let dir = project_dir.join("generated");
    fs::create_dir_all(&dir)?;
    fraia_core::utils::write_json(&dir.join("base-model-brief.json"), brief)?;
    fs::write(
        dir.join("base-model-brief.md"),
        render_base_model_brief_markdown(brief),
    )?;
    Ok(())
}

fn remove_base_model_brief_artifacts(project_dir: &Path) -> Result<()> {
    let dir = project_dir.join("generated");
    for file_name in ["base-model-brief.json", "base-model-brief.md"] {
        let path = dir.join(file_name);
        if path.exists() {
            fs::remove_file(&path)
                .with_context(|| format!("failed to remove {}", path.display()))?;
        }
    }
    Ok(())
}

fn persist_schema_handoff_snapshot(project_dir: &Path, project: &ProjectFile) -> Result<PathBuf> {
    let brief = project
        .base_model_brief
        .as_ref()
        .ok_or_else(|| anyhow!("base model brief must exist before design-option handoff"))?;
    let run_id = format!(
        "design-option-handoff-{}",
        fraia_core::utils::timestamp_id()
    );
    let run_dir = project_dir.join("runs").join(&run_id);
    fs::create_dir_all(&run_dir)?;
    fraia_core::utils::write_json(&run_dir.join("base-model-brief.json"), brief)?;
    fs::write(
        run_dir.join("base-model-brief.md"),
        render_base_model_brief_markdown(brief),
    )?;
    fraia_core::utils::write_json(
        &run_dir.join("run.json"),
        &json!({
            "runId": run_id,
            "kind": "design_option_handoff",
            "createdAt": fraia_core::utils::iso_now(),
            "readyForSchemas": brief.readiness.ready_for_schemas,
            "manualOverrideAllowed": brief.readiness.manual_override_allowed,
            "unresolvedTopics": brief.readiness.unresolved_topics.clone(),
            "note": "This run snapshots the Base Model Brief for design-option generation provenance. Design-option generation itself is a downstream stage."
        }),
    )?;
    fs::write(
        run_dir.join("summary.md"),
        format!(
            "# Design Option Handoff\n\n\
	Status: {}\n\n\
	The Base Model Brief was snapshotted for downstream design-option generation provenance.\n\n\
	See `base-model-brief.json` for the typed handoff source.\n",
            if brief.readiness.ready_for_schemas {
                "ready"
            } else {
                "manual override / unresolved"
            }
        ),
    )?;
    Ok(run_dir)
}

fn has_schema_handoff_snapshot(project_dir: &Path) -> Result<bool> {
    let runs_dir = project_dir.join("runs");
    if !runs_dir.exists() {
        return Ok(false);
    }
    for entry in fs::read_dir(runs_dir)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let run_name = file_name.to_string_lossy();
        if (run_name.starts_with("schema-generation-")
            || run_name.starts_with("design-option-handoff-"))
            && entry.path().join("base-model-brief.json").exists()
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn render_string_list(items: &[String]) -> String {
    if items.is_empty() {
        "- None recorded yet.\n".into()
    } else {
        items.iter().map(|item| format!("- {item}\n")).collect()
    }
}

fn render_base_model_visual_intent(intent: &BaseModelBriefVisualIntent) -> String {
    let mut lines = Vec::new();
    if intent.support_locations.is_empty() && intent.loads.is_empty() {
        lines.push("- None recorded yet.".to_owned());
    }
    for support in &intent.support_locations {
        lines.push(format!(
            "- Support location `{}` at node `{}` ({})",
            support.id, support.target_node, support.status
        ));
    }
    for load in &intent.loads {
        let target = match load.target.kind.as_str() {
            "all_members" => "all members".to_owned(),
            "member" => format!(
                "member `{}`",
                load.target.member_id.as_deref().unwrap_or("unknown")
            ),
            "node" => format!(
                "node `{}`",
                load.target.node_id.as_deref().unwrap_or("unknown")
            ),
            other => other.to_owned(),
        };
        lines.push(format!("- Load `{}`: {} on {}", load.id, load.kind, target));
    }
    format!("{}\n", lines.join("\n"))
}

fn render_base_model_brief_markdown(brief: &BaseModelBrief) -> String {
    format!(
        "# Base Model Brief\n\n\
Version: {}\n\
Session: {}\n\
Updated: {}\n\n\
## Current Understanding\n{}\n\n\
## Confirmed Intent\n{}\
## Open Questions\n{}\
## Soft Assumptions\n{}\
	## Design Option Guidance\n{}\
## Do Not Decide Yet\n{}\
## Visual Intent\n{}\
## Readiness\n\
	- Ready for design options: {}\n\
- Manual override allowed: {}\n\n\
## Unresolved Topics\n{}",
        brief.version,
        brief.session_id,
        brief.updated_at,
        brief.current_understanding,
        render_string_list(&brief.confirmed_intent),
        render_string_list(&brief.open_questions),
        render_string_list(&brief.soft_assumptions),
        render_string_list(&brief.schema_guidance),
        render_string_list(&brief.do_not_decide_yet),
        render_base_model_visual_intent(&brief.visual_intent),
        brief.readiness.ready_for_schemas,
        brief.readiness.manual_override_allowed,
        render_string_list(&brief.readiness.unresolved_topics),
    )
}

fn materialize_current_planning(project: &mut ProjectFile) -> Result<MaterializeOutcome> {
    let draft = planning_draft(project);
    let readiness = evaluate_analysis_readiness(&draft);
    if readiness.status != "ready" && readiness.status != "ready_with_notes" {
        let message = readiness.summary.clone();
        return Ok(MaterializeOutcome {
            can_analyse: false,
            message: message.clone(),
            run_summary: AnalysisRunSummary {
                status: "not_run".into(),
                analysis_kind: "materialise".into(),
                message,
                run_id: None,
            },
        });
    }

    match supported_family(&draft) {
        SupportedFamily::BeamSimplySupported => {
            let beam_parameters = parse_system_parameters::<BeamPlanningSystemParameters>(
                &draft.system_parameters,
                "beam.simply_supported",
            )?
            .unwrap_or_default();
            let section = beam_parameters
                .preferred_section
                .clone()
                .or_else(|| select_demo_section_from_family_preferences(&beam_parameters))
                .or_else(|| {
                    current_simply_supported_beam_builder_params(project)
                        .map(|params| params.section)
                })
                .or_else(first_catalog_section_id)
                .context("section catalog is empty; cannot materialise simply supported beam")?;
            let graph = simply_supported_beam_builder_graph(
                "builder.beam.planning",
                &section,
                project.requirements.span_m,
                project.requirements.gravity_load_kn_per_m,
                beam_parameters.point_load_kn,
                beam_parameters.point_load_x_m,
                None,
                None,
            );
            let structural = materialize_structural_model_from_builder_graph(&graph)
                .context("failed to materialise simply supported beam from planning draft")?;
            project.intent.building_type = "beam".into();
            project.builder_graph = Some(graph);
            project.legacy_builder_instance = None;
            project.structural_model = Some(structural);
            project.updated_at = Some(fraia_core::utils::iso_now());
            Ok(MaterializeOutcome {
                can_analyse: true,
                message:
                    "Created or updated the simply supported beam model from the planning draft."
                        .into(),
                run_summary: AnalysisRunSummary {
                    status: "completed".into(),
                    analysis_kind: "materialise".into(),
                    message: "Created or updated the simply supported beam model.".into(),
                    run_id: None,
                },
            })
        }
        SupportedFamily::PortalFrame => {
            let frame_parameters = parse_system_parameters::<PortalFramePlanningSystemParameters>(
                &draft.system_parameters,
                "portal_frame",
            )?
            .unwrap_or_default();
            let topology_id = frame_parameters.topology_id.unwrap_or_else(|| {
                match project.requirements.max_internal_columns {
                    0 => "clear_span".into(),
                    1 => "one_internal".into(),
                    _ => "two_internal".into(),
                }
            });
            let beam_section = frame_parameters
                .beam_section
                .or_else(first_catalog_section_id)
                .context("section catalog is empty; cannot materialise portal frame beam")?;
            let column_section = frame_parameters
                .column_section
                .or_else(first_catalog_section_id)
                .context("section catalog is empty; cannot materialise portal frame column")?;
            let graph = portal_frame_builder_graph(
                "builder.frame.planning",
                &topology_id,
                &beam_section,
                &column_section,
                project.requirements.span_m,
                project.requirements.height_m,
                project.requirements.gravity_load_kn_per_m,
                project.requirements.lateral_load_kn,
                None,
                None,
            );
            let structural = materialize_structural_model_from_builder_graph(&graph)
                .context("failed to materialise portal frame from planning draft")?;
            project.intent.building_type = "portal_frame".into();
            project.builder_graph = Some(graph);
            project.legacy_builder_instance = None;
            project.structural_model = Some(structural);
            project.updated_at = Some(fraia_core::utils::iso_now());
            Ok(MaterializeOutcome {
                can_analyse: true,
                message: "Created or updated the portal frame model from the planning draft."
                    .into(),
                run_summary: AnalysisRunSummary {
                    status: "completed".into(),
                    analysis_kind: "materialise".into(),
                    message: "Created or updated the portal frame model.".into(),
                    run_id: None,
                },
            })
        }
        SupportedFamily::Unsupported(family) => {
            let message = format!(
                "The planning draft is valid, but the selected system family `{family}` is not yet supported for model materialisation."
            );
            Ok(MaterializeOutcome {
                can_analyse: false,
                message: message.clone(),
                run_summary: AnalysisRunSummary {
                    status: "not_run".into(),
                    analysis_kind: "materialise".into(),
                    message,
                    run_id: None,
                },
            })
        }
    }
}

fn dispatch_analysis(project_dir: &Path, project: &mut ProjectFile) -> Result<AnalysisOutcome> {
    let draft = planning_draft(project);
    match supported_family(&draft) {
        SupportedFamily::BeamSimplySupported => {
            let run_dir = persist_beam_analysis_run(project_dir, project)?;
            let run_id = run_dir
                .file_name()
                .and_then(|name| name.to_str())
                .map(str::to_owned);
            Ok(AnalysisOutcome {
                message: format!(
                    "Ran simply supported beam analysis. Artefacts saved to {}",
                    run_dir.display()
                ),
                run_summary: AnalysisRunSummary {
                    status: "completed".into(),
                    analysis_kind: "beam_analysis".into(),
                    message: "Ran simply supported beam analysis.".into(),
                    run_id,
                },
            })
        }
        SupportedFamily::PortalFrame => {
            let preferred_backend = draft
                .analysis_brief
                .preferred_backend
                .unwrap_or_else(|| "auto".into());
            if preferred_backend.eq_ignore_ascii_case("calculix") {
                let run_dir = persist_frame_calculix_run(project_dir, project)?;
                let run_id = run_dir
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(str::to_owned);
                return Ok(AnalysisOutcome {
                    message: format!(
                        "Ran portal frame CalculiX analysis. Artefacts saved to {}",
                        run_dir.display()
                    ),
                    run_summary: AnalysisRunSummary {
                        status: "completed".into(),
                        analysis_kind: "frame_calculix".into(),
                        message: "Ran portal frame CalculiX analysis.".into(),
                        run_id,
                    },
                });
            }

            persist_project_and_markdown(project_dir, project)?;
            let run_dir = persist_validation_run(project_dir, project)?;
            let run_id = run_dir
                .file_name()
                .and_then(|name| name.to_str())
                .map(str::to_owned);
            Ok(AnalysisOutcome {
                message: format!(
                    "Ran portal frame validation analysis. Artefacts saved to {}",
                    run_dir.display()
                ),
                run_summary: AnalysisRunSummary {
                    status: "completed".into(),
                    analysis_kind: "validation".into(),
                    message: "Ran portal frame validation analysis.".into(),
                    run_id,
                },
            })
        }
        SupportedFamily::Unsupported(family) => Err(anyhow!(
            "unsupported planning family {family} reached dispatch"
        )),
    }
}

fn persist_design_option_analysis_run(
    project_dir: &Path,
    project: &ProjectFile,
    request: &DesignOptionAnalysisRequest,
) -> Result<PathBuf> {
    let base_model = materialize_project_structural_model(project)
        .context("no authored structural model is available for design-option analysis")?;
    let draft = planning_draft(project);
    let understanding = understand_structural_model(&base_model);
    let groups = build_coordination_groups(&draft, &understanding);
    let candidates = design_scheme_candidates(project, &draft, &base_model, &groups);
    let requested_ids = request
        .scope
        .as_ref()
        .map(|scope| scope.option_ids.clone())
        .unwrap_or_default();
    let analyse_all = requested_ids.is_empty()
        || request
            .scope
            .as_ref()
            .is_some_and(|scope| scope.kind == "all_active_design_options");
    let selected_candidates: Vec<_> = candidates
        .into_iter()
        .filter(|candidate| design_option_lifecycle_status(&candidate.intent) == "active")
        .filter(|candidate| {
            analyse_all || requested_ids.iter().any(|id| id == &candidate.intent.id)
        })
        .collect();
    if selected_candidates.is_empty() {
        return Err(anyhow!(
            "no active design options matched the design-option analysis scope"
        ));
    }

    let candidate_policy = request
        .candidate_policy
        .clone()
        .unwrap_or_else(|| "all_candidates".into());
    let check_profile = request
        .check_profile
        .clone()
        .unwrap_or_else(|| "preliminary_conservative_steel".into());
    let run_id = format!(
        "design-option-analysis-{}",
        fraia_core::utils::timestamp_id()
    );
    let run_dir = project_dir.join("runs").join(&run_id);
    fs::create_dir_all(&run_dir)?;

    let mut candidate_inputs = Vec::new();
    let mut solver_results = Vec::new();
    let mut design_actions = Vec::new();
    let mut preliminary_checks = Vec::new();
    let mut option_results = Vec::new();
    let mut run_diagnostics = Vec::new();

    for candidate in &selected_candidates {
        let choices: Vec<_> = groups
            .iter()
            .map(|group| {
                let preferred_families = scheme_family_order(candidate, group);
                design_scheme_choice(group, &base_model, &preferred_families)
            })
            .collect();
        let option_label = if candidate.intent.label.trim().is_empty() {
            candidate.intent.id.clone()
        } else {
            candidate.intent.label.clone()
        };
        let mut option_candidate_results = Vec::new();
        for choice in &choices {
            let Some(group) = groups
                .iter()
                .find(|group| group.id == choice.coordination_group_id)
            else {
                continue;
            };
            let selected_section = choice.selected_section_id.clone();
            for section_id in &choice.candidate_section_ids {
                let selected_candidate = selected_section.as_ref() == Some(section_id);
                let input = DesignOptionCandidateAnalysisInput {
                    option_id: candidate.intent.id.clone(),
                    option_label: option_label.clone(),
                    coordination_group_id: group.id.clone(),
                    section_id: section_id.clone(),
                    selected_candidate,
                    member_ids: group.member_ids.clone(),
                    standardisation_policy: candidate.intent.standardisation_strategy.clone(),
                };
                candidate_inputs.push(input.clone());
                let result = analyse_design_option_candidate_section(
                    project,
                    &base_model,
                    &groups,
                    &choices,
                    candidate,
                    &input,
                    &run_dir,
                );
                match result {
                    Ok(analysis) => {
                        solver_results.push(json!({
                            "runId": run_id.clone(),
                            "optionId": input.option_id,
                            "coordinationGroupId": input.coordination_group_id,
                            "sectionId": input.section_id,
                            "solver": "calculix.ccx.execute.v1",
                            "realizationDiagnostics": analysis.realization.diagnostics,
                            "compiledInputs": analysis.compiled_inputs,
                            "executions": analysis.executions,
                            "nodeDisplacements": analysis.node_displacements,
                            "supportReactions": analysis.support_reactions,
                            "elementStresses": analysis.element_stresses,
                            "diagrams": analysis.diagrams,
                        }));
                        design_actions.push(json!({
                            "optionId": input.option_id,
                            "coordinationGroupId": input.coordination_group_id,
                            "sectionId": input.section_id,
                            "source": "calculix.ccx.execute.v1",
                            "nodeDisplacements": analysis.node_displacements,
                            "supportReactions": analysis.support_reactions,
                            "elementStresses": analysis.element_stresses,
                            "diagrams": analysis.diagrams,
                        }));
                        preliminary_checks.push(json!({
                            "optionId": input.option_id,
                            "coordinationGroupId": input.coordination_group_id,
                            "sectionId": input.section_id,
                            "checks": analysis.checks,
                        }));
                        option_candidate_results.push(analysis.result);
                    }
                    Err(error) => {
                        let diagnostic = format!("{error:#}");
                        let failed = DesignOptionCandidateAnalysisResult {
                            option_id: input.option_id.clone(),
                            option_label: input.option_label.clone(),
                            coordination_group_id: input.coordination_group_id.clone(),
                            section_id: input.section_id.clone(),
                            status: "not_checkable".into(),
                            passed: None,
                            selected_candidate: input.selected_candidate,
                            approximate_mass_kg: approximate_group_section_mass_kg(
                                &base_model,
                                group,
                                &input.section_id,
                            ),
                            max_utilization: None,
                            max_stress_mpa: None,
                            max_moment_knm: None,
                            max_shear_kn: None,
                            max_deflection_mm: None,
                            max_drift_mm: None,
                            max_reaction_kn: None,
                            governing_member_id: None,
                            governing_combo_id: None,
                            diagnostic: Some(diagnostic.clone()),
                        };
                        run_diagnostics.push(WorkbenchDiagnostic {
                            severity: "warning".into(),
                            code: "design_option_candidate_not_checkable".into(),
                            message: format!(
                                "{} / {} / {} could not be analysed.",
                                input.option_label, input.coordination_group_id, input.section_id
                            ),
                            detail: Some(diagnostic),
                        });
                        option_candidate_results.push(failed);
                    }
                }
            }
        }
        let selected_result = selected_design_option_analysis_result(&option_candidate_results);
        option_results.push(DesignOptionAnalysisOptionResult {
            option_id: candidate.intent.id.clone(),
            option_label,
            lifecycle_status: design_option_lifecycle_status(&candidate.intent).into(),
            selected_result,
            candidate_results: option_candidate_results,
            diagnostics: Vec::new(),
        });
    }

    let run_manifest = DesignOptionAnalysisRunManifest {
        run_id: run_id.clone(),
        run_kind: "design_option_analysis".into(),
        generated_at: fraia_core::utils::iso_now(),
        project_name: project.name.clone(),
        option_ids: option_results
            .iter()
            .map(|result| result.option_id.clone())
            .collect(),
        candidate_policy,
        check_profile,
        solver: "calculix.ccx.execute.v1".into(),
    };
    let comparison = DesignOptionAnalysisComparison {
        run_id: run_id.clone(),
        option_results,
    };

    fraia_core::utils::write_json(&run_dir.join("run.json"), &run_manifest)?;
    fraia_core::utils::write_json(&run_dir.join("option-snapshot.json"), project)?;
    fraia_core::utils::write_json(&run_dir.join("candidate-inputs.json"), &candidate_inputs)?;
    fraia_core::utils::write_json(&run_dir.join("solver-results.json"), &solver_results)?;
    fraia_core::utils::write_json(&run_dir.join("design-actions.json"), &design_actions)?;
    fraia_core::utils::write_json(
        &run_dir.join("preliminary-checks.json"),
        &preliminary_checks,
    )?;
    fraia_core::utils::write_json(&run_dir.join("comparison.json"), &comparison)?;
    fraia_core::utils::write_json(&run_dir.join("diagnostics.json"), &run_diagnostics)?;
    fs::write(
        run_dir.join("summary.md"),
        render_design_option_analysis_summary(&comparison),
    )?;
    Ok(run_dir)
}

fn selected_design_option_analysis_result(
    results: &[DesignOptionCandidateAnalysisResult],
) -> Option<DesignOptionCandidateAnalysisResult> {
    results
        .iter()
        .filter(|result| result.passed == Some(true))
        .min_by(|a, b| {
            optional_mass_sort_key(a)
                .total_cmp(&optional_mass_sort_key(b))
                .then_with(|| a.section_id.cmp(&b.section_id))
        })
        .or_else(|| {
            results
                .iter()
                .filter(|result| result.passed == Some(false))
                .min_by(|a, b| {
                    optional_mass_sort_key(a)
                        .total_cmp(&optional_mass_sort_key(b))
                        .then_with(|| a.section_id.cmp(&b.section_id))
                })
        })
        .or_else(|| results.iter().find(|result| result.selected_candidate))
        .cloned()
}

fn optional_mass_sort_key(result: &DesignOptionCandidateAnalysisResult) -> f64 {
    result.approximate_mass_kg.unwrap_or(f64::INFINITY)
}

struct CandidateSectionAnalysis {
    result: DesignOptionCandidateAnalysisResult,
    realization: fraia_core::Frame2DRealization,
    compiled_inputs: Vec<CalculixCompiledInput>,
    executions: Vec<CalculixExecutionArtifacts>,
    node_displacements: Vec<FrameNodeDisplacementPoint>,
    support_reactions: Vec<FrameSupportReactionPoint>,
    element_stresses: Vec<FrameElementStressSummary>,
    diagrams: Value,
    checks: Vec<Value>,
}

#[derive(Debug, Clone, Default)]
struct FrameActionSummary {
    max_moment_knm: Option<f64>,
    max_shear_kn: Option<f64>,
    governing_member_id: Option<String>,
}

fn analyse_design_option_candidate_section(
    project: &ProjectFile,
    base_model: &StructuralModel,
    groups: &[CoordinationGroup],
    choices: &[DesignSchemeGroupChoice],
    candidate: &DesignSchemeCandidate,
    input: &DesignOptionCandidateAnalysisInput,
    run_dir: &Path,
) -> Result<CandidateSectionAnalysis> {
    let mut model = realised_model_for_design_option(project, base_model, candidate);
    apply_design_option_section_assignments(&mut model, groups, choices, input)?;
    if model.loads.is_empty() {
        apply_base_model_visual_loads(project, &mut model);
    }
    if model.loads.is_empty() {
        return Err(anyhow!(
            "the realised design option has no LoadAssignment objects or visual load intent"
        ));
    }
    let realization = realize_structural_model_to_frame2d(&model)
        .context("failed to realise design option as a frame2d model")?;
    let runtime = require_calculix_runtime()?;
    let mut compiled_inputs = Vec::new();
    let mut executions = Vec::new();
    let mut node_displacements = Vec::new();
    let mut support_reactions = Vec::new();
    let mut element_stresses = Vec::new();
    let analysis_dir = run_dir.join("calculix-jobs").join(format!(
        "{}-{}-{}",
        safe_id_fragment(&input.option_id),
        safe_id_fragment(&input.coordination_group_id),
        safe_id_fragment(&input.section_id)
    ));
    fs::create_dir_all(&analysis_dir)?;
    for combo in &realization.model.combos {
        let job_name = format!(
            "option-{}-{}-{}-{}",
            safe_id_fragment(&input.option_id),
            safe_id_fragment(&input.coordination_group_id),
            safe_id_fragment(&input.section_id),
            safe_id_fragment(&combo.id)
        );
        let compiled = compile_frame_model_to_calculix_input(&realization.model, combo, &job_name)
            .context("failed to compile design option to CalculiX input")?;
        let combo_dir = analysis_dir.join(&job_name);
        let execution =
            execute_calculix_compiled_input_with_runtime(&compiled, &combo_dir, runtime.clone())
                .with_context(|| format!("failed to execute CalculiX job {job_name}"))?;
        if !matches!(execution.outcome, CalculixExecutionOutcome::Completed) {
            return Err(anyhow!(
                "CalculiX job {job_name} did not complete successfully: {:?}. {}{}",
                execution.outcome,
                execution.stdout,
                execution.stderr
            ));
        }
        let dat_path = combo_dir.join(format!("{}.dat", compiled.job_name));
        let dat_text = fs::read_to_string(&dat_path)
            .with_context(|| format!("failed to read CalculiX output {}", dat_path.display()))?;
        let (combo_displacements, combo_supports, combo_stresses) =
            extract_frame_calculix_dat(&dat_text, &realization.model)
                .context("failed to extract design-option CalculiX response")?;
        node_displacements.extend(combo_displacements.unwrap_or_default());
        support_reactions.extend(combo_supports.unwrap_or_default());
        element_stresses.extend(combo_stresses.unwrap_or_default());
        compiled_inputs.push(compiled);
        executions.push(execution);
    }
    let group = groups
        .iter()
        .find(|group| group.id == input.coordination_group_id)
        .ok_or_else(|| anyhow!("missing coordination group {}", input.coordination_group_id))?;
    let frame_actions = frame_action_summary(&realization.model, input);
    let result = candidate_result_from_calculix_analysis(
        project,
        &model,
        group,
        input,
        &frame_actions,
        &node_displacements,
        &support_reactions,
        &element_stresses,
    );
    let checks = vec![json!({
        "kind": "preliminary_conservative_stress_check",
        "checkProfile": "preliminary_conservative_steel",
        "status": result.status,
        "passes": result.passed,
        "maxUtilization": result.max_utilization,
        "maxStressMpa": result.max_stress_mpa,
        "limitUtilization": project.requirements.max_utilization,
        "source": "calculix.ccx.execute.v1"
    })];
    let diagrams = design_option_diagrams(&realization.model, input, &node_displacements);
    Ok(CandidateSectionAnalysis {
        result,
        realization,
        compiled_inputs,
        executions,
        node_displacements,
        support_reactions,
        element_stresses,
        diagrams,
        checks,
    })
}

fn realised_model_for_design_option(
    project: &ProjectFile,
    base_model: &StructuralModel,
    candidate: &DesignSchemeCandidate,
) -> StructuralModel {
    let mut model = base_model.clone();
    if model.supports.is_empty() {
        model.supports = design_scheme_supports(project, base_model, candidate);
    }
    model
}

fn apply_design_option_section_assignments(
    model: &mut StructuralModel,
    groups: &[CoordinationGroup],
    choices: &[DesignSchemeGroupChoice],
    input: &DesignOptionCandidateAnalysisInput,
) -> Result<()> {
    for choice in choices {
        let group = groups
            .iter()
            .find(|group| group.id == choice.coordination_group_id)
            .ok_or_else(|| {
                anyhow!(
                    "missing coordination group {}",
                    choice.coordination_group_id
                )
            })?;
        let section_id = if group.id == input.coordination_group_id {
            input.section_id.as_str()
        } else {
            choice
                .selected_section_id
                .as_deref()
                .ok_or_else(|| anyhow!("coordination group {} has no selected section", group.id))?
        };
        section_by_id(section_id)
            .with_context(|| format!("unknown catalogue section `{section_id}`"))?;
        for member_id in &group.member_ids {
            let member = model
                .members
                .iter_mut()
                .find(|member| member.id == *member_id)
                .ok_or_else(|| {
                    anyhow!("coordination group references missing Member {member_id}")
                })?;
            member.section_id = section_id.to_owned();
        }
    }
    Ok(())
}

fn apply_base_model_visual_loads(project: &ProjectFile, model: &mut StructuralModel) {
    let Some(brief) = project.base_model_brief.as_ref() else {
        return;
    };
    for load in &brief.visual_intent.loads {
        match load.kind.as_str() {
            "self_weight" => apply_visual_self_weight_load(load, model),
            "uniform_line" => apply_visual_uniform_line_load(load, model),
            "point" => apply_visual_point_load(load, model),
            _ => {}
        }
    }
}

fn apply_visual_self_weight_load(
    load: &fraia_core::BaseModelBriefLoadIntent,
    model: &mut StructuralModel,
) {
    let member_ids: Vec<String> = match load.target.kind.as_str() {
        "member" => load.target.member_id.clone().into_iter().collect(),
        "all_members" => model
            .members
            .iter()
            .map(|member| member.id.clone())
            .collect(),
        _ => Vec::new(),
    };
    if member_ids.is_empty() {
        return;
    }
    ensure_load_case(model, "self_weight");
    for member_id in member_ids {
        let Some(member) = model.members.iter().find(|member| member.id == member_id) else {
            continue;
        };
        let Some(section) = section_by_id(&member.section_id) else {
            continue;
        };
        let id = unique_load_id(model, &format!("option-self-weight-{member_id}"));
        model.loads.push(LoadAssignment {
            id,
            target: AssignmentTargetRef::Member(member_id),
            load_case_id: "self_weight".into(),
            kind: LoadKind::UniformLine,
            direction: LoadVector {
                x: 0.0,
                y: -1.0,
                z: 0.0,
            },
            magnitude: section.mass_kg_per_m * 9.806_65,
        });
    }
}

fn apply_visual_uniform_line_load(
    load: &fraia_core::BaseModelBriefLoadIntent,
    model: &mut StructuralModel,
) {
    let Some(member_id) = load.target.member_id.as_ref() else {
        return;
    };
    let Some(magnitude) = load.magnitude_n_per_m else {
        return;
    };
    if !model.members.iter().any(|member| &member.id == member_id) {
        return;
    }
    let Some(direction) = resolved_visual_load_direction(load.direction.as_ref(), model) else {
        return;
    };
    ensure_load_case(model, "concept");
    let id = unique_load_id(model, &format!("option-line-load-{member_id}"));
    model.loads.push(LoadAssignment {
        id,
        target: AssignmentTargetRef::Member(member_id.clone()),
        load_case_id: "concept".into(),
        kind: LoadKind::UniformLine,
        direction,
        magnitude,
    });
}

fn apply_visual_point_load(
    load: &fraia_core::BaseModelBriefLoadIntent,
    model: &mut StructuralModel,
) {
    let Some(node_id) = load.target.node_id.as_ref() else {
        return;
    };
    let Some(magnitude) = load.magnitude_n else {
        return;
    };
    if !model.nodes.iter().any(|node| &node.id == node_id) {
        return;
    }
    let Some(direction) = resolved_visual_load_direction(load.direction.as_ref(), model) else {
        return;
    };
    ensure_load_case(model, "concept");
    let id = unique_load_id(model, &format!("option-point-load-{node_id}"));
    model.loads.push(LoadAssignment {
        id,
        target: AssignmentTargetRef::Node(node_id.clone()),
        load_case_id: "concept".into(),
        kind: LoadKind::Point,
        direction,
        magnitude,
    });
}

fn resolved_visual_load_direction(
    direction: Option<&BaseModelBriefLoadDirection>,
    model: &StructuralModel,
) -> Option<LoadVector> {
    let direction = direction?;
    if direction.kind == "toward_node" {
        let from = direction
            .from_node
            .as_ref()
            .and_then(|id| model.node_by_id(id))?;
        let to = direction
            .to_node
            .as_ref()
            .and_then(|id| model.node_by_id(id))?;
        return normalised_load_vector(LoadVector {
            x: to.x - from.x,
            y: to.y - from.y,
            z: to.z - from.z,
        });
    }
    normalised_load_vector(LoadVector {
        x: direction.x.unwrap_or(0.0),
        y: direction.y.unwrap_or(0.0),
        z: direction.z.unwrap_or(0.0),
    })
}

fn normalised_load_vector(vector: LoadVector) -> Option<LoadVector> {
    let length = (vector.x * vector.x + vector.y * vector.y + vector.z * vector.z).sqrt();
    (length > 1e-9).then_some(LoadVector {
        x: vector.x / length,
        y: vector.y / length,
        z: vector.z / length,
    })
}

fn frame_action_summary(
    model: &FrameModel2D,
    input: &DesignOptionCandidateAnalysisInput,
) -> FrameActionSummary {
    let member_filter: BTreeSet<&str> = input.member_ids.iter().map(String::as_str).collect();
    let mut summary = FrameActionSummary::default();
    let mut governing_force = 0.0;
    for combo in &model.combos {
        let Ok(solve) = solve_frame_2d(model, combo) else {
            continue;
        };
        for element in solve.element_results {
            if !member_filter.is_empty() && !member_filter.contains(element.id.as_str()) {
                continue;
            }
            let shear_n = element
                .local_end_forces
                .get(1)
                .copied()
                .unwrap_or(0.0)
                .abs()
                .max(
                    element
                        .local_end_forces
                        .get(4)
                        .copied()
                        .unwrap_or(0.0)
                        .abs(),
                );
            let moment_nm = element
                .local_end_forces
                .get(2)
                .copied()
                .unwrap_or(0.0)
                .abs()
                .max(
                    element
                        .local_end_forces
                        .get(5)
                        .copied()
                        .unwrap_or(0.0)
                        .abs(),
                );
            summary.max_shear_kn = Some(summary.max_shear_kn.unwrap_or(0.0).max(shear_n / 1000.0));
            summary.max_moment_knm = Some(
                summary
                    .max_moment_knm
                    .unwrap_or(0.0)
                    .max(moment_nm / 1000.0),
            );
            let governing_candidate = shear_n.max(moment_nm);
            if governing_candidate > governing_force {
                governing_force = governing_candidate;
                summary.governing_member_id = Some(element.id);
            }
        }
    }
    summary
}

fn design_option_diagrams(
    model: &FrameModel2D,
    input: &DesignOptionCandidateAnalysisInput,
    node_displacements: &[FrameNodeDisplacementPoint],
) -> Value {
    let member_filter: BTreeSet<&str> = input.member_ids.iter().map(String::as_str).collect();
    let node_displacement_lookup: BTreeMap<&str, &FrameNodeDisplacementPoint> = node_displacements
        .iter()
        .map(|point| (point.node_id.as_str(), point))
        .collect();
    let mut action_members = Vec::new();
    if let Some(combo) = model.combos.first() {
        if let Ok(solve) = solve_frame_2d(model, combo) {
            for element_result in solve.element_results {
                if !member_filter.is_empty() && !member_filter.contains(element_result.id.as_str())
                {
                    continue;
                }
                let Some(element) = model
                    .elements
                    .iter()
                    .find(|element| element.id == element_result.id)
                else {
                    continue;
                };
                let Some(start) = model.nodes.iter().find(|node| node.id == element.i) else {
                    continue;
                };
                let Some(end) = model.nodes.iter().find(|node| node.id == element.j) else {
                    continue;
                };
                action_members.push(json!({
                    "memberId": element.id,
                    "role": element.role,
                    "startNodeId": element.i,
                    "endNodeId": element.j,
                    "start": { "xM": start.x, "yM": start.y },
                    "end": { "xM": end.x, "yM": end.y },
                    "shearKn": [
                        element_result.local_end_forces.get(1).copied().unwrap_or(0.0) / 1000.0,
                        element_result.local_end_forces.get(4).copied().unwrap_or(0.0) / 1000.0
                    ],
                    "momentKnm": [
                        element_result.local_end_forces.get(2).copied().unwrap_or(0.0) / 1000.0,
                        element_result.local_end_forces.get(5).copied().unwrap_or(0.0) / 1000.0
                    ],
                    "deflectionMm": [
                        node_displacement_lookup.get(element.i.as_str()).map(|point| point.uy_m * 1000.0).unwrap_or(0.0),
                        node_displacement_lookup.get(element.j.as_str()).map(|point| point.uy_m * 1000.0).unwrap_or(0.0)
                    ]
                }));
            }
        }
    }
    json!({
        "source": "frame2d.internal.actions.v1",
        "members": action_members,
    })
}

fn candidate_result_from_calculix_analysis(
    project: &ProjectFile,
    model: &StructuralModel,
    group: &CoordinationGroup,
    input: &DesignOptionCandidateAnalysisInput,
    frame_actions: &FrameActionSummary,
    node_displacements: &[FrameNodeDisplacementPoint],
    support_reactions: &[FrameSupportReactionPoint],
    element_stresses: &[FrameElementStressSummary],
) -> DesignOptionCandidateAnalysisResult {
    let governing_stress = element_stresses
        .iter()
        .max_by(|a, b| a.max_abs_sxx_pa.total_cmp(&b.max_abs_sxx_pa));
    let max_stress_pa = governing_stress
        .map(|stress| stress.max_abs_sxx_pa)
        .unwrap_or(0.0);
    let max_stress_mpa = max_stress_pa / 1_000_000.0;
    let conservative_limit = steel_material().fy * project.requirements.max_utilization;
    let max_utilization = if conservative_limit > 0.0 {
        max_stress_pa / conservative_limit
    } else {
        f64::INFINITY
    };
    let max_reaction_kn = support_reactions
        .iter()
        .flat_map(|reaction| [reaction.fx_n.abs(), reaction.fy_n.abs()])
        .fold(0.0, f64::max)
        / 1000.0;
    let max_deflection_mm = node_displacements
        .iter()
        .map(|node| node.uy_m.abs())
        .fold(0.0, f64::max)
        * 1000.0;
    let max_drift_mm = node_displacements
        .iter()
        .map(|node| node.ux_m.abs())
        .fold(0.0, f64::max)
        * 1000.0;
    let deflection_limit_mm = if project.requirements.max_deflection_ratio > 0.0 {
        Some(project.requirements.span_m * 1000.0 / project.requirements.max_deflection_ratio)
    } else {
        None
    };
    let drift_limit_mm = if project.requirements.max_drift_ratio > 0.0 {
        Some(project.requirements.height_m * 1000.0 / project.requirements.max_drift_ratio)
    } else {
        None
    };
    let failed = max_utilization > project.requirements.max_utilization
        || deflection_limit_mm.is_some_and(|limit| max_deflection_mm > limit)
        || drift_limit_mm.is_some_and(|limit| max_drift_mm > limit);
    let warning = max_utilization > project.requirements.max_utilization * 0.9
        || deflection_limit_mm.is_some_and(|limit| max_deflection_mm > limit * 0.9)
        || drift_limit_mm.is_some_and(|limit| max_drift_mm > limit * 0.9);
    let status = if failed {
        "fails_preliminary_stress_screen"
    } else if warning {
        "passes_with_warnings"
    } else {
        "passes_preliminary_stress_screen"
    };
    DesignOptionCandidateAnalysisResult {
        option_id: input.option_id.clone(),
        option_label: input.option_label.clone(),
        coordination_group_id: input.coordination_group_id.clone(),
        section_id: input.section_id.clone(),
        status: status.into(),
        passed: Some(!failed),
        selected_candidate: input.selected_candidate,
        approximate_mass_kg: approximate_group_section_mass_kg(model, group, &input.section_id),
        max_utilization: Some(max_utilization),
        max_stress_mpa: Some(max_stress_mpa),
        max_moment_knm: frame_actions.max_moment_knm,
        max_shear_kn: frame_actions.max_shear_kn,
        max_deflection_mm: Some(max_deflection_mm),
        max_drift_mm: Some(max_drift_mm),
        max_reaction_kn: Some(max_reaction_kn),
        governing_member_id: governing_stress
            .map(|stress| stress.element_id.clone())
            .or_else(|| frame_actions.governing_member_id.clone()),
        governing_combo_id: None,
        diagnostic: None,
    }
}

fn approximate_group_section_mass_kg(
    model: &StructuralModel,
    group: &CoordinationGroup,
    section_id: &str,
) -> Option<f64> {
    let section = section_by_id(section_id)?;
    Some(section.mass_kg_per_m * coordination_group_length_m(model, group))
}

fn render_design_option_analysis_summary(comparison: &DesignOptionAnalysisComparison) -> String {
    let mut lines = vec![
        "# Design Option Analysis".to_owned(),
        String::new(),
        format!("Run: `{}`", comparison.run_id),
        String::new(),
        "| Design option | Candidate results | Selected candidate |".to_owned(),
        "| --- | ---: | --- |".to_owned(),
    ];
    for option in &comparison.option_results {
        let selected = option
            .selected_result
            .as_ref()
            .map(|result| {
                format!(
                    "{} / {} / utilisation {}",
                    result.coordination_group_id,
                    result.section_id,
                    result
                        .max_utilization
                        .map(|value| format!("{value:.2}"))
                        .unwrap_or_else(|| "n/a".into())
                )
            })
            .unwrap_or_else(|| "No selected result".into());
        lines.push(format!(
            "| {} | {} | {} |",
            option.option_label,
            option.candidate_results.len(),
            selected
        ));
    }
    lines.push(String::new());
    lines.push(
        "These are preliminary CalculiX beam-model results and conservative stress screens, not code-based member design checks."
            .into(),
    );
    lines.join("\n")
}

fn build_workbench_state(
    project_dir: &Path,
    project: &ProjectFile,
) -> Result<WorkbenchProjectState> {
    let structural_model = materialize_project_structural_model(project);
    let draft = planning_draft(project);
    let mut readiness = evaluate_analysis_readiness(&draft);
    if let Some(model) = structural_model.as_ref() {
        let structural_readiness = evaluate_structural_solve_readiness(model);
        if structural_readiness.status != "ready" {
            readiness = structural_readiness;
        }
    }
    let latest_validate = load_latest_summary(project_dir, "validate-")?;
    let latest_frame_calculix = load_latest_summary(project_dir, "frame-calculix-run-")?;
    let latest_beam_calculix = load_latest_summary(project_dir, "beam-calculix-run-")?;
    let latest_beam_analysis = load_latest_summary(project_dir, "beam-analysis-")?;
    let latest_beam_sizing = load_latest_summary(project_dir, "beam-size-")?;
    let latest_design_option_analysis =
        load_latest_summary(project_dir, "design-option-analysis-")?;
    let latest_import = load_latest_summary(project_dir, "import-stick-")?;
    let latest_run_summary = infer_latest_run_summary(
        &latest_validate,
        &latest_frame_calculix,
        &latest_beam_calculix,
        &latest_beam_analysis,
        &latest_beam_sizing,
        &latest_design_option_analysis,
        &latest_import,
    );
    let design_option_analysis_lookup =
        load_latest_design_option_analysis_lookup(project_dir, project)?;
    let coordination_report = structural_model
        .as_ref()
        .map(|model| {
            build_coordination_report_with_analysis(
                project,
                &draft,
                model,
                design_option_analysis_lookup.as_ref(),
            )
        })
        .transpose()?;
    let schemas_ready = project
        .base_model_brief
        .as_ref()
        .map(|brief| brief.readiness.ready_for_schemas)
        .unwrap_or(false)
        && has_schema_handoff_snapshot(project_dir)?;
    let design_schemes = if schemas_ready {
        coordination_report
            .as_ref()
            .map(|report| report.design_schemes.clone())
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let mut design_option_decisions = effective_design_option_decisions(project);
    sync_decision_analysis_evidence(
        &mut design_option_decisions,
        &design_schemes,
        latest_design_option_analysis
            .as_ref()
            .map(|artifact| artifact.run_id.as_str()),
    );

    Ok(WorkbenchProjectState {
        overview: WorkbenchProjectOverview {
            project_dir: project_dir.display().to_string(),
            name: project.name.clone(),
            building_type: project.intent.building_type.clone(),
            design_stage: project.intent.design_stage.clone(),
            span_m: project.requirements.span_m,
            height_m: project.requirements.height_m,
        },
        unit_profile: project.unit_profile.clone(),
        planning_draft: Some(core_planning_to_api(draft.clone())),
        active_system_family: Some(canonical_system_family_hint(
            &draft.system_brief.system_family_hint,
        )),
        analysis_readiness: Some(readiness.clone()),
        latest_run_summary,
        capability_diagnostics: readiness
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code.starts_with("unsupported."))
            .cloned()
            .collect(),
        scene: structural_model.as_ref().map(build_workbench_scene),
        design_schemes,
        coordination_report,
        latest_validate,
        latest_frame_calculix,
        latest_beam_calculix,
        latest_beam_analysis,
        latest_beam_sizing,
        latest_design_option_analysis,
        agent_state: project.agent_state.clone(),
        base_model_brief: project.base_model_brief.clone(),
        latest_import,
        design_option_decisions,
    })
}

fn build_coordination_report(
    project: &ProjectFile,
    draft: &CorePlanningDraft,
    model: &StructuralModel,
) -> Result<CoordinationReport> {
    build_coordination_report_with_analysis(project, draft, model, None)
}

fn build_coordination_report_with_analysis(
    project: &ProjectFile,
    draft: &CorePlanningDraft,
    model: &StructuralModel,
    analysis_lookup: Option<&DesignOptionAnalysisLookup>,
) -> Result<CoordinationReport> {
    let understanding = understand_structural_model(model);
    let groups = build_coordination_groups(draft, &understanding);
    let design_schemes = build_design_schemes(project, draft, &groups, model, analysis_lookup);
    let active_design_scheme_id = design_schemes.first().map(|scheme| scheme.id.clone());
    let mut diagnostics = Vec::new();
    if groups.is_empty() && !model.members.is_empty() {
        diagnostics.push(warning_diagnostic(
            "coordination.no_groups",
            "No structural coordination groups were recognised for the current model.",
        ));
    }
    if !groups.is_empty() && design_schemes.is_empty() {
        diagnostics.push(warning_diagnostic(
            "coordination.no_design_option_intents",
            "No DesignOptionIntent records are available yet. Ask the planning agent to propose only design-option intents justified by the Base Model, structural design judgement, and user-confirmed constraints.",
        ));
    }
    Ok(CoordinationReport {
        groups: groups.clone(),
        design_schemes,
        active_design_scheme_id,
        diagnostics,
    })
}

fn build_coordination_groups(
    draft: &CorePlanningDraft,
    understanding: &fraia_core::ModelUnderstandingReport,
) -> Vec<CoordinationGroup> {
    let mut groups = Vec::new();
    let member_groups = &understanding.member_groups;
    let mut by_role: BTreeMap<String, Vec<&fraia_core::MemberGroupUnderstanding>> = BTreeMap::new();
    for group in member_groups {
        by_role
            .entry(coordination_role_key(&group.role))
            .or_default()
            .push(group);
    }
    for (role, role_groups) in by_role {
        let same_size_preferred = role_groups.len() > 1;
        let rationale = if same_size_preferred {
            vec![format!(
                "{} member groups share an authored role; review whether they should coordinate family or size.",
                role_groups.len()
            )]
        } else {
            vec!["single engineering member group".into()]
        };
        groups.push(coordination_group_from_member_groups(
            draft,
            &format!("coord-role-{}", safe_id_fragment(&role)),
            &format!("{} group", human_role(&role)),
            &role,
            &role_groups,
            rationale,
            Vec::new(),
            same_size_preferred,
        ));
    }

    groups
}

fn coordination_role_key(role: &str) -> String {
    let trimmed = role.trim();
    if trimmed.is_empty() {
        "member".into()
    } else {
        trimmed.to_ascii_lowercase()
    }
}

fn coordination_group_from_member_groups(
    draft: &CorePlanningDraft,
    id: &str,
    label: &str,
    role: &str,
    groups: &[&fraia_core::MemberGroupUnderstanding],
    rationale: Vec<String>,
    buildability_notes: Vec<String>,
    same_size_preferred: bool,
) -> CoordinationGroup {
    let recommended = unique_strings(groups.iter().flat_map(|group| {
        group
            .recommended_section_families
            .iter()
            .map(|family| family.to_ascii_uppercase())
            .collect::<Vec<_>>()
    }));
    let preferences = coordination_group_preferences(draft, id);
    let allowed = preferences
        .allowed_section_families
        .unwrap_or_else(|| recommended.clone());
    CoordinationGroup {
        id: id.into(),
        label: label.into(),
        role: role.into(),
        member_group_ids: groups.iter().map(|group| group.id.clone()).collect(),
        member_ids: unique_strings(
            groups
                .iter()
                .flat_map(|group| group.member_ids.iter().cloned()),
        ),
        allowed_section_families: normalise_section_families(&allowed),
        recommended_section_families: recommended,
        section_selection_policy: preferences
            .section_selection_policy
            .unwrap_or_else(|| AGENT_JUSTIFIED_SECTION_SELECTION_POLICY.into()),
        same_size_preferred,
        rationale,
        buildability_notes,
    }
}

#[derive(Default)]
struct CoordinationGroupPreferences {
    allowed_section_families: Option<Vec<String>>,
    section_selection_policy: Option<String>,
}

fn coordination_group_preferences(
    draft: &CorePlanningDraft,
    group_id: &str,
) -> CoordinationGroupPreferences {
    let Some(value) = draft.system_parameters.get("coordinationGroups") else {
        return CoordinationGroupPreferences::default();
    };
    let Some(group) = value.get(group_id).and_then(Value::as_object) else {
        return CoordinationGroupPreferences::default();
    };
    CoordinationGroupPreferences {
        allowed_section_families: group
            .get("allowedSectionFamilies")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect()
            }),
        section_selection_policy: group
            .get("sectionSelectionPolicy")
            .and_then(Value::as_str)
            .map(str::to_owned),
    }
}

fn build_design_schemes(
    project: &ProjectFile,
    draft: &CorePlanningDraft,
    groups: &[CoordinationGroup],
    model: &StructuralModel,
    analysis_lookup: Option<&DesignOptionAnalysisLookup>,
) -> Vec<DesignScheme> {
    design_scheme_candidates(project, draft, model, groups)
        .iter()
        .map(|candidate| {
            let mut diagnostics = Vec::new();
            let group_choices: Vec<_> = groups
                .iter()
                .map(|group| {
                    let preferred_families = scheme_family_order(candidate, group);
                    let mut choice = design_scheme_choice(group, model, &preferred_families);
                    let option_analysis_lookup =
                        (design_option_lifecycle_status(&candidate.intent) == "active")
                            .then_some(analysis_lookup)
                            .flatten();
                    apply_candidate_analysis_to_choice(
                        candidate.intent.id.as_str(),
                        &mut choice,
                        option_analysis_lookup,
                    );
                    if !choice.unavailable_families.is_empty() {
                        diagnostics.push(warning_diagnostic(
                            "coordination.family_unavailable",
                            &format!(
                                "{} has no demo catalogue entries for {}.",
                                group.label,
                                choice.unavailable_families.join(", ")
                            ),
                        ));
                    }
                    choice
                })
                .collect();
            diagnostics.extend(grouping_quality_diagnostics(groups, model));
            diagnostics.extend(connection_buildability_diagnostics(&group_choices));
            diagnostics.extend(intent_validation_diagnostics(
                project,
                candidate,
                groups,
                model,
                &group_choices,
            ));
            diagnostics.extend(option_reviewer_justification_diagnostics(candidate));
            diagnostics.extend(holistic_design_option_diagnostics(
                project, candidate, model,
            ));
            let approximate_mass_kg = group_choices.iter().try_fold(0.0, |sum, choice| {
                choice.approximate_mass_kg.map(|mass| sum + mass)
            });
            let option_analysis_lookup = (design_option_lifecycle_status(&candidate.intent)
                == "active")
                .then_some(analysis_lookup)
                .flatten();
            let analysis_summary = option_analysis_lookup
                .and_then(|lookup| lookup.selected_results.get(&candidate.intent.id))
                .map(design_scheme_analysis_summary_from_result);
            let result_preview = option_analysis_lookup
                .and_then(|lookup| lookup.selected_previews.get(&candidate.intent.id))
                .cloned();
            DesignScheme {
                id: candidate.intent.id.clone(),
                label: candidate.intent.label.clone(),
                strategy: candidate.strategy.clone(),
                summary: candidate.summary.clone(),
                differentiation: candidate.differentiation.clone(),
                lifecycle_status: candidate.intent.lifecycle_status.clone(),
                superseded_by: candidate.intent.superseded_by.clone(),
                superseded_reason: candidate.intent.superseded_reason.clone(),
                revision_of: candidate.intent.revision_of.clone(),
                pros: candidate.pros.clone(),
                cons: candidate.cons.clone(),
                support_strategy: scheme_support_strategy(candidate, model),
                standardisation_strategy: Some(candidate.intent.standardisation_strategy.clone()),
                connection_strategy: Some(candidate.intent.connection_strategy.clone()),
                intent: Some(candidate.intent.clone()),
                scene: Some(build_design_scheme_scene(
                    project,
                    model,
                    groups,
                    &group_choices,
                    candidate,
                )),
                group_choices,
                approximate_mass_kg,
                analysis_summary,
                result_preview,
                diagnostics,
            }
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum SchemeSupportMode {
    Authored,
    Unspecified,
    PinnedRoller,
    PinnedPinned,
    FixedFixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum SchemeFamilyPolicy {
    CatalogFamilies,
    OpenSections,
    ClosedSections,
    StiffnessCapable,
    StandardisedFamilies,
}

#[derive(Debug, Clone)]
struct DesignSchemeCandidate {
    intent: DesignOptionIntent,
    strategy: String,
    summary: String,
    differentiation: String,
    pros: Vec<String>,
    cons: Vec<String>,
    support_mode: SchemeSupportMode,
    family_policy: SchemeFamilyPolicy,
    same_size_note: Option<&'static str>,
}

fn design_scheme_candidates(
    project: &ProjectFile,
    draft: &CorePlanningDraft,
    model: &StructuralModel,
    groups: &[CoordinationGroup],
) -> Vec<DesignSchemeCandidate> {
    if groups.is_empty() {
        return Vec::new();
    }
    let base_support_mode = (!model.supports.is_empty()).then_some(SchemeSupportMode::Authored);
    let support_locations_available =
        !design_scheme_support_location_node_ids(project, model).is_empty();
    let authored = authored_design_option_intents(draft)
        .into_iter()
        .filter_map(|intent| {
            candidate_from_authored_intent(intent, base_support_mode, support_locations_available)
        });
    let candidates: Vec<_> = authored.collect();
    let mut seen = BTreeSet::new();
    candidates
        .into_iter()
        .filter(|candidate| seen.insert(candidate.intent.id.clone()))
        .collect()
}

fn authored_design_option_intents(draft: &CorePlanningDraft) -> Vec<DesignOptionIntent> {
    draft
        .system_parameters
        .get("designOptionIntents")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|value| {
                    serde_json::from_value::<DesignOptionIntent>(value.clone()).ok()
                })
                .filter(|intent| !intent.id.trim().is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn design_option_lifecycle_status(intent: &DesignOptionIntent) -> &str {
    intent.lifecycle_status.as_deref().unwrap_or("active")
}

fn validate_design_option_lifecycle(intent: &DesignOptionIntent, name: &str) -> Result<()> {
    match design_option_lifecycle_status(intent) {
        "active" | "superseded" | "rejected" => Ok(()),
        other => Err(anyhow!(
            "DesignOptionIntent `{name}` has unsupported lifecycleStatus `{other}`"
        )),
    }
}

fn validate_design_option_intents(intents: &[DesignOptionIntent]) -> Result<()> {
    for intent in intents {
        let name = if intent.label.trim().is_empty() {
            intent.id.as_str()
        } else {
            intent.label.as_str()
        };
        for (field, value) in [
            ("id", intent.id.as_str()),
            ("label", intent.label.as_str()),
            ("hypothesis", intent.hypothesis.as_str()),
            ("explorationBand", intent.exploration_band.as_str()),
            (
                "standardisationStrategy",
                intent.standardisation_strategy.as_str(),
            ),
            ("connectionStrategy", intent.connection_strategy.as_str()),
            ("supportStrategy", intent.support_strategy.as_str()),
            ("sectionFamilyPolicy", intent.section_family_policy.as_str()),
            (
                "coordinationGroupPolicy",
                intent.coordination_group_policy.as_str(),
            ),
        ] {
            if value.trim().is_empty() {
                return Err(anyhow!("DesignOptionIntent `{name}` is missing {field}"));
            }
        }
        validate_design_option_lifecycle(intent, name)?;
        if !intent
            .objective_tags
            .iter()
            .any(|tag| !tag.trim().is_empty())
        {
            return Err(anyhow!(
                "DesignOptionIntent `{name}` must include at least one objective tag"
            ));
        }
        if !intent
            .assumptions
            .iter()
            .any(|assumption| !assumption.trim().is_empty())
        {
            return Err(anyhow!(
                "DesignOptionIntent `{name}` must state at least one varied assumption"
            ));
        }
        if !intent
            .provenance
            .iter()
            .any(|source| !source.trim().is_empty())
        {
            return Err(anyhow!(
                "DesignOptionIntent `{name}` must include provenance"
            ));
        }
        validate_design_option_wiki_grounded_provenance(intent, name)?;
    }
    Ok(())
}

fn validate_design_option_wiki_grounded_provenance(
    intent: &DesignOptionIntent,
    name: &str,
) -> Result<()> {
    let provenance_text = intent.provenance.join(" ").to_ascii_lowercase();
    if text_contains_any(
        &provenance_text,
        &["agent chose", "typical option", "common option"],
    ) {
        return Err(anyhow!(
            "DesignOptionIntent `{name}` provenance must justify support/restraint choice using retrieved knowledge and project evidence"
        ));
    }
    let evidence_text = format!(
        "{} {} {} {} {} {} {} {} {} {}",
        provenance_text,
        intent.hypothesis,
        intent.exploration_band,
        intent.objective_tags.join(" "),
        intent.assumptions.join(" "),
        intent.standardisation_strategy,
        intent.connection_strategy,
        intent.support_strategy,
        intent.section_family_policy,
        intent.coordination_group_policy
    )
    .to_ascii_lowercase();
    for (theme, needles) in [
        (
            "support/restraint choice",
            &["support", "restraint", "fixity", "foundation", "base"][..],
        ),
        (
            "load path or stability concept",
            &[
                "load path",
                "stability",
                "bracing",
                "frame action",
                "frame-action",
                "haunch",
                "force path",
                "transfer path",
                "stiffness",
                "serviceability",
            ][..],
        ),
        (
            "section-family policy",
            &["section", "family", "member"][..],
        ),
        (
            "coordination/standardisation policy",
            &[
                "coordination",
                "standardisation",
                "standardization",
                "group",
                "repeated",
                "shared",
                "matching",
            ][..],
        ),
        (
            "connection/detailing consequence",
            &["connection", "detail", "foundation", "gusset"][..],
        ),
    ] {
        if !text_contains_any(&evidence_text, needles) {
            return Err(anyhow!(
                "DesignOptionIntent `{name}` provenance must justify {theme} using retrieved knowledge and project evidence"
            ));
        }
    }
    Ok(())
}

fn candidate_from_authored_intent(
    mut intent: DesignOptionIntent,
    base_support_mode: Option<SchemeSupportMode>,
    support_locations_available: bool,
) -> Option<DesignSchemeCandidate> {
    let family_policy = family_policy_from_intent(&intent);
    intent.provenance.push(
        "agent-authored DesignOptionIntent from planning system_parameters.designOptionIntents"
            .into(),
    );
    let explicit_support_mode = support_mode_from_strategy(&intent.support_strategy);
    let support_mode = match explicit_support_mode.or(base_support_mode) {
        Some(SchemeSupportMode::Authored) if base_support_mode.is_some() => {
            SchemeSupportMode::Authored
        }
        Some(SchemeSupportMode::PinnedRoller) => return None,
        Some(mode @ (SchemeSupportMode::PinnedPinned | SchemeSupportMode::FixedFixed))
            if support_locations_available =>
        {
            mode
        }
        _ => return None,
    };
    let standardisation_text = intent.standardisation_strategy.to_ascii_lowercase();
    let same_size_note = text_contains_any(&standardisation_text, &["member-size", "same size"])
        .then_some("same-size preferred");
    Some(candidate_with_fields(
        intent,
        support_mode,
        family_policy,
        same_size_note,
    ))
}

fn candidate_with_fields(
    intent: DesignOptionIntent,
    support_mode: SchemeSupportMode,
    family_policy: SchemeFamilyPolicy,
    same_size_note: Option<&'static str>,
) -> DesignSchemeCandidate {
    let strategy = if intent.objective_tags.is_empty() {
        intent.id.clone()
    } else {
        intent.objective_tags.join(", ")
    };
    let summary = if intent.hypothesis.trim().is_empty() {
        intent.label.clone()
    } else {
        intent.hypothesis.clone()
    };
    let differentiation = intent.exploration_band.clone();
    DesignSchemeCandidate {
        intent,
        strategy,
        summary,
        differentiation,
        pros: Vec::new(),
        cons: Vec::new(),
        support_mode,
        family_policy,
        same_size_note,
    }
}

fn text_contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

fn support_mode_from_strategy(strategy: &str) -> Option<SchemeSupportMode> {
    let lower = strategy.to_ascii_lowercase();
    if lower.contains("fixed") {
        Some(SchemeSupportMode::FixedFixed)
    } else if lower.contains("roller") {
        Some(SchemeSupportMode::PinnedRoller)
    } else if lower.contains("pinned") {
        Some(SchemeSupportMode::PinnedPinned)
    } else if lower.contains("authored supportassignment") || lower.contains("authored supports") {
        Some(SchemeSupportMode::Authored)
    } else {
        None
    }
}

fn family_policy_from_intent(intent: &DesignOptionIntent) -> SchemeFamilyPolicy {
    let text = intent_text(intent);
    if text_contains_any(&text, &["closed", "rhs", "shs", "chs", "tube", "hollow"]) {
        SchemeFamilyPolicy::ClosedSections
    } else if text_contains_any(&text, &["open", "ub", "uc", "pfc", "channel"]) {
        SchemeFamilyPolicy::OpenSections
    } else if text_contains_any(&text, &["standard", "repeat", "same size"]) {
        SchemeFamilyPolicy::StandardisedFamilies
    } else if text_contains_any(&text, &["stiff", "service", "drift", "deflection"]) {
        SchemeFamilyPolicy::StiffnessCapable
    } else {
        SchemeFamilyPolicy::CatalogFamilies
    }
}

fn intent_text(intent: &DesignOptionIntent) -> String {
    format!(
        "{} {} {} {} {} {} {} {} {}",
        intent.id,
        intent.label,
        intent.hypothesis,
        intent.exploration_band,
        intent.objective_tags.join(" "),
        intent.standardisation_strategy,
        intent.connection_strategy,
        intent.section_family_policy,
        intent.assumptions.join(" "),
    )
    .to_ascii_lowercase()
}

fn build_design_scheme_scene(
    project: &ProjectFile,
    model: &StructuralModel,
    groups: &[CoordinationGroup],
    choices: &[DesignSchemeGroupChoice],
    candidate: &DesignSchemeCandidate,
) -> WorkbenchScene {
    let mut scheme_model = model.clone();
    if scheme_model.supports.is_empty() {
        scheme_model.supports = design_scheme_supports(project, model, candidate);
    }
    let mut scene = build_workbench_scene(&scheme_model);
    apply_scheme_member_labels(&mut scene, groups, choices, candidate);
    apply_support_group_labels(&mut scene);
    scene
}

fn apply_scheme_member_labels(
    scene: &mut WorkbenchScene,
    groups: &[CoordinationGroup],
    choices: &[DesignSchemeGroupChoice],
    candidate: &DesignSchemeCandidate,
) {
    let group_map: std::collections::BTreeMap<_, _> =
        groups.iter().map(|group| (&group.id, group)).collect();
    let group_labels: std::collections::BTreeMap<_, _> = groups
        .iter()
        .enumerate()
        .map(|(index, group)| (&group.id, format!("Group {}", index + 1)))
        .collect();
    let family_group_labels = scheme_group_labels(groups, candidate.family_policy);
    let family_group_member_counts = scheme_group_member_counts(groups, &family_group_labels);
    let size_group_labels = scheme_size_group_labels(groups, candidate);
    for choice in choices {
        let Some(group) = group_map.get(&choice.coordination_group_id) else {
            continue;
        };
        let group_label = group_labels
            .get(&choice.coordination_group_id)
            .cloned()
            .unwrap_or_else(|| group.label.clone());
        let note = candidate
            .same_size_note
            .or_else(|| group.same_size_preferred.then_some("same-size preferred"))
            .map(str::to_owned);
        for member_id in &group.member_ids {
            let Some(member) = scene
                .members
                .iter_mut()
                .find(|member| &member.id == member_id)
            else {
                continue;
            };
            member.allowed_section_families = choice.allowed_section_families.clone();
            member.coordination_group_id = Some(group.id.clone());
            member.coordination_group_label = Some(group_label.clone());
            let family_group_label = family_group_labels.get(group.id.as_str()).cloned();
            let family_group_member_count = family_group_label
                .as_ref()
                .and_then(|label| family_group_member_counts.get(label))
                .copied()
                .unwrap_or(0);
            member.family_group_label = family_group_label.clone();
            member.section_coordination = Some(scene_section_coordination(
                family_group_label.as_deref(),
                family_group_member_count,
            ));
            let size_group_label = size_group_labels.get(group.id.as_str()).cloned();
            member.size_group_label = size_group_label.clone();
            member.size_coordination = Some(scene_size_coordination(size_group_label.as_deref()));
            member.scheme_note = note.clone();
        }
    }
    apply_design_option_coordination_overrides(scene, candidate);
}

fn apply_design_option_coordination_overrides(
    scene: &mut WorkbenchScene,
    candidate: &DesignSchemeCandidate,
) {
    for override_rule in &candidate.intent.coordination_overrides {
        let Some(member) = scene
            .members
            .iter_mut()
            .find(|member| member.id == override_rule.member_id)
        else {
            continue;
        };
        if let Some(label) =
            normalise_family_group_label(override_rule.family_group_label.as_deref())
        {
            member.family_group_label = Some(label.clone());
            member.section_coordination = Some(scene_section_coordination(Some(&label), 2));
        }
        if let Some(label) =
            normalise_designation_group_label(override_rule.designation_group_label.as_deref())
        {
            member.size_group_label = Some(label.clone());
            member.size_coordination = Some(scene_size_coordination(Some(&label)));
        }
        if let Some(note) = override_rule
            .note
            .as_deref()
            .map(str::trim)
            .filter(|note| !note.is_empty())
        {
            member.scheme_note = Some(note.to_owned());
        }
    }
}

fn normalise_family_group_label(label: Option<&str>) -> Option<String> {
    normalise_numbered_group_label(label, "Family Group", "GF")
}

fn normalise_designation_group_label(label: Option<&str>) -> Option<String> {
    normalise_numbered_group_label(label, "Size Group", "GD")
}

fn normalise_numbered_group_label(
    label: Option<&str>,
    canonical: &str,
    compact: &str,
) -> Option<String> {
    let trimmed = label?.trim();
    if trimmed.is_empty() {
        return None;
    }
    let upper = trimmed.to_ascii_uppercase();
    if let Some(number) = upper.strip_prefix(compact).and_then(parse_group_number) {
        return Some(format!("{canonical} {number}"));
    }
    if let Some(number) = trimmed.strip_prefix(canonical).and_then(parse_group_number) {
        return Some(format!("{canonical} {number}"));
    }
    Some(trimmed.to_owned())
}

fn parse_group_number(value: &str) -> Option<usize> {
    let digits: String = value.chars().filter(|ch| ch.is_ascii_digit()).collect();
    digits.parse::<usize>().ok().filter(|number| *number > 0)
}

fn scheme_group_member_counts(
    groups: &[CoordinationGroup],
    group_labels: &std::collections::BTreeMap<&str, String>,
) -> std::collections::BTreeMap<String, usize> {
    let mut counts = std::collections::BTreeMap::new();
    for group in groups {
        let Some(label) = group_labels.get(group.id.as_str()) else {
            continue;
        };
        *counts.entry(label.clone()).or_insert(0) += group.member_ids.len();
    }
    counts
}

fn scene_section_coordination(
    family_group_label: Option<&str>,
    member_count: usize,
) -> SceneSectionCoordination {
    match family_group_label {
        Some(label) if member_count > 1 => SceneSectionCoordination {
            kind: "shared".into(),
            group_label: Some(label.to_owned()),
        },
        Some(_) => SceneSectionCoordination {
            kind: "independent".into(),
            group_label: None,
        },
        None => SceneSectionCoordination {
            kind: "unspecified".into(),
            group_label: None,
        },
    }
}

fn scene_size_coordination(size_group_label: Option<&str>) -> SceneSizeCoordination {
    match size_group_label {
        Some(label)
            if label.eq_ignore_ascii_case("Size independent")
                || label.eq_ignore_ascii_case("Unique") =>
        {
            SceneSizeCoordination {
                kind: "independent".into(),
                group_label: None,
            }
        }
        Some(label) => SceneSizeCoordination {
            kind: "shared".into(),
            group_label: Some(label.to_owned()),
        },
        None => SceneSizeCoordination {
            kind: "unspecified".into(),
            group_label: None,
        },
    }
}

fn scheme_group_labels(
    groups: &[CoordinationGroup],
    policy: SchemeFamilyPolicy,
) -> std::collections::BTreeMap<&str, String> {
    match policy {
        SchemeFamilyPolicy::ClosedSections | SchemeFamilyPolicy::StandardisedFamilies => groups
            .iter()
            .map(|group| (group.id.as_str(), "Family Group 1".to_owned()))
            .collect(),
        SchemeFamilyPolicy::CatalogFamilies
        | SchemeFamilyPolicy::OpenSections
        | SchemeFamilyPolicy::StiffnessCapable => {
            groups
                .iter()
                .fold(
                    (
                        std::collections::BTreeMap::<String, usize>::new(),
                        std::collections::BTreeMap::<&str, String>::new(),
                    ),
                    |(mut role_indexes, mut labels), group| {
                        let key = group.role.trim().to_ascii_lowercase();
                        let next_index = role_indexes.len() + 1;
                        let index = *role_indexes.entry(key).or_insert(next_index);
                        labels.insert(group.id.as_str(), format!("Family Group {index}"));
                        (role_indexes, labels)
                    },
                )
                .1
        }
    }
}

fn scheme_size_group_labels<'a>(
    groups: &'a [CoordinationGroup],
    candidate: &DesignSchemeCandidate,
) -> std::collections::BTreeMap<&'a str, String> {
    if candidate.family_policy == SchemeFamilyPolicy::StandardisedFamilies {
        return groups
            .iter()
            .map(|group| (group.id.as_str(), "Size Group 1".to_owned()))
            .collect();
    }
    let family_groups = scheme_group_labels(groups, candidate.family_policy);
    let mut family_indexes: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    groups
        .iter()
        .map(|group| {
            let label = if group.same_size_preferred {
                let family_label = family_groups
                    .get(group.id.as_str())
                    .cloned()
                    .unwrap_or_else(|| "Family Group 1".to_owned());
                let family_number = family_label.strip_prefix("Family Group ").unwrap_or("1");
                let next = family_indexes.entry(family_number.to_owned()).or_insert(0);
                *next += 1;
                format!("Size Group {next}")
            } else {
                "Unique".to_owned()
            };
            (group.id.as_str(), label)
        })
        .collect()
}

fn apply_support_group_labels(scene: &mut WorkbenchScene) {
    let mut signatures: Vec<(bool, bool, bool, bool, bool, bool)> = Vec::new();
    for support in &mut scene.supports {
        let signature = (
            support.ux, support.uy, support.uz, support.rx, support.ry, support.rz,
        );
        let index = signatures
            .iter()
            .position(|existing| *existing == signature)
            .unwrap_or_else(|| {
                signatures.push(signature);
                signatures.len() - 1
            });
        support.support_group_label = Some(format!("Support Group {}", index + 1));
    }
}

fn scheme_support_strategy(candidate: &DesignSchemeCandidate, model: &StructuralModel) -> String {
    if !model.supports.is_empty() {
        return "Uses authored Base Model SupportAssignment objects.".into();
    }
    match candidate.support_mode {
        SchemeSupportMode::Authored => "Uses authored Base Model SupportAssignment objects.",
        SchemeSupportMode::Unspecified => {
            "Support/restraint strategy is not explicit; confirm locations and fixity before solving."
        }
        SchemeSupportMode::PinnedRoller => {
            "Pinned/roller restraint assumption for explicitly authored support locations."
        }
        SchemeSupportMode::PinnedPinned => {
            "Pinned restraint assumption for explicitly authored support locations."
        }
        SchemeSupportMode::FixedFixed => {
            "Fixed restraint assumption for explicitly authored support locations."
        }
    }
    .into()
}

fn design_scheme_supports(
    project: &ProjectFile,
    model: &StructuralModel,
    candidate: &DesignSchemeCandidate,
) -> Vec<SupportAssignment> {
    let target_nodes = design_scheme_support_location_node_ids(project, model);
    if target_nodes.is_empty() {
        return Vec::new();
    }
    target_nodes
        .into_iter()
        .enumerate()
        .filter_map(|(index, target_node)| {
            let support_type = match candidate.support_mode {
                SchemeSupportMode::PinnedPinned => "pinned",
                SchemeSupportMode::PinnedRoller if index == 0 => "pinned",
                SchemeSupportMode::PinnedRoller => "roller",
                SchemeSupportMode::FixedFixed => "fixed",
                SchemeSupportMode::Authored | SchemeSupportMode::Unspecified => return None,
            };
            let (ux, uy, uz, rx, ry, rz) = support_dofs_for_type(support_type);
            Some(SupportAssignment {
                id: format!("scheme-support-{}-{}", candidate.intent.id, index + 1),
                target_node,
                ux,
                uy,
                uz,
                rx,
                ry,
                rz,
            })
        })
        .collect()
}

fn design_scheme_support_location_node_ids(
    project: &ProjectFile,
    model: &StructuralModel,
) -> Vec<String> {
    let model_node_ids: BTreeSet<_> = model.nodes.iter().map(|node| node.id.as_str()).collect();
    let mut seen = BTreeSet::new();
    project
        .base_model_brief
        .as_ref()
        .into_iter()
        .flat_map(|brief| brief.visual_intent.support_locations.iter())
        .filter_map(|support| {
            let target = support.target_node.trim();
            if target.is_empty()
                || !model_node_ids.contains(target)
                || !seen.insert(target.to_owned())
            {
                None
            } else {
                Some(target.to_owned())
            }
        })
        .collect()
}

fn scheme_family_order(
    candidate: &DesignSchemeCandidate,
    group: &CoordinationGroup,
) -> Vec<String> {
    let allowed = normalise_section_families(&group.allowed_section_families);
    let mut requested = families_from_design_intent(&candidate.intent, candidate.family_policy);
    requested.extend(group.recommended_section_families.clone());
    requested.extend(allowed.clone());
    let requested = normalise_section_families(&requested);
    let filtered = filter_families_for_policy(&requested, candidate.family_policy);
    let constrained: Vec<_> = filtered
        .into_iter()
        .filter(|family| allowed.is_empty() || allowed.iter().any(|allowed| allowed == family))
        .collect();
    if !constrained.is_empty() {
        return constrained;
    }
    let fallback = filter_families_for_policy(&allowed, candidate.family_policy);
    if fallback.is_empty() {
        allowed
    } else {
        fallback
    }
}

fn families_from_design_intent(
    intent: &DesignOptionIntent,
    policy: SchemeFamilyPolicy,
) -> Vec<String> {
    let text = format!(
        "{} {} {} {} {} {} {}",
        intent.section_family_policy,
        intent.standardisation_strategy,
        intent.connection_strategy,
        intent.coordination_group_policy,
        intent.hypothesis,
        intent.objective_tags.join(" "),
        intent.assumptions.join(" "),
    );
    let explicit = extract_section_family_mentions(&text);
    if !explicit.is_empty() {
        return explicit;
    }
    filter_families_for_policy(&available_section_families(), policy)
}

fn extract_section_family_mentions(text: &str) -> Vec<String> {
    let available = available_section_families();
    let lower = text.to_ascii_lowercase();
    unique_strings(available.into_iter().filter(|family| {
        let token = family.to_ascii_lowercase();
        lower
            .split(|ch: char| !ch.is_ascii_alphanumeric())
            .any(|part| part == token)
    }))
}

fn filter_families_for_policy(families: &[String], policy: SchemeFamilyPolicy) -> Vec<String> {
    unique_strings(families.iter().filter_map(|family| {
        let keep = match policy {
            SchemeFamilyPolicy::ClosedSections => is_closed_section_family(family),
            SchemeFamilyPolicy::OpenSections => is_open_section_family(family),
            SchemeFamilyPolicy::CatalogFamilies
            | SchemeFamilyPolicy::StiffnessCapable
            | SchemeFamilyPolicy::StandardisedFamilies => true,
        };
        keep.then_some(family.clone())
    }))
}

fn is_closed_section_family(family: &str) -> bool {
    let upper = family.trim().to_ascii_uppercase();
    upper.ends_with("HS") || upper == "CHS"
}

fn is_open_section_family(family: &str) -> bool {
    !is_closed_section_family(family)
}

fn design_scheme_choice(
    group: &CoordinationGroup,
    model: &StructuralModel,
    family_order: &[String],
) -> DesignSchemeGroupChoice {
    let families = normalise_section_families(family_order);
    let mut candidate_sections = Vec::new();
    let mut unavailable = Vec::new();
    for family in &families {
        let sections = sections_for_family(family);
        if sections.is_empty() {
            unavailable.push(family.clone());
        }
        candidate_sections.extend(sections.into_iter().map(|section| section.id));
    }
    let selected_section_id = candidate_sections.first().cloned();
    let group_length_m = coordination_group_length_m(model, group);
    let catalog = section_catalog();
    let approximate_mass_kg = selected_section_id.as_deref().and_then(|section_id| {
        catalog
            .iter()
            .find(|section| section.id == section_id)
            .map(|section| section.mass_kg_per_m * group_length_m)
    });
    let selected_mass_kg = approximate_mass_kg.unwrap_or(0.0);
    let candidate_section_details = candidate_sections
        .iter()
        .filter_map(|section_id| {
            let section = catalog.iter().find(|section| &section.id == section_id)?;
            let approximate_mass_kg = section.mass_kg_per_m * group_length_m;
            Some(DesignSchemeSectionCandidate {
                section_id: section.id.clone(),
                family: section_family(&section.id).unwrap_or("unknown").into(),
                mass_kg_per_m: section.mass_kg_per_m,
                approximate_mass_kg,
                relative_to_selected_kg: approximate_mass_kg - selected_mass_kg,
                analysis_status: None,
                analysis_run_id: None,
                passes_preliminary_check: None,
                max_utilization: None,
                max_stress_mpa: None,
                max_moment_knm: None,
                max_shear_kn: None,
                max_deflection_mm: None,
                max_drift_mm: None,
                max_reaction_kn: None,
                governing_member_id: None,
                diagnostic: None,
            })
        })
        .collect();
    let check_status = if !families.is_empty() {
        "family_constraints".into()
    } else {
        "unavailable".into()
    };
    let mut notes = Vec::new();
    if group.same_size_preferred {
        notes.push("same size preferred within this group".into());
    }
    notes.extend(group.buildability_notes.clone());
    DesignSchemeGroupChoice {
        coordination_group_id: group.id.clone(),
        allowed_section_families: families,
        candidate_section_ids: candidate_sections,
        candidate_sections: candidate_section_details,
        unavailable_families: unavailable,
        selected_section_id,
        approximate_mass_kg,
        check_status,
        notes,
    }
}

fn apply_candidate_analysis_to_choice(
    option_id: &str,
    choice: &mut DesignSchemeGroupChoice,
    analysis_lookup: Option<&DesignOptionAnalysisLookup>,
) {
    let Some(lookup) = analysis_lookup else {
        return;
    };
    for candidate in &mut choice.candidate_sections {
        let Some(result) = lookup.candidate(
            option_id,
            &choice.coordination_group_id,
            &candidate.section_id,
        ) else {
            continue;
        };
        candidate.analysis_status = Some(result.status.clone());
        candidate.analysis_run_id = lookup
            .candidate_run_id(
                option_id,
                &choice.coordination_group_id,
                &candidate.section_id,
            )
            .map(str::to_owned);
        candidate.passes_preliminary_check = result.passed;
        candidate.max_utilization = result.max_utilization;
        candidate.max_stress_mpa = result.max_stress_mpa;
        candidate.max_moment_knm = result.max_moment_knm;
        candidate.max_shear_kn = result.max_shear_kn;
        candidate.max_deflection_mm = result.max_deflection_mm;
        candidate.max_drift_mm = result.max_drift_mm;
        candidate.max_reaction_kn = result.max_reaction_kn;
        candidate.governing_member_id = result.governing_member_id.clone();
        candidate.diagnostic = result.diagnostic.clone();
    }
    if choice
        .candidate_sections
        .iter()
        .any(|candidate| candidate.analysis_status.is_some())
    {
        if let Some((selected_section_id, selected_mass_kg)) = choice
            .candidate_sections
            .iter()
            .filter(|candidate| candidate.passes_preliminary_check == Some(true))
            .min_by(|a, b| {
                a.approximate_mass_kg
                    .total_cmp(&b.approximate_mass_kg)
                    .then_with(|| a.section_id.cmp(&b.section_id))
            })
            .or_else(|| {
                choice
                    .candidate_sections
                    .iter()
                    .filter(|candidate| candidate.passes_preliminary_check == Some(false))
                    .min_by(|a, b| {
                        a.approximate_mass_kg
                            .total_cmp(&b.approximate_mass_kg)
                            .then_with(|| a.section_id.cmp(&b.section_id))
                    })
            })
            .map(|candidate| (candidate.section_id.clone(), candidate.approximate_mass_kg))
        {
            choice.selected_section_id = Some(selected_section_id);
            choice.approximate_mass_kg = Some(selected_mass_kg);
            for candidate in &mut choice.candidate_sections {
                candidate.relative_to_selected_kg =
                    candidate.approximate_mass_kg - selected_mass_kg;
            }
        }
        choice.check_status = "preliminary_analysis_available".into();
    }
}

fn design_scheme_analysis_summary_from_result(
    result: &DesignOptionCandidateAnalysisResult,
) -> fraia_app_api::DesignSchemeAnalysisSummary {
    fraia_app_api::DesignSchemeAnalysisSummary {
        status: result.status.clone(),
        max_utilization: result.max_utilization,
        max_moment_knm: result.max_moment_knm,
        max_shear_kn: result.max_shear_kn,
        max_stress_mpa: result.max_stress_mpa.unwrap_or(0.0),
        max_deflection_mm: result.max_deflection_mm.unwrap_or(0.0),
        max_drift_mm: result.max_drift_mm,
        max_reaction_kn: result.max_reaction_kn,
        governing_member_id: result.governing_member_id.clone(),
        deflected_shape_scale: result
            .max_deflection_mm
            .map(|value| (value / 25.0).clamp(0.25, 3.0))
            .unwrap_or(1.0),
    }
}

fn sections_for_family(family: &str) -> Vec<fraia_core::Section> {
    let family = family.trim().to_ascii_uppercase();
    let mut sections: Vec<_> = section_catalog()
        .into_iter()
        .filter(|section| section_family(&section.id) == Some(family.as_str()))
        .collect();
    sections.sort_by(|a, b| a.mass_kg_per_m.total_cmp(&b.mass_kg_per_m));
    sections
}

fn coordination_group_length_m(model: &StructuralModel, group: &CoordinationGroup) -> f64 {
    group
        .member_ids
        .iter()
        .filter_map(|member_id| model.members.iter().find(|member| &member.id == member_id))
        .filter_map(|member| {
            let start = model.node_by_id(&member.start_node)?;
            let end = model.node_by_id(&member.end_node)?;
            Some(
                ((end.x - start.x).powi(2) + (end.y - start.y).powi(2) + (end.z - start.z).powi(2))
                    .sqrt(),
            )
        })
        .sum()
}

fn holistic_design_option_diagnostics(
    project: &ProjectFile,
    candidate: &DesignSchemeCandidate,
    model: &StructuralModel,
) -> Vec<WorkbenchDiagnostic> {
    let mut diagnostics = Vec::new();
    let intent = &candidate.intent;
    let hypothesis_text = format!(
        "{} {} {} {}",
        intent.hypothesis,
        intent.objective_tags.join(" "),
        intent.assumptions.join(" "),
        intent.provenance.join(" ")
    )
    .to_ascii_lowercase();
    let support_text = intent.support_strategy.to_ascii_lowercase();
    let has_authored_supports = !model.supports.is_empty();
    let support_locations = design_scheme_support_location_node_ids(project, model);
    if matches!(
        candidate.support_mode,
        SchemeSupportMode::PinnedPinned
            | SchemeSupportMode::PinnedRoller
            | SchemeSupportMode::FixedFixed
    ) && !has_authored_supports
    {
        let support_is_the_point = text_contains_any(
            &hypothesis_text,
            &[
                "support",
                "restraint",
                "fixity",
                "fixed",
                "roller",
                "pinned",
                "foundation",
                "base",
                "stability",
                "load path",
                "sensitivity",
            ],
        );
        let support_strategy_is_explained = text_contains_any(
            &support_text,
            &[
                "because",
                "rationale",
                "sensitivity",
                "baseline",
                "compare",
                "confirmed",
                "foundation",
                "load path",
                "stability",
            ],
        );
        if !support_is_the_point || !support_strategy_is_explained {
            diagnostics.push(warning_diagnostic(
                "design_option.support_rationale_missing",
                "This option changes support/restraint assumptions, but the option-level hypothesis/provenance does not clearly justify why that support fixity belongs to the comparison. Treat the support choice as unresolved until the agent explains whether it is baseline inheritance or a sensitivity case.",
            ));
        }
    }
    if candidate.support_mode == SchemeSupportMode::FixedFixed
        && !has_authored_supports
        && !text_contains_any(
            &format!(
                "{support_text} {}",
                intent.assumptions.join(" ").to_ascii_lowercase()
            ),
            &[
                "foundation",
                "base fixity",
                "fixity",
                "sensitivity",
                "because",
                "rationale",
            ],
        )
    {
        diagnostics.push(warning_diagnostic(
            "design_option.fixed_support_holistic_review",
            "Fixed supports materially change moments, reactions, and connection/foundation demand. This option should not use fixed supports unless base fixity is part of the stated design-option hypothesis or an explicitly labelled sensitivity case.",
        ));
    }
    if candidate.support_mode == SchemeSupportMode::PinnedRoller
        && !has_authored_supports
        && !text_contains_any(
            &format!(
                "{support_text} {}",
                intent.assumptions.join(" ").to_ascii_lowercase()
            ),
            &[
                "because",
                "rationale",
                "sensitivity",
                "thermal",
                "movement",
                "lateral restraint",
                "horizontal release",
            ],
        )
    {
        diagnostics.push(warning_diagnostic(
            "design_option.roller_support_holistic_review",
            "Pinned/roller supports are a support-boundary assumption, not a generic bracing/detailing choice. Confirm why one support releases horizontal restraint before comparing this option.",
        ));
    }
    if support_locations.len() < 2 && !has_authored_supports {
        diagnostics.push(warning_diagnostic(
            "design_option.support_locations_holistic_review",
            "This option has fewer than two confirmed support-location nodes. Review the support layout before treating system response or steel comparisons as meaningful.",
        ));
    }
    diagnostics
}

fn option_reviewer_justification_diagnostics(
    candidate: &DesignSchemeCandidate,
) -> Vec<WorkbenchDiagnostic> {
    let mut diagnostics = Vec::new();
    let intent = &candidate.intent;
    let option_text = format!(
        "{} {} {} {} {} {} {} {}",
        intent.id,
        intent.label,
        intent.hypothesis,
        intent.objective_tags.join(" "),
        intent.standardisation_strategy,
        intent.connection_strategy,
        intent.support_strategy,
        intent.section_family_policy,
    )
    .to_ascii_lowercase();
    let provenance_text = intent.provenance.join(" ").to_ascii_lowercase();
    if intent.provenance.len() < 3 {
        diagnostics.push(warning_diagnostic_with_detail(
            "design_option.justification_review_compressed",
            "The option reviewer sees compressed provenance. Ask the proposing agent for separate structural justifications before treating this option as well explained.",
            "Reviewer feedback: provenance should separately explain support/restraint, load path or stability, section family, coordination/standardisation, and connection/detailing decisions. Research queue topic: design-option justification rubric and evidence format.",
        ));
    }
    for gap in option_reviewer_knowledge_gaps(&option_text, &provenance_text) {
        diagnostics.push(warning_diagnostic_with_detail(
            "design_option.knowledge_gap_candidate",
            "The option reviewer found a design-option topic that is not clearly justified by the agent's engineering provenance.",
            &gap,
        ));
    }
    diagnostics
}

fn option_reviewer_knowledge_gaps(option_text: &str, provenance_text: &str) -> Vec<String> {
    let mut gaps = Vec::new();
    for (topic, trigger_terms, evidence_terms, research_topic) in [
        (
            "haunch/local stiffening",
            &["haunch", "local stiffen", "stiffening"][..],
            &["haunch", "stiffen", "knee", "eaves", "local"][..],
            "haunch stiffening behaviour, load path effects, and connection/detailing consequences",
        ),
        (
            "bracing/stability system",
            &["brace", "bracing", "sway", "lateral stability"][..],
            &["brace", "bracing", "sway", "lateral"][..],
            "bracing schemes, stability load paths, and brace connection assumptions",
        ),
        (
            "fixed-base restraint",
            &["fixed", "fixity", "fixed base"][..],
            &[
                "fixed",
                "fixity",
                "foundation",
                "base",
                "moment",
                "sensitivity",
            ][..],
            "fixed-base assumptions, foundation restraint, moment redistribution, and sensitivity cases",
        ),
        (
            "pinned/roller restraint",
            &["roller", "horizontal release"][..],
            &[
                "roller",
                "horizontal release",
                "movement",
                "thermal",
                "lateral restraint",
            ][..],
            "pinned/roller idealisation, horizontal release rationale, and stability implications",
        ),
        (
            "closed-section family choice",
            &["closed", "rhs", "shs", "chs", "tube", "hollow"][..],
            &[
                "closed",
                "rhs",
                "shs",
                "chs",
                "tube",
                "hollow",
                "torsion",
                "connection",
            ][..],
            "closed-section family tradeoffs, member behaviour, fabrication, and connection detailing",
        ),
        (
            "standardised member grouping",
            &[
                "standardise",
                "standardize",
                "standardisation",
                "standardization",
                "fewest member",
                "member-size",
            ][..],
            &[
                "standardise",
                "standardize",
                "standardisation",
                "standardization",
                "coordination",
                "group",
                "fabrication",
                "procurement",
            ][..],
            "member grouping, section standardisation, procurement, and fabrication tradeoffs",
        ),
    ] {
        if text_contains_any(option_text, trigger_terms)
            && !text_contains_any(provenance_text, evidence_terms)
        {
            gaps.push(format!(
                "Reviewer feedback: `{topic}` appears in the option, but the provenance does not show enough engineering evidence for that decision. Research queue topic: {research_topic}."
            ));
        }
    }
    gaps
}

fn intent_validation_diagnostics(
    project: &ProjectFile,
    candidate: &DesignSchemeCandidate,
    groups: &[CoordinationGroup],
    model: &StructuralModel,
    choices: &[DesignSchemeGroupChoice],
) -> Vec<WorkbenchDiagnostic> {
    let mut diagnostics = Vec::new();
    let intent_text = intent_text(&candidate.intent);
    if text_contains_any(
        &intent_text,
        &[
            "fewest member",
            "member-size",
            "member size",
            "same size",
            "procurement",
        ],
    ) && groups
        .iter()
        .all(|group| group.member_ids.len() <= 1 && !group.same_size_preferred)
    {
        diagnostics.push(warning_diagnostic(
            "design_option.intent_weak_grouping",
            "This intent wants few member sizes, but the current coordination groups do not yet justify shared sizing. Review grouping before treating the option as meaningful.",
        ));
    }
    if text_contains_any(
        &intent_text,
        &[
            "fewest connection",
            "connection famil",
            "detail-family",
            "detailing",
            "connection_simplicity",
        ],
    ) {
        let unique_families: BTreeSet<_> = choices
            .iter()
            .flat_map(|choice| choice.allowed_section_families.iter().cloned())
            .collect();
        if unique_families.len() > 3 {
            diagnostics.push(warning_diagnostic(
                "design_option.connection_variation",
                "This intent wants fewer connection families, but the realised family shortlist is still broad. Narrow section-family assumptions before using it as a detailing-simple option.",
            ));
        }
    }
    let has_support_locations = !model.supports.is_empty()
        || !design_scheme_support_location_node_ids(project, model).is_empty();
    if matches!(
        candidate.support_mode,
        SchemeSupportMode::PinnedPinned
            | SchemeSupportMode::PinnedRoller
            | SchemeSupportMode::FixedFixed
    ) && !has_support_locations
    {
        diagnostics.push(warning_diagnostic(
            "design_option.restraint_unsupported",
            "This intent varies support/restraint assumptions, but no authored support locations are available. Confirm support locations before treating the option as realizable.",
        ));
    }
    if !has_support_locations && candidate.support_mode == SchemeSupportMode::Unspecified {
        diagnostics.push(warning_diagnostic(
            "design_option.support_strategy_unresolved",
            "This intent does not state a realizable support/restraint strategy. Ask the agent to justify support assumptions or leave support fixity as an explicit design-option variable before solving.",
        ));
    }
    if model.supports.is_empty() && candidate.support_mode == SchemeSupportMode::FixedFixed {
        diagnostics.push(warning_diagnostic(
            "design_option.fixed_base_review",
            "This stiffness option assumes fixed restraint as a sensitivity case. Confirm that the real support/foundation condition can provide that restraint before treating it as viable.",
        ));
    }
    diagnostics
}

fn connection_buildability_diagnostics(
    choices: &[DesignSchemeGroupChoice],
) -> Vec<WorkbenchDiagnostic> {
    let families: Vec<_> = choices
        .iter()
        .filter_map(|choice| choice.selected_section_id.as_deref())
        .filter_map(section_family)
        .collect();
    let has_chs = families.iter().any(|family| *family == "CHS");
    let has_open = families
        .iter()
        .any(|family| matches!(*family, "UB" | "UC" | "PFC"));
    if has_chs && has_open {
        vec![warning_diagnostic(
            "coordination.connection_review",
            "This design option mixes CHS and open sections; connection detailing should be reviewed before treating it as buildable.",
        )]
    } else {
        Vec::new()
    }
}

fn grouping_quality_diagnostics(
    groups: &[CoordinationGroup],
    model: &StructuralModel,
) -> Vec<WorkbenchDiagnostic> {
    if model.members.len() < 2 || groups.is_empty() {
        return Vec::new();
    }
    let grouped_member_count: usize = groups.iter().map(|group| group.member_ids.len()).sum();
    let has_repeated_group = groups.iter().any(|group| group.member_ids.len() > 1);
    if grouped_member_count >= model.members.len() && !has_repeated_group {
        vec![warning_diagnostic(
            "coordination.weak_grouping",
            "This design option treats every member as its own coordination group; review whether repeated roles should share family or size intent before sizing.",
        )]
    } else {
        Vec::new()
    }
}

fn safe_id_fragment(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

fn human_role(role: &str) -> String {
    let trimmed = role.trim();
    if trimmed.is_empty() {
        "Member".into()
    } else {
        let mut chars = trimmed.chars();
        match chars.next() {
            Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
            None => "Member".into(),
        }
    }
}

fn unique_strings(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    values
        .into_iter()
        .filter_map(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return None;
            }
            let normalised = trimmed.to_owned();
            seen.insert(normalised.clone()).then_some(normalised)
        })
        .collect()
}

fn normalise_section_families(values: &[String]) -> Vec<String> {
    let recognised = available_section_families();
    unique_strings(values.iter().filter_map(|value| {
        let upper = value.trim().to_ascii_uppercase();
        recognised
            .iter()
            .any(|family| family == &upper)
            .then_some(upper)
    }))
}

fn available_section_families() -> Vec<String> {
    let mut families = BTreeSet::new();
    for section in section_catalog() {
        if let Some(family) = section_family(&section.id) {
            families.insert(family.to_owned());
        }
    }
    families.into_iter().collect()
}

fn build_workbench_scene(model: &fraia_core::StructuralModel) -> WorkbenchScene {
    let min_x = model
        .nodes
        .iter()
        .map(|node| node.x)
        .fold(f64::INFINITY, f64::min);
    let max_x = model
        .nodes
        .iter()
        .map(|node| node.x)
        .fold(f64::NEG_INFINITY, f64::max);
    let min_y = model
        .nodes
        .iter()
        .map(|node| node.y)
        .fold(f64::INFINITY, f64::min);
    let max_y = model
        .nodes
        .iter()
        .map(|node| node.y)
        .fold(f64::NEG_INFINITY, f64::max);

    let bounds = if model.nodes.is_empty() {
        SceneBounds {
            min_x: 0.0,
            max_x: 1.0,
            min_y: 0.0,
            max_y: 1.0,
        }
    } else {
        SceneBounds {
            min_x,
            max_x,
            min_y,
            max_y,
        }
    };

    let nodes = model
        .nodes
        .iter()
        .map(|node| SceneNode {
            id: node.id.clone(),
            x: node.x,
            y: node.y,
            z: node.z,
        })
        .collect();
    let members = model
        .members
        .iter()
        .map(|member| SceneMember {
            id: member.id.clone(),
            start_node: member.start_node.clone(),
            end_node: member.end_node.clone(),
            role: member.role.clone(),
            allowed_section_families: Vec::new(),
            coordination_group_id: None,
            coordination_group_label: None,
            family_group_label: None,
            section_coordination: None,
            size_group_label: None,
            size_coordination: None,
            scheme_note: None,
            semantic_tags: member.semantic_tags.clone(),
            section_id: member.section_id.clone(),
            material_id: member.material_id.clone(),
        })
        .collect();
    let plates = model
        .plates
        .iter()
        .map(|plate| ScenePlate {
            id: plate.id.clone(),
            boundary_nodes: plate.boundary_nodes.clone(),
            role: plate.role.clone(),
            semantic_tags: plate.semantic_tags.clone(),
            thickness_m: plate.thickness_m,
            material_id: plate.material_id.clone(),
        })
        .collect();
    let supports = model
        .supports
        .iter()
        .map(|support| SceneSupport {
            id: support.id.clone(),
            target_node: support.target_node.clone(),
            ux: support.ux,
            uy: support.uy,
            uz: support.uz,
            rx: support.rx,
            ry: support.ry,
            rz: support.rz,
            support_group_label: None,
        })
        .collect();
    let loads = model
        .loads
        .iter()
        .map(|load| {
            let (x, y, target_label) = match &load.target {
                fraia_core::AssignmentTargetRef::Node(node_id) => model
                    .node_by_id(node_id)
                    .map(|node| (node.x, node.y, format!("node {node_id}")))
                    .unwrap_or((0.0, 0.0, format!("node {node_id}"))),
                fraia_core::AssignmentTargetRef::Member(member_id) => model
                    .members
                    .iter()
                    .find(|member| member.id == *member_id)
                    .and_then(|member| {
                        let start = model.node_by_id(&member.start_node)?;
                        let end = model.node_by_id(&member.end_node)?;
                        Some((
                            0.5 * (start.x + end.x),
                            0.5 * (start.y + end.y),
                            format!("member {member_id}"),
                        ))
                    })
                    .unwrap_or((0.0, 0.0, format!("member {member_id}"))),
                fraia_core::AssignmentTargetRef::Plate(plate_id) => model
                    .plates
                    .iter()
                    .find(|plate| plate.id == *plate_id)
                    .map(|plate| {
                        let points: Vec<_> = plate
                            .boundary_nodes
                            .iter()
                            .filter_map(|node_id| model.node_by_id(node_id))
                            .collect();
                        let count = points.len().max(1) as f64;
                        let x = points.iter().map(|node| node.x).sum::<f64>() / count;
                        let y = points.iter().map(|node| node.y).sum::<f64>() / count;
                        (x, y, format!("plate {plate_id}"))
                    })
                    .unwrap_or((0.0, 0.0, format!("plate {plate_id}"))),
            };

            SceneLoad {
                id: load.id.clone(),
                target_label,
                kind: load.kind.as_str().into(),
                magnitude: load.magnitude,
                direction_x: load.direction.x,
                direction_y: load.direction.y,
                direction_z: load.direction.z,
                x,
                y,
            }
        })
        .collect();
    let releases = model
        .releases
        .iter()
        .map(|release| SceneRelease {
            id: release.id.clone(),
            member_id: release.target.member_id.clone(),
            end: match release.target.end {
                fraia_core::MemberEnd::Start => "start".into(),
                fraia_core::MemberEnd::End => "end".into(),
            },
            ux: release.ux,
            uy: release.uy,
            uz: release.uz,
            rx: release.rx,
            ry: release.ry,
            rz: release.rz,
        })
        .collect();

    WorkbenchScene {
        bounds,
        nodes,
        members,
        plates,
        supports,
        loads,
        releases,
    }
}

fn load_latest_summary(project_dir: &Path, prefix: &str) -> Result<Option<SummaryArtifactRef>> {
    let runs_dir = project_dir.join("runs");
    if !runs_dir.exists() {
        return Ok(None);
    }
    let latest = fs::read_dir(&runs_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_type()
                .map(|file_type| file_type.is_dir())
                .unwrap_or(false)
                && entry.file_name().to_string_lossy().starts_with(prefix)
                && entry.path().join("summary.md").exists()
        })
        .max_by_key(|entry| entry.file_name());
    let Some(latest) = latest else {
        return Ok(None);
    };
    let run_id = latest.file_name().to_string_lossy().to_string();
    let summary_md = fs::read_to_string(latest.path().join("summary.md")).with_context(|| {
        format!(
            "failed to read summary for latest run {}",
            latest.path().display()
        )
    })?;
    Ok(Some(SummaryArtifactRef { run_id, summary_md }))
}

fn raw_design_option_analysis_run_dir(project_dir: &Path, run_id: Option<&str>) -> Result<PathBuf> {
    let runs_dir = project_dir.join("runs");
    let Some(run_id) = run_id.filter(|id| !id.trim().is_empty()) else {
        let latest = fs::read_dir(&runs_dir)
            .with_context(|| format!("failed to read runs directory {}", runs_dir.display()))?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .file_type()
                    .map(|file_type| file_type.is_dir())
                    .unwrap_or(false)
                    && entry
                        .file_name()
                        .to_string_lossy()
                        .starts_with("design-option-analysis-")
                    && entry.path().join("solver-results.json").exists()
            })
            .max_by_key(|entry| entry.file_name())
            .ok_or_else(|| anyhow!("no design-option analysis run is available"))?;
        return Ok(latest.path());
    };
    let safe_run_id = run_id.trim();
    if safe_run_id.contains('/') || safe_run_id.contains('\\') || safe_run_id.contains("..") {
        return Err(anyhow!(
            "invalid design-option analysis run id `{safe_run_id}`"
        ));
    }
    let run_dir = runs_dir.join(safe_run_id);
    if !run_dir.join("solver-results.json").exists() {
        return Err(anyhow!(
            "design-option analysis run `{safe_run_id}` does not contain solver-results.json"
        ));
    }
    Ok(run_dir)
}

fn load_raw_design_option_analysis(run_dir: &Path) -> Result<Value> {
    let run_id = run_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("design-option-analysis")
        .to_owned();
    let run_manifest: Value = read_json_value_if_exists(&run_dir.join("run.json"))?
        .unwrap_or_else(|| json!({ "runId": run_id.clone() }));
    let comparison: Value = read_json_value_if_exists(&run_dir.join("comparison.json"))?
        .unwrap_or_else(|| json!({ "runId": run_id.clone(), "optionResults": [] }));
    let candidate_inputs: Vec<Value> =
        read_json_value_if_exists(&run_dir.join("candidate-inputs.json"))?
            .and_then(|value| value.as_array().cloned())
            .unwrap_or_default();
    let mut input_lookup = BTreeMap::new();
    for input in candidate_inputs {
        if let (Some(option_id), Some(group_id), Some(section_id)) = (
            json_string_field(&input, &["optionId", "option_id"]),
            json_string_field(&input, &["coordinationGroupId", "coordination_group_id"]),
            json_string_field(&input, &["sectionId", "section_id"]),
        ) {
            input_lookup.insert(
                DesignOptionAnalysisLookup::candidate_key(&option_id, &group_id, &section_id),
                input,
            );
        }
    }

    let mut solver_results: Vec<Value> =
        read_json_value_if_exists(&run_dir.join("solver-results.json"))?
            .and_then(|value| value.as_array().cloned())
            .unwrap_or_default();
    for result in &mut solver_results {
        enrich_raw_solver_result(run_dir, result, &input_lookup)?;
    }

    let diagnostics =
        read_json_value_if_exists(&run_dir.join("diagnostics.json"))?.unwrap_or_else(|| json!([]));
    let summary_md = fs::read_to_string(run_dir.join("summary.md")).unwrap_or_default();
    Ok(json!({
        "runId": run_id,
        "runDir": run_dir.display().to_string(),
        "manifest": run_manifest,
        "comparison": comparison,
        "summaryMd": summary_md,
        "solverResults": solver_results,
        "diagnostics": diagnostics,
    }))
}

fn read_json_value_if_exists(path: &Path) -> Result<Option<Value>> {
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read JSON artefact {}", path.display()))?;
    Ok(Some(serde_json::from_str(&text).with_context(|| {
        format!("failed to parse JSON artefact {}", path.display())
    })?))
}

fn json_string_field(value: &Value, names: &[&str]) -> Option<String> {
    names
        .iter()
        .find_map(|name| value.get(*name).and_then(Value::as_str))
        .map(str::to_owned)
}

fn enrich_raw_solver_result(
    run_dir: &Path,
    result: &mut Value,
    input_lookup: &BTreeMap<String, Value>,
) -> Result<()> {
    let option_id = json_string_field(result, &["optionId", "option_id"]).unwrap_or_default();
    let group_id = json_string_field(result, &["coordinationGroupId", "coordination_group_id"])
        .unwrap_or_default();
    let section_id = json_string_field(result, &["sectionId", "section_id"]).unwrap_or_default();
    let key = DesignOptionAnalysisLookup::candidate_key(&option_id, &group_id, &section_id);
    if let Some(input) = input_lookup.get(&key)
        && let Some(object) = result.as_object_mut()
    {
        object.insert("candidateInput".into(), input.clone());
    }

    let Some(object) = result.as_object_mut() else {
        return Ok(());
    };
    let executions = object
        .get("executions")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let raw_files: Vec<Value> = executions
        .iter()
        .map(|execution| raw_files_for_execution(run_dir, execution))
        .collect::<Result<_>>()?;
    object.insert("rawFiles".into(), Value::Array(raw_files));
    Ok(())
}

fn raw_files_for_execution(run_dir: &Path, execution: &Value) -> Result<Value> {
    let job_name = json_string_field(execution, &["jobName", "job_name"]).unwrap_or_default();
    let working_dir =
        json_string_field(execution, &["workingDir", "working_dir"]).unwrap_or_default();
    let working_path = PathBuf::from(&working_dir);
    let safe_working_path = if working_path.starts_with(run_dir) {
        Some(working_path)
    } else {
        None
    };
    let mut files = serde_json::Map::new();
    for ext in ["inp", "dat", "sta", "cvg"] {
        let content = safe_working_path
            .as_ref()
            .map(|dir| dir.join(format!("{job_name}.{ext}")))
            .filter(|path| path.exists())
            .and_then(|path| fs::read_to_string(path).ok())
            .unwrap_or_default();
        files.insert(ext.into(), Value::String(content));
    }
    Ok(json!({
        "jobName": job_name,
        "workingDir": working_dir,
        "files": files,
    }))
}

fn load_latest_design_option_analysis_lookup(
    project_dir: &Path,
    project: &ProjectFile,
) -> Result<Option<DesignOptionAnalysisLookup>> {
    let runs_dir = project_dir.join("runs");
    if !runs_dir.exists() {
        return Ok(None);
    }
    let mut runs = fs::read_dir(&runs_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_type()
                .map(|file_type| file_type.is_dir())
                .unwrap_or(false)
                && entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("design-option-analysis-")
                && entry.path().join("comparison.json").exists()
        })
        .collect::<Vec<_>>();
    runs.sort_by_key(|entry| entry.file_name());
    if runs.is_empty() {
        return Ok(None);
    }
    let allowed_runs = project
        .design_option_decisions
        .active_batch_id
        .as_ref()
        .map(|active_batch_id| {
            project
                .design_option_decisions
                .batches
                .iter()
                .find(|batch| &batch.id == active_batch_id)
                .map(|batch| {
                    let mut runs = BTreeMap::<String, BTreeSet<String>>::new();
                    for revision in &batch.option_revisions {
                        if let Some(run_id) = revision.latest_analysis_run_id.as_ref() {
                            runs.entry(run_id.clone())
                                .or_default()
                                .insert(revision.option_id.clone());
                        }
                    }
                    runs
                })
                .unwrap_or_default()
        });
    let mut lookup = DesignOptionAnalysisLookup::default();
    let mut loaded_evidence = false;
    for run in runs {
        let comparison: DesignOptionAnalysisComparison =
            serde_json::from_str(&fs::read_to_string(run.path().join("comparison.json"))?)
                .with_context(|| {
                    format!(
                        "failed to read design-option analysis comparison from {}",
                        run.path().display()
                    )
                })?;
        let allowed_option_ids = allowed_runs
            .as_ref()
            .and_then(|runs| runs.get(&comparison.run_id));
        if allowed_runs.is_some() && allowed_option_ids.is_none() {
            continue;
        }
        let solver_results: Vec<Value> = run
            .path()
            .join("solver-results.json")
            .exists()
            .then(|| fs::read_to_string(run.path().join("solver-results.json")))
            .transpose()?
            .map(|text| serde_json::from_str(&text))
            .transpose()
            .with_context(|| {
                format!(
                    "failed to read design-option solver results from {}",
                    run.path().display()
                )
            })?
            .unwrap_or_default();
        for preview in solver_results {
            if allowed_option_ids.is_some_and(|option_ids| {
                preview
                    .get("optionId")
                    .or_else(|| preview.get("option_id"))
                    .and_then(Value::as_str)
                    .is_none_or(|option_id| !option_ids.contains(option_id))
            }) {
                continue;
            }
            lookup.insert_preview(preview);
        }
        for option in comparison.option_results {
            if allowed_option_ids.is_some_and(|option_ids| !option_ids.contains(&option.option_id))
            {
                continue;
            }
            loaded_evidence = true;
            let mut option_candidate_results = Vec::new();
            for result in option.candidate_results {
                lookup.insert_candidate(result.clone(), &comparison.run_id);
                option_candidate_results.push(result);
            }
            let selected = selected_design_option_analysis_result(&option_candidate_results)
                .or(option.selected_result);
            if let Some(selected) = selected {
                if let Some(preview) = lookup.preview(
                    &selected.option_id,
                    &selected.coordination_group_id,
                    &selected.section_id,
                ) {
                    lookup
                        .selected_previews
                        .insert(selected.option_id.clone(), preview.clone());
                }
                lookup
                    .selected_results
                    .insert(selected.option_id.clone(), selected);
            }
        }
    }
    Ok(loaded_evidence.then_some(lookup))
}

fn infer_latest_run_summary(
    latest_validate: &Option<SummaryArtifactRef>,
    latest_frame_calculix: &Option<SummaryArtifactRef>,
    latest_beam_calculix: &Option<SummaryArtifactRef>,
    latest_beam_analysis: &Option<SummaryArtifactRef>,
    latest_beam_sizing: &Option<SummaryArtifactRef>,
    latest_design_option_analysis: &Option<SummaryArtifactRef>,
    latest_import: &Option<SummaryArtifactRef>,
) -> Option<AnalysisRunSummary> {
    let candidates = [
        latest_frame_calculix.as_ref().map(|summary| {
            (
                summary,
                "frame_calculix",
                "Latest frame CalculiX run available.",
            )
        }),
        latest_beam_calculix.as_ref().map(|summary| {
            (
                summary,
                "beam_calculix",
                "Latest beam CalculiX run available.",
            )
        }),
        latest_beam_analysis.as_ref().map(|summary| {
            (
                summary,
                "beam_analysis",
                "Latest beam analysis run available.",
            )
        }),
        latest_beam_sizing
            .as_ref()
            .map(|summary| (summary, "beam_sizing", "Latest beam sizing run available.")),
        latest_design_option_analysis.as_ref().map(|summary| {
            (
                summary,
                "design_option_analysis",
                "Latest design-option analysis run available.",
            )
        }),
        latest_validate
            .as_ref()
            .map(|summary| (summary, "validation", "Latest validation run available.")),
        latest_import
            .as_ref()
            .map(|summary| (summary, "import", "Latest import artefacts available.")),
    ];

    candidates
        .into_iter()
        .flatten()
        .max_by_key(|(summary, _, _)| sort_suffix(&summary.run_id))
        .map(|(summary, kind, message)| AnalysisRunSummary {
            status: "completed".into(),
            analysis_kind: kind.into(),
            message: message.into(),
            run_id: Some(summary.run_id.clone()),
        })
}

fn sort_suffix(run_id: &str) -> String {
    run_id
        .rsplit_once('-')
        .map(|(_, suffix)| suffix.to_owned())
        .unwrap_or_else(|| run_id.to_owned())
}

fn supported_family(draft: &CorePlanningDraft) -> SupportedFamily {
    match canonical_system_family_hint(&draft.system_brief.system_family_hint).as_str() {
        "beam.simply_supported" => SupportedFamily::BeamSimplySupported,
        "portal_frame" => SupportedFamily::PortalFrame,
        other => SupportedFamily::Unsupported(other.to_owned()),
    }
}

fn canonical_system_family_hint(hint: &str) -> String {
    let normalized = hint.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "" => "beam.simply_supported".into(),
        "beam" | "beam.simply_supported" | "simply_supported_beam" | "simply-supported-beam" => {
            "beam.simply_supported".into()
        }
        "portal" | "portal_frame" | "portal-frame" | "frame" | "frame.portal_2d" => {
            "portal_frame".into()
        }
        other => other.to_owned(),
    }
}

fn evaluate_analysis_readiness(draft: &CorePlanningDraft) -> AnalysisReadiness {
    let mut diagnostics = Vec::new();
    let family = supported_family(draft);

    if draft.project_intent.name.trim().is_empty() {
        diagnostics.push(error_diagnostic(
            "planning.name_required",
            "Project name is required before the workbench can materialise or analyse a model.",
        ));
    }
    if draft.geometry_and_loads.span_m <= 0.0 {
        diagnostics.push(error_diagnostic(
            "planning.span_invalid",
            "Span must be greater than zero.",
        ));
    }
    if draft.geometry_and_loads.gravity_line_load_kn_per_m < 0.0 {
        diagnostics.push(error_diagnostic(
            "planning.gravity_invalid",
            "Gravity line load cannot be negative.",
        ));
    }
    if draft.geometry_and_loads.lateral_load_kn < 0.0 {
        diagnostics.push(error_diagnostic(
            "planning.lateral_invalid",
            "Lateral load cannot be negative.",
        ));
    }
    if draft.design_constraints.max_deflection_ratio <= 0.0 {
        diagnostics.push(error_diagnostic(
            "planning.deflection_invalid",
            "Deflection ratio must be greater than zero.",
        ));
    }
    if draft.design_constraints.max_drift_ratio <= 0.0 {
        diagnostics.push(error_diagnostic(
            "planning.drift_invalid",
            "Drift ratio must be greater than zero.",
        ));
    }
    if !(0.0..=1.0).contains(&draft.design_constraints.max_utilization)
        || draft.design_constraints.max_utilization == 0.0
    {
        diagnostics.push(error_diagnostic(
            "planning.utilisation_invalid",
            "Max utilisation must be within 0 and 1.",
        ));
    }
    if draft.design_constraints.allow_internal_columns
        && draft.design_constraints.max_internal_columns == 0
    {
        diagnostics.push(warning_diagnostic(
            "planning.internal_columns_zero",
            "Internal columns are allowed, but the maximum count is zero.",
        ));
    }

    match &family {
        SupportedFamily::BeamSimplySupported => {
            match parse_system_parameters::<BeamPlanningSystemParameters>(
                &draft.system_parameters,
                "beam.simply_supported",
            ) {
                Ok(Some(parameters)) => {
                    if let Some(point_load_x_m) = parameters.point_load_x_m
                        && !(0.0..=draft.geometry_and_loads.span_m).contains(&point_load_x_m)
                    {
                        diagnostics.push(error_diagnostic(
                            "planning.beam_point_load_position_invalid",
                            "Beam point-load position must sit within the current span.",
                        ));
                    }
                }
                Ok(None) => {}
                Err(_error) => diagnostics.push(error_diagnostic(
                    "planning.system_parameters_invalid",
                    "Beam system parameters could not be read.",
                )),
            }
        }
        SupportedFamily::PortalFrame => {
            if draft.geometry_and_loads.height_m <= 0.0 {
                diagnostics.push(error_diagnostic(
                    "planning.height_invalid",
                    "Portal frame height must be greater than zero.",
                ));
            }
            if let Err(_error) = parse_system_parameters::<PortalFramePlanningSystemParameters>(
                &draft.system_parameters,
                "portal_frame",
            ) {
                diagnostics.push(error_diagnostic(
                    "planning.system_parameters_invalid",
                    "Portal frame system parameters could not be read.",
                ));
            }
        }
        SupportedFamily::Unsupported(family) => diagnostics.push(WorkbenchDiagnostic {
            severity: "error".into(),
            code: "unsupported.system_family".into(),
            message: format!(
                "The system family `{family}` is valid planning input, but the app does not yet support model materialisation or analysis for it."
            ),
            detail: None,
        }),
    }

    let status = if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code.starts_with("unsupported."))
    {
        "unsupported"
    } else if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == "error")
    {
        "not_ready"
    } else if diagnostics.is_empty() {
        "ready"
    } else {
        "ready_with_notes"
    };

    let summary = match status {
        "ready" => "Ready to create or analyse the current planning draft.".into(),
        "ready_with_notes" => {
            "Ready to create or analyse the current planning draft, with minor notes.".into()
        }
        "unsupported" => {
            "The planning draft is saved, but the selected family is not yet supported for analysis."
                .into()
        }
        _ => "The planning draft needs attention before analysis can run.".into(),
    };

    AnalysisReadiness {
        status: status.into(),
        summary,
        diagnostics,
    }
}

fn evaluate_structural_solve_readiness(model: &StructuralModel) -> AnalysisReadiness {
    let mut diagnostics = Vec::new();
    if !model.members.is_empty() && model.supports.is_empty() {
        diagnostics.push(error_diagnostic(
            "analysis.supports_required",
            "Assign supports before solving the current structural model.",
        ));
    }
    if !model.members.is_empty() && model.loads.is_empty() {
        diagnostics.push(error_diagnostic(
            "analysis.loads_required",
            "Assign at least one load before solving the current structural model.",
        ));
    }
    for member in &model.members {
        if fraia_core::section_by_id(&member.section_id).is_none() {
            diagnostics.push(error_diagnostic(
                "analysis.sections_required",
                &format!(
                    "Member {} needs an assigned demo section before solving.",
                    member.id
                ),
            ));
        }
    }

    if diagnostics.is_empty() {
        AnalysisReadiness {
            status: "ready".into(),
            summary: "Ready to solve the current structural model.".into(),
            diagnostics,
        }
    } else {
        AnalysisReadiness {
            status: "not_ready".into(),
            summary: "The current structural model needs supports, loads, and sections before analysis can run.".into(),
            diagnostics,
        }
    }
}

fn error_diagnostic(code: &str, message: &str) -> WorkbenchDiagnostic {
    WorkbenchDiagnostic {
        severity: "error".into(),
        code: code.into(),
        message: message.into(),
        detail: None,
    }
}

fn warning_diagnostic(code: &str, message: &str) -> WorkbenchDiagnostic {
    WorkbenchDiagnostic {
        severity: "warning".into(),
        code: code.into(),
        message: message.into(),
        detail: None,
    }
}

fn warning_diagnostic_with_detail(code: &str, message: &str, detail: &str) -> WorkbenchDiagnostic {
    WorkbenchDiagnostic {
        severity: "warning".into(),
        code: code.into(),
        message: message.into(),
        detail: Some(detail.into()),
    }
}

fn merge_diagnostic(state: &mut WorkbenchProjectState, diagnostic: WorkbenchDiagnostic) {
    if diagnostic.code.starts_with("unsupported.") {
        state.capability_diagnostics.push(diagnostic.clone());
    }
    let readiness = state.analysis_readiness.get_or_insert(AnalysisReadiness {
        status: "not_ready".into(),
        summary: "The planning draft needs attention before analysis can run.".into(),
        diagnostics: Vec::new(),
    });
    readiness.diagnostics.push(diagnostic);
}

fn parse_system_parameters<T: DeserializeOwned>(
    system_parameters: &std::collections::BTreeMap<String, Value>,
    key: &str,
) -> Result<Option<T>> {
    let Some(value) = system_parameters.get(key) else {
        return Ok(None);
    };
    let parsed = serde_json::from_value::<T>(value.clone())
        .with_context(|| format!("failed to parse system parameters for `{key}`"))?;
    Ok(Some(parsed))
}

fn api_planning_to_core(draft: ApiPlanningDraft) -> CorePlanningDraft {
    CorePlanningDraft {
        project_intent: CorePlanningProjectIntent {
            name: draft.project_intent.name,
            building_type: draft.project_intent.building_type,
            design_stage: draft.project_intent.design_stage,
            objective_priority: draft.project_intent.objective_priority,
        },
        system_brief: CorePlanningSystemBrief {
            system_family_hint: draft.system_brief.system_family_hint,
            structural_form_hint: draft.system_brief.structural_form_hint,
            notes: draft.system_brief.notes,
        },
        geometry_and_loads: CorePlanningGeometryAndLoads {
            span_m: draft.geometry_and_loads.span_m,
            height_m: draft.geometry_and_loads.height_m,
            gravity_line_load_kn_per_m: draft.geometry_and_loads.gravity_line_load_kn_per_m,
            lateral_load_kn: draft.geometry_and_loads.lateral_load_kn,
        },
        design_constraints: CorePlanningDesignConstraints {
            max_deflection_ratio: draft.design_constraints.max_deflection_ratio,
            max_drift_ratio: draft.design_constraints.max_drift_ratio,
            max_utilization: draft.design_constraints.max_utilization,
            allow_internal_columns: draft.design_constraints.allow_internal_columns,
            max_internal_columns: draft.design_constraints.max_internal_columns,
        },
        analysis_brief: CorePlanningAnalysisBrief {
            requested_analysis_intent: draft.analysis_brief.requested_analysis_intent,
            preferred_backend: draft.analysis_brief.preferred_backend,
            summary_goals: draft.analysis_brief.summary_goals,
        },
        system_parameters: draft.system_parameters,
    }
}

fn core_planning_to_api(draft: CorePlanningDraft) -> ApiPlanningDraft {
    ApiPlanningDraft {
        project_intent: ApiPlanningProjectIntent {
            name: draft.project_intent.name,
            building_type: draft.project_intent.building_type,
            design_stage: draft.project_intent.design_stage,
            objective_priority: draft.project_intent.objective_priority,
        },
        system_brief: ApiPlanningSystemBrief {
            system_family_hint: draft.system_brief.system_family_hint,
            structural_form_hint: draft.system_brief.structural_form_hint,
            notes: draft.system_brief.notes,
        },
        geometry_and_loads: ApiPlanningGeometryAndLoads {
            span_m: draft.geometry_and_loads.span_m,
            height_m: draft.geometry_and_loads.height_m,
            gravity_line_load_kn_per_m: draft.geometry_and_loads.gravity_line_load_kn_per_m,
            lateral_load_kn: draft.geometry_and_loads.lateral_load_kn,
        },
        design_constraints: ApiPlanningDesignConstraints {
            max_deflection_ratio: draft.design_constraints.max_deflection_ratio,
            max_drift_ratio: draft.design_constraints.max_drift_ratio,
            max_utilization: draft.design_constraints.max_utilization,
            allow_internal_columns: draft.design_constraints.allow_internal_columns,
            max_internal_columns: draft.design_constraints.max_internal_columns,
        },
        analysis_brief: ApiPlanningAnalysisBrief {
            requested_analysis_intent: draft.analysis_brief.requested_analysis_intent,
            preferred_backend: draft.analysis_brief.preferred_backend,
            summary_goals: draft.analysis_brief.summary_goals,
        },
        system_parameters: draft.system_parameters,
    }
}

fn persist_validation_run(project_dir: &Path, project: &ProjectFile) -> Result<PathBuf> {
    let structural = materialize_project_structural_model(project).context(
        "no authored structural model or builder-derived structural model saved in the project yet",
    )?;
    let validation = validate_structural_model(&structural);
    let realization = realize_structural_model_to_frame2d(&structural).ok();
    let design_actions = realization
        .as_ref()
        .and_then(|realization| derive_design_action_report(project, &realization.model).ok());
    let checks = design_actions
        .as_ref()
        .map(|actions| derive_conservative_check_report(project, actions));

    let run_dir = project_dir
        .join("runs")
        .join(format!("validate-{}", fraia_core::utils::timestamp_id()));
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
            fraia_core::render_member_actions_csv(design_actions.as_ref()),
        )?;
        fs::write(
            run_dir.join("check-results.csv"),
            fraia_core::render_check_results_csv(&checks.results),
        )?;
        fs::write(
            run_dir.join("support-reactions.csv"),
            fraia_core::render_support_reactions_csv(design_actions.as_ref()),
        )?;
    }
    let summary = render_validation_summary(
        project,
        &validation,
        realization.as_ref(),
        design_actions.as_ref(),
        checks.as_ref(),
    );
    fs::write(run_dir.join("summary.md"), summary)?;
    Ok(run_dir)
}

fn persist_frame_calculix_run(project_dir: &Path, project: &ProjectFile) -> Result<PathBuf> {
    materialize_project_structural_model(project).context(
        "no authored structural model or builder-derived structural model saved in the project yet",
    )?;

    let run_dir = project_dir.join("runs").join(format!(
        "frame-calculix-run-{}",
        fraia_core::utils::timestamp_id()
    ));
    fs::create_dir_all(&run_dir)?;
    let analysis = execute_current_frame_project_in_calculix(project, &run_dir)?;
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
    let summary = render_frame_calculix_execution_summary(project, &analysis);
    fs::write(run_dir.join("summary.md"), summary)?;
    Ok(run_dir)
}

fn persist_beam_sizing_run(project_dir: &Path, project: &mut ProjectFile) -> Result<PathBuf> {
    let sizing = size_current_simply_supported_beam_in_project(project)?;
    save_project(project_dir, project)?;
    update_planning_markdown(project_dir, &default_planning_markdown(project))?;

    let run_dir = project_dir
        .join("runs")
        .join(format!("beam-size-{}", fraia_core::utils::timestamp_id()));
    fs::create_dir_all(&run_dir)?;
    fraia_core::utils::write_json(&run_dir.join("sizing.json"), &sizing)?;
    let summary = render_beam_sizing_summary(project, &sizing);
    fs::write(run_dir.join("summary.md"), summary)?;
    Ok(run_dir)
}

fn persist_beam_analysis_run(project_dir: &Path, project: &ProjectFile) -> Result<PathBuf> {
    persist_project_and_markdown(project_dir, project)?;
    let analysis = analyze_current_simply_supported_beam_project(project)?;
    let run_dir = project_dir.join("runs").join(format!(
        "beam-analysis-{}",
        fraia_core::utils::timestamp_id()
    ));
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
    let summary = render_beam_analysis_summary(project, &analysis);
    fs::write(run_dir.join("summary.md"), summary)?;
    Ok(run_dir)
}

fn parse_port(mut args: impl Iterator<Item = String>) -> Result<u16> {
    let mut port = 7878u16;
    while let Some(arg) = args.next() {
        if arg == "--port" {
            let value = args
                .next()
                .ok_or_else(|| anyhow!("expected value after --port"))?;
            port = value
                .parse::<u16>()
                .with_context(|| format!("invalid port value {value}"))?;
        }
    }
    Ok(port)
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
}

impl From<anyhow::Error> for ApiError {
    fn from(value: anyhow::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("{value:#}"),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorResponse {
                error: self.message,
            }),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appd_authentication_comparison_rejects_missing_wrong_and_truncated_tokens() {
        let token = b"0123456789abcdef0123456789abcdef";
        assert!(constant_time_equal(token, token));
        assert!(!constant_time_equal(token, b""));
        assert!(!constant_time_equal(token, b"0123456789abcdef"));
        assert!(!constant_time_equal(
            token,
            b"0123456789abcdef0123456789abcdeg"
        ));
    }

    #[test]
    fn appd_authentication_requires_an_exact_bearer_header() {
        let token = "0123456789abcdef0123456789abcdef";
        let mut headers = HeaderMap::new();
        assert!(!request_is_authorised(&headers, token));
        headers.insert(AUTHORIZATION, "Basic ignored".parse().unwrap());
        assert!(!request_is_authorised(&headers, token));
        headers.insert(AUTHORIZATION, format!("Bearer {token}x").parse().unwrap());
        assert!(!request_is_authorised(&headers, token));
        headers.insert(AUTHORIZATION, format!("Bearer {token}").parse().unwrap());
        assert!(request_is_authorised(&headers, token));
    }

    fn test_pi_model(provider_id: &str, model_id: &str, available: bool) -> AgentModelOption {
        AgentModelOption {
            provider_id: provider_id.into(),
            slug: model_id.into(),
            display_name: model_id.into(),
            default_reasoning_level: "low".into(),
            supported_reasoning_levels: vec![
                AgentReasoningOption {
                    effort: "off".into(),
                    description: String::new(),
                },
                AgentReasoningOption {
                    effort: "low".into(),
                    description: String::new(),
                },
                AgentReasoningOption {
                    effort: "high".into(),
                    description: String::new(),
                },
            ],
            available,
            context_window: Some(128_000),
            max_tokens: Some(16_384),
        }
    }

    #[test]
    fn pi_model_settings_keep_exact_provider_and_model() {
        let models = vec![
            test_pi_model("openai-codex", "gpt-5.5", true),
            test_pi_model("anthropic", "claude-sonnet", true),
        ];
        let settings = validate_agent_settings_from_models(
            &models,
            Some("anthropic"),
            Some("claude-sonnet"),
            Some("high"),
        );
        assert_eq!(settings.provider_id, "anthropic");
        assert_eq!(settings.model, "claude-sonnet");
        assert_eq!(settings.reasoning_effort, "high");
    }

    #[test]
    fn retired_model_reference_is_preserved_without_silent_switch() {
        let models = vec![test_pi_model("openai-codex", "gpt-new-default", true)];
        let settings = validate_agent_settings_from_models(
            &models,
            Some("openai-codex"),
            Some("retired-model"),
            Some("medium"),
        );
        assert_eq!(settings.provider_id, "openai-codex");
        assert_eq!(settings.model, "retired-model");
        assert_eq!(settings.reasoning_effort, "medium");
    }

    #[test]
    fn legacy_agent_settings_migrate_to_openai_codex_provider() {
        let settings: AgentModelSettings = serde_json::from_value(json!({
            "model": "gpt-5.5",
            "reasoningEffort": "low"
        }))
        .expect("legacy settings deserialize");
        assert_eq!(settings.provider_id, "openai-codex");
        assert_eq!(settings.model, "gpt-5.5");
        let serialized = serde_json::to_value(settings).expect("settings serialize");
        assert_eq!(serialized["providerId"], "openai-codex");
        assert_eq!(serialized["modelId"], "gpt-5.5");
        assert!(serialized.get("model").is_none());
    }

    #[test]
    fn pi_error_summary_ignores_context_json_messages() {
        let raw = "Pi turn failed: model output was not valid\n--------\nuser\nContext JSON:\n{\"diagnostics\":[{\"message\":\"Assign supports before solving the current structural model.\"}]}";
        let summary = summarize_pi_error(raw);
        assert!(summary.contains("model output was not valid"));
        assert!(!summary.contains("Assign supports"));
    }

    #[test]
    fn scheme_surfaces_inherit_default_agent_model_settings() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-agent-settings-inherit-test-{}",
            fraia_core::utils::iso_now().replace([':', '-'], "")
        ));
        let (mut project, _) =
            create_project(&project_dir, "agent settings inherit test").expect("create project");
        ensure_agent_settings(&mut project);
        project.agent_state.settings_by_surface.insert(
            "default".into(),
            AgentModelSettings {
                provider_id: "openai-codex".into(),
                model: "gpt-5.4".into(),
                reasoning_effort: "high".into(),
            },
        );
        project.agent_state.settings_by_surface.insert(
            "pre_solve".into(),
            AgentModelSettings {
                provider_id: "openai-codex".into(),
                model: "gpt-5.4".into(),
                reasoning_effort: "high".into(),
            },
        );

        let inherited = agent_settings_for_surface(&project, "scheme:light-open-frame");
        assert_eq!(inherited.model, "gpt-5.4");
        assert_eq!(inherited.reasoning_effort, "high");

        project.agent_state.settings_by_surface.insert(
            "scheme:light-open-frame".into(),
            AgentModelSettings {
                provider_id: "openai-codex".into(),
                model: "gpt-5.5".into(),
                reasoning_effort: "xhigh".into(),
            },
        );
        let explicit = agent_settings_for_surface(&project, "scheme:light-open-frame");
        assert_eq!(explicit.model, "gpt-5.5");
        assert_eq!(explicit.reasoning_effort, "xhigh");
        let _ = fs::remove_dir_all(project_dir);
    }

    fn test_scheme_session(surface: &str, messages: Vec<AgentMessage>) -> AgentSession {
        AgentSession {
            id: format!("session-{surface}"),
            surface: surface.into(),
            title: "Design option chat".into(),
            status: "active".into(),
            messages,
            plan_items: Vec::new(),
            current_question: None,
            created_at: "2026-05-14T00:00:00Z".into(),
            updated_at: "2026-05-14T00:00:00Z".into(),
        }
    }

    fn test_agent_message(author: &str, text: &str, suggested_replies: Vec<&str>) -> AgentMessage {
        AgentMessage {
            author: author.into(),
            text: text.into(),
            created_at: "2026-05-14T00:00:00Z".into(),
            mode: None,
            model: None,
            provider_id: None,
            reasoning_effort: None,
            catalogue_refreshed_at: None,
            suggested_replies: suggested_replies.into_iter().map(str::to_string).collect(),
            suggested_reply_groups: Vec::new(),
            plan_summary: None,
            proposed_actions: Vec::new(),
        }
    }

    #[test]
    fn scheme_pending_decision_blocks_analysis_until_user_replies() {
        let pending = test_scheme_session(
            "scheme:braced-load-path-option",
            vec![test_agent_message(
                "assistant",
                "May this option assume added bracing between the existing frame Nodes as a concept variation?",
                vec!["Yes", "No"],
            )],
        );
        let decision = pending_scheme_decision_from_session(&pending).expect("pending decision");
        assert_eq!(decision.scheme_id, "braced-load-path-option");

        let answered = test_scheme_session(
            "scheme:braced-load-path-option",
            vec![
                test_agent_message(
                    "assistant",
                    "May this option assume added bracing between the existing frame Nodes as a concept variation?",
                    vec!["Yes", "No"],
                ),
                test_agent_message(
                    "user",
                    "Yes, allow bracing between existing Nodes only.",
                    vec![],
                ),
            ],
        );
        assert!(pending_scheme_decision_from_session(&answered).is_none());
    }

    #[test]
    fn base_model_brief_persists_generated_artifacts() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-base-model-brief-test-{}",
            fraia_core::utils::iso_now().replace([':', '-'], "")
        ));
        let (mut project, _) =
            create_project(&project_dir, "brief artifact test").expect("create project");
        project.base_model_brief = Some(BaseModelBrief {
            version: 1,
            session_id: "session-pre_solve".into(),
            current_understanding: "A raw base model is being discussed.".into(),
            confirmed_intent: vec!["Use this as concept geometry only.".into()],
            open_questions: vec!["Confirm support intent.".into()],
            soft_assumptions: Vec::new(),
            schema_guidance: vec!["Let design options compare support assumptions.".into()],
            do_not_decide_yet: vec!["Do not pick member sections in the base guide.".into()],
            visual_intent: BaseModelBriefVisualIntent::default(),
            readiness: BaseModelBriefReadiness {
                ready_for_schemas: false,
                unresolved_topics: vec!["support intent".into()],
                manual_override_allowed: true,
            },
            updated_at: fraia_core::utils::iso_now(),
        });
        persist_project_and_markdown(&project_dir, &project).expect("persist brief");
        let json_path = project_dir.join("generated/base-model-brief.json");
        let markdown_path = project_dir.join("generated/base-model-brief.md");
        assert!(json_path.exists());
        assert!(markdown_path.exists());
        let markdown = fs::read_to_string(markdown_path).expect("read brief markdown");
        assert!(markdown.contains("# Base Model Brief"));
        assert!(markdown.contains("Let design options compare support assumptions."));
        let _ = fs::remove_dir_all(project_dir);
    }

    #[test]
    fn pre_solve_prompt_keeps_support_type_as_schema_decision() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-pre-solve-prompt-test-{}",
            fraia_core::utils::iso_now().replace([':', '-'], "")
        ));
        let (mut project, _) =
            create_project(&project_dir, "pre solve prompt test").expect("create project");
        project.structural_model = Some(test_portal_frame_model());

        let prompt = build_pi_session_prompt(&project, "pre_solve");

        assert!(prompt.contains("where it can physically be supported"));
        assert!(prompt.contains("whether design options should treat the geometry as standalone"));
        assert!(prompt.contains("hard constraints or no-go zones"));
        assert!(prompt.contains("Do not ask the user to choose support type"));
        assert!(
            prompt.contains("section family, member grouping, base fixity, and stability strategy")
        );
        assert!(prompt.contains("support locations/constraints"));
        assert!(prompt.contains("under 60 words total"));
        assert!(prompt.contains("Do not explain generic support theory"));
        assert!(prompt.contains("horizontal point load at Node N2, direction +X toward Node N3"));
        assert!(prompt.contains("Never use view-dependent location words"));
        assert!(!prompt.contains("broad support/stability intent"));
        let _ = fs::remove_dir_all(project_dir);
    }

    #[test]
    fn pre_solve_prompt_includes_wiki_knowledge_context() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-pre-solve-knowledge-prompt-test-{}",
            fraia_core::utils::iso_now().replace([':', '-'], "")
        ));
        let (mut project, _) = create_project(&project_dir, "pre solve knowledge prompt test")
            .expect("create project");
        project.structural_model = Some(test_portal_frame_model());

        let prompt = build_pi_session_prompt(&project, "pre_solve");

        assert!(prompt.contains("\"knowledgeContext\""));
        assert!(prompt.contains("docs/knowledge/wiki/product/scheme-generation-from-knowledge.md"));
        assert!(prompt.contains("Use knowledgeContext as internal structural knowledge"));
        assert!(prompt.contains("Do not mention the wiki"));
        assert!(prompt.contains("never as project-specific approval"));
        let _ = fs::remove_dir_all(project_dir);
    }

    #[test]
    fn scheme_prompt_includes_wiki_knowledge_context() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-scheme-knowledge-prompt-test-{}",
            fraia_core::utils::iso_now().replace([':', '-'], "")
        ));
        let (mut project, _) =
            create_project(&project_dir, "scheme knowledge prompt test").expect("create project");
        project.structural_model = Some(test_portal_frame_model());
        add_test_brief_support_locations(&mut project);
        let mut draft = planning_draft(&project);
        draft.system_parameters.insert(
            "designOptionIntents".into(),
            json!([{
                "id": "agent-simple-details",
                "label": "Agent simple details",
                "hypothesis": "A compatible closed-section direction is worth exploring because the user prioritised simple connection families.",
                "explorationBand": "concept-option",
                "objectiveTags": ["connection_simplicity"],
                "standardisationStrategy": "section-family repetition",
                "connectionStrategy": "least variation in connection families",
                "supportStrategy": "use authored support locations; compare pinned restraint only when locations are explicit",
                "sectionFamilyPolicy": "closed-section families",
                "coordinationGroupPolicy": "coordinate groups by detail family",
                "assumptions": ["Exact section IDs remain downstream."],
                "provenance": [wiki_grounded_test_provenance()]
            }]),
        );
        project.planning_draft = Some(draft);

        let prompt = build_pi_session_prompt(&project, "scheme:agent-simple-details");

        assert!(prompt.contains("\"knowledgeContext\""));
        assert!(prompt.contains("\"selectedDesignScheme\""));
        assert!(prompt.contains("\"id\": \"agent-simple-details\""));
        assert!(prompt.contains("docs/knowledge/wiki/product/scheme-generation-from-knowledge.md"));
        assert!(
            prompt.contains("docs/knowledge/wiki/product/structural-design-option-intelligence.md")
        );
        assert!(prompt.contains(
            "docs/knowledge/wiki/materials/steel/material-properties-and-section-families.md"
        ));
        assert!(prompt.contains("DesignOptionIntent"));
        assert!(prompt.contains("Do not state or recommend exact member section sizes"));
        assert!(prompt.contains("write a concise opening review that already explains the option"));
        assert!(prompt.contains("load path, support/restraint behaviour"));
        assert!(prompt.contains("Do not use suggestedReplies or suggestedReplyGroups"));
        assert!(prompt.contains("coordination.designOptionReplacement"));
        assert!(prompt.contains("do not ask generic steering questions"));
        assert!(prompt.contains("Design options are immutable comparison artefacts"));
        let _ = fs::remove_dir_all(project_dir);
    }

    #[test]
    fn review_prompt_includes_wiki_knowledge_context() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-review-knowledge-prompt-test-{}",
            fraia_core::utils::iso_now().replace([':', '-'], "")
        ));
        let (mut project, _) =
            create_project(&project_dir, "review knowledge prompt test").expect("create project");
        project.structural_model = Some(test_portal_frame_model());
        let request = AgentReviewReplyRequest {
            project_dir: project_dir.to_string_lossy().into_owned(),
            comment_id: "section-family-portal-rafters".into(),
            comment: json!({
                "title": "Section families for portal rafters",
                "targets": [{ "kind": "coordination_group", "id": "portal-rafters" }]
            }),
            selected_chips: vec!["allow UB and PFC".into()],
            reply: String::new(),
            messages: Vec::new(),
            model: Some("gpt-5.5".into()),
            reasoning_effort: Some("low".into()),
        };

        let prompt = build_pi_review_prompt(&project, &request);

        assert!(prompt.contains("\"knowledgeContext\""));
        assert!(prompt.contains("\"retrievalQueries\""));
        assert!(prompt.contains("review load application equivalent nodal loads"));
        assert!(
            prompt.contains("docs/knowledge/wiki/materials/steel/connections-concept-taxonomy.md")
        );
        assert!(prompt.contains("\"supportedSectionFamilies\""));
        assert!(!prompt.contains("Allowed family names include UB"));
        assert!(prompt.contains("Use knowledgeContext as internal structural knowledge"));
        assert!(prompt.contains("Do not mention the wiki"));
        assert!(prompt.contains("never as project-specific approval"));
        let _ = fs::remove_dir_all(project_dir);
    }

    fn knowledge_context_paths(context: &Value) -> Vec<String> {
        let Some(pages) = context["pages"].as_array() else {
            return Vec::new();
        };
        pages
            .iter()
            .filter_map(|page| page["path"].as_str())
            .map(str::to_owned)
            .collect()
    }

    #[test]
    fn llm_knowledge_context_uses_retrieval_queries_instead_of_fixed_page_menu() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-core-knowledge-context-test-{}",
            fraia_core::utils::iso_now().replace([':', '-'], "")
        ));
        let (project, _) =
            create_project(&project_dir, "core knowledge context test").expect("create project");
        let draft = planning_draft(&project);

        let context = build_llm_knowledge_context(&project, &draft, None, "member", "session");
        let paths = knowledge_context_paths(&context);

        assert!(
            context["retrievalQueries"]
                .as_array()
                .is_some_and(|queries| {
                    queries.iter().any(|query| {
                        query.as_str().is_some_and(|text| {
                            text.contains("authored resolved run artifact boundaries")
                        })
                    })
                })
        );
        assert!(!paths.is_empty());
        assert!(paths.iter().any(|path| path.contains("wiki/product/")));
        assert!(paths.iter().any(|path| {
            path.contains("wiki/modeling/")
                || path.contains("wiki/loads/")
                || path.contains("wiki/analysis/")
                || path.contains("wiki/stability/")
        }));
        let _ = fs::remove_dir_all(project_dir);
    }

    #[test]
    fn portal_frame_models_retrieve_portal_frame_pages() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-portal-knowledge-context-test-{}",
            fraia_core::utils::iso_now().replace([':', '-'], "")
        ));
        let (mut project, _) =
            create_project(&project_dir, "portal knowledge context test").expect("create project");
        project.intent.building_type = "portal_frame".into();
        project.structural_model = Some(test_portal_frame_model());
        let draft = planning_draft(&project);

        let context = build_llm_knowledge_context(
            &project,
            &draft,
            project.structural_model.as_ref(),
            "pre_solve",
            "session",
        );
        let paths = knowledge_context_paths(&context);

        assert!(
            paths
                .iter()
                .any(|path| path.contains("wiki/steel/portal-frames/"))
        );
        let _ = fs::remove_dir_all(project_dir);
    }

    #[test]
    fn review_context_retrieves_load_section_and_coordination_pages() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-review-knowledge-context-test-{}",
            fraia_core::utils::iso_now().replace([':', '-'], "")
        ));
        let (mut project, _) =
            create_project(&project_dir, "review knowledge context test").expect("create project");
        project.structural_model = Some(test_portal_frame_model());
        let draft = planning_draft(&project);

        let context = build_llm_knowledge_context(
            &project,
            &draft,
            project.structural_model.as_ref(),
            "review_reply",
            "review a load question and section-family coordination reply",
        );
        let paths = knowledge_context_paths(&context);

        assert!(
            paths
                .iter()
                .any(|path| path.ends_with("load-application-and-equivalent-nodal-loads.md"))
        );
        assert!(
            paths
                .iter()
                .any(|path| path.ends_with("material-properties-and-section-families.md"))
        );
        assert!(
            paths
                .iter()
                .any(|path| path.ends_with("connection-fixity-and-partial-restraint.md"))
        );
        assert!(
            paths
                .iter()
                .any(|path| path.ends_with("scheme-generation-from-knowledge.md"))
        );
        let _ = fs::remove_dir_all(project_dir);
    }

    #[test]
    fn initial_base_model_brief_tracks_fixed_boundary_questions() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-initial-base-model-brief-test-{}",
            fraia_core::utils::iso_now().replace([':', '-'], "")
        ));
        let (mut project, _) =
            create_project(&project_dir, "initial brief test").expect("create project");
        project.structural_model = Some(test_portal_frame_model());

        let brief = initial_base_model_brief(&project, "session-pre_solve");

        assert!(
            brief
                .open_questions
                .iter()
                .any(|question| question.contains("standalone, representative/repeated"))
        );
        assert!(
            brief
                .open_questions
                .iter()
                .any(|question| question.contains("hard constraints or no-go zones"))
        );
        assert!(
            brief
                .schema_guidance
                .iter()
                .any(|item| item.contains("fixed boundaries"))
        );
        assert!(brief.schema_guidance.iter().any(|item| item.contains(
            "support kind, section family, member grouping, base fixity, and stability strategy"
        )));
        assert!(brief.do_not_decide_yet.iter().any(|item| {
            item.contains("Section family and member grouping remain design-option alternatives")
        }));
        assert!(!brief.readiness.ready_for_schemas);
        let _ = fs::remove_dir_all(project_dir);
    }

    #[test]
    fn base_model_brief_visual_intent_is_validated_against_structural_model() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-brief-visual-intent-test-{}",
            fraia_core::utils::iso_now().replace([':', '-'], "")
        ));
        let (mut project, _) =
            create_project(&project_dir, "brief visual intent test").expect("create project");
        project.structural_model = Some(test_portal_frame_model());
        let mut brief = initial_base_model_brief(&project, "session-pre_solve");
        brief.confirmed_intent = vec![
            "Support nodes n1 and n4 as provisional support locations.".into(),
            "Use self weight and a 10 kN point load at node n2 toward node n3.".into(),
        ];
        brief.visual_intent.support_locations = vec![
            fraia_core::BaseModelBriefSupportLocationIntent {
                id: "support-n1".into(),
                target_node: "n1".into(),
                label: Some("base".into()),
                status: "fixed".into(),
            },
            fraia_core::BaseModelBriefSupportLocationIntent {
                id: "support-missing".into(),
                target_node: "missing".into(),
                label: None,
                status: "location_only".into(),
            },
        ];
        brief.visual_intent.loads = vec![
            fraia_core::BaseModelBriefLoadIntent {
                id: "self-weight".into(),
                kind: "self_weight".into(),
                target: BaseModelBriefLoadTarget {
                    kind: "all_members".into(),
                    member_id: None,
                    node_id: None,
                },
                magnitude_n: None,
                magnitude_n_per_m: None,
                direction: None,
            },
            fraia_core::BaseModelBriefLoadIntent {
                id: "line-e2".into(),
                kind: "uniform_line".into(),
                target: BaseModelBriefLoadTarget {
                    kind: "member".into(),
                    member_id: Some("e2".into()),
                    node_id: None,
                },
                magnitude_n: None,
                magnitude_n_per_m: Some(5_000.0),
                direction: Some(BaseModelBriefLoadDirection {
                    kind: "vector".into(),
                    from_node: None,
                    to_node: None,
                    x: Some(0.0),
                    y: Some(-1.0),
                    z: Some(0.0),
                }),
            },
            fraia_core::BaseModelBriefLoadIntent {
                id: "point-n2-n3".into(),
                kind: "point".into(),
                target: BaseModelBriefLoadTarget {
                    kind: "node".into(),
                    member_id: None,
                    node_id: Some("n2".into()),
                },
                magnitude_n: Some(10_000.0),
                magnitude_n_per_m: None,
                direction: Some(BaseModelBriefLoadDirection {
                    kind: "toward_node".into(),
                    from_node: Some("n2".into()),
                    to_node: Some("n3".into()),
                    x: None,
                    y: None,
                    z: None,
                }),
            },
            fraia_core::BaseModelBriefLoadIntent {
                id: "bad-point".into(),
                kind: "point".into(),
                target: BaseModelBriefLoadTarget {
                    kind: "node".into(),
                    member_id: None,
                    node_id: Some("n2".into()),
                },
                magnitude_n: None,
                magnitude_n_per_m: None,
                direction: Some(BaseModelBriefLoadDirection {
                    kind: "toward_node".into(),
                    from_node: Some("n2".into()),
                    to_node: Some("n3".into()),
                    x: None,
                    y: None,
                    z: None,
                }),
            },
        ];

        let diagnostics = validate_base_model_brief_visual_intent(&project, &mut brief);

        assert_eq!(brief.visual_intent.support_locations.len(), 1);
        assert_eq!(
            brief.visual_intent.support_locations[0].status,
            "location_only"
        );
        assert_eq!(brief.visual_intent.loads.len(), 3);
        assert!(brief.visual_intent.loads.iter().any(|load| {
            load.id == "self-weight"
                && load.kind == "self_weight"
                && load.target.kind == "all_members"
        }));
        assert!(
            brief
                .visual_intent
                .loads
                .iter()
                .any(|load| load.id == "point-n2-n3")
        );
        assert!(
            brief
                .visual_intent
                .loads
                .iter()
                .any(|load| load.id == "line-e2"
                    && load.kind == "uniform_line"
                    && load.target.member_id.as_deref() == Some("e2")
                    && load.magnitude_n_per_m == Some(5_000.0))
        );
        assert_eq!(diagnostics.len(), 2);
        assert!(
            brief
                .open_questions
                .contains(&POINT_LOAD_MAGNITUDE_QUESTION.into())
        );
        let (message, replies) =
            pre_solve_blocking_reply(&brief).expect("missing magnitude should block");
        assert!(message.contains("magnitude is missing"));
        assert!(replies.iter().any(|reply| reply.contains("10 kN")));
        let _ = fs::remove_dir_all(project_dir);
    }

    #[test]
    fn qualitative_brief_intent_does_not_block_schema_readiness() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-qualitative-brief-ready-test-{}",
            fraia_core::utils::timestamp_id()
        ));
        let (mut project, _) =
            create_project(&project_dir, "qualitative brief ready test").expect("create project");
        project.structural_model = Some(test_portal_frame_model());
        let mut brief = initial_base_model_brief(&project, "session-pre_solve");
        brief.confirmed_intent = vec![
            "Treat N1 and N4 as concept support locations; support fixity remains a design-option alternative.".into(),
            "Include self weight and broad roof gravity loading intent for design-option generation.".into(),
        ];
        brief.open_questions = vec![
            LEGACY_SUPPORT_VISUAL_INTENT_QUESTION.into(),
            LEGACY_LOAD_VISUAL_INTENT_QUESTION.into(),
        ];
        brief.readiness.ready_for_schemas = true;
        brief.readiness.unresolved_topics = brief.open_questions.clone();

        let diagnostics = validate_base_model_brief_visual_intent(&project, &mut brief);

        assert!(diagnostics.is_empty());
        assert!(brief.open_questions.is_empty());
        assert!(brief.readiness.unresolved_topics.is_empty());
        assert!(brief.readiness.ready_for_schemas);
        let _ = fs::remove_dir_all(project_dir);
    }

    #[test]
    fn schema_handoff_snapshot_captures_base_model_brief() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-design-option-handoff-test-{}",
            fraia_core::utils::iso_now().replace([':', '-'], "")
        ));
        let (mut project, _) =
            create_project(&project_dir, "design option handoff test").expect("create project");
        project.base_model_brief = Some(BaseModelBrief {
            version: 1,
            session_id: "session-pre_solve".into(),
            current_understanding: "Ready enough for design-option exploration.".into(),
            confirmed_intent: vec!["Treat supports as design-option alternatives.".into()],
            open_questions: Vec::new(),
            soft_assumptions: Vec::new(),
            schema_guidance: vec!["Compare reasonable support assumptions.".into()],
            do_not_decide_yet: Vec::new(),
            visual_intent: BaseModelBriefVisualIntent::default(),
            readiness: BaseModelBriefReadiness {
                ready_for_schemas: true,
                unresolved_topics: Vec::new(),
                manual_override_allowed: true,
            },
            updated_at: fraia_core::utils::iso_now(),
        });
        let run_dir = persist_schema_handoff_snapshot(&project_dir, &project).expect("snapshot");
        assert!(run_dir.join("base-model-brief.json").exists());
        assert!(run_dir.join("run.json").exists());
        let summary = fs::read_to_string(run_dir.join("summary.md")).expect("read summary");
        assert!(summary.contains("Design Option Handoff"));
        let _ = fs::remove_dir_all(project_dir);
    }

    fn test_member_model() -> StructuralModel {
        StructuralModel {
            dimension: "2d".into(),
            nodes: vec![
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
            ],
            members: vec![StructuralMember {
                id: "m1".into(),
                start_node: "n1".into(),
                end_node: "n2".into(),
                role: "rafter".into(),
                semantic_tags: vec!["roof".into(), "primary".into()],
                section_id: "200UB".into(),
                material_id: "steel".into(),
            }],
            plates: Vec::new(),
            supports: Vec::new(),
            loads: Vec::new(),
            releases: Vec::new(),
            load_cases: Vec::new(),
            builder_node_materializations: Vec::new(),
        }
    }

    fn test_portal_frame_model() -> StructuralModel {
        StructuralModel {
            dimension: "2d".into(),
            nodes: vec![
                StructuralNode {
                    id: "n1".into(),
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                StructuralNode {
                    id: "n2".into(),
                    x: 0.0,
                    y: 6.0,
                    z: 0.0,
                },
                StructuralNode {
                    id: "n3".into(),
                    x: 12.0,
                    y: 6.0,
                    z: 0.0,
                },
                StructuralNode {
                    id: "n4".into(),
                    x: 12.0,
                    y: 0.0,
                    z: 0.0,
                },
            ],
            members: vec![
                StructuralMember {
                    id: "e1".into(),
                    start_node: "n1".into(),
                    end_node: "n2".into(),
                    role: "column".into(),
                    semantic_tags: vec!["primary".into(), "gravity".into(), "lateral".into()],
                    section_id: "unassigned".into(),
                    material_id: "steel".into(),
                },
                StructuralMember {
                    id: "e2".into(),
                    start_node: "n2".into(),
                    end_node: "n3".into(),
                    role: "rafter".into(),
                    semantic_tags: vec!["roof".into(), "primary".into(), "gravity".into()],
                    section_id: "unassigned".into(),
                    material_id: "steel".into(),
                },
                StructuralMember {
                    id: "e3".into(),
                    start_node: "n4".into(),
                    end_node: "n3".into(),
                    role: "column".into(),
                    semantic_tags: vec!["primary".into(), "gravity".into(), "lateral".into()],
                    section_id: "unassigned".into(),
                    material_id: "steel".into(),
                },
            ],
            plates: Vec::new(),
            supports: Vec::new(),
            loads: Vec::new(),
            releases: Vec::new(),
            load_cases: Vec::new(),
            builder_node_materializations: Vec::new(),
        }
    }

    fn add_test_brief_support_locations(project: &mut ProjectFile) {
        project.base_model_brief = Some(BaseModelBrief {
            version: 1,
            session_id: "test".into(),
            current_understanding: String::new(),
            confirmed_intent: Vec::new(),
            open_questions: Vec::new(),
            soft_assumptions: Vec::new(),
            schema_guidance: Vec::new(),
            do_not_decide_yet: Vec::new(),
            visual_intent: BaseModelBriefVisualIntent {
                support_locations: vec![
                    fraia_core::BaseModelBriefSupportLocationIntent {
                        id: "support-n1".into(),
                        target_node: "n1".into(),
                        label: Some("Support location".into()),
                        status: "location_only".into(),
                    },
                    fraia_core::BaseModelBriefSupportLocationIntent {
                        id: "support-n4".into(),
                        target_node: "n4".into(),
                        label: Some("Support location".into()),
                        status: "location_only".into(),
                    },
                ],
                loads: Vec::new(),
            },
            readiness: BaseModelBriefReadiness {
                ready_for_schemas: true,
                unresolved_topics: Vec::new(),
                manual_override_allowed: true,
            },
            updated_at: "test".into(),
        });
    }

    #[test]
    fn load_reply_becomes_structured_add_load_action() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-agent-load-benchmark-{}",
            fraia_core::utils::timestamp_id()
        ));
        let (project, _) =
            create_project(&project_dir, "agent load benchmark").expect("create benchmark project");
        let request = AgentReviewReplyRequest {
            project_dir: project_dir.to_string_lossy().into_owned(),
            comment_id: "missing-gravity-load-member-group-m1".into(),
            comment: json!({
                "title": "Load on member-group-m1",
                "targets": [{ "kind": "member_group", "id": "member-group-m1" }]
            }),
            selected_chips: vec!["5 kN/m line load".into()],
            reply: String::new(),
            messages: Vec::new(),
            model: Some("gpt-5.5".into()),
            reasoning_effort: Some("low".into()),
        };

        let response = local_review_agent(&project, &request, "gpt-5.5", "low");

        assert_eq!(response.status, "ready_to_apply");
        assert_eq!(response.proposed_actions.len(), 1);
        let action = &response.proposed_actions[0];
        assert_eq!(action.action_kind, "add_load");
        assert_eq!(action.target_kind, "member_group");
        assert_eq!(action.target_id, "member-group-m1");
        assert_eq!(action.value["magnitude"]["value"], 5000.0);
        assert_eq!(action.value["magnitude"]["quantityKind"], "line_load");
        assert_eq!(action.value["magnitude"]["canonicalUnit"], "N/m");
        let _ = fs::remove_dir_all(project_dir);
    }

    #[test]
    fn add_load_action_expands_member_group_to_structural_load() {
        let mut model = test_member_model();
        let action = AgentProposedAction {
            action_kind: "add_load".into(),
            target_kind: "member_group".into(),
            target_id: "member-group-m1".into(),
            field: "structural_model.loads".into(),
            value: json!({
                "kind": "uniform_line",
                "magnitude": {
                    "value": 5000.0,
                    "quantityKind": "line_load",
                    "canonicalUnit": "N/m"
                },
                "loadCaseId": "gravity",
                "direction": { "x": 0.0, "y": -1.0, "z": 0.0 }
            }),
            summary: "Add 5.00 kN/m downward gravity line load.".into(),
        };

        let summary =
            apply_agent_action_to_structural_model(&mut model, &action).expect("apply load action");

        assert!(summary.contains("added 1 uniform line load"));
        assert_eq!(model.loads.len(), 1);
        assert_eq!(model.loads[0].magnitude, 5000.0);
        assert_eq!(model.loads[0].load_case_id, "gravity");
        assert_eq!(model.load_cases.len(), 1);
        match &model.loads[0].target {
            AssignmentTargetRef::Member(member_id) => assert_eq!(member_id, "m1"),
            other => panic!("unexpected target {other:?}"),
        }
    }

    #[test]
    fn add_load_action_does_not_infer_members_from_load_case() {
        let mut model = test_portal_frame_model();
        let action = AgentProposedAction {
            action_kind: "add_load".into(),
            target_kind: "load_case".into(),
            target_id: "gravity".into(),
            field: "structural_model.loads".into(),
            value: json!({
                "kind": "uniform_line",
                "magnitude": {
                    "value": 5000.0,
                    "quantityKind": "line_load",
                    "canonicalUnit": "N/m"
                },
                "loadCaseId": "gravity",
                "direction": { "x": 0.0, "y": -1.0, "z": 0.0 }
            }),
            summary: "Add gravity load to load case.".into(),
        };

        let error = apply_agent_action_to_structural_model(&mut model, &action)
            .expect_err("load_case target should require explicit authored targets");

        assert!(
            error
                .to_string()
                .contains("does not infer member targets from load case")
        );
        assert!(model.loads.is_empty());
    }

    #[test]
    fn coordinator_does_not_infer_stable_support_fixity() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-coordinator-supports-{}",
            fraia_core::utils::timestamp_id()
        ));
        let (mut project, _) =
            create_project(&project_dir, "coordinator supports").expect("create project");
        project.intent.building_type = "portal_frame".into();
        project.structural_model = Some(test_portal_frame_model());
        let request = AgentCoordinatorRequest {
            project_dir: project_dir.to_string_lossy().into_owned(),
            instruction: "assign all supports how you recommend so the model is stable".into(),
            review_comments: Vec::new(),
            focus_comment_id: None,
            focus_targets: Vec::new(),
            messages: Vec::new(),
            model: Some("gpt-5.5".into()),
            reasoning_effort: Some("low".into()),
        };

        let response = local_coordinator_agent(&project, &request, "gpt-5.5", "low");

        assert_eq!(response.status, "needs_more_information");
        assert!(response.proposed_actions.is_empty());
        assert!(
            response
                .message
                .contains("explicit support locations and restraint intent")
        );

        let explicit_type_request = AgentCoordinatorRequest {
            project_dir: project_dir.to_string_lossy().into_owned(),
            instruction: "add pinned supports".into(),
            review_comments: Vec::new(),
            focus_comment_id: None,
            focus_targets: Vec::new(),
            messages: Vec::new(),
            model: Some("gpt-5.5".into()),
            reasoning_effort: Some("low".into()),
        };
        let explicit_type_response =
            local_coordinator_agent(&project, &explicit_type_request, "gpt-5.5", "low");
        assert_eq!(explicit_type_response.status, "needs_more_information");
        assert!(explicit_type_response.proposed_actions.is_empty());

        let explicit_target_request = AgentCoordinatorRequest {
            project_dir: project_dir.to_string_lossy().into_owned(),
            instruction: "add pinned supports at the selected nodes".into(),
            review_comments: Vec::new(),
            focus_comment_id: None,
            focus_targets: vec![
                AgentCoordinatorTarget {
                    kind: "node".into(),
                    id: "n1".into(),
                },
                AgentCoordinatorTarget {
                    kind: "node".into(),
                    id: "n4".into(),
                },
            ],
            messages: Vec::new(),
            model: Some("gpt-5.5".into()),
            reasoning_effort: Some("low".into()),
        };
        let explicit_target_response =
            local_coordinator_agent(&project, &explicit_target_request, "gpt-5.5", "low");
        assert_eq!(explicit_target_response.status, "ready_to_apply");
        assert_eq!(explicit_target_response.proposed_actions.len(), 2);
        assert!(
            explicit_target_response
                .proposed_actions
                .iter()
                .all(|action| action.value["supportType"] == "pinned")
        );
        let _ = fs::remove_dir_all(project_dir);
    }

    #[test]
    fn add_support_action_updates_structural_model() {
        let mut model = test_portal_frame_model();
        let action = AgentProposedAction {
            action_kind: "add_support".into(),
            target_kind: "node".into(),
            target_id: "n1".into(),
            field: "structural_model.supports".into(),
            value: json!({
                "supportType": "pinned",
                "ux": true,
                "uy": true,
                "uz": false,
                "rx": false,
                "ry": false,
                "rz": false
            }),
            summary: "Add pinned support at node n1.".into(),
        };

        let summary = apply_agent_action_to_structural_model(&mut model, &action)
            .expect("apply support action");

        assert!(summary.contains("added pinned support"));
        assert_eq!(model.supports.len(), 1);
        assert_eq!(model.supports[0].target_node, "n1");
        assert!(model.supports[0].ux);
        assert!(model.supports[0].uy);
        assert!(!model.supports[0].rz);
    }

    #[test]
    fn coordination_groups_are_derived_from_member_roles_without_system_specific_ids() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-coordination-groups-{}",
            fraia_core::utils::timestamp_id()
        ));
        let (mut project, _) =
            create_project(&project_dir, "coordination groups").expect("create project");
        project.intent.building_type = "portal_frame".into();
        project.structural_model = Some(test_portal_frame_model());
        add_test_brief_support_locations(&mut project);
        let mut draft = planning_draft(&project);
        add_test_design_option_intent(
            &mut draft,
            "fewest-member-sizes",
            "Fewest member sizes",
            "standardised family shortlist before exact catalogue sizing",
            "member-size repetition across justified coordination groups",
            "connection families follow the standardised member family direction",
            "use authored support locations; compare pinned restraint only when locations are explicit",
            vec!["standardisation", "procurement_simplicity"],
        );
        add_test_design_option_intent(
            &mut draft,
            "stiffness-serviceability",
            "Stiffness and serviceability",
            "stiffness-capable families before exact catalogue sizing",
            "repeat stiffness-capable family choices where repeated roles justify it",
            "accepts higher connection/base demand if it improves serviceability comparison",
            "use authored supports, otherwise fixed base restraint as a sensitivity case",
            vec!["stiffness", "serviceability"],
        );
        let report = build_coordination_report(
            &project,
            &draft,
            project.structural_model.as_ref().expect("model"),
        )
        .expect("coordination report");

        let columns = report
            .groups
            .iter()
            .find(|group| group.id == "coord-role-column")
            .expect("column coordination group");
        assert_eq!(columns.member_ids.len(), 2);
        assert!(columns.same_size_preferred);
        assert!(
            columns
                .allowed_section_families
                .iter()
                .any(|family| family == "UC")
        );
        assert!(
            report
                .groups
                .iter()
                .any(|group| group.id == "coord-role-rafter")
        );
        assert!(!report.design_schemes.is_empty());
        assert_eq!(report.design_schemes.len(), 2);
        assert_eq!(
            report
                .design_schemes
                .iter()
                .map(|scheme| scheme.label.as_str())
                .collect::<Vec<_>>(),
            vec!["Fewest member sizes", "Stiffness and serviceability"]
        );
        assert!(report.design_schemes.iter().all(|scheme| {
            scheme.intent.as_ref().is_some_and(|intent| {
                !intent.hypothesis.is_empty() && !intent.provenance.is_empty()
            })
        }));
        assert!(report.design_schemes.iter().all(|scheme| {
            scheme
                .intent
                .as_ref()
                .is_some_and(|intent| !intent.provenance.is_empty())
        }));

        let standardised = report
            .design_schemes
            .iter()
            .find(|scheme| scheme.id == "fewest-member-sizes")
            .expect("standardised comparison scheme");
        assert!(
            standardised
                .group_choices
                .iter()
                .all(|choice| choice.coordination_group_id.starts_with("coord-role-"))
        );
        assert!(
            standardised
                .scene
                .as_ref()
                .expect("scheme scene")
                .members
                .iter()
                .all(|member| {
                    member
                        .allowed_section_families
                        .iter()
                        .any(|family| family == "RHS")
                        && member.family_group_label.as_deref() == Some("Family Group 1")
                        && member.size_group_label.as_deref() == Some("Size Group 1")
                        && member
                            .size_coordination
                            .as_ref()
                            .is_some_and(|coordination| {
                                coordination.kind == "shared"
                                    && coordination.group_label.as_deref() == Some("Size Group 1")
                            })
                        && member.scheme_note.as_deref() == Some("same-size preferred")
                })
        );
        assert!(
            standardised
                .scene
                .as_ref()
                .expect("scheme scene")
                .members
                .iter()
                .all(|member| member.coordination_group_label.is_some())
        );
        let _ = fs::remove_dir_all(project_dir);
    }

    fn test_coordination_group(id: &str, role: &str, member_ids: Vec<&str>) -> CoordinationGroup {
        CoordinationGroup {
            id: id.into(),
            label: id.into(),
            role: role.into(),
            member_group_ids: Vec::new(),
            member_ids: member_ids.into_iter().map(str::to_owned).collect(),
            allowed_section_families: vec!["UB".into(), "UC".into(), "RHS".into()],
            recommended_section_families: vec!["UB".into()],
            section_selection_policy: "lightest_feasible".into(),
            same_size_preferred: false,
            rationale: Vec::new(),
            buildability_notes: Vec::new(),
        }
    }

    #[test]
    fn unresolved_support_strategy_does_not_create_design_scheme() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-unresolved-support-option-{}",
            fraia_core::utils::timestamp_id()
        ));
        let (project, _) =
            create_project(&project_dir, "unresolved support option").expect("create project");
        let mut draft = planning_draft(&project);
        add_test_design_option_intent(
            &mut draft,
            "agent-support-alternatives",
            "Agent support alternatives",
            "catalog families",
            "coordinate where justified",
            "review connection demand downstream",
            "compare support assumptions from Base Model evidence",
            vec!["support_strategy"],
        );
        let model = test_portal_frame_model();
        let groups = vec![test_coordination_group("single", "rafter", vec!["e2"])];

        let schemes = build_design_schemes(&project, &draft, &groups, &model, None);

        assert!(schemes.is_empty());
        let _ = fs::remove_dir_all(project_dir);
    }

    #[test]
    fn design_option_scenes_realize_brief_support_locations() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-brief-support-option-{}",
            fraia_core::utils::timestamp_id()
        ));
        let (mut project, _) =
            create_project(&project_dir, "brief support option").expect("create project");
        project.structural_model = Some(test_portal_frame_model());
        project.base_model_brief = Some(BaseModelBrief {
            version: 1,
            session_id: "test".into(),
            current_understanding: String::new(),
            confirmed_intent: Vec::new(),
            open_questions: Vec::new(),
            soft_assumptions: Vec::new(),
            schema_guidance: Vec::new(),
            do_not_decide_yet: Vec::new(),
            visual_intent: BaseModelBriefVisualIntent {
                support_locations: vec![
                    fraia_core::BaseModelBriefSupportLocationIntent {
                        id: "support-n1".into(),
                        target_node: "n1".into(),
                        label: Some("Support location".into()),
                        status: "location_only".into(),
                    },
                    fraia_core::BaseModelBriefSupportLocationIntent {
                        id: "support-n4".into(),
                        target_node: "n4".into(),
                        label: Some("Support location".into()),
                        status: "location_only".into(),
                    },
                ],
                loads: Vec::new(),
            },
            readiness: BaseModelBriefReadiness {
                ready_for_schemas: true,
                unresolved_topics: Vec::new(),
                manual_override_allowed: true,
            },
            updated_at: "test".into(),
        });
        let mut draft = planning_draft(&project);
        add_test_design_option_intent(
            &mut draft,
            "brief-pinned-option",
            "Brief pinned option",
            "catalog families",
            "coordinate where justified",
            "review connection demand downstream",
            "use authored support locations; compare pinned restraint only when locations are explicit",
            vec!["support_strategy"],
        );

        let report = build_coordination_report(
            &project,
            &draft,
            project.structural_model.as_ref().expect("model"),
        )
        .expect("coordination report");
        let scene = report.design_schemes[0]
            .scene
            .as_ref()
            .expect("scheme scene");

        assert_eq!(scene.supports.len(), 2);
        assert!(
            scene
                .supports
                .iter()
                .any(|support| support.target_node == "n1")
        );
        assert!(
            scene
                .supports
                .iter()
                .any(|support| support.target_node == "n4")
        );
        assert!(
            scene
                .supports
                .iter()
                .all(|support| support.ux && support.uy)
        );
        assert!(!scene.supports.iter().any(|support| support.rz));
        assert!(
            !report.design_schemes[0]
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "design_option.restraint_unsupported")
        );
        let _ = fs::remove_dir_all(project_dir);
    }

    #[test]
    fn holistic_checker_flags_unjustified_fixed_support_changes() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-holistic-support-review-{}",
            fraia_core::utils::timestamp_id()
        ));
        let (mut project, _) =
            create_project(&project_dir, "holistic support review").expect("create project");
        project.structural_model = Some(test_portal_frame_model());
        add_test_brief_support_locations(&mut project);
        let mut draft = planning_draft(&project);
        add_test_design_option_intent(
            &mut draft,
            "haunch-with-fixed-base",
            "Haunch with fixed base",
            "stiffness-capable catalog families",
            "coordinate where justified",
            "review haunch connection demand",
            "fixed restraint at confirmed support-location nodes",
            vec!["serviceability"],
        );
        let model = project.structural_model.as_ref().expect("model");
        let groups = vec![test_coordination_group("single", "rafter", vec!["e2"])];
        let schemes = build_design_schemes(&project, &draft, &groups, model, None);
        let fixed = schemes
            .iter()
            .find(|scheme| scheme.id == "haunch-with-fixed-base")
            .expect("fixed scheme");
        assert!(fixed.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "design_option.fixed_support_holistic_review"
        }));
        let _ = fs::remove_dir_all(project_dir);
    }

    #[test]
    fn generated_design_option_intents_reject_roller_supports() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-roller-intent-rejected-{}",
            fraia_core::utils::timestamp_id()
        ));
        let (mut project, _) =
            create_project(&project_dir, "roller intent rejected").expect("create project");
        project.structural_model = Some(test_portal_frame_model());
        add_test_brief_support_locations(&mut project);
        let mut draft = planning_draft(&project);
        add_test_design_option_intent(
            &mut draft,
            "bracing-with-roller",
            "Bracing with roller",
            "closed-section catalog families",
            "coordinate where justified",
            "review brace gussets",
            "pinned/roller restraint at confirmed support-location nodes",
            vec!["bracing"],
        );
        let intents = authored_design_option_intents(&draft);
        let error = validate_design_option_supports_are_realizable(&project, &intents)
            .expect_err("roller support design options should be rejected");

        assert!(
            error
                .to_string()
                .contains("Roller supports are allowed as low-level support primitives")
        );
        let _ = fs::remove_dir_all(project_dir);
    }

    #[test]
    fn option_reviewer_flags_weak_justification_as_wiki_research_feedback() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-option-reviewer-knowledge-gap-{}",
            fraia_core::utils::timestamp_id()
        ));
        let (mut project, _) =
            create_project(&project_dir, "option reviewer knowledge gap").expect("create project");
        project.structural_model = Some(test_portal_frame_model());
        add_test_brief_support_locations(&mut project);
        let mut draft = planning_draft(&project);
        add_test_design_option_intent(
            &mut draft,
            "bracing-with-generic-provenance",
            "Bracing with generic provenance",
            "closed-section catalog families",
            "coordinate where justified",
            "review brace gussets",
            "pinned restraint at confirmed support-location nodes",
            vec!["bracing"],
        );
        let model = project.structural_model.as_ref().expect("model");
        let groups = vec![test_coordination_group("single", "rafter", vec!["e2"])];

        let schemes = build_design_schemes(&project, &draft, &groups, model, None);
        let scheme = schemes
            .iter()
            .find(|scheme| scheme.id == "bracing-with-generic-provenance")
            .expect("scheme");

        assert!(scheme.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "design_option.justification_review_compressed"
        }));
        assert!(scheme.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "design_option.knowledge_gap_candidate"
                && diagnostic
                    .detail
                    .as_deref()
                    .is_some_and(|detail| detail.contains("bracing/stability system"))
        }));
        let _ = fs::remove_dir_all(project_dir);
    }

    fn add_test_design_option_intent(
        draft: &mut CorePlanningDraft,
        id: &str,
        label: &str,
        section_family_policy: &str,
        standardisation_strategy: &str,
        connection_strategy: &str,
        support_strategy: &str,
        objective_tags: Vec<&str>,
    ) {
        let entry = draft
            .system_parameters
            .entry("designOptionIntents".into())
            .or_insert_with(|| json!([]));
        let intents = entry.as_array_mut().expect("designOptionIntents array");
        intents.push(json!({
            "id": id,
            "label": label,
            "hypothesis": format!("{label} is a test-authored design option intent."),
            "explorationBand": "test concept option",
            "objectiveTags": objective_tags,
            "standardisationStrategy": standardisation_strategy,
            "connectionStrategy": connection_strategy,
            "supportStrategy": support_strategy,
            "sectionFamilyPolicy": section_family_policy,
            "coordinationGroupPolicy": "use authored coordination groups",
            "assumptions": ["Exact section IDs remain downstream."],
            "provenance": [wiki_grounded_test_provenance()]
        }));
    }

    fn wiki_grounded_test_provenance() -> &'static str {
        "Structural engineering judgement informs the support/restraint choice, load path and stability concept, section-family member policy, coordination/standardisation group policy, and connection/detailing consequence against project evidence."
    }

    fn write_test_design_option_analysis_run(project_dir: &Path, run_id: &str, option_id: &str) {
        let result = DesignOptionCandidateAnalysisResult {
            option_id: option_id.into(),
            option_label: option_id.into(),
            coordination_group_id: "group-1".into(),
            section_id: "section-1".into(),
            status: "completed".into(),
            passed: Some(true),
            selected_candidate: true,
            approximate_mass_kg: Some(100.0),
            max_utilization: Some(0.5),
            max_stress_mpa: Some(50.0),
            max_moment_knm: Some(10.0),
            max_shear_kn: Some(5.0),
            max_deflection_mm: Some(2.0),
            max_drift_mm: Some(1.0),
            max_reaction_kn: Some(20.0),
            governing_member_id: Some("member-1".into()),
            governing_combo_id: Some("combo-1".into()),
            diagnostic: None,
        };
        let run_dir = project_dir.join("runs").join(run_id);
        fs::create_dir_all(&run_dir).expect("analysis run directory");
        fraia_core::utils::write_json(
            &run_dir.join("comparison.json"),
            &DesignOptionAnalysisComparison {
                run_id: run_id.into(),
                option_results: vec![DesignOptionAnalysisOptionResult {
                    option_id: option_id.into(),
                    option_label: option_id.into(),
                    lifecycle_status: "active".into(),
                    selected_result: Some(result.clone()),
                    candidate_results: vec![result],
                    diagnostics: Vec::new(),
                }],
            },
        )
        .expect("comparison artifact");
    }

    #[test]
    fn design_option_analysis_persists_candidate_solver_results() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-design-option-analysis-{}",
            fraia_core::utils::timestamp_id()
        ));
        let (mut project, _) =
            create_project(&project_dir, "design option analysis").expect("create project");
        project.structural_model = Some(test_portal_frame_model());
        add_test_brief_support_locations(&mut project);
        {
            let brief = project.base_model_brief.as_mut().expect("brief");
            brief.visual_intent.loads = vec![
                fraia_core::BaseModelBriefLoadIntent {
                    id: "self-weight".into(),
                    kind: "self_weight".into(),
                    target: BaseModelBriefLoadTarget {
                        kind: "all_members".into(),
                        member_id: None,
                        node_id: None,
                    },
                    magnitude_n: None,
                    magnitude_n_per_m: None,
                    direction: None,
                },
                fraia_core::BaseModelBriefLoadIntent {
                    id: "roof-line".into(),
                    kind: "uniform_line".into(),
                    target: BaseModelBriefLoadTarget {
                        kind: "member".into(),
                        member_id: Some("e2".into()),
                        node_id: None,
                    },
                    magnitude_n: None,
                    magnitude_n_per_m: Some(20_000.0),
                    direction: Some(BaseModelBriefLoadDirection {
                        kind: "vector".into(),
                        from_node: None,
                        to_node: None,
                        x: Some(0.0),
                        y: Some(-1.0),
                        z: Some(0.0),
                    }),
                },
                fraia_core::BaseModelBriefLoadIntent {
                    id: "lateral".into(),
                    kind: "point".into(),
                    target: BaseModelBriefLoadTarget {
                        kind: "node".into(),
                        member_id: None,
                        node_id: Some("n2".into()),
                    },
                    magnitude_n: Some(50_000.0),
                    magnitude_n_per_m: None,
                    direction: Some(BaseModelBriefLoadDirection {
                        kind: "toward_node".into(),
                        from_node: Some("n2".into()),
                        to_node: Some("n3".into()),
                        x: None,
                        y: None,
                        z: None,
                    }),
                },
            ];
        }
        let mut draft = planning_draft(&project);
        add_test_design_option_intent(
            &mut draft,
            "analysis-pinned-ub",
            "Pinned UB option",
            "UB member section-family policy with support/restraint review",
            "standardise members by coordination group",
            "beam-column connection detailing remains a review item",
            "pinned restraint at confirmed support-location nodes",
            vec!["preliminary_strength"],
        );
        project.planning_draft = Some(draft);

        let fake_ccx = project_dir.join("fake-ccx");
        fs::write(
            &fake_ccx,
            r#"#!/bin/sh
job=""
while [ "$#" -gt 0 ]; do
  if [ "$1" = "-i" ]; then
    shift
    job="$1"
  fi
  shift
done
cat > "${job}.dat" <<'EOF'
 displacements (vx,vy,vz) for set NALL and time  0.1000000E+01

         1  0.000000E+00  0.000000E+00  0.000000E+00
         2  1.000000E-04 -2.500000E-03  0.000000E+00
         3  8.000000E-05 -3.000000E-03  0.000000E+00
         4  0.000000E+00  0.000000E+00  0.000000E+00

 forces (fx,fy,fz) for set SUPPORT_ALL and time  0.1000000E+01

         1 -2.500000E+04  1.100000E+05  0.000000E+00
         4 -2.500000E+04  1.200000E+05  0.000000E+00

 stresses (elem, integ.pnt.,sxx,syy,szz,sxy,sxz,syz) for set EALL and time  0.1000000E+01

         1   1  2.500000E+07  0.000000E+00  0.000000E+00  1.000000E+05  0.000000E+00  0.000000E+00
         2   1  4.200000E+07  0.000000E+00  0.000000E+00  5.000000E+04  0.000000E+00  0.000000E+00
         3   1  3.100000E+07  0.000000E+00  0.000000E+00  2.000000E+05  0.000000E+00  0.000000E+00
EOF
exit 0
"#,
        )
        .expect("write fake ccx");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&fake_ccx).expect("metadata").permissions();
            permissions.set_mode(0o700);
            fs::set_permissions(&fake_ccx, permissions).expect("chmod fake ccx");
        }
        let original_ccx = std::env::var_os("FRAIA_CCX_PATH");
        unsafe {
            std::env::set_var("FRAIA_CCX_PATH", &fake_ccx);
        }
        let run_dir = persist_design_option_analysis_run(
            &project_dir,
            &project,
            &DesignOptionAnalysisRequest {
                project_dir: project_dir.to_string_lossy().into_owned(),
                scope: None,
                candidate_policy: Some("all_candidates".into()),
                check_profile: Some("preliminary_conservative_steel".into()),
            },
        )
        .expect("design option analysis run");
        match original_ccx {
            Some(value) => unsafe {
                std::env::set_var("FRAIA_CCX_PATH", value);
            },
            None => unsafe {
                std::env::remove_var("FRAIA_CCX_PATH");
            },
        }

        assert!(run_dir.join("run.json").exists());
        assert!(run_dir.join("candidate-inputs.json").exists());
        assert!(run_dir.join("solver-results.json").exists());
        assert!(run_dir.join("preliminary-checks.json").exists());
        assert!(run_dir.join("comparison.json").exists());
        let solver_results: Vec<Value> =
            serde_json::from_str(&fs::read_to_string(run_dir.join("solver-results.json")).unwrap())
                .expect("solver results");
        let candidate_inputs: Vec<Value> = serde_json::from_str(
            &fs::read_to_string(run_dir.join("candidate-inputs.json")).unwrap(),
        )
        .expect("candidate inputs");
        assert!(solver_results.iter().any(|result| {
            result
                .get("diagrams")
                .and_then(|diagrams| diagrams.get("members"))
                .and_then(Value::as_array)
                .is_some_and(|members| !members.is_empty())
        }));
        let comparison: DesignOptionAnalysisComparison =
            serde_json::from_str(&fs::read_to_string(run_dir.join("comparison.json")).unwrap())
                .expect("comparison");
        assert_eq!(comparison.option_results.len(), 1);
        let option = &comparison.option_results[0];
        assert!(!option.candidate_results.is_empty());
        assert!(option.candidate_results.iter().any(|result| {
            result.max_utilization.is_some()
                && result.max_stress_mpa.is_some()
                && result.max_moment_knm.is_some()
                && result.max_shear_kn.is_some()
                && result.max_deflection_mm.is_some()
                && result.max_reaction_kn.is_some()
        }));

        let lookup = load_latest_design_option_analysis_lookup(&project_dir, &project)
            .expect("lookup")
            .expect("latest lookup");
        let model = project.structural_model.as_ref().expect("model");
        let report = build_coordination_report_with_analysis(
            &project,
            &planning_draft(&project),
            model,
            Some(&lookup),
        )
        .expect("coordination report");
        assert!(report.design_schemes.iter().any(|scheme| {
            scheme.analysis_summary.is_some()
                && scheme.result_preview.is_some()
                && scheme.group_choices.iter().any(|choice| {
                    choice.candidate_sections.iter().any(|candidate| {
                        candidate.analysis_status.is_some()
                            && candidate.max_moment_knm.is_some()
                            && candidate.max_shear_kn.is_some()
                    })
                })
        }));

        let raw = load_raw_design_option_analysis(&run_dir).expect("raw analysis payload");
        let raw_results = raw
            .get("solverResults")
            .and_then(Value::as_array)
            .expect("raw solver results");
        assert_eq!(raw_results.len(), solver_results.len());
        assert_eq!(raw_results.len(), candidate_inputs.len());
        let raw_candidate = raw_results
            .iter()
            .find(|result| {
                result
                    .get("nodeDisplacements")
                    .and_then(Value::as_array)
                    .is_some_and(|nodes| !nodes.is_empty())
            })
            .expect("raw candidate with extracted solver tables");
        assert!(
            raw_candidate
                .get("optionId")
                .and_then(Value::as_str)
                .is_some_and(|value| !value.is_empty())
        );
        assert!(
            raw_candidate
                .get("coordinationGroupId")
                .and_then(Value::as_str)
                .is_some_and(|value| !value.is_empty())
        );
        assert!(
            raw_candidate
                .get("sectionId")
                .and_then(Value::as_str)
                .is_some_and(|value| !value.is_empty())
        );
        assert!(
            raw_candidate
                .get("compiledInputs")
                .and_then(Value::as_array)
                .is_some_and(|inputs| {
                    inputs.iter().any(|input| {
                        input
                            .get("inputDeck")
                            .or_else(|| input.get("input_deck"))
                            .and_then(Value::as_str)
                            .is_some_and(|deck| deck.contains("*NODE"))
                    })
                })
        );
        assert!(
            raw_candidate
                .get("executions")
                .and_then(Value::as_array)
                .is_some_and(|executions| {
                    executions.iter().any(|execution| {
                        execution
                            .get("command")
                            .and_then(Value::as_array)
                            .is_some_and(|command| !command.is_empty())
                            && execution
                                .get("workingDir")
                                .or_else(|| execution.get("working_dir"))
                                .and_then(Value::as_str)
                                .is_some()
                    })
                })
        );
        assert!(
            raw_candidate
                .get("supportReactions")
                .and_then(Value::as_array)
                .is_some_and(|reactions| !reactions.is_empty())
        );
        assert!(
            raw_candidate
                .get("elementStresses")
                .and_then(Value::as_array)
                .is_some_and(|stresses| !stresses.is_empty())
        );
        assert!(
            raw_candidate
                .get("rawFiles")
                .and_then(Value::as_array)
                .is_some_and(|raw_files| {
                    raw_files.iter().any(|entry| {
                        entry
                            .get("files")
                            .and_then(|files| files.get("dat"))
                            .and_then(Value::as_str)
                            .is_some_and(|dat| dat.contains("displacements"))
                            && entry
                                .get("files")
                                .and_then(|files| files.get("inp"))
                                .and_then(Value::as_str)
                                .is_some_and(|inp| inp.contains("*NODE"))
                    })
                })
        );

        let _ = fs::remove_dir_all(project_dir);
    }

    #[test]
    fn design_scheme_diagnostics_flag_weak_grouping() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-weak-grouping-schemes-{}",
            fraia_core::utils::timestamp_id()
        ));
        let (project, _) =
            create_project(&project_dir, "weak grouping schemes").expect("create project");
        let draft = planning_draft(&project);
        let model = test_portal_frame_model();
        let groups = vec![
            test_coordination_group("left-column", "column", vec!["e1"]),
            test_coordination_group("rafter", "rafter", vec!["e2"]),
            test_coordination_group("right-column", "column", vec!["e3"]),
        ];

        let schemes = build_design_schemes(&project, &draft, &groups, &model, None);

        assert!(schemes.iter().all(|scheme| {
            scheme
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "coordination.weak_grouping")
        }));
        let _ = fs::remove_dir_all(project_dir);
    }

    #[test]
    fn design_scheme_diagnostics_flag_awkward_family_mixes() {
        let ub = section_catalog()
            .into_iter()
            .find(|section| section_family(&section.id) == Some("UB"))
            .expect("UB section")
            .id;
        let chs = section_catalog()
            .into_iter()
            .find(|section| section_family(&section.id) == Some("CHS"))
            .expect("CHS section")
            .id;
        let choices = vec![
            DesignSchemeGroupChoice {
                coordination_group_id: "open".into(),
                allowed_section_families: vec!["UB".into()],
                candidate_section_ids: vec![ub.clone()],
                candidate_sections: Vec::new(),
                unavailable_families: Vec::new(),
                selected_section_id: Some(ub),
                approximate_mass_kg: None,
                check_status: "family_constraints".into(),
                notes: Vec::new(),
            },
            DesignSchemeGroupChoice {
                coordination_group_id: "tube".into(),
                allowed_section_families: vec!["CHS".into()],
                candidate_section_ids: vec![chs.clone()],
                candidate_sections: Vec::new(),
                unavailable_families: Vec::new(),
                selected_section_id: Some(chs),
                approximate_mass_kg: None,
                check_status: "family_constraints".into(),
                notes: Vec::new(),
            },
        ];

        let diagnostics = connection_buildability_diagnostics(&choices);

        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "coordination.connection_review")
        );
    }

    #[test]
    fn design_options_are_not_generated_without_agent_authored_intents() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-contextual-option-shortlist-{}",
            fraia_core::utils::timestamp_id()
        ));
        let (project, _) =
            create_project(&project_dir, "contextual option shortlist").expect("create project");
        let mut draft = planning_draft(&project);
        draft.geometry_and_loads.lateral_load_kn = 0.0;
        let model = test_portal_frame_model();
        let groups = vec![test_coordination_group("single", "beam", vec!["e2"])];

        let fallback = build_design_schemes(&project, &draft, &groups, &model, None);
        assert!(fallback.is_empty());

        draft.system_parameters.insert(
            "designOptionGuidance".into(),
            json!("Connection simplicity matters; avoid detail-family variation."),
        );
        let connection = build_design_schemes(&project, &draft, &groups, &model, None);
        assert!(connection.is_empty());
        let _ = fs::remove_dir_all(project_dir);
    }

    #[test]
    fn authored_design_option_intent_is_realized_deterministically() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-authored-option-intent-{}",
            fraia_core::utils::timestamp_id()
        ));
        let (mut project, _) =
            create_project(&project_dir, "authored option intent").expect("create project");
        add_test_brief_support_locations(&mut project);
        let mut draft = planning_draft(&project);
        draft.system_parameters.insert(
            "designOptionIntents".into(),
            json!([
                {
                    "id": "agent-simple-details",
                    "label": "Agent simple details",
                    "hypothesis": "A compatible closed-section direction is worth exploring because the user prioritised simple connection families.",
                    "explorationBand": "concept-option",
                    "objectiveTags": ["connection_simplicity"],
                    "standardisationStrategy": "section-family repetition",
                    "connectionStrategy": "least variation in connection families",
                    "supportStrategy": "use authored support locations; compare pinned restraint only when locations are explicit",
                    "sectionFamilyPolicy": "closed-section families",
                    "coordinationGroupPolicy": "coordinate groups by detail family",
                    "assumptions": ["Exact section IDs remain downstream."],
                    "provenance": [wiki_grounded_test_provenance()]
                }
            ]),
        );
        let model = test_portal_frame_model();
        let groups = vec![test_coordination_group("single", "beam", vec!["e2"])];

        let schemes = build_design_schemes(&project, &draft, &groups, &model, None);

        assert_eq!(schemes.len(), 1);
        assert_eq!(schemes[0].id, "agent-simple-details");
        assert_eq!(
            schemes[0].intent.as_ref().expect("intent").hypothesis,
            "A compatible closed-section direction is worth exploring because the user prioritised simple connection families."
        );
        assert!(schemes[0].analysis_summary.is_none());
        assert!(
            schemes[0]
                .intent
                .as_ref()
                .expect("intent")
                .provenance
                .iter()
                .any(|item| item.contains("agent-authored DesignOptionIntent"))
        );
        let _ = fs::remove_dir_all(project_dir);
    }

    #[test]
    fn update_planning_draft_action_persists_design_option_intents() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-design-option-intent-action-{}",
            fraia_core::utils::timestamp_id()
        ));
        let (mut project, _) =
            create_project(&project_dir, "design option intent action").expect("create project");
        project.structural_model = Some(test_portal_frame_model());
        add_test_brief_support_locations(&mut project);
        let mut draft = planning_draft(&project);
        let action = AgentProposedAction {
            action_kind: "update_planning_draft".into(),
            target_kind: "coordination".into(),
            target_id: "design-option-intents".into(),
            field: "coordination.designOptionIntents".into(),
            value: json!({
                "designOptionIntents": [{
                    "id": "agent-serviceability-check",
                    "label": "Agent serviceability check",
                    "hypothesis": "A stiffness-led option is worth exploring because drift was raised in the brief.",
                    "explorationBand": "concept-option",
                    "objectiveTags": ["serviceability"],
                    "standardisationStrategy": "repeat families only where coordination groups justify it",
                    "connectionStrategy": "review connection demand downstream",
                    "supportStrategy": "use authored supports, otherwise fixed restraint sensitivity",
                    "sectionFamilyPolicy": "stiffness-capable catalog families",
                    "coordinationGroupPolicy": "use authored coordination groups",
                    "assumptions": ["Exact section IDs remain downstream."],
                    "provenance": [wiki_grounded_test_provenance()]
                }]
            }),
            summary: "Persist agent-authored design option intents.".into(),
        };

        apply_agent_action_to_draft(&project, &mut draft, &action).expect("apply intent action");
        let intents = authored_design_option_intents(&draft);

        assert_eq!(intents.len(), 1);
        assert_eq!(intents[0].id, "agent-serviceability-check");
        let _ = fs::remove_dir_all(project_dir);
    }

    #[test]
    fn design_option_replacement_supersedes_original_and_adds_revision() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-design-option-replacement-{}",
            fraia_core::utils::timestamp_id()
        ));
        let (mut project, _) =
            create_project(&project_dir, "design option replacement").expect("create project");
        project.structural_model = Some(test_portal_frame_model());
        add_test_brief_support_locations(&mut project);
        let mut draft = planning_draft(&project);
        add_test_design_option_intent(
            &mut draft,
            "fixed-base-frame-action",
            "Fixed-base frame action",
            "stiffness-capable catalog families",
            "coordinate member families where frame action justifies it",
            "moment-capable base and beam-column details remain downstream",
            "fixed restraint at confirmed support locations",
            vec!["serviceability", "frame_action"],
        );
        let action = AgentProposedAction {
            action_kind: "update_planning_draft".into(),
            target_kind: "design_option".into(),
            target_id: "fixed-base-frame-action".into(),
            field: "coordination.designOptionReplacement".into(),
            value: json!({
                "supersededOptionId": "fixed-base-frame-action",
                "supersededReason": "The user asked to avoid fixed base restraint in this option chat.",
                "replacementDesignOptionIntent": {
                    "id": "pinned-base-frame-action-revision",
                    "label": "Pinned-base frame action revision",
                    "hypothesis": "A pinned-base revision is worth comparing because it preserves the frame geometry while reducing foundation moment demand.",
                    "explorationBand": "revision of fixed-base-frame-action",
                    "objectiveTags": ["support_strategy", "connection_simplicity"],
                    "standardisationStrategy": "coordinate member families where pinned-base frame action still justifies repetition",
                    "connectionStrategy": "review beam-column continuity while keeping base details nominally pinned",
                    "supportStrategy": "pinned restraint at confirmed support locations",
                    "sectionFamilyPolicy": "stiffness-capable catalog families",
                    "coordinationGroupPolicy": "use authored coordination groups",
                    "coordinationOverrides": [{
                        "memberId": "e2",
                        "familyGroupLabel": "GF2",
                        "designationGroupLabel": "GD1",
                        "note": "user-requested group override"
                    }],
                    "assumptions": ["Exact section IDs remain downstream.", "Revision avoids fixed base restraint requested by the user."],
                    "provenance": [wiki_grounded_test_provenance()]
                }
            }),
            summary: "Create a pinned-base replacement design option.".into(),
        };

        apply_agent_action_to_draft(&project, &mut draft, &action).expect("apply replacement");
        let intents = authored_design_option_intents(&draft);
        let original = intents
            .iter()
            .find(|intent| intent.id == "fixed-base-frame-action")
            .expect("original");
        let replacement = intents
            .iter()
            .find(|intent| intent.id == "pinned-base-frame-action-revision")
            .expect("replacement");

        assert_eq!(original.lifecycle_status.as_deref(), Some("superseded"));
        assert_eq!(
            original.superseded_by.as_deref(),
            Some("pinned-base-frame-action-revision")
        );
        assert_eq!(replacement.lifecycle_status.as_deref(), Some("active"));
        assert_eq!(
            replacement.revision_of.as_deref(),
            Some("fixed-base-frame-action")
        );
        let groups = vec![test_coordination_group("single", "beam", vec!["e2"])];
        let model = project.structural_model.as_ref().expect("model");
        let schemes = build_design_schemes(&project, &draft, &groups, model, None);
        assert!(schemes.iter().any(|scheme| {
            scheme.id == "fixed-base-frame-action"
                && scheme.lifecycle_status.as_deref() == Some("superseded")
        }));
        assert!(schemes.iter().any(|scheme| {
            scheme.id == "pinned-base-frame-action-revision"
                && scheme.revision_of.as_deref() == Some("fixed-base-frame-action")
        }));
        let replacement_scheme = schemes
            .iter()
            .find(|scheme| scheme.id == "pinned-base-frame-action-revision")
            .expect("replacement scheme");
        let overridden_member = replacement_scheme
            .scene
            .as_ref()
            .expect("replacement scene")
            .members
            .iter()
            .find(|member| member.id == "e2")
            .expect("overridden member");
        assert_eq!(
            overridden_member.family_group_label.as_deref(),
            Some("Family Group 2")
        );
        assert_eq!(
            overridden_member.size_group_label.as_deref(),
            Some("Size Group 1")
        );
        assert_eq!(
            overridden_member.scheme_note.as_deref(),
            Some("user-requested group override")
        );
        let _ = fs::remove_dir_all(project_dir);
    }

    #[test]
    fn update_planning_draft_rejects_design_options_without_realizable_supports() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-design-option-intent-no-support-{}",
            fraia_core::utils::timestamp_id()
        ));
        let (mut project, _) =
            create_project(&project_dir, "invalid support option").expect("create project");
        project.structural_model = Some(test_portal_frame_model());
        add_test_brief_support_locations(&mut project);
        let mut draft = planning_draft(&project);
        let action = AgentProposedAction {
            action_kind: "update_planning_draft".into(),
            target_kind: "coordination".into(),
            target_id: "design-option-intents".into(),
            field: "coordination.designOptionIntents".into(),
            value: json!({
                "designOptionIntents": [{
                    "id": "agent-vague-support-option",
                    "label": "Agent vague support option",
                    "hypothesis": "A comparison option is useful only if its support assumption is concrete.",
                    "explorationBand": "concept-option",
                    "objectiveTags": ["support_strategy"],
                    "standardisationStrategy": "coordinate where justified",
                    "connectionStrategy": "review downstream connection demand",
                    "supportStrategy": "compare support assumptions from Base Model evidence",
                    "sectionFamilyPolicy": "catalog families",
                    "coordinationGroupPolicy": "use authored coordination groups",
                    "assumptions": ["Exact section IDs remain downstream."],
                    "provenance": [wiki_grounded_test_provenance()]
                }]
            }),
            summary: "Persist vague design option intent.".into(),
        };

        let error = apply_agent_action_to_draft(&project, &mut draft, &action)
            .expect_err("vague support options should be rejected");

        assert!(
            error
                .to_string()
                .contains("realizable buildable supportStrategy")
        );
        assert!(authored_design_option_intents(&draft).is_empty());
        let _ = fs::remove_dir_all(project_dir);
    }

    #[test]
    fn update_planning_draft_rejects_design_options_without_engineering_provenance() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-design-option-intent-no-wiki-provenance-{}",
            fraia_core::utils::timestamp_id()
        ));
        let (mut project, _) =
            create_project(&project_dir, "invalid provenance option").expect("create project");
        project.structural_model = Some(test_portal_frame_model());
        add_test_brief_support_locations(&mut project);
        let mut draft = planning_draft(&project);
        let action = AgentProposedAction {
            action_kind: "update_planning_draft".into(),
            target_kind: "coordination".into(),
            target_id: "design-option-intents".into(),
            field: "coordination.designOptionIntents".into(),
            value: json!({
                "designOptionIntents": [{
                    "id": "agent-ungrounded-option",
                    "label": "Agent ungrounded option",
                    "hypothesis": "A stiffness-led option is worth exploring because drift was raised in the brief.",
                    "explorationBand": "concept-option",
                    "objectiveTags": ["serviceability"],
                    "standardisationStrategy": "repeat families only where coordination groups justify it",
                    "connectionStrategy": "review connection demand downstream",
                    "supportStrategy": "pinned restraint at confirmed support locations",
                    "sectionFamilyPolicy": "stiffness-capable catalog families",
                    "coordinationGroupPolicy": "use authored coordination groups",
                    "assumptions": ["Exact section IDs remain downstream."],
                    "provenance": ["The agent chose this as a typical option."]
                }]
            }),
            summary: "Persist ungrounded design option intent.".into(),
        };

        let error = apply_agent_action_to_draft(&project, &mut draft, &action)
            .expect_err("ungrounded design-option provenance should be rejected");

        assert!(error.to_string().contains("support/restraint choice"));
        assert!(authored_design_option_intents(&draft).is_empty());
        let _ = fs::remove_dir_all(project_dir);
    }

    #[test]
    fn update_planning_draft_accepts_thematic_agent_provenance_without_magic_words() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-design-option-intent-themed-provenance-{}",
            fraia_core::utils::timestamp_id()
        ));
        let (mut project, _) =
            create_project(&project_dir, "themed provenance option").expect("create project");
        project.structural_model = Some(test_portal_frame_model());
        add_test_brief_support_locations(&mut project);
        let mut draft = planning_draft(&project);
        let action = AgentProposedAction {
            action_kind: "update_planning_draft".into(),
            target_kind: "coordination".into(),
            target_id: "design-option-intents".into(),
            field: "coordination.designOptionIntents".into(),
            value: json!({
                "designOptionIntents": [{
                    "id": "agent-themed-option",
                    "label": "Agent themed option",
                    "hypothesis": "A pinned-base frame is a useful baseline for explicit restraint comparison.",
                    "explorationBand": "concept-option",
                    "objectiveTags": ["baseline", "restraint"],
                    "standardisationStrategy": "coordinate side member families where analysis evidence supports it",
                    "connectionStrategy": "review frame continuity and base connection demand downstream",
                    "supportStrategy": "pinned supports at confirmed support locations",
                    "sectionFamilyPolicy": "steel member families remain solver-informed",
                    "coordinationGroupPolicy": "use authored coordination groups",
                    "assumptions": ["Exact section IDs remain downstream."],
                    "provenance": [
                        "Support/restraint: pinned bases keep foundation moment assumptions lower than fixed-base restraint.",
                        "Load path/stability: frame action through the members provides the stabilising path for the horizontal point load.",
                        "Section-family policy: final member section family selection waits for analysis demand and serviceability evidence.",
                        "Coordination/standardisation: side member grouping keeps comparison focused on restraint effects.",
                        "Connection/detailing consequence: base connection demand remains different from a fixed-base frame and must be reviewed."
                    ]
                }]
            }),
            summary: "Persist themed design option intent.".into(),
        };

        apply_agent_action_to_draft(&project, &mut draft, &action)
            .expect("thematic provenance should not require magic words");
        let intents = authored_design_option_intents(&draft);

        assert_eq!(intents.len(), 1);
        assert_eq!(intents[0].id, "agent-themed-option");
        let _ = fs::remove_dir_all(project_dir);
    }

    #[test]
    fn update_planning_draft_accepts_frame_action_as_stability_evidence() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-design-option-intent-frame-action-provenance-{}",
            fraia_core::utils::timestamp_id()
        ));
        let (mut project, _) =
            create_project(&project_dir, "frame action provenance option").expect("create project");
        project.structural_model = Some(test_portal_frame_model());
        add_test_brief_support_locations(&mut project);
        let mut draft = planning_draft(&project);
        let action = AgentProposedAction {
            action_kind: "update_planning_draft".into(),
            target_kind: "coordination".into(),
            target_id: "design-option-intents".into(),
            field: "coordination.designOptionIntents".into(),
            value: json!({
                "designOptionIntents": [{
                    "id": "agent-frame-action-option",
                    "label": "Agent frame-action option",
                    "hypothesis": "A direct frame action option is the clearest baseline for the confirmed support locations.",
                    "explorationBand": "concept-option",
                    "objectiveTags": ["baseline", "direct-load-path", "frame-action"],
                    "standardisationStrategy": "single-family member coordination",
                    "connectionStrategy": "moment-resisting corner connection detailing remains reviewable downstream",
                    "supportStrategy": "pinned restraint at confirmed support locations",
                    "sectionFamilyPolicy": "one steel member family where sizing evidence supports it",
                    "coordinationGroupPolicy": "use authored coordination groups",
                    "assumptions": ["Frame action is the stabilising concept for this baseline."],
                    "provenance": [
                        "Support/restraint: confirmed support locations are retained as pinned restraints.",
                        "Section-family policy: final member section family selection waits for analysis demand and serviceability evidence.",
                        "Coordination/standardisation: one member grouping keeps the comparison focused.",
                        "Connection/detailing consequence: moment-resisting corner detailing must be reviewed downstream."
                    ]
                }]
            }),
            summary: "Persist frame-action design option intent.".into(),
        };

        apply_agent_action_to_draft(&project, &mut draft, &action)
            .expect("frame action should satisfy load-path/stability evidence");

        assert_eq!(authored_design_option_intents(&draft).len(), 1);
        let _ = fs::remove_dir_all(project_dir);
    }

    #[test]
    fn update_planning_draft_rejects_unjustified_design_option_intents() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-design-option-intent-invalid-{}",
            fraia_core::utils::timestamp_id()
        ));
        let (project, _) =
            create_project(&project_dir, "invalid option intent action").expect("create project");
        let mut draft = planning_draft(&project);
        let action = AgentProposedAction {
            action_kind: "update_planning_draft".into(),
            target_kind: "coordination".into(),
            target_id: "design-option-intents".into(),
            field: "coordination.designOptionIntents".into(),
            value: json!({
                "designOptionIntents": [{
                    "id": "agent-empty-option",
                    "label": "Agent empty option",
                    "hypothesis": "",
                    "explorationBand": "concept-option",
                    "objectiveTags": [],
                    "standardisationStrategy": "coordinate where justified",
                    "connectionStrategy": "review downstream",
                    "supportStrategy": "compare support assumptions",
                    "sectionFamilyPolicy": "catalog families",
                    "coordinationGroupPolicy": "use authored groups",
                    "assumptions": [],
                    "provenance": []
                }]
            }),
            summary: "Persist under-justified design option intent.".into(),
        };

        let error = apply_agent_action_to_draft(&project, &mut draft, &action)
            .expect_err("under-justified intents should be rejected");

        assert!(error.to_string().contains("missing hypothesis"));
        assert!(authored_design_option_intents(&draft).is_empty());
        let _ = fs::remove_dir_all(project_dir);
    }

    #[test]
    fn demo_section_fallback_uses_allowed_catalog_family() {
        let params = BeamPlanningSystemParameters {
            allowed_section_families: Some(vec!["PFC".into()]),
            ..Default::default()
        };

        let section =
            select_demo_section_from_family_preferences(&params).expect("catalog section");

        assert_eq!(section_family(&section), Some("PFC"));
    }

    #[test]
    fn scheme_family_order_uses_intent_and_group_data_not_role_table() {
        let mut group = test_coordination_group("columns", "column", vec!["e1"]);
        group.allowed_section_families = vec!["UB".into(), "PFC".into()];
        group.recommended_section_families = vec!["PFC".into()];
        let candidate = candidate_from_authored_intent(
            DesignOptionIntent {
                id: "agent-open-family-test".into(),
                label: "Agent open family test".into(),
                hypothesis: "Use open section families only if the group data allows them.".into(),
                exploration_band: "test".into(),
                lifecycle_status: None,
                superseded_by: None,
                superseded_reason: None,
                revision_of: None,
                objective_tags: vec!["material_efficiency".into()],
                standardisation_strategy: "use group coordination data".into(),
                connection_strategy: "review connection strategy downstream".into(),
                support_strategy: "pinned restraint at confirmed support locations".into(),
                section_family_policy: "open section families".into(),
                coordination_group_policy: "use authored coordination groups".into(),
                coordination_overrides: Vec::new(),
                assumptions: Vec::new(),
                provenance: vec![wiki_grounded_test_provenance().into()],
            },
            None,
            true,
        )
        .expect("candidate");

        let families = scheme_family_order(&candidate, &group);

        assert_eq!(families, vec!["PFC".to_string(), "UB".to_string()]);
        assert!(!families.iter().any(|family| family == "UC"));
    }

    #[test]
    fn raw_member_schemes_group_section_intent_by_role() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-raw-section-groups-{}",
            fraia_core::utils::timestamp_id()
        ));
        let (mut project, _) =
            create_project(&project_dir, "raw section groups").expect("create project");
        let mut model = test_portal_frame_model();
        for member in &mut model.members {
            member.role = "member".into();
            member.semantic_tags.clear();
            member.material_id = "unassigned".into();
            member.section_id = "unassigned".into();
        }
        project.intent.building_type = "unspecified".into();
        project.structural_model = Some(model);
        add_test_brief_support_locations(&mut project);
        let mut draft = planning_draft(&project);
        add_test_design_option_intent(
            &mut draft,
            "fewest-member-sizes",
            "Fewest member sizes",
            "standardised family shortlist before exact catalogue sizing",
            "member-size repetition across justified coordination groups",
            "connection families follow the standardised member family direction",
            "use authored support locations; compare pinned restraint only when locations are explicit",
            vec!["standardisation", "procurement_simplicity"],
        );
        let report = build_coordination_report(
            &project,
            &draft,
            project.structural_model.as_ref().expect("model"),
        )
        .expect("coordination report");

        let standardised = report
            .design_schemes
            .iter()
            .find(|scheme| scheme.id == "fewest-member-sizes")
            .expect("standardisation intent scheme");
        assert!(
            report
                .design_schemes
                .iter()
                .all(|scheme| scheme.id != "minimum-mass-open-sections")
        );
        assert!(
            standardised
                .scene
                .as_ref()
                .expect("scheme scene")
                .members
                .iter()
                .all(|member| member
                    .size_coordination
                    .as_ref()
                    .is_some_and(|coordination| coordination.kind == "shared"
                        && coordination.group_label.as_deref() == Some("Size Group 1")))
        );
        let _ = fs::remove_dir_all(project_dir);
    }

    #[test]
    fn section_family_reply_targets_coordination_group() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-coordination-reply-{}",
            fraia_core::utils::timestamp_id()
        ));
        let (mut project, _) =
            create_project(&project_dir, "coordination reply").expect("create benchmark project");
        project.structural_model = Some(test_portal_frame_model());
        let request = AgentReviewReplyRequest {
            project_dir: project_dir.to_string_lossy().into_owned(),
            comment_id: "section-family-portal-rafters".into(),
            comment: json!({
                "title": "Section families for portal rafters",
                "targets": [{ "kind": "coordination_group", "id": "portal-rafters" }]
            }),
            selected_chips: vec!["allow UB and PFC".into()],
            reply: String::new(),
            messages: Vec::new(),
            model: Some("gpt-5.5".into()),
            reasoning_effort: Some("low".into()),
        };

        let response = local_review_agent(&project, &request, "gpt-5.5", "low");

        assert_eq!(response.status, "ready_to_apply");
        let action = response.proposed_actions.first().expect("action");
        assert_eq!(action.target_kind, "coordination_group");
        assert_eq!(action.target_id, "coord-role-rafter");
        assert_eq!(action.field, "coordinationGroup.allowedSectionFamilies");
        assert_eq!(
            json_string_array(&action.value, "allowedSectionFamilies"),
            vec!["UB".to_string(), "PFC".to_string()]
        );
        let _ = fs::remove_dir_all(project_dir);
    }

    #[test]
    fn base_model_edit_rejects_duplicate_ids_and_zero_length_members() {
        let mut model = StructuralModel::empty();
        apply_base_model_edit_operation(
            &mut model,
            BaseModelEditOperation::CreateNode {
                id: Some("node.N1".into()),
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        )
        .expect("create node");

        let duplicate = apply_base_model_edit_operation(
            &mut model,
            BaseModelEditOperation::CreateNode {
                id: Some("node.N1".into()),
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
        );
        assert!(duplicate.is_err());

        let zero_length = apply_base_model_edit_operation(
            &mut model,
            BaseModelEditOperation::CreateMember {
                id: Some("member.M1".into()),
                start_node: "node.N1".into(),
                end_node: "node.N1".into(),
                role: Some("beam".into()),
                section_id: None,
                material_id: None,
            },
        );
        assert!(zero_length.is_err());
    }

    #[test]
    fn base_model_edit_request_accepts_frontend_payload_shape() {
        let request: BaseModelEditRequest = serde_json::from_value(json!({
            "projectDir": "/tmp/fraia-edit-test",
            "operations": [
                { "kind": "create_node", "x": 1.0, "y": 2.0, "z": 3.0 },
                { "kind": "create_member", "start_node": "node.N1", "end_node": "node.N2", "role": "beam" },
                { "kind": "add_load", "target_kind": "member", "target_id": "member.M1", "magnitude": 1000.0, "direction_x": 0.0, "direction_y": -1.0, "direction_z": 0.0 }
            ]
        }))
        .expect("frontend payload deserializes");
        assert_eq!(request.operations.len(), 3);
    }

    #[test]
    fn base_model_edit_moves_node_and_rejects_referenced_node_delete_without_cascade() {
        let mut model = StructuralModel::empty();
        apply_base_model_edit_operation(
            &mut model,
            BaseModelEditOperation::CreateNode {
                id: Some("node.N1".into()),
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        )
        .expect("create start node");
        apply_base_model_edit_operation(
            &mut model,
            BaseModelEditOperation::CreateNode {
                id: Some("node.N2".into()),
                x: 4.0,
                y: 0.0,
                z: 0.0,
            },
        )
        .expect("create end node");
        apply_base_model_edit_operation(
            &mut model,
            BaseModelEditOperation::CreateMember {
                id: Some("member.M1".into()),
                start_node: "node.N1".into(),
                end_node: "node.N2".into(),
                role: Some("beam".into()),
                section_id: None,
                material_id: None,
            },
        )
        .expect("create member");

        apply_base_model_edit_operation(
            &mut model,
            BaseModelEditOperation::UpdateNode {
                id: "node.N2".into(),
                x: Some(5.0),
                y: Some(1.0),
                z: Some(0.0),
            },
        )
        .expect("move node");
        let moved = model.node_by_id("node.N2").expect("moved node");
        assert_eq!((moved.x, moved.y, moved.z), (5.0, 1.0, 0.0));

        let delete = apply_base_model_edit_operation(
            &mut model,
            BaseModelEditOperation::DeleteNode {
                id: "node.N2".into(),
                cascade: None,
            },
        );
        assert!(delete.is_err());
        assert!(model.members.iter().any(|member| member.id == "member.M1"));
    }

    #[test]
    fn base_model_edit_delete_node_merges_colinear_members() {
        let mut model = StructuralModel::empty();
        for (id, x) in [("node.N1", 0.0), ("node.N2", 5.0), ("node.N3", 10.0)] {
            apply_base_model_edit_operation(
                &mut model,
                BaseModelEditOperation::CreateNode {
                    id: Some(id.into()),
                    x,
                    y: 0.0,
                    z: 0.0,
                },
            )
            .expect("create node");
        }
        for (id, start_node, end_node) in [
            ("member.M1", "node.N1", "node.N2"),
            ("member.M2", "node.N2", "node.N3"),
        ] {
            apply_base_model_edit_operation(
                &mut model,
                BaseModelEditOperation::CreateMember {
                    id: Some(id.into()),
                    start_node: start_node.into(),
                    end_node: end_node.into(),
                    role: Some("beam".into()),
                    section_id: Some("section.W".into()),
                    material_id: Some("material.steel".into()),
                },
            )
            .expect("create member");
        }

        let message = apply_base_model_edit_operation(
            &mut model,
            BaseModelEditOperation::DeleteNode {
                id: "node.N2".into(),
                cascade: None,
            },
        )
        .expect("delete redundant node");

        assert!(message.contains("deleted unnecessary node node.N2"));
        assert!(model.node_by_id("node.N2").is_none());
        assert_eq!(model.members.len(), 1);
        let member = model
            .members
            .iter()
            .find(|member| member.id == "member.M1")
            .expect("merged member keeps first id");
        assert_eq!(member.start_node, "node.N1");
        assert_eq!(member.end_node, "node.N3");
        assert_eq!(member.role, "beam");
        assert_eq!(member.section_id, "section.W");
        assert_eq!(member.material_id, "material.steel");
        assert!(!model.members.iter().any(|member| member.id == "member.M2"));
    }

    #[test]
    fn base_model_edit_delete_display_numbered_builder_split_node() {
        let mut model = StructuralModel::empty();
        for (id, x, y) in [
            ("builder.frame.review::n2", 0.0, 6.0),
            ("node.N1", 10.0, 6.0),
            ("builder.frame.review::n3", 20.0, 6.0),
        ] {
            apply_base_model_edit_operation(
                &mut model,
                BaseModelEditOperation::CreateNode {
                    id: Some(id.into()),
                    x,
                    y,
                    z: 0.0,
                },
            )
            .expect("create node");
        }
        for (id, start_node, end_node) in [
            ("member.M6", "node.N1", "builder.frame.review::n3"),
            ("member.M7", "builder.frame.review::n2", "node.N1"),
        ] {
            apply_base_model_edit_operation(
                &mut model,
                BaseModelEditOperation::CreateMember {
                    id: Some(id.into()),
                    start_node: start_node.into(),
                    end_node: end_node.into(),
                    role: Some("member".into()),
                    section_id: None,
                    material_id: None,
                },
            )
            .expect("create member");
        }

        let message = apply_base_model_edit_operation(
            &mut model,
            BaseModelEditOperation::DeleteNode {
                id: "node.N1".into(),
                cascade: None,
            },
        )
        .expect("delete visible Node 5 raw id");

        assert!(message.contains("deleted unnecessary node node.N1"));
        assert!(model.node_by_id("node.N1").is_none());
        assert_eq!(model.members.len(), 1);
        let member = model
            .members
            .iter()
            .find(|member| member.id == "member.M6")
            .expect("merged member keeps first id");
        assert_eq!(member.start_node, "builder.frame.review::n2");
        assert_eq!(member.end_node, "builder.frame.review::n3");
    }

    #[test]
    fn base_model_edit_splits_member_and_validates_load_targets() {
        let mut model = StructuralModel::empty();
        for (id, x) in [("node.N1", 0.0), ("node.N2", 10.0)] {
            apply_base_model_edit_operation(
                &mut model,
                BaseModelEditOperation::CreateNode {
                    id: Some(id.into()),
                    x,
                    y: 0.0,
                    z: 0.0,
                },
            )
            .expect("create node");
        }
        apply_base_model_edit_operation(
            &mut model,
            BaseModelEditOperation::CreateMember {
                id: Some("member.M1".into()),
                start_node: "node.N1".into(),
                end_node: "node.N2".into(),
                role: Some("beam".into()),
                section_id: None,
                material_id: None,
            },
        )
        .expect("create member");

        apply_base_model_edit_operation(
            &mut model,
            BaseModelEditOperation::SplitMember {
                id: "member.M1".into(),
                node_id: None,
                x: Some(5.0),
                y: Some(0.0),
                z: Some(0.0),
            },
        )
        .expect("split member");
        assert_eq!(model.members.len(), 2);
        assert!(model.nodes.iter().any(|node| node.id == "node.N3"));

        let invalid_load = apply_base_model_edit_operation(
            &mut model,
            BaseModelEditOperation::AddLoad {
                id: Some("load.L1".into()),
                target_kind: "member".into(),
                target_id: "missing".into(),
                load_case_id: None,
                family: None,
                magnitude: 1000.0,
                direction_x: Some(0.0),
                direction_y: Some(-1.0),
                direction_z: Some(0.0),
            },
        );
        assert!(invalid_load.is_err());
    }

    #[test]
    fn base_model_edit_delete_member_removes_only_newly_free_endpoint_nodes() {
        let mut model = StructuralModel::empty();
        for (id, x) in [("node.N1", 0.0), ("node.N2", 5.0), ("node.N3", 10.0)] {
            apply_base_model_edit_operation(
                &mut model,
                BaseModelEditOperation::CreateNode {
                    id: Some(id.into()),
                    x,
                    y: 0.0,
                    z: 0.0,
                },
            )
            .expect("create node");
        }
        for (id, start_node, end_node) in [
            ("member.M1", "node.N1", "node.N2"),
            ("member.M2", "node.N2", "node.N3"),
        ] {
            apply_base_model_edit_operation(
                &mut model,
                BaseModelEditOperation::CreateMember {
                    id: Some(id.into()),
                    start_node: start_node.into(),
                    end_node: end_node.into(),
                    role: Some("beam".into()),
                    section_id: None,
                    material_id: None,
                },
            )
            .expect("create member");
        }

        apply_base_model_edit_operation(
            &mut model,
            BaseModelEditOperation::DeleteMember {
                id: "member.M2".into(),
            },
        )
        .expect("delete member");

        assert!(model.node_by_id("node.N1").is_some());
        assert!(model.node_by_id("node.N2").is_some());
        assert!(model.node_by_id("node.N3").is_none());
        assert_eq!(model.members.len(), 1);
    }

    #[test]
    fn base_model_edit_delete_member_preserves_supported_endpoint_nodes() {
        let mut model = StructuralModel::empty();
        for (id, x) in [("node.N1", 0.0), ("node.N2", 5.0)] {
            apply_base_model_edit_operation(
                &mut model,
                BaseModelEditOperation::CreateNode {
                    id: Some(id.into()),
                    x,
                    y: 0.0,
                    z: 0.0,
                },
            )
            .expect("create node");
        }
        apply_base_model_edit_operation(
            &mut model,
            BaseModelEditOperation::CreateMember {
                id: Some("member.M1".into()),
                start_node: "node.N1".into(),
                end_node: "node.N2".into(),
                role: Some("beam".into()),
                section_id: None,
                material_id: None,
            },
        )
        .expect("create member");
        apply_base_model_edit_operation(
            &mut model,
            BaseModelEditOperation::AddSupport {
                id: Some("support.S1".into()),
                target_node: "node.N1".into(),
                ux: None,
                uy: None,
                uz: None,
                rx: None,
                ry: None,
                rz: None,
            },
        )
        .expect("add support");

        apply_base_model_edit_operation(
            &mut model,
            BaseModelEditOperation::DeleteMember {
                id: "member.M1".into(),
            },
        )
        .expect("delete member");

        assert!(model.node_by_id("node.N1").is_some());
        assert!(model.node_by_id("node.N2").is_none());
        assert_eq!(model.supports.len(), 1);
        assert_eq!(model.members.len(), 0);
    }

    #[test]
    fn base_model_edit_splits_crossing_members_on_create() {
        let mut model = StructuralModel::empty();
        for (id, x, y) in [
            ("node.N1", 0.0, 0.0),
            ("node.N2", 10.0, 0.0),
            ("node.N3", 5.0, -5.0),
            ("node.N4", 5.0, 5.0),
        ] {
            apply_base_model_edit_operation(
                &mut model,
                BaseModelEditOperation::CreateNode {
                    id: Some(id.into()),
                    x,
                    y,
                    z: 0.0,
                },
            )
            .expect("create node");
        }

        apply_base_model_edit_operation(
            &mut model,
            BaseModelEditOperation::CreateMember {
                id: Some("member.M1".into()),
                start_node: "node.N1".into(),
                end_node: "node.N2".into(),
                role: Some("beam".into()),
                section_id: None,
                material_id: None,
            },
        )
        .expect("create existing member");

        apply_base_model_edit_operation(
            &mut model,
            BaseModelEditOperation::CreateMember {
                id: Some("member.M2".into()),
                start_node: "node.N3".into(),
                end_node: "node.N4".into(),
                role: Some("column".into()),
                section_id: None,
                material_id: None,
            },
        )
        .expect("create crossing member");

        let intersection_node = model
            .nodes
            .iter()
            .find(|node| {
                (node.x - 5.0).abs() <= 1e-9 && node.y.abs() <= 1e-9 && node.z.abs() <= 1e-9
            })
            .expect("intersection node");
        let intersection_id = intersection_node.id.clone();
        assert_eq!(model.nodes.len(), 5);
        assert_eq!(model.members.len(), 4);

        let member_spans = model
            .members
            .iter()
            .map(|member| {
                let mut ids = [member.start_node.as_str(), member.end_node.as_str()];
                ids.sort();
                (ids[0].to_string(), ids[1].to_string())
            })
            .collect::<BTreeSet<_>>();
        for expected_span in [
            ("node.N1".to_string(), intersection_id.clone()),
            ("node.N2".to_string(), intersection_id.clone()),
            ("node.N3".to_string(), intersection_id.clone()),
            ("node.N4".to_string(), intersection_id.clone()),
        ] {
            let mut ids = [expected_span.0.as_str(), expected_span.1.as_str()];
            ids.sort();
            assert!(member_spans.contains(&(ids[0].to_string(), ids[1].to_string())));
        }
    }

    #[test]
    fn base_model_edit_delete_member_cleans_up_obsolete_intersection_split() {
        let mut model = StructuralModel::empty();
        for (id, x, y) in [
            ("node.N1", 0.0, 0.0),
            ("node.N2", 10.0, 0.0),
            ("node.N3", 5.0, -5.0),
            ("node.N4", 5.0, 5.0),
        ] {
            apply_base_model_edit_operation(
                &mut model,
                BaseModelEditOperation::CreateNode {
                    id: Some(id.into()),
                    x,
                    y,
                    z: 0.0,
                },
            )
            .expect("create node");
        }
        apply_base_model_edit_operation(
            &mut model,
            BaseModelEditOperation::CreateMember {
                id: Some("member.M1".into()),
                start_node: "node.N1".into(),
                end_node: "node.N2".into(),
                role: Some("beam".into()),
                section_id: Some("section.W".into()),
                material_id: Some("material.steel".into()),
            },
        )
        .expect("create horizontal member");
        apply_base_model_edit_operation(
            &mut model,
            BaseModelEditOperation::CreateMember {
                id: Some("member.M2".into()),
                start_node: "node.N3".into(),
                end_node: "node.N4".into(),
                role: Some("column".into()),
                section_id: Some("section.W".into()),
                material_id: Some("material.steel".into()),
            },
        )
        .expect("create crossing member");
        let intersection_id = model
            .nodes
            .iter()
            .find(|node| (node.x - 5.0).abs() <= 1e-9 && node.y.abs() <= 1e-9)
            .expect("intersection node")
            .id
            .clone();
        assert!(model.members.iter().any(|member| member.id == "member.M4"));

        apply_base_model_edit_operation(
            &mut model,
            BaseModelEditOperation::DeleteMember {
                id: "member.M2".into(),
            },
        )
        .expect("delete first crossing segment");
        let message = apply_base_model_edit_operation(
            &mut model,
            BaseModelEditOperation::DeleteMember {
                id: "member.M4".into(),
            },
        )
        .expect("delete second crossing segment");

        assert!(message.contains("merged 1 split node"));
        assert!(model.node_by_id(&intersection_id).is_none());
        assert_eq!(model.members.len(), 1);
        let member = model
            .members
            .iter()
            .find(|member| member.id == "member.M1")
            .expect("horizontal member keeps first id");
        assert_eq!(member.start_node, "node.N1");
        assert_eq!(member.end_node, "node.N2");
        assert!(!model.members.iter().any(|member| member.id == "member.M3"));
    }

    #[test]
    fn design_option_batch_defaults_to_included_and_becomes_outdated_after_base_edit() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-design-option-batch-{}",
            fraia_core::utils::timestamp_id()
        ));
        let (mut project, _) = create_project(&project_dir, "decision batch").expect("project");
        let mut draft = planning_draft(&project);
        add_test_design_option_intent(
            &mut draft,
            "option-a",
            "Option A",
            "UB",
            "repeat sections",
            "simple joints",
            "fixed bases",
            vec!["mass"],
        );
        add_test_design_option_intent(
            &mut draft,
            "option-b",
            "Option B",
            "RHS",
            "repeat sections",
            "simple joints",
            "pinned bases",
            vec!["buildability"],
        );
        project.planning_draft = Some(draft);
        create_active_design_option_batch(&mut project);

        let batch = project
            .design_option_decisions
            .batches
            .last()
            .expect("active batch");
        assert_eq!(batch.status, "active");
        assert_eq!(batch.option_revisions.len(), 2);
        assert!(
            batch
                .option_revisions
                .iter()
                .all(|revision| revision.included)
        );

        project.structural_model = Some(test_portal_frame_model());
        refresh_design_option_batch_freshness(&mut project);
        let batch = project
            .design_option_decisions
            .batches
            .last()
            .expect("outdated batch");
        assert_eq!(batch.status, "outdated");
        assert!(
            batch
                .option_revisions
                .iter()
                .all(|revision| revision.analysis_status == "stale")
        );
        let _ = fs::remove_dir_all(project_dir);
    }

    #[test]
    fn comparison_run_records_exact_revision_evidence_and_reads_legacy_runs() {
        let legacy: DesignOptionComparisonRun = serde_json::from_value(json!({
            "runId": "legacy-comparison",
            "createdAt": "2026-01-01T00:00:00Z",
            "optionIds": ["option-a"],
            "objective": "legacy objective",
            "recommendedOptionId": "option-a",
            "explanation": "legacy explanation",
            "limitations": []
        }))
        .expect("legacy comparison without evidence references remains readable");
        assert!(legacy.evidence_references.is_empty());

        let project_dir = std::env::temp_dir().join(format!(
            "fraia-design-option-comparison-provenance-{}",
            fraia_core::utils::timestamp_id()
        ));
        let (mut project, _) =
            create_project(&project_dir, "comparison provenance").expect("project");
        let mut draft = planning_draft(&project);
        for (id, label) in [
            ("option-a", "Option A"),
            ("option-b", "Option B"),
            ("option-c", "Option C"),
        ] {
            add_test_design_option_intent(
                &mut draft,
                id,
                label,
                "UB",
                "repeat sections",
                "simple joints",
                "fixed bases",
                vec!["mass"],
            );
        }
        project.planning_draft = Some(draft);
        create_active_design_option_batch(&mut project);

        let active_batch_id = project
            .design_option_decisions
            .active_batch_id
            .clone()
            .expect("active batch id");
        let batch = project
            .design_option_decisions
            .batches
            .iter_mut()
            .find(|batch| batch.id == active_batch_id)
            .expect("active batch");
        for revision in &mut batch.option_revisions {
            revision.analysis_status = "current".into();
            revision.latest_analysis_run_id = Some(format!("analysis-{}", revision.option_id));
            if revision.option_id == "option-c" {
                revision.included = false;
            }
        }
        let expected_references = batch
            .option_revisions
            .iter()
            .filter(|revision| revision.included)
            .map(|revision| DesignOptionComparisonEvidenceReference {
                option_revision_id: revision.revision_id.clone(),
                analysis_run_id: revision
                    .latest_analysis_run_id
                    .clone()
                    .expect("analysis run"),
            })
            .collect::<Vec<_>>();

        record_design_option_comparison_run(&mut project, "comparison-current", &[], &[]);

        let comparison = project
            .design_option_decisions
            .batches
            .iter()
            .find(|batch| batch.id == active_batch_id)
            .and_then(|batch| batch.comparison_runs.last())
            .expect("recorded comparison");
        assert_eq!(
            comparison.option_ids,
            vec!["option-a".to_string(), "option-b".to_string()]
        );
        assert_eq!(comparison.evidence_references, expected_references);
        assert!(
            comparison
                .evidence_references
                .iter()
                .all(|reference| { reference.option_revision_id.starts_with(&active_batch_id) })
        );

        let _ = fs::remove_dir_all(project_dir);
    }

    #[test]
    fn legacy_revision_references_are_backfilled_to_stable_batch_identity() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-design-option-identity-migration-{}",
            fraia_core::utils::timestamp_id()
        ));
        let (mut project, _) = create_project(&project_dir, "identity migration").expect("project");
        let mut draft = planning_draft(&project);
        add_test_design_option_intent(
            &mut draft,
            "option-a",
            "Option A",
            "UB",
            "repeat sections",
            "simple joints",
            "fixed bases",
            vec!["mass"],
        );
        project.planning_draft = Some(draft);
        create_active_design_option_batch(&mut project);
        let batch = project
            .design_option_decisions
            .batches
            .last_mut()
            .expect("batch");
        let revision = batch.option_revisions.first_mut().expect("revision");
        revision.revision_id.clear();
        revision.analysis_status = "current".into();
        revision.latest_analysis_run_id = Some("analysis-old".into());
        batch.comparison_runs.push(DesignOptionComparisonRun {
            run_id: "comparison-old".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
            option_ids: vec!["option-a".into()],
            evidence_references: vec![DesignOptionComparisonEvidenceReference {
                option_revision_id: "option-a".into(),
                analysis_run_id: "analysis-old".into(),
            }],
            objective: "legacy objective".into(),
            recommended_option_id: Some("option-a".into()),
            explanation: "legacy comparison".into(),
            limitations: Vec::new(),
        });
        project
            .design_option_decisions
            .development_paths
            .push(DevelopmentPath {
                id: "path-old".into(),
                option_id: "option-a".into(),
                option_revision_id: "option-a".into(),
                status: "active".into(),
                created_at: "2026-01-01T00:00:00Z".into(),
                updated_at: "2026-01-01T00:00:00Z".into(),
                source_analysis_run_id: Some("analysis-old".into()),
            });

        ensure_design_option_revision_identities(&mut project);

        let batch = project
            .design_option_decisions
            .batches
            .last()
            .expect("batch");
        let revision_id = &batch.option_revisions[0].revision_id;
        assert!(!revision_id.is_empty());
        assert!(revision_id.starts_with(&batch.id));
        assert_eq!(
            batch.comparison_runs[0].evidence_references[0].option_revision_id,
            *revision_id
        );
        assert_eq!(
            project.design_option_decisions.development_paths[0].option_revision_id,
            *revision_id
        );
        let _ = fs::remove_dir_all(project_dir);
    }

    #[tokio::test]
    async fn same_option_id_regeneration_does_not_reuse_evidence_or_development_path() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-design-option-regeneration-identity-{}",
            fraia_core::utils::timestamp_id()
        ));
        let (mut project, _) =
            create_project(&project_dir, "regeneration identity").expect("project");
        let mut draft = planning_draft(&project);
        add_test_design_option_intent(
            &mut draft,
            "reused-option-id",
            "Reused option",
            "UB",
            "repeat sections",
            "simple joints",
            "fixed bases",
            vec!["mass"],
        );
        project.planning_draft = Some(draft);
        create_active_design_option_batch(&mut project);
        let old_batch_id = project
            .design_option_decisions
            .active_batch_id
            .clone()
            .expect("old batch");
        let old_revision_id = {
            let revision = project
                .design_option_decisions
                .batches
                .last_mut()
                .and_then(|batch| batch.option_revisions.first_mut())
                .expect("old revision");
            revision.analysis_status = "current".into();
            revision.latest_analysis_run_id = Some("design-option-analysis-old".into());
            revision.revision_id.clone()
        };
        write_test_design_option_analysis_run(
            &project_dir,
            "design-option-analysis-old",
            "reused-option-id",
        );
        assert!(
            load_latest_design_option_analysis_lookup(&project_dir, &project)
                .expect("old lookup")
                .is_some()
        );
        project
            .design_option_decisions
            .development_paths
            .push(DevelopmentPath {
                id: "development-old".into(),
                option_id: "reused-option-id".into(),
                option_revision_id: old_revision_id.clone(),
                status: "active".into(),
                created_at: "2026-01-01T00:00:00Z".into(),
                updated_at: "2026-01-01T00:00:00Z".into(),
                source_analysis_run_id: Some("design-option-analysis-old".into()),
            });
        project.design_option_decisions.active_development_path_id = Some("development-old".into());

        archive_active_design_option_batch(&mut project);
        create_active_design_option_batch(&mut project);
        let new_batch_id = project
            .design_option_decisions
            .active_batch_id
            .clone()
            .expect("new batch");
        let new_revision_id = project
            .design_option_decisions
            .batches
            .last()
            .and_then(|batch| batch.option_revisions.first())
            .expect("new revision")
            .revision_id
            .clone();
        assert_ne!(new_batch_id, old_batch_id);
        assert_ne!(new_revision_id, old_revision_id);
        assert!(
            load_latest_design_option_analysis_lookup(&project_dir, &project)
                .expect("new batch lookup")
                .is_none(),
            "the new batch must not inherit same-ID evidence from the archived batch"
        );

        {
            let revision = project
                .design_option_decisions
                .batches
                .last_mut()
                .and_then(|batch| batch.option_revisions.first_mut())
                .expect("new revision");
            revision.analysis_status = "current".into();
            revision.latest_analysis_run_id = Some("design-option-analysis-new".into());
        }
        write_test_design_option_analysis_run(
            &project_dir,
            "design-option-analysis-new",
            "reused-option-id",
        );
        save_project(&project_dir, &project).expect("save project");

        let Json(develop_response) =
            design_option_decision_handler(Json(DesignOptionDecisionUpdateRequest {
                project_dir: project_dir.to_string_lossy().into_owned(),
                action: "develop".into(),
                option_id: Some("reused-option-id".into()),
                included: None,
            }))
            .await
            .expect("develop regenerated option");
        let decisions = develop_response.state.design_option_decisions;
        assert_eq!(decisions.development_paths.len(), 2);
        assert!(decisions.development_paths.iter().any(|path| {
            path.option_revision_id == old_revision_id
                && path.source_analysis_run_id.as_deref() == Some("design-option-analysis-old")
        }));
        assert!(decisions.development_paths.iter().any(|path| {
            path.option_revision_id == new_revision_id
                && path.source_analysis_run_id.as_deref() == Some("design-option-analysis-new")
        }));

        let Json(refresh_response) =
            design_option_decision_handler(Json(DesignOptionDecisionUpdateRequest {
                project_dir: project_dir.to_string_lossy().into_owned(),
                action: "refresh_comparison".into(),
                option_id: None,
                included: None,
            }))
            .await
            .expect("refresh comparison without rerunning analysis");
        let comparison = refresh_response
            .state
            .design_option_decisions
            .batches
            .iter()
            .find(|batch| batch.id == new_batch_id)
            .and_then(|batch| batch.comparison_runs.last())
            .expect("refreshed comparison");
        assert_eq!(comparison.option_ids, vec!["reused-option-id"]);
        assert_eq!(
            comparison.evidence_references,
            vec![DesignOptionComparisonEvidenceReference {
                option_revision_id: new_revision_id,
                analysis_run_id: "design-option-analysis-new".into(),
            }]
        );
        let _ = fs::remove_dir_all(project_dir);
    }

    #[test]
    fn replacement_revision_inherits_inclusion_and_supersedes_original_in_comparison() {
        let project_dir = std::env::temp_dir().join(format!(
            "fraia-design-option-revision-{}",
            fraia_core::utils::timestamp_id()
        ));
        let (mut project, _) = create_project(&project_dir, "decision revision").expect("project");
        let mut draft = planning_draft(&project);
        add_test_design_option_intent(
            &mut draft,
            "option-original",
            "Original",
            "UB",
            "repeat sections",
            "simple joints",
            "fixed bases",
            vec!["mass"],
        );
        project.planning_draft = Some(draft.clone());
        create_active_design_option_batch(&mut project);

        let intents = draft
            .system_parameters
            .get_mut("designOptionIntents")
            .and_then(Value::as_array_mut)
            .expect("intents");
        intents[0]["lifecycleStatus"] = json!("superseded");
        intents[0]["supersededBy"] = json!("option-revision");
        let mut replacement = intents[0].clone();
        replacement["id"] = json!("option-revision");
        replacement["label"] = json!("Revised option");
        replacement["lifecycleStatus"] = json!("active");
        replacement["revisionOf"] = json!("option-original");
        replacement["supersededBy"] = Value::Null;
        intents.push(replacement);
        project.planning_draft = Some(draft);

        sync_active_design_option_revisions(&mut project);
        let batch = project
            .design_option_decisions
            .batches
            .last()
            .expect("active batch");
        let original = batch
            .option_revisions
            .iter()
            .find(|revision| revision.option_id == "option-original")
            .expect("original");
        let revision = batch
            .option_revisions
            .iter()
            .find(|revision| revision.option_id == "option-revision")
            .expect("revision");
        assert!(!original.included);
        assert_eq!(original.analysis_status, "superseded");
        assert!(revision.included);
        assert_eq!(revision.revision_of.as_deref(), Some("option-original"));
        assert_eq!(revision.analysis_status, "not_run");
        let _ = fs::remove_dir_all(project_dir);
    }
}
