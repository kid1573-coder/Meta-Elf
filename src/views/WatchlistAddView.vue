<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch, nextTick } from "vue";
import { useRoute, useRouter } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import YjSelect from "../components/YjSelect.vue";
import { useSettings } from "../composables/useSettings";
import { LAST_WATCH_GROUP_KEY, resolveTargetGroupId } from "../constants/watchGroup";
import type { SuggestItem } from "../types/app";
import { changeClass, fmtFixed } from "../utils/format";
import { pruneWatchlistIfOrphaned } from "../utils/watchGroups";

const SEARCH_HISTORY_KEY = "yj-add-search-history";
const SEARCH_HISTORY_MAX = 24;

const route = useRoute();
const router = useRouter();
const { settings, load, save } = useSettings();

const query = ref("");
const results = ref<SuggestItem[]>([]);
const searching = ref(false);
/** 搜索框为空时：东财股吧人气榜 */
const hotStocks = ref<SuggestItem[]>([]);
const hotLoading = ref(false);
const toast = ref("");
const searchHistory = ref<string[]>([]);
const historyOpen = ref(false);
const searchRowRef = ref<HTMLElement | null>(null);
const historyPanelRef = ref<HTMLElement | null>(null);
const historyPanelStyle = ref<Record<string, string>>({});
let debounceTimer: ReturnType<typeof setTimeout> | undefined;

function loadSearchHistory(): string[] {
  try {
    const raw = localStorage.getItem(SEARCH_HISTORY_KEY);
    if (!raw) return [];
    const arr = JSON.parse(raw) as unknown;
    if (!Array.isArray(arr)) return [];
    return arr
      .filter((x): x is string => typeof x === "string" && x.trim().length > 0)
      .map((x) => x.trim())
      .slice(0, SEARCH_HISTORY_MAX);
  } catch {
    return [];
  }
}

function persistSearchHistory(items: string[]) {
  try {
    localStorage.setItem(
      SEARCH_HISTORY_KEY,
      JSON.stringify(items.slice(0, SEARCH_HISTORY_MAX)),
    );
  } catch {
    /* ignore */
  }
}

function rememberSearchTerm(term: string) {
  const t = term.trim();
  if (t.length < 1) return;
  const next = [t, ...searchHistory.value.filter((x) => x !== t)];
  searchHistory.value = next.slice(0, SEARCH_HISTORY_MAX);
  persistSearchHistory(searchHistory.value);
}

function positionHistoryPanel() {
  const row = searchRowRef.value;
  if (!row) return;
  const r = row.getBoundingClientRect();
  const w = Math.min(Math.max(Math.round(r.width), 168), 360, window.innerWidth - 16);
  const left = Math.round(Math.min(r.left, window.innerWidth - w - 8));
  historyPanelStyle.value = {
    position: "fixed",
    zIndex: "2147483000",
    top: `${Math.round(r.bottom + 4)}px`,
    left: `${left}px`,
    width: `${w}px`,
  };
}

function toggleHistory() {
  if (!historyOpen.value) {
    positionHistoryPanel();
  }
  historyOpen.value = !historyOpen.value;
}

function applyHistory(term: string) {
  query.value = term;
  historyOpen.value = false;
}

function clearSearchHistory() {
  searchHistory.value = [];
  persistSearchHistory([]);
  historyOpen.value = false;
  showToast("已清空搜索历史", 1200);
}

function onHistoryPointerDownCapture(e: PointerEvent) {
  if (!historyOpen.value) return;
  const t = e.target as Node;
  if (searchRowRef.value?.contains(t) || historyPanelRef.value?.contains(t)) return;
  historyOpen.value = false;
}

function onHistoryKeyDown(e: KeyboardEvent) {
  if (e.key === "Escape") historyOpen.value = false;
}

function onHistoryScrollResize() {
  if (historyOpen.value) positionHistoryPanel();
}

