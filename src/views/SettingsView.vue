<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import { useSettings } from "../composables/useSettings";
import { COLUMN_DEFS } from "../types/app";

const router = useRouter();
const { settings, load, save, applyWindowPrefs } = useSettings();
const bossDraft = ref("");
const saveMsg = ref("");
let ready = false;

onMounted(async () => {
  await load();
  bossDraft.value = settings.value?.bossShortcut ?? "Ctrl+Shift+H";
  ready = true;
});

watch(
  () => settings.value?.bossShortcut,
  (v) => {
    if (v) bossDraft.value = v;
  },
);

watch(
  settings,
  async () => {
    if (!ready || !settings.value) return;
    await applyWindowPrefs();
  },
  { deep: true },
);

function toggleColumn(id: string) {
  if (!settings.value) return;
  const s = settings.value;
  const set = new Set(s.visibleColumns);
  if (set.has(id)) set.delete(id);
  else set.add(id);
  s.visibleColumns = Array.from(set);
}

async function persist() {
  if (!settings.value) return;
  await save();
  saveMsg.value = "已保存";
  setTimeout(() => (saveMsg.value = ""), 1600);
}

async function applyBossKey() {
  if (!settings.value) return;
  settings.value.bossShortcut = bossDraft.value.trim() || "Ctrl+Shift+H";
  await invoke("update_boss_shortcut", { shortcut: settings.value.bossShortcut });
  await save();
  saveMsg.value = "老板键已更新";
  setTimeout(() => (saveMsg.value = ""), 1600);
}

watch(
  () => settings.value?.theme,
  async (to, from) => {
    if (!ready || from === undefined || to === from) return;
    await save();
    saveMsg.value = "主题已保存";
    setTimeout(() => (saveMsg.value = ""), 1200);
  },
);

watch(
  () => settings.value?.quoteSource,
  async (to, from) => {
    if (!ready || from === undefined || to === from) return;
    await save();
    saveMsg.value = "行情源已保存";
    setTimeout(() => (saveMsg.value = ""), 1200);
  },
);
</script>

<template>
  <div class="settings">
    <header class="bar" data-tauri-drag-region>
      <button type="button" class="back" @click="router.push('/')">返回看盘</button>
      <h1 data-tauri-drag-region>设置</h1>
      <span class="msg">{{ saveMsg }}</span>
    </header>

    <div class="layout">
      <aside class="side" data-tauri-drag-region>
        <div class="nav active">通用</div>
        <div class="nav disabled">账户</div>
        <div class="nav disabled">快捷键</div>
        <div class="nav disabled">代理</div>
        <div class="nav disabled">关于</div>
      </aside>

      <main class="content" v-if="settings">
        <section>
          <h2>界面主题</h2>
          <p class="sub">黑白灰配色，不含紫色；涨跌颜色仍由下方「涨跌幅颜色」控制。</p>
          <div class="row">
            <label class="radio">
              <input v-model="settings.theme" type="radio" value="light" />
              浅色
            </label>
            <label class="radio">
              <input v-model="settings.theme" type="radio" value="dark" />
              深色
            </label>
          </div>
        </section>

        <section>
          <h2>行情数据源</h2>
          <p class="sub">
            默认使用东方财富公开接口；自选列表买一卖一在东财「列表」字段上与部分 App 不一致时，可改用腾讯行情。分时、K
            线仍走东财。接口可能变更或限频，仅供个人盯盘，不构成投资建议。断网或调试时可切本地 Mock。
          </p>
          <div class="row">
            <label class="radio">
              <input v-model="settings.quoteSource" type="radio" value="eastmoney" />
              东财公开接口（推荐）
            </label>
            <label class="radio">
              <input v-model="settings.quoteSource" type="radio" value="tencent" />
              腾讯行情（列表买一卖一/五档更准）
            </label>
            <label class="radio">
              <input v-model="settings.quoteSource" type="radio" value="mock" />
              本地 Mock
            </label>
          </div>
        </section>

        <section>
          <h2>首页字体</h2>
          <select v-model.number="settings.fontSizePx" class="yj-field-control font-size-select">
            <option :value="12">12 号</option>
            <option :value="13">13 号</option>
            <option :value="14">14 号</option>
            <option :value="15">15 号</option>
            <option :value="16">16 号</option>
          </select>
        </section>

        <section>
          <h2>看盘列表</h2>
          <p class="sub">选择主窗口中显示的列</p>
          <div class="chips">
            <button
              v-for="c in COLUMN_DEFS"
              :key="c.id"
              type="button"
              class="chip"
              :class="{ on: settings.visibleColumns.includes(c.id) }"
              @click="toggleColumn(c.id)"
            >
              {{ c.label }}
            </button>
          </div>
        </section>

        <section>
          <h2>面板控制</h2>
          <div class="row">
            <label class="radio">
              <input v-model="settings.panelMode" type="radio" value="normal" />
              普通模式
            </label>
            <label class="radio">
              <input v-model="settings.panelMode" type="radio" value="minimal" />
              极简模式
            </label>
          </div>
          <label class="switch-line">
            <input v-model="settings.autoHideEdge" type="checkbox" />
            面板在边缘时自动隐藏
          </label>
        </section>

        <section>
          <h2>窗口与隐私</h2>
          <label class="switch-line">
            <input v-model="settings.alwaysOnTop" type="checkbox" />
            置顶显示
          </label>
          <label class="switch-line">
            <input v-model="settings.skipTaskbar" type="checkbox" />
            不在任务栏显示（更隐蔽，请用托盘找回）
          </label>
          <div class="row slider-row">
            <span>透明度</span>
            <input v-model.number="settings.opacity" type="range" min="0.35" max="1" step="0.01" />
            <span class="mono">{{ settings.opacity.toFixed(2) }}</span>
          </div>
        </section>

        <section>
          <h2>涨跌幅颜色</h2>
          <div class="row">
            <label class="radio">
              <input v-model="settings.colorScheme" type="radio" value="redUp" />
              红涨绿跌
            </label>
            <label class="radio">
              <input v-model="settings.colorScheme" type="radio" value="greenUp" />
              绿涨红跌
            </label>
          </div>
        </section>

        <section>
          <h2>总盈亏显示</h2>
          <div class="row">
            <label class="radio">
              <input v-model="settings.profitDisplay" type="radio" value="both" />
              金额和百分比
            </label>
            <label class="radio">
              <input v-model="settings.profitDisplay" type="radio" value="pct" />
              百分比
            </label>
            <label class="radio">
              <input v-model="settings.profitDisplay" type="radio" value="amount" />
              金额
            </label>
          </div>
          <p class="sub">（当前 Mock 行情下列为占位，接入真实数据后生效）</p>
        </section>

        <section>
          <h2>老板键</h2>
          <p class="sub">默认 Ctrl+Shift+H：显示/隐藏主窗口。修改后请点击应用。</p>
          <div class="row boss">
            <input v-model="bossDraft" class="yj-field-control boss-input" placeholder="Ctrl+Shift+H" />
            <button type="button" class="btn" @click="applyBossKey">应用老板键</button>
          </div>
        </section>

        <section>
          <h2>网络代理</h2>
          <p class="sub">如果无法获取行情数据（网络受限），可以配置代理服务器。留空则直连。</p>
          <div class="row proxy">
            <input
              v-model="settings.proxyUrl"
              class="yj-field-control proxy-input"
              placeholder="http://127.0.0.1:7890"
            />
          </div>
          <p class="sub">例如：Clash Verge 常用 http://127.0.0.1:7890，V2Ray 常用 http://127.0.0.1:10809</p>
        </section>

        <footer class="actions">
          <button type="button" class="btn primary" @click="persist">保存设置</button>
        </footer>
      </main>
    </div>
  </div>
