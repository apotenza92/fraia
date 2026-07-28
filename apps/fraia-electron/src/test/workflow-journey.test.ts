import { describe, expect, it } from 'vitest';

import {
  WORKFLOW_GATE_REASONS,
  initialWorkflowStage,
  runtimeWorkflowStage,
  workflowJourneyFrom,
} from '@/lib/workflowJourney';
import type {
  DesignOptionBatchState,
  DesignOptionRevisionState,
  WorkbenchState,
} from '@/lib/types';

function revision(
  optionId: string,
  included: boolean,
  analysisStatus: string = 'not_run',
  latestAnalysisRunId: string | null = null,
): DesignOptionRevisionState {
  return {
    revisionId: `batch-1::revision::${optionId}`,
    optionId,
    label: optionId,
    included,
    analysisStatus,
    latestAnalysisRunId,
  };
}

function stateWithBatch(
  revisions: DesignOptionRevisionState[],
  overrides: Partial<DesignOptionBatchState> = {},
): WorkbenchState {
  const batch: DesignOptionBatchState = {
    id: 'batch-1',
    generatedAt: '2026-07-22T00:00:00Z',
    baseModelFingerprint: 'base-1',
    status: 'active',
    optionRevisions: revisions,
    comparisonRuns: [],
    ...overrides,
  };
  return {
    designOptionDecisions: {
      activeBatchId: batch.id,
      batches: [batch],
      developmentPaths: [],
    },
  };
}

