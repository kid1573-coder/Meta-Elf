export type MarketSectorBrief = {
  name: string;
  changePct: number;
};

export type RibbonIndex = {
  /** 与后端一致，如 sh000001、glob_n225 */
  id: string;
  name: string;
  changePct: number;
};

export type MarketRibbonSnapshot = {
  indices: RibbonIndex[];
  upCount: number;
  downCount: number;
  turnoverToday: number;
  turnoverYesterday: number;
  /** 沪深行业板块涨幅前六（东财） */
  sectorGainers: MarketSectorBrief[];
  /** 沪深行业板块跌幅前六 */
  sectorLosers: MarketSectorBrief[];
};
