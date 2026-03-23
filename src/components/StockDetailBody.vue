<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useSettings } from "../composables/useSettings";
import { useStockDetailMarket } from "../composables/useStockDetailMarket";
import { resolveTargetGroupId } from "../constants/watchGroup";
import { stripCodeFromAllGroups } from "../utils/watchGroups";
import { useQuotes } from "../composables/useQuotes";
import type { QuoteRow } from "../types/app";
import type {
  ChartIndicatorPresetId,
  ChartInlineStats,
  ChartTabId,
  OrderBook,
} from "../types/marketDetail";
import {
  chartIndicatorOptionsForTab,
  DEFAULT_CHART_INDICATOR_BY_TAB,
  normalizeIndicatorPresetForTab,
} from "../constants/chartIndicatorPreset";
import {
  changeClass,
  fmtDeltaTurnoverCn,
  fmtFixed,
  fmtTurnoverCn,
  fmtVolume,
} from "../utils/format";
import { shouldShowStockDetailOrderBook } from "../utils/orderBookEligibility";
import { displayStockName } from "../utils/stockDisplay";
import OrderBookPanel from "./OrderBookPanel.vue";
import StockChartPane from "./StockChartPane.vue";
import YjSelect from "./YjSelect.vue";

const props = withDefaults(
  defineProps<{
    code: string;
    /** 主面板右侧抽屉：显示顶栏（关闭 + 标题） */
    embedded?: boolean;
  }>(),
  { embedded: false },
);

const emit = defineEmits<{
  close: [];
  "update:title": [string];
}>();

const { settings, save } = useSettings();

const codeNorm = computed(() => props.code.trim().toLowerCase());

const codesRef = computed(() => (codeNorm.value ? [codeNorm.value] : []));
const quoteSourceRef = computed(() => settings.value?.quoteSource ?? "eastmoney");

const { rows, err, start } = useQuotes(codesRef, quoteSourceRef, 3500);

const row = computed<QuoteRow | null>(() => rows.value[0] ?? null);

const titleName = computed(() => {
  const r = row.value;
  if (!r) return codeNorm.value || "个股";
  return displayStockName(r, settings.value);
});

watch(
  titleName,
  (t) => emit("update:title", t),
  { immediate: true },
);

const inWatchlist = computed(() => {
  const c = codeNorm.value;
  if (!c || !settings.value) return false;
  return settings.value.watchlist.some((w) => w.code.toLowerCase() === c);
});

onMounted(() => {
  // 勿调用全局 load()：会与 App 共用 loading，主面板 v-if="loading" 会卸掉整页含抽屉。
  start();
});

async function toggleWatch() {
  if (!settings.value || !codeNorm.value) return;
  const c = codeNorm.value;
  const list = settings.value.watchlist;
  const i = list.findIndex((w) => w.code.toLowerCase() === c);
  if (i >= 0) {
    list.splice(i, 1);
    stripCodeFromAllGroups(settings.value.watchGroups, c);
    await save();
    return;
  }
  const r = row.value;
  const name = r ? displayStockName(r, settings.value) : c;
  list.push({ code: c, name: name !== c ? name : c });
  const gid = resolveTargetGroupId(settings.value, undefined);
  const g = settings.value.watchGroups.find((x) => x.id === gid);
  if (g && !g.codes.some((x) => x.toLowerCase() === c)) {
    g.codes.push(c);
  }
  await save();
}

const changeAmt = computed(() => {
  const r = row.value;
  if (!r) return 0;
  return r.price - r.prevClose;
});

const {
  chartTab,
  intradaySeries,
  currentKline,
  orderBook,
  chartLoading,
  chartErr,
  bookLoading,
  bookErr,
} = useStockDetailMarket(
  () => codeNorm.value,
  () => settings.value?.quoteSource ?? "eastmoney",
);

const chartTabs: { id: ChartTabId; label: string }[] = [
  { id: "intraday", label: "分时" },
  { id: "day", label: "日K" },
  { id: "week", label: "周K" },
  { id: "month", label: "月K" },
];

/** 各周期独立记忆下拉指标 */
const indicatorPresetByTab = ref<Record<ChartTabId, ChartIndicatorPresetId>>({
  ...DEFAULT_CHART_INDICATOR_BY_TAB,
});

watch(codeNorm, () => {
  indicatorPresetByTab.value = { ...DEFAULT_CHART_INDICATOR_BY_TAB };
});

const chartIndicatorOptions = computed(() => chartIndicatorOptionsForTab(chartTab.value));

const paneIndicatorPreset = computed(() =>
  normalizeIndicatorPresetForTab(chartTab.value, indicatorPresetByTab.value[chartTab.value]),
);

function onIndicatorPresetPick(v: string) {
  const tab = chartTab.value;
  indicatorPresetByTab.value = {
    ...indicatorPresetByTab.value,
    [tab]: normalizeIndicatorPresetForTab(tab, v),
  };
}

function dayStatTrend(curr: number, prev: number) {
  if (!Number.isFinite(curr) || !Number.isFinite(prev) || prev <= 0) return "flat" as const;
  const ratio = (curr - prev) / prev;
  if (ratio > 1e-7) return "up" as const;
  if (ratio < -1e-7) return "down" as const;
  return "flat" as const;
}

