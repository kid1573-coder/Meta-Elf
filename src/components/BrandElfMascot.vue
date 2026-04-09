<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import type { BrandMoodBucket } from "../utils/brandKaomoji";

const props = defineProps<{
  bucket: BrandMoodBucket;
  variant: number;
  theme: "light" | "dark";
  colorScheme?: "redUp" | "greenUp";
  ariaLabel: string;
}>();

// ── blink (overlay) ──
const blink = ref(false);
let blinkT1: ReturnType<typeof setTimeout> | undefined;
let blinkT2: ReturnType<typeof setTimeout> | undefined;

function scheduleBlink() {
  blinkT1 = window.setTimeout(() => {
    blink.value = true;
    blinkT2 = window.setTimeout(() => {
      blink.value = false;
      scheduleBlink();
    }, 100);
  }, 2200 + Math.floor(Math.random() * 2800));
}

watch(() => props.bucket, () => { blink.value = false; });
onMounted(scheduleBlink);
onUnmounted(() => {
  if (blinkT1) clearTimeout(blinkT1);
  if (blinkT2) clearTimeout(blinkT2);
});

// ── mood flags ──
const isWaiting = computed(() => props.bucket === "waiting" || props.bucket === "empty");
const isStrongUp = computed(() => props.bucket === "strongUp" || props.bucket === "strongUpAll");
const isStrongDown = computed(() => props.bucket === "strongDown" || props.bucket === "strongDownAll");
const isMildUp = computed(() => props.bucket === "mildUp");
const isMildDown = computed(() => props.bucket === "mildDown");
const isMicroUp = computed(() => props.bucket === "microUp");
const isMicroDown = computed(() => props.bucket === "microDown");

const animClass = computed(() => {
  if (isStrongUp.value) return "excited";
  if (isMildUp.value || isMicroUp.value) return "happy";
  if (isStrongDown.value) return "devastated";
  if (isMildDown.value || isMicroDown.value) return "sad";
  if (isWaiting.value) return "sleepy";
  return "idle";
});

const moodEmoji = computed(() => {
  if (blink.value) return "−";
  if (isStrongUp.value) return "★";
  if (isStrongDown.value) return "×";
  if (isWaiting.value) return "…";
  return "";
});

type Pix = [number, number, string];

const GRID = 16;
const P = 1.5; // 缩小为 1.5 (24x24)，之前是 2 (32x32)

function buildMascotPixels(): Pix[] {
  const OUT = props.theme === "dark" ? "#0f172a" : "#1e1b4b";
  
  // 保持脸部绝对纯白，只改变高光或阴影来提示涨跌
  let bodyW = '#f8fafc'; // 主体脸部
  let bodyL = '#ffffff'; // 高光
  let bodyD = '#cbd5e1'; // 阴影
  const B = '#93c5fd';
  const P_BLUSH = '#fca5a5';

  const isUp = isStrongUp.value || isMildUp.value || isMicroUp.value;
  const isDown = isStrongDown.value || isMildDown.value || isMicroDown.value;

  if (isUp) {
    if (props.colorScheme === "greenUp") {
      bodyD = '#bbf7d0'; // 阴影带点绿
    } else {
      bodyD = '#fecaca'; // 阴影带点红
    }
  } else if (isDown) {
    if (props.colorScheme === "greenUp") {
      bodyD = '#fecaca'; // 阴影带点红
    } else {
      bodyD = '#bbf7d0'; // 阴影带点绿
    }
  }

  if (isWaiting.value || props.bucket === "flat") {
     bodyW = '#f8fafc';
     bodyL = '#ffffff';
     bodyD = '#cbd5e1';
  }

  const grid = new Map<string, string>();
  function set(x: number, y: number, c: string) { grid.set(`${x},${y}`, c); }

  const basePixels = [
    "......OOOO......",
    "....OOLLLLO.....",
    "...OLLWWWWLOO...",
    "..OLWWWWWWWWWO..",
    ".OLWWWWWWWWWWWO.",
    ".OLWWWWWWWWWWWO.",
    ".OLWBEWWBWBEWWO.",
    ".OLWWWWWWWWWWWO.",
    ".OLWPWWWWWWWPWO.",
    ".OLWWWWWWWWWWWO.",
    "..ODWWWWWWWWWDO.",
    "...ODDDDDDDWDO..",
    "....OOOOOODWDO..",
    ".........ODDO...",
    ".........OO.....",
    "................"
  ];

  for (let y = 0; y < 16; y++) {
    for (let x = 0; x < 16; x++) {
      const char = basePixels[y][x];
      if (char === 'O') set(x, y, OUT);
      else if (char === 'W') set(x, y, bodyW);
      else if (char === 'L') set(x, y, bodyL);
      else if (char === 'D') set(x, y, bodyD);
      else if (char === 'B') set(x, y, B);
      else if (char === 'E') set(x, y, OUT);
      else if (char === 'P') set(x, y, P_BLUSH);
    }
  }
  
  // Overrides for face
  function clearFace() {
    set(4, 6, bodyW); set(5, 6, bodyW); set(6, 6, bodyW); set(9, 6, bodyW); set(10, 6, bodyW); set(11, 6, bodyW);
    set(4, 8, bodyW); set(12, 8, bodyW); // blush pos
  }

  function drawDefaultEyes() {
    set(4, 6, B); set(5, 6, OUT); set(6, 6, bodyW);
    set(9, 6, B); set(10, 6, OUT); set(11, 6, bodyW);
  }

  function drawClosedEyes() {
    set(4, 6, OUT); set(5, 6, OUT); set(6, 6, bodyW);
    set(9, 6, OUT); set(10, 6, OUT); set(11, 6, bodyW);
  }

  function drawBlush() {
    set(4, 8, P_BLUSH); set(12, 8, P_BLUSH);
  }

  if (isWaiting.value) {
    clearFace();
    drawClosedEyes();
    set(7, 8, OUT); set(8, 8, OUT); // sleep mouth
  } else if (isDown) {
    clearFace();
    drawClosedEyes();
    set(7, 8, OUT); set(8, 8, OUT); // frown
    if (isStrongDown.value) {
      set(4, 7, '#60a5fa'); set(4, 8, '#60a5fa'); // tear
    }
  } else if (isUp) {
    clearFace();
    drawDefaultEyes();
    drawBlush();
    set(7, 7, OUT); set(8, 7, OUT); // smile
    if (isStrongUp.value) {
      set(13, 3, '#fcd34d'); set(14, 2, '#fcd34d'); set(14, 4, '#fcd34d'); set(15, 3, '#fcd34d');
    }
  } else {
    // Flat/Neutral
    clearFace();
    drawDefaultEyes();
    drawBlush();
  }

  if (blink.value && !isWaiting.value && !isDown) {
    clearFace();
    drawClosedEyes();
    drawBlush();
  }

  return Array.from(grid.entries()).map(([k, c]) => {
    const [xs, ys] = k.split(",");
    return [Number(xs), Number(ys), c] as Pix;
  });
}

