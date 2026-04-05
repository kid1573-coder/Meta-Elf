<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import type { BrandMoodBucket } from "../utils/brandKaomoji";

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
    }, 120);
  }, 2500 + Math.floor(Math.random() * 2000));
}

watch(() => props.bucket, () => { blink.value = false; });
onMounted(scheduleNextBlink);
onUnmounted(() => {
  if (blinkScheduleId) window.clearTimeout(blinkScheduleId);
  if (blinkShutId) window.clearTimeout(blinkShutId);
});

const isUpMood = computed(() => props.bucket.toLowerCase().includes("up"));
const isDownMood = computed(() => props.bucket.toLowerCase().includes("down"));
const isWaiting = computed(() => props.bucket === "waiting" || props.bucket === "empty");
const isStrongUp = computed(() => props.bucket === "strongUp" || props.bucket === "strongUpAll");
const isStrongDown = computed(() => props.bucket === "strongDown" || props.bucket === "strongDownAll");

// GBA风格调色板 - 鲜艳活泼
const colors = computed(() => {
  const isDark = props.theme === "dark";
  return {
    main: isDark ? "#ff6b6b" : "#e84545",
    light: isDark ? "#ff8787" : "#ff6b6b",
    highlight: isDark ? "#ffa8a8" : "#ff8787",
    dark: isDark ? "#c0392b" : "#b71c1c",
    shadow: isDark ? "#7f1d1d" : "#5c0a0a",
    white: "#ffffff",
    black: "#1a1a2e",
    cheek: "#ffb3b3",
    tear: "#87ceeb",
  };
});

// 20x18 像素螃蟹精灵 - 更大更可爱
const P = 1.5;

// 像素类型
type Pixel = [number, number, string];

// 身体主体 (更圆润)
const bodyPixels = computed((): Pixel[] => {
  const c = colors.value;
  return [
    // 壳顶高光 - 更亮
    [8, 0, c.highlight], [9, 0, c.highlight], [10, 0, c.highlight],
    [7, 1, c.light], [8, 1, c.white], [9, 1, c.white], [10, 1, c.white], [11, 1, c.light],
    // 壳主体 - 更饱满
    [5, 2, c.main], [6, 2, c.light], [7, 2, c.light], [8, 2, c.highlight], [9, 2, c.highlight], [10, 2, c.highlight], [11, 2, c.light], [12, 2, c.light], [13, 2, c.main],
    [4, 3, c.dark], [5, 3, c.main], [6, 3, c.main], [7, 3, c.light], [8, 3, c.light], [9, 3, c.light], [10, 3, c.light], [11, 3, c.light], [12, 3, c.main], [13, 3, c.main], [14, 3, c.dark],
    [3, 4, c.dark], [4, 4, c.main], [5, 4, c.main], [6, 4, c.main], [7, 4, c.main], [8, 4, c.main], [9, 4, c.main], [10, 4, c.main], [11, 4, c.main], [12, 4, c.main], [13, 4, c.main], [14, 4, c.dark],
    [3, 5, c.shadow], [4, 5, c.dark], [5, 5, c.main], [6, 5, c.main], [7, 5, c.main], [8, 5, c.main], [9, 5, c.main], [10, 5, c.main], [11, 5, c.main], [12, 5, c.main], [13, 5, c.dark], [14, 5, c.shadow],
    [4, 6, c.shadow], [5, 6, c.dark], [6, 6, c.main], [7, 6, c.main], [8, 6, c.main], [9, 6, c.main], [10, 6, c.main], [11, 6, c.main], [12, 6, c.dark], [13, 6, c.shadow],
    // 底部圆润
    [5, 7, c.shadow], [6, 7, c.dark], [7, 7, c.main], [8, 7, c.main], [9, 7, c.main], [10, 7, c.main], [11, 7, c.dark], [12, 7, c.shadow],
    [6, 8, c.shadow], [7, 8, c.dark], [8, 8, c.dark], [9, 8, c.dark], [10, 8, c.dark], [11, 8, c.shadow],
    // 腿 (6条，更明显)
    [2, 6, c.dark], [2, 7, c.shadow], [1, 7, c.shadow],
    [3, 7, c.dark], [3, 8, c.shadow],
    [14, 7, c.dark], [14, 8, c.shadow],
    [15, 6, c.dark], [15, 7, c.shadow], [16, 7, c.shadow],
    [5, 9, c.dark], [5, 10, c.shadow],
    [12, 9, c.dark], [12, 10, c.shadow],
  ];
});