function dayCompareHint(label: string, curr: number, prev: number) {
  if (!Number.isFinite(curr) || !Number.isFinite(prev) || prev <= 0) return label;
  const pct = ((curr - prev) / prev) * 100;
  const sign = pct > 0 ? "+" : "";
  return `${label} 较上一交易日 ${sign}${pct.toFixed(2)}%`;
}

/** 小窗：量/额/换手叠在图上；周 K / 月 K 不展示；日 K 与上一根日 K 对比涨跌箭头 */
const chartInlineStats = computed((): ChartInlineStats | null => {
  if (!props.embedded || !row.value) return null;
  if (chartTab.value === "week" || chartTab.value === "month") return null;
  const r = row.value;
  const base: ChartInlineStats = {
    volume: fmtVolume(r.volume),
    turnover: fmtVolume(r.turnover),
    turnoverRate: `${fmtFixed(r.turnoverRate, 2)}%`,
  };
  if (chartTab.value !== "day") return base;
  const pts = currentKline.value?.points;
  if (!pts || pts.length < 2) return base;
  const prevK = pts[pts.length - 2]!;
  const pv = prevK.volume;
  const pt = prevK.turnover ?? 0;
  const pr = prevK.turnoverRate ?? 0;
  return {
    ...base,
    volumeTrend: dayStatTrend(r.volume, pv),
    volumeHint: dayCompareHint("成交量", r.volume, pv),
    turnoverTrend: pt > 0 ? dayStatTrend(r.turnover, pt) : "flat",
    turnoverHint: pt > 0 ? dayCompareHint("成交额", r.turnover, pt) : "成交额",
    turnoverRateTrend: pr > 0 ? dayStatTrend(r.turnoverRate, pr) : "flat",
    turnoverRateHint: pr > 0 ? dayCompareHint("换手率", r.turnoverRate, pr) : "换手率",
    yesterdayInline: {
      volume: fmtVolume(pv),
      turnover: pt > 0 ? fmtVolume(pt) : "—",
      turnoverRate: pr > 0 ? `${fmtFixed(pr, 2)}%` : "—",
    },
  };
});

/** 快照昨收：与列表涨跌幅同源，避免与分时接口 preClose 不一致 */
const chartPrevClose = computed(() => {
  const r = row.value;
  if (!r) return null;
  const p = r.prevClose;
  return p > 0 && Number.isFinite(p) ? p : null;
});

/** 日 K 无盘口；分时先出图+盘口，继续拉宽才出左侧集合竞价（不在图下方展示） */
const INTRADAY_BOOK_MIN_PX = 380;
const INTRADAY_AUCTION_MIN_PX = 640;

const marketBlockEl = ref<HTMLElement | null>(null);
const marketBlockWidth = ref(0);
let marketBlockRo: ResizeObserver | null = null;

watch(
  () => marketBlockEl.value,
  (el) => {
    marketBlockRo?.disconnect();
    marketBlockRo = null;
    if (!el || typeof ResizeObserver === "undefined") return;
    const ro = new ResizeObserver((entries) => {
      marketBlockWidth.value = Math.round(entries[0]?.contentRect.width ?? 0);
    });
    ro.observe(el);
    marketBlockRo = ro;
    marketBlockWidth.value = Math.round(el.getBoundingClientRect().width);
  },
  { immediate: true, flush: "post" },
);

onUnmounted(() => {
  marketBlockRo?.disconnect();
  marketBlockRo = null;
});

const showIntradayBook = computed(
  () => chartTab.value === "intraday" && marketBlockWidth.value >= INTRADAY_BOOK_MIN_PX,
);

const showIntradayAuctionRail = computed(
  () =>
    chartTab.value === "intraday" &&
    marketBlockWidth.value >= INTRADAY_AUCTION_MIN_PX &&
    intradaySeries.value?.auction != null,
);

/** 仅分时且宽度足够时展示盘口；日 / 周 / 月 K 不展示 */
const showBookColumn = computed(() => {
  if (!shouldShowStockDetailOrderBook(codeNorm.value)) return false;
  if (chartTab.value !== "intraday") return false;
  return showIntradayBook.value;
});

