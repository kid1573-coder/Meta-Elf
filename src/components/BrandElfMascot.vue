<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import type { BrandMoodBucket } from "../utils/brandKaomoji";

const props = defineProps<{
  bucket: BrandMoodBucket;
  variant: number;
  theme: "light" | "dark";
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

const GRID = 14;
const P = 1; // 缩小像素缩放比例，让 logo 更小

function buildMascotPixels(): Pix[] {
  const C  = "#e97451"; // coral main
  const CL = "#f09070"; // coral light
  const CD = "#c4533a"; // coral dark
  const CK = "#a83820"; // coral deep
  const W  = "#ffcc66"; // warm center
  const WL = "#ffe0a0"; // warm highlight
  const T  = "#ff8c42"; // orange tip

  const starburstBase: Pix[] = [
    // top ray
    [7,0,T],[8,0,T],
    [6,1,CL],[7,1,T],[8,1,T],[9,1,CL],
    [7,2,C],[8,2,C],
    // upper left ray
    [3,2,T],[4,2,CL],
    [2,3,T],[3,3,CL],[4,3,C],
    [3,4,C],[4,4,C],
    // upper right ray
    [11,2,CL],[12,2,T],
    [11,3,C],[12,3,CL],[13,3,T],
    [11,4,C],[12,4,C],
    // left ray
    [1,7,T],[1,8,T],
    [2,6,CL],[2,7,C],[2,8,C],[2,9,CL],
    [3,7,C],[3,8,C],
    // right ray
    [13,7,T],[13,8,T],
    [13,6,CL],[13,7,C],[13,8,C],[13,9,CL],
    [12,7,C],[12,8,C],
    // lower left ray
    [3,11,C],[4,11,C],
    [2,10,CL],[3,10,C],[4,10,C],
    [3,12,CL],[4,12,T],
    [3,13,T],
    // lower right ray
    [11,11,C],[12,11,C],
    [11,10,C],[12,10,CL],[13,10,T],
    [11,12,C],[12,12,CL],
    [12,13,T],
    // bottom ray
    [7,14,T],[8,14,T],
    [6,13,CL],[7,13,C],[8,13,C],[9,13,CL],
    [7,12,C],[8,12,C],
    // inner body (circle)
    [4,4,C],[5,4,C],[6,4,C],[7,4,C],[8,4,C],[9,4,C],[10,4,C],[11,4,C],
    [3,5,C],[4,5,C],[5,5,C],[6,5,C],[7,5,C],[8,5,C],[9,5,C],[10,5,C],[11,5,C],[12,5,C],
    [3,6,C],[4,6,C],[5,6,CD],[6,6,CD],[7,6,CD],[8,6,CD],[9,6,CD],[10,6,CD],[11,6,C],[12,6,C],
    [3,7,CD],[4,7,CD],[5,7,CD],[6,7,W],[7,7,W],[8,7,W],[9,7,W],[10,7,CD],[11,7,CD],[12,7,CD],
    [3,8,CD],[4,8,CD],[5,8,W],[6,8,W],[7,8,WL],[8,8,WL],[9,8,W],[10,8,W],[11,8,CD],[12,8,CD],
    [3,9,CD],[4,9,CD],[5,9,CD],[6,9,W],[7,9,W],[8,9,W],[9,9,W],[10,9,CD],[11,9,CD],[12,9,CD],
    [3,10,C],[4,10,C],[5,10,CD],[6,10,CD],[7,10,CD],[8,10,CD],[9,10,CD],[10,10,CD],[11,10,C],[12,10,C],
    [4,11,C],[5,11,C],[6,11,C],[7,11,C],[8,11,C],[9,11,C],[10,11,C],[11,11,C],
  ];

  const eyesBase: Pix[] = [
    [6,7,CK],[7,6,CK],
    [8,7,CK],[9,6,CK],
  ];

  const sparklePix: Pix[] = [
    [0,6,W],[0,8,W],[14,6,W],[14,8,W],
    [1,5,WL],[1,9,WL],[13,5,WL],[13,9,WL],
  ];

  const base = [...starburstBase, ...eyesBase];
  if (isStrongUp.value || isMildUp.value || isMicroUp.value) {
    return [...base, ...sparklePix];
  }
  return base;
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
