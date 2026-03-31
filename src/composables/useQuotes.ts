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
      const newRows = await invoke<QuoteRow[]>("get_quotes", {
        codes: c,
        quoteSource: src,
      });
      // Merge with existing rows to prevent jumping to 0 on temporary API failures
      if (rows.value.length > 0) {
        rows.value = newRows.map(newRow => {
          if (Math.abs(newRow.price) < 1e-9 && Math.abs(newRow.prevClose) < 1e-9 && newRow.volume === 0) {
            const oldRow = rows.value.find(r => r.code === newRow.code);
            if (oldRow) {
              return {
                ...oldRow,
                sector: newRow.sector ?? oldRow.sector,
                sectorChangePct: newRow.sectorChangePct ?? oldRow.sectorChangePct,
                volumeRatio: newRow.volumeRatio ?? oldRow.volumeRatio,
              };
            }
          }
          return newRow;
        });
      } else {
        rows.value = newRows;
      }
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
