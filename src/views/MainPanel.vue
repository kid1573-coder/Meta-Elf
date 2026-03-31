<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import { useSettings } from "../composables/useSettings";
import { useQuotes } from "../composables/useQuotes";
import { useEdgeHide } from "../composables/useEdgeHide";
import { DEFAULT_QUOTE_CODES } from "../constants/quote";
import { LAST_WATCH_GROUP_KEY } from "../constants/watchGroup";
import { COLUMN_DEFS, type QuoteRow, type WatchGroup } from "../types/app";
import { changeClass, fmtFixed, fmtVolume, truncateUnicodeChars } from "../utils/format";
import { displayStockName } from "../utils/stockDisplay";
import BrandElfMascot from "../components/BrandElfMascot.vue";
import MarketRibbon from "../components/MarketRibbon.vue";
import StockDetailBody from "../components/StockDetailBody.vue";
import type { MarketRibbonSnapshot } from "../types/marketRibbon";
import { getBrandMood, getBrandMoodSingle } from "../utils/brandKaomoji";
import { orderQuoteRowsForGroupCodes, orderQuoteRowsForPanel } from "../utils/quoteDisplayOrder";
import {
  codeInAnyGroup,
  pruneWatchlistIfOrphaned,
  stripCodeFromAllGroups,
} from "../utils/watchGroups";

const router = useRouter();
const { settings, loading, save, applyWindowPrefs } = useSettings();

const codesForQuotes = computed(() => {
  const w = settings.value?.watchlist.map((x) => x.code) ?? [];
  if (w.length === 0) {
    return [...DEFAULT_QUOTE_CODES];
  }
  return w;
});
const quoteSourceRef = computed(() => settings.value?.quoteSource ?? "eastmoney");

const { rows, err, refresh, start } = useQuotes(
  codesForQuotes,
  quoteSourceRef,
  6000,
);

const selectedTab = ref<"all" | string>("all");
const groupToast = ref("");

/** 主面板右侧个股详情抽屉（不跳转全屏路由） */
const sideDetailCode = ref<string | null>(null);

const ribbon = ref<MarketRibbonSnapshot | null>(null);
const ribbonLoading = ref(false);
const ribbonErr = ref<string | null>(null);
let ribbonPoll: ReturnType<typeof setInterval> | null = null;
/** 市场快览（含板块）单次请求多路东财接口，略低于列表行情轮询，避免过猛；仍明显快于原 60s「像不刷新」 */
const RIBBON_POLL_MS = 15_000;

useEdgeHide(settings);

async function loadRibbon() {
  const src = settings.value?.quoteSource ?? "eastmoney";
  ribbonLoading.value = true;
  ribbonErr.value = null;
  try {
    ribbon.value = await invoke<MarketRibbonSnapshot>("get_market_ribbon", { quoteSource: src });
  } catch (e) {
    ribbonErr.value = String(e);
  } finally {
    ribbonLoading.value = false;
  }
}

async function refreshQuotesAndRibbon() {
  await refresh();
  void loadRibbon();
}

watch(
  () => loading.value,
  (v) => {
    if (!v) void loadRibbon();
  },
  { immediate: true },
);

watch(quoteSourceRef, () => {
  void loadRibbon();
});

onMounted(() => {
  start();
  ribbonPoll = setInterval(() => void loadRibbon(), RIBBON_POLL_MS);
});

function persistLastGroupId(id: string) {
  try {
    sessionStorage.setItem(LAST_WATCH_GROUP_KEY, id);
  } catch {
    /* ignore */
  }
}

function selectTab(next: "all" | string) {
  selectedTab.value = next;
  if (next !== "all") persistLastGroupId(next);
}

watch(
  () => settings.value?.watchGroups,
  (groups) => {
    if (!groups?.length) return;
    if (selectedTab.value !== "all" && !groups.some((g) => g.id === selectedTab.value)) {
      selectedTab.value = "all";
    }
  },
  { deep: true },
);

function defaultTargetGroupId(): string {
  const s = settings.value;
  if (!s?.watchGroups.length) return "";
  if (selectedTab.value !== "all" && s.watchGroups.some((g) => g.id === selectedTab.value)) {
    return selectedTab.value;
  }
  try {
    const last = sessionStorage.getItem(LAST_WATCH_GROUP_KEY);
    if (last && s.watchGroups.some((g) => g.id === last)) return last;
  } catch {
    /* ignore */
  }
  return s.watchGroups[0].id;
}

function groupCodeIndex(g: WatchGroup, code: string): number {
  const c = code.toLowerCase();
  return g.codes.findIndex((x) => x.toLowerCase() === c);
}

function showGroupToast(msg: string) {
  groupToast.value = msg;
  setTimeout(() => {
    groupToast.value = "";
  }, 2000);
}

const visibleCols = computed(() => {
  if (sideDetailCode.value?.trim()) {
    return COLUMN_DEFS.filter((c) => c.id === "name");
  }
  const s = settings.value;
  if (!s)
    return COLUMN_DEFS.filter((c) =>
      ["name", "price", "changePct", "volumeRatio", "sectorBlock", "sectorPct"].includes(c.id),
    );
  const set = new Set(s.visibleColumns);
  // 保持 COLUMN_DEFS 中定义的顺序
  return COLUMN_DEFS.filter((c) => set.has(c.id));
});

const displayRows = computed(() => {
  const s = settings.value;
  if (!s) return rows.value;
  if (selectedTab.value === "all") {
    return orderQuoteRowsForPanel(rows.value, s.watchlist, "all");
  }
  const g = s.watchGroups.find((x) => x.id === selectedTab.value);
  if (!g) return orderQuoteRowsForPanel(rows.value, s.watchlist, "all");
  return orderQuoteRowsForGroupCodes(rows.value, g.codes);
});

