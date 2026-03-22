<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useSettings } from "../composables/useSettings";
import StockDetailBody from "../components/StockDetailBody.vue";

const route = useRoute();
const router = useRouter();
const { load } = useSettings();

const codeParam = computed(() => String(route.params.code ?? "").trim().toLowerCase());
const headerTitle = ref("");

const pageHeading = computed(() => headerTitle.value || codeParam.value || "个股");

function onDetailTitle(t: string) {
  headerTitle.value = t;
}

onMounted(async () => {
  await load();
});
</script>

<template>
  <div class="detail-page">
    <header class="bar" data-tauri-drag-region>
      <button type="button" class="back" @click="router.push('/')">看盘</button>
      <h1 data-tauri-drag-region>{{ pageHeading }}</h1>
    </header>

    <StockDetailBody
      v-if="codeParam"
      class="detail-page__body"
      :code="codeParam"
      @update:title="onDetailTitle"
    />
    <p v-else class="hint">无效代码</p>
  </div>
</template>

<style scoped>
.detail-page {
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

.bar .back {
  -webkit-app-region: no-drag;
  app-region: no-drag;
}

.bar h1 {
  flex: 0 1 auto;
  max-width: min(16em, calc(100% - 5.5rem));
  margin: 0;
  font-size: 0.86em;
  font-weight: 600;
  letter-spacing: 0.02em;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.back {
  border-radius: 8px;
  padding: 6px 12px;
  cursor: pointer;
  font-size: 0.78em;
  border: 1px solid var(--yj-back-border);
  background: var(--yj-back-bg);
  color: var(--yj-back-color);
}

.detail-page__body {
  flex: 1;
  min-height: 0;
}

.hint {
  padding: 24px;
  color: var(--yj-text-muted);
  font-size: 0.8em;
}
</style>
