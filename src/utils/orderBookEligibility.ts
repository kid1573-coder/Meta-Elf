/**
 * 详情页右侧盘口：主要指数、ETF/LOF 的买卖档与 A 股股票口径不同或参考意义低，不展示且不请求接口。
 */
export function shouldShowStockDetailOrderBook(code: string): boolean {
  const c = code.trim().toLowerCase();
  if (!c) return false;
  // 上证系列指数 sh000001、sh000300 等
  if (/^sh000\d{3}$/.test(c)) return false;
  // 深证系列指数 sz399001、sz399006 等
  if (/^sz399\d{3}$/.test(c)) return false;
  // 北证指数等 bj899050
  if (/^bj89\d{4}$/.test(c)) return false;
  // 沪市 ETF/常见场内基金 sh510300、sh588000 等
  if (/^sh51\d{4}$/.test(c)) return false;
  if (/^sh56\d{4}$/.test(c)) return false;
  if (/^sh58\d{4}$/.test(c)) return false;
  // 深市 ETF sz159xxx、LOF sz16xxxx
  if (/^sz159\d{3}$/.test(c)) return false;
  if (/^sz16\d{4}$/.test(c)) return false;
  return true;
}
