import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { AppMenuBar } from '@/components/layout/AppMenuBar';

describe('AppMenuBar application identity', () => {
  it('fills the native menu row without overriding the official menubar frame', () => {
    Object.defineProperty(window, 'fraia', {
      configurable: true,
      value: {},
    });

    render(<AppMenuBar />);

    const menubar = screen.getByRole('menubar', { name: 'Application menu' });
    const frame = menubar.closest('[data-app-menu-frame]');

    expect(frame).toHaveClass('relative', 'flex', 'w-full', 'items-center');
    expect(menubar).toHaveClass('contents');
    expect(menubar).not.toHaveClass('border-0', 'rounded-none', 'shadow-none');
    expect(frame?.querySelector('[data-slot="separator"]')).toHaveAttribute('data-orientation', 'horizontal');
  });

  it('uses the canonical stable identity in visible chrome and the document title', async () => {
    const quitApp = vi.fn().mockResolvedValue({ ok: true });
    Object.defineProperty(window, 'fraia', {
      configurable: true,
      value: {
        applicationMetadata: vi.fn().mockResolvedValue({
          channel: 'stable',
          productName: 'Fraia',
        }),
        quitApp,
      },
    });

    const user = userEvent.setup();
    render(<AppMenuBar />);

    const appMenu = await screen.findByRole('menuitem', { name: 'Fraia' });
    expect(screen.getAllByRole('menuitem')[0]).toHaveTextContent('Fraia');
    expect(screen.queryByRole('menuitem', { name: 'Settings' })).not.toBeInTheDocument();
    expect(screen.queryByRole('menuitem', { name: 'Developer' })).not.toBeInTheDocument();
    await waitFor(() => expect(document.title).toBe('Fraia'));
    await user.click(appMenu);
    expect(screen.queryByRole('menuitem', { name: 'Fraia AI…' })).not.toBeInTheDocument();
    await user.click(await screen.findByRole('menuitem', { name: 'Quit Fraia' }));

    expect(quitApp).toHaveBeenCalledOnce();
  });

  it('runs a manual update check in the Fraia menu and always shows the result', async () => {
    let statusListener: ((status: any) => void) | undefined;
    const checkForUpdates = vi.fn().mockImplementation(async () => {
      const status = {
        channel: 'stable',
        currentVersion: '0.0.5',
        enabled: true,
        frequency: 'daily',
        lastSuccessfulCheckAt: Date.now(),
        phase: 'up-to-date',
      };
      statusListener?.(status);
      return status;
    });
    Object.defineProperty(window, 'fraia', {
      configurable: true,
      value: {
        applicationMetadata: vi.fn().mockResolvedValue({
          channel: 'stable',
          productName: 'Fraia',
        }),
        checkForUpdates,
        onOpenUpdateDialog: vi.fn(() => () => {}),
        onUpdateStatus: vi.fn((listener) => {
          statusListener = listener;
          return () => {};
        }),
        updateStatus: vi.fn().mockResolvedValue({
          channel: 'stable',
          currentVersion: '0.0.5',
          enabled: true,
          frequency: 'daily',
          phase: 'idle',
        }),
      },
    });

    const user = userEvent.setup();
    render(<AppMenuBar />);

    await user.click(await screen.findByRole('menuitem', { name: 'Fraia' }));
    const checkForUpdatesItem = await screen.findByRole('menuitem', { name: 'Check for Updates…' });
    expect(checkForUpdatesItem.closest('[data-slot="menubar-content"]')).toHaveClass(
      'w-max',
      'min-w-max',
      'whitespace-nowrap',
    );
    await user.click(checkForUpdatesItem);

    expect(checkForUpdates).toHaveBeenCalledOnce();
    expect(await screen.findByRole('dialog', { name: 'Fraia Updates' })).toBeVisible();
    expect(await screen.findByText('Fraia is up to date')).toBeVisible();
  });

  it('exposes the automatic check schedule directly from the Fraia menu', async () => {
    const setUpdateFrequency = vi.fn().mockImplementation(async (frequency) => ({
      channel: 'stable',
      currentVersion: '0.0.6',
      enabled: true,
      frequency,
      phase: 'idle',
    }));
    Object.defineProperty(window, 'fraia', {
      configurable: true,
      value: {
        applicationMetadata: vi.fn().mockResolvedValue({
          channel: 'stable',
          productName: 'Fraia',
        }),
        onOpenUpdateDialog: vi.fn(() => () => {}),
        onUpdateStatus: vi.fn(() => () => {}),
        setUpdateFrequency,
        updateStatus: vi.fn().mockResolvedValue({
          channel: 'stable',
          currentVersion: '0.0.6',
          enabled: true,
          frequency: 'daily',
          phase: 'idle',
        }),
      },
    });

    const user = userEvent.setup();
    render(<AppMenuBar />);

    await user.click(await screen.findByRole('menuitem', { name: 'Fraia' }));
    const schedule = await screen.findByRole('menuitem', { name: 'Check Automatically' });
    await user.hover(schedule);
    const daily = await screen.findByRole('menuitemradio', { name: 'Daily' });
    expect(daily).toHaveAttribute('aria-checked', 'true');
    expect(daily.closest('[data-slot="menubar-sub-content"]')).toHaveClass(
      'w-max',
      'min-w-max',
      'whitespace-nowrap',
    );

    screen.getByRole('menuitemradio', { name: 'Weekly' }).focus();
    await user.keyboard('{Enter}');
    expect(setUpdateFrequency).toHaveBeenCalledWith('weekly');
  });
});
