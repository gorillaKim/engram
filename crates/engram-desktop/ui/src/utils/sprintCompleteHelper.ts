import type { Issue } from '../ipc/types';

export function getUnfinishedIssues(issues: Issue[]): Issue[] {
  return issues.filter(
    (issue) => issue.status !== 'finished' && issue.status !== 'cancelled'
  );
}

export function getUnfinishedMissions(issues: Issue[]): number[] {
  const unfinished = getUnfinishedIssues(issues);
  const missionIds = unfinished
    .map((issue) => issue.mission_id)
    .filter((id): id is number => id !== null && id !== undefined);
  return Array.from(new Set(missionIds));
}
