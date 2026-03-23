<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import type { AppSettings } from "../types/app";
import type { OrderBook } from "../types/marketDetail";
import { changeClass, fmtFixed, fmtVolume } from "../utils/format";

const props = withDefaults(
  defineProps<{
    book: OrderBook | null;
    loading: boolean;
    colorScheme: AppSettings["colorScheme"];
    /** 与现价比较用：用于涨跌色 */
    lastPrice: number;
    /** 分时昨收/零轴在视口中的 Y（与 getBoundingClientRect 一致） */
    chartBaselineY?: number | null;
    /** 一档时：买卖分界线与零轴对齐 */
    alignBidAskMidToBaseline?: boolean;
  }>(),
  {
    colorScheme: "redUp",
    lastPrice: 0,
    chartBaselineY: null,
    alignBidAskMidToBaseline: false,
  },
);

const rootRef = ref<HTMLElement | null>(null);
const tableRef = ref<HTMLElement | null>(null);
const dividerRef = ref<HTMLElement | null>(null);
const topSpacerPx = ref(0);
let ro: ResizeObserver | null = null;

const maxVol = computed(() => {
  if (!props.book) return 1;
  let m = 1;
  for (const x of props.book.asks) m = Math.max(m, x.volume);
  for (const x of props.book.bids) m = Math.max(m, x.volume);
  return m || 1;
});

/** 现价贴近买一、卖档无量：多为涨停封单，买一行标注「封」 */
const showSealOnBid1 = computed(() => {
  const b = props.book;
  if (!b || props.loading || !b.bids.length) return false;
  const top = b.bids[0]!;
  if (top.volume <= 0 || !Number.isFinite(top.price) || top.price <= 0) return false;
  const ref = props.lastPrice;
  if (!Number.isFinite(ref) || ref <= 0) return false;
  const tol = Math.max(ref * 1.2e-4, 0.011);
  if (Math.abs(top.price - ref) > tol) return false;
  const asksDead =
    b.asks.length === 0 ||
    b.asks.every((a) => !Number.isFinite(a.volume) || a.volume === 0);
  return asksDead;
});

const useBaselineLayout = computed(
  () =>
    props.alignBidAskMidToBaseline &&
    !props.loading &&
    props.book != null &&
    props.chartBaselineY != null &&
    props.book.asks.length === 1 &&
    props.book.bids.length === 1,
);

function measureTopSpacer() {
  if (!useBaselineLayout.value) {
    topSpacerPx.value = 0;
    return;
  }
  const baselineScreen = props.chartBaselineY;
  if (baselineScreen == null || !Number.isFinite(baselineScreen)) {
    topSpacerPx.value = 0;
    return;
  }
  const div = dividerRef.value;
  const root = rootRef.value;
  if (!div || !root) return;
  const d = div.getBoundingClientRect();
  const midScreen = d.top + d.height / 2;
  const rootH = root.getBoundingClientRect().height;
  const tableH = tableRef.value?.getBoundingClientRect().height ?? 56;
  /* 顶 spacer + 表格不能超过盘口高度，否则 flex-shrink:0 会把买一档裁在面板外 */
  const maxSpacer = Math.max(0, Math.floor(rootH - tableH - 4));
  const next = Math.round(topSpacerPx.value + baselineScreen - midScreen);
  topSpacerPx.value = Math.max(0, Math.min(next, maxSpacer));
}

/** 不在测量前把 spacer 清零，避免每次零轴更新时盘口先塌一帧再跳 */
function scheduleMeasure() {
  void nextTick(() => {
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        measureTopSpacer();
      });
    });
  });
}

watch(
  () => [
    props.chartBaselineY,
    props.alignBidAskMidToBaseline,
    props.loading,
    props.book?.asks.length,
    props.book?.bids.length,
  ],
  () => scheduleMeasure(),
);

watch(useBaselineLayout, (on) => {
  if (on) scheduleMeasure();
  else topSpacerPx.value = 0;
});

onMounted(() => {
  void nextTick(() => {
    if (typeof ResizeObserver !== "undefined") {
      ro = new ResizeObserver(() => {
        requestAnimationFrame(() => {
          requestAnimationFrame(measureTopSpacer);
        });
      });
      if (rootRef.value) ro.observe(rootRef.value);
    }
    scheduleMeasure();
  });
});

onUnmounted(() => {
  ro?.disconnect();
  ro = null;
});

function barPct(vol: number) {
  return `${Math.min(100, (vol / maxVol.value) * 100)}%`;
}

function volLabel(v: number) {
  if (v <= 0) return "—";
  return fmtVolume(v);
}

function pxCls(price: number) {
  const ref = props.lastPrice;
  if (!Number.isFinite(ref) || ref <= 0) return "flat";
  const d = price - ref;
  if (Math.abs(d) < 1e-9) return "flat";
  return changeClass(d > 0 ? 1 : -1, props.colorScheme);
}
</script>