const moodQuoteRows = computed(() => {
  const s = settings.value;
  if (!s) return [];
  if (selectedTab.value === "all") {
    return orderQuoteRowsForPanel(rows.value, s.watchlist, "watch");
  }
  const g = s.watchGroups.find((x) => x.id === selectedTab.value);
  if (!g) return [];
  return orderQuoteRowsForGroupCodes(rows.value, g.codes);
});

const brandMood = computed(() => {
  const s = settings.value;
  const wl = s?.watchlist ?? [];
  const focus = sideDetailCode.value?.trim().toLowerCase();
  if (focus) {
    const row = rows.value.find((r) => r.code.trim().toLowerCase() === focus);
    if (row) {
      return getBrandMoodSingle(focus, row, displayStockName(row, s ?? null));
    }
  }
  return getBrandMood(wl, moodQuoteRows.value);
});

const brandTheme = computed(() =>
  settings.value?.theme === "light" ? "light" : "dark",
);

const emptyTabHint = computed(() => {
  if (selectedTab.value === "all") return "暂无数据";
  return "本分组暂无可见行情：点「+」添加自选，或到「自选股管理」检查列表。";
});

function onAddClick() {
  const gid = defaultTargetGroupId();
  if (gid) {
    router.push({ path: "/watchlist/add", query: { group: gid } });
  } else {
    router.push("/watchlist/add");
  }
}

function openDetail(row: QuoteRow) {
  sideDetailCode.value = row.code.trim().toLowerCase();
}

function closeSideDetail() {
  sideDetailCode.value = null;
}

function openMarketMovesPage() {
  router.push("/market-moves");
}

function displayName(row: QuoteRow): string {
  return displayStockName(row, settings.value);
}

const SECTOR_LABEL_MAX_CHARS = 4;

function sectorBlockCellText(row: QuoteRow): string {
  const n = row.sector?.trim() ?? "";
  if (!n) return "—";
  return truncateUnicodeChars(n, SECTOR_LABEL_MAX_CHARS);
}

function cell(row: QuoteRow, colId: string): string {
  switch (colId) {
    case "name":
      return displayName(row);
    case "changePct":
      return `${fmtFixed(row.changePct, 2)}%`;
    case "volumeRatio":
      return row.volumeRatio != null && Number.isFinite(row.volumeRatio)
        ? fmtFixed(row.volumeRatio, 2)
        : "—";
    case "sectorBlock":
      return sectorBlockCellText(row);
    case "sectorPct": {
      const p = row.sectorChangePct;
      if (p == null || !Number.isFinite(p)) return "—";
      return `${p >= 0 ? "+" : ""}${fmtFixed(p, 2)}%`;
    }
    case "amplitude": {
      if (row.prevClose <= 0 || !Number.isFinite(row.prevClose)) return "—";
      const amp = ((row.high - row.low) / row.prevClose) * 100;
      return Number.isFinite(amp) ? `${fmtFixed(amp, 2)}%` : "—";
    }
    case "price":
      return fmtFixed(row.price, 2);
    case "prevClose":
      return fmtFixed(row.prevClose, 2);
    case "open":
      return fmtFixed(row.open, 2);
    case "volume":
      return fmtVolume(row.volume);
    case "turnover":
      return fmtVolume(row.turnover);
    case "turnoverRate":
      return `${fmtFixed(row.turnoverRate, 2)}%`;
    case "commissionRatio":
      return `${fmtFixed(row.commissionRatio, 2)}%`;
    case "totalPl":
      return fmtFixed(row.totalPl, 2);
    case "dailyPl":
      return fmtFixed(row.dailyPl, 2);
    case "high":
      return fmtFixed(row.high, 2);
    case "low":
      return fmtFixed(row.low, 2);
    case "bid1":
      return fmtFixed(row.bid1, 3);
    case "ask1":
      return fmtFixed(row.ask1, 3);
    default:
      return "—";
  }
}

function pctForRow(row: QuoteRow, colId: string): number | null {
  if (colId === "changePct") return row.changePct;
  if (colId === "dailyPl") return row.dailyPl;
  if (colId === "sectorPct") {
    const p = row.sectorChangePct;
    return p != null && Number.isFinite(p) ? p : null;
  }
  return null;
}

async function minimizeWin() {
  await invoke("minimize_main_window");
}

async function toggleAlwaysOnTop() {
  if (!settings.value) return;
  const next = !settings.value.alwaysOnTop;
  await save({ alwaysOnTop: next });
  await applyWindowPrefs();
}

async function toggleTheme() {
  if (!settings.value) return;
  const next = settings.value.theme === "light" ? "dark" : "light";
  await save({ theme: next });
}

const minimal = computed(() => settings.value?.panelMode === "minimal");

/** 右键菜单：已在自选或当前分组内 → 排序/删除；否则 → 加入自选 + 查看详情 */
const ctxOpen = ref(false);
const ctxX = ref(0);
const ctxY = ref(0);
const ctxRow = ref<QuoteRow | null>(null);

function watchlistIndex(code: string): number {
  const c = code.toLowerCase();
  return settings.value?.watchlist.findIndex((w) => w.code.toLowerCase() === c) ?? -1;
}

function rowInWatchlist(row: QuoteRow): boolean {
  return watchlistIndex(row.code) >= 0;
}

/** 是否展示「自选级」菜单：在全局自选中，或当前选中的分组里包含该代码（修复仅分组有、自选数组缺项时的菜单误判） */
function rowShowsManagedCtxMenu(row: QuoteRow): boolean {
  const s = settings.value;
  if (!s) return false;
  if (rowInWatchlist(row)) return true;
  if (selectedTab.value === "all") return false;
  const g = s.watchGroups.find((x) => x.id === selectedTab.value);
  if (!g) return false;
  return groupCodeIndex(g, row.code) >= 0;
}

const ctxRowManaged = computed(() =>
  ctxRow.value ? rowShowsManagedCtxMenu(ctxRow.value) : false,
);

