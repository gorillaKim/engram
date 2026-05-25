import type { Epic, Sprint } from '../ipc/types';

export interface SprintDistEntry {
  label: string;
  count: number;
  sprintId: number | null;
}

export function summarizeEpicSprints(epics: Epic[], sprints: Sprint[]): SprintDistEntry[] {
  const buckets = new Map<number | null, number>();
  for (const e of epics) {
    const key = e.sprint_id ?? null;
    buckets.set(key, (buckets.get(key) ?? 0) + 1);
  }
  return Array.from(buckets, ([sprintId, count]) => ({
    sprintId,
    count,
    label: sprintId === null ? '백로그' : (sprints.find(s => s.id === sprintId)?.name ?? `S${sprintId}`),
  })).sort((a, b) => (a.sprintId ?? 9e9) - (b.sprintId ?? 9e9));
}
