<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useSettings } from "../composables/useSettings";
import { useQuotes } from "../composables/useQuotes";
import { useEdgeHide } from "../composables/useEdgeHide";
import { COLUMN_DEFS, type QuoteRow } from "../types/app";
import { changeClass, fmtFixed, fmtVolume } from "../utils/format";

const router = useRouter();
const { settings, loading } = useSettings();
const { rows, err, refresh, start } = useQuotes(6000);
const tab = ref<"all" | "watch">("all");

useEdgeHide(settings);

onMounted(() => start());

const visibleCols = computed(() => {
  const s = settings.value;
  if (!s) return COLUMN_DEFS.filter((c) => ["name", "changePct", "price"].includes(c.id));
  const set = new Set(s.visibleColumns);
  return COLUMN_DEFS.filter((c) => set.has(c.id));
});

const displayRows = computed(() => {
  const s = settings.value;
  if (!s) return rows.value;
  if (tab.value === "watch") {
    const codes = new Set(s.watchlist.map((w) => w.code));
    return rows.value.filter((r) => codes.has(r.code));
  }
  return rows.value;
});

function cell(row: QuoteRow, colId: string): string {
  switch (colId) {
    case "name":
      return row.name;
    case "changePct":
      return `${fmtFixed(row.changePct, 2)}%`;
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
  return null;
}

async function hideWin() {
  await getCurrentWindow().hide();
}

const minimal = computed(() => settings.value?.panelMode === "minimal");
</script>

<template>
  <div class="panel" :class="{ minimal }">
    <header class="titlebar" data-tauri-drag-region>
      <div class="brand" data-tauri-drag-region>
        <span class="dot" />
        <span class="name">元精灵</span>
      </div>
      <div class="tabs" data-tauri-drag-region>
        <button type="button" class="tab" :class="{ on: tab === 'all' }" @click="tab = 'all'">
          全部
        </button>
        <button
          type="button"
          class="tab"
          :class="{ on: tab === 'watch' }"
          @click="tab = 'watch'"
        >
          自选股
        </button>
        <span class="tab ghost">基金</span>
        <span class="tab ghost">美股</span>
      </div>
      <div class="win-actions">
        <button type="button" class="icon-btn" title="隐藏到托盘" @click="hideWin">—</button>
      </div>
    </header>

    <div v-if="loading" class="hint">加载配置…</div>
    <div v-else class="body">
      <div v-if="err" class="err">{{ err }}</div>
      <div class="table-wrap">
        <table class="grid">
          <thead>
            <tr>
              <th v-for="c in visibleCols" :key="c.id">{{ c.label }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="r in displayRows" :key="r.code">
              <td
                v-for="c in visibleCols"
                :key="c.id"
                :class="{
                  num: c.id !== 'name',
                  chg:
                    c.id === 'changePct' ||
                    c.id === 'dailyPl' ||
                    c.id === 'totalPl',
                }"
              >
                <span
                  v-if="pctForRow(r, c.id) !== null"
                  :class="changeClass(pctForRow(r, c.id)!, settings?.colorScheme ?? 'redUp')"
                >
                  {{ cell(r, c.id) }}
                </span>
                <span v-else>{{ cell(r, c.id) }}</span>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <footer v-if="!minimal" class="toolbar">
      <button type="button" class="tool" title="刷新" @click="refresh">↻</button>
      <button type="button" class="tool primary" title="设置" @click="router.push('/settings')">
        ⚙
      </button>
      <span class="tool muted" title="占位">＋</span>
    </footer>
  </div>
</template>

<style scoped>
.panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}

.panel.minimal .body {
  padding-bottom: 4px;
}

.titlebar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px 6px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  user-select: none;
}

.brand {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  letter-spacing: 0.04em;
}

.dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: linear-gradient(135deg, #c084fc, #6366f1);
  box-shadow: 0 0 12px rgba(129, 140, 248, 0.7);
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
  overflow: hidden;
}

.tab {
  border: none;
  background: rgba(255, 255, 255, 0.04);
  color: rgba(232, 230, 242, 0.72);
  padding: 4px 10px;
  border-radius: 999px;
  font-size: 0.78em;
  cursor: pointer;
  transition: background 0.18s ease, color 0.18s ease;
}

.tab.on {
  background: linear-gradient(120deg, rgba(99, 102, 241, 0.55), rgba(168, 85, 247, 0.45));
  color: #fff;
}

.tab.ghost {
  cursor: default;
  opacity: 0.35;
}

.win-actions {
  display: flex;
  align-items: center;
}

.icon-btn {
  width: 28px;
  height: 24px;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(0, 0, 0, 0.2);
  color: rgba(255, 255, 255, 0.75);
  cursor: pointer;
  line-height: 1;
}

.icon-btn:hover {
  background: rgba(255, 255, 255, 0.08);
}

.hint,
.err {
  padding: 12px 14px;
  font-size: 0.85em;
}

.err {
  color: #fda4af;
}

.body {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  padding: 0 6px 4px;
}

.table-wrap {
  flex: 1;
  min-height: 0;
  overflow: auto;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.05);
  background: rgba(0, 0, 0, 0.12);
}

.grid {
  width: 100%;
  border-collapse: collapse;
  font-variant-numeric: tabular-nums;
  font-size: 0.88em;
}

.grid th,
.grid td {
  padding: 6px 8px;
  text-align: left;
  white-space: nowrap;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
}

.grid th {
  position: sticky;
  top: 0;
  z-index: 1;
  background: rgba(12, 10, 28, 0.92);
  color: rgba(220, 218, 235, 0.65);
  font-weight: 600;
  font-size: 0.8em;
}

.grid td.num {
  text-align: right;
  font-family: "DM Sans", "Noto Sans SC", sans-serif;
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
  color: rgba(232, 230, 242, 0.55);
}

.toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px 8px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
}

.tool {
  min-width: 32px;
  height: 30px;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.05);
  color: #c4b5fd;
  cursor: pointer;
  font-size: 0.95em;
}

.tool.primary {
  background: linear-gradient(120deg, rgba(99, 102, 241, 0.45), rgba(168, 85, 247, 0.35));
  border-color: rgba(129, 140, 248, 0.35);
  color: #fff;
}

.tool.muted {
  opacity: 0.35;
  cursor: default;
}

.tool:hover:not(.muted) {
  filter: brightness(1.08);
}
</style>
