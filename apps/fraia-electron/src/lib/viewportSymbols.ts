export type ViewportSymbolSpec = {
  widthPx: number;
  heightPx: number;
  strokeWidth: number;
  draw: (ctx: CanvasRenderingContext2D) => void;
};

export const SUPPORT_SYMBOL_SCALE = 0.94;
export const SUPPORT_LABEL_GAP_PX = 6;

const baseMemberStroke = 3.6;
const memberStroke = 2.4;

export const viewportStroke = {
  base: memberStroke,
  member: memberStroke,
  memberFocused: memberStroke * 1.5,
  support: baseMemberStroke * 0.72,
  load: baseMemberStroke * 1.5,
  loadFocused: baseMemberStroke * 1.8,
  get symbol() {
    return this.support;
  },
};

export type ViewportVisualProfile = {
  detail: number;
  memberStrokePx: number;
  loadStrokePx: number;
  haloExtraPx: number;
  memberEndInsetPx: number;
  nodeScale: number;
  symbolScale: number;
  loadArrowDetail: number;
  loadArrowCount: number;
  baseLabelOpacity: number;
};

function mix(from: number, to: number, amount: number) {
  return from + (to - from) * amount;
}

export const VIEWPORT_DETAIL_ZOOM_THRESHOLD = 0.5;
export const VIEWPORT_LOAD_ARROW_ZOOM_THRESHOLD = 0.8;

/**
 * Returns screen-space styling for an orthographic camera whose fitted view is zoom 1.
 * Core topology remains visible in overview while annotation weight and artificial gaps recede.
 */
export function viewportVisualProfile(relativeZoom: number): ViewportVisualProfile {
  const detail = relativeZoom >= VIEWPORT_DETAIL_ZOOM_THRESHOLD ? 1 : 0;
  const loadArrowDetail = relativeZoom >= VIEWPORT_LOAD_ARROW_ZOOM_THRESHOLD ? 1 : 0;
  const memberStrokePx = mix(1.35, viewportStroke.member, detail);
  return {
    detail,
    memberStrokePx,
    loadStrokePx: memberStrokePx,
    haloExtraPx: mix(0.4, 7, detail),
    memberEndInsetPx: mix(0, 16, detail),
    nodeScale: mix(0.3, 1, detail),
    symbolScale: mix(0.34, 1, detail),
    loadArrowDetail,
    loadArrowCount: detail ? 4 : 2,
    baseLabelOpacity: detail,
  };
}

/**
 * Returns dense candidate attachment points along a UDL rail. Endpoints remain
 * clear of nodes and the centre is always preferred before searching outward.
 */
export function viewportLoadLeaderFractions(requestedCount: number) {
  const count = Math.max(3, Math.floor(requestedCount / 2) * 2 + 1);
  return Array.from({ length: count }, (_, index) => (index + 1) / (count + 1))
    .sort((left, right) => {
      const distanceDifference = Math.abs(left - 0.5) - Math.abs(right - 0.5);
      return Math.abs(distanceDifference) > 1e-12 ? distanceDifference : left - right;
    });
}

type ViewportPoint2 = { x: number; y: number };
type ViewportSize2 = { width: number; height: number };

/**
 * Grows a screen-aligned label along an arbitrary screen direction while keeping
 * its rear supporting edge fixed. Cardinal directions preserve one full edge;
 * diagonal directions preserve the supporting corner/edge perpendicular to the vector.
 */
export function expandedLabelCenterAlongDirection(
  compactCenter: ViewportPoint2,
  compactSize: ViewportSize2,
  expandedSize: ViewportSize2,
  expansionDirection: ViewportPoint2,
) {
  const length = Math.hypot(expansionDirection.x, expansionDirection.y);
  if (length <= 1e-6) return { ...compactCenter };
  const direction = {
    x: expansionDirection.x / length,
    y: expansionDirection.y / length,
  };
  const supportGrowth = (
    Math.abs(direction.x) * (expandedSize.width - compactSize.width)
    + Math.abs(direction.y) * (expandedSize.height - compactSize.height)
  ) / 2;
  return {
    x: compactCenter.x + direction.x * supportGrowth,
    y: compactCenter.y + direction.y * supportGrowth,
  };
}


function filledTriangle(ctx: CanvasRenderingContext2D, points: Array<[number, number]>) {
  ctx.beginPath();
  ctx.moveTo(points[0][0], points[0][1]);
  points.slice(1).forEach(([x, y]) => ctx.lineTo(x, y));
  ctx.closePath();
  ctx.fill();
}

