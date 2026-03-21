export type WatchItem = {
  code: string;
  name: string;
};

export type AppSettings = {
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
};

export type ColumnDef = { id: string; label: string };

export const COLUMN_DEFS: ColumnDef[] = [
  { id: "name", label: "股票名" },
  { id: "changePct", label: "涨跌幅" },
  { id: "price", label: "现价" },
  { id: "prevClose", label: "昨收" },
  { id: "open", label: "今开" },
  { id: "volume", label: "成交量" },
  { id: "turnover", label: "成交额" },
  { id: "turnoverRate", label: "换手率" },
  { id: "commissionRatio", label: "委比" },
  { id: "totalPl", label: "总盈亏" },
  { id: "dailyPl", label: "当日盈亏" },
  { id: "high", label: "最高" },
  { id: "low", label: "最低" },
  { id: "bid1", label: "买一" },
  { id: "ask1", label: "卖一" },
];
