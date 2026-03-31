<script setup lang="ts">
import {
  ColorType,
  CrosshairMode,
  LineStyle,
  createChart,
  type CandlestickData,
  type HistogramData,
  type IChartApi,
  type ISeriesApi,
  type LineData,
  type Time,
  type UTCTimestamp,
  type WhitespaceData,
} from "lightweight-charts";
import { computed, nextTick, onMounted, onUnmounted, ref, shallowRef, watch } from "vue";
import type { AppSettings } from "../types/app";
import type {
  ChartIndicatorPresetId,
  ChartInlineStats,
  ChartTabId,
  IntradayPoint,
  IntradaySeries,
  KlineSeries,
} from "../types/marketDetail";
import {
  computeBollingerSeries,
  computeKdjSeries,
  computeMaLineData,
  computeMacdSeries,
  computeRsiSeries,
  computeTdSequentialMarkers,
  latestBollingerTriple,
  latestMaTriple,
} from "../utils/chartIndicators";
import {
  formatCrosshairTime,
  formatIntradayTickShanghai,
  formatKlineAxisTick,
  shanghaiUnixToBusinessDay,
} from "../utils/chartTime";
import {
  buildIntradayAxisAnchors,
  buildIntradayAvgLineDataFromItems,
  buildIntradayChartItems,
  buildSessionAnchorLineData,
  intradaySessionVisibleRangeUnix,
} from "../utils/intradayChartItems";
import { changeClass, fmtFixed } from "../utils/format";

export type { ChartInlineStats };

const props = withDefaults(
  defineProps<{
    chartTab: ChartTabId;
    intraday: IntradaySeries | null;
    kline: KlineSeries | null;
    loading: boolean;
    colorScheme: AppSettings["colorScheme"];
    theme: AppSettings["theme"];
    inlineStats?: ChartInlineStats | null;
    /** 快照昨收：分时昨收线与日 K 标记，与行情涨跌幅同源 */
    prevClose?: number | null;
    /** 与周期匹配的下拉预设（分时 i_* / K 线 k_*） */
    indicatorPreset?: ChartIndicatorPresetId;
  }>(),
  {
    colorScheme: "redUp",
    theme: "dark",
    inlineStats: null,
    prevClose: null,
    indicatorPreset: "i_none",
  },
);

function decodeKlineIndicator(preset: ChartIndicatorPresetId): {
  osc: "volume" | "macd" | "kdj" | "rsi";
  td9: boolean;
  ma: boolean;
  boll: boolean;
} {
  switch (preset) {
    case "k_macd":
      return { osc: "macd", td9: false, ma: false, boll: false };
    case "k_kdj":
      return { osc: "kdj", td9: false, ma: false, boll: false };
    case "k_rsi":
      return { osc: "rsi", td9: false, ma: false, boll: false };
    case "k_ma":
      return { osc: "volume", td9: false, ma: true, boll: false };
    case "k_boll":
      return { osc: "volume", td9: false, ma: false, boll: true };
    case "k_td9":
      return { osc: "volume", td9: true, ma: false, boll: false };
    default:
      return { osc: "volume", td9: false, ma: false, boll: false };
  }
}

const rootEl = ref<HTMLElement | null>(null);
/** 与「昨收」标签 position top 同一参照（chart-pane__plot） */
const plotEl = ref<HTMLElement | null>(null);
const chartApi = shallowRef<IChartApi | null>(null);
/** 分时主图系列（昨收为零轴的 Baseline，或兜底 Line）：用于 priceToCoordinate 贴「昨收」标签 */
const intradayLineSeries = shallowRef<ISeriesApi<"Baseline"> | ISeriesApi<"Line"> | ISeriesApi<"Area"> | null>(null);
/** 分时昨收标签的垂直位置（相对图表容器顶边，px） */
const prevCloseOverlayTopPx = ref<number | null>(null);
/** 隐藏库内置横轴后，底部自定义时间刻度（对齐 timeToCoordinate） */
const intradayAxisSlots = ref<{ leftPx: number; label: string }[]>([]);
let ro: ResizeObserver | null = null;
let tsVisibleRangeListener: (() => void) | null = null;

const emit = defineEmits<{
  /** 分时昨收/零轴在视口中的 Y（getBoundingClientRect + priceToCoordinate），与盘口分隔线同一坐标系 */
  intradayBaselineY: [screenY: number | null];
}>();

function intradayPrevClosePrice(): number {
  const preApi = props.intraday?.preClose ?? 0;
  if (props.prevClose != null && props.prevClose > 0) return props.prevClose;
  return preApi > 0 ? preApi : 0;
}

const intradayPrevCloseLabel = computed(() => {
  const p = intradayPrevClosePrice();
  if (p <= 0) return "";
  return `昨收 ${fmtFixed(p, 2)}`;
});