watch(historyOpen, (v) => {
  if (v) {
    window.addEventListener("pointerdown", onHistoryPointerDownCapture, true);
    window.addEventListener("keydown", onHistoryKeyDown);
    window.addEventListener("scroll", onHistoryScrollResize, true);
    window.addEventListener("resize", onHistoryScrollResize);
    nextTick(() => positionHistoryPanel());
  } else {
    window.removeEventListener("pointerdown", onHistoryPointerDownCapture, true);
    window.removeEventListener("keydown", onHistoryKeyDown);
    window.removeEventListener("scroll", onHistoryScrollResize, true);
    window.removeEventListener("resize", onHistoryScrollResize);
  }
});

onUnmounted(() => {
  window.removeEventListener("pointerdown", onHistoryPointerDownCapture, true);
  window.removeEventListener("keydown", onHistoryKeyDown);
  window.removeEventListener("scroll", onHistoryScrollResize, true);
  window.removeEventListener("resize", onHistoryScrollResize);
});

function showToast(msg: string, ms = 1600) {
  toast.value = msg;
  setTimeout(() => {
    toast.value = "";
  }, ms);
}

async function loadHotStocks() {
  hotLoading.value = true;
  try {
    hotStocks.value = await invoke<SuggestItem[]>("fetch_hot_stocks");
  } catch {
    hotStocks.value = [];
    showToast("人气榜加载失败，请稍后重试", 2000);
  } finally {
    hotLoading.value = false;
  }
}

onMounted(async () => {
  searchHistory.value = loadSearchHistory();
  await load();
  void loadHotStocks();
});

async function runSearch(q: string) {
  const t = q.trim();
  if (t.length < 1) {
    results.value = [];
    searching.value = false;
    return;
  }
  searching.value = true;
  try {
    results.value = await invoke<SuggestItem[]>("search_securities", { query: t });
    rememberSearchTerm(t);
  } catch {
    results.value = [];
    showToast("搜索失败，请检查网络", 2000);
  } finally {
    searching.value = false;
  }
}

watch(query, (q) => {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => void runSearch(q), 320);
});

const targetGroupId = computed(() =>
  resolveTargetGroupId(settings.value, route.query.group as string | undefined),
);

const targetGroup = computed(() => {
  const s = settings.value;
  const id = targetGroupId.value;
  if (!s || !id) return null;
  return s.watchGroups.find((g) => g.id === id) ?? null;
});

const groupSelectOptions = computed(() =>
  settings.value?.watchGroups.map((g) => ({ value: g.id, label: g.name })) ?? [],
);

const colorScheme = computed(() => settings.value?.colorScheme ?? "redUp");

function isInTargetGroup(code: string): boolean {
  const g = targetGroup.value;
  if (!g) return false;
  const c = code.toLowerCase();
  return g.codes.some((x) => x.toLowerCase() === c);
}

function openStockDetail(item: SuggestItem) {
  router.push({ name: "stock", params: { code: item.code } });
}

function onSelectTargetGroup(id: string) {
  router.replace({
    path: route.path,
    query: { ...route.query, group: id },
  });
  try {
    sessionStorage.setItem(LAST_WATCH_GROUP_KEY, id);
  } catch {
    /* ignore */
  }
}

