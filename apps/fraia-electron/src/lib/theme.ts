import { useLayoutEffect } from 'react';

const THEME_STORAGE_KEY = 'fraia:theme-mode';
const SYSTEM_DARK_QUERY = '(prefers-color-scheme: dark)';

export function applySystemTheme() {
  const resolved = window.matchMedia(SYSTEM_DARK_QUERY).matches ? 'dark' : 'light';
  document.documentElement.classList.toggle('dark', resolved === 'dark');
  document.documentElement.style.colorScheme = resolved;
  document.documentElement.dataset.themeMode = 'system';
  window.dispatchEvent(new CustomEvent('fraia:themechange', { detail: { mode: 'system', resolved } }));
}

export function useSystemTheme() {
  useLayoutEffect(() => {
    const query = window.matchMedia(SYSTEM_DARK_QUERY);
    function handleSystemThemeChange() {
      applySystemTheme();
    }

    window.localStorage.removeItem(THEME_STORAGE_KEY);
    applySystemTheme();
    window.fraia?.setThemeSource?.('system').catch(() => undefined);
    query.addEventListener('change', handleSystemThemeChange);
    return () => query.removeEventListener('change', handleSystemThemeChange);
  }, []);
}