/** 五档接口空时用列表快照兜底；量为 0 或与列表 f24/f36 不一致时取较大值（东财五档偶发偏小、腾讯补量仅在全档无总量时触发） */
const orderBookForPanel = computed((): OrderBook | null => {
  const raw = orderBook.value;
  const r = row.value;
  let b = raw;
  if (b && r && b.bids.length > 0) {
    const top = b.bids[0]!;
    const qv = Math.max(0, Math.round(r.bid1Vol ?? 0));
    const ok =
      !Number.isFinite(r.bid1) ||
      r.bid1 <= 0 ||
      Math.abs(top.price - r.bid1) <= Math.max(r.bid1 * 1.5e-4, 0.02);
    if (ok && qv > 0) {
      const merged = Math.max(top.volume, qv);
      if (merged !== top.volume) {
        b = {
          ...b,
          bids: b.bids.map((lv, i) => (i === 0 ? { ...lv, volume: merged } : lv)),
        };
      }
    }
  }
  if (b && r && b.asks.length > 0) {
    const sell1 = b.asks[b.asks.length - 1]!;
    const av = Math.max(0, Math.round(r.ask1Vol ?? 0));
    const ok =
      !Number.isFinite(r.ask1) ||
      r.ask1 <= 0 ||
      Math.abs(sell1.price - r.ask1) <= Math.max(r.ask1 * 1.5e-4, 0.02);
    if (ok && av > 0) {
      const merged = Math.max(sell1.volume, av);
      if (merged !== sell1.volume) {
        const last = b.asks.length - 1;
        b = {
          ...b,
          asks: b.asks.map((lv, i) => (i === last ? { ...lv, volume: merged } : lv)),
        };
      }
    }
  }
  /* 接口偶发只给一侧档位时，上面会直接 return，买/卖一缺失；用列表 f19/f31 快照补全 */
  if (b && r) {
    let bids = b.bids;
    let asks = b.asks;
    if (bids.length === 0 && r.bid1 > 0 && Number.isFinite(r.bid1)) {
      const bv = Math.max(0, Math.round(r.bid1Vol ?? 0));
      bids = [{ price: r.bid1, volume: bv }];
    }
    if (asks.length === 0 && r.ask1 > 0 && Number.isFinite(r.ask1)) {
      const av = Math.max(0, Math.round(r.ask1Vol ?? 0));
      asks = [{ price: r.ask1, volume: av }];
    }
    if (bids !== b.bids || asks !== b.asks) {
      b = {
        ...b,
        bids,
        asks,
        maxLevels: Math.max(b.maxLevels ?? 1, 1),
      };
    }
  }
  if (b && (b.asks.length > 0 || b.bids.length > 0)) return b;
  if (!showBookColumn.value) return b;
  if (!r) return b;
  const bv = Math.max(0, Math.round(r.bid1Vol ?? 0));
  const av = Math.max(0, Math.round(r.ask1Vol ?? 0));
  const bids =
    r.bid1 > 0 && Number.isFinite(r.bid1) ? [{ price: r.bid1, volume: bv }] : [];
  const asks =
    r.ask1 > 0 && Number.isFinite(r.ask1) ? [{ price: r.ask1, volume: av }] : [];
  if (bids.length === 0 && asks.length === 0) return b;
  return { asks, bids, maxLevels: 1 };
});

/** 分时昨收/零轴在视口中的 Y（与盘口 DOM 同一坐标系），来自 StockChartPane */
const chartBaselineY = ref<number | null>(null);

function setChartBaselineY(y: number | null) {
  chartBaselineY.value = y;
}

/** 仅横向并排（图+盘口）时：一档买卖分界与零轴对齐 */
const alignOrderBookToBaseline = computed(() => {
  if (chartTab.value !== "intraday") return false;
  if (!showBookColumn.value) return false;
  if (chartBaselineY.value == null) return false;
  if (props.embedded && marketBlockWidth.value > 0 && marketBlockWidth.value < 440) return false;
  const b = orderBookForPanel.value;
  if (!b || bookLoading.value) return false;
  return b.asks.length === 1 && b.bids.length === 1;
});

const intradayAuction = computed(() => intradaySeries.value?.auction ?? null);

const intradayAuctionDeltaLabel = computed(() => {
  const a = intradayAuction.value;
  if (
    a == null ||
    a.prevRefTurnover == null ||
    !Number.isFinite(a.prevRefTurnover) ||
    a.prevRefTurnover <= 0 ||
    !Number.isFinite(a.matchTurnover)
  ) {
    return null;
  }
  return fmtDeltaTurnoverCn(a.matchTurnover - a.prevRefTurnover);
});

</script>

