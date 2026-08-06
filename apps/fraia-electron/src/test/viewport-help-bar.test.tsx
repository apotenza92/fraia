import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { ViewportHelpBar } from '@/components/layout/ViewportHelpBar';
import { DEFAULT_VIEWPORT_CUSTOM_NAVIGATION_SETTINGS } from '@/lib/viewportNavigation';

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
    expect(screen.getByTestId('viewport-help-status')).toHaveTextContent('Member · Plane Auto');
    expect(screen.getByTestId('viewport-help-essentials')).not.toHaveTextContent('Select · Click');
    expect(screen.getByTestId('viewport-help-essentials')).toHaveTextContent('Rotate · Left drag');
    expect(Array.from(screen.getByTestId('viewport-help-essentials').querySelectorAll('[data-mouse-gesture]')).map((icon) => (
      icon.getAttribute('data-mouse-gesture')
    ))).toEqual(['left', 'right', 'wheel']);

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
    expect(screen.getByRole('heading', { name: 'Controls' })).toBeInTheDocument();
    expect(screen.getByRole('combobox', { name: 'Navigation profile' })).toHaveTextContent('Fraia — SPACE GASS');
    expect(screen.getByText('Toggle axis lock')).toBeInTheDocument();
    expect(screen.queryByText('Plane')).not.toBeInTheDocument();
    expect(screen.queryByText('Click to toggle')).not.toBeInTheDocument();
    expect(screen.queryByText('Click to clear')).not.toBeInTheDocument();
    expect(screen.queryByText('Click, move, click')).not.toBeInTheDocument();
    expect(screen.getByRole('dialog').querySelectorAll('[data-mouse-gesture="wheel"]')).toHaveLength(1);
    await user.click(screen.getByRole('button', { name: 'Left-handed mouse' }));
    expect(onMouseHandedness).toHaveBeenCalledWith('left');
  });

  it('swaps physical button copy and icons for a left-handed mouse', () => {
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
    expect(Array.from(essentials.querySelectorAll('[data-mouse-gesture]')).map((icon) => (
      icon.getAttribute('data-mouse-gesture')
    ))).toEqual(['right', 'left', 'wheel']);
  });

  it('shows editable mouse assignments only for the Custom profile', async () => {
    const user = userEvent.setup();
    const onCustomNavigationSettings = vi.fn();
    render(
      <ViewportHelpBar
        availableWidth={900}
        status="Select"
        navigationProfileId="custom"
        customNavigationSettings={DEFAULT_VIEWPORT_CUSTOM_NAVIGATION_SETTINGS}
        mouseHandedness="right"
        contextualShortcuts={[]}
        onNavigationProfileId={vi.fn()}
        onCustomNavigationSettings={onCustomNavigationSettings}
        onMouseHandedness={vi.fn()}
      />,
    );
    await user.click(screen.getByRole('button', { name: 'Controls' }));
    expect(screen.getByText('Custom mouse buttons')).toBeInTheDocument();
    expect(screen.getByRole('combobox', { name: 'Left button action' })).toHaveTextContent('Rotate');
    expect(screen.getByRole('combobox', { name: 'Middle button action' })).toHaveTextContent('Pan');
    expect(screen.getByRole('combobox', { name: 'Right button action' })).toHaveTextContent('No camera action');
    expect(screen.getByText('The wheel always zooms.')).toBeInTheDocument();

    await user.click(screen.getByRole('combobox', { name: 'Right button action' }));
    await user.click(screen.getByRole('option', { name: 'Zoom' }));
    expect(onCustomNavigationSettings).toHaveBeenCalledWith({ left: 'rotate', middle: 'pan', right: 'zoom' });
  });
});
