import type { Issue, Epic } from '../ipc/types';

export function getUnfinishedIssues(issues: Issue[]): Issue[] {
  return issues.filter(
    (issue) => issue.status !== 'finished' && issue.status !== 'cancelled'
  );
}

/**
 * ADR-0014: Epic 이 sprint SSOT. 미완료 이슈의 epic_id 들을 모은 뒤 distinct.
 * 이 epic 들을 다른 sprint 로 옮기면 산하 issue 가 자동으로 따라온다.
 */
export function getUnfinishedEpics(issues: Issue[]): number[] {
  const unfinished = getUnfinishedIssues(issues);
  const epicIds = unfinished
    .map((issue) => issue.epic_id)
    .filter((id): id is number => id !== null && id !== undefined);
  return Array.from(new Set(epicIds));
}

export interface EpicClassification {
  toComplete: Epic[];
  toTransfer: Epic[];
}

export function classifyEpics(epics: Epic[], issues: Issue[]): EpicClassification {
  const toComplete: Epic[] = [];
  const toTransfer: Epic[] = [];

  for (const epic of epics) {
    const epicIssues = issues.filter((i) => i.epic_id === epic.id);
    const hasIssues = epicIssues.length > 0;
    const allCompleted = hasIssues && epicIssues.every(
      (i) => i.status === 'finished' || i.status === 'cancelled'
    );

    if (allCompleted) {
      toComplete.push(epic);
    } else {
      toTransfer.push(epic);
    }
  }

  return { toComplete, toTransfer };
}

export function getUnfinishedIssuesForEpic(epicId: number, issues: Issue[]): Issue[] {
  return getUnfinishedIssues(issues).filter((issue) => issue.epic_id === epicId);
}