export function supportSymbolSpec(kind: string, groupLabel?: string): ViewportSymbolSpec {
  void groupLabel;
  const widthPx = 48;
  const heightPx = 48;
  return {
    widthPx,
    heightPx,
    strokeWidth: viewportStroke.symbol,
    draw: (ctx) => {
      const dx = 0;
      if (kind === 'Location') {
        ctx.beginPath();
        ctx.moveTo(dx + 9, 24);
        ctx.lineTo(dx + 39, 24);
        ctx.stroke();
        return;
      }
      if (kind === 'Indicative') {
        ctx.beginPath();
        ctx.arc(dx + 24, 24, 10, 0, Math.PI * 2);
        ctx.stroke();
        return;
      }
      if (kind === 'Fixed') {
        ctx.beginPath();
        ctx.moveTo(dx + 8, 33); ctx.lineTo(dx + 40, 33);
        ctx.moveTo(dx + 10, 41); ctx.lineTo(dx + 18, 33);
        ctx.moveTo(dx + 20, 41); ctx.lineTo(dx + 28, 33);
        ctx.moveTo(dx + 30, 41); ctx.lineTo(dx + 38, 33);
        ctx.stroke();
        return;
      }
      if (kind === 'Roller') {
        filledTriangle(ctx, [[dx + 24, 20], [dx + 12, 33], [dx + 36, 33]]);
        ctx.beginPath();
        ctx.arc(dx + 18, 36.8, 3.8, 0, Math.PI * 2);
        ctx.arc(dx + 30, 36.8, 3.8, 0, Math.PI * 2);
        ctx.fill();
        ctx.beginPath();
        ctx.moveTo(dx + 12, 43); ctx.lineTo(dx + 36, 43);
        ctx.stroke();
        return;
      }
      filledTriangle(ctx, [[dx + 24, 20], [dx + 12, 35], [dx + 36, 35]]);
    },
  };
}

export function supportSymbolOffset(kind: string) {
  return { x: 0, y: kind === 'Fixed' ? 8 : kind === 'Indicative' ? 0 : 18 };
}

function supportSymbolPaintBounds(kind: string) {
  if (kind === 'Fixed') return { left: 8, top: 33, right: 40, bottom: 41 };
  if (kind === 'Roller') return { left: 12, top: 20, right: 36, bottom: 43 };
  if (kind === 'Location') return { left: 9, top: 24, right: 39, bottom: 24 };
  if (kind === 'Indicative') return { left: 14, top: 14, right: 34, bottom: 34 };
  return { left: 12, top: 20, right: 36, bottom: 35 };
}

export function supportSymbolHitRegion(kind: string, overviewScale = 1) {
  const symbol = supportSymbolSpec(kind);
  const symbolOffset = supportSymbolOffset(kind);
  const bounds = supportSymbolPaintBounds(kind);
  const paintCenterX = (bounds.left + bounds.right) / 2 - symbol.widthPx / 2;
  const paintCenterY = (bounds.top + bounds.bottom) / 2 - symbol.heightPx / 2;
  return {
    x: (symbolOffset.x + paintCenterX * SUPPORT_SYMBOL_SCALE) * overviewScale,
    y: (symbolOffset.y + paintCenterY * SUPPORT_SYMBOL_SCALE) * overviewScale,
    width: Math.max(2, (bounds.right - bounds.left) * SUPPORT_SYMBOL_SCALE * overviewScale),
    height: Math.max(2, (bounds.bottom - bounds.top) * SUPPORT_SYMBOL_SCALE * overviewScale),
  };
}

function supportSymbolPaintBottom(kind: string) {
  if (kind === 'Fixed') return 41;
  if (kind === 'Roller') return 43;
  if (kind === 'Location') return 24;
  if (kind === 'Indicative') return 34;
  return 35;
}

export function supportLabelOffset(kind: string, labelHeightPx: number) {
  const symbol = supportSymbolSpec(kind);
  const symbolOffset = supportSymbolOffset(kind);
  const paintedBottomFromCenter = (supportSymbolPaintBottom(kind) - symbol.heightPx / 2) * SUPPORT_SYMBOL_SCALE;
  return {
    x: symbolOffset.x,
    y: symbolOffset.y + paintedBottomFromCenter + SUPPORT_LABEL_GAP_PX + labelHeightPx / 2,
  };
}

export function supportLabelOffsetCandidates(
  preferred: { x: number; y: number },
  labelWidthPx: number,
  labelHeightPx: number,
) {
  const horizontalStep = labelWidthPx / 2 + SUPPORT_LABEL_GAP_PX + 8;
  const verticalStep = labelHeightPx + SUPPORT_LABEL_GAP_PX + 4;
  return [
    preferred,
    { x: preferred.x - horizontalStep, y: preferred.y },
    { x: preferred.x + horizontalStep, y: preferred.y },
    { x: preferred.x, y: preferred.y + verticalStep },
    { x: preferred.x - horizontalStep, y: preferred.y + verticalStep },
    { x: preferred.x + horizontalStep, y: preferred.y + verticalStep },
  ];
}

export function releaseSymbolSpec(kind: 'pinned' | 'fixed'): ViewportSymbolSpec {
  return {
    widthPx: 58,
    heightPx: 38,
    strokeWidth: viewportStroke.symbol,
    draw: (ctx) => {
      if (kind === 'fixed') {
        ctx.beginPath();
        ctx.moveTo(6, 19); ctx.lineTo(52, 19);
        ctx.moveTo(29, 8); ctx.lineTo(29, 30);
        ctx.moveTo(22, 10); ctx.lineTo(36, 10);
        ctx.moveTo(22, 28); ctx.lineTo(36, 28);
        ctx.stroke();
        return;
      }
      ctx.beginPath();
      ctx.moveTo(6, 19); ctx.lineTo(22, 19);
      ctx.moveTo(36, 19); ctx.lineTo(52, 19);
      ctx.stroke();
      ctx.beginPath();
      ctx.arc(29, 19, 6, 0, Math.PI * 2);
      ctx.stroke();
    },
  };
}

export const loadArrowSymbol = {
  shaftLength: 1.65,
  headBack: 0.28,
  headHalfWidth: 0.2,
  strokeWidth: viewportStroke.load,
};
