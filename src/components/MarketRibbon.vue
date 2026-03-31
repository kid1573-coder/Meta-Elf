<script setup lang="ts">
/**
 * 底部「市场信息带」：左侧按波动/涨跌突出指数（多项时轮播；沉寂时轮播简报；
 * 相邻快照涨跌幅跳变时有方向性闪烁，与轮播区分）+ 右侧涨跌家数/成交额。
 * 产品侧可称「市场快览」。
 */
import { computed, nextTick, onMounted, ref, watch } from "vue";
import type { AppSettings } from "../types/app";
import type { MarketRibbonSnapshot, RibbonIndex } from "../types/marketRibbon";
import { changeClass, fmtDeltaTurnoverCn, fmtFixed, fmtTurnoverCn } from "../utils/format";

/** 底栏此区域窄于该值时，涨跌家数/成交额等收入浮层，把横向空间留给大盘指数 */
const INLINE_STATS_MIN_PX = 580;

const props = defineProps<{
  snapshot: MarketRibbonSnapshot | null;
  loading: boolean;
  error: string | null;
  colorScheme: AppSettings["colorScheme"];
  theme?: "dark" | "light";
  /** 为 true 时不展示涨跌家数、成交额、较昨日及展开按钮（如侧栏查看个股时） */
  hideBreadthStats?: boolean;
}>();

const emit = defineEmits<{
  retry: [];
}>();

const rootEl = ref<HTMLElement | null>(null);
const flyoutRootRef = ref<HTMLElement | null>(null);
const toggleEl = ref<HTMLButtonElement | null>(null);
const ribbonWidth = ref(9999);
const statsOpen = ref(false);
const flyoutStyle = ref<Record<string, string>>({});

const indices = computed(() => props.snapshot?.indices ?? []);
const sectorGainers = computed(() => props.snapshot?.sectorGainers ?? []);
const sectorLosers = computed(() => props.snapshot?.sectorLosers ?? []);

/** Δ（这轮-上轮涨跌幅）≥此值视为「明显异动」，播方向动画 */
const RIBBON_DELTA_THRESH_MID = 0.2;

type RibbonIndexState = {
  lastPct: number;
  delta: number;
};

const ribbonStates = ref<Map<string, RibbonIndexState>>(new Map());

/** 记录哪些 index 当前正在播放方向动画（key=index key） */
const ribbonAnimatingUp = ref<Set<string>>(new Set());
const ribbonAnimatingDn = ref<Set<string>>(new Set());

function ribbonIndexKey(it: RibbonIndex): string {
  const k = (it.id || it.name || "").trim();
  return k || "_";
}

/**
 * 每次快照更新时计算 Δ = changePct_本轮 - changePct_上轮。
 * Δ > 0 → 短线向上（回升）；Δ < 0 → 短线向下（回落）。
 * Δ 的绝对值越大 → 短线异动越强 → 越应该排在前面。
 */
watch(
  () => props.snapshot?.indices,
  (list) => {
    if (!list?.length) {
      ribbonStates.value = new Map();
      ribbonAnimatingUp.value = new Set();
      ribbonAnimatingDn.value = new Set();
      return;
    }
    const next = new Map(ribbonStates.value);

    for (const idx of list) {
      const key = ribbonIndexKey(idx);
      const existing = next.get(key);
      if (!existing) {
        next.set(key, { lastPct: idx.changePct, delta: 0 });
        continue;
      }
      const delta = idx.changePct - existing.lastPct;
      next.set(key, { lastPct: idx.changePct, delta });

      /* 触发方向动画：Δ 明显为正 → 向上跳；Δ 明显为负 → 向下跳 */
      if (Math.abs(delta) >= RIBBON_DELTA_THRESH_MID) {
        if (delta > 0) {
          ribbonAnimatingDn.value.delete(key);
          ribbonAnimatingUp.value.add(key);
          setTimeout(() => ribbonAnimatingUp.value.delete(key), 750);
        } else {
          ribbonAnimatingUp.value.delete(key);
          ribbonAnimatingDn.value.add(key);
          setTimeout(() => ribbonAnimatingDn.value.delete(key), 750);
        }
      } else {
        ribbonAnimatingUp.value.delete(key);
        ribbonAnimatingDn.value.delete(key);
      }
    }
    ribbonStates.value = next;
  },
  { deep: true },
);

