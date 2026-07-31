export const UPDATE_FREQUENCY_LABELS = {
  never: 'Never',
  startup: 'On startup',
  hourly: 'Hourly',
  sixHours: 'Every 6 hours',
  twelveHours: 'Every 12 hours',
  daily: 'Daily',
  weekly: 'Weekly',
} as const;

export type UpdateFrequency = keyof typeof UPDATE_FREQUENCY_LABELS;

export type UpdatePhase =
  | 'available'
  | 'checking'
  | 'disabled'
  | 'downloading'
  | 'error'
  | 'idle'
  | 'initializing'
  | 'installing'
  | 'managed'
  | 'ready'
  | 'up-to-date';

export type UpdateProgress = {
  bytesPerSecond: number;
  etaSeconds: number | null;
  percent: number;
  total: number;
  transferred: number;
};

export type UpdateStatus = {
  channel: 'stable' | 'beta' | null;
  currentVersion: string | null;
  enabled: boolean;
  errorMessage?: string | null;
  frequency: UpdateFrequency;
  lastAttemptAt?: number | null;
  lastSuccessfulCheckAt?: number | null;
  phase: UpdatePhase;
  progress?: UpdateProgress;
  reason?: string;
  releaseNotes?: string;
  trustedMetadata?: boolean;
  version?: string | null;
};

export function formatBytes(bytes: number) {
  if (!Number.isFinite(bytes) || bytes <= 0) return '0 MB';
  const megabytes = bytes / (1024 * 1024);
  if (megabytes >= 1024) return `${(megabytes / 1024).toFixed(1)} GB`;
  return `${megabytes >= 10 ? megabytes.toFixed(0) : megabytes.toFixed(1)} MB`;
}

export function formatEta(seconds: number | null | undefined) {
  if (!Number.isFinite(seconds) || !seconds || seconds <= 0) return 'Estimating time remaining…';
  if (seconds < 60) return `About ${Math.max(5, Math.ceil(seconds / 5) * 5)} seconds remaining`;
  const minutes = Math.ceil(seconds / 60);
  if (minutes < 60) return `About ${minutes} minute${minutes === 1 ? '' : 's'} remaining`;
  const hours = Math.ceil(minutes / 60);
  return `About ${hours} hour${hours === 1 ? '' : 's'} remaining`;
}

export function formatLastChecked(timestamp: number | null | undefined) {
  if (!timestamp) return 'Not checked yet';
  const elapsedSeconds = Math.max(0, Math.round((Date.now() - timestamp) / 1000));
  if (elapsedSeconds < 60) return 'Checked just now';
  const minutes = Math.round(elapsedSeconds / 60);
  if (minutes < 60) return `Checked ${minutes} minute${minutes === 1 ? '' : 's'} ago`;
  const hours = Math.round(minutes / 60);
  if (hours < 24) return `Checked ${hours} hour${hours === 1 ? '' : 's'} ago`;
  return `Checked ${new Date(timestamp).toLocaleDateString()}`;
}
