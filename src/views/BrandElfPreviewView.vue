<script setup lang="ts">
import { ref } from "vue";
import BrandElfMascot from "../components/BrandElfMascot.vue";
import type { BrandMoodBucket } from "../utils/brandKaomoji";

const theme = ref<"light" | "dark">("dark");

type Row = { bucket: BrandMoodBucket; title: string; hint: string };

/** 与 getBrandMood 分桶一致的自选均涨跌幅示意 */
const rows: Row[] = [
  { bucket: "empty", title: "暂无自选", hint: "未添加自选股" },
  { bucket: "waiting", title: "等待行情", hint: "有自选但暂无行情数据" },
  { bucket: "microUp", title: "微涨", hint: "自选均涨 ≥ +0.1%" },
  { bucket: "mildUp", title: "小涨", hint: "自选均涨 ≥ +0.6%" },
  { bucket: "strongUp", title: "强涨", hint: "自选均涨 ≥ +2.5%" },
  {
    bucket: "strongUpAll",
    title: "强涨（全红）",
    hint: "强涨桶 + 至少 2 只且当日全涨",
  },
  { bucket: "flat", title: "横盘", hint: "自选均涨在 (-0.1%, +0.1%)" },
  { bucket: "microDown", title: "微跌", hint: "自选均涨 ≤ -0.1%" },
  { bucket: "mildDown", title: "小跌", hint: "自选均涨 ≤ -0.6%" },
  { bucket: "strongDown", title: "大跌", hint: "自选均涨 ≤ -2.5%" },
  {
    bucket: "strongDownAll",
    title: "大跌（全绿）",
    hint: "大跌桶 + 至少 2 只且当日全跌",
  },
];
</script>

<template>
  <div class="preview" :data-theme="theme">
    <header class="bar">
      <h1>元精灵 · 情绪预览</h1>
      <p class="sub">
        实际界面由「自选等权平均涨跌幅」分桶；此处逐格展示动画与配色。打开地址：
        <code>#/elf-preview</code>
      </p>
      <div class="tools">
        <span>主题</span>
        <button type="button" :class="{ on: theme === 'light' }" @click="theme = 'light'">
          浅色
        </button>
        <button type="button" :class="{ on: theme === 'dark' }" @click="theme = 'dark'">
          深色
        </button>
        <RouterLink class="back" to="/">返回主面板</RouterLink>
      </div>
    </header>
    <div class="grid">
      <section v-for="(r, i) in rows" :key="r.bucket" class="card">
        <div class="elf-wrap">
          <BrandElfMascot
            :bucket="r.bucket"
            :variant="i"
            :theme="theme"
            :ariaLabel="`${r.title}：${r.hint}`"
          />
        </div>
        <div class="meta">
          <strong>{{ r.title }}</strong>
          <span class="bucket">{{ r.bucket }}</span>
          <p>{{ r.hint }}</p>
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.preview {
  min-height: 100%;
  padding: 20px 16px 40px;
  font-family: "Noto Sans SC", system-ui, sans-serif;
}

.preview[data-theme="dark"] {
  background: #0d0d0d;
  color: #f5f5f5;
}

.preview[data-theme="light"] {
  background: #ececec;
  color: #171717;
}

.bar {
  max-width: 960px;
  margin: 0 auto 24px;
}

h1 {
  margin: 0 0 8px;
  font-size: 0.95em;
  font-weight: 700;
}

.sub {
  margin: 0 0 16px;
  font-size: 0.88em;
  opacity: 0.85;
  line-height: 1.5;
}

code {
  font-size: 0.85em;
  padding: 2px 6px;
  border-radius: 4px;
}

.preview[data-theme="dark"] code {
  background: rgba(255, 255, 255, 0.08);
}

.preview[data-theme="light"] code {
  background: rgba(0, 0, 0, 0.06);
}

.tools {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}

.tools span {
  font-size: 0.82em;
  opacity: 0.8;
}

.tools button {
  padding: 6px 12px;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.15);
  background: rgba(255, 255, 255, 0.06);
  color: inherit;
  cursor: pointer;
  font-size: 0.82em;
}

.preview[data-theme="light"] .tools button {
  border-color: rgba(0, 0, 0, 0.12);
  background: rgba(0, 0, 0, 0.04);
}

.tools button.on {
  border-color: #fbbf24;
  background: rgba(251, 191, 36, 0.15);
}

.back {
  margin-left: auto;
  font-size: 0.88em;
  color: #38bdf8;
  text-decoration: none;
}

.back:hover {
  text-decoration: underline;
}

.grid {
  max-width: 960px;
  margin: 0 auto;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: 14px;
}

.card {
  display: flex;
  align-items: flex-start;
  gap: 14px;
  padding: 14px 16px;
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.04);
}

.preview[data-theme="light"] .card {
  border-color: rgba(0, 0, 0, 0.08);
  background: rgba(255, 255, 255, 0.7);
}

.elf-wrap {
  flex-shrink: 0;
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.meta strong {
  display: block;
  font-size: 0.88em;
  margin-bottom: 4px;
}

.bucket {
  font-size: 0.72em;
  font-family: ui-monospace, monospace;
  opacity: 0.55;
}

.meta p {
  margin: 8px 0 0;
  font-size: 0.8em;
  line-height: 1.45;
  opacity: 0.88;
}

/* 预览区略放大，便于观察动效 */
.elf-wrap :deep(.elf-svg) {
  width: 48px;
  height: 48px;
}
</style>