/** 同一板块名始终得到相同色相，用于列表里区分行业 */
function sectorHue(sector: string): number {
  let h = 2166136261;
  for (let i = 0; i < sector.length; i++) {
    h ^= sector.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return (h >>> 0) % 360;
}

function sectorChipStyle(sector: string) {
  const hue = sectorHue(sector);
  return {
    background: `hsla(${hue}, 52%, 50%, 0.22)`,
    color: `hsl(${hue}, 42%, 92%)`,
    borderColor: `hsla(${hue}, 55%, 58%, 0.45)`,
  };
}

function sectorDotStyle(sector: string) {
  const hue = sectorHue(sector);
  return {
    background: `hsl(${hue}, 72%, 58%)`,
    boxShadow: `0 0 0 1px hsla(${hue}, 65%, 42%, 0.45)`,
  };
}

/** 微信热股式：前三名单独配色 */
function rankBadgeClass(rank: number) {
  if (rank === 1) return "rk--1";
  if (rank === 2) return "rk--2";
  if (rank === 3) return "rk--3";
  return "";
}

function marketAbbr(code: string) {
  const c = code.toLowerCase();
  if (c.startsWith("sh")) return "沪";
  if (c.startsWith("sz")) return "深";
  if (c.startsWith("bj")) return "北";
  if (c.startsWith("hk")) return "港";
  return "";
}

function mktTagClass(code: string) {
  const c = code.toLowerCase();
  if (c.startsWith("sh")) return "mkt-tag--sh";
  if (c.startsWith("sz")) return "mkt-tag--sz";
  if (c.startsWith("bj")) return "mkt-tag--bj";
  if (c.startsWith("hk")) return "mkt-tag--hk";
  return "mkt-tag--na";
}

async function toggleGroupMembership(item: SuggestItem) {
  if (!settings.value || !targetGroup.value) return;
  const c = item.code.toLowerCase();
  const g = targetGroup.value;
  const idx = g.codes.findIndex((x) => x.toLowerCase() === c);
  if (idx >= 0) {
    g.codes.splice(idx, 1);
    pruneWatchlistIfOrphaned(settings.value);
    await save();
    showToast(`已从「${g.name}」移除`, 1200);
    return;
  }
  if (!settings.value.watchlist.some((w) => w.code.toLowerCase() === c)) {
    settings.value.watchlist.push({ code: c, name: item.name });
  }
  g.codes.push(c);
  await save();
  showToast(`已加入「${g.name}」`, 1200);
}
</script>

<template>
  <div class="add-page add-page--dense">
    <header class="bar bar--dense" data-tauri-drag-region>
      <button
        type="button"
        class="back"
        title="返回看盘"
        aria-label="返回看盘"
        @click="router.push('/')"
      >
        ‹
      </button>
      <h1 data-tauri-drag-region>添加自选</h1>
      <span class="bar__spacer" aria-hidden="true" />
    </header>

    <div v-if="settings" class="main">
      <div class="control-strip">
        <div v-if="settings.watchGroups.length" class="group-row">
          <span class="group-row__label">分组</span>
          <YjSelect
            class="group-select"
            aria-label="加入目标分组"
            :model-value="targetGroupId ?? ''"
            :options="groupSelectOptions"
            @update:model-value="onSelectTargetGroup"
          />
        </div>
        <div ref="searchRowRef" class="search-box">
          <input
            v-model="query"
            type="search"
            class="yj-field-control search-field"
            placeholder="代码 / 名称 / 拼音（可搜：日经225、原油、焦煤等）"
            autocomplete="off"
            enterkeyhint="search"
            autofocus
          />
          <span v-if="searching" class="hint-inline">…</span>
          <button
            type="button"
            class="history-btn"
            title="搜索历史"
            aria-label="搜索历史"
            :aria-expanded="historyOpen"
            aria-haspopup="listbox"
            @click="toggleHistory"
          >
            <svg
              class="history-btn__svg"
              viewBox="0 0 24 24"
              width="16"
              height="16"
              aria-hidden="true"
            >
              <circle
                cx="12"
                cy="12"
                r="8.25"
                fill="none"
                stroke="currentColor"
                stroke-width="1.5"
                opacity="0.35"
              />
              <path
                fill="none"
                stroke="currentColor"
                stroke-width="1.75"
                stroke-linecap="round"
                d="M12 7.25V12l3.25 2"
              />
            </svg>
          </button>
        </div>
      </div>

      <Teleport to="#yj-root">
        <div
          v-show="historyOpen"
          ref="historyPanelRef"
          class="yj-select-panel history-panel"
          role="listbox"
          :style="historyPanelStyle"
        >
          <p v-if="!searchHistory.length" class="history-panel__empty">暂无搜索记录</p>
          <template v-else>
            <button
              v-for="h in searchHistory"
              :key="h"
              type="button"
              class="yj-select-option history-panel__item"
              role="option"
              @click="applyHistory(h)"
            >
              {{ h }}
            </button>
            <button
              type="button"
              class="yj-select-option history-panel__clear"
              @click.stop="clearSearchHistory"
            >
              清空记录
            </button>
          </template>
        </div>
      </Teleport>

      <section class="card results">
        <div class="results-head">
          <h2>{{ query.trim() ? "搜索" : "人气" }}</h2>
          <span v-if="!query.trim()" class="results-head__meta">东财股吧 · 参考</span>
          <span v-else class="results-head__meta">东财联想</span>
        </div>

        <ul v-if="query.trim() && results.length" class="list">
          <li
            v-for="r in results"
            :key="r.code + r.quoteId"
            :class="{ 'in-group': isInTargetGroup(r.code) }"
          >
            <button type="button" class="row-btn" @click="openStockDetail(r)">
              <span class="row-lead">
                <span
                  class="sector-dot"
                  :class="{ 'sector-dot--plain': !r.sector }"
                  :style="r.sector ? sectorDotStyle(r.sector) : undefined"
                  aria-hidden="true"
                />
                <span
                  v-if="r.rank != null"
                  class="rk"
                  :class="rankBadgeClass(r.rank)"
                >{{ r.rank }}</span>
              </span>
              <span class="row-main">
                <span class="nm-title-line">
                  <span class="nm-title">{{ r.name }}</span>
                  <span
                    v-if="r.sector"
                    class="sector-chip"
                    :style="sectorChipStyle(r.sector)"
                  >{{ r.sector }}</span>
                </span>
                <span class="code-line">
                  <span class="mkt-tag" :class="mktTagClass(r.code)">{{ marketAbbr(r.code) }}</span>
                  <span class="cd cd--sub">{{ r.code }}</span>
                </span>
              </span>
              <span
                v-if="r.changePct != null || r.price != null"
                class="row-quote"
              >
                <span
                  v-if="r.changePct != null"
                  class="chg"
                  :class="changeClass(r.changePct, colorScheme)"
                >
                  {{ r.changePct > 0 ? "+" : "" }}{{ fmtFixed(r.changePct, 2) }}%
                </span>
                <span v-if="r.price != null" class="px">{{ fmtFixed(r.price, 2) }}</span>
              </span>
            </button>
            <button
              type="button"
              class="toggle-pin"
              :class="{ on: isInTargetGroup(r.code) }"
              :title="isInTargetGroup(r.code) ? '移出本分组' : '加入本分组'"
              :aria-pressed="isInTargetGroup(r.code)"
              :aria-label="isInTargetGroup(r.code) ? '移出本分组' : '加入本分组'"
              @click.stop="toggleGroupMembership(r)"
            >
              <span class="toggle-pin-glyph" aria-hidden="true">
                {{ isInTargetGroup(r.code) ? "✓" : "+" }}
              </span>
            </button>
          </li>
        </ul>

        <ul v-else-if="!query.trim() && hotStocks.length" class="list">
          <li
            v-for="r in hotStocks"
            :key="'hot-' + r.code + (r.rank ?? '')"
            :class="{ 'in-group': isInTargetGroup(r.code) }"
          >
            <button type="button" class="row-btn" @click="openStockDetail(r)">
              <span class="row-lead">
                <span
                  class="sector-dot"
                  :class="{ 'sector-dot--plain': !r.sector }"
                  :style="r.sector ? sectorDotStyle(r.sector) : undefined"
                  aria-hidden="true"
                />
                <span
                  v-if="r.rank != null"
                  class="rk"
                  :class="rankBadgeClass(r.rank)"
                >{{ r.rank }}</span>
              </span>
              <span class="row-main">
                <span class="nm-title-line">
                  <span class="nm-title">{{ r.name }}</span>
                  <span
                    v-if="r.sector"
                    class="sector-chip"
                    :style="sectorChipStyle(r.sector)"
                  >{{ r.sector }}</span>
                </span>
                <span class="code-line">
                  <span class="mkt-tag" :class="mktTagClass(r.code)">{{ marketAbbr(r.code) }}</span>
                  <span class="cd cd--sub">{{ r.code }}</span>
                </span>
              </span>
              <span
                v-if="r.changePct != null || r.price != null"
                class="row-quote"
              >
                <span
                  v-if="r.changePct != null"
                  class="chg"
                  :class="changeClass(r.changePct, colorScheme)"
                >
                  {{ r.changePct > 0 ? "+" : "" }}{{ fmtFixed(r.changePct, 2) }}%
                </span>
                <span v-if="r.price != null" class="px">{{ fmtFixed(r.price, 2) }}</span>
              </span>
            </button>
            <button
              type="button"
              class="toggle-pin"
              :class="{ on: isInTargetGroup(r.code) }"
              :title="isInTargetGroup(r.code) ? '移出本分组' : '加入本分组'"
              :aria-pressed="isInTargetGroup(r.code)"
              :aria-label="isInTargetGroup(r.code) ? '移出本分组' : '加入本分组'"
              @click.stop="toggleGroupMembership(r)"
            >
              <span class="toggle-pin-glyph" aria-hidden="true">
                {{ isInTargetGroup(r.code) ? "✓" : "+" }}
              </span>
            </button>
          </li>
        </ul>

        <p v-else-if="!query.trim() && hotLoading" class="empty">人气榜加载中…</p>
        <p v-else-if="query.trim() && searching" class="empty">搜索中…</p>
        <p v-else-if="query.trim() && !searching" class="empty">无匹配结果，换个关键词试试</p>
        <p v-else-if="!query.trim() && !hotLoading" class="empty">暂无人气榜数据</p>
      </section>
    </div>
    <div v-else class="hint">加载中…</div>
    <div v-if="toast" class="toast">{{ toast }}</div>
  </div>
</template>

<style scoped>
.add-page {
  container-type: inline-size;
  container-name: addpg;
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: linear-gradient(
    180deg,
    var(--yj-settings-bg-1),
    var(--yj-settings-bg-2)
  );
  color: var(--yj-text);
}

/* 与主窗口标题栏同量级：小窗盯盘时一屏尽量多看到列表 */
.add-page--dense .bar--dense {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px 6px;
  border-bottom: 1px solid var(--yj-bar-border);
  flex-shrink: 0;
  -webkit-app-region: drag;
  app-region: drag;
}

.bar--dense .back {
  flex-shrink: 0;
  -webkit-app-region: no-drag;
  app-region: no-drag;
}

.bar--dense h1 {
  flex: 1;
  margin: 0;
  font-size: 0.78em;
  font-weight: 600;
  letter-spacing: 0.04em;
  text-align: center;
  min-width: 0;
}

.bar__spacer {
  width: 36px;
  flex-shrink: 0;
  pointer-events: none;
}

.back {
  border-radius: 8px;
  border: 1px solid var(--yj-back-border);
  background: var(--yj-back-bg);
  color: var(--yj-back-color);
  padding: 4px 0;
  min-width: 36px;
  cursor: pointer;
  font-size: 1.1em;
  font-weight: 600;
  line-height: 1.1;
}

.main {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 8px 10px 10px;
  -webkit-app-region: no-drag;
  app-region: no-drag;
}

.control-strip {
  display: flex;
  flex-wrap: nowrap;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
  min-width: 0;
}

.group-row {
  display: inline-flex;
  flex-wrap: nowrap;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
  max-width: 46%;
}

.group-row__label {
  font-size: 0.72em;
  color: var(--yj-text-muted);
  flex-shrink: 0;
}

.group-select {
  min-width: 72px;
  max-width: 140px;
  flex: 1;
}

.search-box {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
  min-width: 0;
}

.history-btn {
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  padding: 0;
  border-radius: 8px;
  border: 1px solid var(--yj-field-border);
  background: var(--yj-field-bg);
  color: var(--yj-field-color);
  cursor: pointer;
  transition:
    border-color 0.15s ease,
    filter 0.15s ease;
  -webkit-app-region: no-drag;
  app-region: no-drag;
}

.history-btn:hover {
  filter: brightness(1.08);
}

.history-btn:focus-visible {
  outline: none;
  border-color: rgba(255, 255, 255, 0.22);
  box-shadow: 0 0 0 2px rgba(255, 255, 255, 0.07);
}

.yj-root[data-theme="light"] .history-btn:focus-visible {
  border-color: rgba(0, 0, 0, 0.2);
  box-shadow: 0 0 0 2px rgba(0, 0, 0, 0.06);
}

.history-btn__svg {
  display: block;
  opacity: 0.88;
}

.history-panel__empty {
  margin: 0;
  padding: 10px 12px;
  font-size: 0.78em;
  color: var(--yj-text-muted);
  text-align: center;
}

.history-panel__item {
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.history-panel__clear {
  margin-top: 2px;
  padding-top: 10px;
  border-top: 1px solid var(--yj-row-border);
  font-size: 0.78em;
  color: var(--yj-text-muted);
}

.history-panel__clear:hover {
  color: var(--yj-text);
}

.search-field {
  flex: 1;
  min-width: 0;
  max-width: none;
  padding-top: 6px;
  padding-bottom: 6px;
}

/* 与主面板列表数字列一致：等宽数字 + DM Sans */
.search-field::-webkit-search-cancel-button {
  filter: opacity(0.65);
}

.hint-inline {
  font-size: 0.72em;
  color: var(--yj-text-muted);
  flex-shrink: 0;
  width: 1em;
  text-align: center;
}

.card {
  border: 1px solid var(--yj-table-wrap-border);
  background: var(--yj-table-wrap-bg);
  border-radius: 8px;
  padding: 8px 8px 6px;
}

.results-head {
  display: flex;
  align-items: baseline;
  justify-content: flex-start;
  gap: 8px;
  flex-wrap: wrap;
  margin: 0 0 6px;
  padding: 0 2px;
}

.results-head h2 {
  margin: 0;
  font-size: 0.72em;
  font-weight: 600;
  color: var(--yj-text-muted);
  letter-spacing: 0.06em;
  text-transform: none;
}

.results-head__meta {
  font-size: 0.68em;
  color: var(--yj-text-muted);
  opacity: 0.85;
}

.row-lead {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
  flex-shrink: 0;
  width: 22px;
}

.sector-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}

.sector-dot--plain {
  background: var(--yj-text-muted);
  opacity: 0.35;
  box-shadow: none;
}

.row-main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: 2px;
}

