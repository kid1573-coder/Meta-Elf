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
        <div class="nav disabled">分组</div>
        <div class="nav disabled">代理</div>
        <div class="nav disabled">关于</div>
      </aside>

      <main class="content" v-if="settings">
        <section>
          <h2>首页字体</h2>
          <select v-model.number="settings.fontSizePx" class="field">
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
            <input v-model="bossDraft" class="field text" placeholder="Ctrl+Shift+H" />
            <button type="button" class="btn" @click="applyBossKey">应用老板键</button>
          </div>
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
  background: linear-gradient(180deg, rgba(14, 12, 30, 0.96), rgba(18, 16, 36, 0.98));
  color: #e8e6f2;
}

.bar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 14px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.bar h1 {
  flex: 1;
  margin: 0;
  font-size: 1.05rem;
  font-weight: 600;
}

.back {
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  background: rgba(255, 255, 255, 0.06);
  color: #ddd6fe;
  padding: 6px 12px;
  cursor: pointer;
}

.msg {
  font-size: 0.82rem;
  color: #86efac;
  min-width: 5rem;
  text-align: right;
}

.layout {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: 160px 1fr;
}

.side {
  border-right: 1px solid rgba(255, 255, 255, 0.06);
  padding: 12px 10px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.nav {
  padding: 8px 10px;
  border-radius: 8px;
  font-size: 0.88rem;
  color: rgba(232, 230, 242, 0.55);
}

.nav.active {
  background: rgba(99, 102, 241, 0.25);
  color: #fff;
}

.nav.disabled {
  opacity: 0.35;
}

.content {
  padding: 16px 20px 24px;
  overflow: auto;
}

section {
  margin-bottom: 22px;
}

section h2 {
  margin: 0 0 8px;
  font-size: 0.95rem;
  font-weight: 600;
}

.sub {
  margin: 0 0 10px;
  font-size: 0.8rem;
  color: rgba(232, 230, 242, 0.55);
}

.chips {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.chip {
  border-radius: 999px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  background: rgba(255, 255, 255, 0.04);
  color: rgba(232, 230, 242, 0.85);
  padding: 6px 12px;
  font-size: 0.82rem;
  cursor: pointer;
}

.chip.on {
  background: linear-gradient(120deg, rgba(99, 102, 241, 0.55), rgba(168, 85, 247, 0.4));
  border-color: rgba(129, 140, 248, 0.45);
  color: #fff;
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
  font-size: 0.88rem;
}

.switch-line {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 10px;
  font-size: 0.88rem;
}

.field {
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  background: rgba(0, 0, 0, 0.25);
  color: #f5f3ff;
  padding: 8px 10px;
  min-width: 160px;
}

.field.text {
  flex: 1;
  min-width: 200px;
  font-family: ui-monospace, monospace;
}

.slider-row {
  margin-top: 12px;
}

.mono {
  font-family: "DM Sans", ui-monospace, monospace;
  font-size: 0.85rem;
  opacity: 0.85;
  min-width: 3rem;
}

.boss {
  margin-top: 8px;
}

.actions {
  margin-top: 8px;
}

.btn {
  border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.14);
  background: rgba(255, 255, 255, 0.06);
  color: #e9d5ff;
  padding: 8px 14px;
  cursor: pointer;
  font-size: 0.88rem;
}

.btn.primary {
  background: linear-gradient(120deg, rgba(99, 102, 241, 0.65), rgba(168, 85, 247, 0.5));
  border-color: rgba(129, 140, 248, 0.5);
  color: #fff;
}
</style>
