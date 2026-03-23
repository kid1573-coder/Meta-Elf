<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useSettings } from "./composables/useSettings";

const { load, applyWindowPrefs, rootStyle, settings } = useSettings();

const dataTheme = computed(() =>
  settings.value?.theme === "light" ? "light" : "dark",
);

onMounted(async () => {
  await load();
  await applyWindowPrefs();
});
</script>

<template>
  <div
    class="yj-shell"
    :data-theme="dataTheme"
    :style="rootStyle"
  >
    <div id="yj-root" class="yj-root" :data-theme="dataTheme">
      <router-view />
    </div>
  </div>
</template>

<style>
@import url("https://fonts.googleapis.com/css2?family=DM+Sans:ital,opsz,wght@0,9..40,400;0,9..40,500;0,9..40,600;0,9..40,700;1,9..40,500&family=Noto+Sans+SC:wght@400;500;600;700&display=swap");

*,
*::before,
*::after {
  box-sizing: border-box;
}

html,
body,
#app,
.yj-shell,
.yj-root {
  margin: 0;
  height: 100%;
  width: 100%;
  overflow: hidden;
}

/* 挂载前兜底；设为透明以支持圆角窗口 */
html,
#app {
  background-color: transparent;
}

/*
 * 设为透明，让 .yj-root 的圆角真正生效，四角透出桌面
 */
.yj-shell[data-theme="dark"] {
  background-color: transparent;
}

.yj-shell[data-theme="light"] {
  background-color: transparent;
}

body {
  font-family: "Noto Sans SC", "DM Sans", system-ui, sans-serif;
  background: transparent;
  -webkit-font-smoothing: antialiased;
}

/* 原生表单控件（含 select 弹出层）随主题使用暗/亮色，减轻 WebView 下灰底菜单跳脱 */
html:has(.yj-shell[data-theme="dark"]) {
  color-scheme: dark;
}

html:has(.yj-shell[data-theme="light"]) {
  color-scheme: light;
}

