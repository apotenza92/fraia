import { useEffect, useState } from 'react';

export type ThemeMode = 'light' | 'dark' | 'system';

const THEME_STORAGE_KEY = 'fraia:theme-mode';
const SYSTEM_DARK_QUERY = '(prefers-color-scheme: dark)';

function storedThemeMode(): ThemeMode {
  const stored = window.localStorage.getItem(THEME_STORAGE_KEY);
  return stored === 'light' || stored === 'dark' || stored === 'system' ? stored : 'system';
}

function resolvedThemeMode(mode: ThemeMode) {
  if (mode === 'system') return window.matchMedia(SYSTEM_DARK_QUERY).matches ? 'dark' : 'light';
  return mode;
}

export function applyThemeMode(mode: ThemeMode) {
  const resolved = resolvedThemeMode(mode);
  document.documentElement.classList.toggle('dark', resolved === 'dark');
  document.documentElement.style.colorScheme = resolved;
  document.documentElement.dataset.themeMode = mode;
  window.dispatchEvent(new CustomEvent('fraia:themechange', { detail: { mode, resolved } }));
}

export function useThemeMode() {
  const [themeMode, setThemeMode] = useState<ThemeMode>(() => storedThemeMode());

  useEffect(() => {
    applyThemeMode(themeMode);
    window.localStorage.setItem(THEME_STORAGE_KEY, themeMode);
    window.fraia?.setThemeSource?.(themeMode).catch(() => undefined);
  }, [themeMode]);

  useEffect(() => {
    const query = window.matchMedia(SYSTEM_DARK_QUERY);
    function handleSystemThemeChange() {
      if (storedThemeMode() === 'system') applyThemeMode('system');
    }
    query.addEventListener('change', handleSystemThemeChange);
    return () => query.removeEventListener('change', handleSystemThemeChange);
  }, []);

  return { themeMode, setThemeMode };
}