// 左钳子 (更大更圆润)
const leftClawPixels = computed(() => {
  const c = colors.value;
  return [
    [0, 3, c.dark], [0, 4, c.main], [0, 5, c.dark],
    [1, 2, c.dark], [1, 3, c.light], [1, 4, c.highlight], [1, 5, c.main], [1, 6, c.shadow],
    [2, 2, c.main], [2, 3, c.light], [2, 4, c.highlight], [2, 5, c.main], [2, 6, c.dark],
    [3, 3, c.main], [3, 4, c.light], [3, 5, c.dark],
  ];
});

// 右钳子 (更大更圆润)
const rightClawPixels = computed(() => {
  const c = colors.value;
  return [
    [15, 3, c.dark], [15, 4, c.main], [15, 5, c.dark],
    [16, 2, c.dark], [16, 3, c.light], [16, 4, c.highlight], [16, 5, c.main], [16, 6, c.shadow],
    [17, 2, c.main], [17, 3, c.light], [17, 4, c.highlight], [17, 5, c.main], [17, 6, c.dark],
    [18, 3, c.main], [18, 4, c.light], [18, 5, c.dark],
  ];
});

// 眼睛柄 (更明显)
const eyeStalkPixels = computed(() => {
  const c = colors.value;
  return [
    [6, 0, c.dark], [7, -1, c.dark],
    [11, -1, c.dark], [12, 0, c.dark],
  ];
});

// 眼睛 (更大更可爱)
const eyePixels = computed(() => {
  const c = colors.value;
  const pixels: [number, number, string][] = [];

  if (blink.value) {
    // 眨眼 - 弯弯的眯眼
    pixels.push([5, -2, c.black], [6, -3, c.black], [7, -2, c.black]);
    pixels.push([11, -2, c.black], [12, -3, c.black], [13, -2, c.black]);
  } else if (isStrongUp.value) {
    // 超开心弯眼 ^ ^
    pixels.push([5, -3, c.black], [6, -4, c.black], [7, -3, c.black]);
    pixels.push([5, -2, c.black], [7, -2, c.black]);
    pixels.push([11, -3, c.black], [12, -4, c.black], [13, -3, c.black]);
    pixels.push([11, -2, c.black], [13, -2, c.black]);
  } else if (isStrongDown.value) {
    // X眼 + 眼泪
    pixels.push([5, -4, c.black], [6, -3, c.black], [7, -2, c.black]);
    pixels.push([5, -2, c.black], [6, -3, c.black], [7, -4, c.black]);
    pixels.push([11, -4, c.black], [12, -3, c.black], [13, -2, c.black]);
    pixels.push([11, -2, c.black], [12, -3, c.black], [13, -4, c.black]);
    // 眼泪
    pixels.push([4, -1, c.tear], [4, 0, c.tear]);
    pixels.push([14, -1, c.tear], [14, 0, c.tear]);
  } else if (isDownMood.value) {
    // 委屈下垂眼
    pixels.push([5, -4, c.black], [6, -3, c.black], [7, -3, c.black], [7, -2, c.black]);
    pixels.push([11, -4, c.black], [12, -3, c.black], [13, -3, c.black], [13, -2, c.black]);
  } else if (isWaiting.value) {
    // 发呆点眼 (´・ω・`)
    pixels.push([6, -3, c.black]);
    pixels.push([12, -3, c.black]);
  } else {
    // 默认大圆眼 - 更可爱
    pixels.push([5, -4, c.black], [6, -4, c.black], [7, -4, c.black]);
    pixels.push([5, -3, c.black], [6, -3, c.white], [7, -3, c.black]);
    pixels.push([5, -2, c.black], [6, -2, c.black], [7, -2, c.black]);
    // 高光点
    pixels.push([6, -4, c.white]);
    pixels.push([11, -4, c.black], [12, -4, c.black], [13, -4, c.black]);
    pixels.push([11, -3, c.black], [12, -3, c.white], [13, -3, c.black]);
    pixels.push([11, -2, c.black], [12, -2, c.black], [13, -2, c.black]);
    pixels.push([12, -4, c.white]);
  }
  return pixels;
});