/* ========== 深色：黑白灰 ========== */
.yj-root[data-theme="dark"] {
  color: #f5f5f5;
  --yj-text: #f5f5f5;
  --yj-text-muted: #a3a3a3;
  --yj-titlebar-border: rgba(255, 255, 255, 0.1);
  --yj-dot: #fafafa;
  --yj-dot-shadow: rgba(255, 255, 255, 0.25);
  --yj-tab-bg: rgba(255, 255, 255, 0.08);
  --yj-tab-color: rgba(245, 245, 245, 0.75);
  --yj-tab-on-bg: #fafafa;
  --yj-tab-on-color: #0a0a0a;
  --yj-icon-border: rgba(255, 255, 255, 0.14);
  --yj-icon-bg: rgba(0, 0, 0, 0.35);
  --yj-icon-color: rgba(245, 245, 245, 0.85);
  --yj-icon-hover-bg: rgba(255, 255, 255, 0.1);
  --yj-err: #fca5a5;
  --yj-table-wrap-border: rgba(255, 255, 255, 0.1);
  --yj-table-wrap-bg: rgba(0, 0, 0, 0.25);
  --yj-ctx-menu-bg: rgba(30, 30, 30, 0.97);
  /* 弹窗/遮罩：勿用 table-wrap 半透明底，否则透明窗口下不可读 */
  --yj-modal-scrim: rgba(0, 0, 0, 0.55);
  --yj-modal-panel-bg: #1e1e1e;
  --yj-modal-panel-border: rgba(255, 255, 255, 0.12);
  --yj-scrollbar-track: rgb(255 255 255 / calc(0.022 * var(--yj-window-opacity, 1)));
  --yj-scrollbar-thumb: rgb(255 255 255 / calc(0.1 * var(--yj-window-opacity, 1)));
  --yj-scrollbar-thumb-hover: rgb(255 255 255 / calc(0.18 * var(--yj-window-opacity, 1)));
  --yj-row-border: rgba(255, 255, 255, 0.06);
  --yj-row-hover-bg: rgba(255, 255, 255, 0.06);
  --yj-th-bg: rgba(20, 20, 20, 0.95);
  --yj-th-color: rgba(245, 245, 245, 0.55);
  --yj-flat: rgba(245, 245, 245, 0.45);
  --yj-toolbar-border: rgba(255, 255, 255, 0.1);
  --yj-tool-border: rgba(255, 255, 255, 0.12);
  --yj-tool-bg: rgba(255, 255, 255, 0.06);
  --yj-tool-color: #e5e5e5;
  --yj-tool-primary-bg: #fafafa;
  --yj-tool-primary-border: #d4d4d4;
  --yj-tool-primary-color: #0a0a0a;
  --yj-settings-bg-1: #0f0f0f;
  --yj-settings-bg-2: #141414;
  --yj-bar-border: rgba(255, 255, 255, 0.1);
  --yj-back-border: rgba(255, 255, 255, 0.14);
  --yj-back-bg: rgba(255, 255, 255, 0.06);
  --yj-back-color: #f5f5f5;
  --yj-msg: #a3a3a3;
  --yj-side-border: rgba(255, 255, 255, 0.08);
  --yj-nav-color: rgba(245, 245, 245, 0.45);
  --yj-nav-active-bg: rgba(255, 255, 255, 0.12);
  --yj-nav-active-color: #fafafa;
  --yj-sub-color: rgba(245, 245, 245, 0.5);
  --yj-chip-border: rgba(255, 255, 255, 0.14);
  --yj-chip-bg: rgba(255, 255, 255, 0.05);
  --yj-chip-color: rgba(245, 245, 245, 0.88);
  --yj-chip-on-bg: #fafafa;
  --yj-chip-on-border: #e5e5e5;
  --yj-chip-on-color: #0a0a0a;
  --yj-field-border: rgba(255, 255, 255, 0.14);
  --yj-field-bg: rgba(0, 0, 0, 0.35);
  --yj-field-color: #fafafa;
  --yj-btn-border: rgba(255, 255, 255, 0.16);
  --yj-btn-bg: rgba(255, 255, 255, 0.06);
  --yj-btn-color: #f5f5f5;
  --yj-btn-primary-bg: #fafafa;
  --yj-btn-primary-border: #e5e5e5;
  --yj-btn-primary-color: #0a0a0a;
  background: linear-gradient(145deg, #121212 0%, #1a1a1a 52%, #0d0d0d 100%);
  backdrop-filter: blur(18px) saturate(0.85);
}

/* ========== 浅色：黑白灰 ========== */
.yj-root[data-theme="light"] {
  color: #171717;
  --yj-text: #171717;
  --yj-text-muted: #737373;
  --yj-titlebar-border: rgba(0, 0, 0, 0.1);
  --yj-dot: #171717;
  --yj-dot-shadow: rgba(0, 0, 0, 0.15);
  --yj-tab-bg: rgba(0, 0, 0, 0.06);
  --yj-tab-color: rgba(23, 23, 23, 0.72);
  --yj-tab-on-bg: #171717;
  --yj-tab-on-color: #fafafa;
  --yj-icon-border: rgba(0, 0, 0, 0.12);
  --yj-icon-bg: rgba(0, 0, 0, 0.04);
  --yj-icon-color: rgba(23, 23, 23, 0.8);
  --yj-icon-hover-bg: rgba(0, 0, 0, 0.06);
  --yj-err: #dc2626;
  --yj-table-wrap-border: rgba(0, 0, 0, 0.1);
  --yj-table-wrap-bg: rgba(0, 0, 0, 0.03);
  --yj-ctx-menu-bg: rgba(252, 252, 252, 0.98);
  --yj-modal-scrim: rgba(0, 0, 0, 0.4);
  --yj-modal-panel-bg: #fcfcfc;
  --yj-modal-panel-border: rgba(0, 0, 0, 0.12);
  --yj-scrollbar-track: rgb(0 0 0 / calc(0.03 * var(--yj-window-opacity, 1)));
  --yj-scrollbar-thumb: rgb(0 0 0 / calc(0.1 * var(--yj-window-opacity, 1)));
  --yj-scrollbar-thumb-hover: rgb(0 0 0 / calc(0.18 * var(--yj-window-opacity, 1)));
  --yj-row-border: rgba(0, 0, 0, 0.06);
  --yj-row-hover-bg: rgba(0, 0, 0, 0.04);
  --yj-th-bg: rgba(250, 250, 250, 0.96);
  --yj-th-color: rgba(23, 23, 23, 0.55);
  --yj-flat: rgba(23, 23, 23, 0.45);
  --yj-toolbar-border: rgba(0, 0, 0, 0.08);
  --yj-tool-border: rgba(0, 0, 0, 0.12);
  --yj-tool-bg: rgba(0, 0, 0, 0.04);
  --yj-tool-color: #262626;
  --yj-tool-primary-bg: #171717;
  --yj-tool-primary-border: #262626;
  --yj-tool-primary-color: #fafafa;
  --yj-settings-bg-1: #fafafa;
  --yj-settings-bg-2: #f0f0f0;
  --yj-bar-border: rgba(0, 0, 0, 0.1);
  --yj-back-border: rgba(0, 0, 0, 0.12);
  --yj-back-bg: rgba(0, 0, 0, 0.04);
  --yj-back-color: #171717;
  --yj-msg: #525252;
  --yj-side-border: rgba(0, 0, 0, 0.08);
  --yj-nav-color: rgba(23, 23, 23, 0.45);
  --yj-nav-active-bg: rgba(0, 0, 0, 0.08);
  --yj-nav-active-color: #0a0a0a;
  --yj-sub-color: rgba(23, 23, 23, 0.5);
  --yj-chip-border: rgba(0, 0, 0, 0.12);
  --yj-chip-bg: rgba(0, 0, 0, 0.03);
  --yj-chip-color: rgba(23, 23, 23, 0.88);
  --yj-chip-on-bg: #171717;
  --yj-chip-on-border: #262626;
  --yj-chip-on-color: #fafafa;
  --yj-field-border: rgba(0, 0, 0, 0.14);
  --yj-field-bg: #ffffff;
  --yj-field-color: #171717;
  --yj-btn-border: rgba(0, 0, 0, 0.14);
  --yj-btn-bg: rgba(0, 0, 0, 0.04);
  --yj-btn-color: #171717;
  --yj-btn-primary-bg: #171717;
  --yj-btn-primary-border: #262626;
  --yj-btn-primary-color: #fafafa;
  background: linear-gradient(145deg, #ffffff 0%, #f7f7f7 52%, #eeeeee 100%);
  backdrop-filter: blur(18px) saturate(0.9);
}

.yj-root {
  scrollbar-width: thin;
  scrollbar-color: var(--yj-scrollbar-thumb) var(--yj-scrollbar-track);
}

.yj-root *::-webkit-scrollbar {
  width: 1px;
  height: 1px;
}

.yj-root *::-webkit-scrollbar-track {
  background: var(--yj-scrollbar-track);
}

.yj-root *::-webkit-scrollbar-thumb {
  background-color: var(--yj-scrollbar-thumb);
  border-radius: 999px;
  border: none;
  background-clip: padding-box;
}

.yj-root *::-webkit-scrollbar-thumb:hover {
  background-color: var(--yj-scrollbar-thumb-hover);
}

.yj-root *::-webkit-scrollbar-button {
  display: none;
  width: 0;
  height: 0;
}

/*
 * 通用文本输入：与主面板表格/工具栏字号、字体栈对齐（Noto Sans SC + DM Sans）。
 * 子页面为 input/textarea 增加 class="yj-field-control" 即可复用，避免各页自成一套。
 */
.yj-root input.yj-field-control,
.yj-root textarea.yj-field-control {
  font-family: "Noto Sans SC", "DM Sans", system-ui, sans-serif;
  /* em：相对 .yj-shell 的「首页字体」；勿用 rem（相对 html）否则比 Tab 等控件显大 */
  font-size: 0.88em;
  line-height: 1.42;
  font-weight: 400;
  border-radius: 8px;
  border: 1px solid var(--yj-field-border);
  background: var(--yj-field-bg);
  color: var(--yj-field-color);
  padding: 8px 12px;
  outline: none;
  transition:
    border-color 0.15s ease,
    box-shadow 0.15s ease;
}

.yj-root input.yj-field-control::placeholder,
.yj-root textarea.yj-field-control::placeholder {
  color: var(--yj-text-muted);
  opacity: 0.72;
}

.yj-root[data-theme="dark"] input.yj-field-control:focus-visible,
.yj-root[data-theme="dark"] textarea.yj-field-control:focus-visible {
  border-color: rgba(255, 255, 255, 0.22);
  box-shadow: 0 0 0 2px rgba(255, 255, 255, 0.07);
}

.yj-root[data-theme="light"] input.yj-field-control:focus-visible,
.yj-root[data-theme="light"] textarea.yj-field-control:focus-visible {
  border-color: rgba(0, 0, 0, 0.2);
  box-shadow: 0 0 0 2px rgba(0, 0, 0, 0.06);
}

.yj-root select.yj-field-control,
.yj-root button.yj-field-control.yj-select-trigger {
  font-family: "Noto Sans SC", "DM Sans", system-ui, sans-serif;
  font-size: 0.88em;
  line-height: 1.42;
  font-weight: 400;
  border-radius: 8px;
  border: 1px solid var(--yj-field-border);
  background: var(--yj-field-bg);
  color: var(--yj-field-color);
  padding: 8px 12px;
  outline: none;
  cursor: pointer;
  transition:
    border-color 0.15s ease,
    box-shadow 0.15s ease;
}

.yj-root button.yj-field-control.yj-select-trigger {
  margin: 0;
  width: 100%;
  display: inline-flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  text-align: left;
  -webkit-appearance: none;
  appearance: none;
}

.yj-root[data-theme="dark"] select.yj-field-control:focus-visible,
.yj-root[data-theme="dark"] button.yj-field-control.yj-select-trigger:focus-visible {
  border-color: rgba(255, 255, 255, 0.22);
  box-shadow: 0 0 0 2px rgba(255, 255, 255, 0.07);
}

.yj-root[data-theme="light"] select.yj-field-control:focus-visible,
.yj-root[data-theme="light"] button.yj-field-control.yj-select-trigger:focus-visible {
  border-color: rgba(0, 0, 0, 0.2);
  box-shadow: 0 0 0 2px rgba(0, 0, 0, 0.06);
}

/* 自定义下拉面板（与右键菜单同系变量，避免系统灰底） */
.yj-select-panel {
  padding: 6px 0;
  border-radius: 10px;
  border: 1px solid var(--yj-table-wrap-border, rgba(255, 255, 255, 0.14));
  background-color: var(--yj-ctx-menu-bg, rgba(30, 30, 30, 0.97));
  background-image: none;
  box-shadow: 0 12px 36px rgba(0, 0, 0, 0.55);
  max-height: min(50vh, 280px);
  overflow-y: auto;
  -webkit-app-region: no-drag;
  app-region: no-drag;
}

.yj-root[data-theme="light"] .yj-select-panel {
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.15);
}

.yj-select-option {
  display: block;
  width: 100%;
  box-sizing: border-box;
  padding: 8px 12px;
  margin: 0;
  border: none;
  background: transparent;
  color: var(--yj-text, #f5f5f5);
  font-size: 0.88em;
  font-family: "Noto Sans SC", "DM Sans", system-ui, sans-serif;
  text-align: left;
  cursor: pointer;
  line-height: 1.35;
}

.yj-select-option:hover {
  background: var(--yj-row-hover-bg, rgba(255, 255, 255, 0.08));
}

.yj-select-option--on {
  font-weight: 600;
  background: var(--yj-row-hover-bg, rgba(255, 255, 255, 0.06));
}

.yj-root[data-theme="light"] .yj-select-option {
  color: var(--yj-text, #171717);
}

/* ========== 全局弹窗（Teleport 到 #yj-root，与右键菜单同为高不透明度浮层）========== */
.yj-root .yj-modal {
  position: fixed;
  inset: 0;
  z-index: 2147483600;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: var(--yj-modal-scrim);
  -webkit-app-region: no-drag;
  app-region: no-drag;
}

.yj-root .modal-panel {
  width: 100%;
  max-width: 300px;
  padding: 14px 16px;
  border-radius: 10px;
  border: 1px solid var(--yj-modal-panel-border);
  background-color: var(--yj-modal-panel-bg);
  background-image: none;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.55);
  font-family: "Noto Sans SC", "DM Sans", system-ui, sans-serif;
}

.yj-root[data-theme="light"] .modal-panel {
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.18);
}

.yj-root .modal-title {
  margin: 0 0 10px;
  /* 与标题栏 Tab（约 0.78em）同级视觉：略大一线作小标题，仍随首页字体缩放 */
  font-size: 0.82em;
  font-weight: 600;
  line-height: 1.35;
  color: var(--yj-text);
  letter-spacing: 0.02em;
}

.yj-root .modal-panel .modal-panel__input {
  width: 100%;
  box-sizing: border-box;
  margin-bottom: 12px;
}

.yj-root .modal-actions {
  display: flex;
  justify-content: flex-end;
  flex-wrap: wrap;
  gap: 8px;
}

.yj-root .modal-btn {
  padding: 6px 12px;
  border-radius: 8px;
  border: 1px solid var(--yj-btn-border);
  background: var(--yj-btn-bg);
  color: var(--yj-btn-color);
  font-size: 0.82em;
  font-weight: 500;
  font-family: inherit;
  cursor: pointer;
  line-height: 1.35;
  transition: filter 0.15s ease;
}

.yj-root .modal-btn:hover {
  filter: brightness(1.06);
}

.yj-root .modal-btn.primary {
  background: var(--yj-btn-primary-bg);
  border-color: var(--yj-btn-primary-border);
  color: var(--yj-btn-primary-color);
}
</style>
