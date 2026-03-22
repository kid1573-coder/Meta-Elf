<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, useId, watch } from "vue";
import type { BrandMoodBucket } from "../utils/brandKaomoji";

const svgUid = useId().replace(/[^a-zA-Z0-9_-]/g, "") || "elf";
const tailFilterId = `elf-tail-glow-${svgUid}`;

const props = defineProps<{
  bucket: BrandMoodBucket;
  variant: number;
  theme: "light" | "dark";
  ariaLabel: string;
}>();

const blink = ref(false);
let blinkScheduleId: ReturnType<typeof setTimeout> | undefined;
let blinkShutId: ReturnType<typeof setTimeout> | undefined;

function scheduleNextBlink() {
  blinkScheduleId = window.setTimeout(() => {
    blink.value = true;
    blinkShutId = window.setTimeout(() => {
      blink.value = false;
      scheduleNextBlink();
    }, 95);
  }, 2200 + Math.floor(Math.random() * 2000));
}

watch(
  () => props.bucket,
  () => {
    blink.value = false;
  },
);

onMounted(() => {
  scheduleNextBlink();
});

onUnmounted(() => {
  if (blinkScheduleId !== undefined) window.clearTimeout(blinkScheduleId);
  if (blinkShutId !== undefined) window.clearTimeout(blinkShutId);
});

const phaseDelay = computed(() => `${(props.variant % 10) * 0.07}s`);

const showTears = computed(
  () => props.bucket === "strongDownAll",
);

const mouthPath = computed(() => {
  const b = props.bucket;
  if (b === "strongUp" || b === "strongUpAll") {
    return "M 24 45 Q 32 54 40 45";
  }
  if (b === "mildUp") {
    return "M 25 46 Q 32 51 39 46";
  }
  if (b === "microUp") {
    return "M 26 46.5 Q 32 49.5 38 46.5";
  }
  if (b === "strongDown" || b === "strongDownAll") {
    return "M 24 50 Q 32 41 40 50";
  }
  if (b === "mildDown") {
    return "M 25 49 Q 32 44 39 49";
  }
  if (b === "microDown") {
    return "M 26 48.5 Q 32 45.2 38 48.5";
  }
  return "M 25 48.5 L 39 48.5";
});
</script>

