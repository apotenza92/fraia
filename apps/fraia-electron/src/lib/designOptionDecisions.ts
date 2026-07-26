import type {
  DesignOptionBatchState,
  DesignOptionComparisonRunState,
  DesignOptionDecisionState,
  DesignOptionRevisionState,
  DevelopmentPathState,
  WorkbenchState,
} from './types';

const EMPTY_DECISIONS: DesignOptionDecisionState = {
  batches: [],
  developmentPaths: [],
};

export function decisionStateFrom(state: WorkbenchState | null | undefined): DesignOptionDecisionState {
  const decisions = state?.designOptionDecisions ?? state?.design_option_decisions;
  if (!decisions) return EMPTY_DECISIONS;
  return {
    ...decisions,
    batches: decisions.batches ?? [],
    developmentPaths: decisions.developmentPaths ?? decisions.development_paths ?? [],
  };
}

export function activeBatchFrom(state: WorkbenchState | null | undefined): DesignOptionBatchState | null {
  const decisions = decisionStateFrom(state);
  const activeId = decisions.activeBatchId ?? decisions.active_batch_id;
  return decisions.batches.find((batch) => batch.id === activeId) ?? null;
}

export function optionRevisions(batch: DesignOptionBatchState | null): DesignOptionRevisionState[] {
  return batch?.optionRevisions ?? batch?.option_revisions ?? [];
}

export function revisionForOption(state: WorkbenchState | null | undefined, optionId: string): DesignOptionRevisionState | null {
  return optionRevisions(activeBatchFrom(state)).find((revision) => (revision.optionId ?? revision.option_id) === optionId) ?? null;
}

export function latestComparisonFrom(state: WorkbenchState | null | undefined): DesignOptionComparisonRunState | null {
  const batch = activeBatchFrom(state);
  const runs = batch?.comparisonRuns ?? batch?.comparison_runs ?? [];
  return runs[runs.length - 1] ?? null;
}

export function developmentPathsFrom(state: WorkbenchState | null | undefined): DevelopmentPathState[] {
  return decisionStateFrom(state).developmentPaths ?? [];
}

export function activeDevelopmentPathFrom(state: WorkbenchState | null | undefined): DevelopmentPathState | null {
  const decisions = decisionStateFrom(state);
  const activeId = decisions.activeDevelopmentPathId ?? decisions.active_development_path_id;
  return developmentPathsFrom(state).find((path) => path.id === activeId) ?? null;
}

export function optionIdForPath(path: DevelopmentPathState): string {
  return path.optionId ?? path.option_id ?? '';
}
