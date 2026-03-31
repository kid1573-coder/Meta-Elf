<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, useId, watch } from "vue";
import type { BrandMoodBucket } from "../utils/brandKaomoji";

const svgUid = useId().replace(/[^a-zA-Z0-9_-]/g, "") || "meta-elf";

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
  }, 2500 + Math.floor(Math.random() * 2500));
}

watch(
  () => props.bucket,
  () => {
    blink.value = false;
  }
);

onMounted(() => {
  scheduleNextBlink();
});

onUnmounted(() => {
  if (blinkScheduleId !== undefined) window.clearTimeout(blinkScheduleId);
  if (blinkShutId !== undefined) window.clearTimeout(blinkShutId);
});

const phaseDelay = computed(() => `${(props.variant % 10) * 0.07}s`);

const isUpMood = computed(() => props.bucket.toLowerCase().includes("up"));
const isDownMood = computed(() => props.bucket.toLowerCase().includes("down"));
const isWaiting = computed(() => props.bucket === "waiting" || props.bucket === "empty");
const isStrongUp = computed(() => props.bucket === "strongUp" || props.bucket === "strongUpAll");
const isStrongDown = computed(() => props.bucket === "strongDown" || props.bucket === "strongDownAll");

// 眼睛路径 (3/4 透视：左眼小且靠左，右眼大且靠中)
const eyePaths = computed(() => {
  if (blink.value && !isStrongUp.value && !isStrongDown.value) {
    return {
      l: "M 17 56 L 23 56",
      r: "M 38 58 L 46 58"
    };
  }
  
  const b = props.bucket;
  if (b === "strongUpAll") {
    // 震惊/意外：圆睁空心眼 O O (极速拉升)
    return {
      l: "M 17 56 A 3 3 0 1 0 23 56 A 3 3 0 1 0 17 56",
      r: "M 38 58 A 4 4 0 1 0 46 58 A 4 4 0 1 0 38 58"
    };
  }
  if (b === "strongDownAll") {
    // 震惊/意外：圆睁空心眼 O O (极速跳水)
    return {
      l: "M 17 56 A 3 3 0 1 0 23 56 A 3 3 0 1 0 17 56",
      r: "M 38 58 A 4 4 0 1 0 46 58 A 4 4 0 1 0 38 58"
    };
  }
  if (b === "strongUp") {
    // 极度开心弯弯眼 ^ ^
    return {
      l: "M 17 57 Q 20 52 23 57",
      r: "M 38 59 Q 42 53 46 59"
    };
  }
  if (b === "mildUp" || b === "microUp") {
    // 微笑眼
    return {
      l: "M 18 56 Q 20.5 53 23 56",
      r: "M 39 58 Q 42.5 54 46 58"
    };
  }
  if (b === "strongDown") {
    // 痛苦紧闭眼 > <
    return {
      l: "M 18 53 L 22 56 L 18 59",
      r: "M 46 55 L 40 58 L 46 61"
    };
  }
  if (b === "mildDown" || b === "microDown") {
    // 委屈眼
    return {
      l: "M 18 55 Q 20.5 58 23 55",
      r: "M 39 57 Q 42.5 60 46 57"
    };
  }
  if (b === "flat") {
    // 坚定/发力 \ /
    return {
      l: "M 18 54 L 22 58",
      r: "M 46 56 L 40 60"
    };
  }
  if (isWaiting.value) {
    // 发呆/等待时的平淡眼 - -
    return {
      l: "M 18 56 L 22 56",
      r: "M 40 58 L 44 58"
    };
  }
  // 默认圆眼 (使用小圆点模拟 3D 眼睛)
  return {
    l: "M 20 56 L 20.1 56",
    r: "M 42 58 L 42.1 58"
  };
});