<template>
  <div class="stock-detail-body" :class="{ 'stock-detail-body--embedded': embedded }">
    <div v-if="embedded" class="stock-detail-body__head">
      <button
        type="button"
        class="stock-detail-body__close"
        aria-label="关闭详情"
        @click="emit('close')"
      >
        ×
      </button>
      <div class="stock-detail-body__head-row">
        <h2 class="stock-detail-body__title">{{ titleName }}</h2>
        <div v-if="row" class="stock-detail-body__head-quote">
          <span
            class="stock-detail-body__head-price"
            :class="changeClass(row.changePct, settings?.colorScheme ?? 'redUp')"
          >
            <span
              v-if="row.changePct > 0 || row.changePct < 0"
              class="quote-chg-arrow"
              aria-hidden="true"
            >
              <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                <path
                  :d="
                    row.changePct > 0
                      ? 'M12 4 20 18 4 18z'
                      : 'M12 20 4 6 20 6z'
                  "
                />
              </svg>
            </span>
            {{ fmtFixed(row.price, 2) }}
          </span>
          <span
            class="stock-detail-body__head-chip"
            :class="changeClass(row.changePct, settings?.colorScheme ?? 'redUp')"
          >
            <span class="stock-detail-body__head-pct">{{ fmtFixed(row.changePct, 2) }}%</span>
            <span class="stock-detail-body__head-chip-sep" aria-hidden="true">/</span>
            <span class="stock-detail-body__head-amt">{{ fmtFixed(changeAmt, 2) }}</span>
          </span>
        </div>
      </div>
    </div>

    <div class="stock-detail-body__main">
      <div v-if="err" class="err">{{ err }}</div>

      <template v-if="row">
        <section v-if="!embedded" class="hero">
          <div class="price-line">
            <div class="price-line__cluster">
              <div class="price-line__values">
                <span
                  class="price"
                  :class="changeClass(row.changePct, settings?.colorScheme ?? 'redUp')"
                >
                  <span
                    v-if="row.changePct > 0 || row.changePct < 0"
                    class="quote-chg-arrow quote-chg-arrow--hero"
                    aria-hidden="true"
                  >
                    <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                      <path
                        :d="
                          row.changePct > 0
                            ? 'M12 4 20 18 4 18z'
                            : 'M12 20 4 6 20 6z'
                        "
                      />
                    </svg>
                  </span>
                  {{ fmtFixed(row.price, 2) }}
                </span>
                <span
                  class="pct"
                  :class="changeClass(row.changePct, settings?.colorScheme ?? 'redUp')"
                >
                  {{ fmtFixed(row.changePct, 2) }}%
                </span>
                <span
                  class="amt"
                  :class="changeClass(row.changePct, settings?.colorScheme ?? 'redUp')"
                >
                  {{ fmtFixed(changeAmt, 2) }}
                </span>
              </div>
              <span class="code">{{ row.code }}</span>
            </div>
            <button
              type="button"
              class="btn hero-watch"
              :class="{ primary: !inWatchlist }"
              :title="inWatchlist ? '从自选移除' : '加入自选'"
              @click="toggleWatch"
            >
              {{ inWatchlist ? "移除" : "自选" }}
            </button>
          </div>
        </section>

        <section v-if="!embedded" class="grid">
          <div class="cell">
            <span class="k">今开</span>
            <span class="v">{{ fmtFixed(row.open, 2) }}</span>
          </div>
          <div class="cell">
            <span class="k">昨收</span>
            <span class="v">{{ fmtFixed(row.prevClose, 2) }}</span>
          </div>
          <div class="cell">
            <span class="k">最高</span>
            <span class="v">{{ fmtFixed(row.high, 2) }}</span>
          </div>
          <div class="cell">
            <span class="k">最低</span>
            <span class="v">{{ fmtFixed(row.low, 2) }}</span>
          </div>
          <div class="cell">
            <span class="k">成交量</span>
            <span class="v">{{ fmtVolume(row.volume) }}</span>
          </div>
          <div class="cell">
            <span class="k">成交额</span>
            <span class="v">{{ fmtVolume(row.turnover) }}</span>
          </div>
          <div class="cell">
            <span class="k">换手率</span>
            <span class="v">{{ fmtFixed(row.turnoverRate, 2) }}%</span>
          </div>
        </section>

        <section ref="marketBlockEl" class="market-block">
          <div v-if="chartErr" class="err market-block__err">{{ chartErr }}</div>
          <div v-if="bookErr && showBookColumn" class="err market-block__err">{{ bookErr }}</div>
          <div class="chart-tabs-row">
            <div class="chart-tabs">
              <button
                v-for="t in chartTabs"
                :key="t.id"
                type="button"
                class="chart-tab"
                :class="{ 'chart-tab--on': chartTab === t.id }"
                @click="chartTab = t.id"
              >
                {{ t.label }}
              </button>
            </div>
            <div class="chart-subtools">
              <YjSelect
                class="chart-indicator-select"
                :model-value="paneIndicatorPreset"
                :options="chartIndicatorOptions"
                aria-label="图表指标"
                @update:model-value="onIndicatorPresetPick"
              />
            </div>
          </div>
          <div
            class="market-split"
            :class="{
              'market-split--no-book': !showBookColumn,
              'market-split--intraday-auction': showIntradayAuctionRail,
            }"
          >
            <div
              v-if="showIntradayAuctionRail && intradayAuction"
              class="intraday-auction-rail"
              role="group"
              aria-label="集合竞价摘要"
            >
              <div class="intraday-auction-rail__head">
                <span class="intraday-auction-rail__title">集合竞价</span>
                <div
                  class="intraday-auction-rail__pill"
                  :class="changeClass(intradayAuction.pctVsPreClose, settings?.colorScheme ?? 'redUp')"
                >
                  <div class="intraday-auction-rail__pill-row">
                    <span class="intraday-auction-rail__pill-k">撮合价</span>
                    <span class="intraday-auction-rail__price">{{
                      fmtFixed(intradayAuction.matchPrice, 2)
                    }}</span>
                  </div>
                  <div class="intraday-auction-rail__pill-row intraday-auction-rail__pill-row--pct">
                    <span class="intraday-auction-rail__pill-k">较昨收</span>
                    <span class="intraday-auction-rail__pct"
                      >{{ fmtFixed(intradayAuction.pctVsPreClose, 2) }}%</span
                    >
                  </div>
                </div>
              </div>
              <div class="intraday-auction-rail__stats" role="list">
                <div class="intraday-auction-rail__stat" role="listitem">
                  <span class="intraday-auction-rail__k">成交量</span>
                  <span class="intraday-auction-rail__num">
                    {{ fmtVolume(intradayAuction.matchVolumeLots) }} 手
                  </span>
                </div>
                <div class="intraday-auction-rail__stat" role="listitem">
                  <span class="intraday-auction-rail__k">成交额</span>
                  <span class="intraday-auction-rail__num">
                    {{ fmtTurnoverCn(intradayAuction.matchTurnover) }}
                  </span>
                </div>
                <div
                  v-if="intradayAuctionDeltaLabel != null"
                  class="intraday-auction-rail__stat intraday-auction-rail__stat--ref"
                  role="listitem"
                  title="与上一交易日 9:30 首分钟成交额之差（口径与竞价额不同，仅供参考）"
                >
                  <span class="intraday-auction-rail__k">昨首差</span>
                  <span class="intraday-auction-rail__num">{{ intradayAuctionDeltaLabel }}</span>
                </div>
              </div>
            </div>
            <div class="chart-col">
              <StockChartPane
                :chart-tab="chartTab"
                :intraday="intradaySeries"
                :kline="currentKline"
                :loading="chartLoading"
                :color-scheme="settings?.colorScheme ?? 'redUp'"
                :theme="settings?.theme ?? 'dark'"
                :inline-stats="chartInlineStats"
                :prev-close="chartPrevClose"
                :indicator-preset="paneIndicatorPreset"
                @intraday-baseline-y="setChartBaselineY"
              />
            </div>
            <div
              v-if="showBookColumn"
              class="book-col"
              :class="{ 'book-col--baseline': alignOrderBookToBaseline }"
            >
              <div class="book-panel">
                <OrderBookPanel
                  :book="orderBookForPanel"
                  :loading="bookLoading"
                  :color-scheme="settings?.colorScheme ?? 'redUp'"
                  :last-price="row.price"
                  :chart-baseline-y="chartBaselineY"
                  :align-bid-ask-mid-to-baseline="alignOrderBookToBaseline"
                />
              </div>
            </div>
          </div>
        </section>
      </template>

      <p v-else-if="!err" class="hint">加载行情…</p>
    </div>
  </div>
