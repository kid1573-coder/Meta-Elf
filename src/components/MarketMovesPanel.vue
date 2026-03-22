<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import type { MarketMoveItem } from "../types/marketMoves";
import { useSettings } from "../composables/useSettings";
import { changeClass, fmtFixed } from "../utils/format";

const { settings } = useSettings();
const movesTheme = computed(() =>
  settings.value?.theme === "light" ? "market-moves--light" : "market-moves--dark",
);

const props = withDefaults(
  defineProps<{
    quoteSource: string;
    colorScheme?: "redUp" | "greenUp";
    /** drawer：侧栏内嵌（顶栏含关闭）；page：整页由外层标题栏负责返回/刷新 */
    layout?: "drawer" | "page";
  }>(),
  { colorScheme: "redUp", layout: "drawer" },
);

const emit = defineEmits<{
  close: [];
  "select-stock": [code: string];
}>();

defineExpose({
  reload: () => void load(),
});

const items = ref<MarketMoveItem[]>([]);
const loading = ref(false);
const err = ref<string | null>(null);
let pollTimer: ReturnType<typeof setInterval> | null = null;

async function load() {
  loading.value = true;
  err.value = null;
  try {
    const list = await invoke<MarketMoveItem[]>("get_market_moves", {
      quoteSource: props.quoteSource || "eastmoney",
    });
    items.value = list ?? [];
  } catch (e) {
    err.value = String(e);
    items.value = [];
  } finally {
    loading.value = false;
  }
}

function startPoll() {
  stopPoll();
  pollTimer = setInterval(() => {
    void load();
  }, 50_000);
}

function stopPoll() {
  if (pollTimer != null) {
    clearInterval(pollTimer);
    pollTimer = null;
  }
}

onMounted(() => {
  void load();
  startPoll();
});

onUnmounted(() => {
  stopPoll();
});

watch(
  () => props.quoteSource,
  () => {
    void load();
  },
);

function onChipClick(code: string) {
  emit("select-stock", code.trim().toLowerCase());
}
</script>

<template>
  <div
    class="market-moves"
    :class="[layout === 'page' && 'market-moves--page', movesTheme]"
  >
    <header v-if="layout === 'drawer'" class="market-moves__head">
      <h2 class="market-moves__title">股市异动</h2>
      <div class="market-moves__actions">
        <button
          type="button"
          class="market-moves__icon-btn"
          title="刷新"
          aria-label="刷新异动"
          :disabled="loading"
          @click="load"
        >
          ↻
        </button>
        <button
          type="button"
          class="market-moves__icon-btn"
          aria-label="关闭"
          title="关闭"
          @click="emit('close')"
        >
          ×
        </button>
      </div>
    </header>

    <div class="market-moves__body">
      <p v-if="loading && items.length === 0" class="market-moves__hint">加载中…</p>
      <p v-else-if="err" class="market-moves__err">{{ err }}</p>
      <p v-else-if="items.length === 0" class="market-moves__hint">暂无异动数据</p>
      <ul v-else class="market-moves__list">
        <li
          v-for="(it, idx) in items"
          :key="idx"
          class="market-moves__item"
          :class="{ 'market-moves__item--latest': idx === 0 }"
        >
          <div class="market-moves__block">
            <span class="market-moves__time">{{ it.time }}</span>
            <p class="market-moves__text">{{ it.text }}</p>
            <div v-if="it.stocks?.length" class="market-moves__chips">
              <button
                v-for="(s, j) in it.stocks"
                :key="j"
                type="button"
                class="market-moves__chip"
                @click="onChipClick(s.code)"
              >
                <span class="market-moves__chip-name">{{ s.name }}</span>
                <span
                  class="market-moves__chip-pct"
                  :class="changeClass(s.changePct, colorScheme)"
                >
                  {{ s.changePct >= 0 ? "+" : "" }}{{ fmtFixed(s.changePct, 2) }}%
                </span>
              </button>
            </div>
          </div>
        </li>
      </ul>
    </div>
  </div>
</template>

