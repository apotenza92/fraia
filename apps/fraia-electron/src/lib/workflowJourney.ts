import {
  activeBatchFrom,
  activeDevelopmentPathFrom,
  latestComparisonFrom,
  optionRevisions,
} from './designOptionDecisions';
import type {
  DesignOptionRevisionState,
  DevelopmentPathState,
  WorkbenchState,
} from './types';

export const WORKFLOW_STAGES = ['base', 'options', 'analysis'] as const;

export type WorkflowStage = (typeof WORKFLOW_STAGES)[number];

export const WORKFLOW_STAGE_LABELS: Record<WorkflowStage, string> = {
  base: 'Base Model',
  options: 'Design Options',
  analysis: 'Analysis & Comparison',
};

export const WORKFLOW_GATE_REASONS = {
  generateOptions: 'Generate options from the Base Model to continue.',
  regenerateOptions: 'The Base Model changed. Regenerate options to continue.',
  includeOption: 'Include at least one design option for analysis to continue.',
} as const;

export type WorkflowStageState = {
  stage: WorkflowStage;
  label: string;
  available: boolean;
  completed: boolean;
  current: boolean;
  gated: boolean;
  gateReason: string | null;
};

export type WorkflowJourney = {
  currentStage: WorkflowStage;
  stages: WorkflowStageState[];
  previousStage: WorkflowStage | null;
  nextStage: WorkflowStage | null;
  nextGateReason: string | null;
  activeBatchId: string | null;
  activePathId: string | null;
  includedOptionIds: string[];
  missingAnalysisOptionIds: string[];
  staleAnalysisOptionIds: string[];
  missingOrStaleOptionIds: string[];
  hasExactCurrentAnalysis: boolean;
  hasEligibleActivePath: boolean;
};

function revisionOptionId(revision: DesignOptionRevisionState): string {
  return revision.optionId ?? revision.option_id ?? '';
}

function revisionIdentity(revision: DesignOptionRevisionState): string {
  return revision.revisionId ?? revision.revision_id ?? revisionOptionId(revision);
}

function revisionAnalysisStatus(revision: DesignOptionRevisionState): string {
  return revision.analysisStatus ?? revision.analysis_status ?? 'not_run';
}

function revisionAnalysisRunId(revision: DesignOptionRevisionState): string | null {
  return revision.latestAnalysisRunId ?? revision.latest_analysis_run_id ?? null;
}

function pathRevisionId(path: DevelopmentPathState): string {
  return path.optionRevisionId ?? path.option_revision_id ?? '';
}

function pathAnalysisRunId(path: DevelopmentPathState): string | null {
  return path.sourceAnalysisRunId ?? path.source_analysis_run_id ?? null;
}

function uniqueNonEmpty(values: string[]): string[] {
  return values.filter((value, index) => Boolean(value) && values.indexOf(value) === index);
}

function hasCurrentEvidence(revision: DesignOptionRevisionState): boolean {
  return revisionAnalysisStatus(revision) === 'current' && Boolean(revisionAnalysisRunId(revision));
}

function sameOptionSet(left: string[], right: string[]): boolean {
  const leftSet = new Set(left);
  const rightSet = new Set(right);
  if (leftSet.size !== left.length || rightSet.size !== right.length || leftSet.size !== rightSet.size) return false;
  return [...leftSet].every((value) => rightSet.has(value));
}

function activeBatchIsCurrent(state: WorkbenchState | null | undefined): boolean {
  const batch = activeBatchFrom(state);
  return batch?.status === 'active' && optionRevisions(batch).length > 0;
}

function includedRevisions(state: WorkbenchState | null | undefined): DesignOptionRevisionState[] {
  const batch = activeBatchFrom(state);
  if (batch?.status !== 'active') return [];
  return optionRevisions(batch).filter((revision) => revision.included && Boolean(revisionOptionId(revision)));
}

function exactCurrentAnalysis(state: WorkbenchState | null | undefined): boolean {
  const included = includedRevisions(state);
  if (!included.length || !included.every(hasCurrentEvidence)) return false;

  const comparison = latestComparisonFrom(state);
  const comparisonOptionIds = comparison?.optionIds ?? comparison?.option_ids ?? [];
  const evidenceReferences = comparison?.evidenceReferences ?? comparison?.evidence_references ?? [];
  if (!comparison || !sameOptionSet(comparisonOptionIds, included.map(revisionOptionId))) return false;
  if (evidenceReferences.length !== included.length) return false;
  return included.every((revision) => {
    const optionId = revisionOptionId(revision);
    const revisionId = revisionIdentity(revision);
    const analysisRunId = revisionAnalysisRunId(revision);
    return evidenceReferences.some((reference) => (
      (reference.optionRevisionId ?? reference.option_revision_id) === revisionId
      && (reference.analysisRunId ?? reference.analysis_run_id) === analysisRunId
    ));
  });
}

