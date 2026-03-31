import type { AppSettings } from "../types/app";

/** 按 Unicode 码位截断（适合中文），不超过 maxChars 个字 */
export function truncateUnicodeChars(s: string, maxChars: number): string {
  const t = s.trim();
  if (!t) return "";
  const chars = [...t];
  if (chars.length <= maxChars) return t;
  return chars.slice(0, maxChars).join("");
}

export function fmtFixed(n: number, digits: number) {
  if (!Number.isFinite(n)) return "—";
  return n.toFixed(digits);
}

export function fmtVolume(n: number) {
  if (!Number.isFinite(n)) return "—";
  if (n >= 1e8) return `${(n / 1e8).toFixed(2)}亿`;
  if (n >= 1e4) return `${(n / 1e4).toFixed(1)}万`;
  return String(Math.round(n));
}

export function changeClass(pct: number, scheme: AppSettings["colorScheme"]) {
  if (pct > 0) return scheme === "redUp" ? "up-red" : "up-green";
  if (pct < 0) return scheme === "redUp" ? "down-green" : "down-red";
  return "flat";
}

/** 人民币成交额短格式（元 → 万亿 / 亿 / 万） */
export function fmtTurnoverCn(yuan: number) {
  if (!Number.isFinite(yuan) || yuan <= 0) return "—";
  const yi = 1e8;
  const wanYi = 1e12;
  if (yuan >= wanYi) return `${(yuan / wanYi).toFixed(2)}万亿`;
  if (yuan >= yi) return `${(yuan / yi).toFixed(1)}亿`;
  return `${(yuan / 1e4).toFixed(0)}万`;
}

/** 与昨日差额（元），带符号，亿/万 */
export function fmtDeltaTurnoverCn(deltaYuan: number) {
  if (!Number.isFinite(deltaYuan)) return "—";
  if (deltaYuan === 0) return "0";
  const yi = 1e8;
  const sign = deltaYuan > 0 ? "+" : "";
  const v = Math.abs(deltaYuan);
  if (v >= yi) return `${sign}${(deltaYuan / yi).toFixed(1)}亿`;
  return `${sign}${(deltaYuan / 1e4).toFixed(0)}万`;
}
