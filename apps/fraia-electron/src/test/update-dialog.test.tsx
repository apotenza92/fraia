import { render, screen } from '@testing-library/react';
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
  });

  it('makes deferred installation explicit and keeps restart available', async () => {
    const onInstall = vi.fn();
    const onOpenChange = vi.fn();
    renderDialog({
      ...baseStatus,
      phase: 'ready',
      releaseNotes: 'Improved update feedback.',
      version: '0.0.6',
    }, { onInstall, onOpenChange });

    expect(screen.getByText(/install the update when the app closes/i)).toBeVisible();
    expect(screen.getByText('Improved update feedback.')).toBeVisible();
    expect(screen.getByText(/updater works independently of package managers/i)).toBeVisible();

    const user = userEvent.setup();
    await user.click(screen.getByRole('button', { name: 'Install when Fraia closes' }));
    expect(onOpenChange).toHaveBeenCalledWith(false);
    await user.click(screen.getByRole('button', { name: 'Restart and update' }));
    expect(onInstall).toHaveBeenCalledOnce();
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
