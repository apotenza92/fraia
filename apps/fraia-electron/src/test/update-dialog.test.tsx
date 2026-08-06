import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { UpdateDialog } from '@/components/updates/UpdateDialog';
import type { UpdateStatus } from '@/lib/updateStatus';

const baseStatus: UpdateStatus = {
  channel: 'stable',
  currentVersion: '0.0.5',
  enabled: true,
  frequency: 'daily',
  phase: 'idle',
  trustedMetadata: true,
};

function renderDialog(status: UpdateStatus, overrides: Partial<React.ComponentProps<typeof UpdateDialog>> = {}) {
  const props: React.ComponentProps<typeof UpdateDialog> = {
    checking: false,
    installing: false,
    onCheck: vi.fn(),
    onInstall: vi.fn(),
    onOpenChange: vi.fn(),
    onSetFrequency: vi.fn(),
    open: true,
    status,
    ...overrides,
  };
  render(<UpdateDialog {...props} />);
  return props;
}

describe('UpdateDialog', () => {
  it('shows download completion, transferred size, speed, and time remaining accessibly', () => {
    renderDialog({
      ...baseStatus,
      phase: 'downloading',
      progress: {
        bytesPerSecond: 2 * 1024 * 1024,
        etaSeconds: 70,
        percent: 48,
        total: 100 * 1024 * 1024,
        transferred: 48 * 1024 * 1024,
      },
      version: '0.0.6',
    });

    expect(screen.getByRole('dialog', { name: 'Fraia Updates' })).toBeVisible();
    expect(screen.getByRole('progressbar', { name: 'Downloading Fraia 0.0.6' })).toHaveAttribute('aria-valuenow', '48');
    expect(screen.getByText('48%')).toBeVisible();
    expect(screen.getByText('48 MB of 100 MB')).toBeVisible();
    expect(screen.getByText('2.0 MB/s · About 2 minutes remaining')).toBeVisible();
    expect(screen.getByRole('status')).toBeVisible();
  });

  it('uses the official accessible spinner while checking', () => {
    renderDialog({ ...baseStatus, phase: 'checking' }, { checking: true });

    expect(screen.getAllByRole('status', { name: 'Loading' })).toHaveLength(2);
    expect(screen.getByText('Checking for updates')).toBeVisible();
  });

  it('keeps the ready state focused on release notes and install actions', async () => {
    const onInstall = vi.fn();
    const onOpenChange = vi.fn();
    renderDialog({
      ...baseStatus,
      phase: 'ready',
      releaseNotes: 'Improved update feedback.',
      version: '0.0.6',
    }, { onInstall, onOpenChange });

    expect(screen.getByRole('dialog', { name: 'Fraia 0.0.6 is ready' })).toBeVisible();
    expect(screen.getByText("What's new")).toBeVisible();
    expect(screen.getByText('Improved update feedback.')).toBeVisible();
    expect(screen.queryByText(/Fraia verifies the update/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/updater works independently of package managers/i)).not.toBeInTheDocument();
    expect(screen.queryByText('Automatic checks')).not.toBeInTheDocument();
    expect(screen.queryByText(/install the update when the app closes/i)).not.toBeInTheDocument();

    const user = userEvent.setup();
    await user.click(screen.getByRole('button', { name: 'Later' }));
    expect(onOpenChange).toHaveBeenCalledWith(false);
    await user.click(screen.getByRole('button', { name: 'Restart and update' }));
    expect(onInstall).toHaveBeenCalledOnce();
  });

  it('keeps long release notes inside a keyboard-scrollable viewport', () => {
    const releaseNotes = Array.from(
      { length: 24 },
      (_, index) => `• Change ${index + 1} keeps update details readable without escaping the dialog.`,
    ).join('\n');

    renderDialog({
      ...baseStatus,
      phase: 'ready',
      releaseNotes,
      version: '0.0.9',
    });

    const notes = screen.getByRole('region', { name: 'Release notes for Fraia 0.0.9' });
    expect(notes).toHaveClass('h-44');
    expect(notes).toHaveAttribute('tabindex', '0');
    expect(notes.querySelector('[data-slot="scroll-area-viewport"]')).not.toBeNull();
    expect(within(notes).getByText(/Change 24 keeps update details readable/)).toBeInTheDocument();
  });

  it('explains Linux package-manager ownership without claiming the in-app updater is broken', () => {
    renderDialog({
      ...baseStatus,
      enabled: false,
      frequency: 'never',
      phase: 'managed',
      reason: 'linux-package-manager',
    });

    expect(screen.getByText('Updates are managed by your Linux package manager')).toBeVisible();
    expect(screen.getByText(/AppImage builds update themselves in Fraia/i)).toBeVisible();
  });
});