</template>

<style scoped>
.stock-detail-body {
  display: flex;
  flex-direction: column;
  min-height: 0;
  min-width: 0;
  /* 侧栏 detail-rail 等为纵向 flex 时用 flex 占满高度；单独 height:100% 在部分嵌套下无法把剩余高度传给图表区 */
  flex: 1 1 auto;
  height: 100%;
  color: var(--yj-text);
  container-type: size;
  container-name: stockdetail;
  /* 分时：集合竞价与盘口窄幅即可容纳价量，把横向留给主图 */
  --yj-auction-rail-w: 96px;
  --yj-order-book-w: 96px;
  /* 侧栏卡片（集合竞价 / 盘口）共用 */
  --yj-side-rail-border: color-mix(in srgb, var(--yj-row-border) 84%, transparent);
  --yj-side-rail-surface-1: color-mix(in srgb, var(--yj-table-wrap-bg) 88%, transparent);
  --yj-side-rail-surface-2: color-mix(in srgb, var(--yj-table-wrap-bg) 72%, transparent);
}

.stock-detail-body--embedded {
  background: linear-gradient(
    180deg,
    var(--yj-settings-bg-1),
    var(--yj-settings-bg-2)
  );
}

.stock-detail-body__head {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
  padding: 8px 10px 8px 8px;
  border-bottom: 1px solid var(--yj-bar-border);
  background: color-mix(in srgb, var(--yj-table-wrap-bg) 55%, transparent);
  -webkit-app-region: no-drag;
  app-region: no-drag;
}

.stock-detail-body__head-row {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: row;
  flex-wrap: nowrap;
  align-items: center;
  gap: 8px 10px;
}

.stock-detail-body__head-quote {
  display: flex;
  flex-wrap: nowrap;
  align-items: baseline;
  gap: 8px 10px;
  flex-shrink: 0;
}

.stock-detail-body__head-price {
  display: inline-flex;
  align-items: baseline;
  gap: 3px;
  font-family: "DM Sans", "Noto Sans SC", sans-serif;
  font-size: 1.02em;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  letter-spacing: -0.02em;
}

.quote-chg-arrow {
  display: inline-flex;
  flex-shrink: 0;
  align-self: center;
  line-height: 0;
}

.quote-chg-arrow svg {
  width: 0.62em;
  height: 0.62em;
  display: block;
  transform: translateY(0.06em);
}

.quote-chg-arrow--hero svg {
  width: 0.58em;
  height: 0.58em;
  transform: translateY(0.05em);
}

.stock-detail-body__head-chip {
  display: inline-flex;
  align-items: baseline;
  gap: 4px;
  padding: 2px 8px;
  border-radius: 6px;
  font-family: "DM Sans", "Noto Sans SC", sans-serif;
  font-size: 0.74em;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  border: 1px solid color-mix(in srgb, currentColor 28%, transparent);
  background: color-mix(in srgb, currentColor 12%, transparent);
}

.stock-detail-body__head-chip.flat {
  border-color: var(--yj-row-border);
  background: color-mix(in srgb, var(--yj-table-wrap-bg) 80%, transparent);
}

.stock-detail-body__head-chip-sep {
  opacity: 0.45;
  font-weight: 500;
  user-select: none;
}

.stock-detail-body__head-pct,
.stock-detail-body__head-amt {
  font-weight: 600;
}

