import type { Time } from "lightweight-charts";
import type { SeriesMarker } from "lightweight-charts";
import type { KlinePoint } from "../types/marketDetail";

export type LineTimeValue = { time: Time; value: number };

function emaEwma(vals: number[], period: number): number[] {
  const k = 2 / (period + 1);
  const out: number[] = [];
  let prev = vals[0] ?? 0;
  for (let i = 0; i < vals.length; i++) {
    prev = vals[i]! * k + prev * (1 - k);
    out.push(prev);
  }
  return out;
}

export type MacdPoint = {
  time: Time;
  dif: number;
  dea: number;
  hist: number;
};

/** 收盘序列 MACD(12,26,9)，柱状为 2×(DIF−DEA)（常见看盘口径） */
export function computeMacdSeries(
  points: KlinePoint[],
  timeForBar: (unixSec: number) => Time,
): MacdPoint[] {
  if (points.length === 0) return [];
  const closes = points.map((p) => p.close);
  const e12 = emaEwma(closes, 12);
  const e26 = emaEwma(closes, 26);
  const dif = closes.map((_, i) => e12[i]! - e26[i]!);
  const dea = emaEwma(dif, 9);
  return points.map((p, i) => {
    const d = dif[i]!;
    const e = dea[i]!;
    return {
      time: timeForBar(p.time),
      dif: d,
      dea: e,
      hist: 2 * (d - e),
    };
  });
}

/**
 * 神奇九转（简化）：连续 9 根 K 线收盘价分别高于 / 低于 4 根前的收盘价时在主图标记 1…9。
 * 上涨序列标在 K 线上方（常见见顶提示），下跌序列在下方（见底提示）。
 */
export function computeTdSequentialMarkers(
  points: KlinePoint[],
  timeForBar: (unixSec: number) => Time,
  colorUp: string,
  colorDown: string,
): SeriesMarker<Time>[] {
  const markers: SeriesMarker<Time>[] = [];
  let upStreak = 0;
  let downStreak = 0;
  for (let i = 0; i < points.length; i++) {
    if (i < 4) continue;
    const c = points[i]!.close;
    const c4 = points[i - 4]!.close;
    if (c > c4) {
      upStreak += 1;
      downStreak = 0;
      const n = Math.min(upStreak, 9);
      markers.push({
        time: timeForBar(points[i]!.time),
        position: "aboveBar",
        shape: "square",
        color: n === 9 ? colorUp : `${colorUp}b3`,
        text: String(n),
        size: n === 9 ? 1.15 : 0.9,
      });
    } else if (c < c4) {
      downStreak += 1;
      upStreak = 0;
      const n = Math.min(downStreak, 9);
      markers.push({
        time: timeForBar(points[i]!.time),
        position: "belowBar",
        shape: "square",
        color: n === 9 ? colorDown : `${colorDown}b3`,
        text: String(n),
        size: n === 9 ? 1.15 : 0.9,
      });
    } else {
      upStreak = 0;
      downStreak = 0;
    }
  }
  return markers;
}

function smaCloseAt(closes: number[], index: number, period: number): number | null {
  if (index < period - 1) return null;
  let s = 0;
  for (let j = 0; j < period; j++) {
    s += closes[index - j]!;
  }
  return s / period;
}

/** MA 收盘简单移动平均 */
export function computeMaLineData(
  points: KlinePoint[],
  timeForBar: (unixSec: number) => Time,
  period: number,
): LineTimeValue[] {
  if (points.length === 0) return [];
  const closes = points.map((p) => p.close);
  const out: LineTimeValue[] = [];
  for (let i = 0; i < points.length; i++) {
    const v = smaCloseAt(closes, i, period);
    if (v != null) {
      out.push({ time: timeForBar(points[i]!.time), value: v });
    }
  }
  return out;
}

export type BollPoint = { time: Time; upper: number; mid: number; lower: number };

/** 布林带：中轨 SMA20，上下轨 ±2 倍标准差 */
export function computeBollingerSeries(
  points: KlinePoint[],
  timeForBar: (unixSec: number) => Time,
  period = 20,
  mult = 2,
): BollPoint[] {
  if (points.length < period) return [];
  const closes = points.map((p) => p.close);
  const out: BollPoint[] = [];
  for (let i = period - 1; i < points.length; i++) {
    let sum = 0;
    for (let j = 0; j < period; j++) {
      sum += closes[i - j]!;
    }
    const mid = sum / period;
    let varSum = 0;
    for (let j = 0; j < period; j++) {
      const d = closes[i - j]! - mid;
      varSum += d * d;
    }
    const std = Math.sqrt(varSum / period);
    const u = mid + mult * std;
    const lo = mid - mult * std;
    out.push({ time: timeForBar(points[i]!.time), upper: u, mid, lower: lo });
  }
  return out;
}