describe('workflow journey derivation', () => {
  it('starts at Base Model and gates later stages without a batch', () => {
    const journey = workflowJourneyFrom(undefined);

    expect(journey.currentStage).toBe('base');
    expect(journey.previousStage).toBeNull();
    expect(journey.nextStage).toBe('options');
    expect(journey.nextGateReason).toBe(WORKFLOW_GATE_REASONS.generateOptions);
    expect(journey.stages.map(({ available, completed, gated }) => ({ available, completed, gated }))).toEqual([
      { available: true, completed: false, gated: false },
      { available: false, completed: false, gated: true },
      { available: false, completed: false, gated: true },
    ]);
  });

  it('opens Design Options for a current batch and gates analysis when nothing is included', () => {
    const state = stateWithBatch([revision('option-a', false)]);
    const journey = workflowJourneyFrom(state);

    expect(journey.currentStage).toBe('options');
    expect(journey.previousStage).toBe('base');
    expect(journey.nextStage).toBe('analysis');
    expect(journey.nextGateReason).toBe(WORKFLOW_GATE_REASONS.includeOption);
    expect(journey.stages.find(({ stage }) => stage === 'options')).toMatchObject({ available: true, completed: false });
    expect(journey.stages.find(({ stage }) => stage === 'analysis')).toMatchObject({ available: false, gated: true });
  });

  it('does not expose an empty active batch as a completed generation step', () => {
    const state = stateWithBatch([]);

    expect(initialWorkflowStage(state)).toBe('base');
    expect(workflowJourneyFrom(state).stages.find(({ stage }) => stage === 'options')).toMatchObject({
      available: false,
      completed: false,
      gated: true,
    });
  });

  it('identifies exactly the included revisions whose evidence is missing or stale', () => {
    const state = stateWithBatch([
      revision('current', true, 'current', 'run-current'),
      revision('stale', true, 'stale', 'run-old'),
      revision('failed', true, 'failed'),
      revision('missing-run', true, 'current'),
      revision('excluded-stale', false, 'stale', 'run-excluded'),
    ]);
    const journey = workflowJourneyFrom(state, 'analysis');

    expect(journey.includedOptionIds).toEqual(['current', 'stale', 'failed', 'missing-run']);
    expect(journey.staleAnalysisOptionIds).toEqual(['stale']);
    expect(journey.missingAnalysisOptionIds).toEqual(['failed', 'missing-run']);
    expect(journey.missingOrStaleOptionIds).toEqual(['stale', 'failed', 'missing-run']);
    expect(journey.currentStage).toBe('analysis');
    expect(journey.stages.find(({ stage }) => stage === 'options')?.completed).toBe(true);
    expect(journey.stages.find(({ stage }) => stage === 'analysis')?.completed).toBe(false);
  });

  it('starts at Analysis for an exact current comparison regardless of option order', () => {
    const state = stateWithBatch(
      [
        revision('option-a', true, 'current', 'run-a'),
        revision('option-b', true, 'current', 'run-b'),
      ],
      {
        comparisonRuns: [{
          runId: 'comparison-1',
          createdAt: '2026-07-22T00:00:01Z',
          optionIds: ['option-b', 'option-a'],
          evidenceReferences: [
            { optionRevisionId: 'batch-1::revision::option-a', analysisRunId: 'run-a' },
            { optionRevisionId: 'batch-1::revision::option-b', analysisRunId: 'run-b' },
          ],
          objective: 'least mass',
          explanation: 'Option A is lighter.',
          limitations: ['Preliminary analysis only.'],
        }],
      },
    );
    const journey = workflowJourneyFrom(state);

    expect(initialWorkflowStage(state)).toBe('analysis');
    expect(journey.hasExactCurrentAnalysis).toBe(true);
    expect(journey.stages.find(({ stage }) => stage === 'analysis')?.completed).toBe(true);
    expect(journey.previousStage).toBe('options');
    expect(journey.nextStage).toBeNull();
  });

  it('does not treat a comparison for a different shortlist as current', () => {
    const state = stateWithBatch(
      [
        revision('option-a', true, 'current', 'run-a'),
        revision('option-b', true, 'current', 'run-b'),
      ],
      {
        comparisonRuns: [{
          runId: 'comparison-old',
          createdAt: '2026-07-22T00:00:01Z',
          optionIds: ['option-a'],
          objective: 'least mass',
          explanation: 'Old shortlist.',
          limitations: [],
        }],
      },
    );

    expect(initialWorkflowStage(state)).toBe('options');
    expect(workflowJourneyFrom(state).hasExactCurrentAnalysis).toBe(false);
  });

  it('keeps legacy comparisons readable but does not treat them as current evidence', () => {
    const state = stateWithBatch(
      [revision('option-a', true, 'current', 'run-a')],
      {
        comparisonRuns: [{
          runId: 'legacy-comparison',
          createdAt: '2026-07-22T00:00:01Z',
          optionIds: ['option-a'],
          objective: 'least mass',
          explanation: 'Legacy comparison without frozen evidence references.',
          limitations: [],
        }],
      },
    );

    expect(initialWorkflowStage(state)).toBe('options');
    expect(workflowJourneyFrom(state).hasExactCurrentAnalysis).toBe(false);
  });

  it('starts at Analysis for an eligible preserved active path', () => {
    const state = stateWithBatch([revision('option-a', true, 'current', 'run-a')]);
    state.designOptionDecisions = {
      ...state.designOptionDecisions!,
      activeDevelopmentPathId: 'path-a',
      developmentPaths: [{
        id: 'path-a',
        optionId: 'option-a',
        optionRevisionId: 'batch-1::revision::option-a',
        sourceAnalysisRunId: 'run-a',
        status: 'active',
        createdAt: '2026-07-22T00:00:02Z',
        updatedAt: '2026-07-22T00:00:02Z',
      }],
    };

    const journey = workflowJourneyFrom(state);
    expect(journey.currentStage).toBe('analysis');
    expect(journey.hasEligibleActivePath).toBe(true);
    expect(journey.hasExactCurrentAnalysis).toBe(false);
    expect(journey.activePathId).toBe('path-a');
  });

  it('does not reopen a path from an older batch that reused the same authored option id', () => {
    const state = stateWithBatch([revision('option-a', true, 'current', 'run-new')]);
    state.designOptionDecisions = {
      ...state.designOptionDecisions!,
      activeDevelopmentPathId: 'path-old',
      developmentPaths: [{
        id: 'path-old',
        optionId: 'option-a',
        optionRevisionId: 'batch-old::revision::option-a',
        sourceAnalysisRunId: 'run-old',
        status: 'active',
        createdAt: '2026-07-21T00:00:00Z',
        updatedAt: '2026-07-21T00:00:00Z',
      }],
    };

    expect(workflowJourneyFrom(state).hasEligibleActivePath).toBe(false);
  });

  it('falls back deterministically after shortlist or Base Model invalidation', () => {
    const noIncluded = stateWithBatch([revision('option-a', false)]);
    const outdated = stateWithBatch([revision('option-a', true)], { status: 'outdated' });
    const current = stateWithBatch([revision('option-a', true)]);

    expect(runtimeWorkflowStage('analysis', noIncluded)).toBe('options');
    expect(runtimeWorkflowStage('analysis', outdated)).toBe('base');
    expect(runtimeWorkflowStage('options', outdated)).toBe('base');
    expect(runtimeWorkflowStage('analysis', current)).toBe('analysis');
    expect(runtimeWorkflowStage('base', current)).toBe('base');
  });

  it('reads snake-case persisted decision, revision, comparison, and path fields', () => {
    const snakeState = {
      design_option_decisions: {
        active_batch_id: 'batch-snake',
        batches: [{
          id: 'batch-snake',
          generated_at: '2026-07-22T00:00:00Z',
          base_model_fingerprint: 'base-snake',
          status: 'active',
          option_revisions: [{
            revision_id: 'batch-snake::revision::option-snake',
            option_id: 'option-snake',
            label: 'Snake option',
            included: true,
            analysis_status: 'current',
            latest_analysis_run_id: 'run-snake',
          }],
          comparison_runs: [{
            run_id: 'comparison-snake',
            created_at: '2026-07-22T00:00:01Z',
            option_ids: ['option-snake'],
            evidence_references: [{
              option_revision_id: 'batch-snake::revision::option-snake',
              analysis_run_id: 'run-snake',
            }],
            objective: 'least mass',
            explanation: 'Current comparison.',
            limitations: [],
          }],
        }],
        active_development_path_id: 'path-snake',
        development_paths: [{
          id: 'path-snake',
          option_id: 'option-snake',
          option_revision_id: 'batch-snake::revision::option-snake',
          source_analysis_run_id: 'run-snake',
          status: 'active',
          created_at: '2026-07-22T00:00:02Z',
          updated_at: '2026-07-22T00:00:02Z',
        }],
      },
    } as unknown as WorkbenchState;

    const journey = workflowJourneyFrom(snakeState);
    expect(journey.currentStage).toBe('analysis');
    expect(journey.includedOptionIds).toEqual(['option-snake']);
    expect(journey.hasExactCurrentAnalysis).toBe(true);
    expect(journey.hasEligibleActivePath).toBe(true);
    expect(journey.activeBatchId).toBe('batch-snake');
    expect(journey.activePathId).toBe('path-snake');
  });
});