// 嘴巴 (更丰富)
const mouthPixels = computed(() => {
  const c = colors.value;
  const pixels: [number, number, string][] = [];

  if (isStrongUp.value) {
    // 大笑 w形嘴
    pixels.push([7, 6, c.black], [8, 5, c.black], [9, 5, c.white], [10, 5, c.black], [11, 6, c.black]);
    pixels.push([7, 7, c.black], [8, 7, c.black], [9, 7, c.black], [10, 7, c.black]);
  } else if (isUpMood.value) {
    // 开心微笑
    pixels.push([8, 6, c.black], [9, 6, c.black], [10, 6, c.black]);
  } else if (isStrongDown.value) {
    // 大哭波浪嘴
    pixels.push([7, 6, c.black], [8, 7, c.black], [9, 6, c.black], [10, 7, c.black], [11, 6, c.black]);
  } else if (isDownMood.value) {
    // 委屈撇嘴
    pixels.push([7, 7, c.black], [8, 6, c.black], [9, 7, c.black], [10, 6, c.black], [11, 7, c.black]);
  } else if (isWaiting.value) {
    // 发呆小o嘴
    pixels.push([8, 6, c.black], [9, 6, c.black], [10, 6, c.black]);
    pixels.push([8, 7, c.black], [10, 7, c.black]);
    pixels.push([8, 8, c.black], [9, 8, c.black], [10, 8, c.black]);
  } else {
    // 默认小嘴
    pixels.push([8, 6, c.black], [9, 6, c.black], [10, 6, c.black]);
  }
  return pixels;
});

// 腮红 (更明显)
const cheekPixels = computed(() => {
  if (isStrongDown.value) return [];
  const c = colors.value;
  if (isStrongUp.value) {
    // 超开心大腮红
    return [
      [3, 4, c.cheek], [4, 4, c.cheek], [3, 5, c.cheek],
      [14, 4, c.cheek], [15, 4, c.cheek], [15, 5, c.cheek],
    ];
  }
  if (isUpMood.value) {
    return [
      [3, 4, c.cheek], [4, 4, c.cheek],
      [14, 4, c.cheek], [15, 4, c.cheek],
    ];
  }
  return [];
});

// 生成 box-shadow
function toShadow(pixels: [number, number, string][]) {
  return pixels.map(([x, y, c]) => `${x * P}px ${y * P}px ${c}`).join(", ");
}

const bodyShadow = computed(() => toShadow(bodyPixels.value));
const leftClawShadow = computed(() => toShadow(leftClawPixels.value));
const rightClawShadow = computed(() => toShadow(rightClawPixels.value));
const eyeStalkShadow = computed(() => toShadow(eyeStalkPixels.value));
const eyeShadow = computed(() => toShadow(eyePixels.value));
const mouthShadow = computed(() => toShadow(mouthPixels.value));
const cheekShadow = computed(() => toShadow(cheekPixels.value));