function onRowContextMenu(e: MouseEvent, row: QuoteRow) {
  e.preventDefault();
  e.stopPropagation();
  closeTabCtxMenu();
  ctxRow.value = row;
  const pad = 8;
  const mw = 200;
  const mh = rowShowsManagedCtxMenu(row) ? 248 : 132;
  ctxX.value = Math.max(pad, Math.min(e.clientX, window.innerWidth - mw - pad));
  ctxY.value = Math.max(pad, Math.min(e.clientY, window.innerHeight - mh - pad));
  ctxOpen.value = true;
}

async function ctxAddToWatchlist() {
  const row = ctxRow.value;
  if (!row || !settings.value) return;
  if (rowInWatchlist(row)) return;
  const code = row.code.trim().toLowerCase();
  const name = (() => {
    const n = displayName(row).trim();
    return n.toLowerCase() !== code ? n : code;
  })();
  settings.value.watchlist.push({ code, name });
  const gid = defaultTargetGroupId();
  const g = settings.value.watchGroups.find((x) => x.id === gid);
  if (g && !g.codes.some((c) => c.toLowerCase() === code)) {
    g.codes.push(code);
  }
  await save();
  closeCtxMenu();
}

function ctxOpenDetailAndClose() {
  const row = ctxRow.value;
  if (!row) return;
  closeCtxMenu();
  openDetail(row);
}

/** WebView2 会弹出系统「返回/刷新/另存为」菜单；在捕获阶段拦截 .panel 内所有默认右键 */
function onPanelBlockNativeContextMenu(e: MouseEvent) {
  const el = e.target;
  if (!(el instanceof Element)) return;
  if (el.closest(".ctx-menu")) {
    e.preventDefault();
    return;
  }
  if (!el.closest(".panel")) return;
  e.preventDefault();
}

function closeCtxMenu() {
  ctxOpen.value = false;
  ctxRow.value = null;
}

const tabCtxOpen = ref(false);
const tabCtxX = ref(0);
const tabCtxY = ref(0);
const tabCtxGroup = ref<WatchGroup | null>(null);

function onGroupTabContextMenu(e: MouseEvent, g: WatchGroup) {
  e.preventDefault();
  e.stopPropagation();
  closeCtxMenu();
  tabCtxGroup.value = g;
  const pad = 8;
  const mw = 168;
  const mh = 120;
  tabCtxX.value = Math.max(pad, Math.min(e.clientX, window.innerWidth - mw - pad));
  tabCtxY.value = Math.max(pad, Math.min(e.clientY, window.innerHeight - mh - pad));
  tabCtxOpen.value = true;
}

function closeTabCtxMenu() {
  tabCtxOpen.value = false;
  tabCtxGroup.value = null;
}

const newGroupOpen = ref(false);
const newGroupName = ref("");

function openNewGroupModal() {
  newGroupName.value = "";
  newGroupOpen.value = true;
  closeTabCtxMenu();
}

async function confirmNewGroup() {
  if (!settings.value) return;
  const name = newGroupName.value.trim() || "新分组";
  const id = crypto.randomUUID();
  settings.value.watchGroups.push({ id, name, codes: [] });
  await save();
  newGroupOpen.value = false;
  selectTab(id);
}

const renameOpen = ref(false);
const renameValue = ref("");
const renameTargetId = ref("");

function openRenameGroup() {
  const g = tabCtxGroup.value;
  if (!g) return;
  renameTargetId.value = g.id;
  renameValue.value = g.name;
  renameOpen.value = true;
  closeTabCtxMenu();
}

async function confirmRenameGroup() {
  if (!settings.value) return;
  const g = settings.value.watchGroups.find((x) => x.id === renameTargetId.value);
  if (!g) return;
  const name = renameValue.value.trim();
  if (name) g.name = name;
  renameOpen.value = false;
  await save();
}

async function deleteGroupFromMenu() {
  const g = tabCtxGroup.value;
  if (!g || !settings.value) return;
  closeTabCtxMenu();
  if (settings.value.watchGroups.length <= 1) {
    showGroupToast("至少保留一个分组");
    return;
  }
  if (!confirm(`确定删除分组「${g.name}」？组内股票若未出现在其他分组，将从自选中移除。`)) return;
  const codes = [...g.codes];
  const idx = settings.value.watchGroups.findIndex((x) => x.id === g.id);
  if (idx < 0) return;
  settings.value.watchGroups.splice(idx, 1);
  for (const code of codes) {
    if (!codeInAnyGroup(settings.value.watchGroups, code)) {
      const wi = settings.value.watchlist.findIndex(
        (w) => w.code.toLowerCase() === code.toLowerCase(),
      );
      if (wi >= 0) settings.value.watchlist.splice(wi, 1);
    }
  }
  if (selectedTab.value === g.id) {
    selectedTab.value = settings.value.watchGroups[0]?.id ?? "all";
  }
  await save();
}

/** 用 click 冒泡关闭，避免 capture 阶段 pointerdown 与菜单项点击抢事件（WebView2 下部分区域点不中） */
function onDocClickCloseCtx(e: MouseEvent) {
  const t = e.target as HTMLElement;
  if (t.closest?.(".ctx-menu") || t.closest?.(".yj-modal")) return;
  closeCtxMenu();
  closeTabCtxMenu();
}

onMounted(() => {
  document.addEventListener("contextmenu", onPanelBlockNativeContextMenu, true);
  document.addEventListener("click", onDocClickCloseCtx);
});

function onDetailEsc(e: KeyboardEvent) {
  if (e.key === "Escape") closeSideDetail();
}

watch(sideDetailCode, (code) => {
  if (code) document.addEventListener("keydown", onDetailEsc);
  else document.removeEventListener("keydown", onDetailEsc);
});

onUnmounted(() => {
  document.removeEventListener("contextmenu", onPanelBlockNativeContextMenu, true);
  document.removeEventListener("click", onDocClickCloseCtx);
  document.removeEventListener("keydown", onDetailEsc);
  if (ribbonPoll != null) {
    clearInterval(ribbonPoll);
    ribbonPoll = null;
  }
});