<template>
  <div
    class="brand-elf-shell"
    role="img"
    :aria-label="ariaLabel"
    :data-bucket="bucket"
    :data-theme="theme"
    :class="{
      'brand-elf-shell--glowUp': bucket === 'strongUp' || bucket === 'strongUpAll',
      'brand-elf-shell--glowDown':
        bucket === 'strongDown' || bucket === 'strongDownAll',
    }"
    :style="{ '--elf-anim-delay': phaseDelay }"
  >
    <svg
      class="elf-svg"
      viewBox="0 0 64 64"
      width="24"
      height="24"
      aria-hidden="true"
      focusable="false"
    >
      <defs>
        <linearGradient :id="`elf-body-fill-${svgUid}`" x1="32" y1="14" x2="32" y2="58" gradientUnits="userSpaceOnUse">
          <stop offset="0%" stop-color="var(--elf-body-top)" />
          <stop offset="55%" stop-color="var(--elf-body-mid)" />
          <stop offset="100%" stop-color="var(--elf-body-bot)" />
        </linearGradient>
        <linearGradient :id="`elf-tail-fill-${svgUid}`" x1="28" y1="4" x2="56" y2="28" gradientUnits="userSpaceOnUse">
          <stop offset="0%" stop-color="var(--elf-tail-root)" />
          <stop offset="55%" stop-color="var(--elf-tail-mid)" />
          <stop offset="100%" stop-color="var(--elf-tail-tip)" />
        </linearGradient>
        <filter :id="tailFilterId" x="-45%" y="-45%" width="190%" height="190%" color-interpolation-filters="sRGB">
          <feGaussianBlur in="SourceGraphic" stdDeviation="1.35" result="b" />
          <feColorMatrix
            in="b"
            type="matrix"
            values="1 0 0 0 0  0 1 0 0 0  0 0 1 0 0  0 0 0 0.55 0"
            result="g"
          />
          <feMerge>
            <feMergeNode in="g" />
            <feMergeNode in="SourceGraphic" />
          </feMerge>
        </filter>
      </defs>

      <!-- 钩尾（后层）：黑→银白渐变 + 柔光 -->
      <path
        class="elf-tail"
        :filter="`url(#${tailFilterId})`"
        :fill="`url(#elf-tail-fill-${svgUid})`"
        d="M 30 22
           C 28 10 36 4 44 6
           C 54 9 58 18 54 26
           C 50 32 44 30 40 24
           C 36 20 32 22 30 22 Z"
      />

      <!-- 主体梨形 -->
      <path
        class="elf-body"
        :fill="`url(#elf-body-fill-${svgUid})`"
        stroke="var(--elf-outline)"
        stroke-width="1.2"
        stroke-linejoin="round"
        d="M 32 20
           C 46 20 54 30 52 42
           C 50 54 42 58 32 58
           C 22 58 14 54 12 42
           C 10 30 18 20 32 20 Z"
      />

      <!-- 侧鳍 -->
      <ellipse class="elf-fin elf-fin--l" cx="15" cy="40" rx="5" ry="6" />
      <ellipse class="elf-fin elf-fin--r" cx="49" cy="40" rx="5" ry="6" />

      <!-- 眼睛 -->
      <g
        class="elf-eyes"
        :class="{ 'elf-eyes--blink': blink }"
        style="transform-origin: 32px 36px"
      >
        <circle class="elf-eye" cx="25" cy="36" r="3.2" fill="var(--elf-eye)" />
        <circle class="elf-eye" cx="39" cy="36" r="3.2" fill="var(--elf-eye)" />
        <circle class="elf-eye-shine" cx="26.2" cy="34.8" r="0.9" fill="var(--elf-eye-shine)" />
        <circle class="elf-eye-shine" cx="40.2" cy="34.8" r="0.9" fill="var(--elf-eye-shine)" />
      </g>

      <!-- 嘴：弧线加大 + 描边加粗，涨跌与横盘都可辨 -->
      <path
        class="elf-mouth"
        fill="none"
        stroke="var(--elf-mouth)"
        stroke-width="2.15"
        stroke-linecap="round"
        stroke-linejoin="round"
        :d="mouthPath"
      />

      <!-- 泪滴（仅 strongDownAll） -->
      <g v-if="showTears" class="elf-tears">
        <rect class="elf-tear" x="22.5" y="41" width="2.2" height="7" rx="1" fill="var(--elf-tear)" />
        <rect class="elf-tear" x="39.3" y="41" width="2.2" height="7" rx="1" fill="var(--elf-tear)" />
      </g>
    </svg>
  </div>
</template>

<style scoped>
.brand-elf-shell {
  flex-shrink: 0;
  line-height: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  animation: elf-breathe-base 3.2s ease-in-out infinite;
  animation-delay: var(--elf-anim-delay, 0s);
}

.brand-elf-shell--glowUp {
  filter: drop-shadow(0 0 4px var(--elf-shell-glow, rgba(251, 191, 36, 0.55)));
}

.brand-elf-shell--glowDown {
  filter: drop-shadow(0 0 4px var(--elf-shell-glow-down, rgba(248, 113, 113, 0.55)));
}

.elf-svg {
  display: block;
  overflow: visible;
  shape-rendering: geometricPrecision;
}

.elf-tail {
  animation: elf-tail-glow 2.8s ease-in-out infinite;
  animation-delay: var(--elf-anim-delay, 0s);
}

.elf-mouth {
  filter: drop-shadow(0 0.35px 0 color-mix(in srgb, var(--elf-mouth) 55%, transparent));
}

.elf-fin {
  fill: var(--elf-fin-top);
  opacity: 0.92;
}

.elf-eyes {
  transform-box: fill-box;
  transform-origin: 32px 36px;
  transition: transform 0.06s ease-out;
}

.elf-eyes--blink {
  animation: elf-blink 0.11s ease-in-out;
}

.elf-tears .elf-tear {
  animation: elf-tear 1.1s ease-in-out infinite;
  animation-delay: var(--elf-anim-delay, 0s);
}

