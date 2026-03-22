<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import MarketMovesPanel from "../components/MarketMovesPanel.vue";
import { useSettings } from "../composables/useSettings";

const router = useRouter();
const { settings, load } = useSettings();
const panelRef = ref<InstanceType<typeof MarketMovesPanel> | null>(null);

onMounted(async () => {
  await load();
});

function onSelectStock(code: string) {
  router.push({ name: "stock", params: { code: code.trim().toLowerCase() } });
}

function onRefresh() {
  panelRef.value?.reload();
}
</script>

<template>
  <div class="moves-page moves-page--dense">
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
      <h1 data-tauri-drag-region>股市异动</h1>
      <button
        type="button"
        class="bar__action"
        title="刷新"
        aria-label="刷新异动"
        @click="onRefresh"
      >
        ↻
      </button>
    </header>

    <div v-if="settings" class="main main--fill">
      <MarketMovesPanel
        ref="panelRef"
        layout="page"
        :quote-source="settings.quoteSource ?? 'eastmoney'"
        :color-scheme="settings.colorScheme ?? 'redUp'"
        @select-stock="onSelectStock"
      />
    </div>
    <div v-else class="hint">加载中…</div>
  </div>
</template>

<style scoped>
.moves-page {
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

/* 与主窗口 .win-actions .icon-btn 同量级，适配小窗 */
.moves-page--dense .bar--dense {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 6px 4px;
  border-bottom: 1px solid var(--yj-bar-border);
  flex-shrink: 0;
  -webkit-app-region: drag;
  app-region: drag;
}

.bar--dense .back,
.bar--dense .bar__action {
  flex-shrink: 0;
  -webkit-app-region: no-drag;
  app-region: no-drag;
}

.bar--dense h1 {
  flex: 1;
  margin: 0;
  /* 与主窗口 Tab 字号一致（约 0.78em） */
  font-size: 0.78em;
  font-weight: 600;
  letter-spacing: 0.02em;
  text-align: center;
  min-width: 0;
}

.back {
  box-sizing: border-box;
  width: 20px;
  height: 18px;
  min-width: 20px;
  padding: 0;
  border-radius: 5px;
  border: 1px solid var(--yj-back-border);
  background: var(--yj-back-bg);
  color: var(--yj-back-color);
  cursor: pointer;
  font-size: 0.85em;
  font-weight: 600;
  line-height: 1;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.bar__action {
  box-sizing: border-box;
  width: 20px;
  height: 18px;
  min-width: 20px;
  padding: 0;
  border-radius: 5px;
  border: 1px solid var(--yj-icon-border);
  background: var(--yj-icon-bg);
  color: var(--yj-icon-color);
  cursor: pointer;
  font-size: 0.68em;
  line-height: 1;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.bar__action:hover {
  background: var(--yj-icon-hover-bg);
}

.main {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  -webkit-app-region: no-drag;
  app-region: no-drag;
}

.main--fill {
  padding: 0;
}

.hint {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.88em;
  color: var(--yj-text-muted);
  -webkit-app-region: no-drag;
  app-region: no-drag;
}
</style>