// 嘴巴路径 (3/4 透视)
const mouthPath = computed(() => {
  const b = props.bucket;
  if (b === "strongUpAll" || b === "strongDownAll") {
    // 小圆嘴 (震惊)
    return "M 29 65 A 2 2 0 1 0 33 65 A 2 2 0 1 0 29 65";
  }
  if (b === "strongUp") {
    // 大笑张开的嘴巴
    return "M 26 63 Q 30 69 35 64";
  }
  if (b === "mildUp" || b === "microUp") {
    // 微笑
    return "M 27 64 Q 30 67 34 64";
  }
  if (b === "strongDown") {
    // 委屈波浪
    return "M 26 66 Q 28 63 30 66 T 35 66";
  }
  if (b === "mildDown" || b === "microDown") {
    // 撇嘴
    return "M 27 66 Q 30 63 34 66";
  }
  if (b === "flat") {
    // 坚定紧闭
    return "M 28 65 L 33 65";
  }
  if (isWaiting.value) {
    // 平直嘴
    return "M 28 65 L 33 65";
  }
  // 默认
  return "M 28 65 L 33 65";
});

// 霓虹发光颜色映射
const themeColors = computed(() => {
  if (isStrongUp.value) return "var(--yj-up, #ff3b30)";
  if (isUpMood.value) return "var(--yj-up, #ff6b22)";
  if (isStrongDown.value) return "var(--yj-down, #34c759)";
  if (isDownMood.value) return "var(--yj-down, #30d158)";
  if (isWaiting.value) return "#0a84ff"; // 科技蓝
  return "#ff1e1e"; // 默认状态使用 3D 图片中标志性的红色
});
</script>

<template>
  <div
    class="brand-elf-shell"
    role="img"
    :aria-label="ariaLabel"
    :data-bucket="bucket"
    :data-theme="theme"
    :style="{ '--elf-anim-delay': phaseDelay, '--elf-glow-color': themeColors }"
  >
    <svg
      class="elf-svg"
      viewBox="0 0 100 100"
      width="24"
      height="24"
      aria-hidden="true"
      focusable="false"
    >
      <defs>
        <!-- 3D 身体材质：黑曜石/亮黑球体渐变 -->
        <radialGradient :id="`body-grad-${svgUid}`" cx="35%" cy="35%" r="65%">
          <stop offset="0%" stop-color="#b3e5fc" />
          <stop offset="20%" stop-color="#4fc3f7" />
          <stop offset="55%" stop-color="#0288d1" />
          <stop offset="80%" stop-color="#01579b" />
          <stop offset="100%" stop-color="#0d2137" />
        </radialGradient>

        <!-- 水晶尾巴材质：通透渐变 -->
        <linearGradient :id="`tail-grad-${svgUid}`" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stop-color="#81d4fa" />
          <stop offset="35%" stop-color="#4fc3f7" />
          <stop offset="65%" stop-color="#29b6f6" />
          <stop offset="85%" stop-color="#0277bd" />
          <stop offset="100%" stop-color="#01579b" />
        </linearGradient>

        <!-- 霓虹发光滤镜 -->
        <filter :id="`neon-glow-${svgUid}`" x="-50%" y="-50%" width="200%" height="200%">
          <feGaussianBlur stdDeviation="1.5" result="blur1" />
          <feGaussianBlur stdDeviation="3" result="blur2" />
          <feMerge>
            <feMergeNode in="blur2" />
            <feMergeNode in="blur1" />
            <feMergeNode in="SourceGraphic" />
          </feMerge>
        </filter>
      </defs>

      <g class="elf-group">
        <!-- 尾巴 (液态金属质感，从右后方卷起，3/4 透视) -->
        <path
          class="elf-tail"
          d="M 55 35 C 70 5, 95 15, 95 45 C 95 65, 80 85, 65 85 C 75 80, 82 65, 80 45 C 78 25, 65 20, 55 35 Z"
          :fill="`url(#tail-grad-${svgUid})`"
        />

        <!-- 身体 (水晶通透质感，水滴状，3/4 透视) -->
        <path
          class="elf-body"
          d="M 45 92 C 10 92, 5 55, 22 32 C 35 12, 65 15, 75 40 C 85 65, 75 92, 45 92 Z"
          :fill="`url(#body-grad-${svgUid})`"
          fill-opacity="0.88"
          stroke="#4fc3f7"
          stroke-width="0.8"
          stroke-opacity="0.5"
        />

        <!-- 身体高光 (左上角强反光，更亮更通透) -->
        <path
          class="elf-highlight"
          d="M 25 40 C 35 25, 55 25, 60 35 C 55 30, 40 28, 30 42 C 28 45, 24 43, 25 40 Z"
          fill="#e3f2fd"
          opacity="0.65"
          filter="blur(1px)"
        />

        <!-- 侧鳍/小脚 (水晶质感，3/4 透视，左远右近) -->
        <ellipse class="elf-fin elf-fin--l" cx="22" cy="78" rx="3.5" ry="6" :fill="`url(#tail-grad-${svgUid})`" transform="rotate(-30 22 78)" />
        <ellipse class="elf-fin elf-fin--r" cx="58" cy="86" rx="4.5" ry="8" :fill="`url(#tail-grad-${svgUid})`" transform="rotate(-15 58 86)" />

        <!-- 五官 (红光/主题色发光) -->
        <g class="elf-face" :filter="`url(#neon-glow-${svgUid})`">
          <!-- 眼睛 -->
          <path 
            class="elf-eye" 
            :d="eyePaths.l" 
            fill="none" 
            :stroke="themeColors" 
            stroke-width="3.5" 
            stroke-linecap="round" 
            stroke-linejoin="round" 
          />
          <path 
            class="elf-eye" 
            :d="eyePaths.r" 
            fill="none" 
            :stroke="themeColors" 
            stroke-width="3.5" 
            stroke-linecap="round" 
            stroke-linejoin="round" 
          />
          
          <!-- 嘴巴 -->
          <path 
            class="elf-mouth" 
            :d="mouthPath" 
            fill="none" 
            :stroke="themeColors" 
            stroke-width="2.5" 
            stroke-linecap="round" 
            stroke-linejoin="round" 
            opacity="0.9"
          />
        </g>
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
  width: 24px;
  height: 24px;
  margin-right: 2px;
  filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.4));
}

