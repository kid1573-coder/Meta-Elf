export type WatchItem = {
  code: string;
  name: string;
};

/** 自选分组：codes 为组内顺序；同一 code 可出现在多个分组 */
export type WatchGroup = {
  id: string;
  name: string;
  codes: string[];
};

export type AppSettings = {
  /** 界面主题：浅色 / 深色（黑白灰） */
  theme: "light" | "dark";
  fontSizePx: number;
  visibleColumns: string[];
  panelMode: "normal" | "minimal";
  autoHideEdge: boolean;
  colorScheme: "redUp" | "greenUp";
  profitDisplay: "both" | "pct" | "amount";
  opacity: number;
  alwaysOnTop: boolean;
  skipTaskbar: boolean;
  bossShortcut: string;
  watchlist: WatchItem[];
  watchGroups: WatchGroup[];
  /** 行情源：东财列表、`tencent` 腾讯行情（买一卖一/五档与常见 App 一致）、`mock` 本地模拟 */
  quoteSource: "eastmoney" | "tencent" | "mock";
  /** HTTP 代理地址，如 `http://127.0.0.1:7890`，留空则直连 */
  proxyUrl: string;
  aiEnabled?: boolean;
  deepseekApiKey?: string;
};

export type SuggestItem = {
  code: string;
  name: string;
  quoteId: string;
  /** 东财行业板块（f100），如「光伏设备」 */
  sector?: string;
  /** 东财现价（f2） */
  price?: number;
  /** 东财涨跌幅 %（f3） */
  changePct?: number;
  /** 东财股吧人气榜名次，仅人气推荐列表返回 */
  rank?: number;
};

export type QuoteRow = {
  code: string;
  name: string;
  changePct: number;
  price: number;
  prevClose: number;
  open: number;
  volume: number;
  turnover: number;
  turnoverRate: number;
  commissionRatio: number;
  totalPl: number;
  dailyPl: number;
  high: number;
  low: number;
  bid1: number;
  ask1: number;
  /** 买一量（手），列表/快照兜底五档用 */
  bid1Vol?: number;
  /** 卖一量（手） */
  ask1Vol?: number;
  /** 量比（东财 f10） */
  volumeRatio?: number | null;
  /** 行业板块名（东财 f100） */
  sector?: string | null;
  /** 对应行业板块当日涨跌幅 % */
  sectorChangePct?: number | null;
};

export type ColumnDef = { id: string; label: string };

export const COLUMN_DEFS: ColumnDef[] = [
    { id: "name", label: "股票名" },
    { id: "price", label: "现价" },
    { id: "changePct", label: "涨跌幅" },
    { id: "volumeRatio", label: "量比" },
    { id: "sectorBlock", label: "AI30m" },
    { id: "aiHighPred", label: "最高" },
    { id: "aiLowPred", label: "最低" },
    { id: "amplitude", label: "振幅" },
    { id: "turnoverRate", label: "换手率" },
    { id: "volume", label: "成交量" },
    { id: "turnover", label: "成交额" },
    { id: "commissionRatio", label: "委比" },
    { id: "high", label: "日高" },
    { id: "low", label: "日低" },
    { id: "dailyPl", label: "当日盈亏" },
    { id: "totalPl", label: "总盈亏" },
    { id: "prevClose", label: "昨收" },
    { id: "open", label: "今开" },
    { id: "bid1", label: "买一" },
    { id: "ask1", label: "卖一" },
];
