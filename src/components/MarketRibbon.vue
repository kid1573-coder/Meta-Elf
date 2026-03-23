<script setup lang="ts">
/**
 * 底部「市场信息带」：左侧按波动/涨跌突出指数（多项时轮播；沉寂时轮播简报；
 * 相邻快照涨跌幅跳变时有方向性闪烁，与轮播区分）+ 右侧涨跌家数/成交额。
 * 产品侧可称「市场快览」。
 */
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
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

/** 两次刷新间涨跌幅变化小于该值（百分点）视为「无动静」 */
const RIBBON_QUIET_EPS = 0.025;
/** 连续若干次无动静后不再占底栏（15s×80≈20min，与 MainPanel ribbon 轮询一致） */
const RIBBON_STALE_AFTER_POLLS = 80;
/**
 * |涨跌幅|≥此值不因「长期无波动」移出 spotlight（隔夜外盘、持续偏强/偏弱）
 * 高亮样式另见 RIBBON_HOT_ABS_PCT
 */
const RIBBON_STICKY_ABS_PCT = 0.35;
/** |涨跌幅|≥此值 chip 使用醒目样式 */
const RIBBON_HOT_ABS_PCT = 0.5;
/**
 * 相邻两次快照间 |Δ涨跌幅|≥此值（百分点）视为「快速变动」，播放方向性闪烁，与定时轮播区分
 *（应明显大于 RIBBON_QUIET_EPS）
 */
const RIBBON_FAST_MOVE_DELTA = 0.06;

/** 活跃指数超过该数量时轮播展示（减少动效偏好下改为横向滚动全部） */
function maxActiveChipsForWidth(w: number): number {
  if (w < 420) return 2;
  if (w < 640) return 3;
  return 4;
}

type RibbonQuietState = { lastPct: number; quietStreak: number };
const ribbonQuietById = ref<Map<string, RibbonQuietState>>(new Map());

/** 快速变动：按涨跌方向闪烁（与轮播换条无关） */
type RibbonFlashKind = "pos" | "neg";
const ribbonFlashById = ref<Map<string, RibbonFlashKind>>(new Map());

/** 需在 indices 的 watch 之前声明，避免回调里读到 TDZ */
const reduceMotion = ref(false);

function ribbonIndexKey(it: RibbonIndex): string {
  const k = (it.id || it.name || "").trim();
  return k || "_";
}

watch(
  () => props.snapshot?.indices,
  (list) => {
    if (!list?.length) {
      ribbonQuietById.value = new Map();
      ribbonFlashById.value = new Map();
      return;
    }
    const next = new Map(ribbonQuietById.value);
    const noAnim = reduceMotion.value;
    const fastMoveByKey = new Map<string, RibbonFlashKind>();

    for (const it of list) {
      const key = ribbonIndexKey(it);
      const existing = next.get(key);
      if (!existing) {
        next.set(key, { lastPct: it.changePct, quietStreak: 0 });
        continue;
      }
      const delta = it.changePct - existing.lastPct;
      let { lastPct, quietStreak } = existing;
      if (Math.abs(it.changePct - lastPct) < RIBBON_QUIET_EPS) {
        quietStreak += 1;
      } else {
        quietStreak = 0;
        lastPct = it.changePct;
      }
      next.set(key, { lastPct, quietStreak });

      if (!noAnim && Math.abs(delta) >= RIBBON_FAST_MOVE_DELTA) {
        fastMoveByKey.set(key, delta >= 0 ? "pos" : "neg");
      }
    }
    ribbonQuietById.value = next;

    const fastMoves = [...fastMoveByKey.entries()].map(([key, kind]) => ({ key, kind }));
    if (!fastMoves.length) return;
    const cleared = new Map(ribbonFlashById.value);
    for (const f of fastMoves) cleared.delete(f.key);
    ribbonFlashById.value = cleared;
    nextTick(() => {
      const m = new Map(ribbonFlashById.value);
      for (const f of fastMoves) m.set(f.key, f.kind);
      ribbonFlashById.value = m;
    });
  },
  { deep: true },
);

function ribbonChipFlashClass(key: string): Record<string, boolean> {
  const k = ribbonFlashById.value.get(key);
  return {
    "market-ribbon__chip--flash-pos": k === "pos",
    "market-ribbon__chip--flash-neg": k === "neg",
  };
}