function syncPrevCloseOverlay() {
  prevCloseOverlayTopPx.value = null;
  let baselineScreenY: number | null = null;
  if (props.chartTab === "intraday") {
    const line = intradayLineSeries.value;
    const pre = intradayPrevClosePrice();
    const plot = plotEl.value;
    if (line && pre > 0 && plot) {
      const y = line.priceToCoordinate(pre);
      if (y != null && Number.isFinite(y)) {
        prevCloseOverlayTopPx.value = y;
        const br = plot.getBoundingClientRect();
        baselineScreenY = br.top + y;
      }
    }
  }
  emit("intradayBaselineY", baselineScreenY);
}

const upDown = computed(() => {
  const up = props.colorScheme === "redUp" ? "#ef4444" : "#22c55e";
  const down = props.colorScheme === "redUp" ? "#22c55e" : "#ef4444";
  return { up, down };
});

const chartText = computed(() =>
  props.theme === "light" ? "rgba(23,23,23,0.75)" : "rgba(245,245,245,0.78)",
);

const chartGrid = computed(() =>
  props.theme === "light" ? "rgba(0,0,0,0.06)" : "rgba(255,255,255,0.07)",
);

const hasSeriesData = computed(() => {
  if (props.chartTab === "intraday") {
    return (props.intraday?.points?.length ?? 0) > 0;
  }
  return (props.kline?.points?.length ?? 0) > 0;
});

function applyIntradayTimeRange(chart: IChartApi, points: IntradayPoint[]) {
  const sessionR = intradaySessionVisibleRangeUnix(points);
  if (!sessionR) return;
  try {
    chart.timeScale().setVisibleRange({
      from: sessionR.from as UTCTimestamp,
      to: sessionR.to as UTCTimestamp,
    });
  } catch {
    /* 极窄容器下库可能拒设 */
  }
  chart.timeScale().applyOptions({ rightOffset: 0 });
}

const showInlineStrip = computed(
  () =>
    props.inlineStats != null &&
    !props.loading &&
    hasSeriesData.value,
);

const chartLegendVisible = computed(() => hasSeriesData.value && !props.loading);


const klineMaReadout = computed(() => {
  if (props.chartTab === "intraday" || props.indicatorPreset !== "k_ma" || !props.kline?.points?.length) {
    return null;
  }
  return latestMaTriple(props.kline.points);
});

const klineBollReadout = computed(() => {
  if (props.chartTab === "intraday" || props.indicatorPreset !== "k_boll" || !props.kline?.points?.length) {
    return null;
  }
  return latestBollingerTriple(props.kline.points, 20, 2);
});

const klineMacdReadout = computed(() => {
  if (props.chartTab === "intraday" || props.indicatorPreset !== "k_macd" || !props.kline?.points?.length) {
    return null;
  }
  const arr = computeMacdSeries(props.kline.points, shanghaiUnixToBusinessDay);
  return arr.length ? arr[arr.length - 1]! : null;
});

const klineKdjReadout = computed(() => {
  if (props.chartTab === "intraday" || props.indicatorPreset !== "k_kdj" || !props.kline?.points?.length) {
    return null;
  }
  const arr = computeKdjSeries(props.kline.points, shanghaiUnixToBusinessDay);
  return arr.length ? arr[arr.length - 1]! : null;
});

const klineRsiReadout = computed(() => {
  if (props.chartTab === "intraday" || props.indicatorPreset !== "k_rsi" || !props.kline?.points?.length) {
    return null;
  }
  const arr = computeRsiSeries(props.kline.points, shanghaiUnixToBusinessDay);
  return arr.length ? arr[arr.length - 1]!.value : null;
});

const maLegendColors = computed(() =>
  props.theme === "light"
    ? { m5: "#d97706", m10: "#0891b2", m20: "#7c3aed" }
    : { m5: "#fbbf24", m10: "#38bdf8", m20: "#c084fc" },
);

function fmtMaCell(v: number | null, digits = 2): string {
  return v != null && Number.isFinite(v) ? fmtFixed(v, digits) : "—";
}

function trendToPct(t: "up" | "down" | "flat" | undefined) {
  if (t === "up") return 1;
  if (t === "down") return -1;
  return 0;
}

function stripTrendClass(t: "up" | "down" | "flat" | undefined) {
  return changeClass(trendToPct(t), props.colorScheme);
}

function disposeChart() {
  const c = chartApi.value;
  if (c && tsVisibleRangeListener) {
    c.timeScale().unsubscribeVisibleLogicalRangeChange(tsVisibleRangeListener);
    tsVisibleRangeListener = null;
  }
  intradayAxisSlots.value = [];
  intradayLineSeries.value = null;
  prevCloseOverlayTopPx.value = null;
  emit("intradayBaselineY", null);
  if (c) {
    c.remove();
  }
  chartApi.value = null;
}

/** 改尺寸后库内部价轴布局晚一帧才稳定，双 rAF 再算昨收 Y，盘口零轴才能跟手拖拽同步 */
function syncPrevCloseAfterChartLayout() {
  requestAnimationFrame(() => {
    requestAnimationFrame(() => {
      syncPrevCloseOverlay();
      syncIntradayAxisSlots();
    });
  });
}

