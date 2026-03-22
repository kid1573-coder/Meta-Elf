import type { ChartIndicatorPresetId, ChartTabId } from "../types/marketDetail";

export const INTRADAY_CHART_INDICATOR_OPTIONS: { value: ChartIndicatorPresetId; label: string }[] = [
  { value: "i_none", label: "无" },
  { value: "i_ai_extrema", label: "AI顶底" },
];

/** 仅选副图/主图在 lightweight-charts 里层次清晰、标尺稳定的项 */
export const KLINE_CHART_INDICATOR_OPTIONS: { value: ChartIndicatorPresetId; label: string }[] = [
  { value: "k_vol", label: "成交量" },
  { value: "k_macd", label: "MACD" },
  { value: "k_kdj", label: "KDJ" },
  { value: "k_rsi", label: "RSI(14)" },
  { value: "k_ma", label: "均线 MA5/10/20" },
  { value: "k_boll", label: "布林带(20,2)" },
  { value: "k_td9", label: "神奇九转" },
];

export const DEFAULT_CHART_INDICATOR_BY_TAB: Record<ChartTabId, ChartIndicatorPresetId> = {
  intraday: "i_none",
  day: "k_vol",
  week: "k_vol",
  month: "k_vol",
};

export function chartIndicatorOptionsForTab(
  tab: ChartTabId,
): { value: ChartIndicatorPresetId; label: string }[] {
  return tab === "intraday" ? INTRADAY_CHART_INDICATOR_OPTIONS : KLINE_CHART_INDICATOR_OPTIONS;
}

export function normalizeIndicatorPresetForTab(
  tab: ChartTabId,
  preset: ChartIndicatorPresetId | string,
): ChartIndicatorPresetId {
  const allowed = new Set(chartIndicatorOptionsForTab(tab).map((o) => o.value));
  if (allowed.has(preset as ChartIndicatorPresetId)) {
    return preset as ChartIndicatorPresetId;
  }
  return DEFAULT_CHART_INDICATOR_BY_TAB[tab];
}