.stock-detail-body__close {
  flex-shrink: 0;
  width: 22px;
  height: 22px;
  padding: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  border: 1px solid var(--yj-icon-border);
  background: var(--yj-icon-bg);
  color: var(--yj-icon-color);
  font-size: 1rem;
  line-height: 1;
  cursor: pointer;
}

.stock-detail-body__close:hover {
  background: var(--yj-icon-hover-bg);
}

.stock-detail-body__title {
  flex: 1 1 0;
  margin: 0;
  font-size: 0.88em;
  font-weight: 600;
  letter-spacing: 0.03em;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  line-height: 1.2;
}

/* 侧栏：标题不占满整行，把宽度留给右侧报价 */
.stock-detail-body--embedded .stock-detail-body__title {
  flex: 0 1 auto;
  max-width: min(11em, calc(100% - 8.5rem));
}

.stock-detail-body__main {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 12px 14px 16px;
  -webkit-app-region: no-drag;
  app-region: no-drag;
}

/* 抽屉：主区纵向 flex；指标在价量与图表之间；超出时可纵向滚动 */
.stock-detail-body--embedded .stock-detail-body__main {
  overflow-x: hidden;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  /* 略收左右内边距，让分时+盘口更贴边，减轻「图与盘口中间空一截」观感 */
  padding: 10px 8px 12px;
}

.stock-detail-body--embedded .stock-detail-body__main > .err,
.stock-detail-body--embedded .stock-detail-body__main > .hint {
  order: 0;
  flex-shrink: 0;
}

/* 抽屉：顶栏含价量，主区直接进图表 */
.stock-detail-body--embedded .stock-detail-body__main > section.market-block {
  flex: 1 1 auto;
  display: flex;
  flex-direction: column;
  margin-top: 0;
  min-height: 0;
}

/* 吃掉 market-block 里除 Tab 外的纵向剩余空间，否则拉高窗口时下方会空一大块 */
.stock-detail-body--embedded .market-split {
  flex: 1 1 auto;
  min-height: 0;
}

/* 窄屏规则里已有 flex:1；这里保证任意宽度下图表列都是纵向 flex，子组件 chart-pane 的 flex:1 才能生效 */
.stock-detail-body--embedded .chart-col {
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.err {
  color: var(--yj-err);
  font-size: 0.88em;
  margin-bottom: 12px;
}

.hero {
  margin-bottom: 14px;
}

.price-line {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  justify-content: space-between;
  gap: 8px 10px;
}

.price-line__cluster {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
  flex: 1;
}

.price-line__values {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 10px 14px;
  min-width: 0;
}

.price {
  display: inline-flex;
  align-items: baseline;
  gap: 4px;
  font-family: "DM Sans", "Noto Sans SC", sans-serif;
  font-size: 1.35em;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  letter-spacing: -0.02em;
}

.pct,
.amt {
  font-family: "DM Sans", "Noto Sans SC", sans-serif;
  font-size: 0.9em;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}

.code {
  font-family: "DM Sans", "Noto Sans SC", ui-monospace, monospace;
  font-variant-numeric: tabular-nums;
  font-size: 0.78em;
  color: var(--yj-text-muted);
}

.grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 8px 10px;
  margin-bottom: 12px;
}

.cell {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 10px;
  border-radius: 8px;
  border: 1px solid var(--yj-row-border);
  background: var(--yj-table-wrap-bg);
  font-size: 0.82em;
}

.k {
  color: var(--yj-text-muted);
}

.v {
  font-family: "DM Sans", "Noto Sans SC", sans-serif;
  font-variant-numeric: tabular-nums;
  font-weight: 500;
}

.btn {
  border-radius: 8px;
  border: 1px solid var(--yj-btn-border);
  background: var(--yj-btn-bg);
  color: var(--yj-btn-color);
  padding: 10px 18px;
  cursor: pointer;
  font-size: 0.82em;
}

.btn.primary {
  background: var(--yj-btn-primary-bg);
  border-color: var(--yj-btn-primary-border);
  color: var(--yj-btn-primary-color);
}

.btn.hero-watch {
  flex-shrink: 0;
  padding: 3px 8px;
  font-size: 0.72em;
  font-weight: 500;
  line-height: 1.25;
  border-radius: 6px;
  align-self: center;
}

.hint {
  color: var(--yj-text-muted);
  font-size: 0.8em;
}

.market-block {
  margin-top: 4px;
}

.stock-detail-body--embedded .chart-tabs-row {
  flex-shrink: 0;
}

.market-block__err {
  margin-bottom: 8px;
}

.chart-tabs-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  margin-bottom: 10px;
}

.chart-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 0;
}

.chart-subtools {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: flex-end;
  gap: 4px;
  flex-shrink: 0;
  min-width: 0;
}

.chart-indicator-select {
  width: min(100%, 8rem);
  min-width: 4.75rem;
}

/* 覆盖 App.vue 里 yj-select 的 8px/12px，与左侧周期 chip 同量级 */
.chart-tabs-row .chart-indicator-select :deep(button.yj-field-control.yj-select-trigger) {
  padding: 3px 8px;
  font-size: 0.72em;
  line-height: 1.25;
  font-weight: 500;
  border-radius: 6px;
  gap: 4px;
}

.chart-tabs-row .chart-indicator-select :deep(.yj-select-trigger__chev) {
  font-size: 0.62em;
  opacity: 0.7;
}