.elf-tears .elf-tear:last-child {
  animation-delay: calc(var(--elf-anim-delay, 0s) + 0.15s);
}

@keyframes elf-breathe-base {
  0%,
  100% {
    transform: translateY(0) scale(1);
  }
  50% {
    transform: translateY(-0.5px) scale(1.03);
  }
}

@keyframes elf-tail-glow {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.88;
  }
}

@keyframes elf-blink {
  0% {
    transform: scaleY(1);
  }
  45% {
    transform: scaleY(0.12);
  }
  100% {
    transform: scaleY(1);
  }
}

@keyframes elf-tear {
  0%,
  100% {
    opacity: 0.35;
  }
  50% {
    opacity: 0.95;
  }
}

/* ---------- 深色主题 + 分桶 ---------- */
.brand-elf-shell[data-theme="dark"][data-bucket="empty"] {
  --elf-body-top: #2a2a2a;
  --elf-body-mid: #121212;
  --elf-body-bot: #0a0a0a;
  --elf-outline: #3f3f46;
  --elf-tail-root: #1a1a1a;
  --elf-tail-mid: #525252;
  --elf-tail-tip: #d4d4d8;
  --elf-fin-top: #27272a;
  --elf-fin-bot: #18181b;
  --elf-eye: #f87171;
  --elf-eye-shine: #fecaca;
  --elf-mouth: #a1a1aa;
  --elf-tear: #38bdf8;
}

.brand-elf-shell[data-theme="dark"][data-bucket="waiting"] {
  --elf-body-top: #262626;
  --elf-body-mid: #141414;
  --elf-body-bot: #0a0a0a;
  --elf-outline: #525252;
  --elf-tail-root: #171717;
  --elf-tail-mid: #737373;
  --elf-tail-tip: #e5e5e5;
  --elf-fin-top: #262626;
  --elf-fin-bot: #171717;
  --elf-eye: #f87171;
  --elf-eye-shine: #fecaca;
  --elf-mouth: #cbd5e1;
  --elf-tear: #94a3b8;
}

.brand-elf-shell[data-theme="dark"][data-bucket="flat"] {
  --elf-body-top: #3f3f46;
  --elf-body-mid: #27272a;
  --elf-body-bot: #18181b;
  --elf-outline: #52525b;
  --elf-tail-root: #27272a;
  --elf-tail-mid: #71717a;
  --elf-tail-tip: #a1a1aa;
  --elf-fin-top: #3f3f46;
  --elf-fin-bot: #27272a;
  --elf-eye: #e4e4e7;
  --elf-eye-shine: #fafafa;
  --elf-mouth: #d4d4d8;
  --elf-tear: #7dd3fc;
}

.brand-elf-shell[data-theme="dark"][data-bucket="microUp"],
.brand-elf-shell[data-theme="dark"][data-bucket="mildUp"] {
  --elf-body-top: #262626;
  --elf-body-mid: #171717;
  --elf-body-bot: #0a0a0a;
  --elf-outline: #525252;
  --elf-tail-root: #1f1f1f;
  --elf-tail-mid: #a3a3a3;
  --elf-tail-tip: #fef9c3;
  --elf-fin-top: #262626;
  --elf-fin-bot: #171717;
  --elf-eye: #fcd34d;
  --elf-eye-shine: #fffbeb;
  --elf-mouth: #fde047;
  --elf-tear: #94a3b8;
}

.brand-elf-shell[data-theme="dark"][data-bucket="strongUp"],
.brand-elf-shell[data-theme="dark"][data-bucket="strongUpAll"] {
  --elf-body-top: #1c1c1c;
  --elf-body-mid: #0f0f0f;
  --elf-body-bot: #050505;
  --elf-outline: #737373;
  --elf-tail-root: #292524;
  --elf-tail-mid: #fbbf24;
  --elf-tail-tip: #fffbeb;
  --elf-fin-top: #262626;
  --elf-fin-bot: #171717;
  --elf-eye: #fde047;
  --elf-eye-shine: #fff;
  --elf-mouth: #fef08a;
  --elf-tear: #94a3b8;
  --elf-shell-glow: rgba(251, 191, 36, 0.42);
}

