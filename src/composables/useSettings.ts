import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { AppSettings } from "../types/app";
import { ensureWatchGroupsField } from "../utils/watchGroups";

const settings = ref<AppSettings | null>(null);
const loading = ref(true);

export function useSettings() {
  async function load() {
    loading.value = true;
    try {
      const s = await invoke<AppSettings>("load_settings");
      const qs = (s as AppSettings).quoteSource;
      settings.value = {
        ...s,
        theme: (s as AppSettings).theme === "light" ? "light" : "dark",
        panelMode: (s.panelMode as AppSettings["panelMode"]) ?? "normal",
        colorScheme: (s.colorScheme as AppSettings["colorScheme"]) ?? "redUp",
        profitDisplay: (s.profitDisplay as AppSettings["profitDisplay"]) ?? "both",
        quoteSource:
          qs === "mock" ? "mock" : qs === "tencent" ? "tencent" : "eastmoney",
      };
    } finally {
      loading.value = false;
    }
  }

  async function save(partial?: Partial<AppSettings>) {
    if (!settings.value) return;
    const next = { ...settings.value, ...partial };
    const saved = await invoke<AppSettings>("save_settings", { settings: next });
    const qs = (saved as AppSettings).quoteSource;
    settings.value = {
      ...saved,
      watchGroups: ensureWatchGroupsField(saved),
      theme: (saved as AppSettings).theme === "light" ? "light" : "dark",
      panelMode: (saved.panelMode as AppSettings["panelMode"]) ?? "normal",
      colorScheme: (saved.colorScheme as AppSettings["colorScheme"]) ?? "redUp",
      profitDisplay: (saved.profitDisplay as AppSettings["profitDisplay"]) ?? "both",
      quoteSource:
        qs === "mock" ? "mock" : qs === "tencent" ? "tencent" : "eastmoney",
    };
    return settings.value;
  }

  async function applyWindowPrefs() {
    if (!settings.value) return;
    await invoke("apply_window_prefs", {
      opacity: settings.value.opacity,
      alwaysOnTop: settings.value.alwaysOnTop,
      skipTaskbar: settings.value.skipTaskbar,
    });
  }

  const rootStyle = computed(() => {
    const s = settings.value;
    if (!s) return {};
    return {
      fontSize: `${s.fontSizePx}px`,
      "--yj-window-opacity": String(s.opacity),
    } as Record<string, string>;
  });

  return {
    settings,
    loading,
    load,
    save,
    applyWindowPrefs,
    rootStyle,
  };
}
