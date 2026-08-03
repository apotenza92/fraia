import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import {
  WorkflowStageBar,
  type WorkflowStageNavigationState,
} from '@/components/layout/WorkflowStageBar';
import { TooltipProvider } from '@/components/ui/tooltip';

const availableStages: WorkflowStageNavigationState[] = [
  { stage: 'base', available: true, gateReason: null },
  { stage: 'options', available: true, gateReason: null },
  { stage: 'analysis', available: true, gateReason: null },
];

function renderStageBar({
  currentStage = 'options',
  stages = availableStages,
  onNavigate = vi.fn(),
}: Partial<React.ComponentProps<typeof WorkflowStageBar>> = {}) {
  render(
    <TooltipProvider>
      <WorkflowStageBar
        currentStage={currentStage}
        stages={stages}
        onNavigate={onNavigate}
      />
    </TooltipProvider>,
  );

  return { onNavigate };
}

describe('WorkflowStageBar', () => {
  it('shows the exact three-stage journey and announces the current step', () => {
    renderStageBar();

    const workflow = screen.getByRole('navigation', { name: 'Design workflow' });
    expect(within(workflow).getByText('Base Model')).toBeVisible();
    expect(within(workflow).getByText('Design Options')).toHaveAttribute('aria-current', 'step');
    expect(within(workflow).getByText('Analysis & Comparison')).toBeVisible();
    expect(screen.getByText('Step 2 of 3: Design Options')).toHaveAttribute('aria-live', 'polite');
  });

  it('keeps earlier stages navigable while future breadcrumb stages stay noninteractive', async () => {
    const user = userEvent.setup();
    const onNavigate = vi.fn();
    renderStageBar({ onNavigate });

    await user.click(screen.getByRole('button', { name: 'Base Model' }));
    expect(onNavigate).toHaveBeenCalledWith('base');

    const futureStage = screen.getByText('Analysis & Comparison');
    expect(futureStage.closest('button')).toBeNull();
    expect(futureStage).toHaveAttribute('aria-disabled', 'true');
  });

  it('uses Previous and Next only to request adjacent-stage navigation', async () => {
    const user = userEvent.setup();
    const onNavigate = vi.fn();
    renderStageBar({ onNavigate });

    await user.click(screen.getByRole('button', { name: 'Previous' }));
    await user.click(screen.getByRole('button', { name: 'Next' }));

    expect(onNavigate.mock.calls).toEqual([['base'], ['analysis']]);
  });

  it('omits Previous on the first stage', () => {
    renderStageBar({ currentStage: 'base' });

    expect(screen.queryByRole('button', { name: 'Previous' })).not.toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Next' })).toBeVisible();
  });

  it('omits Next on the last stage', () => {
    renderStageBar({ currentStage: 'analysis' });

    expect(screen.getByRole('button', { name: 'Previous' })).toBeVisible();
    expect(screen.queryByRole('button', { name: 'Next' })).not.toBeInTheDocument();
  });

  it('keeps a gated Next focusable, explains the gate, and suppresses activation', async () => {
    const user = userEvent.setup();
    const onNavigate = vi.fn();
    const gateReason = 'Include at least one design option for analysis to continue.';
    renderStageBar({
      onNavigate,
      stages: [
        ...availableStages.slice(0, 2),
        { stage: 'analysis', available: false, gateReason },
      ],
    });

    const next = screen.getByRole('button', { name: 'Next' });
    expect(next).not.toBeDisabled();
    expect(next).toHaveAttribute('aria-disabled', 'true');
    const descriptionId = next.getAttribute('aria-describedby');
    expect(descriptionId).toBeTruthy();
    expect(document.getElementById(descriptionId ?? '')).toHaveTextContent(gateReason);

    screen.getByRole('button', { name: 'Previous' }).focus();
    await user.tab();
    await user.tab();
    expect(next).toHaveFocus();
    await user.hover(next);
    expect(
      await screen.findByText(gateReason, { selector: '[data-slot="tooltip-content"]' }),
    ).toBeVisible();

    await user.keyboard('{Enter}');
    await user.click(next);
    expect(onNavigate).not.toHaveBeenCalled();
  });

  it('pins the edge controls around a horizontally scrollable stage path', () => {
    renderStageBar();

    const workflow = screen.getByRole('navigation', { name: 'Design workflow' });
    const scrollArea = workflow.querySelector('[data-slot="workflow-stage-scroll"]');
    expect(scrollArea).toHaveClass('min-w-0', 'overflow-x-auto');
    expect(within(workflow).getByRole('button', { name: 'Previous' })).toBeVisible();
    expect(within(workflow).getByRole('button', { name: 'Next' })).toBeVisible();
  });
});