/** 短线异动最强的指数排在最前：主排序 = |Δ|，次排序 = |changePct| */
const ribbonSpotlightIndices = computed((): RibbonIndex[] => {
  const list = props.snapshot?.indices ?? [];
  if (!list.length) return [];
  const states = ribbonStates.value;
  return [...list].sort((a, b) => {
    const sa = states.get(ribbonIndexKey(a));
    const sb = states.get(ribbonIndexKey(b));
    const da = Math.abs(sa?.delta ?? 0);
    const db = Math.abs(sb?.delta ?? 0);
    if (db !== da) return db - da;
    const pa = Math.abs(a.changePct);
    const pb = Math.abs(b.changePct);
    if (pb !== pa) return pb - pa;
    return ribbonIndexKey(a).localeCompare(ribbonIndexKey(b));
  });
});

const deltaYuan = computed(() => {
  const s = props.snapshot;
  if (!s) return 0;
  return s.turnoverToday - s.turnoverYesterday;
});

const deltaClass = computed(() => changeClass(deltaYuan.value, props.colorScheme));

const useInlineStats = computed(() => ribbonWidth.value >= INLINE_STATS_MIN_PX);

const showStatsFlyout = computed(
  () =>
    !!props.snapshot &&
    !props.hideBreadthStats &&
    !useInlineStats.value &&
    statsOpen.value,
);

function updateFlyoutPosition() {
  const btn = toggleEl.value;
  if (!btn) return;
  const r = btn.getBoundingClientRect();
  const pad = 8;
  const avail = window.innerWidth - pad * 2;
  const maxW = Math.min(720, Math.max(440, Math.floor(window.innerWidth * 0.78)), avail);
  flyoutStyle.value = {
    position: "fixed",
    left: `${Math.min(Math.max(pad, r.right - maxW), window.innerWidth - pad - maxW)}px`,
    top: `${r.top}px`,
    transform: "translateY(calc(-100% - 6px))",
    width: `${maxW}px`,
    maxWidth: `${maxW}px`,
    zIndex: "2147482000",
  };
}

function closeStatsFlyout() {
  statsOpen.value = false;
}

let docDownCleanup: (() => void) | null = null;

watch(showStatsFlyout, async (open) => {
  docDownCleanup?.();
  docDownCleanup = null;
  if (!open) return;
  await nextTick();
  requestAnimationFrame(() => updateFlyoutPosition());
  const onDocDown = (e: MouseEvent) => {
    const t = e.target as Node;
    if (rootEl.value?.contains(t)) return;
    if (flyoutRootRef.value?.contains(t)) return;
    closeStatsFlyout();
  };
  document.addEventListener("mousedown", onDocDown, true);
  docDownCleanup = () => document.removeEventListener("mousedown", onDocDown, true);
});

watch(useInlineStats, (inline) => {
  if (inline) {
    statsOpen.value = false;
    docDownCleanup?.();
    docDownCleanup = null;
  }
});

watch(
  () => props.hideBreadthStats,
  (hide) => {
    if (hide) {
      statsOpen.value = false;
      docDownCleanup?.();
      docDownCleanup = null;
    }
  },
);

function onToggleStats() {
  if (useInlineStats.value) return;
  if (!statsOpen.value) {
    updateFlyoutPosition();
  }
  statsOpen.value = !statsOpen.value;
}

function onWinResizeOrScroll() {
  if (showStatsFlyout.value) updateFlyoutPosition();
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === "Escape" && statsOpen.value) {
    e.preventDefault();
    closeStatsFlyout();
  }
}

let ro: ResizeObserver | null = null;

onMounted(() => {
  window.addEventListener("resize", onWinResizeOrScroll);
  window.addEventListener("scroll", onWinResizeOrScroll, true);
  window.addEventListener("keydown", onKeydown);
  const el = rootEl.value;
  if (el && typeof ResizeObserver !== "undefined") {
    ro = new ResizeObserver((entries) => {
      const w = entries[0]?.contentRect.width ?? 0;
      ribbonWidth.value = w;
    });
    ro.observe(el);
    ribbonWidth.value = el.getBoundingClientRect().width;
  }
});
</script>