.brand-elf-shell[data-theme="dark"][data-bucket="microDown"],
.brand-elf-shell[data-theme="dark"][data-bucket="mildDown"] {
  --elf-body-top: #2a2a2a;
  --elf-body-mid: #171717;
  --elf-body-bot: #0a0a0a;
  --elf-outline: #525252;
  --elf-tail-root: #1a1a1a;
  --elf-tail-mid: #737373;
  --elf-tail-tip: #e2e8f0;
  --elf-fin-top: #262626;
  --elf-fin-bot: #171717;
  --elf-eye: #fb7185;
  --elf-eye-shine: #ffe4e6;
  --elf-mouth: #fda4af;
  --elf-tear: #94a3b8;
}

.brand-elf-shell[data-theme="dark"][data-bucket="strongDown"],
.brand-elf-shell[data-theme="dark"][data-bucket="strongDownAll"] {
  --elf-body-top: #7f1d1d;
  --elf-body-mid: #450a0a;
  --elf-body-bot: #2a0a0a;
  --elf-outline: #fca5a5;
  --elf-tail-root: #450a0a;
  --elf-tail-mid: #dc2626;
  --elf-tail-tip: #fecaca;
  --elf-fin-top: #991b1b;
  --elf-fin-bot: #450a0a;
  --elf-eye: #fecaca;
  --elf-eye-shine: #fff;
  --elf-mouth: #fecaca;
  --elf-tear: #7dd3fc;
  --elf-shell-glow-down: rgba(248, 113, 113, 0.6);
}

/* 浅色主题 */
.brand-elf-shell[data-theme="light"][data-bucket="empty"] {
  --elf-body-top: #f5f5f4;
  --elf-body-mid: #e7e5e4;
  --elf-body-bot: #d6d3d1;
  --elf-outline: #78716c;
  --elf-tail-root: #d6d3d1;
  --elf-tail-mid: #a8a29e;
  --elf-tail-tip: #fafaf9;
  --elf-fin-top: #e7e5e4;
  --elf-fin-bot: #d6d3d1;
  --elf-eye: #dc2626;
  --elf-eye-shine: #fecaca;
  --elf-mouth: #57534e;
  --elf-tear: #0284c7;
}

.brand-elf-shell[data-theme="light"][data-bucket="waiting"] {
  --elf-body-top: #f5f5f5;
  --elf-body-mid: #e8e8e8;
  --elf-body-bot: #d4d4d4;
  --elf-outline: #737373;
  --elf-tail-root: #d4d4d4;
  --elf-tail-mid: #a3a3a3;
  --elf-tail-tip: #fafafa;
  --elf-fin-top: #e8e8e8;
  --elf-fin-bot: #d4d4d4;
  --elf-eye: #dc2626;
  --elf-eye-shine: #fecaca;
  --elf-mouth: #525252;
  --elf-tear: #64748b;
}

.brand-elf-shell[data-theme="light"][data-bucket="flat"] {
  --elf-body-top: #f4f4f5;
  --elf-body-mid: #e4e4e7;
  --elf-body-bot: #d4d4d8;
  --elf-outline: #71717a;
  --elf-tail-root: #d4d4d8;
  --elf-tail-mid: #a1a1aa;
  --elf-tail-tip: #fafafa;
  --elf-fin-top: #e4e4e7;
  --elf-fin-bot: #d4d4d8;
  --elf-eye: #27272a;
  --elf-eye-shine: #fafafa;
  --elf-mouth: #3f3f46;
  --elf-tear: #0284c7;
}

.brand-elf-shell[data-theme="light"][data-bucket="microUp"],
.brand-elf-shell[data-theme="light"][data-bucket="mildUp"] {
  --elf-body-top: #f4f4f5;
  --elf-body-mid: #e4e4e7;
  --elf-body-bot: #d4d4d8;
  --elf-outline: #71717a;
  --elf-tail-root: #d4d4d8;
  --elf-tail-mid: #a1a1aa;
  --elf-tail-tip: #fef3c7;
  --elf-fin-top: #e4e4e7;
  --elf-fin-bot: #d4d4d8;
  --elf-eye: #b45309;
  --elf-eye-shine: #fffbeb;
  --elf-mouth: #b45309;
  --elf-tear: #64748b;
}

