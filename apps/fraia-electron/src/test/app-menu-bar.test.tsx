import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { AppMenuBar } from '@/components/layout/AppMenuBar';

describe('AppMenuBar application identity', () => {
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
    await waitFor(() => expect(document.title).toBe('Fraia'));
    await user.click(appMenu);
    expect(await screen.findByRole('menuitem', { name: 'Fraia AI…' })).toBeVisible();
    await user.click(await screen.findByRole('menuitem', { name: 'Quit Fraia' }));

    expect(quitApp).toHaveBeenCalledOnce();
  });

  it('opens Fraia AI from the Fraia application menu', async () => {
    Object.defineProperty(window, 'fraia', {
      configurable: true,
      value: {
        applicationMetadata: vi.fn().mockResolvedValue({
          channel: 'stable',
          productName: 'Fraia',
        }),
        aiProviders: vi.fn().mockResolvedValue({
          providers: [],
          models: [],
          catalogue: { refreshedAt: '2026-07-27T00:00:00Z', stale: false, source: 'test' },
          secureCredentialStorageAvailable: true,
        }),
        aiRefreshCatalog: vi.fn(),
        onAiRuntimeStatus: vi.fn(() => () => {}),
      },
    });

    const user = userEvent.setup();
    render(<AppMenuBar />);

    await user.click(await screen.findByRole('menuitem', { name: 'Fraia' }));
    await user.click(await screen.findByRole('menuitem', { name: 'Fraia AI…' }));

    expect(await screen.findByRole('dialog', { name: 'Fraia AI' })).toBeVisible();
  });
});
