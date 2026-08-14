import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { AnalysisHistorySheet } from '@/components/conversation/AnalysisHistorySheet';

describe('AnalysisHistorySheet', () => {
  it('renders camelCase and snake_case dates and safely labels missing or malformed dates', async () => {
    Object.assign(window, { fraia: {
      listDesignRuns: vi.fn().mockResolvedValue({ projectId: 'project-a', designId: 'design-a', legacyRuns: [], runs: [
        { runId: 'camel', runKind: 'linear_static', createdAt: '2026-08-14T00:00:00Z', status: 'completed', authoredRevisionId: 'r-a', authoredSnapshotId: 's-a' },
        { runId: 'snake', runKind: 'linear_static', created_at: '2026-08-15T00:00:00Z', status: 'completed', authoredRevisionId: 'r-b', authoredSnapshotId: 's-b' },
        { runId: 'missing', runKind: 'linear_static', status: 'completed', authoredRevisionId: 'r-c', authoredSnapshotId: 's-c' },
        { runId: 'malformed', runKind: 'linear_static', createdAt: 'not-a-date', status: 'completed', authoredRevisionId: 'r-d', authoredSnapshotId: 's-d' },
      ] }),
      listDesignRunStatuses: vi.fn().mockResolvedValue([]),
      inspectDesignRun: vi.fn(),
    } });
    render(<AnalysisHistorySheet open projectDir="/project" designId="design-a" designName="Frame" currentSnapshotId="snapshot-current" ancestorSnapshotIds={[]} onOpenChange={vi.fn()} />);
    expect(await screen.findAllByTestId('design-run-row')).toHaveLength(4);
    expect(screen.getAllByText('Recorded run')).toHaveLength(2);
    expect(screen.queryByText('Invalid Date')).not.toBeInTheDocument();
  });

  it('shows canonical staleness and immutable run provenance', async () => {
    const user = userEvent.setup();
    Object.assign(window, { fraia: {
      listDesignRuns: vi.fn().mockResolvedValue({ projectId: 'project-a', designId: 'design-a', legacyRuns: [], runs: [{ runId: 'run-123', runKind: 'linear_static', createdAt: '2026-08-14T00:00:00Z', status: 'completed', authoredRevisionId: 'revision-a', authoredSnapshotId: 'snapshot-a' }] }),
      listDesignRunStatuses: vi.fn().mockResolvedValue([{ runId: 'run-123', status: 'completed', staleness: 'stale_dependency', interpretationDependencies: { revisionIds: ['interpretation-a'], inferenceIds: ['inference-a'] }, stalenessReasons: [{ code: 'interpretation.revision_superseded', message: 'backend copy must not define the UI label', interpretationRevisionId: 'interpretation-a', currentInterpretationRevisionId: 'interpretation-b' }], authoredRevisionId: 'revision-a', authoredSnapshotId: 'snapshot-a', resolvedSnapshotId: 'resolved-a', solverIdentity: 'calculix-2.23', runtimeIdentity: 'fraia-runtime-1', settingsIdentity: 'settings-abc', diagnostics: [] }]),
      inspectDesignRun: vi.fn().mockResolvedValue({ format: 'canonical', manifest: { schemaVersion: 'fraia.design-run.v1', runId: 'run-123', projectId: 'project-a', designId: 'design-a', createdAt: '2026-08-14T00:00:00Z', actor: { actorType: 'user', actorId: 'user' }, runKind: 'linear_static', authoredRevisionId: 'revision-a', authoredSnapshotId: 'snapshot-a', resolvedSnapshotId: 'resolved-a', requestIdentity: 'request-abc', request: {}, settingsIdentity: 'settings-abc', settings: {}, solverIdentity: 'calculix-2.23', runtimeIdentity: 'fraia-runtime-1', inputIdentity: 'input-abc', resultIdentity: 'result-abc', status: 'completed', diagnostics: [], attachments: [{ name: 'results.json', role: 'result', mediaType: 'application/json', sha256: 'hash-abc', byteSize: 42 }] } }),
    } });
    render(<AnalysisHistorySheet open projectDir="/project" designId="design-a" designName="Frame" currentSnapshotId="snapshot-current" ancestorSnapshotIds={['snapshot-a']} onOpenChange={vi.fn()} />);
    expect(await screen.findByText('Stale')).toBeVisible();
    expect(screen.getByText('A drawing interpretation was corrected after this run.')).toHaveAttribute('data-staleness-code', 'interpretation.revision_superseded');
    await user.click(screen.getByRole('button', { name: 'Open' }));
    await waitFor(() => expect(window.fraia.inspectDesignRun).toHaveBeenCalledWith({ projectDir: '/project', designId: 'design-a', runId: 'run-123' }));
    await user.click(screen.getByRole('tab', { name: 'Run details' }));
    expect(screen.getByTestId('canonical-run-details')).toHaveTextContent('Solver: calculix-2.23');
    expect(screen.getByTestId('canonical-run-details')).toHaveTextContent('Resolved snapshot: resolved-a');
    expect(screen.getByText(/SHA-256 hash-abc/)).toBeVisible();
  });
});
