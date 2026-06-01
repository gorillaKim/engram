import { useMemo } from 'react';
import type { Epic, Issue, Mission } from '../ipc/types';
import { filterFinishedIssues } from '../utils/issueFilterHelper';

export interface GroupedEpic {
  epic: Epic;
  issues: Issue[];
}

export interface GroupedMission {
  mission: Mission | null;
  epics: GroupedEpic[];
}

interface UseGroupedMissionsParams {
  issuesInView: Issue[];
  allEpics: Epic[];
  missions: Mission[];
  hideFinished: boolean;
  hideFinishedEpics: boolean;
  selectedMissionIds: number[];
  selectedEpicIds: number[];
  debouncedQuery: string;
}

export function useGroupedMissions({
  issuesInView,
  allEpics,
  missions,
  hideFinished,
  hideFinishedEpics,
  selectedMissionIds,
  selectedEpicIds,
  debouncedQuery,
}: UseGroupedMissionsParams): GroupedMission[] {
  // 3단계 계층형 가공 (Mission -> Epic -> Issue)
  const groupedMissions = useMemo(() => {
    const filteredIssues = filterFinishedIssues(issuesInView, hideFinished);
    
    // 1. epic_id -> issues mapping
    const issuesByEpic = new Map<number, Issue[]>();
    for (const issue of filteredIssues) {
      const list = issuesByEpic.get(issue.epic_id) ?? [];
      list.push(issue);
      issuesByEpic.set(issue.epic_id, list);
    }
    
    // 2. mission_id -> GroupedEpic[] mapping
    const epicsByMission = new Map<number | null, GroupedEpic[]>();
    
    for (const [epicId, epicIssues] of issuesByEpic) {
      const epic = allEpics.find((e) => e.id === epicId);
      if (!epic) continue;
      
      // 완료/취소 에픽 숨기기 필터링
      if (hideFinishedEpics && (epic.status === 'completed' || epic.status === 'cancelled')) {
        continue;
      }

      const missionId = epic.mission_id ?? null;
      const list = epicsByMission.get(missionId) ?? [];
      list.push({ epic, issues: epicIssues });
      epicsByMission.set(missionId, list);
    }
    
    const result: GroupedMission[] = [];
    
    // 3. 미션 목록 기준으로 GroupedMission 빌드
    for (const mission of missions) {
      const epics = epicsByMission.get(mission.id) ?? [];
      if (epics.length > 0 || mission.status === 'active') {
        result.push({
          mission,
          epics
        });
        epicsByMission.delete(mission.id);
      }
    }
    
    // 4. mission_id가 없거나 missions 목록에 없지만 leftover인 에픽들을 "미분류"로 수집
    const leftoverEpics: GroupedEpic[] = [];
    for (const [, epics] of epicsByMission) {
      leftoverEpics.push(...epics);
    }
    
    if (leftoverEpics.length > 0) {
      result.push({
        mission: null,
        epics: leftoverEpics
      });
    }
    
    return result;
  }, [issuesInView, allEpics, missions, hideFinished, hideFinishedEpics]);

  const filteredGroupedMissions = useMemo(() => {
    let list = groupedMissions;
    
    // 1. 미션 필터 적용
    if (selectedMissionIds.length > 0) {
      list = list.filter((gm) => {
        if (gm.mission === null) {
          return selectedMissionIds.includes(0);
        }
        return selectedMissionIds.includes(gm.mission.id);
      });
    }
    
    // 2. 에픽 필터 적용
    if (selectedEpicIds.length > 0) {
      list = list.map((gm) => ({
        ...gm,
        epics: gm.epics.filter((ge) => selectedEpicIds.includes(ge.epic.id))
      })).filter((gm) => gm.epics.length > 0);
    }
    
    // 3. 검색어 필터 적용
    const q = debouncedQuery.trim().toLowerCase();
    if (q) {
      const isIdSearch = q.startsWith('#');
      const targetId = isIdSearch ? parseInt(q.slice(1)) : NaN;
      
      list = list.map((gm) => ({
        ...gm,
        epics: gm.epics.map((ge) => ({
          ...ge,
          issues: ge.issues.filter((i) =>
            isIdSearch ? i.id === targetId : i.title.toLowerCase().includes(q)
          )
        })).filter((ge) => ge.issues.length > 0)
      })).filter((gm) => gm.epics.length > 0);
    }
    
    return list;
  }, [groupedMissions, selectedMissionIds, selectedEpicIds, debouncedQuery]);

  return filteredGroupedMissions;
}
