import { onUnmounted, ref, watch, type Ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { QuoteRow } from "../types/app";

export function useQuotes(
  codes: Ref<string[]>,
  quoteSource: Ref<string>,
  intervalMs = 6000,
) {
  const rows = ref<QuoteRow[]>([]);
  const err = ref<string | null>(null);
  let timer: ReturnType<typeof setInterval> | undefined;

  async function refresh() {
    const c = codes.value;
    const src = quoteSource.value || "eastmoney";
    if (c.length === 0) {
      rows.value = [];
      err.value = null;
      return;
    }
    try {
      err.value = null;
      rows.value = await invoke<QuoteRow[]>("get_quotes", {
        codes: c,
        quoteSource: src,
      });
    } catch (e) {
      err.value = String(e);
    }
  }

  function start() {
    void refresh();
    if (timer) clearInterval(timer);
    timer = setInterval(() => void refresh(), intervalMs);
  }

  function stop() {
    if (timer) clearInterval(timer);
    timer = undefined;
  }

  watch(
    [codes, quoteSource],
    () => {
      void refresh();
    },
    { deep: true },
  );

  onUnmounted(stop);

  return { rows, err, refresh, start, stop };
}