const clawAnim = computed(() => {
  if (isStrongUp.value) return "happy";
  if (isStrongDown.value) return "sad";
  if (isUpMood.value) return "cheer";
  return "";
});
</script>

<template>
  <div
    class="crab-mascot"
    :class="clawAnim"
    :data-bucket="bucket"
    role="img"
    :aria-label="ariaLabel"
  >
    <div class="pixel crab-body" :style="{ boxShadow: bodyShadow }"></div>
    <div class="pixel crab-claw claw-l" :style="{ boxShadow: leftClawShadow }"></div>
    <div class="pixel crab-claw claw-r" :style="{ boxShadow: rightClawShadow }"></div>
    <div class="pixel crab-stalk" :style="{ boxShadow: eyeStalkShadow }"></div>
    <div class="pixel crab-eye" :style="{ boxShadow: eyeShadow }"></div>
    <div class="pixel crab-mouth" :style="{ boxShadow: mouthShadow }"></div>
    <div v-if="cheekShadow" class="pixel crab-cheek" :style="{ boxShadow: cheekShadow }"></div>
  </div>
</template>

<style scoped>
.crab-mascot {
  position: relative;
  width: 28px;
  height: 20px;
  flex-shrink: 0;
  image-rendering: pixelated;
  image-rendering: crisp-edges;
  /* 垂直居中对齐 */
  display: flex;
  align-items: center;
  justify-content: center;
}

.pixel {
  position: absolute;
  top: 12px;
  left: -1px;
  width: 1px;
  height: 1px;
  pointer-events: none;
}

.crab-body {
  animation: idle 2s ease-in-out infinite;
}

.crab-eye {
  animation: eye-sparkle 3s ease-in-out infinite;
}

.crab-claw {
  transform-origin: center right;
  transition: transform 0.2s ease;
}

.claw-r {
  transform-origin: center left;
}

/* 开心时钳子举起摇摆 */
.crab-mascot.happy .claw-l {
  animation: wave-left 0.3s ease-in-out infinite;
}

.crab-mascot.happy .claw-r {
  animation: wave-right 0.3s ease-in-out infinite;
}

.crab-mascot.happy .crab-body {
  animation: bounce 0.4s ease-in-out infinite;
  filter: drop-shadow(0 0 3px rgba(255, 200, 0, 0.6));
}

/* 轻松开心 */
.crab-mascot.cheer .claw-l {
  animation: sway-left 0.6s ease-in-out infinite;
}

.crab-mascot.cheer .claw-r {
  animation: sway-right 0.6s ease-in-out infinite;
}

/* 难过时钳子下垂 */
.crab-mascot.sad .claw-l,
.crab-mascot.sad .claw-r {
  transform: translateY(2px) scaleY(0.85);
}

.crab-mascot.sad .crab-body {
  animation: shiver 0.2s ease-in-out infinite;
}

@keyframes idle {
  0%, 100% { transform: translateY(0) scaleY(1); }
  50% { transform: translateY(-0.5px) scaleY(1.02); }
}

@keyframes bounce {
  0%, 100% { transform: translateY(0) scaleY(1); }
  50% { transform: translateY(-1.5px) scaleY(1.03); }
}

@keyframes eye-sparkle {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.9; }
}

@keyframes wave-left {
  0%, 100% { transform: rotate(0deg); }
  50% { transform: rotate(-30deg) translateY(-3px); }
}

@keyframes wave-right {
  0%, 100% { transform: rotate(0deg); }
  50% { transform: rotate(30deg) translateY(-3px); }
}

@keyframes sway-left {
  0%, 100% { transform: rotate(0deg); }
  50% { transform: rotate(-15deg); }
}

@keyframes sway-right {
  0%, 100% { transform: rotate(0deg); }
  50% { transform: rotate(15deg); }
}

@keyframes shiver {
  0%, 100% { transform: translateX(0); }
  25% { transform: translateX(-0.5px); }
  75% { transform: translateX(0.5px); }
}
</style>
