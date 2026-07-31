import { describe, expect, it, vi } from 'vitest';

import { formatBytes, formatEta, formatLastChecked } from '@/lib/updateStatus';

describe('update status formatting', () => {
  it('formats transfer sizes and conservative time estimates', () => {
    expect(formatBytes(1.5 * 1024 * 1024)).toBe('1.5 MB');
    expect(formatBytes(2 * 1024 * 1024 * 1024)).toBe('2.0 GB');
    expect(formatEta(null)).toBe('Estimating time remaining…');
    expect(formatEta(7)).toBe('About 10 seconds remaining');
    expect(formatEta(61)).toBe('About 2 minutes remaining');
  });

  it('formats the last successful check rather than an unsuccessful attempt', () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-07-31T10:00:00Z'));
    expect(formatLastChecked(Date.parse('2026-07-31T09:55:00Z'))).toBe('Checked 5 minutes ago');
    vi.useRealTimers();
  });
});