function eligibleActivePath(state: WorkbenchState | null | undefined): boolean {
  const path = activeDevelopmentPathFrom(state);
  if (!path) return false;

  const revision = includedRevisions(state).find((candidate) => revisionIdentity(candidate) === pathRevisionId(path));
  const sourceRunId = pathAnalysisRunId(path);
  return Boolean(
    revision
      && hasCurrentEvidence(revision)
      && sourceRunId
      && sourceRunId === revisionAnalysisRunId(revision),
  );
}

export function initialWorkflowStage(state: WorkbenchState | null | undefined): WorkflowStage {
  if (eligibleActivePath(state) || exactCurrentAnalysis(state)) return 'analysis';
  if (activeBatchIsCurrent(state)) return 'options';
  return 'base';
}

export function runtimeWorkflowStage(
  requestedStage: WorkflowStage,
  state: WorkbenchState | null | undefined,
): WorkflowStage {
  if (requestedStage === 'base') return 'base';
  if (!activeBatchIsCurrent(state)) return 'base';
  if (requestedStage === 'analysis' && !includedRevisions(state).length) return 'options';
  return requestedStage;
}

export function workflowJourneyFrom(
  state: WorkbenchState | null | undefined,
  requestedStage?: WorkflowStage,
): WorkflowJourney {
  const currentStage = requestedStage == null
    ? initialWorkflowStage(state)
    : runtimeWorkflowStage(requestedStage, state);
  const batch = activeBatchFrom(state);
  const batchIsCurrent = activeBatchIsCurrent(state);
  const included = includedRevisions(state);
  const includedOptionIds = uniqueNonEmpty(included.map(revisionOptionId));
  const missingAnalysisOptionIds = uniqueNonEmpty(
    included
      .filter((revision) => revisionAnalysisStatus(revision) !== 'stale' && !hasCurrentEvidence(revision))
      .map(revisionOptionId),
  );
  const staleAnalysisOptionIds = uniqueNonEmpty(
    included
      .filter((revision) => revisionAnalysisStatus(revision) === 'stale')
      .map(revisionOptionId),
  );
  const missingOrStaleOptionIds = uniqueNonEmpty(
    included
      .filter((revision) => !hasCurrentEvidence(revision))
      .map(revisionOptionId),
  );
  const hasExactCurrentAnalysis = exactCurrentAnalysis(state);
  const hasEligibleActivePath = eligibleActivePath(state);

  const optionsGateReason = batchIsCurrent
    ? null
    : batch
      ? WORKFLOW_GATE_REASONS.regenerateOptions
      : WORKFLOW_GATE_REASONS.generateOptions;
  const analysisGateReason = optionsGateReason
    ?? (included.length ? null : WORKFLOW_GATE_REASONS.includeOption);

  const availability: Record<WorkflowStage, boolean> = {
    base: true,
    options: batchIsCurrent,
    analysis: batchIsCurrent && included.length > 0,
  };
  const completion: Record<WorkflowStage, boolean> = {
    base: batchIsCurrent,
    options: batchIsCurrent && included.length > 0,
    analysis: hasExactCurrentAnalysis,
  };
  const gateReasons: Record<WorkflowStage, string | null> = {
    base: null,
    options: optionsGateReason,
    analysis: analysisGateReason,
  };
  const stages = WORKFLOW_STAGES.map((stage): WorkflowStageState => ({
    stage,
    label: WORKFLOW_STAGE_LABELS[stage],
    available: availability[stage],
    completed: completion[stage],
    current: stage === currentStage,
    gated: !availability[stage],
    gateReason: gateReasons[stage],
  }));

  const currentIndex = WORKFLOW_STAGES.indexOf(currentStage);
  const previousStage = currentIndex > 0 ? WORKFLOW_STAGES[currentIndex - 1] : null;
  const nextStage = currentIndex < WORKFLOW_STAGES.length - 1 ? WORKFLOW_STAGES[currentIndex + 1] : null;

  return {
    currentStage,
    stages,
    previousStage,
    nextStage,
    nextGateReason: nextStage ? gateReasons[nextStage] : null,
    activeBatchId: batch?.id ?? null,
    activePathId: activeDevelopmentPathFrom(state)?.id ?? null,
    includedOptionIds,
    missingAnalysisOptionIds,
    staleAnalysisOptionIds,
    missingOrStaleOptionIds,
    hasExactCurrentAnalysis,
    hasEligibleActivePath,
  };
}
