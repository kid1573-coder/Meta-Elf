<script setup lang="ts">
import { onMounted } from "vue";
import { useSettings } from "./composables/useSettings";

const { load, applyWindowPrefs, rootStyle } = useSettings();

onMounted(async () => {
  await load();
  await applyWindowPrefs();
});
</script>

<template>
  <div id="yj-root" class="yj-root" :style="rootStyle">
    <router-view />
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
.yj-root {
  margin: 0;
  height: 100%;
  width: 100%;
  overflow: hidden;
}

body {
  font-family: "Noto Sans SC", "DM Sans", system-ui, sans-serif;
  background: transparent;
  color: #e8e6f2;
  -webkit-font-smoothing: antialiased;
}

.yj-root {
  background: linear-gradient(
    145deg,
    rgba(22, 18, 42, 0.78) 0%,
    rgba(32, 24, 58, 0.82) 55%,
    rgba(18, 22, 48, 0.86) 100%
  );
  backdrop-filter: blur(18px) saturate(1.35);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 12px;
  box-shadow: 0 18px 50px rgba(0, 0, 0, 0.45), inset 0 1px 0 rgba(255, 255, 255, 0.06);
}
</style>
