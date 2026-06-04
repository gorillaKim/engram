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
  selectedStatuses?: string[];
  selectedPriorities?: string[];
  selectedAgents?: string[];
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
  selectedStatuses = [],
  selectedPriorities = [],
  selectedAgents = [],
}: UseGroupedMissionsParams): GroupedMission[] {
  // 3단계 계층형 가공 (Mission -> Epic -> Issue)
  // 핵심: 에픽/미션 구조는 전체 이슈(unfiltered)로 결정하고,
  //       각 에픽 내 표시 이슈만 필터링한다.
  //       이전에는 필터링된 이슈로 역추적해서 에픽 멤버십을 결정했기 때문에,
  //       모든 이슈가 finished/cancelled인 에픽→미션→프로젝트가 통째로 사라졌다.
  const groupedMissions = useMemo(() => {
    let filteredIssues = filterFinishedIssues(issuesInView, hideFinished);
    
    // 상태 필터 적용
    if (selectedStatuses.length > 0) {
      filteredIssues = filteredIssues.filter((i) => selectedStatuses.includes(i.status));
    }
    // 우선순위 필터 적용
    if (selectedPriorities.length > 0) {
      filteredIssues = filteredIssues.filter((i) => selectedPriorities.includes(i.priority));
    }
    // 담당 에이전트 필터 적용
    if (selectedAgents.length > 0) {
      filteredIssues = filteredIssues.filter((i) => {
        const agent = i.assigned_agent ?? 'unassigned';
        return selectedAgents.includes(agent);
      });
    }
    
    // 1. 전체 이슈로 에픽 멤버십 결정 (필터와 무관하게 에픽 구조 유지)
    const allIssueEpicIds = new Set(issuesInView.map(i => i.epic_id));
    
    // 2. 필터링된 이슈를 epic_id별로 그룹핑 (실제 표시용)
    const filteredIssuesByEpic = new Map<number, Issue[]>();
    for (const issue of filteredIssues) {
      const list = filteredIssuesByEpic.get(issue.epic_id) ?? [];
      list.push(issue);
      filteredIssuesByEpic.set(issue.epic_id, list);
    }
    
    // 3. mission_id -> GroupedEpic[] mapping (이슈가 존재하는 모든 에픽 포함)
    const epicsByMission = new Map<number | null, GroupedEpic[]>();
    
    for (const epicId of allIssueEpicIds) {
      const epic = allEpics.find((e) => e.id === epicId);
      if (!epic) continue;
      
      // 완료/취소 에픽 숨기기 필터링
      if (hideFinishedEpics && (epic.status === 'completed' || epic.status === 'cancelled')) {
        continue;
      }

      const missionId = epic.mission_id ?? null;
      const issues = filteredIssuesByEpic.get(epicId) ?? [];
      const list = epicsByMission.get(missionId) ?? [];
      list.push({ epic, issues });
      epicsByMission.set(missionId, list);
    }
    
    const result: GroupedMission[] = [];
    
    // 4. 미션 목록 기준으로 GroupedMission 빌드
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
    
    // 5. mission_id가 없거나 missions 목록에 없지만 leftover인 에픽들을 "미분류"로 수집
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
  }, [
    issuesInView,
    allEpics,
    missions,
    hideFinished,
    hideFinishedEpics,
    selectedStatuses,
    selectedPriorities,
    selectedAgents,
  ]);

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