</template>

<style scoped>
.settings {
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
  -webkit-app-region: drag;
  app-region: drag;
}

.bar .back {
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

.msg {
  font-size: 0.78em;
  color: var(--yj-msg);
  min-width: 5em;
  text-align: right;
}

.layout {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: 160px 1fr;
}

.side {
  border-right: 1px solid var(--yj-side-border);
  padding: 12px 10px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  -webkit-app-region: drag;
  app-region: drag;
}

.nav {
  padding: 8px 10px;
  border-radius: 8px;
  font-size: 0.85em;
  color: var(--yj-nav-color);
}

.nav.active {
  background: var(--yj-nav-active-bg);
  color: var(--yj-nav-active-color);
}

.nav.disabled {
  opacity: 0.35;
}

.content {
  padding: 16px 20px 24px;
  overflow: auto;
  -webkit-app-region: no-drag;
  app-region: no-drag;
}

section {
  margin-bottom: 22px;
}

section h2 {
  margin: 0 0 8px;
  font-size: 0.8em;
  font-weight: 600;
}

.sub {
  margin: 0 0 10px;
  font-size: 0.78em;
  color: var(--yj-sub-color);
}

.chips {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.chip {
  border-radius: 999px;
  border: 1px solid var(--yj-chip-border);
  background: var(--yj-chip-bg);
  color: var(--yj-chip-color);
  padding: 6px 12px;
  font-size: 0.8em;
  cursor: pointer;
}

.chip.on {
  background: var(--yj-chip-on-bg);
  border-color: var(--yj-chip-on-border);
  color: var(--yj-chip-on-color);
}

.row {
  display: flex;
  flex-wrap: wrap;
  gap: 14px 18px;
  align-items: center;
}

.radio {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 0.88em;
}

.switch-line {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 10px;
  font-size: 0.88em;
}

.font-size-select {
  min-width: 140px;
}

.boss-input {
  flex: 1;
  min-width: 200px;
  font-family: "DM Sans", ui-monospace, monospace;
  font-variant-numeric: tabular-nums;
}

.slider-row {
  margin-top: 12px;
}

.mono {
  font-family: "DM Sans", ui-monospace, monospace;
  font-size: 0.82em;
  font-variant-numeric: tabular-nums;
  opacity: 0.92;
  min-width: 3em;
}

.boss {
  margin-top: 8px;
}

.proxy-input {
  flex: 1;
  min-width: 200px;
  font-family: "DM Sans", ui-monospace, monospace;
  font-variant-numeric: tabular-nums;
}

.actions {
  margin-top: 8px;
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
</style>
