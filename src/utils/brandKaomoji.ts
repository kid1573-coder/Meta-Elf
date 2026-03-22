import type { QuoteRow, WatchItem } from "../types/app";

/**
 * 自选品牌情绪：按「自选等权平均涨跌幅」分桶（单位 %）。
 * `BrandElfMascot` 根据 bucket + variant 做造型与动效相位。
 *
 * 阈值：强涨 ≥2.5 | 小涨 ≥0.6 | 微涨 ≥0.1 | 横盘 (-0.1,0.1) | 微跌 ≤-0.1 | 小跌 ≤-0.6 | 大跌 ≤-2.5
 */

function hashCodesKey(watchlist: WatchItem[]): string {
  return [...watchlist.map((w) => w.code.toLowerCase())].sort().join(",");
}

/** 稳定非负哈希，用于同桶内固定轮换 */
function stableHash(s: string): number {
  let h = 2166136261;
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return Math.abs(h >>> 0);
}

export type BrandMoodBucket =
  | "empty"
  | "waiting"
  | "strongUp"
  | "strongUpAll"
  | "mildUp"
  | "microUp"
  | "flat"
  | "microDown"
  | "mildDown"
  | "strongDown"
  | "strongDownAll";

/** 每桶变体数量（稳定哈希取模，用于精灵动效相位等） */
export const BRAND_MOOD_VARIANT_COUNT: Record<BrandMoodBucket, number> = {
  empty: 3,
  waiting: 3,
  strongUp: 3,
  strongUpAll: 3,
  mildUp: 3,
  microUp: 3,
  flat: 3,
  microDown: 3,
  mildDown: 3,
  strongDown: 3,
  strongDownAll: 3,
};

type AvgBucket =
  | "empty"
  | "waiting"
  | "strongUp"
  | "mildUp"
  | "microUp"
  | "flat"
  | "microDown"
  | "mildDown"
  | "strongDown";

function bucketForAvg(avg: number): Exclude<AvgBucket, "empty" | "waiting"> {
  if (avg >= 2.5) return "strongUp";
  if (avg >= 0.6) return "mildUp";
  if (avg >= 0.1) return "microUp";
  if (avg > -0.1) return "flat";
  if (avg > -0.6) return "microDown";
  if (avg > -2.5) return "mildDown";
  return "strongDown";
}

function bucketLabelCn(b: AvgBucket): string {
  switch (b) {
    case "empty":
      return "暂无自选";
    case "waiting":
      return "等待行情";
    case "strongUp":
      return "强涨";
    case "mildUp":
      return "小涨";
    case "microUp":
      return "微涨";
    case "flat":
      return "横盘";
    case "microDown":
      return "微跌";
    case "mildDown":
      return "小跌";
    case "strongDown":
      return "大跌";
    default:
      return "";
  }
}

function toPixelBucket(
  b: Exclude<AvgBucket, "empty" | "waiting">,
  allUp: boolean,
  allDown: boolean,
  n: number,
): BrandMoodBucket {
  if (b === "strongUp" && allUp && n >= 2) return "strongUpAll";
  if (b === "strongDown" && allDown && n >= 2) return "strongDownAll";
  return b;
}

/**
 * 有效涨跌幅：`changePct` 接近 0 但现价与昨收明显偏离时，用现价推算（与列表展示一致）。
 */
export function effectiveQuoteChangePct(row: QuoteRow | null | undefined): number | null {
  if (!row) return null;
  const pc = row.changePct;
  let derived: number | null = null;
  if (row.prevClose > 1e-9 && Number.isFinite(row.price)) {
    derived = ((row.price - row.prevClose) / row.prevClose) * 100;
  }
  if (!Number.isFinite(pc)) {
    return derived;
  }
  if (Math.abs(pc) < 0.01 && derived != null && Math.abs(derived) > 0.025) {
    return derived;
  }
  return pc;
}

function moodPctForRow(row: QuoteRow): number {
  const e = effectiveQuoteChangePct(row);
  if (e != null && Number.isFinite(e)) return e;
  return Number.isFinite(row.changePct) ? row.changePct : 0;
}

/** 单票分桶（与自选阈值一致；横盘带略窄） */
function bucketForSingleStockPct(pct: number): Exclude<AvgBucket, "empty" | "waiting"> {
  const eps = 0.028;
  if (pct >= 2.5) return "strongUp";
  if (pct >= 0.6) return "mildUp";
  if (pct >= 0.1) return "microUp";
  if (pct > eps) return "microUp";
  if (pct <= -2.5) return "strongDown";
  if (pct <= -0.6) return "mildDown";
  if (pct <= -0.1) return "microDown";
  if (pct < -eps) return "microDown";
  return "flat";
}

