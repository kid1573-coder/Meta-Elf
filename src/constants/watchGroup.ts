import type { AppSettings } from "../types/app";

export const LAST_WATCH_GROUP_KEY = "yj-last-watch-group";

/** 从 URL query、最近选中或第一个分组解析目标分组 id */
export function resolveTargetGroupId(
  settings: Pick<AppSettings, "watchGroups"> | null,
  queryGroup: string | undefined,
): string {
  if (!settings?.watchGroups.length) return "";
  const q = String(queryGroup ?? "").trim();
  if (q && settings.watchGroups.some((g) => g.id === q)) return q;
  try {
    const last = sessionStorage.getItem(LAST_WATCH_GROUP_KEY);
    if (last && settings.watchGroups.some((g) => g.id === last)) return last;
  } catch {
    /* ignore */
  }
  return settings.watchGroups[0].id;
}
