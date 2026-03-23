import type { IntradayPoint } from "../types/marketDetail";
import { formatIntradayAxisHHmm } from "./chartTime";

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
 * 分时底部固定锚点 + 最后一根 K 时间（与沪深常规时段对齐；收盘多为 15:00）。
 */
export function buildIntradayAxisAnchors(points: IntradayPoint[]): { unix: number; label: string }[] {
  if (points.length === 0) return [];
  const sorted = [...points].sort((a, b) => a.time - b.time);
  const first = sorted[0]!.time;
  const last = sorted[sorted.length - 1]!.time;
  const ymd = shanghaiYmdString(first);
  const wall = (h: number, m: number) => shanghaiWallUnixSec(ymd, h, m);
  const lab = (h: number, m: number) => `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}`;
  const anchors = [
    { unix: wall(9, 30), label: lab(9, 30) },
    { unix: wall(11, 30), label: lab(11, 30) },
    { unix: wall(13, 0), label: lab(13, 0) },
    { unix: wall(15, 0), label: lab(15, 0) },
  ];
  
  // 如果最新时间不在锚点上，且距离锚点有一定距离，则加上最新时间
  const isNearAnchor = anchors.some((a) => Math.abs(a.unix - last) < 60);
  if (!isNearAnchor && last < wall(15, 0)) {
    anchors.push({ unix: last, label: formatIntradayAxisHHmm(last) });
  }
  
  // 按照时间排序
  return anchors.sort((a, b) => a.unix - b.unix);
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