function syncIntradayAxisSlots() {
  const chart = chartApi.value;
  const el = rootEl.value;
  if (!chart || !el || props.chartTab !== "intraday" || !props.intraday?.points?.length) {
    intradayAxisSlots.value = [];
    return;
  }
  const ts = chart.timeScale();
  const anchors = buildIntradayAxisAnchors(props.intraday.points);
  const w = el.clientWidth;
  const padL = 4;
  const padR = 4;
  const next: { leftPx: number; label: string }[] = [];
  for (const a of anchors) {
    const c = ts.timeToCoordinate(a.unix as UTCTimestamp);
    if (c == null) continue;
    const x = Number(c);
    if (x < padL || x > w - padR) continue;
    next.push({ leftPx: x, label: a.label });
  }
  intradayAxisSlots.value = next;
}

function resizeToContainer() {
  const c = chartApi.value;
  const el = rootEl.value;
  if (!c || !el) return;
  const h = Math.max(80, el.clientHeight || 112);
  const w = Math.max(80, el.clientWidth);
  c.applyOptions({ width: w, height: h });
  if (props.chartTab === "intraday" && props.intraday?.points?.length) {
    applyIntradayTimeRange(c, props.intraday.points);
  }
  syncPrevCloseAfterChartLayout();
}

function rebuildChart() {
  disposeChart();
  if (props.loading || !hasSeriesData.value) return;
  const el = rootEl.value;
  if (!el) return;
  const w = Math.max(80, el.clientWidth);
  const h = Math.max(80, el.clientHeight || 112);
  if (w < 60) return;

  const { up, down } = upDown.value;
  const chart = createChart(el, {
    layout: {
      background: { type: ColorType.Solid, color: "transparent" },
      textColor: chartText.value,
      /* 关闭画布上的 TV 角标；Apache-2.0 仍建议在关于页等保留对 TradingView / 本库的署名 */
      attributionLogo: false,
    },
    grid: {
      vertLines: { color: chartGrid.value },
      horzLines: { color: chartGrid.value },
    },
    crosshair: { mode: CrosshairMode.Normal },
    localization: {
      locale: "zh-CN",
      timeFormatter: (time: Time) => formatCrosshairTime(time, props.chartTab),
    },
    leftPriceScale: {
      visible: props.chartTab === "intraday",
      borderVisible: false,
      scaleMargins: { top: 0.08, bottom: 0.22 },
    },
    rightPriceScale: {
      visible: true,
      borderVisible: false,
      scaleMargins: { top: 0.08, bottom: 0.22 },
    },
    timeScale: {
      borderVisible: false,
      /* 分时改用底部自定义刻度，保证 09:15/09:30/11:30/13:00/15:00 齐全 */
      visible: props.chartTab !== "intraday",
      timeVisible: props.chartTab === "intraday",
      secondsVisible: false,
      /* 新数据到达时不要自动平移可视区，否则会背离「全日横轴先铺好」 */
      shiftVisibleRangeOnNewBar: props.chartTab === "intraday" ? false : true,
      lockVisibleTimeRangeOnResize: props.chartTab === "intraday",
      /* 避免 fitContent 后在最右侧留出「空白柱距」，图与右侧盘口之间像多出一截 */
      rightOffset: 0,
      /* 允许滚动到最右侧时不留空隙 */
      fixRightEdge: true,
      tickMarkFormatter: (time: Time) =>
        props.chartTab === "intraday"
          ? formatIntradayTickShanghai(time)
          : formatKlineAxisTick(time),
    },
    width: w,
    height: h,
  });
  chartApi.value = chart;

  /** 成交量与价格量级不同；关闭右侧「最新成交量」浮动标签（易与价格混淆），量以顶部条与柱高为准 */
  const histBaseOpts = {
    priceScaleId: "yj-vol",
    lastValueVisible: false,
    priceLineVisible: false,
  } as const;

  if (props.chartTab === "intraday" && props.intraday?.points?.length) {
    const pre = intradayPrevClosePrice();
    const pts = props.intraday.points;
    const chartItems = buildIntradayChartItems(pts);
    const lineData: (LineData | WhitespaceData)[] = chartItems.map((row) =>
      row.kind === "bar"
        ? { time: row.p.time as UTCTimestamp, value: row.p.close }
        : { time: row.time as UTCTimestamp },
    );
    const sessionAnchorData = pre > 0
      ? buildSessionAnchorLineData(pts, pre)
      : [];
    
    const currentPrice = pts[pts.length - 1]?.close ?? pre;
    const isUp = currentPrice >= pre;
    const lineColor = isUp ? up : down;

    /* 零轴：比背景网格略醒目的虚线，仍保持中性灰 */
    const zeroAxisLineColor =
      props.theme === "light" ? "rgba(0, 0, 0, 0.12)" : "rgba(255, 255, 255, 0.16)";

    let maxDiff = 0;
    if (pre > 0) {
      for (const pt of props.intraday.points) {
        if (pt.close > 0) maxDiff = Math.max(maxDiff, Math.abs(pt.close - pre));
        if (pt.avgPrice != null && pt.avgPrice > 0) maxDiff = Math.max(maxDiff, Math.abs(pt.avgPrice - pre));
      }
      if (maxDiff === 0) maxDiff = pre * 0.01;
    }

    const autoscaleInfoProvider = pre > 0 ? (original: () => any) => {
      const res = original();
      if (res !== null) {
        const visibleMaxDiff = Math.max(
          Math.abs(res.priceRange.maxValue - pre),
          Math.abs(res.priceRange.minValue - pre)
        );
        const finalDiff = visibleMaxDiff > 0 ? visibleMaxDiff : maxDiff;
        res.priceRange.minValue = pre - finalDiff;
        res.priceRange.maxValue = pre + finalDiff;
        return res;
      }
      return {
        priceRange: {
          minValue: pre - maxDiff,
          maxValue: pre + maxDiff,
        },
      };
    } : undefined;

    if (pre > 0) {
      // 透明辅助线：用真实值数据锚定 09:30-15:00 完整交易时段 + 左侧价格刻度
      // 必须在主线之前添加，使 fitContent 先看到完整时段
      const dummyLeft = chart.addLineSeries({
        color: "transparent",
        lineWidth: 1,
        crosshairMarkerVisible: false,
        priceLineVisible: false,
        lastValueVisible: false,
        autoscaleInfoProvider,
        priceScaleId: "left",
        priceFormat: {
          type: "price",
          precision: 2,
          minMove: 0.01,
        },
      });
      dummyLeft.setData(
        sessionAnchorData.map((d) => ({ time: d.time as UTCTimestamp, value: d.value })),
      );

      const mainLine = chart.addLineSeries({
        color: lineColor,
        lineWidth: 2,
        priceLineVisible: true,
        lastValueVisible: true,
        autoscaleInfoProvider,
        priceScaleId: "right",
        crosshairMarkerRadius: 4,
        priceFormat: {
          type: "custom",
          formatter: (price: number) => {
            const pct = ((price - pre) / pre) * 100;
            return `${pct > 0 ? "+" : ""}${pct.toFixed(2)}%`;
          },
        },
      });
      intradayLineSeries.value = mainLine as any;
      mainLine.setData(lineData);
      mainLine.createPriceLine({
        price: pre,
        color: zeroAxisLineColor,
        lineWidth: 1,
        lineStyle: LineStyle.Dashed,
        axisLabelVisible: false,
      });
    } else {
      const line = chart.addLineSeries({
        color: lineColor,
        lineWidth: 2,
        priceLineVisible: true,
        autoscaleInfoProvider,
        priceScaleId: "left",
      });
      intradayLineSeries.value = line as any;
      line.setData(lineData as (LineData | WhitespaceData)[]);
    }

    const avgColor =
      props.theme === "light" ? "rgba(161, 98, 7, 0.95)" : "rgba(234, 179, 8, 0.9)";
    const avgRaw = buildIntradayAvgLineDataFromItems(chartItems);
    const avgData: (LineData | WhitespaceData)[] = avgRaw.map((x) =>
      x.value != null && Number.isFinite(x.value)
        ? { time: x.time as UTCTimestamp, value: x.value }
        : { time: x.time as UTCTimestamp },
    );
    if (avgData.length > 0) {
      const avgLine = chart.addLineSeries({
        color: avgColor,
        lineWidth: 1,
        lineStyle: LineStyle.Solid,
        priceLineVisible: false,
        lastValueVisible: true,
      });
      avgLine.setData(avgData);
    }

    const lineForMarkers = intradayLineSeries.value;
    if (lineForMarkers) {
      // 找出最后一个有有效值的点绘制当前价圆点
      let lastValidPoint = null;
      for (let i = lineData.length - 1; i >= 0; i--) {
        if ("value" in lineData[i]!) {
          lastValidPoint = lineData[i];
          break;
        }
      }
      if (lastValidPoint) {
        lineForMarkers.setMarkers([
          {
            time: lastValidPoint.time,
            position: "inBar",
            color: lineColor,
            shape: "circle",
            size: 1,
          },
        ]);
      } else {
        lineForMarkers.setMarkers([]);
      }
    }

    const mainLine = intradayLineSeries.value;
    if (mainLine) {
      mainLine.priceScale().applyOptions({
        scaleMargins: { top: 0.05, bottom: 0.05 },
      });
      if (pre > 0) {
        chart.priceScale("left").applyOptions({
          scaleMargins: { top: 0.05, bottom: 0.05 },
        });
      }
    }
    chart.timeScale().fitContent();
    chart.timeScale().applyOptions({ rightOffset: 0, fixRightEdge: true });
    tsVisibleRangeListener = () => {
      syncPrevCloseOverlay();
      syncIntradayAxisSlots();
    };
    chart.timeScale().subscribeVisibleLogicalRangeChange(tsVisibleRangeListener);
    syncPrevCloseAfterChartLayout();
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        applyIntradayTimeRange(chart, pts);
        syncIntradayAxisSlots();
        syncPrevCloseOverlay();
      });
    });
  } else if (props.kline?.points?.length) {
    const pts = props.kline.points;
    const timeAt = (u: number) => shanghaiUnixToBusinessDay(u);
    const isLight = props.theme === "light";
    const cdata: CandlestickData[] = pts.map((p) => ({
      time: timeAt(p.time),
      open: p.open,
      high: p.high,
      low: p.low,
      close: p.close,
    }));

    const layout = decodeKlineIndicator(props.indicatorPreset);
    let osc = layout.osc;
    if (osc === "rsi" && computeRsiSeries(pts, timeAt).length < 2) {
      osc = "volume";
    }
    if (osc === "kdj" && computeKdjSeries(pts, timeAt).length < 2) {
      osc = "volume";
    }

    const maColors = isLight ? ["#d97706", "#0891b2", "#7c3aed"] : ["#fbbf24", "#38bdf8", "#c084fc"];
    if (layout.ma) {
      const s5 = chart.addLineSeries({
        color: maColors[0]!,
        lineWidth: 1,
        priceLineVisible: false,
        lastValueVisible: false,
      });
      s5.setData(computeMaLineData(pts, timeAt, 5));
      const s10 = chart.addLineSeries({
        color: maColors[1]!,
        lineWidth: 1,
        priceLineVisible: false,
        lastValueVisible: false,
      });
      s10.setData(computeMaLineData(pts, timeAt, 10));
      const s20 = chart.addLineSeries({
        color: maColors[2]!,
        lineWidth: 1,
        priceLineVisible: false,
        lastValueVisible: false,
      });
      s20.setData(computeMaLineData(pts, timeAt, 20));
    }

    if (layout.boll) {
      const boll = computeBollingerSeries(pts, timeAt, 20, 2);
      const uCol = isLight ? "rgba(219, 39, 119, 0.88)" : "rgba(244, 114, 182, 0.92)";
      const mCol = isLight ? "rgba(100, 116, 139, 0.72)" : "rgba(148, 163, 184, 0.72)";
      const bu = chart.addLineSeries({
        color: uCol,
        lineWidth: 1,
        priceLineVisible: false,
        lastValueVisible: false,
      });
      bu.setData(boll.map((b) => ({ time: b.time, value: b.upper })));
      const bm = chart.addLineSeries({
        color: mCol,
        lineWidth: 1,
        lineStyle: LineStyle.Dashed,
        priceLineVisible: false,
        lastValueVisible: false,
      });
      bm.setData(boll.map((b) => ({ time: b.time, value: b.mid })));
      const bln = chart.addLineSeries({
        color: uCol,
        lineWidth: 1,
        priceLineVisible: false,
        lastValueVisible: false,
      });
      bln.setData(boll.map((b) => ({ time: b.time, value: b.lower })));
    }

    const candle = chart.addCandlestickSeries({
      upColor: up,
      downColor: down,
      borderVisible: false,
      wickUpColor: up,
      wickDownColor: down,
      priceLineVisible: false,
      lastValueVisible: false,
    });
    candle.setData(cdata);
    if (layout.td9) {
      candle.setMarkers(computeTdSequentialMarkers(pts, timeAt, up, down));
    } else {
      candle.setMarkers([]);
    }

    const oscScaleId = "yj-osc";
    const oscBand = { scaleMargins: { top: 0.64, bottom: 0.02 }, visible: true, borderVisible: false };

    if (osc !== "volume") {
      candle.priceScale().applyOptions({ scaleMargins: { top: 0.06, bottom: 0.36 } });
      if (osc === "macd") {
        const macdPts = computeMacdSeries(pts, timeAt);
        const difLine = chart.addLineSeries({
          color: isLight ? "#c2410c" : "#fbbf24",
          lineWidth: 1,
          priceScaleId: oscScaleId,
          priceLineVisible: false,
          lastValueVisible: false,
        });
        const deaLine = chart.addLineSeries({
          color: isLight ? "#0369a1" : "#38bdf8",
          lineWidth: 1,
          priceScaleId: oscScaleId,
          priceLineVisible: false,
          lastValueVisible: false,
        });
        const macdHist = chart.addHistogramSeries({
          ...histBaseOpts,
          priceScaleId: oscScaleId,
          priceFormat: { type: "price", precision: 4, minMove: 0.0001 },
        });
        difLine.setData(macdPts.map((m) => ({ time: m.time, value: m.dif })));
        deaLine.setData(macdPts.map((m) => ({ time: m.time, value: m.dea })));
        macdHist.setData(
          macdPts.map((m) => ({
            time: m.time,
            value: m.hist,
            color: m.hist >= 0 ? `${up}80` : `${down}80`,
          })),
        );
        macdHist.priceScale().applyOptions(oscBand);
      } else if (osc === "kdj") {
        const kdjPts = computeKdjSeries(pts, timeAt);
        const kLn = chart.addLineSeries({
          color: isLight ? "#c2410c" : "#fbbf24",
          lineWidth: 1,
          priceScaleId: oscScaleId,
          priceLineVisible: false,
          lastValueVisible: false,
        });
        const dLn = chart.addLineSeries({
          color: isLight ? "#0369a1" : "#38bdf8",
          lineWidth: 1,
          priceScaleId: oscScaleId,
          priceLineVisible: false,
          lastValueVisible: false,
        });
        const jLn = chart.addLineSeries({
          color: isLight ? "#7c3aed" : "#c084fc",
          lineWidth: 1,
          priceScaleId: oscScaleId,
          priceLineVisible: false,
          lastValueVisible: false,
        });
        kLn.setData(kdjPts.map((x) => ({ time: x.time, value: x.k })));
        dLn.setData(kdjPts.map((x) => ({ time: x.time, value: x.d })));
        jLn.setData(kdjPts.map((x) => ({ time: x.time, value: x.j })));
        kLn.priceScale().applyOptions(oscBand);
      } else if (osc === "rsi") {
        const rsiPts = computeRsiSeries(pts, timeAt);
        const rsiLine = chart.addLineSeries({
          color: isLight ? "#0f766e" : "#2dd4bf",
          lineWidth: 1,
          priceScaleId: oscScaleId,
          priceLineVisible: false,
          lastValueVisible: false,
        });
        rsiLine.setData(rsiPts);
        const ref = isLight ? "rgba(100,116,139,0.38)" : "rgba(148,163,184,0.38)";
        rsiLine.createPriceLine({
          price: 70,
          color: ref,
          lineWidth: 1,
          lineStyle: LineStyle.Dotted,
          axisLabelVisible: false,
        });
        rsiLine.createPriceLine({
          price: 30,
          color: ref,
          lineWidth: 1,
          lineStyle: LineStyle.Dotted,
          axisLabelVisible: false,
        });
        rsiLine.priceScale().applyOptions(oscBand);
      }
    } else {
      candle.priceScale().applyOptions({ scaleMargins: { top: 0.08, bottom: 0.22 } });
      const hist = chart.addHistogramSeries({
        ...histBaseOpts,
        priceFormat: { type: "volume" },
      });
      hist.priceScale().applyOptions({
        scaleMargins: { top: 0.8, bottom: 0 },
        visible: false,
        borderVisible: false,
      });
      const histData: HistogramData[] = pts.map((p) => ({
        time: timeAt(p.time),
        value: p.volume,
        color: p.close >= p.open ? `${up}55` : `${down}55`,
      }));
      hist.setData(histData);
    }
    chart.timeScale().fitContent();
    chart.timeScale().applyOptions({ rightOffset: 0, fixRightEdge: true });
  }
}

