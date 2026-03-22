import type { SeriesMarker, Time, UTCTimestamp } from "lightweight-charts";
import type { IntradayPoint } from "../types/marketDetail";

export type IntradayExtremumCandidate = { i: number; price: number; time: number };

/** 5 根 K 窗口内：中心为严格最高价（或并列峰值取中心点） */
function collectLocalHighs(points: IntradayPoint[], halfW = 2): IntradayExtremumCandidate[] {
  const n = points.length;
  const out: IntradayExtremumCandidate[] = [];
  for (let i = halfW; i < n - halfW; i++) {
    const h = points[i]!.high;
    let ok = true;
    for (let j = i - halfW; j <= i + halfW; j++) {
      if (j !== i && points[j]!.high > h) {
        ok = false;
        break;
      }
    }
    if (ok) {
      out.push({ i, price: h, time: points[i]!.time });
    }
  }
  return out;
}

function collectLocalLows(points: IntradayPoint[], halfW = 2): IntradayExtremumCandidate[] {
  const n = points.length;
  const out: IntradayExtremumCandidate[] = [];
  for (let i = halfW; i < n - halfW; i++) {
    const lo = points[i]!.low;
    let ok = true;
    for (let j = i - halfW; j <= i + halfW; j++) {
      if (j !== i && points[j]!.low < lo) {
        ok = false;
        break;
      }
    }
    if (ok) {
      out.push({ i, price: lo, time: points[i]!.time });
    }
  }
  return out;
}

/**
 * 在候选点中贪心选取至多 `count` 个，两两索引间距 ≥ minBarGap（分钟根数）。
 * preferHigh：true 时优先更高（日内高），false 时优先更低（日内低）。
 */
function pickSpacedExtrema(
  cands: IntradayExtremumCandidate[],
  count: number,
  minBarGap: number,
  preferHigh: boolean,
): IntradayExtremumCandidate[] {
  if (cands.length === 0 || count <= 0) return [];
  const sorted = [...cands].sort((a, b) => (preferHigh ? b.price - a.price : a.price - b.price));
  const picked: IntradayExtremumCandidate[] = [];
  for (const c of sorted) {
    if (picked.length >= count) break;
    if (picked.every((p) => Math.abs(p.i - c.i) >= minBarGap)) {
      picked.push(c);
    }
  }
  return picked;
}

/**
 * 分时「AI顶底」：在局部高低点中各选至多 3 个日内显著高/低点（按极值排序 + 时间间隔约束）。
 * 仅供盘面参考，不构成投资建议。
 */
export function computeIntradaySessionExtremaMarkers(
  points: IntradayPoint[],
  opts: {
    countEach: number;
    minBarGap: number;
    colorTop: string;
    colorBottom: string;
  },
): SeriesMarker<Time>[] {
  const highs = pickSpacedExtrema(
    collectLocalHighs(points, 2),
    opts.countEach,
    opts.minBarGap,
    true,
  );
  const lows = pickSpacedExtrema(
    collectLocalLows(points, 2),
    opts.countEach,
    opts.minBarGap,
    false,
  );

  const markers: SeriesMarker<Time>[] = [];

  highs.forEach((p, idx) => {
    markers.push({
      time: p.time as UTCTimestamp,
      position: "aboveBar",
      shape: "arrowDown",
      color: opts.colorTop,
      text: `高${idx + 1}`,
      size: 1,
    });
  });

  lows.forEach((p, idx) => {
    markers.push({
      time: p.time as UTCTimestamp,
      position: "belowBar",
      shape: "arrowUp",
      color: opts.colorBottom,
      text: `低${idx + 1}`,
      size: 1,
    });
  });

  markers.sort((a, b) => (Number(a.time) || 0) - (Number(b.time) || 0));
  return markers;
}
