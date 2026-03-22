<script setup lang="ts">
import {
  computed,
  nextTick,
  onUnmounted,
  ref,
  useAttrs,
  watch,
} from "vue";

defineOptions({ inheritAttrs: false });

const props = defineProps<{
  modelValue: string;
  options: { value: string; label: string }[];
  ariaLabel?: string;
}>();

const emit = defineEmits<{ (e: "update:modelValue", v: string): void }>();

const attrs = useAttrs();
const rootEl = ref<HTMLElement | null>(null);
const panelRef = ref<HTMLElement | null>(null);
const open = ref(false);
const panelStyle = ref<Record<string, string>>({});

const currentLabel = computed(() => {
  const hit = props.options.find((o) => o.value === props.modelValue);
  return hit?.label ?? props.options[0]?.label ?? "";
});

function positionPanel() {
  const root = rootEl.value;
  if (!root) return;
  const trigger = root.querySelector(".yj-select-trigger") as HTMLElement | null;
  if (!trigger) return;
  const r = trigger.getBoundingClientRect();
  panelStyle.value = {
    position: "fixed",
    zIndex: "2147483000",
    top: `${Math.round(r.bottom + 4)}px`,
    left: `${Math.round(r.left)}px`,
    minWidth: `${Math.round(r.width)}px`,
  };
}

function toggle() {
  open.value = !open.value;
}

function pick(v: string) {
  emit("update:modelValue", v);
  open.value = false;
}

function onPointerDownCapture(e: PointerEvent) {
  if (!open.value) return;
  const t = e.target as Node;
  if (rootEl.value?.contains(t) || panelRef.value?.contains(t)) return;
  open.value = false;
}

function onWinReposition() {
  if (open.value) positionPanel();
}

function onKeyDown(e: KeyboardEvent) {
  if (e.key === "Escape") open.value = false;
}

watch(open, (v) => {
  if (v) {
    window.addEventListener("pointerdown", onPointerDownCapture, true);
    window.addEventListener("scroll", onWinReposition, true);
    window.addEventListener("resize", onWinReposition);
    window.addEventListener("keydown", onKeyDown);
    nextTick(() => {
      positionPanel();
    });
  } else {
    window.removeEventListener("pointerdown", onPointerDownCapture, true);
    window.removeEventListener("scroll", onWinReposition, true);
    window.removeEventListener("resize", onWinReposition);
    window.removeEventListener("keydown", onKeyDown);
  }
});

watch(
  () => props.options,
  () => {
    if (open.value) nextTick(positionPanel);
  },
  { deep: true },
);

onUnmounted(() => {
  window.removeEventListener("pointerdown", onPointerDownCapture, true);
  window.removeEventListener("scroll", onWinReposition, true);
  window.removeEventListener("resize", onWinReposition);
  window.removeEventListener("keydown", onKeyDown);
});
</script>

<template>
  <div ref="rootEl" class="yj-select-root" v-bind="attrs">
    <button
      type="button"
      class="yj-field-control yj-select-trigger"
      :aria-expanded="open"
      aria-haspopup="listbox"
      :aria-label="ariaLabel"
      @click="toggle"
    >
      <span class="yj-select-trigger__text">{{ currentLabel }}</span>
      <span class="yj-select-trigger__chev" aria-hidden="true">▾</span>
    </button>
    <Teleport to="#yj-root">
      <div
        v-show="open"
        ref="panelRef"
        class="yj-select-panel"
        role="listbox"
        :style="panelStyle"
      >
        <button
          v-for="o in options"
          :key="o.value"
          type="button"
          class="yj-select-option"
          :class="{ 'yj-select-option--on': o.value === modelValue }"
          role="option"
          :aria-selected="o.value === modelValue"
          @click="pick(o.value)"
        >
          {{ o.label }}
        </button>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.yj-select-root {
  display: block;
  min-width: 0;
}

.yj-select-trigger__text {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  text-align: left;
}

.yj-select-trigger__chev {
  flex-shrink: 0;
  font-size: 0.72em;
  opacity: 0.65;
  line-height: 1;
}
</style>
