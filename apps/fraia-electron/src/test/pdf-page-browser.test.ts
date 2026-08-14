import { describe, expect, it, vi } from 'vitest';
import { applyInferredDraftValues, cachedInference, cropRasterToSourceTransform, displayPointToSource, ocrRotationRadians } from '@/components/sources/PdfPageBrowser';

describe('PDF page source coordinate transform', () => {
  it('inverts the persisted source-to-display transform with zoom and a non-zero origin', () => {
    // Source (40, 50) maps to displayed (0, 0), with a 90-degree page rotation.
    const sourceToDisplay = [0, -1, 1, 0, -50, 1160];
    expect(displayPointToSource({ x: 200, y: 400 }, 2, sourceToDisplay)).toEqual({ x: 960, y: 150 });
    expect(displayPointToSource({ x: 0, y: 0 }, 2, sourceToDisplay)).toEqual({ x: 1160, y: 50 });
  });

  it('rejects a non-invertible transform', () => {
    expect(() => displayPointToSource({ x: 1, y: 1 }, 1, [1, 2, 2, 4, 0, 0])).toThrow(/not invertible/);
  });

  it('maps OCR crop raster pixels back to exact rotated PDF source coordinates', () => {
    const sourceToDisplay = [0, -1, 1, 0, -50, 1160];
    expect(cropRasterToSourceTransform({ x: 200, y: 400 }, 2, 2, sourceToDisplay)).toEqual([
      0,
      0.25,
      -0.25,
      0,
      960,
      150,
    ]);
    expect(ocrRotationRadians(90)).toBeCloseTo(-Math.PI / 2);
    expect(ocrRotationRadians(-90)).toBeCloseTo(Math.PI / 2);
  });

  it('does not overwrite a user-edited reference name or view role when inference finishes late', () => {
    expect(applyInferredDraftValues(
      { name: 'User corrected scanned detail', viewRole: 'detail' },
      { name: 'NORTH ELEVATION', viewRole: 'elevation' },
      { name: true, viewRole: true },
    )).toEqual({ name: 'User corrected scanned detail', viewRole: 'detail' });
  });

  it('runs OCR inference once for the same stable file, page, and crop key', async () => {
    const cache = new Map();
    const load = vi.fn().mockResolvedValue({ suggestions: [], diagnostics: [] });
    const first = cachedInference(cache, 'hash:1:10,20;30,40', load);
    const second = cachedInference(cache, 'hash:1:10,20;30,40', load);
    expect(second).toBe(first);
    await first;
    expect(load).toHaveBeenCalledTimes(1);
  });
});
