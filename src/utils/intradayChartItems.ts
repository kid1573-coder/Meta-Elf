import type { Time, UTCTimestamp } from "lightweight-charts";
import type { IntradayPoint } from "../types/marketDetail";

export type IntradayChartItem =
  | { kind: "bar"; p: IntradayPoint }
  /** 午休断点：无成交价，用于断开折线且保持与真实时间轴对齐（单点即可） */
  | { kind: "gap"; time: number };

function shanghaiMinutesSinceMidnight(tsSec: number): number {
  const parts = new Intl.DateTimeFormat("en-GB", {
    timeZone: "Asia/Shanghai",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  }).formatToParts(new Date(tsSec * 1000));
  const h = parseInt(parts.find((x) => x.type === "hour")?.value ?? "0", 10);
  const m = parseInt(parts.find((x) => x.type === "minute")?.value ?? "0", 10);
  return h * 60 + m;
}

/** `YYYY-MM-DD`（上海日历日） */
export function shanghaiYmdString(tsSec: number): string {
  return new Intl.DateTimeFormat("en-CA", {
    timeZone: "Asia/Shanghai",
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).format(new Date(tsSec * 1000));
}

function shanghaiWallUnixSec(ymd: string, hour: number, minute: number): number {
  const hh = String(hour).padStart(2, "0");
  const mm = String(minute).padStart(2, "0");
  return Date.parse(`${ymd}T${hh}:${mm}:00+08:00`) / 1000;
}

/**
 * 相邻两根 K 若跨沪深午休（11:30 后 → 13:00 前），在中间插入一个 gap，折线不在午休上相连。
 */
function shouldInsertLunchGap(prevTs: number, nextTs: number): boolean {
  if (nextTs - prevTs < 45 * 60) return false;
  const mp = shanghaiMinutesSinceMidnight(prevTs);
  const mn = shanghaiMinutesSinceMidnight(nextTs);
  return mp <= 11 * 60 + 30 && mn >= 13 * 60;
}

function lunchGapUnix(prevTs: number, nextTs: number): number | null {
  const ymd = shanghaiYmdString(prevTs);
  const noon = shanghaiWallUnixSec(ymd, 12, 0);
  if (noon > prevTs && noon < nextTs) return noon;
  return null;
}

/**
 * 按时间排序，并在跨午休处插入单点 gap（lightweight-charts WhitespaceData）。
 */
export function buildIntradayChartItems(points: IntradayPoint[]): IntradayChartItem[] {
  if (points.length === 0) return [];
  const sorted = [...points].sort((a, b) => a.time - b.time);
  const out: IntradayChartItem[] = [];
  for (let i = 0; i < sorted.length; i++) {
    const p = sorted[i]!;
    out.push({ kind: "bar", p });
    const n = sorted[i + 1];
    if (!n) continue;
    if (shouldInsertLunchGap(p.time, n.time)) {
      const g = lunchGapUnix(p.time, n.time);
      if (g != null) {
        out.push({ kind: "gap", time: g });
      }
    }
  }
  return out;
}

/**
 * 分时横轴固定为 A 股完整展示时段（上海 09:15 集合竞价起～15:00 收盘），与常见行情一致：未到时刻留白，不「走到哪算到哪」。
 */
export function intradaySessionVisibleRangeUnix(
  points: IntradayPoint[],
): { from: number; to: number } | null {
  if (points.length === 0) return null;
  const sorted = [...points].sort((a, b) => a.time - b.time);
  const ymd = shanghaiYmdString(sorted[0]!.time);
  return {
    from: shanghaiWallUnixSec(ymd, 9, 15),
    to: shanghaiWallUnixSec(ymd, 15, 0),
  };
}

/**
 * 在折线序列首尾补会话边界的空白时间点，锚定完整横轴（否则库易按已有 K 线收缩可视区）。
 */
export function padIntradayLineDataWithSessionEdges<T extends { time: Time; value?: number }>(
  lineData: T[],
  session: { from: number; to: number },
): T[] {
  if (lineData.length === 0) return lineData;
  let minT = Infinity;
  let maxT = -Infinity;
  for (const d of lineData) {
    const t = d.time as number;
    if (typeof t === "number" && Number.isFinite(t)) {
      minT = Math.min(minT, t);
      maxT = Math.max(maxT, t);
    }
  }
  if (!Number.isFinite(minT) || !Number.isFinite(maxT)) return lineData;
  const out = [...lineData];
  if (minT > session.from) {
    out.unshift({ time: session.from as UTCTimestamp } as T);
  }
  if (maxT < session.to) {
    out.push({ time: session.to as UTCTimestamp } as T);
  }
  return out;
}

/**
 * 分时底部固定锚点：09:15 / 09:30 / 11:30 / 13:00 / 15:00（与完整交易日横轴一致）。
 */
export function buildIntradayAxisAnchors(points: IntradayPoint[]): { unix: number; label: string }[] {
  if (points.length === 0) return [];
  const sorted = [...points].sort((a, b) => a.time - b.time);
  const first = sorted[0]!.time;
  const ymd = shanghaiYmdString(first);
  const wall = (h: number, m: number) => shanghaiWallUnixSec(ymd, h, m);
  const lab = (h: number, m: number) => `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}`;
  return [
    { unix: wall(9, 15), label: lab(9, 15) },
    { unix: wall(9, 30), label: lab(9, 30) },
    { unix: wall(11, 30), label: lab(11, 30) },
    { unix: wall(13, 0), label: lab(13, 0) },
    { unix: wall(15, 0), label: lab(15, 0) },
  ];
}

export function buildIntradayAvgLineDataFromItems(
  items: IntradayChartItem[],
): { time: number; value?: number }[] {
  let sumTurn = 0;
  let sumLots = 0;
  const out: { time: number; value?: number }[] = [];
  for (const row of items) {
    if (row.kind === "gap") {
      out.push({ time: row.time });
      continue;
    }
    const p = row.p;
    sumTurn += p.turnover ?? 0;
    sumLots += p.volume;
    const api = p.avgPrice;
    const v =
      api != null && Number.isFinite(api) && api > 0
        ? api
        : sumLots > 0 && sumTurn > 0
          ? sumTurn / (sumLots * 100)
          : p.close;
    out.push({ time: p.time, value: v });
  }
  return out;
}
