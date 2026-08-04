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
    expect(workflow).toHaveClass('select-none');
    expect(within(workflow).getByText('Base Model')).toBeVisible();
    expect(within(workflow).getByText('Design Options')).toHaveAttribute('aria-current', 'step');
    expect(within(workflow).getByText('Analysis & Comparison')).toBeVisible();
    expect(screen.getByText('Step 2 of 3: Design Options')).toHaveAttribute('aria-live', 'polite');
  });

  it('explains the shared generation gate when either later stage is hovered', async () => {
    const user = userEvent.setup();
    const gateReason = 'Generate options from the Base Model to continue.';
    renderStageBar({
      currentStage: 'base',
      stages: [
        { stage: 'base', available: true, gateReason: null },
        { stage: 'options', available: false, gateReason },
        { stage: 'analysis', available: false, gateReason },
      ],
    });

    for (const label of ['Design Options', 'Analysis & Comparison']) {
      const stage = screen.getByText(label);
      expect(stage).toHaveAttribute('aria-disabled', 'true');
      expect(document.getElementById(stage.getAttribute('aria-describedby') ?? '')).toHaveTextContent(gateReason);
      await user.hover(stage);
      expect(
        await screen.findByText(gateReason, { selector: '[data-slot="tooltip-content"]' }),
      ).toBeVisible();
      await user.unhover(stage);
    }
  });

  it('uses the three stage labels to navigate directly to any available stage', async () => {
    const user = userEvent.setup();
    const onNavigate = vi.fn();
    renderStageBar({ onNavigate });

    await user.click(screen.getByRole('button', { name: 'Base Model' }));
    await user.click(screen.getByRole('button', { name: 'Analysis & Comparison' }));

    expect(onNavigate.mock.calls).toEqual([['base'], ['analysis']]);
  });

  it('explains a gated stage and keeps it noninteractive', async () => {
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

    const analysis = screen.getByText('Analysis & Comparison');
    expect(analysis.closest('button')).toBeNull();
    expect(analysis).toHaveAttribute('aria-disabled', 'true');
    const descriptionId = analysis.getAttribute('aria-describedby');
    expect(descriptionId).toBeTruthy();
    expect(document.getElementById(descriptionId ?? '')).toHaveTextContent(gateReason);

    await user.hover(analysis);
    expect(
      await screen.findByText(gateReason, { selector: '[data-slot="tooltip-content"]' }),
    ).toBeVisible();

    expect(onNavigate).not.toHaveBeenCalled();
  });

  it('keeps the stage path horizontally scrollable at constrained widths', () => {
    renderStageBar();

    const workflow = screen.getByRole('navigation', { name: 'Design workflow' });
    const scrollArea = workflow.querySelector('[data-slot="workflow-stage-scroll"]');
    expect(workflow).not.toHaveClass('px-2');
    expect(scrollArea).toHaveClass('min-w-0', 'overflow-x-auto', 'px-2');
    expect(workflow.querySelector('[data-slot="separator"]')).toBeVisible();
    expect(within(workflow).queryByRole('button', { name: 'Previous' })).not.toBeInTheDocument();
    expect(within(workflow).queryByRole('button', { name: 'Next' })).not.toBeInTheDocument();
  });
});
