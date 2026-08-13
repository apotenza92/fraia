import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { ViewportHelpBar } from '@/components/layout/ViewportHelpBar';
import { DEFAULT_VIEWPORT_CUSTOM_NAVIGATION_SETTINGS } from '@/lib/viewportNavigation';
import { DEFAULT_VIEWPORT_CUSTOM_SELECTION_SETTINGS } from '@/lib/viewportSelection';

const shortcuts = [
  { id: 'axis-lock', keys: ['1', '2', '3'], label: 'Toggle axis lock' },
  { id: 'snap-off', keys: ['Shift'], label: 'Temporarily disable snaps' },
];

describe('viewport help bar', () => {
  it('shows camera essentials without click help at wide and medium canvas widths', () => {
    const { rerender } = render(
      <ViewportHelpBar
        availableWidth={900}
        status="Member · Plane Auto"
        navigationProfileId="spacegass"
        customNavigationSettings={DEFAULT_VIEWPORT_CUSTOM_NAVIGATION_SETTINGS}
        mouseHandedness="right"
        contextualShortcuts={shortcuts}
        onNavigationProfileId={vi.fn()}
        onCustomNavigationSettings={vi.fn()}
        onMouseHandedness={vi.fn()}
      />,
    );
    expect(screen.getByTestId('viewport-help-status')).toHaveTextContent('Member · Plane Auto · Toggle picks · Two-click window');
    expect(screen.getByTestId('viewport-help-essentials')).not.toHaveTextContent('Select · Click');
    expect(screen.getByTestId('viewport-help-essentials')).toHaveTextContent('Rotate · Left drag');
    expect(screen.getByTestId('viewport-help-essentials').querySelector('[data-mouse-gesture]')).not.toBeInTheDocument();

    rerender(
      <ViewportHelpBar
        availableWidth={600}
        status="Select"
        navigationProfileId="spacegass"
        customNavigationSettings={DEFAULT_VIEWPORT_CUSTOM_NAVIGATION_SETTINGS}
        mouseHandedness="right"
        contextualShortcuts={[]}
        onNavigationProfileId={vi.fn()}
        onCustomNavigationSettings={vi.fn()}
        onMouseHandedness={vi.fn()}
      />,
    );
    expect(screen.getByTestId('viewport-help-essentials')).not.toHaveTextContent('Select · Click');

    rerender(
      <ViewportHelpBar
        availableWidth={400}
        status="Select"
        navigationProfileId="spacegass"
        customNavigationSettings={DEFAULT_VIEWPORT_CUSTOM_NAVIGATION_SETTINGS}
        mouseHandedness="right"
        contextualShortcuts={[]}
        onNavigationProfileId={vi.fn()}
        onCustomNavigationSettings={vi.fn()}
        onMouseHandedness={vi.fn()}
      />,
    );
    expect(screen.queryByTestId('viewport-help-essentials')).not.toBeInTheDocument();
  });

  it('opens accessible help with the profile selector and current shortcuts', async () => {
    const user = userEvent.setup();
    const onMouseHandedness = vi.fn();
    render(
      <ViewportHelpBar
        availableWidth={900}
        status="Member · Plane Auto"
        navigationProfileId="spacegass"
        customNavigationSettings={DEFAULT_VIEWPORT_CUSTOM_NAVIGATION_SETTINGS}
        mouseHandedness="right"
        contextualShortcuts={shortcuts}
        onNavigationProfileId={vi.fn()}
        onCustomNavigationSettings={vi.fn()}
        onMouseHandedness={onMouseHandedness}
      />,
    );
    await user.click(screen.getByRole('button', { name: 'Controls' }));
    expect(screen.getByRole('button', { name: 'Controls' })).toHaveTextContent('');
    expect(screen.getByRole('heading', { name: 'Controls' })).toBeInTheDocument();
    expect(screen.getByRole('combobox', { name: 'Control style' })).toHaveTextContent('Fraia — SPACE GASS');
    expect(screen.getByText('Toggle axis lock')).toBeInTheDocument();
    expect(screen.queryByText('Plane')).not.toBeInTheDocument();
    expect(screen.queryByText('Click to toggle')).not.toBeInTheDocument();
    expect(screen.queryByText('Click to clear')).not.toBeInTheDocument();
    expect(screen.queryByText('Click, move, click')).not.toBeInTheDocument();
    expect(screen.getByText('Selection')).toBeInTheDocument();
    expect(screen.getByText(/Click toggles items/)).toBeInTheDocument();
    expect(screen.getByText(/Shift forces the first corner/)).toBeInTheDocument();
    expect(screen.queryByText('Delete selection')).not.toBeInTheDocument();
    expect(screen.queryByText('Cancel or clear')).not.toBeInTheDocument();
    expect(screen.getByRole('dialog').querySelector('[data-mouse-gesture]')).not.toBeInTheDocument();
    expect(within(screen.getByRole('group', { name: 'Mouse hand' })).getAllByRole('button').map((button) => button.getAttribute('aria-label')))
      .toEqual(['Left-handed mouse', 'Right-handed mouse']);
    await user.click(screen.getByRole('button', { name: 'Left-handed mouse' }));
    expect(onMouseHandedness).toHaveBeenCalledWith('left');
    await user.click(screen.getByRole('button', { name: 'Controls' }));
  });

  it('swaps physical button copy for a left-handed mouse without mouse icons', () => {
    render(
      <ViewportHelpBar
        availableWidth={900}
        status="Select"
        navigationProfileId="spacegass"
        customNavigationSettings={DEFAULT_VIEWPORT_CUSTOM_NAVIGATION_SETTINGS}
        mouseHandedness="left"
        contextualShortcuts={[]}
        onNavigationProfileId={vi.fn()}
        onCustomNavigationSettings={vi.fn()}
        onMouseHandedness={vi.fn()}
      />,
    );
    const essentials = screen.getByTestId('viewport-help-essentials');
    expect(essentials).toHaveTextContent('Rotate · Right drag');
    expect(essentials).toHaveTextContent('Pan · Left drag');
    expect(essentials.querySelector('[data-mouse-gesture]')).not.toBeInTheDocument();
  });

  it('shows editable mouse assignments only for the Custom profile', async () => {
    const user = userEvent.setup();
    const onCustomNavigationSettings = vi.fn();
    const onCustomSelectionSettings = vi.fn();
    render(
      <ViewportHelpBar
        availableWidth={900}
        status="Select"
        navigationProfileId="custom"
        customNavigationSettings={DEFAULT_VIEWPORT_CUSTOM_NAVIGATION_SETTINGS}
        customSelectionSettings={DEFAULT_VIEWPORT_CUSTOM_SELECTION_SETTINGS}
        mouseHandedness="right"
        contextualShortcuts={[]}
        onNavigationProfileId={vi.fn()}
        onCustomNavigationSettings={onCustomNavigationSettings}
        onCustomSelectionSettings={onCustomSelectionSettings}
        onMouseHandedness={vi.fn()}
      />,
    );
    await user.click(screen.getByRole('button', { name: 'Controls' }));
    expect(screen.getByText('Custom mouse buttons')).toBeInTheDocument();
    expect(screen.getByRole('combobox', { name: 'Left button action' })).toHaveTextContent('Rotate');
    expect(screen.getByRole('combobox', { name: 'Middle button action' })).toHaveTextContent('Pan');
    expect(screen.getByRole('combobox', { name: 'Right button action' })).toHaveTextContent('No camera action');
    expect(screen.getByText('The wheel always zooms.')).toBeInTheDocument();
    expect(screen.getByText('Custom selection')).toBeInTheDocument();
    expect(screen.getByRole('combobox', { name: 'Direct click' })).toHaveTextContent('Replace');
    expect(screen.getByRole('combobox', { name: 'Modifiers' })).toHaveTextContent('Shift add · Ctrl/Cmd remove');

    await user.click(screen.getByRole('combobox', { name: 'Right button action' }));
    await user.keyboard('zoom{Enter}');
    expect(onCustomNavigationSettings).toHaveBeenCalledWith({ left: 'rotate', middle: 'pan', right: 'zoom' });

    await user.click(screen.getByRole('combobox', { name: 'Direct click' }));
    await user.keyboard('add{Enter}');
    expect(onCustomSelectionSettings).toHaveBeenCalledWith({ ...DEFAULT_VIEWPORT_CUSTOM_SELECTION_SETTINGS, pickBehavior: 'add' });
  });
});