.brand-elf-shell[data-theme="light"][data-bucket="strongUp"],
.brand-elf-shell[data-theme="light"][data-bucket="strongUpAll"] {
  --elf-body-top: #f5f5f4;
  --elf-body-mid: #e7e5e4;
  --elf-body-bot: #d6d3d1;
  --elf-outline: #78716c;
  --elf-tail-root: #d6d3d1;
  --elf-tail-mid: #fbbf24;
  --elf-tail-tip: #fff;
  --elf-fin-top: #e7e5e4;
  --elf-fin-bot: #d6d3d1;
  --elf-eye: #a16207;
  --elf-eye-shine: #fff;
  --elf-mouth: #a16207;
  --elf-tear: #64748b;
  --elf-shell-glow: rgba(251, 191, 36, 0.38);
}

.brand-elf-shell[data-theme="light"][data-bucket="microDown"],
.brand-elf-shell[data-theme="light"][data-bucket="mildDown"] {
  --elf-body-top: #f4f4f5;
  --elf-body-mid: #e4e4e7;
  --elf-body-bot: #d4d4d8;
  --elf-outline: #71717a;
  --elf-tail-root: #d4d4d8;
  --elf-tail-mid: #94a3b8;
  --elf-tail-tip: #f1f5f9;
  --elf-fin-top: #e4e4e7;
  --elf-fin-bot: #d4d4d8;
  --elf-eye: #be123c;
  --elf-eye-shine: #ffe4e6;
  --elf-mouth: #be123c;
  --elf-tear: #64748b;
}

.brand-elf-shell[data-theme="light"][data-bucket="strongDown"],
.brand-elf-shell[data-theme="light"][data-bucket="strongDownAll"] {
  --elf-body-top: #e7e5e4;
  --elf-body-mid: #d6d3d1;
  --elf-body-bot: #c4c2bf;
  --elf-outline: #991b1b;
  --elf-tail-root: #fca5a5;
  --elf-tail-mid: #ef4444;
  --elf-tail-tip: #fff;
  --elf-fin-top: #d6d3d1;
  --elf-fin-bot: #c4c2bf;
  --elf-eye: #7f1d1d;
  --elf-eye-shine: #fff;
  --elf-mouth: #991b1b;
  --elf-tear: #64748b;
  --elf-shell-glow-down: rgba(220, 38, 38, 0.35);
}

/* 分桶节奏：涨跌抖 / 等待慢漂 */
.brand-elf-shell[data-bucket="strongUp"],
.brand-elf-shell[data-bucket="strongUpAll"],
.brand-elf-shell[data-bucket="microUp"],
.brand-elf-shell[data-bucket="mildUp"] {
  animation-name: elf-breathe-up;
  animation-duration: 2.2s;
}

.brand-elf-shell[data-bucket="strongDown"],
.brand-elf-shell[data-bucket="strongDownAll"],
.brand-elf-shell[data-bucket="microDown"],
.brand-elf-shell[data-bucket="mildDown"] {
  animation-name: elf-shake-mild;
  animation-duration: 0.45s;
  animation-timing-function: ease-in-out;
}

.brand-elf-shell[data-bucket="waiting"] {
  animation-name: elf-float-wait;
  animation-duration: 2.6s;
}

.brand-elf-shell[data-bucket="empty"],
.brand-elf-shell[data-bucket="flat"] {
  animation-duration: 3.6s;
}

@keyframes elf-breathe-up {
  0%,
  100% {
    transform: translateY(0) scale(1);
  }
  40% {
    transform: translateY(-0.5px) scale(1.03);
  }
  60% {
    transform: translateY(0) scale(1.015);
  }
}

@keyframes elf-shake-mild {
  0%,
  100% {
    transform: translateX(0) rotate(0deg);
  }
  25% {
    transform: translateX(-0.8px) rotate(-1.2deg);
  }
  75% {
    transform: translateX(0.8px) rotate(1.2deg);
  }
}

@keyframes elf-float-wait {
  0%,
  100% {
    transform: translateY(0);
    opacity: 1;
  }
  50% {
    transform: translateY(-1px);
    opacity: 0.92;
  }
}
</style>
