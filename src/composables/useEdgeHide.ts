import { onUnmounted, watch, type Ref } from "vue";
import { currentMonitor, getCurrentWindow } from "@tauri-apps/api/window";
import type { AppSettings } from "../types/app";

const EDGE_PX = 14;

/**
 * 窗口贴近屏幕左右边缘时自动隐藏（可通过托盘或老板键唤回）
 */
export function useEdgeHide(settings: Ref<AppSettings | null>) {
  let unlisten: (() => void) | undefined;

  async function attach() {
    const win = getCurrentWindow();
    unlisten?.();
    unlisten = await win.onMoved(async () => {
      const s = settings.value;
      if (!s?.autoHideEdge) return;
      try {
        const pos = await win.outerPosition();
        const size = await win.outerSize();
        const mon = await currentMonitor();
        if (!mon) return;
        const wa = mon.workArea;
        const x = wa.position.x;
        const y = wa.position.y;
        const mw = wa.size.width;
        const mh = wa.size.height;
        const cx = pos.x;
        const cy = pos.y;
        const w = size.width;
        const h = size.height;
        const nearLeft = cx <= x + EDGE_PX;
        const nearRight = cx + w >= x + mw - EDGE_PX;
        const nearTop = cy <= y + EDGE_PX;
        const nearBottom = cy + h >= y + mh - EDGE_PX;
        if (nearLeft || nearRight || nearTop || nearBottom) {
          await win.hide();
        }
      } catch {
        /* ignore */
      }
    });
  }

  watch(
    () => settings.value?.autoHideEdge,
    async (on) => {
      if (on) await attach();
      else {
        unlisten?.();
        unlisten = undefined;
      }
    },
    { immediate: true },
  );

  onUnmounted(() => {
    unlisten?.();
    unlisten = undefined;
  });
}
