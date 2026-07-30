export {};
declare global {
  interface Window {
    fraia: Record<string, (...args: any[]) => Promise<any>> & {
      applicationMetadata?: () => Promise<{
        channel: 'stable' | 'beta';
        productName: string;
        userDataDirectoryName: string;
      }>;
      defaultProjectDir: () => Promise<string>;
      setThemeSource?: (themeSource: 'light' | 'dark' | 'system') => Promise<{ ok: boolean; themeSource?: string }>;
      reloadWindow?: () => Promise<{ ok: boolean }>;
      forceReloadWindow?: () => Promise<{ ok: boolean }>;
      quitApp?: () => Promise<{ ok: boolean }>;
    };
  }
}