.nm-title-line {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: 6px;
  min-width: 0;
}

.nm-title {
  flex: 1;
  min-width: 0;
  font-weight: 500;
  font-size: 0.82em;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sector-chip {
  flex-shrink: 0;
  max-width: 40%;
  padding: 1px 6px;
  border-radius: 999px;
  font-size: 0.62em;
  font-weight: 600;
  letter-spacing: 0.02em;
  line-height: 1.35;
  border: 1px solid transparent;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.rk {
  flex-shrink: 0;
  min-width: 1.2em;
  padding: 1px 4px;
  border-radius: 5px;
  font-size: 0.62em;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  line-height: 1.2;
  text-align: center;
  color: var(--yj-text-muted);
  background: var(--yj-row-hover-bg, rgba(128, 128, 128, 0.12));
  border: 1px solid var(--yj-row-border);
}

.rk--1 {
  color: #fff;
  background: linear-gradient(145deg, #e11d48, #fb7185);
  border-color: transparent;
}

.rk--2 {
  color: #fff;
  background: linear-gradient(145deg, #ea580c, #fb923c);
  border-color: transparent;
}

.rk--3 {
  color: #1c1917;
  background: linear-gradient(145deg, #ca8a04, #facc15);
  border-color: transparent;
}

.code-line {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}

.mkt-tag {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 1.15rem;
  padding: 1px 4px;
  border-radius: 4px;
  font-size: 0.62em;
  font-weight: 700;
  line-height: 1.35;
  flex-shrink: 0;
}

.mkt-tag--sh {
  background: rgba(248, 113, 113, 0.22);
  color: #fecaca;
}

.mkt-tag--sz {
  background: rgba(96, 165, 250, 0.22);
  color: #bfdbfe;
}

.mkt-tag--bj {
  background: rgba(167, 139, 250, 0.26);
  color: #ddd6fe;
}

.mkt-tag--hk {
  background: rgba(52, 211, 153, 0.22);
  color: #6ee7b7;
}

.mkt-tag--na {
  background: var(--yj-row-hover-bg, rgba(128, 128, 128, 0.12));
  color: var(--yj-text-muted);
}

.row-quote {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  justify-content: center;
  gap: 0;
  flex-shrink: 0;
  min-width: 3.6rem;
}

.chg {
  font-size: 0.8em;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  line-height: 1.2;
  white-space: nowrap;
}

.px {
  font-size: 0.65em;
  font-weight: 500;
  font-variant-numeric: tabular-nums;
  color: var(--yj-text-muted);
  line-height: 1.2;
  white-space: nowrap;
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
  color: var(--yj-flat, var(--yj-text-muted));
}

.results {
  width: 100%;
  max-width: none;
}

.list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.list li {
  display: flex;
  align-items: center;
  gap: 6px;
  border-bottom: 1px solid var(--yj-row-border);
  min-height: 44px;
}

.list li.in-group {
  background: var(--yj-row-hover-bg, rgba(128, 128, 128, 0.08));
}

.row-btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: flex-start;
  gap: 6px;
  min-width: 0;
  padding: 4px 2px;
  border: none;
  background: transparent;
  color: inherit;
  cursor: pointer;
  text-align: left;
  font: inherit;
  border-radius: 6px;
}

.row-btn:hover {
  background: var(--yj-row-hover-bg, rgba(128, 128, 128, 0.06));
}

.cd {
  font-family: "DM Sans", "Noto Sans SC", ui-monospace, monospace;
  font-variant-numeric: tabular-nums;
  font-size: 0.82em;
  color: var(--yj-text-muted);
}

.cd--sub {
  font-size: 0.74em;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.toggle-pin {
  flex-shrink: 0;
  width: 26px;
  height: 26px;
  padding: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--yj-btn-border);
  border-radius: 8px;
  background: var(--yj-btn-bg);
  color: var(--yj-btn-color);
  cursor: pointer;
  font: inherit;
  line-height: 1;
  transition:
    background 0.15s ease,
    border-color 0.15s ease,
    color 0.15s ease;
}

.toggle-pin:hover {
  filter: brightness(1.08);
}

.toggle-pin.on {
  background: var(--yj-tool-primary-bg);
  border-color: var(--yj-tool-primary-border);
  color: var(--yj-tool-primary-color);
}

.toggle-pin-glyph {
  font-size: 0.78em;
  font-weight: 600;
  line-height: 1;
  transform: translateY(-0.5px);
}

.empty {
  margin: 0;
  padding: 0.85em 8px;
  text-align: center;
  color: var(--yj-text-muted);
  font-size: 0.76em;
  line-height: 1.45;
}

/* 极窄宽度：隐藏「分组」文字，涨跌与现价同一行，收敛板块胶囊 */
@container addpg (max-width: 340px) {
  .group-row__label {
    display: none;
  }

  .group-select {
    max-width: 100px;
  }

  .sector-chip {
    display: none;
  }

  .row-quote {
    flex-direction: row;
    align-items: baseline;
    gap: 4px;
    min-width: 0;
  }

  .px {
    margin-top: 0;
  }
}

.hint {
  padding: 24px;
  color: var(--yj-text-muted);
}

.toast {
  position: fixed;
  bottom: 20px;
  left: 50%;
  transform: translateX(-50%);
  padding: 10px 16px;
  border-radius: 8px;
  font-size: 0.82em;
  background: var(--yj-tool-primary-bg);
  color: var(--yj-tool-primary-color);
  border: 1px solid var(--yj-tool-primary-border);
  z-index: 50;
  pointer-events: none;
}
</style>
