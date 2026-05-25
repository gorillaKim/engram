import type { Issue, Epic, Mission } from '../ipc/types';

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

export interface GroupedHistoryEpic {
  epic: Epic;
  issues: Issue[];
}

export interface GroupedHistoryMission {
  mission: Mission | null; // null이면 미분류
  epics: GroupedHistoryEpic[];
}

export function groupIssuesByMissionAndEpic(
  issues: Issue[],
  epics: Epic[],
  missions: Mission[]
): GroupedHistoryMission[] {
  const issuesByEpic = new Map<number, Issue[]>();
  const orphanIssues: Issue[] = [];

  for (const issue of issues) {
    if (!issue.epic_id) {
      orphanIssues.push(issue);
      continue;
    }
    const list = issuesByEpic.get(issue.epic_id) ?? [];
    list.push(issue);
    issuesByEpic.set(issue.epic_id, list);
  }

  const epicsByMission = new Map<number | null, GroupedHistoryEpic[]>();

  for (const [epicId, epicIssues] of issuesByEpic) {
    const epic = epics.find((e) => e.id === epicId);
    if (!epic) {
      const dummyEpic: Epic = {
        id: epicId,
        project_key: 'SYSTEM',
        mission_id: null,
        sprint_id: null,
        title: '(에픽 정보 없음)',
        description: null,
        status: 'active',
        created_at: '',
        updated_at: '',
      };
      const list = epicsByMission.get(null) ?? [];
      list.push({ epic: dummyEpic, issues: epicIssues });
      epicsByMission.set(null, list);
      continue;
    }

    const missionId = epic.mission_id ?? null;
    const list = epicsByMission.get(missionId) ?? [];
    list.push({ epic, issues: epicIssues });
    epicsByMission.set(missionId, list);
  }

  if (orphanIssues.length > 0) {
    const dummyEpic: Epic = {
      id: 0,
      project_key: 'SYSTEM',
      mission_id: null,
      sprint_id: null,
      title: '(에픽 없음)',
      description: null,
      status: 'active',
      created_at: '',
      updated_at: '',
    };
    const list = epicsByMission.get(null) ?? [];
    list.push({ epic: dummyEpic, issues: orphanIssues });
    epicsByMission.set(null, list);
  }

  const result: GroupedHistoryMission[] = [];

  for (const mission of missions) {
    const missionEpics = epicsByMission.get(mission.id) ?? [];
    if (missionEpics.length > 0) {
      result.push({
        mission,
        epics: missionEpics,
      });
      epicsByMission.delete(mission.id);
    }
  }

  const leftoverEpics: GroupedHistoryEpic[] = [];
  for (const [, list] of epicsByMission) {
    leftoverEpics.push(...list);
  }

  if (leftoverEpics.length > 0) {
    result.push({
      mission: null,
      epics: leftoverEpics,
    });
  }

  return result;
}