function onRibbonChipFlashEnd(e: AnimationEvent, key: string) {
  const n = e.animationName;
  if (n !== "ribbon-flash-bump-pos" && n !== "ribbon-flash-bump-neg") return;
  if (!ribbonFlashById.value.has(key)) return;
  const m = new Map(ribbonFlashById.value);
  m.delete(key);
  ribbonFlashById.value = m;
}

/** 盯盘：有波动或涨跌幅够大才占栏位；按 |涨跌| 排序，大变动更显眼 */
const ribbonSpotlightIndices = computed(() => {
  const list = props.snapshot?.indices ?? [];
  if (!list.length) return [];
  const m = ribbonQuietById.value;
  const active = list.filter((it) => {
    const st = m.get(ribbonIndexKey(it));
    const streak = st?.quietStreak ?? 0;
    return streak < RIBBON_STALE_AFTER_POLLS || Math.abs(it.changePct) >= RIBBON_STICKY_ABS_PCT;
  });
  if (!active.length) return [];
  return [...active].sort((a, b) => Math.abs(b.changePct) - Math.abs(a.changePct));
});

const ribbonCalmHint = computed(() => {
  const n = props.snapshot?.indices?.length ?? 0;
  return n > 0 && ribbonSpotlightIndices.value.length === 0;
});

const maxActiveChips = computed(() => maxActiveChipsForWidth(ribbonWidth.value));

const ribbonMarqueePaused = ref(false);
const rotateOffsetActive = ref(0);
const calmSlotIndex = ref(0);

type CalmSlot = { kind: "hint" } | { kind: "idx"; it: RibbonIndex };

/** 沉寂模式：逐个指数简报，每满 3 条插入一次说明文案 */
const calmSlots = computed((): CalmSlot[] => {
  const list = indices.value;
  const slots: CalmSlot[] = [];
  list.forEach((it, i) => {
    slots.push({ kind: "idx", it });
    if ((i + 1) % 3 === 0) slots.push({ kind: "hint" });
  });
  return slots;
});

const activeNeedsRotation = computed(() => {
  const list = ribbonSpotlightIndices.value;
  return list.length > maxActiveChips.value && !reduceMotion.value;
});

const ribbonVisibleSpotlight = computed((): RibbonIndex[] => {
  const list = ribbonSpotlightIndices.value;
  const cap = maxActiveChips.value;
  if (!list.length) return [];
  if (!activeNeedsRotation.value) return list;
  const n = list.length;
  const off = ((rotateOffsetActive.value % n) + n) % n;
  const out: RibbonIndex[] = [];
  for (let i = 0; i < Math.min(cap, n); i++) {
    out.push(list[(off + i) % n]!);
  }
  return out;
});

const currentCalmSlot = computed((): CalmSlot | null => {
  const slots = calmSlots.value;
  if (!slots.length) return null;
  const i = ((calmSlotIndex.value % slots.length) + slots.length) % slots.length;
  return slots[i] ?? null;
});

const CALM_HINT_TEXT =
  "主要指数窄幅波动，底栏仅在有动静时突出展示（市场快览里仍有涨跌家数与板块）";

const marqueeTickIntervalMs = computed(() => {
  if (!indices.value.length || (props.loading && !props.snapshot)) return 0;
  if (ribbonCalmHint.value) {
    return reduceMotion.value ? 18_000 : 5000;
  }
  if (activeNeedsRotation.value) return reduceMotion.value ? 0 : 5000;
  return 0;
});

let marqueeRotateTimer: ReturnType<typeof setInterval> | null = null;

function clearMarqueeRotateTimer() {
  if (marqueeRotateTimer != null) {
    clearInterval(marqueeRotateTimer);
    marqueeRotateTimer = null;
  }
}

function tickMarqueeRotate() {
  if (ribbonMarqueePaused.value) return;
  if (ribbonCalmHint.value) {
    const slots = calmSlots.value;
    if (slots.length) calmSlotIndex.value = (calmSlotIndex.value + 1) % slots.length;
    return;
  }
  const list = ribbonSpotlightIndices.value;
  const cap = maxActiveChips.value;
  if (list.length > cap && !reduceMotion.value) {
    rotateOffsetActive.value = (rotateOffsetActive.value + 1) % list.length;
  }
}

