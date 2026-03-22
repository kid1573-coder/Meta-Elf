<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import YjSelect from "../components/YjSelect.vue";
import { useSettings } from "../composables/useSettings";
import type { WatchItem } from "../types/app";
import { pruneWatchlistIfOrphaned } from "../utils/watchGroups";

const router = useRouter();
const route = useRoute();
const { settings, load, save } = useSettings();

const searchQuery = ref("");
const newCode = ref("");
const newName = ref("");
const toast = ref("");
const selectedGroupId = ref("");

watch(
  () => settings.value?.watchGroups,
  (groups) => {
    if (!groups?.length) return;
    const qg = String(route.query.group ?? "").trim();
    if (qg && groups.some((g) => g.id === qg)) {
      selectedGroupId.value = qg;
    } else if (!selectedGroupId.value || !groups.some((g) => g.id === selectedGroupId.value)) {
      selectedGroupId.value = groups[0].id;
    }
  },
  { immediate: true },
);

function showToast(msg: string, ms = 1600) {
  toast.value = msg;
  setTimeout(() => {
    toast.value = "";
  }, ms);
}

onMounted(async () => {
  await load();
});

const currentGroup = computed(() => {
  const s = settings.value;
  if (!s?.watchGroups.length) return null;
  return s.watchGroups.find((g) => g.id === selectedGroupId.value) ?? s.watchGroups[0];
});

const groupSelectOptions = computed(() =>
  settings.value?.watchGroups.map((g) => ({ value: g.id, label: g.name })) ?? [],
);

const groupEntries = computed(() => {
  const s = settings.value;
  const g = currentGroup.value;
  if (!s || !g) return [] as { w: WatchItem; codeIndex: number }[];
  const out: { w: WatchItem; codeIndex: number }[] = [];
  g.codes.forEach((code, codeIndex) => {
    const w = s.watchlist.find((x) => x.code.toLowerCase() === code.toLowerCase());
    if (w) out.push({ w, codeIndex });
  });
  return out;
});

const filteredEntries = computed(() => {
  const q = searchQuery.value.trim().toLowerCase();
  return groupEntries.value.filter(
    ({ w }) =>
      !q || w.code.toLowerCase().includes(q) || w.name.toLowerCase().includes(q),
  );
});

async function persistList() {
  if (!settings.value) return;
  await save();
  showToast("已保存到本地配置");
}

async function addManual() {
  if (!settings.value || !currentGroup.value) return;
  const code = newCode.value.trim().toLowerCase();
  const name = newName.value.trim();
  if (!code) {
    showToast("请填写代码", 1200);
    return;
  }
  const g = currentGroup.value;
  if (g.codes.some((x) => x.toLowerCase() === code)) {
    showToast("本分组已有该代码", 1500);
    return;
  }
  if (!settings.value.watchlist.some((x) => x.code.toLowerCase() === code)) {
    settings.value.watchlist.push({ code, name: name || code });
  }
  g.codes.push(code);
  newCode.value = "";
  newName.value = "";
  await save();
  showToast("已添加");
}

async function removeAt(codeIndex: number) {
  if (!settings.value || !currentGroup.value) return;
  const g = currentGroup.value;
  if (codeIndex < 0 || codeIndex >= g.codes.length) return;
  g.codes.splice(codeIndex, 1);
  pruneWatchlistIfOrphaned(settings.value);
  await save();
  showToast("已删除");
}

async function onCellChange(entry: { w: WatchItem; codeIndex: number }) {
  if (!settings.value || !currentGroup.value) return;
  const g = currentGroup.value;
  const normalized = entry.w.code.trim().toLowerCase();
  entry.w.code = normalized;
  if (entry.codeIndex >= 0 && entry.codeIndex < g.codes.length) {
    g.codes[entry.codeIndex] = normalized;
  }
  await save();
}

function openDetail(code: string) {
  router.push({ name: "stock", params: { code } });
}
</script>

