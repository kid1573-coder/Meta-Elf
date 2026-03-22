export type IntradayPoint = {
  time: number;
  open: number;
  close: number;
  high: number;
  low: number;
  volume: number;
  /** 该分钟成交额（元） */
  turnover?: number;
  /** 累计成交均价（分时均价），东财接口有则直接用 */
  avgPrice?: number;
};

/** 集合竞价撮合（东财 iscr=1：9:25 后首根带量分钟） */
export type IntradayAuctionSummary = {
  matchPrice: number;
  matchVolumeLots: number;
  matchTurnover: number;
  pctVsPreClose: number;
  /** 上一交易日 9:30 首分钟成交额，口径与竞价额不同，仅对照 */
  prevRefTurnover?: number | null;
};

export type IntradaySeries = {
  preClose: number;
  points: IntradayPoint[];
  auction?: IntradayAuctionSummary | null;
};

export type KlinePoint = {
  time: number;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
  /** 成交额（元），东财日 K 解析 */
  turnover?: number;
  /** 换手率（%） */
  turnoverRate?: number;
};

export type KlineSeries = {
  points: KlinePoint[];
};

export type OrderBookLevel = {
  price: number;
  volume: number;
};

export type OrderBook = {
  asks: OrderBookLevel[];
  bids: OrderBookLevel[];
  maxLevels: number;
};

/** 详情图 Tab：分时 / 日K / 周K / 月K */
export type ChartTabId = "intraday" | "day" | "week" | "month";

/**
 * 图表指标预设（单一下拉）：分时为主图可选 AI 顶底；K 线为副图振荡类 + 主图叠线/标记。
 * 未列入的（如分时 MACD、OBV 等）在轻量图副图里易糊或量级不稳，故不接入。
 */
export type ChartIndicatorPresetId =
  | "i_none"
  | "i_ai_extrema"
  | "k_vol"
  | "k_macd"
  | "k_kdj"
  | "k_rsi"
  | "k_ma"
  | "k_boll"
  | "k_td9";

export type KlinePeriodArg = "day" | "week" | "month";

/** 叠在图上的量/额/换手：日 K 可与上一根 K 对比涨跌 */
export type ChartInlineStatTrend = "up" | "down" | "flat";

export type ChartInlineStats = {
  volume: string;
  turnover: string;
  turnoverRate: string;
  volumeTrend?: ChartInlineStatTrend;
  turnoverTrend?: ChartInlineStatTrend;
  turnoverRateTrend?: ChartInlineStatTrend;
  volumeHint?: string;
  turnoverHint?: string;
  turnoverRateHint?: string;
  /** 日 K：上一交易日三项，叠在当日数据下方 */
  yesterdayInline?: {
    volume: string;
    turnover: string;
    turnoverRate: string;
  };
};