function restartMarqueeRotateTimer() {
  clearMarqueeRotateTimer();
  const ms = marqueeTickIntervalMs.value;
  if (ms <= 0) return;
  marqueeRotateTimer = setInterval(tickMarqueeRotate, ms);
}

let reduceMotionMql: MediaQueryList | null = null;
let reduceMotionListener: (() => void) | null = null;

watch(marqueeTickIntervalMs, () => {
  restartMarqueeRotateTimer();
});

watch(ribbonCalmHint, (calm) => {
  rotateOffsetActive.value = 0;
  if (calm) calmSlotIndex.value = 0;
});

watch(
  () => calmSlots.value.length,
  () => {
    if (ribbonCalmHint.value) calmSlotIndex.value = 0;
  },
);

watch(
  () => ribbonSpotlightIndices.value.map((x) => ribbonIndexKey(x)).join("\0"),
  () => {
    rotateOffsetActive.value = 0;
  },
);

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
  /* 横向卡片：偏宽、偏低，减少「窄高条」违和感 */
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
  reduceMotionMql = window.matchMedia("(prefers-reduced-motion: reduce)");
  reduceMotion.value = reduceMotionMql.matches;
  reduceMotionListener = () => {
    reduceMotion.value = reduceMotionMql?.matches ?? false;
  };
  reduceMotionMql.addEventListener("change", reduceMotionListener);
  restartMarqueeRotateTimer();
});

