export type ViewportSymbolSpec = {
  widthPx: number;
  heightPx: number;
  strokeWidth: number;
  draw: (ctx: CanvasRenderingContext2D) => void;
};

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
        ctx.rect(dx + 14, 21, 20, 14);
        ctx.fill();
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