const pixelData = computed<Pix[]>(() => {
  return buildMascotPixels();
});

// box-shadow builder
function sh(pixels: Pix[], scale: number): string {
  return pixels.map(([x, y, c]) => `${x * scale}px ${y * scale}px 0 0 ${c}`).join(",");
}

const artStyle = computed(() => ({
  width: `${GRID * P}px`,
  height: `${GRID * P}px`,
}));

const shadowStyle = computed(() => ({
  width: `${P}px`,
  height: `${P}px`,
  boxShadow: sh(pixelData.value, P),
}));
</script>

<template>
  <div
    class="elf"
    :class="animClass"
    :data-bucket="bucket"
    role="img"
    :aria-label="ariaLabel"
    :style="artStyle"
  >
    <div class="px" :style="shadowStyle"></div>
    <span v-if="moodEmoji" class="mood-overlay">{{ moodEmoji }}</span>
  </div>
</template>

<style scoped>
.elf {
  position: relative;
  display: flex;
  align-items: center;
  flex-shrink: 0;
  line-height: 0;
}

.px {
  display: block;
  image-rendering: pixelated;
  image-rendering: crisp-edges;
}

.mood-overlay {
  position: absolute;
  top: -1px;
  right: -1px;
  font-size: 8px;
  line-height: 1;
  pointer-events: none;
  color: #ffd700;
  font-weight: bold;
  text-shadow: 0 0 2px rgba(255,215,0,0.6);
}

/* idle */
.idle .px { animation: bob 2.4s ease-in-out infinite; }

/* excited */
.excited .px { animation: bounce 0.35s ease-in-out infinite; }

/* happy */
.happy .px { animation: bob 1.6s ease-in-out infinite; }

/* devastated */
.devastated .px { animation: shake 0.18s ease-in-out infinite; }

/* sad */
.sad .px { animation: bob 3s ease-in-out infinite; opacity: 0.85; }

/* sleepy */
.sleepy .px { animation: breathe 3s ease-in-out infinite; }

@keyframes bob       { 0%,100%{transform:translateY(0)} 50%{transform:translateY(-1px)} }
@keyframes bounce    { 0%,100%{transform:translateY(0) scaleY(1)} 50%{transform:translateY(-2px) scaleY(1.03)} }
@keyframes shake     { 0%,100%{transform:translateX(0)} 25%{transform:translateX(-1px)} 75%{transform:translateX(1px)} }
@keyframes breathe   { 0%,100%{transform:scaleY(1)} 50%{transform:scaleY(1.03)} }
</style>
