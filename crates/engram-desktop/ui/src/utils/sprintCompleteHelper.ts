import type { Issue } from '../ipc/types';

export function getUnfinishedIssues(issues: Issue[]): Issue[] {
  return issues.filter(
    (issue) => issue.status !== 'finished' && issue.status !== 'cancelled'
  );
}
