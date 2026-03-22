export type MarketRibbonSnapshot = {
  indices: { name: string; changePct: number }[];
  upCount: number;
  downCount: number;
  turnoverToday: number;
  turnoverYesterday: number;
};
