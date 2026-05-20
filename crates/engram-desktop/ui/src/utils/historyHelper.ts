import type { Issue } from '../ipc/types';

export type SortKey = 'id' | 'title' | 'updated_at' | 'priority';
export type SortOrder = 'asc' | 'desc';

const PRIORITY_WEIGHTS = {
  critical: 4,
  high: 3,
  medium: 2,
  low: 1,
};

export function sortHistoryIssues(
  issues: Issue[],
  sortKey: SortKey,
  sortOrder: SortOrder
): Issue[] {
  const sorted = [...issues];
  sorted.sort((a, b) => {
    let comparison = 0;

    if (sortKey === 'priority') {
      const weightA = PRIORITY_WEIGHTS[a.priority] ?? 0;
      const weightB = PRIORITY_WEIGHTS[b.priority] ?? 0;
      comparison = weightA - weightB;
    } else if (sortKey === 'id') {
      comparison = a.id - b.id;
    } else {
      const valA = a[sortKey] ?? '';
      const valB = b[sortKey] ?? '';
      comparison = String(valA).localeCompare(String(valB));
    }

    return sortOrder === 'asc' ? comparison : -comparison;
  });

  return sorted;
}

export function paginateHistoryIssues(
  issues: Issue[],
  visibleCount: number
): Issue[] {
  return issues.slice(0, visibleCount);
}

export function calculateHistoryStats(issues: Issue[]) {
  const priorityCounts = {
    critical: 0,
    high: 0,
    medium: 0,
    low: 0,
  };

  for (const issue of issues) {
    if (issue.priority in priorityCounts) {
      priorityCounts[issue.priority]++;
    }
  }

  return {
    totalCount: issues.length,
    priorityCounts,
  };
}