async function scheduleRebuild() {
  await nextTick();
  rebuildChart();
}

watch(
  () => [
    props.chartTab,
    props.loading,
    props.intraday?.points?.length,
    props.kline?.points?.length,
    props.colorScheme,
    props.theme,
    props.prevClose,
    props.indicatorPreset,
  ],
  () => {
    void scheduleRebuild();
  },
);

watch(
  () => props.intraday,
  () => {
    void scheduleRebuild();
  },
  { deep: true },
);

watch(
  () => props.kline,
  () => {
    void scheduleRebuild();
  },
  { deep: true },
);

onMounted(() => {
  void (async () => {
    await nextTick();
    ro = new ResizeObserver(() => {
      requestAnimationFrame(() => {
        if (chartApi.value) resizeToContainer();
        else void scheduleRebuild();
      });
    });
    if (rootEl.value) ro.observe(rootEl.value);
    void scheduleRebuild();
  })();
});

onUnmounted(() => {
  ro?.disconnect();
  ro = null;
  disposeChart();
});
</script>

<template>
  <div class="chart-pane">
    <div
      v-if="showInlineStrip && inlineStats"
      class="chart-pane__strip-header"
      role="group"
      aria-label="成交量额与换手率"
    >
      <div class="chart-pane__strip-item" :title="inlineStats.volumeHint ?? '成交量'">
        <span class="chart-pane__strip-k">量</span>
        <span class="chart-pane__strip-num">{{ inlineStats.volume }}</span>
        <span
          v-if="inlineStats.volumeTrend && inlineStats.volumeTrend !== 'flat'"
          class="chart-pane__strip-arrow"
          :class="stripTrendClass(inlineStats.volumeTrend)"
          aria-hidden="true"
        >{{ inlineStats.volumeTrend === "up" ? "↑" : "↓" }}</span>
        <span v-if="inlineStats.yesterdayInline" class="chart-pane__strip-yest">
          (昨 {{ inlineStats.yesterdayInline.volume }})
        </span>
      </div>
      <div class="chart-pane__strip-item" :title="inlineStats.turnoverHint ?? '成交额'">
        <span class="chart-pane__strip-k">额</span>
        <span class="chart-pane__strip-num">{{ inlineStats.turnover }}</span>
        <span
          v-if="inlineStats.turnoverTrend && inlineStats.turnoverTrend !== 'flat'"
          class="chart-pane__strip-arrow"
          :class="stripTrendClass(inlineStats.turnoverTrend)"
          aria-hidden="true"
        >{{ inlineStats.turnoverTrend === "up" ? "↑" : "↓" }}</span>
        <span v-if="inlineStats.yesterdayInline" class="chart-pane__strip-yest">
          (昨 {{ inlineStats.yesterdayInline.turnover }})
        </span>
      </div>
      <div class="chart-pane__strip-item" :title="inlineStats.turnoverRateHint ?? '换手率'">
        <span class="chart-pane__strip-k">换手</span>
        <span class="chart-pane__strip-num">{{ inlineStats.turnoverRate }}</span>
        <span
          v-if="inlineStats.turnoverRateTrend && inlineStats.turnoverRateTrend !== 'flat'"
          class="chart-pane__strip-arrow"
          :class="stripTrendClass(inlineStats.turnoverRateTrend)"
          aria-hidden="true"
        >{{ inlineStats.turnoverRateTrend === "up" ? "↑" : "↓" }}</span>
        <span v-if="inlineStats.yesterdayInline" class="chart-pane__strip-yest">
          (昨 {{ inlineStats.yesterdayInline.turnoverRate }})
        </span>
      </div>
    </div>
    <div
      ref="plotEl"
      class="chart-pane__plot"
    >
      <div ref="rootEl" class="chart-pane__canvas" />
      <div
        v-if="chartTab === 'intraday' && intradayAxisSlots.length > 0"
        class="chart-pane__intraday-axis"
        aria-hidden="true"
      >
        <span
          v-for="(s, i) in intradayAxisSlots"
          :key="`${s.label}-${i}`"
          class="chart-pane__intraday-axis-tick"
          :style="{ left: `${s.leftPx}px` }"
          >{{ s.label }}</span
        >
      </div>
      <div
        v-if="intradayPrevCloseLabel && prevCloseOverlayTopPx != null"
        class="chart-pane__prev-close-tag"
        :style="{ top: `${prevCloseOverlayTopPx}px` }"
      >
        {{ intradayPrevCloseLabel }}
      </div>


      <div
        v-if="
          chartLegendVisible &&
          (klineMaReadout ||
            klineBollReadout ||
            klineMacdReadout ||
            klineKdjReadout ||
            klineRsiReadout != null)
        "
        class="chart-pane__legend-stack"
      >
        <div
          v-if="klineMaReadout"
          class="chart-pane__legend chart-pane__legend--kline"
          role="status"
          aria-label="均线数值"
        >
          <span class="chart-pane__legend-ma" :style="{ color: maLegendColors.m5 }"
            >MA5 {{ fmtMaCell(klineMaReadout.ma5) }}</span
          >
          <span class="chart-pane__legend-ma" :style="{ color: maLegendColors.m10 }"
            >MA10 {{ fmtMaCell(klineMaReadout.ma10) }}</span
          >
          <span class="chart-pane__legend-ma" :style="{ color: maLegendColors.m20 }"
            >MA20 {{ fmtMaCell(klineMaReadout.ma20) }}</span
          >
        </div>
        <div
          v-if="klineBollReadout"
          class="chart-pane__legend chart-pane__legend--kline"
          role="status"
          aria-label="布林带数值"
        >
          <span class="chart-pane__legend-boll">BOLL上 {{ fmtFixed(klineBollReadout.upper, 2) }}</span>
          <span class="chart-pane__legend-boll">中 {{ fmtFixed(klineBollReadout.mid, 2) }}</span>
          <span class="chart-pane__legend-boll">下 {{ fmtFixed(klineBollReadout.lower, 2) }}</span>
        </div>
        <div
          v-if="klineMacdReadout"
          class="chart-pane__legend chart-pane__legend--kline"
          role="status"
          aria-label="MACD 数值"
        >
          <span class="chart-pane__legend-macd"
            >DIF {{ fmtFixed(klineMacdReadout.dif, 3) }} · DEA
            {{ fmtFixed(klineMacdReadout.dea, 3) }} · MACD
            {{ fmtFixed(klineMacdReadout.hist, 3) }}</span
          >
        </div>
        <div
          v-if="klineKdjReadout"
          class="chart-pane__legend chart-pane__legend--kline"
          role="status"
          aria-label="KDJ 数值"
        >
          <span class="chart-pane__legend-kdj"
            >K {{ fmtFixed(klineKdjReadout.k, 2) }} · D {{ fmtFixed(klineKdjReadout.d, 2) }} · J
            {{ fmtFixed(klineKdjReadout.j, 2) }}</span
          >
        </div>
        <div
          v-if="klineRsiReadout != null"
          class="chart-pane__legend chart-pane__legend--kline"
          role="status"
          aria-label="RSI 数值"
        >
          <span class="chart-pane__legend-rsi">RSI(14) {{ fmtFixed(klineRsiReadout, 2) }}</span>
        </div>
      </div>
    </div>
    <div v-if="loading" class="chart-pane__overlay">加载图表…</div>
    <div
      v-else-if="chartTab === 'intraday' && !(intraday?.points?.length)"
      class="chart-pane__overlay"
    >
      暂无分时数据
    </div>
    <div
      v-else-if="chartTab !== 'intraday' && !(kline?.points?.length)"
      class="chart-pane__overlay"
    >
      暂无 K 线数据
    </div>
  </div>