<style scoped>
.market-moves {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  color: var(--yj-text);
  background: linear-gradient(
    180deg,
    var(--yj-settings-bg-1),
    var(--yj-settings-bg-2)
  );
}

.market-moves--page {
  background: transparent;
}

/* 与主界面行情表 .grid 同级：0.88em 正文 */
.market-moves--page .market-moves__body {
  padding: 6px 10px 10px;
}

.market-moves--page .market-moves__hint,
.market-moves--page .market-moves__err {
  font-size: 1em;
  padding: 8px 4px;
}

.market-moves__head {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 8px 10px;
  border-bottom: 1px solid var(--yj-bar-border);
  -webkit-app-region: no-drag;
  app-region: no-drag;
}

.market-moves__title {
  margin: 0;
  font-size: 0.78em;
  font-weight: 600;
  letter-spacing: 0.02em;
}

.market-moves__actions {
  display: flex;
  align-items: center;
  gap: 4px;
}

.market-moves__icon-btn {
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
  font-size: 0.85rem;
  line-height: 1;
  cursor: pointer;
}

.market-moves__icon-btn:hover:not(:disabled) {
  background: var(--yj-icon-hover-bg);
}

.market-moves__icon-btn:disabled {
  opacity: 0.45;
  cursor: default;
}

.market-moves__body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 8px 10px 10px;
  font-size: 0.88em;
  -webkit-app-region: no-drag;
  app-region: no-drag;
}

.market-moves__hint {
  margin: 0;
  font-size: 1em;
  color: var(--yj-text-muted);
}

.market-moves__err {
  margin: 0;
  font-size: 1em;
  color: var(--yj-err);
}

.market-moves__list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 0;
}

.market-moves__item {
  border-bottom: 1px solid var(--yj-row-border);
}

.market-moves__item:last-child {
  border-bottom: none;
}

/* 参考看盘软件：首条略强调 */
.market-moves__item--latest .market-moves__block {
  margin: 0 -6px;
  padding: 6px 6px 8px;
  border-radius: 8px;
  background: rgba(37, 99, 235, 0.14);
}

.market-moves--light .market-moves__item--latest .market-moves__block {
  background: rgba(59, 130, 246, 0.12);
}

/* 时间 | 标题 在第一行；标签独占第二行且与标题左缘对齐（炒股软件常见排版） */
.market-moves__block {
  display: grid;
  grid-template-columns: 3.1em 1fr;
  column-gap: 8px;
  row-gap: 6px;
  align-items: start;
  padding: 6px 0 8px;
}

.market-moves__time {
  grid-column: 1;
  grid-row: 1;
  font-size: 0.91em;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  color: var(--yj-th-color);
  line-height: 1.4;
  padding-top: 0.06em;
}

.market-moves__text {
  grid-column: 2;
  grid-row: 1;
  margin: 0;
  font-size: 1em;
  font-weight: 500;
  line-height: 1.45;
  color: var(--yj-text);
  word-break: break-word;
}

.market-moves__chips {
  grid-column: 2;
  grid-row: 2;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px 8px;
  min-width: 0;
}

.market-moves__chip {
  display: inline-flex;
  align-items: baseline;
  gap: 5px;
  max-width: 100%;
  padding: 3px 8px;
  border-radius: 6px;
  border: 1px solid var(--yj-row-border);
  background: var(--yj-table-wrap-bg);
  cursor: pointer;
  font-family: inherit;
  font-size: 0.91em;
  line-height: 1.35;
}

.market-moves__chip:hover {
  background: var(--yj-row-hover-bg);
}

/* 暗色：金色股名；亮色：深琥珀，保证对比度 */
.market-moves--dark .market-moves__chip-name {
  color: #e8c547;
}

.market-moves--light .market-moves__chip-name {
  color: #a16207;
}

.market-moves__chip-name {
  font-weight: 600;
  max-width: 7.5em;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.market-moves__chip-pct {
  font-family: "DM Sans", "Noto Sans SC", sans-serif;
  font-variant-numeric: tabular-nums;
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
