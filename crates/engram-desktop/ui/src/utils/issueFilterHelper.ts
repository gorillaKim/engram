import type { Issue } from '../ipc/types';

export function filterFinishedIssues(issues: Issue[], hide: boolean): Issue[] {
  if (!hide) return issues;
  return issues.filter(
    (issue) => issue.status !== 'finished' && issue.status !== 'cancelled'
  );
}