</template>

<style scoped>
.chart-pane {
  position: relative;
  width: 100%;
  flex: 1;
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.chart-pane__plot {
  position: relative;
  flex: 1 1 auto;
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.chart-pane__canvas {
  flex: 1 1 auto;
  min-width: 0;
  min-height: 0;
  width: 100%;
}

.chart-pane__intraday-axis {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 3px;
  height: 14px;
  pointer-events: none;
  z-index: 2;
  font-size: 0.65em;
  line-height: 1;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  color: color-mix(in srgb, var(--yj-th-color, rgba(245, 245, 245, 0.75)) 88%, transparent);
  text-shadow: 0 1px 2px color-mix(in srgb, var(--yj-settings-bg-1, #000) 85%, transparent);
}

.chart-pane__intraday-axis-tick {
  position: absolute;
  transform: translateX(-50%);
  white-space: nowrap;
}

.chart-pane__prev-close-tag {
  position: absolute;
  left: 4px;
  z-index: 3;
  transform: translateY(-50%);
  pointer-events: none;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 0.68em;
  line-height: 1.25;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  color: color-mix(in srgb, var(--yj-text, #e5e5e5) 92%, transparent);
  background: color-mix(in srgb, var(--yj-settings-bg-1, #0a0a0a) 78%, #fff);
  border: 1px solid color-mix(in srgb, var(--yj-text-muted, #888) 35%, transparent);
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.35);
  max-width: calc(100% - 56px);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.chart-pane__legend-stack {
  position: absolute;
  left: 6px;
  right: 52px;
  top: 6px;
  z-index: 2;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 4px;
  max-width: none;
  pointer-events: none;
}

.chart-pane__legend {
  padding: 3px 8px;
  border-radius: 6px;
  font-size: 0.62em;
  line-height: 1.38;
  font-variant-numeric: tabular-nums;
  background: color-mix(in srgb, var(--yj-settings-bg-1, #0a0a0a) 84%, transparent);
  border: 1px solid color-mix(in srgb, var(--yj-text-muted, #888) 32%, transparent);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.35);
  width: fit-content;
  max-width: min(20rem, calc(100% - 58px));
  box-sizing: border-box;
}

.chart-pane__legend--intraday {
  position: absolute;
  left: 6px;
  top: 6px;
  z-index: 2;
  pointer-events: none;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 2px;
  text-align: left;
  width: fit-content;
  max-width: min(18rem, calc(100% - 58px));
  box-sizing: border-box;
}

.chart-pane__legend-main {
  font-weight: 600;
  color: color-mix(in srgb, var(--yj-text, #e5e5e5) 96%, transparent);
}

.chart-pane__legend-sub {
  font-size: 0.92em;
  color: var(--yj-text-muted);
  max-width: 14rem;
  line-height: 1.3;
}

.chart-pane__legend--kline {
  display: inline-flex;
  flex-wrap: wrap;
  gap: 6px 10px;
  justify-content: flex-start;
  text-align: left;
}

.chart-pane__legend-ma {
  font-weight: 600;
}

.chart-pane__legend-boll,
.chart-pane__legend-macd,
.chart-pane__legend-kdj,
.chart-pane__legend-rsi {
  color: color-mix(in srgb, var(--yj-text, #e5e5e5) 93%, transparent);
  font-weight: 500;
}

.chart-pane__strip-header {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 4px 12px;
  padding: 4px 6px 2px;
  font-size: 0.72em;
  color: var(--yj-text-muted);
  flex-shrink: 0;
}

.chart-pane__strip-item {
  display: flex;
  align-items: baseline;
  gap: 3px;
}

.chart-pane__strip-k {
  opacity: 0.88;
  font-weight: 500;
}

.chart-pane__strip-num {
  font-family: "DM Sans", "Noto Sans SC", sans-serif;
  font-variant-numeric: tabular-nums;
}

.chart-pane__strip-arrow {
  margin-left: 1px;
  font-weight: 700;
  font-size: 1.05em;
  line-height: 1;
}

.chart-pane__strip-yest {
  opacity: 0.65;
  font-size: 0.95em;
  margin-left: 2px;
}

.chart-pane__strip-arrow.up-red {
  color: #fb7185;
}
.chart-pane__strip-arrow.down-green {
  color: #4ade80;
}
.chart-pane__strip-arrow.up-green {
  color: #4ade80;
}
.chart-pane__strip-arrow.down-red {
  color: #fb7185;
}
.chart-pane__strip-arrow.flat {
  color: var(--yj-flat);
}

.chart-pane__overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.8em;
  color: var(--yj-text-muted);
  pointer-events: none;
  background: color-mix(in srgb, var(--yj-settings-bg-1) 40%, transparent);
  z-index: 4;
}
</style>