.chart-tab {
  border-radius: 6px;
  border: 1px solid var(--yj-chip-border);
  background: var(--yj-chip-bg);
  color: var(--yj-chip-color);
  padding: 4px 10px;
  font-size: 0.72em;
  font-weight: 500;
  cursor: pointer;
  font-family: inherit;
}

.chart-tab--on {
  background: var(--yj-chip-on-bg);
  border-color: var(--yj-chip-on-border);
  color: var(--yj-chip-on-color);
}

.market-split {
  display: flex;
  flex-direction: row;
  gap: 4px;
  align-items: stretch;
  min-height: 0;
}

/* 日 K：无盘口列，K 线占满横向宽度 */
.market-split--no-book {
  gap: 0;
}

.market-split--no-book .chart-col {
  flex: 1 1 auto;
  width: 100%;
  min-width: 0;
}

.stock-detail-body--embedded .market-split--no-book {
  flex: 1 1 auto;
  min-height: 0;
}

/* 分时左侧：集合竞价（窗口拉宽至 INTRADAY_AUCTION_MIN_PX 后才出现，不在图下） */
.intraday-auction-rail {
  flex: 0 0 var(--yj-auction-rail-w);
  width: var(--yj-auction-rail-w);
  min-width: max(108px, calc(var(--yj-auction-rail-w) - 10px));
  max-width: calc(var(--yj-auction-rail-w) + 14px);
  align-self: stretch;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 7px 7px 8px;
  border-radius: 10px;
  border: 1px solid var(--yj-side-rail-border);
  background: linear-gradient(
    165deg,
    var(--yj-side-rail-surface-1) 0%,
    var(--yj-side-rail-surface-2) 100%
  );
  box-shadow: inset 0 1px 0 color-mix(in srgb, #fff 6%, transparent);
  font-size: 0.68em;
  line-height: 1.32;
  overflow-x: hidden;
  overflow-y: auto;
}

.intraday-auction-rail__head {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.intraday-auction-rail__title {
  font-weight: 600;
  font-size: 0.65em;
  color: var(--yj-text-muted);
  letter-spacing: 0.12em;
  text-transform: uppercase;
  opacity: 0.92;
}

.intraday-auction-rail__pill {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 6px 7px;
  border-radius: 8px;
  border: 1px solid color-mix(in srgb, currentColor 22%, transparent);
  background: linear-gradient(
    180deg,
    color-mix(in srgb, currentColor 12%, transparent),
    color-mix(in srgb, currentColor 7%, transparent)
  );
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}

.intraday-auction-rail__pill-row {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px;
  min-width: 0;
}

.intraday-auction-rail__pill-row--pct {
  opacity: 0.92;
}

.intraday-auction-rail__pill-k {
  flex-shrink: 0;
  color: var(--yj-text-muted);
  font-weight: 500;
  font-size: 0.9em;
  white-space: nowrap;
}

.intraday-auction-rail__price {
  font-size: 1.12em;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  text-align: right;
}

.intraday-auction-rail__pct {
  font-size: 0.98em;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  text-align: right;
}

.intraday-auction-rail__stats {
  margin: 0;
  padding: 5px 0 0;
  border-top: 1px solid color-mix(in srgb, var(--yj-row-border) 65%, transparent);
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.intraday-auction-rail__stat {
  margin: 0;
  display: grid;
  grid-template-columns: max-content minmax(0, 1fr);
  align-items: baseline;
  column-gap: 8px;
  font-variant-numeric: tabular-nums;
}

.intraday-auction-rail__stat .intraday-auction-rail__k {
  margin: 0;
  white-space: nowrap;
}

.intraday-auction-rail__stat .intraday-auction-rail__num {
  margin: 0;
  text-align: right;
  min-width: 0;
}

.intraday-auction-rail__stat--ref .intraday-auction-rail__num {
  color: var(--yj-text);
}

.intraday-auction-rail__k {
  color: var(--yj-text-muted);
  font-weight: 500;
  font-size: 0.9em;
  white-space: nowrap;
}

.intraday-auction-rail__num {
  font-weight: 600;
  font-size: 1.02em;
  font-variant-numeric: tabular-nums;
}

.stock-detail-body--embedded .intraday-auction-rail {
  flex-shrink: 0;
  min-height: 0;
}

.market-split--intraday-auction {
  align-items: stretch;
}

.chart-col {
  flex: 1;
  min-width: 0;
  min-height: 112px;
}

.book-col {
  box-sizing: border-box;
  flex: 0 0 var(--yj-order-book-w);
  width: var(--yj-order-book-w);
  max-width: 100%;
  min-width: 0;
  align-self: flex-start;
}

.book-col--baseline {
  align-self: stretch;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.book-panel {
  box-sizing: border-box;
  border-radius: 10px;
  border: 1px solid var(--yj-side-rail-border);
  background: linear-gradient(
    165deg,
    var(--yj-side-rail-surface-1) 0%,
    var(--yj-side-rail-surface-2) 100%
  );
  box-shadow: inset 0 1px 0 color-mix(in srgb, #fff 6%, transparent);
  padding: 4px 5px 5px;
}

.book-col--baseline .book-panel {
  flex: 1 1 auto;
  min-height: 0;
  display: flex;
  flex-direction: column;
  /* 顶 spacer 与零轴对齐时偶发超出高度，hidden 会裁掉买一档；允许纵向滚动兜底 */
  overflow-x: hidden;
  overflow-y: auto;
}

/* 窄抽屉：盘口叠在图下 */
@container stockdetail (max-width: 439px) {
  .stock-detail-body--embedded .market-split {
    flex-direction: column;
    flex: 1;
    min-height: 0;
  }

  .stock-detail-body--embedded .book-col {
    flex: none;
    width: 100%;
    max-width: none;
    max-height: min(36vh, 200px);
    overflow-y: auto;
  }

  .stock-detail-body--embedded .intraday-auction-rail {
    flex: none;
    width: 100%;
    max-width: none;
    flex-direction: column;
    align-items: stretch;
    max-height: none;
  }

  .stock-detail-body--embedded .intraday-auction-rail__head {
    flex-direction: row;
    flex-wrap: wrap;
    align-items: flex-start;
    justify-content: space-between;
    gap: 8px;
  }

  .stock-detail-body--embedded .intraday-auction-rail__stats {
    flex-flow: row wrap;
    border-top: none;
    padding-top: 0;
    gap: 10px 16px;
  }

  .stock-detail-body--embedded .intraday-auction-rail__stat {
    grid-template-columns: auto auto;
    column-gap: 6px;
  }

  .stock-detail-body--embedded .intraday-auction-rail__stat .intraday-auction-rail__num {
    text-align: left;
  }
}

/* 宽侧板（默认）：图 + 五档横排，图区尽量高 */
@container stockdetail (min-width: 440px) {
  .stock-detail-body--embedded .market-split {
    flex-direction: row;
    align-items: stretch;
  }

  .stock-detail-body--embedded .chart-col {
    flex: 1;
    min-width: 0;
  }

  .stock-detail-body--embedded .book-col {
    flex: 0 0 var(--yj-order-book-w);
    width: var(--yj-order-book-w);
    max-width: 100%;
    max-height: none;
    overflow-y: visible;
  }
}

/* 小窗：压缩主信息区与指标，代码与涨跌挤到同一视觉块 */
@container stockdetail (max-width: 420px) {
  .stock-detail-body__main {
    padding: 8px 10px 10px;
  }

  .hero {
    margin-bottom: 8px;
  }

  .price-line__cluster {
    flex-flow: row wrap;
    align-items: baseline;
    gap: 6px 10px;
  }

  .code {
    font-size: 0.72em;
  }

  .price {
    font-size: 1.12em;
  }

  .pct,
  .amt {
    font-size: 0.82em;
  }

  .btn.hero-watch {
    padding: 2px 6px;
    font-size: 0.68em;
  }

  .grid {
    gap: 5px 6px;
    margin-bottom: 8px;
  }

  .cell {
    padding: 5px 7px;
    font-size: 0.74em;
    border-radius: 6px;
  }

  .chart-tabs-row {
    gap: 6px;
    margin-bottom: 8px;
  }

  .chart-tabs {
    gap: 4px;
  }

  .chart-indicator-select {
    width: min(100%, 7.25rem);
    min-width: 4.5rem;
  }

  .chart-tabs-row .chart-indicator-select :deep(button.yj-field-control.yj-select-trigger) {
    padding: 2px 7px;
    font-size: 0.68em;
  }

  .chart-tab {
    padding: 3px 8px;
    font-size: 0.66em;
  }

  .chart-col {
    min-height: 96px;
  }

  .market-split {
    gap: 6px;
  }
}

/* 极窄：全页单列指标 */
@container stockdetail (max-width: 300px) {
  .grid {
    grid-template-columns: 1fr;
  }

  .price-line__values {
    gap: 6px 8px;
  }

  .stock-detail-body__head-row {
    gap: 4px 6px;
  }

  .stock-detail-body__head-price {
    font-size: 0.92em;
  }

  .stock-detail-body__head-chip {
    font-size: 0.66em;
    padding: 1px 6px;
  }
}

/* 矮窗口：减少头部占位，让图表区多占纵向空间 */
@container stockdetail (max-height: 520px) {
  .stock-detail-body--embedded .stock-detail-body__head {
    padding: 6px 8px;
  }

  .stock-detail-body--embedded .stock-detail-body__title {
    font-size: 0.82em;
  }

  .stock-detail-body--embedded .stock-detail-body__head-price {
    font-size: 1em;
  }

  .stock-detail-body--embedded .stock-detail-body__head-chip {
    font-size: 0.72em;
  }

  .hero {
    margin-bottom: 6px;
  }

  .grid {
    margin-bottom: 6px;
  }

  .chart-col {
    min-height: 80px;
  }
}

@media (max-width: 520px) {
  .market-split {
    flex-direction: column;
  }

  .book-col {
    flex: none;
    width: 100%;
  }
}

/* 涨跌色（与看盘列表一致；本组件 scoped，需自带给详情/顶栏） */
.up-red {
  color: #fb7185;
}
.down-green {
  color: #4ade80;
}
.up-green {
  color: #4ade80;
}
.down-red {
  color: #fb7185;
}
.flat {
  color: var(--yj-flat);
}
</style>