const ctxSortContext = computed(() => {
  const row = ctxRow.value;
  const s = settings.value;
  if (!row || !s) return { mode: "none" as const };
  if (selectedTab.value === "all") {
    return { mode: "watchlist" as const, index: watchlistIndex(row.code) };
  }
  const g = s.watchGroups.find((x) => x.id === selectedTab.value);
  if (!g) return { mode: "none" as const };
  return { mode: "group" as const, group: g, index: groupCodeIndex(g, row.code) };
});

const ctxCanUp = computed(
  () => ctxSortContext.value.mode !== "none" && ctxSortContext.value.index > 0,
);
const ctxCanDown = computed(() => {
  const c = ctxSortContext.value;
  if (c.mode === "watchlist") {
    const wl = settings.value?.watchlist.length ?? 0;
    return c.index >= 0 && c.index < wl - 1;
  }
  if (c.mode === "group") {
    return c.index >= 0 && c.index < c.group.codes.length - 1;
  }
  return false;
});

const ctxRemoveLabel = computed(() =>
  selectedTab.value === "all" ? "删除自选" : "从本分组移除",
);

async function ctxRemove() {
  const row = ctxRow.value;
  if (!row || !settings.value) return;
  if (selectedTab.value === "all") {
    const i = watchlistIndex(row.code);
    if (i < 0) return;
    settings.value.watchlist.splice(i, 1);
    stripCodeFromAllGroups(settings.value.watchGroups, row.code);
    await save();
    closeCtxMenu();
    return;
  }
  const g = settings.value.watchGroups.find((x) => x.id === selectedTab.value);
  if (!g) return;
  const gi = groupCodeIndex(g, row.code);
  if (gi < 0) return;
  g.codes.splice(gi, 1);
  pruneWatchlistIfOrphaned(settings.value);
  await save();
  closeCtxMenu();
}

async function ctxToTop() {
  const row = ctxRow.value;
  if (!row || !settings.value) return;
  const ctx = ctxSortContext.value;
  if (ctx.mode === "watchlist") {
    if (ctx.index <= 0) {
      closeCtxMenu();
      return;
    }
    const [item] = settings.value.watchlist.splice(ctx.index, 1);
    settings.value.watchlist.unshift(item);
    await save();
    closeCtxMenu();
    return;
  }
  if (ctx.mode === "group" && ctx.index > 0) {
    const arr = ctx.group.codes;
    const [entry] = arr.splice(ctx.index, 1);
    arr.unshift(entry);
    await save();
    closeCtxMenu();
    return;
  }
  closeCtxMenu();
}

async function ctxToBottom() {
  const row = ctxRow.value;
  if (!row || !settings.value) return;
  const ctx = ctxSortContext.value;
  if (ctx.mode === "watchlist") {
    const wl = settings.value.watchlist;
    if (ctx.index < 0 || ctx.index >= wl.length - 1) {
      closeCtxMenu();
      return;
    }
    const [item] = wl.splice(ctx.index, 1);
    wl.push(item);
    await save();
    closeCtxMenu();
    return;
  }
  if (ctx.mode === "group") {
    const arr = ctx.group.codes;
    if (ctx.index < 0 || ctx.index >= arr.length - 1) {
      closeCtxMenu();
      return;
    }
    const [entry] = arr.splice(ctx.index, 1);
    arr.push(entry);
    await save();
    closeCtxMenu();
    return;
  }
  closeCtxMenu();
}

async function ctxMoveUp() {
  const row = ctxRow.value;
  if (!row || !settings.value) return;
  const ctx = ctxSortContext.value;
  if (ctx.mode === "watchlist") {
    const wl = settings.value.watchlist;
    if (ctx.index <= 0) {
      closeCtxMenu();
      return;
    }
    [wl[ctx.index - 1], wl[ctx.index]] = [wl[ctx.index], wl[ctx.index - 1]];
    await save();
    closeCtxMenu();
    return;
  }
  if (ctx.mode === "group") {
    const arr = ctx.group.codes;
    const i = ctx.index;
    if (i <= 0) {
      closeCtxMenu();
      return;
    }
    [arr[i - 1], arr[i]] = [arr[i], arr[i - 1]];
    await save();
    closeCtxMenu();
    return;
  }
  closeCtxMenu();
}

async function ctxMoveDown() {
  const row = ctxRow.value;
  if (!row || !settings.value) return;
  const ctx = ctxSortContext.value;
  if (ctx.mode === "watchlist") {
    const wl = settings.value.watchlist;
    if (ctx.index < 0 || ctx.index >= wl.length - 1) {
      closeCtxMenu();
      return;
    }
    [wl[ctx.index], wl[ctx.index + 1]] = [wl[ctx.index + 1], wl[ctx.index]];
    await save();
    closeCtxMenu();
    return;
  }
  if (ctx.mode === "group") {
    const arr = ctx.group.codes;
    const i = ctx.index;
    if (i < 0 || i >= arr.length - 1) {
      closeCtxMenu();
      return;
    }
    [arr[i], arr[i + 1]] = [arr[i + 1], arr[i]];
    await save();
    closeCtxMenu();
    return;
  }
  closeCtxMenu();
}
</script>

