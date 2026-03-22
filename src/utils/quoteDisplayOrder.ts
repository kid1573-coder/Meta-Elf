import type { QuoteRow, WatchItem } from "../types/app";

/**
 * 按自选顺序排列行情行，使「置顶 / 上移下移」与列表一致。
 *
 * - `all`：先按 watchlist 顺序列出所有在 rows 中的自选，再接上非自选（保持 rows 原有相对顺序，如默认示例 ETF）。
 * - `watch`：仅自选，严格 watchlist 顺序。
 */
export function orderQuoteRowsForPanel(
  rows: QuoteRow[],
  watchlist: WatchItem[],
  mode: "all" | "watch",
): QuoteRow[] {
  const rowByCode = new Map(rows.map((r) => [r.code.toLowerCase(), r] as const));
  const wlSet = new Set(watchlist.map((w) => w.code.toLowerCase()));

  const wlOrdered: QuoteRow[] = [];
  for (const w of watchlist) {
    const r = rowByCode.get(w.code.toLowerCase());
    if (r) wlOrdered.push(r);
  }

  if (mode === "watch") {
    return wlOrdered;
  }

  const rest = rows.filter((r) => !wlSet.has(r.code.toLowerCase()));
  return [...wlOrdered, ...rest];
}

/** 按分组 codes 顺序列出行情行（仅含当前有报价的代码） */
export function orderQuoteRowsForGroupCodes(rows: QuoteRow[], codes: string[]): QuoteRow[] {
  const rowByCode = new Map(rows.map((r) => [r.code.toLowerCase(), r] as const));
  const out: QuoteRow[] = [];
  for (const c of codes) {
    const r = rowByCode.get(c.toLowerCase());
    if (r) out.push(r);
  }
  return out;
}