onUnmounted(() => {
  window.removeEventListener("resize", onWinResizeOrScroll);
  window.removeEventListener("scroll", onWinResizeOrScroll, true);
  window.removeEventListener("keydown", onKeydown);
  ro?.disconnect();
  docDownCleanup?.();
  clearMarqueeRotateTimer();
  if (reduceMotionMql && reduceMotionListener) {
    reduceMotionMql.removeEventListener("change", reduceMotionListener);
  }
  reduceMotionMql = null;
  reduceMotionListener = null;
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
      <div
        v-else
        class="market-ribbon__marquee-rotate-wrap"
        @mouseenter="ribbonMarqueePaused = true"
        @mouseleave="ribbonMarqueePaused = false"
      >
        <div v-if="ribbonCalmHint" class="market-ribbon__calm-rotate">
          <p
            v-if="currentCalmSlot?.kind === 'hint'"
            key="calm-hint"
            class="market-ribbon__hint market-ribbon__hint--calm"
          >
            {{ CALM_HINT_TEXT }}
          </p>
          <div
            v-else-if="currentCalmSlot?.kind === 'idx'"
            :key="'calm-' + ribbonIndexKey(currentCalmSlot.it)"
            class="market-ribbon__chip market-ribbon__chip--calm"
            :class="ribbonChipFlashClass(ribbonIndexKey(currentCalmSlot.it))"
            role="listitem"
            @animationend="onRibbonChipFlashEnd($event, ribbonIndexKey(currentCalmSlot.it))"
          >
            <span class="market-ribbon__index-name market-ribbon__index-name--calm">{{
              currentCalmSlot.it.name
            }}</span>
            <span
              class="market-ribbon__index-pct"
              :class="changeClass(currentCalmSlot.it.changePct, colorScheme)"
            >
              {{ currentCalmSlot.it.changePct >= 0 ? "+" : "" }}{{ fmtFixed(currentCalmSlot.it.changePct, 2) }}%
            </span>
          </div>
        </div>
        <div v-else class="market-ribbon__chip-scroll" role="list">
          <div
            v-for="s in ribbonVisibleSpotlight"
            :key="ribbonIndexKey(s)"
            class="market-ribbon__chip"
            :class="[
              { 'market-ribbon__chip--hot': Math.abs(s.changePct) >= RIBBON_HOT_ABS_PCT },
              ribbonChipFlashClass(ribbonIndexKey(s)),
            ]"
            role="listitem"
            @animationend="onRibbonChipFlashEnd($event, ribbonIndexKey(s))"
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
      title="市场快览：涨跌家数、成交额、行业板块"
      aria-label="展开市场快览"
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

.market-ribbon__marquee-rotate-wrap {
  min-width: 0;
  flex: 1;
  display: flex;
  align-items: center;
}

.market-ribbon__calm-rotate {
  min-width: 0;
  flex: 1;
  display: flex;
  align-items: center;
  min-height: 1.45em;
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
  background: color-mix(in srgb, var(--yj-table-wrap-bg) 50%, transparent);
  font-size: 0.78em;
  line-height: 1.25;
  animation: ribbon-chip-settle 0.18s ease-out both;
}

.market-ribbon__chip--hot {
  border-color: color-mix(in srgb, var(--yj-text-muted) 30%, var(--yj-row-border));
  box-shadow: 0 0 0 1px color-mix(in srgb, #e8c547 16%, transparent);
}

/* 相邻快照涨跌幅跳变：方向性弹跳 + 光晕，与定时轮播的轻微 settle 区分 */
.market-ribbon__chip--flash-pos {
  --ribbon-flash-rgb: 251, 113, 133;
  animation: ribbon-flash-bump-pos 0.88s cubic-bezier(0.33, 1.06, 0.52, 1) 1;
  z-index: 1;
}

.market-ribbon__chip--flash-neg {
  --ribbon-flash-rgb: 74, 222, 128;
  animation: ribbon-flash-bump-neg 0.88s cubic-bezier(0.33, 1.06, 0.52, 1) 1;
  z-index: 1;
}

.market-ribbon--green-up .market-ribbon__chip--flash-pos {
  --ribbon-flash-rgb: 74, 222, 128;
}

.market-ribbon--green-up .market-ribbon__chip--flash-neg {
  --ribbon-flash-rgb: 251, 113, 133;
}

.market-ribbon__chip--calm {
  border-color: color-mix(in srgb, var(--yj-row-border) 55%, transparent);
  background: color-mix(in srgb, var(--yj-table-wrap-bg) 35%, transparent);
  animation: none;
  max-width: 100%;
}

.market-ribbon__chip--calm.market-ribbon__chip--flash-pos {
  animation-name: ribbon-flash-bump-pos;
  animation-duration: 0.88s;
  animation-timing-function: cubic-bezier(0.33, 1.06, 0.52, 1);
  animation-iteration-count: 1;
}

.market-ribbon__chip--calm.market-ribbon__chip--flash-neg {
  animation-name: ribbon-flash-bump-neg;
  animation-duration: 0.88s;
  animation-timing-function: cubic-bezier(0.33, 1.06, 0.52, 1);
  animation-iteration-count: 1;
}

.market-ribbon__index-name--calm {
  color: var(--yj-text-muted);
  font-weight: 500;
}

.market-ribbon--light .market-ribbon__chip--hot {
  box-shadow: 0 0 0 1px color-mix(in srgb, #b45309 14%, transparent);
}

@keyframes ribbon-flash-bump-pos {
  0%,
  100% {
    transform: translateY(0);
    box-shadow: none;
  }
  16% {
    transform: translateY(-3px);
    box-shadow:
      0 0 0 1px rgb(var(--ribbon-flash-rgb) / 0.45),
      0 0 14px 3px rgb(var(--ribbon-flash-rgb) / 0.32);
  }
  42% {
    transform: translateY(-1px);
    box-shadow: 0 0 0 1px rgb(var(--ribbon-flash-rgb) / 0.2);
  }
}

@keyframes ribbon-flash-bump-neg {
  0%,
  100% {
    transform: translateY(0);
    box-shadow: none;
  }
  16% {
    transform: translateY(3px);
    box-shadow:
      0 0 0 1px rgb(var(--ribbon-flash-rgb) / 0.45),
      0 0 14px 3px rgb(var(--ribbon-flash-rgb) / 0.32);
  }
  42% {
    transform: translateY(1px);
    box-shadow: 0 0 0 1px rgb(var(--ribbon-flash-rgb) / 0.2);
  }
}

.market-ribbon--light .market-ribbon__index-name--calm {
  color: var(--yj-text-muted);
}

@keyframes ribbon-chip-settle {
  from {
    opacity: 0.82;
  }
  to {
    opacity: 1;
  }
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

.market-ribbon__hint--calm {
  font-size: 0.82em;
  line-height: 1.3;
  white-space: normal;
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
  /* 勿用 table-wrap 半透明底：透明窗口下与背后跑马灯叠字 */
  background: var(--yj-modal-panel-bg);
  box-shadow: 0 2px 14px rgba(0, 0, 0, 0.28);
}

.market-ribbon--light .market-ribbon__stats--inline {
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.07);
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

/* 每侧 6 条排成 2 列 × 3 行，降低总高度 */
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
