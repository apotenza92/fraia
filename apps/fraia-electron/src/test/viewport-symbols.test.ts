import { describe, expect, it } from 'vitest';

import { loadArrowSymbol, releaseSymbolSpec, supportSymbolSpec } from '@/lib/viewportSymbols';

describe('viewport domain icon geometry', () => {
  it('preserves the reviewed canvas dimensions outside shadcn descendant icon sizing', () => {
    expect(supportSymbolSpec('Fixed')).toMatchObject({
      widthPx: 48,
      heightPx: 48,
      strokeWidth: 2.592,
    });
    expect(releaseSymbolSpec('pinned')).toMatchObject({
      widthPx: 58,
      heightPx: 38,
      strokeWidth: 2.592,
    });
    expect(loadArrowSymbol).toEqual({
      shaftLength: 1.65,
      headBack: 0.28,
      headHalfWidth: 0.2,
      strokeWidth: 5.4,
    });
  });
});
