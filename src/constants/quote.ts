/**
 * 与自选合并拉取的默认行情代码。
 * 上证/深证/沪深300/创业板等主要指数已在底部 **MarketRibbon** 展示，勿再放进此处，
 * 否则用户从自选删除后仍会出现在「全部」列表（`codesForQuotes = 默认 ∪ 自选`）。
 * 保留一只常见 ETF 作为空自选时的示例标的。
 */
export const DEFAULT_QUOTE_CODES = ["sh510300"];