<template>
  <div class="watch-page">
    <header class="bar" data-tauri-drag-region>
      <button type="button" class="back" @click="router.push('/')">
        返回看盘
      </button>
      <h1 data-tauri-drag-region>自选股管理</h1>
      <button
        type="button"
        class="link-btn"
        @click="router.push('/settings')"
      >
        通用设置
      </button>
    </header>

    <div v-if="settings" class="main">
      <div class="top-actions">
        <button
          type="button"
          class="btn primary"
          @click="
            router.push({
              path: '/watchlist/add',
              query: selectedGroupId ? { group: selectedGroupId } : {},
            })
          "
        >
          添加自选
        </button>
        <label v-if="currentGroup" class="group-pick">
          <span class="meta">分组</span>
          <YjSelect
            v-model="selectedGroupId"
            class="group-select"
            aria-label="自选分组"
            :options="groupSelectOptions"
          />
        </label>
        <span class="meta"
          >本组 {{ currentGroup?.codes.length ?? 0 }} / 自选共
          {{ settings.watchlist.length }} 只</span
        >
      </div>

      <div class="search-row">
        <input
          v-model="searchQuery"
          type="search"
          class="yj-field-control search-input"
          placeholder="在自选列表中筛选代码或名称…"
          autocomplete="off"
        />
        <span class="meta">
          显示 {{ filteredEntries.length }} / {{ groupEntries.length }} 条
        </span>
      </div>

      <details class="advanced">
        <summary>高级：手动输入代码</summary>
        <p class="sub">
          格式为 <code class="inline-code">sh600519</code>、<code class="inline-code">sz000001</code> 等；一般请优先用「添加自选」页搜索。
        </p>
        <div class="add-row">
          <input
            v-model="newCode"
            class="yj-field-control add-code"
            placeholder="代码 *"
            @keydown.enter.prevent="addManual"
          />
          <input
            v-model="newName"
            class="yj-field-control add-name"
            placeholder="显示名称（可空）"
            @keydown.enter.prevent="addManual"
          />
          <button type="button" class="btn" @click="addManual">添加</button>
        </div>
      </details>

      <section class="card">
        <div class="card-head">
          <h2>分组「{{ currentGroup?.name ?? "—" }}」</h2>
          <button type="button" class="btn" @click="persistList">
            保存修改
          </button>
        </div>
        <p class="sub">
          仅展示当前分组内的标的顺序；改代码会同步分组引用。点击名称可打开个股详情。
        </p>

        <div class="table-wrap">
          <table class="grid">
            <thead>
              <tr>
                <th class="col-code">代码</th>
                <th class="col-name">名称</th>
                <th class="col-act">操作</th>
              </tr>
            </thead>
            <tbody>
              <tr v-if="filteredEntries.length === 0">
                <td colspan="3" class="empty">
                  {{
                    searchQuery.trim()
                      ? "没有符合筛选条件的条目"
                      : "本分组暂无标的，请使用「添加自选」"
                  }}
                </td>
              </tr>
              <tr
                v-for="entry in filteredEntries"
                :key="entry.codeIndex + '-' + entry.w.code"
              >
                <td>
                  <input
                    v-model="entry.w.code"
                    class="field cell-input"
                    @change="onCellChange(entry)"
                  />
                </td>
                <td>
                  <button
                    type="button"
                    class="name-link"
                    @click="openDetail(entry.w.code)"
                  >
                    {{ entry.w.name }}
                  </button>
                  <input
                    v-model="entry.w.name"
                    class="yj-field-control cell-input name-input"
                    @change="onCellChange(entry)"
                  />
                </td>
                <td>
                  <button
                    type="button"
                    class="btn danger"
                    @click="removeAt(entry.codeIndex)"
                  >
                    删除
                  </button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>
    </div>

    <div v-else class="hint">加载中…</div>
    <div v-if="toast" class="toast">{{ toast }}</div>
  </div>
</template>

<style scoped>
.watch-page {
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

.bar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 14px;
  border-bottom: 1px solid var(--yj-bar-border);
  flex-shrink: 0;
  -webkit-app-region: drag;
  app-region: drag;
}

.bar .back,
.bar .link-btn {
  -webkit-app-region: no-drag;
  app-region: no-drag;
}

.bar h1 {
  flex: 1;
  margin: 0;
  font-size: 0.86em;
  font-weight: 600;
  letter-spacing: 0.02em;
}

.back {
  border-radius: 8px;
  border: 1px solid var(--yj-back-border);
  background: var(--yj-back-bg);
  color: var(--yj-back-color);
  padding: 6px 12px;
  cursor: pointer;
  font-size: 0.78em;
}