.elf-svg {
  width: 100%;
  height: 100%;
  overflow: visible;
}

/* 动画：整体悬浮呼吸 */
.elf-group {
  transform-origin: 50px 50px;
  animation: elf-float 4s ease-in-out infinite;
  animation-delay: var(--elf-anim-delay);
}

@keyframes elf-float {
  0%, 100% { transform: translateY(0) scale(1); }
  50% { transform: translateY(-2px) scale(1.02); }
}

/* 动画：尾巴轻微摆动 */
.elf-tail {
  transform-origin: 65px 65px;
  animation: elf-tail-wag 4s ease-in-out infinite;
  animation-delay: var(--elf-anim-delay);
}

@keyframes elf-tail-wag {
  0%, 100% { transform: rotate(0deg) scale(1); }
  50% { transform: rotate(-4deg) scale(1.02); }
}

/* 动画：鳍部微动 */
.elf-fin {
  transform-origin: center;
  animation: elf-fin-flap 4s ease-in-out infinite;
  animation-delay: var(--elf-anim-delay);
}

@keyframes elf-fin-flap {
  0%, 100% { transform: scaleX(1) rotate(var(--rot, 0deg)); }
  50% { transform: scaleX(0.8) rotate(var(--rot, 0deg)); }
}
.elf-fin--l { --rot: -30deg; }
.elf-fin--r { --rot: 20deg; }

/* 动画：面部微表情跟随浮动 (3/4 视差) */
.elf-face {
  transform-origin: 31px 58px;
  animation: elf-face-shift 4s ease-in-out infinite;
  animation-delay: var(--elf-anim-delay);
}

@keyframes elf-face-shift {
  0%, 100% { transform: translate(0, 0) scale(1); }
  50% { transform: translate(0.5px, 1.5px) scale(0.98); }
}

/* 大跌时的脉冲闪烁效果 */
.brand-elf-shell[data-bucket^="strongDown"] .elf-face {
  animation: elf-alert-pulse 1.5s ease-in-out infinite;
}

@keyframes elf-alert-pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}
</style>