<template>
  <div
    ref="rootEl"
    class="market-ribbon"
    :class="{
      'market-ribbon--light': theme === 'light',
      'market-ribbon--green-up': colorScheme === 'greenUp',
    }"
    aria-label="市场概览"
  >
    <div class="market-ribbon__marquee" role="presentation">
      <div v-if="loading && !snapshot" class="market-ribbon__hint">市场数据加载中…</div>
      <div v-else-if="error && !snapshot" class="market-ribbon__hint market-ribbon__hint--err">
        {{ error }}
        <button type="button" class="market-ribbon__retry" @click="emit('retry')">重试</button>
      </div>
      <div v-else-if="!indices.length" class="market-ribbon__hint">暂无指数数据</div>
      <div v-else class="market-ribbon__chip-scroll" role="list">
        <div
          v-for="s in ribbonSpotlightIndices"
          :key="ribbonIndexKey(s)"
          class="market-ribbon__chip"
          :class="[
            /* 背景色：当前涨跌（涨=红/绿，跌=绿/红），始终反映实际所处状态 */
            s.changePct >= 0 ? 'market-ribbon__chip--up' : 'market-ribbon__chip--dn',
            /* Δ > 0 且明显 → 向上跳动画 */
            ribbonAnimatingUp.has(ribbonIndexKey(s)) ? 'market-ribbon__chip--anim-up' : '',
            /* Δ < 0 且明显 → 向下跳动画 */
            ribbonAnimatingDn.has(ribbonIndexKey(s)) ? 'market-ribbon__chip--anim-dn' : '',
          ]"
          role="listitem"
        >
          <span class="market-ribbon__index-name">{{ s.name }}</span>
          <span
            class="market-ribbon__index-pct"
            :class="changeClass(s.changePct, colorScheme)"
          >
            {{ s.changePct >= 0 ? "+" : "" }}{{ fmtFixed(s.changePct, 2) }}%
          </span>
        </div>
      </div>
    </div>

    <div
      v-if="snapshot && useInlineStats && !hideBreadthStats"
      class="market-ribbon__stats market-ribbon__stats--inline"
    >
      <button
        type="button"
        class="market-ribbon__stats-toggle market-ribbon__stats-toggle--inline"
        :class="{ 'market-ribbon__stats-toggle--open': statsOpen }"
        title="展开/收起市场统计"
        aria-label="展开/收起市场统计"
        @click="statsOpen = !statsOpen"
      >
        <span class="market-ribbon__stats-toggle-icon">▼</span>
      </button>
      <template v-if="statsOpen">
        <span class="market-ribbon__breadth" :class="changeClass(1, colorScheme)">
          涨{{ snapshot.upCount }}家
        </span>
        <span class="market-ribbon__breadth" :class="changeClass(-1, colorScheme)">
          跌{{ snapshot.downCount }}家
        </span>
        <span class="market-ribbon__v" aria-hidden="true">|</span>
        <span class="market-ribbon__muted">成交额</span>
        <span class="market-ribbon__val">{{ fmtTurnoverCn(snapshot.turnoverToday) }}</span>
        <span class="market-ribbon__v" aria-hidden="true">|</span>
        <span class="market-ribbon__cmp-yday">
          <span class="market-ribbon__muted">较昨日</span>
          <span class="market-ribbon__delta" :class="deltaClass">
            {{ fmtDeltaTurnoverCn(deltaYuan) }}
          </span>
        </span>
      </template>
    </div>

    <button
      v-else-if="snapshot && !hideBreadthStats"
      ref="toggleEl"
      type="button"
      class="market-ribbon__stats-toggle"
      :class="{ 'market-ribbon__stats-toggle--open': statsOpen }"
      title="市场快览：涨跌家数、成交额、行业板块"
      aria-label="展开市场快览"
      :aria-expanded="statsOpen"
      @click="onToggleStats"
    >
      <span class="market-ribbon__stats-toggle-icon">▼</span>
    </button>

    <Teleport to="#yj-root">
      <div
        v-if="showStatsFlyout"
        ref="flyoutRootRef"
        class="market-ribbon__stats-flyout"
        :class="{ 'market-ribbon__stats-flyout--light': theme === 'light' }"
        :style="flyoutStyle"
        role="dialog"
        aria-label="市场快览"
        @mousedown.stop
      >
        <div class="market-ribbon__flyout-head">
          <span class="market-ribbon__flyout-title">市场快览</span>
          <div class="market-ribbon__flyout-kpis">
            <span class="market-ribbon__breadth" :class="changeClass(1, colorScheme)">
              涨{{ snapshot!.upCount }}家
            </span>
            <span class="market-ribbon__flyout-dot" aria-hidden="true">·</span>
            <span class="market-ribbon__breadth" :class="changeClass(-1, colorScheme)">
              跌{{ snapshot!.downCount }}家
            </span>
            <span class="market-ribbon__flyout-sep" aria-hidden="true">|</span>
            <span class="market-ribbon__muted">成交额</span>
            <span class="market-ribbon__val">{{ fmtTurnoverCn(snapshot!.turnoverToday) }}</span>
            <span class="market-ribbon__flyout-sep" aria-hidden="true">|</span>
            <span class="market-ribbon__muted">较昨日</span>
            <span class="market-ribbon__delta" :class="deltaClass">
              {{ fmtDeltaTurnoverCn(deltaYuan) }}
            </span>
          </div>
        </div>

        <p class="market-ribbon__sector-note">
          沪深行业板块（东财，涨幅/跌幅各前 6）
        </p>

        <p
          v-if="sectorGainers.length === 0 && sectorLosers.length === 0"
          class="market-ribbon__flyout-hint"
        >
          板块数据暂不可用
        </p>
        <div v-else class="market-ribbon__sector-grid">
          <div class="market-ribbon__sector-col">
            <div class="market-ribbon__sector-cap">领涨行业</div>
            <ul class="market-ribbon__sector-list market-ribbon__sector-list--twocol">
              <li
                v-for="(it, i) in sectorGainers"
                :key="'g' + i"
                class="market-ribbon__sector-row"
              >
                <span class="market-ribbon__sector-name" :title="it.name">{{ it.name }}</span>
                <span
                  class="market-ribbon__sector-pct"
                  :class="changeClass(it.changePct, colorScheme)"
                >
                  {{ it.changePct >= 0 ? "+" : "" }}{{ fmtFixed(it.changePct, 2) }}%
                </span>
              </li>
            </ul>
          </div>
          <div class="market-ribbon__sector-col">
            <div class="market-ribbon__sector-cap">领跌行业</div>
            <ul class="market-ribbon__sector-list market-ribbon__sector-list--twocol">
              <li
                v-for="(it, i) in sectorLosers"
                :key="'l' + i"
                class="market-ribbon__sector-row"
              >
                <span class="market-ribbon__sector-name" :title="it.name">{{ it.name }}</span>
                <span
                  class="market-ribbon__sector-pct"
                  :class="changeClass(it.changePct, colorScheme)"
                >
                  {{ it.changePct >= 0 ? "+" : "" }}{{ fmtFixed(it.changePct, 2) }}%
                </span>
              </li>
            </ul>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.market-ribbon {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 8px;
  margin-left: 4px;
  padding: 0 2px 0 6px;
  border-left: 1px solid var(--yj-toolbar-border);
  font-size: 0.88em;
  line-height: 1.35;
  -webkit-app-region: no-drag;
  app-region: no-drag;
}