function rsvAt(points: KlinePoint[], i: number, n: number): number {
  const start = i - n + 1;
  let lo = Infinity;
  let hi = -Infinity;
  for (let j = start; j <= i; j++) {
    lo = Math.min(lo, points[j]!.low);
    hi = Math.max(hi, points[j]!.high);
  }
  const c = points[i]!.close;
  if (!Number.isFinite(lo) || !Number.isFinite(hi) || hi === lo) return 50;
  return ((c - lo) / (hi - lo)) * 100;
}

export type KdjPoint = { time: Time; k: number; d: number; j: number };

/** KDJ(9,3,3) 平滑：K=⅓·RSV+⅔·昨K，D=⅓·K+⅔·昨D，J=3K−2D */
export function computeKdjSeries(
  points: KlinePoint[],
  timeForBar: (unixSec: number) => Time,
  n = 9,
): KdjPoint[] {
  const out: KdjPoint[] = [];
  let k = 50;
  let d = 50;
  for (let i = 0; i < points.length; i++) {
    if (i < n - 1) continue;
    const rsv = rsvAt(points, i, n);
    k = rsv / 3 + (2 * k) / 3;
    d = k / 3 + (2 * d) / 3;
    const j = 3 * k - 2 * d;
    out.push({ time: timeForBar(points[i]!.time), k, d, j });
  }
  return out;
}

/** RSI Wilder，默认 14 */
export function computeRsiSeries(
  points: KlinePoint[],
  timeForBar: (unixSec: number) => Time,
  period = 14,
): LineTimeValue[] {
  if (points.length < period + 1) return [];
  const closes = points.map((p) => p.close);
  const out: LineTimeValue[] = [];

  function rsToRsi(ag: number, al: number): number {
    if (al <= 0 && ag <= 0) return 50;
    if (al <= 0) return 100;
    if (ag <= 0) return 0;
    const rs = ag / al;
    return 100 - 100 / (1 + rs);
  }

  let avgGain = 0;
  let avgLoss = 0;
  for (let i = 1; i <= period; i++) {
    const ch = closes[i]! - closes[i - 1]!;
    avgGain += ch > 0 ? ch : 0;
    avgLoss += ch < 0 ? -ch : 0;
  }
  avgGain /= period;
  avgLoss /= period;
  out.push({
    time: timeForBar(points[period]!.time),
    value: rsToRsi(avgGain, avgLoss),
  });

  for (let i = period + 1; i < points.length; i++) {
    const ch = closes[i]! - closes[i - 1]!;
    const g = ch > 0 ? ch : 0;
    const l = ch < 0 ? -ch : 0;
    avgGain = (avgGain * (period - 1) + g) / period;
    avgLoss = (avgLoss * (period - 1) + l) / period;
    out.push({
      time: timeForBar(points[i]!.time),
      value: rsToRsi(avgGain, avgLoss),
    });
  }
  return out;
}

export type LatestMaTriple = { ma5: number | null; ma10: number | null; ma20: number | null };

/** 最新一根 K 对应的 MA5/10/20（根数不足则为 null） */
export function latestMaTriple(points: KlinePoint[]): LatestMaTriple {
  const closes = points.map((p) => p.close);
  const n = closes.length - 1;
  if (n < 0) {
    return { ma5: null, ma10: null, ma20: null };
  }
  return {
    ma5: smaCloseAt(closes, n, 5),
    ma10: smaCloseAt(closes, n, 10),
    ma20: smaCloseAt(closes, n, 20),
  };
}

/** 最新一根 K 的布林带三轨（根数不足 period 则 null） */
export function latestBollingerTriple(
  points: KlinePoint[],
  period = 20,
  mult = 2,
): { upper: number; mid: number; lower: number } | null {
  const closes = points.map((p) => p.close);
  const i = closes.length - 1;
  if (i < period - 1) return null;
  let sum = 0;
  for (let j = 0; j < period; j++) {
    sum += closes[i - j]!;
  }
  const mid = sum / period;
  let varSum = 0;
  for (let j = 0; j < period; j++) {
    const d = closes[i - j]! - mid;
    varSum += d * d;
  }
  const std = Math.sqrt(varSum / period);
  return { upper: mid + mult * std, mid, lower: mid - mult * std };
}