.link-btn {
  border-radius: 8px;
  border: 1px solid var(--yj-chip-border);
  background: var(--yj-chip-bg);
  color: var(--yj-chip-color);
  padding: 6px 12px;
  cursor: pointer;
  font-size: 0.78em;
}

.main {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 14px 16px 24px;
  -webkit-app-region: no-drag;
  app-region: no-drag;
}

.top-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 12px 16px;
  margin-bottom: 14px;
}

.group-pick {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.group-select {
  min-width: 140px;
}

.search-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px 14px;
  margin-bottom: 14px;
}

.search-input {
  flex: 1;
  min-width: 200px;
  max-width: 420px;
}

.meta {
  font-size: 0.78em;
  color: var(--yj-text-muted);
}

.advanced {
  border: 1px solid var(--yj-table-wrap-border);
  background: var(--yj-table-wrap-bg);
  border-radius: 8px;
  padding: 10px 14px;
  margin-bottom: 14px;
}

.advanced summary {
  cursor: pointer;
  font-size: 0.85em;
  font-weight: 500;
  user-select: none;
}

.card {
  border: 1px solid var(--yj-table-wrap-border);
  background: var(--yj-table-wrap-bg);
  border-radius: 8px;
  padding: 14px 16px;
  margin-bottom: 14px;
}

.card h2 {
  margin: 0 0 8px;
  font-size: 0.8em;
  font-weight: 600;
}

.card-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
  margin-bottom: 6px;
}

.card-head h2 {
  margin: 0;
}

.sub {
  margin: 0 0 12px;
  font-size: 0.78em;
  line-height: 1.45;
  color: var(--yj-sub-color);
}

.inline-code {
  font-family: "DM Sans", "Noto Sans SC", ui-monospace, monospace;
  font-variant-numeric: tabular-nums;
  font-size: 0.8em;
  padding: 1px 5px;
  border-radius: 4px;
  background: var(--yj-chip-bg);
  border: 1px solid var(--yj-chip-border);
  color: var(--yj-chip-color);
}

.add-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
}

.add-code {
  min-width: 120px;
}

.add-name {
  flex: 1;
  min-width: 160px;
}

.btn {
  border-radius: 8px;
  border: 1px solid var(--yj-btn-border);
  background: var(--yj-btn-bg);
  color: var(--yj-btn-color);
  padding: 8px 14px;
  cursor: pointer;
  font-size: 0.82em;
}

.btn.primary {
  background: var(--yj-btn-primary-bg);
  border-color: var(--yj-btn-primary-border);
  color: var(--yj-btn-primary-color);
}

.btn.danger {
  border-color: var(--yj-err);
  color: var(--yj-err);
  background: transparent;
}

.btn.danger:hover {
  background: var(--yj-table-wrap-bg);
}

.table-wrap {
  overflow: auto;
  border-radius: 8px;
  border: 1px solid var(--yj-row-border);
}

.grid {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.88em;
  font-variant-numeric: tabular-nums;
}

.grid th,
.grid td {
  padding: 8px 10px;
  text-align: left;
  border-bottom: 1px solid var(--yj-row-border);
  vertical-align: middle;
}

.grid th {
  font-weight: 600;
  color: var(--yj-th-color);
  background: var(--yj-th-bg);
  font-size: 0.8em;
}

.col-code {
  width: 28%;
  min-width: 120px;
}

.col-name {
  width: auto;
}

.col-act {
  width: 88px;
  text-align: right;
}

.cell-input {
  width: 100%;
  min-width: 0;
  box-sizing: border-box;
}

.cell-input--code {
  font-family: "DM Sans", "Noto Sans SC", ui-monospace, monospace;
  font-variant-numeric: tabular-nums;
}

.name-link {
  display: block;
  margin-bottom: 6px;
  padding: 0;
  border: none;
  background: none;
  color: var(--yj-tab-on-color, #60a5fa);
  cursor: pointer;
  font-size: 0.88em;
  font-weight: 500;
  text-align: left;
  text-decoration: underline;
  text-underline-offset: 2px;
}

.name-input {
  margin-top: 0;
}

.empty {
  text-align: center;
  color: var(--yj-text-muted);
  padding: 2em 12px !important;
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
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
  z-index: 50;
  pointer-events: none;
}
</style>
