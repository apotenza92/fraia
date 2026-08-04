import { render, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { useSystemTheme } from '@/lib/theme';

function ThemeHarness() {
  useSystemTheme();
  return null;
}

function installSystemTheme(dark: boolean) {
  let matches = dark;
  let listener: (() => void) | undefined;
  Object.defineProperty(window, 'matchMedia', {
    configurable: true,
    value: vi.fn(() => ({
      matches,
      media: '(prefers-color-scheme: dark)',
      onchange: null,
      addEventListener: vi.fn((_event: string, nextListener: () => void) => {
        listener = nextListener;
      }),
      removeEventListener: vi.fn(),
      addListener: vi.fn(),
      removeListener: vi.fn(),
      dispatchEvent: vi.fn(() => false),
    })),
  });

  return {
    setDark(nextDark: boolean) {
      matches = nextDark;
      listener?.();
    },
  };
}

describe('system theme', () => {
  it('discards a stale manual override and starts in the system light appearance', async () => {
    const setThemeSource = vi.fn().mockResolvedValue({ ok: true, themeSource: 'system' });
    Object.defineProperty(window, 'fraia', {
      configurable: true,
      value: { setThemeSource },
    });
    window.localStorage.setItem('fraia:theme-mode', 'dark');
    installSystemTheme(false);

    render(<ThemeHarness />);

    expect(document.documentElement).not.toHaveClass('dark');
    expect(document.documentElement).toHaveStyle({ colorScheme: 'light' });
    expect(document.documentElement).toHaveAttribute('data-theme-mode', 'system');
    expect(window.localStorage.getItem('fraia:theme-mode')).toBeNull();
    await waitFor(() => expect(setThemeSource).toHaveBeenCalledWith('system'));
  });

  it('starts dark and responds when the system appearance changes', () => {
    Object.defineProperty(window, 'fraia', {
      configurable: true,
      value: { setThemeSource: vi.fn().mockResolvedValue({ ok: true, themeSource: 'system' }) },
    });
    const systemTheme = installSystemTheme(true);

    render(<ThemeHarness />);
    expect(document.documentElement).toHaveClass('dark');
    expect(document.documentElement).toHaveStyle({ colorScheme: 'dark' });

    systemTheme.setDark(false);
    expect(document.documentElement).not.toHaveClass('dark');
    expect(document.documentElement).toHaveStyle({ colorScheme: 'light' });
  });
});