<template>
  <div
    ref="rootRef"
    class="order-book"
    :class="{ 'order-book--baseline': useBaselineLayout }"
  >
    <div v-if="loading" class="order-book__hint">盘口加载…</div>
    <template v-else-if="book && (book.asks.length || book.bids.length)">
      <div
        v-if="useBaselineLayout"
        class="order-book__spacer-top"
        :style="{ height: `${topSpacerPx}px` }"
      />
      <div ref="tableRef" class="order-book__table">
        <div
          v-for="(lv, i) in book.asks"
          :key="'a' + i"
          class="ob-row ob-row--ask"
        >
          <span class="ob-lab">卖{{ book.asks.length - i }}</span>
          <span class="ob-price" :class="pxCls(lv.price)">{{ fmtFixed(lv.price, 2) }}</span>
          <span class="ob-vol-wrap">
            <span class="ob-vol-bar ob-vol-bar--ask" :style="{ width: barPct(lv.volume) }" />
            <span class="ob-vol">{{ volLabel(lv.volume) }}</span>
          </span>
        </div>
        <div ref="dividerRef" class="ob-divider" role="presentation" />
        <div
          v-for="(lv, i) in book.bids"
          :key="'b' + i"
          class="ob-row ob-row--bid"
        >
          <span class="ob-lab"
            >买{{ i + 1
            }}<span
              v-if="i === 0 && showSealOnBid1"
              class="ob-seal"
              title="涨停价一档封单量"
              >封</span
            ></span
          >
          <span class="ob-price" :class="pxCls(lv.price)">{{ fmtFixed(lv.price, 2) }}</span>
          <span class="ob-vol-wrap">
            <span class="ob-vol-bar ob-vol-bar--bid" :style="{ width: barPct(lv.volume) }" />
            <span class="ob-vol">{{ volLabel(lv.volume) }}</span>
          </span>
        </div>
      </div>
      <div v-if="useBaselineLayout" class="order-book__spacer-bottom" />
    </template>
  </div>
</template>

<style scoped>
.order-book {
  font-size: 0.68em;
  line-height: 1.14;
  min-width: 0;
  width: 100%;
  max-width: 100%;
  box-sizing: border-box;
}

.order-book--baseline {
  height: 100%;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.order-book__hint {
  color: var(--yj-text-muted);
  padding: 4px 2px;
  text-align: center;
}

.order-book__spacer-top {
  flex-shrink: 0;
  width: 100%;
  min-height: 0;
}

.order-book__spacer-bottom {
  flex: 1 1 0;
  min-height: 0;
  width: 100%;
}

.order-book__table {
  display: flex;
  flex-direction: column;
  gap: 0;
  flex: 0 0 auto;
}

.order-book--baseline .order-book__table {
  flex-shrink: 0;
}

/* 左：卖1/买1 + 价；右：手数（集合竞价式两端对齐） */
.ob-row {
  display: grid;
  grid-template-columns: max-content max-content 1fr minmax(2.55em, max-content);
  align-items: center;
  column-gap: 4px;
  min-height: 0;
  padding: 2px 0;
}

.ob-lab {
  color: var(--yj-text-muted);
  font-variant-numeric: tabular-nums;
  display: inline-flex;
  align-items: center;
  gap: 2px;
  justify-self: start;
}

.ob-seal {
  display: inline-block;
  padding: 0 2px;
  border-radius: 2px;
  font-size: 0.68em;
  font-weight: 700;
  line-height: 1.2;
  color: color-mix(in srgb, var(--yj-text-muted) 95%, transparent);
  border: 1px solid color-mix(in srgb, var(--yj-row-border) 90%, transparent);
  background: color-mix(in srgb, var(--yj-table-wrap-bg) 65%, transparent);
}

.ob-price {
  font-family: "DM Sans", "Noto Sans SC", sans-serif;
  font-variant-numeric: tabular-nums;
  font-weight: 600;
  text-align: left;
  justify-self: start;
  min-width: 0;
}

.ob-price.up-red {
  color: #fb7185;
}
.ob-price.down-green {
  color: #4ade80;
}
.ob-price.up-green {
  color: #4ade80;
}
.ob-price.down-red {
  color: #fb7185;
}
.ob-price.flat {
  color: var(--yj-flat);
}

.ob-vol-wrap {
  position: relative;
  grid-column: 4;
  justify-self: stretch;
  width: 100%;
  text-align: right;
  min-width: 0;
  min-height: 1em;
}

.ob-vol-bar {
  position: absolute;
  right: 0;
  top: 0;
  bottom: 0;
  max-width: 100%;
  border-radius: 2px;
  opacity: 0.2;
  pointer-events: none;
}

.ob-vol-bar--ask {
  background: #22c55e;
}

.ob-vol-bar--bid {
  background: #ef4444;
}

.ob-vol {
  position: relative;
  z-index: 1;
  font-variant-numeric: tabular-nums;
  color: var(--yj-text-muted);
}

.ob-divider {
  height: 2px;
  margin: 3px 0 2px;
  border: none;
  border-radius: 1px;
  background: linear-gradient(
    90deg,
    transparent 0%,
    color-mix(in srgb, var(--yj-text-muted) 42%, transparent) 12%,
    color-mix(in srgb, var(--yj-text-muted) 42%, transparent) 88%,
    transparent 100%
  );
  box-shadow: 0 0 0 1px color-mix(in srgb, var(--yj-row-border) 50%, transparent);
  opacity: 0.9;
}
</style>
