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
    await waitFor(() => expect(document.title).toBe('Fraia'));
    await user.click(appMenu);
    await user.click(await screen.findByRole('menuitem', { name: 'Quit Fraia' }));

    expect(quitApp).toHaveBeenCalledOnce();
  });
});
