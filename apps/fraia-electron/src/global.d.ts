export {};
declare global {
  interface Window {
    fraia: Record<string, (...args: any[]) => Promise<any>> & {
      applicationMetadata?: () => Promise<{
        channel: 'stable' | 'beta';
        productName: string;
        userDataDirectoryName: string;
      }>;
      pickProjectFile: () => Promise<string | null>;
      updateStatus?: () => Promise<import('@/lib/updateStatus').UpdateStatus>;
      checkForUpdates?: () => Promise<import('@/lib/updateStatus').UpdateStatus>;
      setUpdateFrequency?: (frequency: import('@/lib/updateStatus').UpdateFrequency) => Promise<import('@/lib/updateStatus').UpdateStatus>;
      installUpdate?: () => Promise<import('@/lib/updateStatus').UpdateStatus>;
      onUpdateStatus?: (listener: (status: import('@/lib/updateStatus').UpdateStatus) => void) => () => void;
      onOpenUpdateDialog?: (listener: () => void) => () => void;
      defaultProjectDir: () => Promise<string>;
      setThemeSource?: (themeSource: 'light' | 'dark' | 'system') => Promise<{ ok: boolean; themeSource?: string }>;
      reloadWindow?: () => Promise<{ ok: boolean }>;
      forceReloadWindow?: () => Promise<{ ok: boolean }>;
      quitApp?: () => Promise<{ ok: boolean }>;
    };
  }
}