<template>
  <div class="panel" :class="{ minimal }">
    <header class="titlebar" data-tauri-drag-region>
      <div class="brand" :title="brandMood.title" @click.stop="toggleTheme">
        <BrandElfMascot
          class="brand-face"
          :bucket="brandMood.bucket"
          :variant="brandMood.variant"
          :theme="brandTheme"
          :ariaLabel="brandMood.ariaLabel"
        />
        <span class="name">元精灵</span>
      </div>
      <div class="tabs" data-tauri-drag-region>
        <button
          type="button"
          class="tab"
          :class="{ on: selectedTab === 'all' }"
          @click="selectTab('all')"
        >
          全部
        </button>
        <button
          v-for="g in settings?.watchGroups ?? []"
          :key="g.id"
          type="button"
          class="tab"
          :class="{ on: selectedTab === g.id }"
          @click="selectTab(g.id)"
          @contextmenu.prevent.stop="onGroupTabContextMenu($event, g)"
        >
          {{ g.name }}
        </button>
        <button
          type="button"
          class="tab tab--add"
          title="新建分组"
          aria-label="新建分组"
          @click="openNewGroupModal"
        >
          +
        </button>
      </div>
      <span v-if="groupToast" class="group-toast">{{ groupToast }}</span>
      <div class="win-actions">
        <button
          v-if="minimal"
          type="button"
          class="icon-btn icon-btn-add"
          title="添加自选"
          aria-label="添加自选"
          @click="onAddClick"
        >
          +
        </button>
        <button
          type="button"
          class="icon-btn icon-btn-moves"
          title="股市异动"
          aria-label="股市异动"
          @click="openMarketMovesPage"
        >
          <svg
            viewBox="0 0 24 24"
            width="13"
            height="13"
            aria-hidden="true"
            fill="none"
            stroke="currentColor"
            stroke-width="1.75"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M13 2L3 14h8l-2 8 10-12h-8l2-8z" />
          </svg>
        </button>
        <button
          type="button"
          class="icon-btn icon-btn-pin"
          :class="{ 'icon-btn-pin--on': settings?.alwaysOnTop }"
          :title="
            settings?.alwaysOnTop
              ? '已置于最前（点击取消）'
              : '未置于最前（点击置顶）'
          "
          :aria-pressed="settings?.alwaysOnTop ?? false"
          aria-label="主窗口置顶开关"
          @click="toggleAlwaysOnTop"
        >
          <svg
            class="circle-svg"
            viewBox="0 0 24 24"
            width="13"
            height="13"
            aria-hidden="true"
          >
            <circle
              class="circle-svg__ring"
              cx="12"
              cy="12"
              r="6.75"
              fill="none"
              stroke="currentColor"
              stroke-width="1.6"
            />
            <circle
              class="circle-svg__core"
              cx="12"
              cy="12"
              r="3.6"
              fill="currentColor"
            />
          </svg>
        </button>
        <button type="button" class="icon-btn" title="最小化" @click="minimizeWin">
          —
        </button>
      </div>
    </header>

    <div v-if="loading" class="hint">加载配置…</div>
    <div v-else class="panel-main" :class="{ 'panel-main--detail-open': !!sideDetailCode }">
      <div class="panel-main__primary">
        <div class="panel-stack">
          <div class="body">
            <div v-if="err" class="err">{{ err }}</div>
            <div class="table-wrap" :class="{ 'table-wrap--detail-rail': !!sideDetailCode }">
              <table class="grid">
                <thead>
                  <tr>
                    <th
                      v-for="c in visibleCols"
                      :key="c.id"
                      :class="{ num: c.id !== 'name', 'col-name': c.id === 'name' }"
                    >
                      {{ c.label }}
                    </th>
                    <th class="filler"></th>
                  </tr>
                </thead>
                <tbody>
                  <template v-if="displayRows.length === 0">
                    <tr>
                      <td :colspan="visibleCols.length + 1" class="empty-msg">
                        {{ emptyTabHint }}
                      </td>
                    </tr>
                  </template>
                  <template v-else>
                    <tr
                      v-for="r in displayRows"
                      :key="r.code"
                      class="row-click"
                      @click="openDetail(r)"
                      @contextmenu="onRowContextMenu($event, r)"
                    >
                      <td
                        v-for="c in visibleCols"
                        :key="c.id"
                        :class="{
                          num: c.id !== 'name',
                          'col-name': c.id === 'name',
                          chg:
                            c.id === 'changePct' ||
                            c.id === 'dailyPl' ||
                            c.id === 'totalPl',
                        }"
                      >
                        <span
                          v-if="c.id === 'sectorBlock'"
                          class="sector-block-cell sector-block-cell--w4 num"
                          :title="r.sector?.trim() ? r.sector.trim() : undefined"
                        >{{ sectorBlockCellText(r) }}</span>
                        <span
                          v-else-if="pctForRow(r, c.id) !== null"
                          :class="changeClass(pctForRow(r, c.id)!, settings?.colorScheme ?? 'redUp')"
                        >
                          {{ cell(r, c.id) }}
                        </span>
                        <span v-else>{{ cell(r, c.id) }}</span>
                      </td>
                      <td class="filler"></td>
                    </tr>
                  </template>
                </tbody>
              </table>
            </div>
          </div>

          <footer v-if="!minimal" class="toolbar">
            <div class="toolbar__tools">
              <button type="button" class="tool" title="刷新" @click="refreshQuotesAndRibbon">
                ↻
              </button>
              <button type="button" class="tool" title="设置" @click="router.push('/settings')">
                ⚙
              </button>
              <button
                type="button"
                class="tool tool-add"
                title="添加自选"
                @click="onAddClick"
              >
                <span class="tool-add-inner" aria-hidden="true">+</span>
              </button>
            </div>
            <MarketRibbon
              v-if="settings"
              :snapshot="ribbon"
              :loading="ribbonLoading"
              :error="ribbonErr"
              :color-scheme="settings.colorScheme ?? 'redUp'"
              :theme="settings.theme === 'light' ? 'light' : 'dark'"
              :hide-breadth-stats="!!sideDetailCode"
              @retry="loadRibbon"
            />
          </footer>
        </div>
      </div>

      <!-- 与列表并排：整块主区左窄右宽，从右侧「推入」详情，非叠在主界面之上 -->
      <Transition name="detail-rail">
        <aside
          v-if="sideDetailCode"
          :key="sideDetailCode"
          class="detail-rail detail-rail--grow"
          aria-label="个股详情"
        >
          <StockDetailBody embedded :code="sideDetailCode" @close="closeSideDetail" />
        </aside>
      </Transition>
    </div>

    <!-- 挂到 #yj-root 才能继承深色/浅色 CSS 变量；挂 body 时变量未定义，字会变成默认黑字 -->
    <Teleport to="#yj-root">
      <div
        v-if="ctxOpen && ctxRow"
        class="ctx-menu"
        :class="{ 'ctx-menu--compact': !ctxRowManaged }"
        :style="{ left: ctxX + 'px', top: ctxY + 'px' }"
        role="menu"
        @click.stop
        @contextmenu.prevent
      >
        <div class="ctx-title">{{ displayName(ctxRow) }}</div>

        <template v-if="ctxRowManaged">
          <button type="button" class="ctx-item" @click="ctxToTop">置顶</button>
          <button type="button" class="ctx-item" @click="ctxToBottom">置底</button>
          <button type="button" class="ctx-item" :disabled="!ctxCanUp" @click="ctxMoveUp">
            上移
          </button>
          <button type="button" class="ctx-item" :disabled="!ctxCanDown" @click="ctxMoveDown">
            下移
          </button>
          <button type="button" class="ctx-item danger" @click="ctxRemove">{{ ctxRemoveLabel }}</button>
        </template>
        <template v-else>
          <button type="button" class="ctx-item" @click="ctxAddToWatchlist">加入自选</button>
          <button type="button" class="ctx-item" @click="ctxOpenDetailAndClose">查看详情</button>
        </template>
      </div>
    </Teleport>

    <Teleport to="#yj-root">
      <div
        v-if="tabCtxOpen && tabCtxGroup"
        class="ctx-menu"
        :style="{ left: tabCtxX + 'px', top: tabCtxY + 'px' }"
        role="menu"
        @click.stop
        @contextmenu.prevent
      >
        <div class="ctx-title">{{ tabCtxGroup.name }}</div>
        <button type="button" class="ctx-item" @click="openRenameGroup">重命名</button>
        <button type="button" class="ctx-item danger" @click="deleteGroupFromMenu">删除分组</button>
      </div>
    </Teleport>

    <Teleport to="#yj-root">
      <div v-if="newGroupOpen" class="yj-modal" @click.self="newGroupOpen = false">
        <div class="modal-panel" role="dialog" aria-labelledby="new-group-title" @click.stop>
          <h3 id="new-group-title" class="modal-title">新建分组</h3>
          <input
            v-model="newGroupName"
            type="text"
            class="yj-field-control modal-panel__input"
            placeholder="分组名称"
            @keydown.enter.prevent="confirmNewGroup"
          />
          <div class="modal-actions">
            <button type="button" class="modal-btn" @click="newGroupOpen = false">取消</button>
            <button type="button" class="modal-btn primary" @click="confirmNewGroup">创建</button>
          </div>
        </div>
      </div>
    </Teleport>

    <Teleport to="#yj-root">
      <div v-if="renameOpen" class="yj-modal" @click.self="renameOpen = false">
        <div class="modal-panel" role="dialog" aria-labelledby="rename-group-title" @click.stop>
          <h3 id="rename-group-title" class="modal-title">重命名分组</h3>
          <input
            v-model="renameValue"
            type="text"
            class="yj-field-control modal-panel__input"
            placeholder="新名称"
            @keydown.enter.prevent="confirmRenameGroup"
          />
          <div class="modal-actions">
            <button type="button" class="modal-btn" @click="renameOpen = false">取消</button>
            <button type="button" class="modal-btn primary" @click="confirmRenameGroup">保存</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  position: relative;
  --detail-rail-width: min(72vw, 820px);
}

