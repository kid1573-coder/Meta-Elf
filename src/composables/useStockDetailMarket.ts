import { computed, onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type {
  ChartTabId,
  IntradaySeries,
  KlinePeriodArg,
  KlineSeries,
  OrderBook,
} from "../types/marketDetail";
import { shouldShowStockDetailOrderBook } from "../utils/orderBookEligibility";

function tabToKlinePeriod(tab: ChartTabId): KlinePeriodArg | null {
  if (tab === "day") return "day";
  if (tab === "week") return "week";
  if (tab === "month") return "month";
  return null;
}

export function useStockDetailMarket(
  code: () => string,
  quoteSource: () => string,
) {
  const chartTab = ref<ChartTabId>("intraday");
  const intradaySeries = ref<IntradaySeries | null>(null);
  const klineByPeriod = ref<Partial<Record<KlinePeriodArg, KlineSeries>>>({});
  const orderBook = ref<OrderBook | null>(null);
  const chartLoading = ref(false);
  const chartErr = ref<string | null>(null);
  const bookLoading = ref(false);
  const bookErr = ref<string | null>(null);

  const currentKline = computed(() => {
    const p = tabToKlinePeriod(chartTab.value);
    if (!p) return null;
    return klineByPeriod.value[p] ?? null;
  });

  function resetCaches() {
    intradaySeries.value = null;
    klineByPeriod.value = {};
    orderBook.value = null;
    chartErr.value = null;
    bookErr.value = null;
  }

  async function loadIntraday() {
    const c = code().trim().toLowerCase();
    const src = quoteSource();
    if (!c) return;
    chartLoading.value = true;
    chartErr.value = null;
    try {
      intradaySeries.value = await invoke<IntradaySeries>("get_stock_intraday", {
        code: c,
        quoteSource: src,
      });
    } catch (e) {
      chartErr.value = String(e);
      intradaySeries.value = null;
    } finally {
      chartLoading.value = false;
    }
  }

  async function loadKline(period: KlinePeriodArg) {
    const c = code().trim().toLowerCase();
    const src = quoteSource();
    if (!c) return;
    chartLoading.value = true;
    chartErr.value = null;
    try {
      const data = await invoke<KlineSeries>("get_stock_kline", {
        code: c,
        period,
        quoteSource: src,
      });
      klineByPeriod.value = { ...klineByPeriod.value, [period]: data };
    } catch (e) {
      chartErr.value = String(e);
    } finally {
      chartLoading.value = false;
    }
  }

  async function loadOrderBook() {
    const c = code().trim().toLowerCase();
    const src = quoteSource();
    if (!c) return;
    if (!shouldShowStockDetailOrderBook(c)) {
      orderBook.value = null;
      bookErr.value = null;
      bookLoading.value = false;
      return;
    }
    bookLoading.value = true;
    bookErr.value = null;
    try {
      orderBook.value = await invoke<OrderBook>("get_stock_order_book", {
        code: c,
        quoteSource: src,
      });
    } catch (e) {
      bookErr.value = String(e);
      orderBook.value = null;
    } finally {
      bookLoading.value = false;
    }
  }

  async function ensureChartForTab() {
    const tab = chartTab.value;
    if (tab === "intraday") {
      if (!intradaySeries.value) await loadIntraday();
    } else {
      const p = tabToKlinePeriod(tab);
      if (p && !klineByPeriod.value[p]) await loadKline(p);
    }
  }

  watch(
    () => [code(), quoteSource()] as const,
    () => {
      resetCaches();
      const c = code().trim().toLowerCase();
      if (!c) return;
      void loadOrderBook();
      void ensureChartForTab();
    },
  );

  watch(chartTab, () => {
    void ensureChartForTab();
  });

  onMounted(() => {
    if (!code().trim()) return;
    void loadOrderBook();
    void ensureChartForTab();
  });

  return {
    chartTab,
    intradaySeries,
    currentKline,
    orderBook,
    chartLoading,
    chartErr,
    bookLoading,
    bookErr,
    loadOrderBook,
    ensureChartForTab,
  };
}
