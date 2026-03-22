import type { AppSettings, WatchGroup } from "../types/app";

export function codeInAnyGroup(groups: WatchGroup[], code: string): boolean {
  const c = code.toLowerCase();
  return groups.some((g) => g.codes.some((x) => x.toLowerCase() === c));
}

export function stripCodeFromAllGroups(groups: WatchGroup[], code: string): void {
  const c = code.toLowerCase();
  for (const g of groups) {
    g.codes = g.codes.filter((x) => x.toLowerCase() !== c);
  }
}

/** 从 watchlist 移除已不属于任何分组的代码 */
export function pruneWatchlistIfOrphaned(settings: AppSettings): void {
  const { watchlist, watchGroups } = settings;
  settings.watchlist = watchlist.filter((w) => codeInAnyGroup(watchGroups, w.code));
}

export function ensureWatchGroupsField(s: {
  watchlist: { code: string }[];
  watchGroups?: WatchGroup[];
}): WatchGroup[] {
  if (s.watchGroups && s.watchGroups.length > 0) return s.watchGroups;
  return [
    {
      id: crypto.randomUUID(),
      name: "自选股",
      codes: s.watchlist.map((w) => w.code),
    },
  ];
}
