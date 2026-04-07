import type { Ref } from "vue";
import type { AppSettings } from "../types/app";

/**
 * 贴边自动隐藏已禁用 — 窗口仅在手动最小化时隐藏。
 */
export function useEdgeHide(_settings: Ref<AppSettings | null>) {
  // no-op
}
