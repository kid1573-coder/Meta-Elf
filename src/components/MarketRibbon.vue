<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import type { AppSettings } from "../types/app";
import type { MarketRibbonSnapshot } from "../types/marketRibbon";
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
  const maxW = Math.min(320, window.innerWidth - pad * 2);
  flyoutStyle.value = {
    position: "fixed",
    left: `${Math.min(Math.max(pad, r.right - maxW), window.innerWidth - pad - maxW)}px`,
    top: `${r.top}px`,
    transform: "translateY(calc(-100% - 6px))",
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

onUnmounted(() => {
  window.removeEventListener("resize", onWinResizeOrScroll);
  window.removeEventListener("scroll", onWinResizeOrScroll, true);
  window.removeEventListener("keydown", onKeydown);
  ro?.disconnect();
  docDownCleanup?.();
});
</script>

<template>
  <div
    ref="rootEl"
    class="market-ribbon"
    :class="{ 'market-ribbon--light': theme === 'light' }"
    aria-label="市场概览"
  >
    <div class="market-ribbon__marquee" role="presentation">
      <div v-if="loading && !snapshot" class="market-ribbon__hint">市场数据加载中…</div>
      <div v-else-if="error && !snapshot" class="market-ribbon__hint market-ribbon__hint--err">
        {{ error }}
        <button type="button" class="market-ribbon__retry" @click="emit('retry')">重试</button>
      </div>
      <div v-else-if="!indices.length" class="market-ribbon__hint">暂无指数数据</div>
      <div v-else class="market-ribbon__marquee-clip">
        <div class="market-ribbon__track">
          <template v-for="dup in 2" :key="'d' + dup">
            <template v-for="(s, i) in indices" :key="dup + '-' + i">
              <span class="market-ribbon__index-name">{{ s.name }}</span>
              <span
                class="market-ribbon__index-pct"
                :class="changeClass(s.changePct, colorScheme)"
              >
                {{ s.changePct >= 0 ? "+" : "" }}{{ fmtFixed(s.changePct, 2) }}%
              </span>
              <span class="market-ribbon__dot" aria-hidden="true">·</span>
            </template>
          </template>
        </div>
      </div>
    </div>

    <div
      v-if="snapshot && useInlineStats && !hideBreadthStats"
      class="market-ribbon__stats market-ribbon__stats--inline"
    >
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
    </div>

    <button
      v-else-if="snapshot && !hideBreadthStats"
      ref="toggleEl"
      type="button"
      class="market-ribbon__stats-toggle"
      :class="{ 'market-ribbon__stats-toggle--open': statsOpen }"
      title="涨跌家数、成交额、较昨日"
      aria-label="展开市场统计：涨跌家数、成交额、较昨日"
      :aria-expanded="statsOpen"
      @click="onToggleStats"
    >
      ▼
    </button>

    <Teleport to="#yj-root">
      <div
        v-if="showStatsFlyout"
        ref="flyoutRootRef"
        class="market-ribbon__stats-flyout"
        :class="{ 'market-ribbon__stats-flyout--light': theme === 'light' }"
        :style="flyoutStyle"
        role="dialog"
        aria-label="市场统计"
        @mousedown.stop
      >
        <div class="market-ribbon__stats market-ribbon__stats--stacked">
          <div class="market-ribbon__stats-row">
            <span class="market-ribbon__breadth" :class="changeClass(1, colorScheme)">
              涨{{ snapshot!.upCount }}家
            </span>
            <span class="market-ribbon__breadth" :class="changeClass(-1, colorScheme)">
              跌{{ snapshot!.downCount }}家
            </span>
          </div>
          <div class="market-ribbon__stats-row">
            <span class="market-ribbon__muted">成交额</span>
            <span class="market-ribbon__val">{{ fmtTurnoverCn(snapshot!.turnoverToday) }}</span>
          </div>
          <div class="market-ribbon__stats-row market-ribbon__cmp-yday">
            <span class="market-ribbon__muted">较昨日</span>
            <span class="market-ribbon__delta" :class="deltaClass">
              {{ fmtDeltaTurnoverCn(deltaYuan) }}
            </span>
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

.market-ribbon__marquee-clip {
  overflow: hidden;
}

.market-ribbon__track {
  display: inline-flex;
  flex-wrap: nowrap;
  align-items: baseline;
  gap: 0 6px;
  width: max-content;
  animation: ribbon-marquee 52s linear infinite;
}

.market-ribbon__marquee:hover .market-ribbon__track {
  animation-play-state: paused;
}

@keyframes ribbon-marquee {
  from {
    transform: translateX(0);
  }
  to {
    transform: translateX(-50%);
  }
}

.market-ribbon__index-name {
  color: #e8c547;
  font-weight: 500;
  white-space: nowrap;
}

.market-ribbon__index-pct {
  font-family: "DM Sans", "Noto Sans SC", sans-serif;
  font-variant-numeric: tabular-nums;
  font-weight: 600;
  white-space: nowrap;
}

.market-ribbon__dot {
  color: var(--yj-text-muted);
  opacity: 0.45;
  margin-right: 2px;
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
}

.market-ribbon__stats--stacked {
  flex-direction: column;
  align-items: stretch;
  gap: 6px;
  white-space: normal;
}

.market-ribbon__stats-row {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 6px 10px;
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
  transition: opacity 0.15s ease, transform 0.15s ease;
}

.market-ribbon__stats-toggle:hover {
  opacity: 1;
  filter: brightness(1.08);
}

.market-ribbon__stats-toggle--open {
  transform: rotate(180deg);
}

.market-ribbon__stats-flyout {
  padding: 8px 10px;
  border-radius: 8px;
  border: 1px solid var(--yj-table-wrap-border, rgba(255, 255, 255, 0.14));
  background: var(--yj-table-wrap-bg, #2a2a2e);
  box-shadow: 0 10px 32px rgba(0, 0, 0, 0.45);
  font-size: 0.88em;
  line-height: 1.35;
  -webkit-app-region: no-drag;
  app-region: no-drag;
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
