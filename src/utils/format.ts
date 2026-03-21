import type { AppSettings } from "../types/app";

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