/* 板块列：固定约 4 个中文字宽，与「涨幅」列对齐整齐 */
.sector-block-cell {
  display: inline-block;
  vertical-align: bottom;
}

.sector-block-cell--w4 {
  width: 4em;
  min-width: 4em;
  max-width: 4em;
  overflow: hidden;
  text-overflow: clip;
  white-space: nowrap;
  text-align: left;
  color: color-mix(in srgb, var(--yj-text-muted, #888) 82%, var(--yj-text, #e5e5e5));
  font-variant-numeric: tabular-nums;
}

.sector-block-cell--w4[title]:not([title=""]) {
  cursor: default;
}

/* 小窗极简：打开详情时让侧栏占满主区宽度，列表被挤没，避免窄缝里看图 */
.panel.minimal {
  --detail-rail-width: 100%;
}

/* 标题栏以下：横向 flex，左侧为完整主界面（列表+底栏），右侧为并排滑入的详情轨 */
.panel-main {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: row;
  align-items: stretch;
  overflow: hidden;
}

.panel-main__primary {
  flex: 1 1 0;
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

/*
 * 并排详情：左侧用固定窄轨（rem，不跟 vw 放大），把横向尽量留给主图；子级一律 100% 铺满该轨，避免百分比相对整行误算撑宽。
 */
.panel-main--detail-open {
  --yj-name-rail-w: clamp(5.5rem, 12vw, 8.875rem);
}

.panel-main--detail-open .panel-main__primary {
  flex: 0 0 var(--yj-name-rail-w);
  width: var(--yj-name-rail-w);
  min-width: var(--yj-name-rail-w);
  max-width: var(--yj-name-rail-w);
  align-items: stretch;
  box-sizing: border-box;
}

.panel-main--detail-open .panel-stack {
  width: 100%;
  min-width: 0;
  box-sizing: border-box;
}

.panel-main--detail-open .body {
  width: 100%;
  min-width: 0;
  box-sizing: border-box;
  padding: 0 3px 4px;
}

.panel-main--detail-open .table-wrap.table-wrap--detail-rail {
  width: 100%;
  min-width: 0;
  box-sizing: border-box;
}

.panel-main--detail-open .toolbar {
  box-sizing: border-box;
  overflow: hidden;
  min-width: 0;
  width: 100%;
}

/* 与 .panel.minimal 说明一致：极简窗打开详情时列表不占宽，详情占满主区横条 */
.panel.minimal .panel-main--detail-open .panel-main__primary {
  flex: 0 0 0;
  width: 0;
  min-width: 0;
  max-width: 0;
  overflow: hidden;
  opacity: 0;
  visibility: hidden;
  pointer-events: none;
}

.panel.minimal .panel-main--detail-open .detail-rail {
  flex: 1 1 100%;
}

.panel-stack {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  position: relative;
}

.detail-rail {
  flex: none;
  box-sizing: border-box;
  width: min(var(--detail-rail-width), 100%);
  max-width: 100%;
  min-height: 0;
  display: flex;
  flex-direction: column;
  align-items: stretch;
  overflow: hidden;
  border-left: 1px solid var(--yj-bar-border, rgba(255, 255, 255, 0.14));
  box-shadow: -10px 0 40px rgba(0, 0, 0, 0.28);
}

/* 与列表并排时占满主区剩余宽度；!important 防止与 .detail-rail 的 width:min(...) 并存时竞态导致中间留白 */
.detail-rail--grow {
  flex: 1 1 0% !important;
  width: 0 !important;
  min-width: 0 !important;
  max-width: none !important;
}

.detail-rail-enter-active,
.detail-rail-leave-active {
  overflow: hidden;
  transition:
    width 0.38s cubic-bezier(0.22, 1, 0.36, 1),
    opacity 0.28s ease,
    border-left-width 0.28s ease,
    box-shadow 0.38s ease;
}

.detail-rail-enter-from,
.detail-rail-leave-to {
  width: 0 !important;
  min-width: 0;
  border-left-width: 0;
  box-shadow: none;
}

.detail-rail--grow.detail-rail-enter-active,
.detail-rail--grow.detail-rail-leave-active {
  transition:
    opacity 0.28s ease,
    border-left-width 0.28s ease,
    box-shadow 0.38s ease;
}

.detail-rail--grow.detail-rail-enter-from,
.detail-rail--grow.detail-rail-leave-to {
  flex: 0 1 0 !important;
  width: 0 !important;
  min-width: 0 !important;
  max-width: none !important;
  opacity: 0;
  border-left-width: 0;
  box-shadow: none;
}

@media (prefers-reduced-motion: reduce) {
  .detail-rail-enter-active,
  .detail-rail-leave-active {
    transition-duration: 0.15s;
  }

  .detail-rail--grow.detail-rail-enter-active,
  .detail-rail--grow.detail-rail-leave-active {
    transition-duration: 0.15s;
  }
}

.panel.minimal .body {
  padding-bottom: 4px;
}

/* Windows WebView2：无边框窗口需在 CSS 中声明可拖动区，否则 data-tauri-drag-region 不生效 */
.titlebar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 2px 8px;
  border-bottom: 1px solid var(--yj-titlebar-border);
  color: var(--yj-text);
  user-select: none;
  -webkit-app-region: drag;
  app-region: drag;
}

.titlebar button,
.titlebar .tab,
.titlebar .brand {
  -webkit-app-region: no-drag;
  app-region: no-drag;
}

.brand {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  letter-spacing: 0.04em;
  cursor: pointer;
  border-radius: 8px;
  margin: -2px -4px;
  padding: 2px 4px;
}

.brand-face {
  flex-shrink: 0;
  line-height: 0;
  display: flex;
  align-items: center;
}

.name {
  font-size: 0.92em;
  opacity: 0.95;
}

.tabs {
  display: flex;
  align-items: center;
  gap: 4px;
  flex: 1;
  min-width: 0;
  overflow-x: auto;
  overflow-y: hidden;
  flex-wrap: nowrap;
  scrollbar-width: thin;
}

.tab--add {
  flex-shrink: 0;
  min-width: 28px;
  padding: 2px 8px;
  font-weight: 600;
  line-height: 1;
}

.group-toast {
  font-size: 0.72em;
  color: var(--yj-err, #fca5a5);
  white-space: nowrap;
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tab {
  border: none;
  background: var(--yj-tab-bg);
  color: var(--yj-tab-color);
  padding: 2px 10px;
  border-radius: 999px;
  font-size: 0.78em;
  cursor: pointer;
  transition: background 0.18s ease, color 0.18s ease;
}

.tab.on {
  background: var(--yj-tab-on-bg);
  color: var(--yj-tab-on-color);
}

.win-actions {
  display: flex;
  align-items: center;
  gap: 4px;
  -webkit-app-region: no-drag;
  app-region: no-drag;
}

.icon-btn {
  width: 28px;
  height: 24px;
  border-radius: 8px;
  border: 1px solid var(--yj-icon-border);
  background: var(--yj-icon-bg);
  color: var(--yj-icon-color);
  cursor: pointer;
  line-height: 1;
}

.win-actions .icon-btn {
  width: 20px;
  height: 18px;
  border-radius: 5px;
  font-size: 0.68em;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0;
}

.win-actions .icon-btn-add {
  font-weight: 700;
  font-size: 0.78em;
}

.win-actions .icon-btn-moves svg {
  opacity: 0.85;
}

.win-actions .icon-btn-pin .circle-svg {
  display: block;
  flex-shrink: 0;
}

/* 未置顶：空心圆略淡；置顶：内圆实心，仍无按钮底色变化 */
.win-actions .icon-btn-pin:not(.icon-btn-pin--on) .circle-svg {
  opacity: 0.4;
}

.win-actions .icon-btn-pin--on .circle-svg {
  opacity: 1;
}

.win-actions .icon-btn-pin:not(.icon-btn-pin--on) .circle-svg__core {
  opacity: 0;
}

.win-actions .icon-btn-pin--on .circle-svg__core {
  opacity: 0.85;
}

.icon-btn:hover {
  background: var(--yj-icon-hover-bg);
}

.hint,
.err {
  padding: 12px 14px;
  font-size: 0.85em;
}

.hint {
  color: var(--yj-text-muted);
}

.err {
  color: var(--yj-err);
}

.grid td.empty-msg {
  text-align: center;
  vertical-align: middle;
  padding: 1.35em 12px;
  font-size: 0.86em;
  line-height: 1.45;
  color: var(--yj-text-muted);
  border-bottom: none !important;
}

.body {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  padding: 0 6px 2px;
}

.table-wrap {
  flex: 1;
  min-height: 0;
  overflow: auto;
  border-radius: 8px;
  border: 1px solid var(--yj-table-wrap-border);
  background: var(--yj-table-wrap-bg);
}

.grid {
  width: 100%;
  table-layout: auto;
  border-collapse: collapse;
  font-variant-numeric: tabular-nums;
  font-size: 0.88em;
}

.row-click {
  cursor: pointer;
}

.row-click:hover {
  background: var(--yj-row-hover-bg, rgba(128, 128, 128, 0.08));
}

.grid th,
.grid td {
  padding: 4px 8px;
  text-align: left;
  white-space: nowrap;
  border-bottom: 1px solid var(--yj-row-border);
}

.grid th {
  position: sticky;
  top: 0;
  z-index: 1;
  background: var(--yj-th-bg);
  color: var(--yj-th-color);
  font-weight: 600;
  font-size: 0.8em;
}

.grid th.num {
  text-align: right;
  font-family: "DM Sans", "Noto Sans SC", sans-serif;
  width: 1%;
  padding-left: 12px;
}

.grid td.num {
  text-align: right;
  font-family: "DM Sans", "Noto Sans SC", sans-serif;
  width: 1%;
  padding-left: 12px;
}

/* 名称列：固定窄宽 + ellipsis，其余列吃满剩余（小窗尤其明显） */
.grid th.col-name,
.grid td.col-name {
  width: 1%;
  max-width: 6em;
  min-width: 0;
  box-sizing: border-box;
  padding-left: 8px;
  padding-right: 2px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.grid th.filler,
.grid td.filler {
  width: auto;
  padding: 0;
  pointer-events: none;
}

/* 名称轨已锁窄：表格铺满轨宽，单列 fixed，过长省略（勿再用 max-content 把表缩在宽容器里） */
.panel-main--detail-open .table-wrap--detail-rail .grid {
  table-layout: fixed !important;
  width: 100% !important;
}

.panel-main--detail-open .table-wrap--detail-rail .grid th.col-name,
.panel-main--detail-open .table-wrap--detail-rail .grid td.col-name {
  width: 100% !important;
  max-width: 100% !important;
  min-width: 0;
  box-sizing: border-box;
  padding-left: 4px;
  padding-right: 3px;
  overflow: hidden;
  text-overflow: ellipsis;
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
  color: var(--yj-flat);
}

.toolbar {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 0px 8px 2px;
  border-top: 1px solid var(--yj-toolbar-border);
}

.toolbar__tools {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 4px;
}

/* 与标题栏 .win-actions .icon-btn（最小化等）同尺寸 */
.tool {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  min-width: 20px;
  width: 20px;
  height: 18px;
  padding: 0;
  border-radius: 5px;
  border: 1px solid var(--yj-tool-border);
  background: var(--yj-tool-bg);
  color: var(--yj-tool-color);
  cursor: pointer;
  font-size: 0.68em;
  line-height: 1;
}

.tool-add-inner {
  display: block;
  font-size: 0.78em;
  font-weight: 700;
  line-height: 1;
  transform: translateY(-0.5px);
}

.tool:hover {
  filter: brightness(1.08);
}

.ctx-menu {
  position: fixed;
  z-index: 2147483000;
  min-width: 160px;
  padding: 6px 0;
  border-radius: 10px;
  /* 不透明底 + 明确 fallback，避免脱离主题根时黑字/透底看不清 */
  border: 1px solid var(--yj-table-wrap-border, rgba(255, 255, 255, 0.14));
  background-color: var(--yj-ctx-menu-bg, #2a2a2e);
  background-image: none;
  box-shadow: 0 12px 36px rgba(0, 0, 0, 0.55);
  pointer-events: auto;
  -webkit-app-region: no-drag;
  app-region: no-drag;
}

.ctx-title {
  padding: 6px 12px 8px;
  font-size: 0.78em;
  font-weight: 600;
  color: var(--yj-text-muted, #a3a3a3);
  border-bottom: 1px solid var(--yj-row-border, rgba(255, 255, 255, 0.08));
  margin-bottom: 4px;
  max-width: 240px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.ctx-item {
  display: block;
  width: 100%;
  box-sizing: border-box;
  padding: 8px 12px;
  margin: 0;
  border: none;
  background: transparent;
  color: var(--yj-text, #f5f5f5);
  font-size: 0.88em;
  font-family: inherit;
  text-align: left;
  cursor: pointer;
  line-height: 1.35;
  pointer-events: auto;
  -webkit-app-region: no-drag;
  app-region: no-drag;
}

.ctx-item:hover:not(:disabled) {
  background: var(--yj-row-hover-bg, rgba(255, 255, 255, 0.1));
}

.ctx-item:disabled {
  opacity: 0.38;
  cursor: not-allowed;
}

.ctx-item.danger {
  color: var(--yj-err, #fca5a5);
  margin-top: 4px;
  border-top: 1px solid var(--yj-row-border, rgba(255, 255, 255, 0.08));
  padding-top: 12px;
}

/* 浅色主题：挂在 #yj-root 下随 data-theme 继承变量；再补一层兜底 */
.yj-root[data-theme="light"] .ctx-item {
  color: var(--yj-text, #171717);
}

.yj-root[data-theme="light"] .ctx-title {
  color: var(--yj-text-muted, #737373);
}

.yj-root[data-theme="light"] .ctx-item.danger {
  color: var(--yj-err, #dc2626);
}
</style>