/**
 * 主界面在查看侧栏个股详情时：精灵随该只涨跌变化（无「全红全绿」强桶）。
 */
export function getBrandMoodSingle(
  code: string,
  quote: QuoteRow,
  displayName?: string,
): { bucket: BrandMoodBucket; variant: number; title: string; ariaLabel: string } {
  const c = code.trim().toLowerCase();
  const changePct = effectiveQuoteChangePct(quote);
  if (!c || changePct == null || !Number.isFinite(changePct)) {
    const bucket: BrandMoodBucket = "waiting";
    const n = BRAND_MOOD_VARIANT_COUNT[bucket];
    const variant = stableHash(c + ":focus-waiting") % n;
    const name = displayName?.trim() || c || "该股";
    return {
      bucket,
      variant,
      title: `等待「${name}」行情… · 点击切换主题`,
      ariaLabel: `等待 ${name} 行情数据`,
    };
  }
  const avgB = bucketForSingleStockPct(changePct);
  const bucket = toPixelBucket(avgB, false, false, 1);
  const mod = BRAND_MOOD_VARIANT_COUNT[bucket];
  const variant = stableHash(c + ":focus:" + bucket) % mod;
  const tag = bucketLabelCn(avgB);
  const pctStr = `${changePct >= 0 ? "+" : ""}${changePct.toFixed(2)}%`;
  const label =
    displayName?.trim() && displayName.trim().toLowerCase() !== c
      ? `${displayName.trim()}（${c}）`
      : c;
  return {
    bucket,
    variant,
    title: `本股 ${pctStr} · ${tag} · 点击切换主题`,
    ariaLabel: `${label} 涨跌幅 ${pctStr}，${tag}`,
  };
}

/**
 * @param watchlist 用户自选列表（含未拉到行情的代码）
 * @param matchedRows 当前 `rows` 中与自选交集（顺序、只数可与 watchlist 不一致）
 */
export function getBrandMood(
  watchlist: WatchItem[],
  matchedRows: QuoteRow[],
): { bucket: BrandMoodBucket; variant: number; title: string; ariaLabel: string } {
  if (watchlist.length === 0) {
    const bucket: BrandMoodBucket = "empty";
    const n = BRAND_MOOD_VARIANT_COUNT[bucket];
    const variant = stableHash(":empty") % n;
    return {
      bucket,
      variant,
      title: "暂无自选 · 点「+」添加 · 表情随自选涨跌而变 · 点击切换主题",
      ariaLabel: "暂无自选，表情随自选涨跌变化",
    };
  }

  const codesKey = hashCodesKey(watchlist);

  if (matchedRows.length === 0) {
    const bucket: BrandMoodBucket = "waiting";
    const n = BRAND_MOOD_VARIANT_COUNT[bucket];
    const variant = stableHash(codesKey + ":waiting") % n;
    return {
      bucket,
      variant,
      title: `已添加 ${watchlist.length} 只自选，等待行情数据… · 点击切换主题`,
      ariaLabel: `等待行情数据，已添加 ${watchlist.length} 只自选`,
    };
  }

  const sum = matchedRows.reduce((a, r) => a + moodPctForRow(r), 0);
  const avg = sum / matchedRows.length;
  const nRows = matchedRows.length;
  const allUp = nRows >= 2 && matchedRows.every((r) => moodPctForRow(r) > 0);
  const allDown = nRows >= 2 && matchedRows.every((r) => moodPctForRow(r) < 0);

  const avgB = bucketForAvg(avg);
  const bucket = toPixelBucket(avgB, allUp, allDown, nRows);
  const mod = BRAND_MOOD_VARIANT_COUNT[bucket];
  const variant = stableHash(codesKey + ":" + bucket) % mod;
  const tag = bucketLabelCn(avgB);
  const avgStr = `${avg >= 0 ? "+" : ""}${avg.toFixed(2)}%`;
  const title = `自选均 ${avgStr}（${nRows}只）· ${tag} · 点击切换主题`;
  const ariaLabel = `自选平均涨跌幅 ${avgStr}，${nRows} 只，${tag}`;

  return { bucket, variant, title, ariaLabel };
}
