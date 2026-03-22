export type MarketMoveStock = {
  code: string;
  name: string;
  changePct: number;
};

export type MarketMoveItem = {
  time: string;
  text: string;
  stocks: MarketMoveStock[];
};
