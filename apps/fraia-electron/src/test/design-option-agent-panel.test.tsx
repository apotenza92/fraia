import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { DesignOptionAgentPanel } from '@/components/options/DesignOptionAgentPanel';
import type { DesignOptionRevisionState, EngineeringScheme } from '@/lib/types';

vi.mock('@/components/panels/SchemeChatPanel', () => ({
  SchemeChatPanel: ({ surface, showHeader }: { surface: string; showHeader?: boolean }) => (
    <div data-testid="scheme-chat" data-surface={surface} data-show-header={String(showHeader)} />
  ),
}));

const scheme = {
  id: 'portal-frame',
  name: 'Portal frame',
} as EngineeringScheme;

const revision = {
  revisionId: 'revision-1',
  optionId: scheme.id,
  label: scheme.name,
  included: true,
  analysisStatus: 'not_run',
} as DesignOptionRevisionState;

describe('DesignOptionAgentPanel', () => {
  it('keeps comparison selection beside the focused option agent', async () => {
    const user = userEvent.setup();
    const onIncludedChange = vi.fn();

    render(
      <DesignOptionAgentPanel
        state={null}
        scheme={scheme}
        revision={revision}
        busy={false}
        onState={vi.fn()}
        onIncludedChange={onIncludedChange}
      />,
    );

    const checkbox = screen.getByRole('checkbox', { name: 'Compare' });
    expect(checkbox).toBeChecked();
    await user.click(checkbox);
    expect(onIncludedChange).toHaveBeenCalledWith(false);

    expect(screen.getByTestId('scheme-chat')).toHaveAttribute('data-surface', 'scheme:portal-frame');
    expect(screen.getByTestId('scheme-chat')).toHaveAttribute('data-show-header', 'false');
  });
});
