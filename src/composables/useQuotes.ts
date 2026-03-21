import { onUnmounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { QuoteRow } from "../types/app";

export function useQuotes(intervalMs = 5000) {
  const rows = ref<QuoteRow[]>([]);
  const err = ref<string | null>(null);
  let timer: ReturnType<typeof setInterval> | undefined;

  async function refresh() {
    try {
      err.value = null;
      rows.value = await invoke<QuoteRow[]>("get_quotes");
    } catch (e) {
      err.value = String(e);
    }
  }

  function start() {
    void refresh();
    timer = setInterval(() => void refresh(), intervalMs);
  }

  function stop() {
    if (timer) clearInterval(timer);
    timer = undefined;
  }

  onUnmounted(stop);

  return { rows, err, refresh, start, stop };
}
