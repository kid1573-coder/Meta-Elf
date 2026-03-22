import type { AppSettings, QuoteRow } from "../types/app";

/** 列表/详情标题：优先自选里保存的名称，避免接口把 name 填成代码 */
export function displayStockName(
  row: QuoteRow,
  settings: AppSettings | null,
): string {
  const w = settings?.watchlist.find(
    (x) => x.code.toLowerCase() === row.code.toLowerCase(),
  );
  if (w?.name?.trim()) {
    const nm = w.name.trim();
    if (nm.toLowerCase() !== w.code.toLowerCase()) return nm;
  }
  const api = row.name?.trim() ?? "";
  if (api && api.toLowerCase() !== row.code.toLowerCase()) return api;
  return row.code;
}