.market-ribbon__marquee {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  mask-image: linear-gradient(90deg, transparent, #000 6px, #000 calc(100% - 6px), transparent);
}

.market-ribbon__chip-scroll {
  display: flex;
  flex-wrap: nowrap;
  align-items: center;
  gap: 4px;
  overflow-x: auto;
  overflow-y: hidden;
  padding: 1px 0;
  scrollbar-width: none;
  -webkit-overflow-scrolling: touch;
}

.market-ribbon__chip-scroll::-webkit-scrollbar {
  display: none;
}

.market-ribbon__chip {
  display: inline-flex;
  align-items: baseline;
  flex-shrink: 0;
  gap: 3px;
  padding: 1px 5px 2px;
  border-radius: 5px;
  border: 1px solid color-mix(in srgb, var(--yj-row-border) 65%, transparent);
  font-size: 0.78em;
  line-height: 1.25;
  /* 数据刷新时平滑过渡背景色 */
  transition: background-color 0.35s ease, border-color 0.35s ease, box-shadow 0.35s ease;
}

/* 背景色：反映当前涨跌状态（涨=红/绿，跌=绿/红） */
.market-ribbon__chip--up {
  background: color-mix(in srgb, rgb(251, 113, 133) 12%, var(--yj-table-wrap-bg) 55%);
  border-color: color-mix(in srgb, rgb(251, 113, 133) 45%, var(--yj-row-border));
}
.market-ribbon--green-up .market-ribbon__chip--up {
  background: color-mix(in srgb, rgb(74, 222, 128) 12%, var(--yj-table-wrap-bg) 55%);
  border-color: color-mix(in srgb, rgb(74, 222, 128) 45%, var(--yj-row-border));
}

.market-ribbon__chip--dn {
  background: color-mix(in srgb, rgb(74, 222, 128) 12%, var(--yj-table-wrap-bg) 55%);
  border-color: color-mix(in srgb, rgb(74, 222, 128) 45%, var(--yj-row-border));
}
.market-ribbon--green-up .market-ribbon__chip--dn {
  background: color-mix(in srgb, rgb(251, 113, 133) 12%, var(--yj-table-wrap-bg) 55%);
  border-color: color-mix(in srgb, rgb(251, 113, 133) 45%, var(--yj-row-border));
}

/* 短线异动方向动画：Δ > 0 → 向上跳，Δ < 0 → 向下跳，|Δ| 越大跳幅越大 */
@keyframes ribbon-chip-up {
  0% { transform: translateY(0); }
  20% { transform: translateY(-3px); }
  50% { transform: translateY(-1px); }
  100% { transform: translateY(0); }
}

@keyframes ribbon-chip-dn {
  0% { transform: translateY(0); }
  20% { transform: translateY(3px); }
  50% { transform: translateY(1px); }
  100% { transform: translateY(0); }
}

.market-ribbon__chip--anim-up {
  animation: ribbon-chip-up 0.7s cubic-bezier(0.32, 0.72, 0.24, 1) both;
}

.market-ribbon__chip--anim-dn {
  animation: ribbon-chip-dn 0.7s cubic-bezier(0.32, 0.72, 0.24, 1) both;
}

.market-ribbon__index-name {
  color: #e8c547;
  font-weight: 500;
  white-space: nowrap;
  font-size: 0.96em;
}

.market-ribbon__index-pct {
  font-family: "DM Sans", "Noto Sans SC", sans-serif;
  font-variant-numeric: tabular-nums;
  font-weight: 600;
  white-space: nowrap;
  font-size: 0.98em;
}

.market-ribbon__hint {
  font-size: 0.92em;
  color: var(--yj-text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.market-ribbon__hint--err {
  color: var(--yj-err);
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.market-ribbon__retry {
  border: 1px solid var(--yj-btn-border);
  background: var(--yj-btn-bg);
  color: var(--yj-btn-color);
  border-radius: 4px;
  padding: 1px 6px;
  font-size: 0.85em;
  cursor: pointer;
}

.market-ribbon__stats {
  display: inline-flex;
  align-items: baseline;
  flex-wrap: nowrap;
  gap: 0 6px;
  justify-content: flex-end;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
  min-width: 0;
}

.market-ribbon__stats--inline {
  flex-shrink: 0;
  padding: 4px 9px;
  border-radius: 7px;
  border: 1px solid var(--yj-modal-panel-border);
  background: var(--yj-modal-panel-bg);
  box-shadow: 0 2px 14px rgba(0, 0, 0, 0.28);
}

.market-ribbon--light .market-ribbon__stats--inline {
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.07);
}

.market-ribbon__stats-toggle {
  flex-shrink: 0;
  box-sizing: border-box;
  min-width: 22px;
  width: 22px;
  height: 18px;
  padding: 0;
  border-radius: 5px;
  border: 1px solid var(--yj-tool-border);
  background: var(--yj-tool-bg);
  color: var(--yj-tool-color);
  cursor: pointer;
  font-size: 0.55em;
  line-height: 1;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  opacity: 0.88;
  transition: opacity 0.15s ease;
}

.market-ribbon__stats-toggle:hover {
  opacity: 1;
  filter: brightness(1.08);
}

.market-ribbon__stats-toggle-icon {
  display: inline-block;
  transition: transform 0.2s ease;
}

.market-ribbon__stats-toggle--open .market-ribbon__stats-toggle-icon {
  transform: rotate(180deg);
}

.market-ribbon__stats-flyout {
  padding: 10px 14px 12px;
  border-radius: 10px;
  border: 1px solid var(--yj-modal-panel-border);
  background: var(--yj-modal-panel-bg);
  box-shadow: 0 10px 32px rgba(0, 0, 0, 0.45);
  font-size: 0.86em;
  line-height: 1.32;
  box-sizing: border-box;
  max-height: min(70vh, 400px);
  overflow-x: hidden;
  overflow-y: auto;
  -webkit-app-region: no-drag;
  app-region: no-drag;
}

.market-ribbon__flyout-head {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px 20px;
  margin-bottom: 8px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--yj-row-border);
}

.market-ribbon__flyout-title {
  font-size: 0.8em;
  font-weight: 600;
  letter-spacing: 0.04em;
  color: var(--yj-text-muted);
  flex-shrink: 0;
}

.market-ribbon__flyout-kpis {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 4px 8px;
  font-variant-numeric: tabular-nums;
  min-width: 0;
  flex: 1 1 auto;
  justify-content: flex-end;
}

.market-ribbon__flyout-sep {
  color: var(--yj-row-border);
  font-weight: 300;
  margin: 0 2px;
  user-select: none;
}

.market-ribbon__flyout-dot {
  color: var(--yj-text-muted);
  opacity: 0.5;
  user-select: none;
}

.market-ribbon__sector-note {
  margin: 0 0 8px;
  font-size: 0.72em;
  color: var(--yj-text-muted);
  line-height: 1.35;
}

.market-ribbon__flyout-hint {
  margin: 0;
  font-size: 0.86em;
  color: var(--yj-text-muted);
}

.market-ribbon__sector-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  gap: 8px 20px;
  align-items: start;
}

.market-ribbon__sector-col {
  min-width: 0;
}

.market-ribbon__sector-cap {
  font-size: 0.72em;
  font-weight: 700;
  letter-spacing: 0.06em;
  margin-bottom: 4px;
  padding-bottom: 3px;
  border-bottom: 1px solid var(--yj-row-border);
  color: var(--yj-text-muted);
}

.market-ribbon__sector-list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.market-ribbon__sector-list--twocol {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  column-gap: 12px;
  row-gap: 1px;
}

.market-ribbon__sector-row {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 4px;
  padding: 2px 0;
  font-size: 0.74em;
  font-variant-numeric: tabular-nums;
  min-width: 0;
  border-bottom: none;
}

.market-ribbon__sector-name {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--yj-text);
  font-weight: 500;
}

.market-ribbon__sector-pct {
  flex-shrink: 0;
  font-family: "DM Sans", "Noto Sans SC", sans-serif;
  font-weight: 600;
}

.market-ribbon__stats-flyout--light {
  box-shadow: 0 10px 28px rgba(0, 0, 0, 0.12);
}

.market-ribbon__cmp-yday {
  display: inline-flex;
  align-items: baseline;
  gap: 4px;
  white-space: nowrap;
  flex-shrink: 0;
}

.market-ribbon__breadth {
  font-weight: 600;
  white-space: nowrap;
}

.market-ribbon--light .market-ribbon__index-name {
  color: #b45309;
}

.market-ribbon__muted {
  color: var(--yj-text-muted);
  font-size: 0.94em;
  font-weight: 400;
}

.market-ribbon__val {
  color: var(--yj-text);
  font-weight: 600;
}

.market-ribbon__v {
  color: var(--yj-row-border);
  font-weight: 300;
  margin: 0 1px;
}

.market-ribbon__delta {
  font-weight: 600;
}

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
