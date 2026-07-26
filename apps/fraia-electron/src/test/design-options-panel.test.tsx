import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { DesignOptionsPanel } from '@/components/layout/ModelWorkspaceSidebar';
import type { DesignOptionBatchState, EngineeringScheme } from '@/lib/types';

const scheme = {
  id: 'option-a',
  name: 'Option A',
  summary: 'A lightweight option.',
  comparison: {
    supportStrategy: 'Pinned supports',
    bracingStrategy: 'Braced bay',
    loadStrategy: 'Direct load path',
    connectionImplication: 'Simple connections',
    readiness: 'Ready',
  },
} as unknown as EngineeringScheme;

function batch(id: string, status: string, included = true): DesignOptionBatchState {
  return {
    id,
    generatedAt: '2026-07-22T00:00:00Z',
    baseModelFingerprint: `fingerprint-${id}`,
    status,
    optionRevisions: [{
      revisionId: `${id}::revision::option-a`,
      optionId: 'option-a',
      label: 'Option A',
      included,
      analysisStatus: 'not_run',
    }],
    comparisonRuns: [],
  };
}

describe('DesignOptionsPanel', () => {
  it('keeps previous generation batches available as read-only history', async () => {
    const user = userEvent.setup();
    const current = batch('batch-current', 'active');
    const historical = batch('batch-previous', 'superseded');

    render(
      <DesignOptionsPanel
        active={{ kind: 'scheme', id: 'option-a' }}
        schemes={[scheme]}
        batch={current}
        batches={[historical, current]}
        stage="options"
        busy={false}
        onSelectScheme={vi.fn()}
        onIncludedChange={vi.fn()}
      />,
    );

    await user.click(screen.getByRole('button', { name: 'History' }));
    expect(await screen.findByText('Previous option batches')).toBeVisible();
    expect(screen.getByText('batch-previous')).toBeVisible();
    expect(screen.getByText('superseded')).toBeVisible();
    expect(screen.queryByRole('button', { name: 'batch-previous' })).not.toBeInTheDocument();
  });

  it('keeps excluded options recoverable only in the shortlisting stage', async () => {
    const user = userEvent.setup();
    const onIncludedChange = vi.fn();
    const current = batch('batch-current', 'active', false);

    render(
      <DesignOptionsPanel
        active={{ kind: 'scheme', id: 'option-a' }}
        schemes={[scheme]}
        batch={current}
        batches={[current]}
        stage="options"
        busy={false}
        onSelectScheme={vi.fn()}
        onIncludedChange={onIncludedChange}
      />,
    );

    await user.click(screen.getByRole('button', { name: 'Excluded options (1)' }));
    const checkbox = screen.getByRole('checkbox', { name: 'Include Option A for analysis' });
    expect(checkbox).not.toBeDisabled();
    await user.click(checkbox);
    expect(onIncludedChange).toHaveBeenCalledWith('option-a', true);
  });
});
