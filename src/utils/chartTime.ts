import type { Time } from "lightweight-charts";
import type { ChartTabId } from "../types/marketDetail";

function isBusinessDay(t: Time): t is { year: number; month: number; day: number } {
  return typeof t === "object" && t !== null && "year" in t && "month" in t && "day" in t;
}

/**
 * 将「东八区当日 0 点」对应的 Unix 秒转为 lightweight-charts 的 BusinessDay，
 * 避免库按 UTC 解析时间戳导致日/周/月 K 横轴日期错位、年与月份刻度异常。
 */
export function shanghaiUnixToBusinessDay(tsSec: number): Time {
  const parts = new Intl.DateTimeFormat("en-CA-u-ca-gregory", {
    timeZone: "Asia/Shanghai",
    year: "numeric",
    month: "numeric",
    day: "numeric",
  }).formatToParts(new Date(tsSec * 1000));
  const get = (ty: Intl.DateTimeFormatPartTypes) =>
    parseInt(parts.find((p) => p.type === ty)?.value ?? "", 10);
  const year = get("year");
  const month = get("month");
  const day = get("day");
  if (!Number.isFinite(year) || !Number.isFinite(month) || !Number.isFinite(day)) {
    return tsSec as Time;
  }
  return { year, month, day };
}

/** 分时底部自定义轴：固定两位数 HH:mm，避免 zh-CN 出现「上午」等 */
export function formatIntradayAxisHHmm(tsSec: number): string {
  const parts = new Intl.DateTimeFormat("en-GB", {
    timeZone: "Asia/Shanghai",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  }).formatToParts(new Date(tsSec * 1000));
  const h = parts.find((x) => x.type === "hour")?.value ?? "00";
  const m = parts.find((x) => x.type === "minute")?.value ?? "00";
  return `${h}:${m}`;
}

/** 分时横轴刻度：固定按上海时区显示 HH:mm（库内置刻度用） */
export function formatIntradayTickShanghai(time: Time): string | null {
  if (typeof time !== "number") return null;
  return formatIntradayAxisHHmm(time);
}

/**
 * 日/周/月 K 横轴刻度：仅日期，避免库默认英文月份缩写与数字堆叠。
 */
export function formatKlineAxisTick(time: Time): string {
  if (isBusinessDay(time)) {
    return `${time.year}/${time.month}/${time.day}`;
  }
  if (typeof time === "number") {
    return new Date(time * 1000).toLocaleDateString("en-CA", { timeZone: "Asia/Shanghai" });
  }
  if (typeof time === "string") return time;
  return "";
}

/**
 * 十字光标竖线时间标签：覆盖库默认 dateFormat，去掉日 K 上无意义的「日 + 月 + 年 + 时分」杂糅（如 20 3月 '26 02:22）。
 */
export function formatCrosshairTime(time: Time, chartTab: ChartTabId): string {
  if (typeof time === "string") return time;
  if (chartTab === "intraday") {
    if (typeof time === "number") {
      const parts = new Intl.DateTimeFormat("zh-CN", {
        timeZone: "Asia/Shanghai",
        month: "numeric",
        day: "numeric",
      }).formatToParts(new Date(time * 1000));
      const mo = parts.find((x) => x.type === "month")?.value ?? "";
      const da = parts.find((x) => x.type === "day")?.value ?? "";
      return `${mo}/${da} ${formatIntradayAxisHHmm(time)}`;
    }
    if (isBusinessDay(time)) {
      return `${time.year}年${time.month}月${time.day}日`;
    }
    return "";
  }
  /* 日 K / 周 K / 月 K：只显示交易日，不带具体时刻 */
  if (isBusinessDay(time)) {
    return `${time.year}年${time.month}月${time.day}日`;
  }
  if (typeof time === "number") {
    return new Date(time * 1000).toLocaleDateString("zh-CN", {
      timeZone: "Asia/Shanghai",
      year: "numeric",
      month: "long",
      day: "numeric",
    });
  }
  return "";
}
