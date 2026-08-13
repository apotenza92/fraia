import { describe, expect, it } from 'vitest';

import { expandedLabelCenterAlongDirection, loadArrowSymbol, SUPPORT_LABEL_GAP_PX, SUPPORT_SYMBOL_SCALE, VIEWPORT_DETAIL_ZOOM_THRESHOLD, VIEWPORT_LOAD_ARROW_MAX_OFFSET_PX, VIEWPORT_LOAD_ARROW_ZOOM_THRESHOLD, releaseSymbolSpec, supportLabelOffset, supportLabelOffsetCandidates, supportSymbolHitRegion, supportSymbolOffset, supportSymbolSpec, viewportLoadArrowIsNearLine, viewportLoadLeaderFractions, viewportVisualProfile } from '@/lib/viewportSymbols';

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

  it('keeps support labels directly below their rendered support marker', () => {
    for (const [kind, paintedBottom] of [['Fixed', 41], ['Pinned', 35], ['Roller', 43]] as const) {
      const symbol = supportSymbolSpec(kind);
      const symbolOffset = supportSymbolOffset(kind);
      const labelHeight = 24;
      const labelOffset = supportLabelOffset(kind, labelHeight);
      const paintedMarkerBottom = symbolOffset.y + (paintedBottom - symbol.heightPx / 2) * SUPPORT_SYMBOL_SCALE;
      const labelTop = labelOffset.y - labelHeight / 2;

      expect(labelOffset.x).toBe(symbolOffset.x);
      expect(labelTop - paintedMarkerBottom).toBeCloseTo(SUPPORT_LABEL_GAP_PX);
    }
  });

  it('uses the painted support marker rather than its host node as the support hit region', () => {
    for (const kind of ['Fixed', 'Pinned', 'Roller']) {
      const region = supportSymbolHitRegion(kind);
      expect(region.y - region.height / 2).toBeGreaterThan(6);
      expect(region.width).toBeGreaterThan(0);
      expect(region.height).toBeGreaterThan(0);
    }
  });

  it('centres the proposed-support outline on its node', () => {
    expect(supportSymbolOffset('Indicative')).toEqual({ x: 0, y: 0 });
    expect(supportSymbolHitRegion('Indicative')).toMatchObject({ x: 0, y: 0 });
    expect(supportSymbolHitRegion('Indicative').width).toBeCloseTo(20 * SUPPORT_SYMBOL_SCALE);
    expect(supportSymbolHitRegion('Indicative').height).toBeCloseTo(20 * SUPPORT_SYMBOL_SCALE);
  });

  it('only resolves support-label collisions beside or farther below the preferred position', () => {
    const preferred = supportLabelOffset('Indicative', 24);
    const candidates = supportLabelOffsetCandidates(preferred, 88, 24);

    expect(candidates[0]).toEqual(preferred);
    expect(candidates.every((candidate) => candidate.y >= preferred.y)).toBe(true);
  });

  it('reduces annotation weight and artificial gaps in an overview', () => {
    expect(viewportVisualProfile(0.15)).toEqual({
      detail: 0,
      memberStrokePx: 1.35,
      loadStrokePx: 1.35,
      haloExtraPx: 0.4,
      memberEndInsetPx: 0,
      nodeScale: 0.3,
      symbolScale: 0.34,
      loadArrowDetail: 0,
      loadArrowCount: 2,
      baseLabelOpacity: 0,
    });
  });

  it('preserves the reviewed styling at fitted and closer views', () => {
    expect(viewportVisualProfile(1)).toEqual({
      detail: 1,
      memberStrokePx: 2.4,
      loadStrokePx: 2.4,
      haloExtraPx: 7,
      memberEndInsetPx: 16,
      nodeScale: 1,
      symbolScale: 1,
      loadArrowDetail: 1,
      loadArrowCount: 4,
      baseLabelOpacity: 1,
    });
  });

  it('switches every semantic-zoom value at one shared cutoff without fading', () => {
    const overview = viewportVisualProfile(VIEWPORT_DETAIL_ZOOM_THRESHOLD - 0.001);
    const detail = viewportVisualProfile(VIEWPORT_DETAIL_ZOOM_THRESHOLD);
    const arrows = viewportVisualProfile(VIEWPORT_LOAD_ARROW_ZOOM_THRESHOLD);

    expect(overview).toEqual(viewportVisualProfile(0.15));
    expect(overview.baseLabelOpacity).toBe(0);
    expect(overview.memberEndInsetPx).toBe(0);
    expect(detail.baseLabelOpacity).toBe(1);
    expect(detail.memberEndInsetPx).toBe(16);
    expect(detail.loadStrokePx).toBe(detail.memberStrokePx);
    expect(detail.loadArrowDetail).toBe(0);
    expect(detail.loadArrowCount).toBe(4);
    expect(arrows.loadArrowDetail).toBe(1);
    expect(arrows.loadArrowCount).toBe(4);
  });

  it('searches UDL leader attachment points densely from the centre outward', () => {
    expect(viewportLoadLeaderFractions(5)).toEqual([0.5, 1 / 3, 2 / 3, 1 / 6, 5 / 6]);
    const dense = viewportLoadLeaderFractions(19);
    expect(dense[0]).toBe(0.5);
    expect(dense).toHaveLength(19);
    expect(Math.min(...dense)).toBeGreaterThan(0);
    expect(Math.max(...dense)).toBeLessThan(1);
  });

  it('hides UDL arrows that project too far from their load line', () => {
    const linePoint = { x: 40, y: 80 };
    expect(viewportLoadArrowIsNearLine(linePoint, { x: 40, y: 80 + VIEWPORT_LOAD_ARROW_MAX_OFFSET_PX })).toBe(true);
    expect(viewportLoadArrowIsNearLine(linePoint, { x: 40, y: 81 + VIEWPORT_LOAD_ARROW_MAX_OFFSET_PX })).toBe(false);
    expect(viewportLoadArrowIsNearLine(linePoint, { x: 40, y: 79 - VIEWPORT_LOAD_ARROW_MAX_OFFSET_PX })).toBe(false);
  });

  it('keeps the rear label edge fixed while expanding along cardinal and diagonal directions', () => {
    const compactCenter = { x: 100, y: 100 };
    const compactSize = { width: 40, height: 20 };
    const expandedSize = { width: 100, height: 40 };

    expect(expandedLabelCenterAlongDirection(compactCenter, compactSize, expandedSize, { x: 1, y: 0 }))
      .toEqual({ x: 130, y: 100 });
    expect(expandedLabelCenterAlongDirection(compactCenter, compactSize, expandedSize, { x: 0, y: -1 }))
      .toEqual({ x: 100, y: 90 });

    const diagonal = expandedLabelCenterAlongDirection(compactCenter, compactSize, expandedSize, { x: 1, y: -1 });
    const axis = Math.SQRT1_2;
    const compactRearSupport = (compactCenter.x - compactCenter.y) * axis
      - (compactSize.width + compactSize.height) * axis / 2;
    const expandedRearSupport = (diagonal.x - diagonal.y) * axis
      - (expandedSize.width + expandedSize.height) * axis / 2;
    expect(expandedRearSupport).toBeCloseTo(compactRearSupport);
  });

});
