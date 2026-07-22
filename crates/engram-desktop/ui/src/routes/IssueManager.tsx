import { useState, useEffect, useMemo } from 'react';
import { useDebounce } from '../hooks/useDebounce';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import {
  sprintList, sprintUpdate, sprintDelete,
  epicList,
  issueList,
  missionList,
  epicUpdate,
  issueSetStatus,
  issueSetPriority,
} from '../ipc/invoke';
import { useUIStore } from '../store/ui';
import type { Sprint, Epic, Issue, Mission } from '../ipc/types';
import { toggleAllEpics } from '../utils/epicHelper';
import { getUnfinishedIssuesForEpic } from '../utils/sprintCompleteHelper';
import { ConfirmBulkActionModal } from '../components/ConfirmBulkActionModal';
import { MissionHierarchy } from '../components/MissionHierarchy';
import { BulkActionBar } from '../components/BulkActionBar';
import { SprintSidebar } from '../components/SprintSidebar';
import { IssueFilterPanel } from '../components/IssueFilterPanel';
import { IssueManagerModals } from '../components/IssueManagerModals';
import { useGroupedMissions } from '../hooks/useGroupedMissions';

const BACKLOG_ID = 0;

export function IssueManager() {
  const { selectedSprintId, selectSprint, selectIssue, setView, selectEpic, selectMission } = useUIStore();
  const qc = useQueryClient();

  const handleSelectSprint = (id: number | null) => {
    setSearchQuery('');
    setSelectedMissionIds([]);
    setSelectedEpicIds([]);
    setSelectedStatuses([]);
    setSelectedPriorities([]);
    setSelectedAgents([]);
    setFilterOpen(false);
    setBulkSelectedEpics(new Set());
    selectSprint(id);
  };

  // 미션 접기/펼치기 맵 상태
  const [missionExpandedMap, setMissionExpandedMap] = useState<Record<string, boolean>>({});
  const toggleMission = (key: string) => {
    setMissionExpandedMap((prev) => ({ ...prev, [key]: !prev[key] }));
  };

  // 에픽 접기/펼치기 맵 상태
  const [epicExpandedMap, setEpicExpandedMap] = useState<Record<number, boolean>>({});

  // 에픽 일괄 선택 상태
  const [bulkSelectedEpics, setBulkSelectedEpics] = useState<Set<number>>(new Set());

  // 완료된 이슈/에픽 숨기기 토글 상태
  const [hideFinished, setHideFinished] = useState(true);
  const [hideFinishedEpics, setHideFinishedEpics] = useState(true);

  // URL 쿼리 파라미터 분석용 헬퍼
  const parseCsvParam = (key: string): string[] => {
    const params = new URLSearchParams(window.location.search);
    const val = params.get(key);
    return val ? val.split(',').filter(Boolean) : [];
  };

  const parseCsvIntParam = (key: string): number[] => {
    const params = new URLSearchParams(window.location.search);
    const val = params.get(key);
    return val ? val.split(',').map(Number).filter(n => !isNaN(n)) : [];
  };

  // 미션 / 에픽 / 상태 / 우선순위 / 담당자 필터 상태 (URL 쿼리 파라미터 연동)
  const [selectedMissionIds, setSelectedMissionIds] = useState<number[]>(() => parseCsvIntParam('missions'));
  const [selectedEpicIds, setSelectedEpicIds] = useState<number[]>(() => parseCsvIntParam('epics'));
  const [selectedStatuses, setSelectedStatuses] = useState<string[]>(() => parseCsvParam('statuses'));
  const [selectedPriorities, setSelectedPriorities] = useState<string[]>(() => parseCsvParam('priorities'));
  const [selectedAgents, setSelectedAgents] = useState<string[]>(() => parseCsvParam('agents'));
  
  // 쿼리 파라미터에 필터링 조건이 지정되어 있다면 기본적으로 필터 영역을 확장하여 노출
  const [filterOpen, setFilterOpen] = useState(() => {
    const params = new URLSearchParams(window.location.search);
    return (
      params.has('missions') ||
      params.has('epics') ||
      params.has('statuses') ||
      params.has('priorities') ||
      params.has('agents')
    );
  });

  // 모달 상태
  const [sprintModalOpen, setSprintModalOpen] = useState(false);
  const [missionModalOpen, setMissionModalOpen] = useState(false);
  const [epicModalOpen, setEpicModalOpen] = useState(false);
  const [editMission, setEditMission] = useState<Mission | null>(null);
  const [issueModalEpicId, setIssueModalEpicId] = useState<number | null>(null);
  const [editEpic, setEditEpic] = useState<Epic | null>(null);
  const [editSprint, setEditSprint] = useState<Sprint | null>(null);
  const [completeSprintTarget, setCompleteSprintTarget] = useState<Sprint | null>(null);
  const [bulkActionTarget, setBulkActionTarget] = useState<{
    type: 'epic' | 'mission';
    id: number;
    items: { id: number; title: string }[];
  } | null>(null);
  
  const [searchQuery, setSearchQuery] = useState(() => {
    const params = new URLSearchParams(window.location.search);
    return params.get('q') || '';
  });
  const debouncedQuery = useDebounce(searchQuery);

  // 1. Queries
  const { data: sprints = [] } = useQuery<Sprint[]>({
    queryKey: ['sprintList'],
    queryFn: sprintList,
    refetchInterval: 30_000,
  });

  const isBacklog = selectedSprintId === BACKLOG_ID;

  useEffect(() => {
    if (sprints.length === 0) return;
    if (selectedSprintId === BACKLOG_ID) return;
    if (selectedSprintId != null && sprints.some((s) => s.id === selectedSprintId)) return;
    const active = sprints.find((s) => s.status === 'active') ?? sprints[0];
    selectSprint(active.id);
  }, [sprints, selectedSprintId, selectSprint]);

  // URL에서 초기 sprint 읽어와 동기화
  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const val = params.get('sprint');
    if (val === 'backlog') {
      selectSprint(BACKLOG_ID);
    } else if (val !== null) {
      const num = Number(val);
      if (!isNaN(num)) {
        selectSprint(num);
      }
    }
  }, [selectSprint]);

  // 필터 상태 변경 시 URL 동기화
  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    params.set('view', 'issues');

    if (selectedSprintId === BACKLOG_ID) {
      params.set('sprint', 'backlog');
    } else if (selectedSprintId !== null) {
      params.set('sprint', String(selectedSprintId));
    } else {
      params.delete('sprint');
    }

    if (selectedMissionIds.length > 0) {
      params.set('missions', selectedMissionIds.join(','));
    } else {
      params.delete('missions');
    }

    if (selectedEpicIds.length > 0) {
      params.set('epics', selectedEpicIds.join(','));
    } else {
      params.delete('epics');
    }

    if (selectedStatuses.length > 0) {
      params.set('statuses', selectedStatuses.join(','));
    } else {
      params.delete('statuses');
    }

    if (selectedPriorities.length > 0) {
      params.set('priorities', selectedPriorities.join(','));
    } else {
      params.delete('priorities');
    }

    if (selectedAgents.length > 0) {
      params.set('agents', selectedAgents.join(','));
    } else {
      params.delete('agents');
    }

    if (searchQuery) {
      params.set('q', searchQuery);
    } else {
      params.delete('q');
    }

    const newSearch = params.toString();
    const newUrl = `${window.location.pathname}${newSearch ? '?' + newSearch : ''}`;
    window.history.replaceState(null, '', newUrl);
  }, [
    selectedSprintId,
    selectedMissionIds,
    selectedEpicIds,
    selectedStatuses,
    selectedPriorities,
    selectedAgents,
    searchQuery
  ]);

  const { data: issuesInView = [], isLoading: issuesLoading } = useQuery<Issue[]>({
    queryKey: ['issueList', isBacklog ? 'backlog' : selectedSprintId],
    queryFn: () => isBacklog
      ? issueList({ backlog_only: true } as any)
      : issueList({ sprint_id: selectedSprintId } as any),
    enabled: selectedSprintId != null,
  });

  const { data: backlogIssues = [] } = useQuery<Issue[]>({
    queryKey: ['issueList', 'backlog'],
    queryFn: () => issueList({ backlog_only: true } as any),
    refetchInterval: 30_000,
  });

  const { data: allEpics = [], isLoading: epicsLoading } = useQuery<Epic[]>({
    queryKey: ['epicList'],
    queryFn: () => epicList(undefined, true),
    refetchInterval: 30_000,
  });

  const { data: missions = [], isLoading: missionsLoading } = useQuery<Mission[]>({
    queryKey: ['missionList'],
    queryFn: () => missionList(true),
  });

  const loading = issuesLoading || epicsLoading || missionsLoading;

  // 2. Mutations
  const bulkUpdateEpics = useMutation({
    mutationFn: async ({
      epicIds,
      sprintId,
      status,
    }: {
      epicIds: number[];
      sprintId?: number | null | undefined;
      status?: 'active' | 'completed' | 'cancelled' | undefined;
    }) => {
      const promises = epicIds.map((id) =>
        epicUpdate(id, {
          ...(sprintId !== undefined ? { sprint_id: sprintId, update_sprint_id: true } : {}),
          ...(status !== undefined ? { status } : {}),
        })
      );
      return Promise.all(promises);
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['epicList'] });
      qc.invalidateQueries({ queryKey: ['issueList'] });
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      toast.success('선택한 에픽들이 변경되었습니다');
      setBulkSelectedEpics(new Set());
    },
    onError: (e) => toast.error(`일괄 변경 실패: ${e}`),
  });

  const updateIssueStatus = useMutation({
    mutationFn: ({ id, status }: { id: number; status: string }) => issueSetStatus(id, status),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['issueList'] });
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      toast.success('이슈 상태가 변경되었습니다');
    },
    onError: (e) => toast.error(`상태 변경 실패: ${e}`),
  });

  const updateIssuePriority = useMutation({
    mutationFn: ({ id, priority }: { id: number; priority: string }) => issueSetPriority(id, priority),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['issueList'] });
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      toast.success('이슈 우선순위가 변경되었습니다');
    },
    onError: (e) => toast.error(`우선순위 변경 실패: ${e}`),
  });

  const bulkCompleteIssues = useMutation({
    mutationFn: async (issueIds: number[]) => {
      const promises = issueIds.map(id => issueSetStatus(id, 'finished'));
      return Promise.all(promises);
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['issueList'] });
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      toast.success('에픽 하위의 모든 미완료 이슈가 완료되었습니다');
      setBulkActionTarget(null);
    },
    onError: (e) => toast.error(`이슈 일괄 완료 실패: ${e}`),
  });

  const bulkCompleteMissionEpics = useMutation({
    mutationFn: async (epicIds: number[]) => {
      const promises = epicIds.map(id => epicUpdate(id, { status: 'completed' }));
      return Promise.all(promises);
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['epicList'] });
      qc.invalidateQueries({ queryKey: ['issueList'] });
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      toast.success('미션 하위의 모든 에픽이 완료되었습니다');
      setBulkActionTarget(null);
    },
    onError: (e) => toast.error(`에픽 일괄 완료 실패: ${e}`),
  });

  const handleConfirmBulkAction = () => {
    if (!bulkActionTarget) return;
    const ids = bulkActionTarget.items.map(item => item.id);
    if (bulkActionTarget.type === 'epic') {
      bulkCompleteIssues.mutate(ids);
    } else {
      bulkCompleteMissionEpics.mutate(ids);
    }
  };

  const handleTriggerMissionEpicsComplete = (missionId: number) => {
    const targetEpics = allEpics.filter(
      e => e.mission_id === missionId && e.status !== 'completed' && e.status !== 'cancelled'
    );
    if (targetEpics.length === 0) {
      toast.info('완료 처리할 에픽이 없습니다.');
      return;
    }
    setBulkActionTarget({
      type: 'mission',
      id: missionId,
      items: targetEpics.map(e => ({ id: e.id, title: e.title })),
    });
  };

  const handleTriggerEpicIssuesComplete = (epicId: number) => {
    const unfinished = getUnfinishedIssuesForEpic(epicId, issuesInView);
    if (unfinished.length === 0) {
      toast.info('완료 처리할 이슈가 없습니다.');
      return;
    }
    setBulkActionTarget({
      type: 'epic',
      id: epicId,
      items: unfinished.map(i => ({ id: i.id, title: i.title })),
    });
  };

  const activateSprint = useMutation({
    mutationFn: (id: number) => sprintUpdate(id, 'active'),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['sprintList'] });
      qc.invalidateQueries({ queryKey: ['sprintCurrent'] });
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      toast.success('스프린트가 활성화되었습니다');
    },
    onError: (e) => toast.error(`활성화 실패: ${e}`),
  });

  const deleteSprint = useMutation({
    mutationFn: (id: number) => sprintDelete(id),
    onSuccess: (_, deletedId) => {
      if (selectedSprintId === deletedId) selectSprint(null);
      qc.invalidateQueries({ queryKey: ['sprintList'] });
      qc.invalidateQueries({ queryKey: ['sprintCurrent'] });
      qc.invalidateQueries({ queryKey: ['issueList'] });
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      toast.success('스프린트가 삭제되었습니다');
    },
    onError: (e) => toast.error(`삭제 실패: ${e}`),
  });

  const availableAgents = useMemo(() => {
    const agents = new Set<string>();
    let hasUnassigned = false;
    for (const issue of issuesInView) {
      if (issue.assigned_agent) {
        agents.add(issue.assigned_agent);
      } else {
        hasUnassigned = true;
      }
    }
    const list = Array.from(agents).sort();
    if (hasUnassigned) {
      list.push('unassigned');
    }
    return list;
  }, [issuesInView]);

  // 3. Custom Hook (3단계 계층형 가공)
  const filteredGroupedMissions = useGroupedMissions({
    issuesInView,
    allEpics,
    missions,
    hideFinished,
    hideFinishedEpics,
    selectedMissionIds,
    selectedEpicIds,
    debouncedQuery,
    selectedStatuses,
    selectedPriorities,
    selectedAgents,
  });



  // 해당 스프린트에 속한 미션들만 필터링
  const filteredMissionsForFilter = useMemo(() => {
    if (selectedSprintId === null) return [];
    if (selectedSprintId === BACKLOG_ID) {
      return missions.filter((m) => m.sprint_id === null);
    }
    return missions.filter((m) => m.sprint_id === selectedSprintId);
  }, [missions, selectedSprintId]);

  // 현재 스프린트에 속한 에픽들만 필터링
  const sprintEpics = useMemo(() => {
    if (selectedSprintId === null) return [];
    const sprintMissionIds = new Set(
      missions
        .filter((m) => selectedSprintId === BACKLOG_ID ? m.sprint_id === null : m.sprint_id === selectedSprintId)
        .map((m) => m.id)
    );
    const issueEpicIds = new Set(issuesInView.map((i) => i.epic_id));
    return allEpics.filter((epic) => {
      if (epic.mission_id !== null && sprintMissionIds.has(epic.mission_id)) return true;
      if (issueEpicIds.has(epic.id)) return true;
      return false;
    });
  }, [allEpics, missions, issuesInView, selectedSprintId]);

  // 필터 패널에서 사용할 에픽 목록
  const filterAvailableEpics = useMemo(() => {
    if (selectedMissionIds.length === 0) return sprintEpics;
    return sprintEpics.filter((epic) => {
      if (epic.mission_id === null) return selectedMissionIds.includes(0);
      return selectedMissionIds.includes(epic.mission_id);
    });
  }, [sprintEpics, selectedMissionIds]);

  // 미션 선택 변경 시 에픽 필터 초기화
  useEffect(() => {
    setSelectedEpicIds([]);
  }, [selectedMissionIds]);

  const selectedSprint = isBacklog ? null : sprints.find((s) => s.id === selectedSprintId);

  return (
    <div className="flex h-full overflow-hidden">
      {/* Sprint Sidebar component */}
      <SprintSidebar
        sprints={sprints}
        backlogCount={backlogIssues.length}
        selectedSprintId={selectedSprintId}
        onSelectSprint={handleSelectSprint}
        onActivateSprint={(id) => activateSprint.mutate(id)}
        onCompleteSprint={setCompleteSprintTarget}
        onDeleteSprint={(id) => deleteSprint.mutate(id)}
        onEditSprint={setEditSprint}
        onAddSprint={() => setSprintModalOpen(true)}
      >

      </SprintSidebar>

      {/* Main content */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-3 border-b border-slate-200 flex-shrink-0">
          <div>
            <h2 className="text-base font-semibold text-slate-800">
              {isBacklog ? '백로그' : (selectedSprint ? selectedSprint.name : '스프린트를 선택하세요')}
            </h2>
            {isBacklog && (
              <p className="text-xs text-slate-400 mt-0.5">스프린트에 아직 들어가지 않은 이슈 모음</p>
            )}
            {!isBacklog && selectedSprint?.goal && (
              <p className="text-xs text-slate-400 mt-0.5">{selectedSprint.goal}</p>
            )}
          </div>
          <div className="flex items-center gap-1.5 sm:gap-2 min-w-0 flex-nowrap">
            <label className="flex items-center gap-1.5 text-xs text-slate-500 font-semibold cursor-pointer bg-slate-100 px-2 sm:px-2.5 py-1.5 rounded-lg border border-slate-200 hover:bg-slate-200/50 transition-all select-none shrink min-w-0 max-w-[130px] md:max-w-none">
              <input
                type="checkbox"
                checked={hideFinished}
                onChange={(e) => setHideFinished(e.target.checked)}
                className="rounded text-indigo-600 focus:ring-indigo-500/20 border-slate-300 w-3.5 h-3.5 flex-shrink-0"
              />
              <span className="truncate min-w-0 whitespace-nowrap">완료된 이슈 숨기기</span>
            </label>

            <label className="flex items-center gap-1.5 text-xs text-slate-500 font-semibold cursor-pointer bg-slate-100 px-2 sm:px-2.5 py-1.5 rounded-lg border border-slate-200 hover:bg-slate-200/50 transition-all select-none shrink min-w-0 max-w-[130px] md:max-w-none">
              <input
                type="checkbox"
                checked={hideFinishedEpics}
                onChange={(e) => setHideFinishedEpics(e.target.checked)}
                className="rounded text-indigo-600 focus:ring-indigo-500/20 border-slate-300 w-3.5 h-3.5 flex-shrink-0"
              />
              <span className="truncate min-w-0 whitespace-nowrap">완료된 에픽 숨기기</span>
            </label>

            <div className="flex items-center gap-0.5 bg-slate-100 p-0.5 rounded-lg border border-slate-200 shrink min-w-0 whitespace-nowrap">
              <button
                type="button"
                onClick={() => {
                  const epicIds = allEpics.map(e => e.id);
                  setEpicExpandedMap(toggleAllEpics(epicIds, true));
                }}
                className="text-[11px] px-1.5 sm:px-2 py-1.5 text-slate-600 hover:text-slate-900 font-semibold truncate whitespace-nowrap min-w-0 max-w-[90px] md:max-w-none"
                title="모든 에픽 펼치기"
              >
                ▼ 모두 펼치기
              </button>
              <span className="w-px h-3 bg-slate-200 flex-shrink-0" />
              <button
                type="button"
                onClick={() => {
                  const epicIds = allEpics.map(e => e.id);
                  setEpicExpandedMap(toggleAllEpics(epicIds, false));
                }}
                className="text-[11px] px-1.5 sm:px-2 py-1.5 text-slate-600 hover:text-slate-900 font-semibold truncate whitespace-nowrap min-w-0 max-w-[90px] md:max-w-none"
                title="모든 에픽 접기"
              >
                ▶ 모두 접기
              </button>
            </div>

            <button
              type="button"
              onClick={() => setFilterOpen((v) => !v)}
              className={`flex items-center gap-1.5 text-xs font-semibold px-2 sm:px-2.5 py-1.5 rounded-lg border transition-all select-none hover:bg-slate-200/50 shrink-0 ${
                filterOpen
                  ? 'bg-indigo-50 border-indigo-200 text-indigo-700 hover:bg-indigo-100/50'
                  : 'bg-slate-100 border-slate-200 text-slate-600'
              }`}
            >
              <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor" className="w-3.5 h-3.5">
                <path strokeLinecap="round" strokeLinejoin="round" d="M12 3c2.755 0 5.455.232 8.083.678.533.09.917.556.917 1.096v3.059a2.25 2.25 0 0 1-.659 1.591l-5.432 5.432a2.25 2.25 0 0 0-.659 1.591v2.927a2.25 2.25 0 0 1-1.24 2.013L9.75 21v-6.568a2.25 2.25 0 0 0-.659-1.591L3.659 7.409A2.25 2.25 0 0 1 3 5.818V4.774c0-.54.384-1.006.917-1.096A48.32 48.32 0 0 1 12 3Z" />
              </svg>
              <span>필터</span>
              {(selectedMissionIds.length > 0 || selectedEpicIds.length > 0 || selectedStatuses.length > 0 || selectedPriorities.length > 0 || selectedAgents.length > 0) && (
                <span className="bg-indigo-500 text-white text-[9px] rounded-full px-1.5 py-0.2 font-bold leading-normal">
                  {selectedMissionIds.length + selectedEpicIds.length + selectedStatuses.length + selectedPriorities.length + selectedAgents.length}
                </span>
              )}
            </button>

            <input
              type="text"
              placeholder="#ID 또는 이슈 검색…"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="text-xs sm:text-sm border border-slate-200 rounded-lg px-2 py-1.5 bg-white focus:outline-none focus:ring-2 focus:ring-indigo-500/20 min-w-[70px] flex-1 sm:flex-initial sm:min-w-[120px] md:min-w-[180px] max-w-[180px]"
            />
            {/* 추가 액션 버튼들에 아이콘 도입 및 디자인 보강 (#365) */}
            <button
              type="button"
              onClick={() => {
                setEditMission(null);
                setMissionModalOpen(true);
              }}
              className="text-xs sm:text-sm px-2 sm:px-2.5 py-1.5 bg-violet-100 hover:bg-violet-200 text-violet-700 rounded-md shrink min-w-0 max-w-[80px] md:max-w-none whitespace-nowrap flex items-center justify-center gap-1 transition-all hover:scale-105 active:scale-95"
            >
              <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={2.5} stroke="currentColor" className="w-3.5 h-3.5">
                <path strokeLinecap="round" strokeLinejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
              </svg>
              <span className="truncate min-w-0 font-semibold">새 미션</span>
            </button>
            <button
              type="button"
              onClick={() => setEpicModalOpen(true)}
              className="text-xs sm:text-sm px-2 sm:px-2.5 py-1.5 bg-slate-200 hover:bg-slate-300 text-slate-700 rounded-md shrink min-w-0 max-w-[80px] md:max-w-none whitespace-nowrap flex items-center justify-center gap-1 transition-all hover:scale-105 active:scale-95"
            >
              <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={2.5} stroke="currentColor" className="w-3.5 h-3.5">
                <path strokeLinecap="round" strokeLinejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
              </svg>
              <span className="truncate min-w-0 font-semibold">새 에픽</span>
            </button>
            {allEpics.length > 0 && (
              <button
                type="button"
                onClick={() => setIssueModalEpicId(allEpics[0].id)}
                className="text-xs sm:text-sm px-2 sm:px-2.5 py-1.5 bg-indigo-600 hover:bg-indigo-500 text-white rounded-md shrink min-w-0 max-w-[80px] md:max-w-none whitespace-nowrap flex items-center justify-center gap-1 transition-all hover:scale-105 active:scale-95"
              >
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={2.5} stroke="currentColor" className="w-3.5 h-3.5">
                  <path strokeLinecap="round" strokeLinejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
                </svg>
                <span className="truncate min-w-0 font-semibold">새 이슈</span>
              </button>
            )}
          </div>
        </div>

        <div
          className={`grid transition-all duration-300 ease-in-out flex-shrink-0 ${
            filterOpen && (filteredMissionsForFilter.length > 0 || filterAvailableEpics.length > 0 || availableAgents.length > 0)
              ? 'grid-rows-[1fr] opacity-100'
              : 'grid-rows-[0fr] opacity-0 pointer-events-none'
          }`}
        >
          <div className="overflow-hidden">
            <IssueFilterPanel
              filteredMissions={filteredMissionsForFilter}
              availableEpics={filterAvailableEpics}
              selectedMissionIds={selectedMissionIds}
              setSelectedMissionIds={setSelectedMissionIds}
              selectedEpicIds={selectedEpicIds}
              setSelectedEpicIds={setSelectedEpicIds}
              selectedStatuses={selectedStatuses}
              setSelectedStatuses={setSelectedStatuses}
              selectedPriorities={selectedPriorities}
              setSelectedPriorities={setSelectedPriorities}
              selectedAgents={selectedAgents}
              setSelectedAgents={setSelectedAgents}
              availableAgents={availableAgents}
              onClose={() => setFilterOpen(false)}
            />
          </div>
        </div>

        {/* Epic + Issue tree */}
        {loading && (
          <div className="flex items-center justify-center h-full text-slate-400 text-sm bg-white">
            이슈 및 미션 데이터를 불러오는 중…
          </div>
        )}
        <div className="flex-1 overflow-y-auto p-6 bg-slate-50/50">
          {selectedSprint && (selectedSprint.status === 'completed' || selectedSprint.status === 'cancelled') && (
            <div className="mb-6 px-4 py-3 bg-indigo-50/80 border border-indigo-100 rounded-xl flex items-center justify-between text-xs text-indigo-700 animate-fade-in">
              <div className="flex items-center gap-2">
                <span className="text-sm">ℹ️</span>
                <span>이 스프린트는 이미 <strong>완료</strong> 또는 <strong>취소</strong>된 스프린트입니다. 상세 리포트는 완료 히스토리에서 보실 수 있습니다.</span>
              </div>
              <button
                type="button"
                onClick={() => setView('history')}
                className="text-xs text-indigo-600 hover:text-indigo-800 font-bold underline cursor-pointer"
              >
                히스토리 보기 →
              </button>
            </div>
          )}

          {selectedSprintId == null && (
            <p className="text-slate-400 text-center mt-20">왼쪽에서 스프린트나 백로그를 선택하세요</p>
          )}
          {!loading && selectedSprintId != null && filteredGroupedMissions.length === 0 && (
            <p className="text-slate-400 text-center mt-20">
              {debouncedQuery.trim()
                ? `"${debouncedQuery.trim()}" 에 일치하는 이슈가 없습니다.`
                : isBacklog
                  ? '백로그가 비어 있습니다. 새 이슈를 백로그로 추가하세요.'
                  : '이슈가 없습니다. "+ 새 이슈" 로 이 스프린트에 이슈를 추가하세요.'}
            </p>
          )}
          {!loading && selectedSprintId != null && (
            <MissionHierarchy
              groupedMissions={filteredGroupedMissions}
              sprints={sprints}
              expandedMissions={missionExpandedMap}
              onToggleMission={toggleMission}
              expandedEpics={epicExpandedMap}
              onToggleEpic={(id) => {
                setEpicExpandedMap(prev => ({
                  ...prev,
                  [id]: !(prev[id] !== false)
                }));
              }}
              onIssueClick={selectIssue}
              readOnly={false}
              showEpicCheckboxes={true}
              bulkSelectedEpics={bulkSelectedEpics}
              onEpicCheck={(epicId, checked) => {
                setBulkSelectedEpics((prev) => {
                  const next = new Set(prev);
                  if (checked) {
                    next.add(epicId);
                  } else {
                    next.delete(epicId);
                  }
                  return next;
                });
              }}
              onEpicEdit={(epic) => selectEpic(epic.id)}
              onMissionEdit={(mission) => selectMission(mission.id)}
              onIssueStatusChange={(id, status) => updateIssueStatus.mutate({ id, status })}
              onIssuePriorityChange={(id, priority) => updateIssuePriority.mutate({ id, priority })}
              renderMissionActions={(mission) => {
                if (!mission) return null;
                return (
                  <button
                    type="button"
                    onClick={() => handleTriggerMissionEpicsComplete(mission.id)}
                    className="text-[11px] px-2 py-1 bg-violet-600 hover:bg-violet-700 text-white rounded font-semibold transition-all hover:scale-105 active:scale-95 flex items-center justify-center gap-0.5"
                    title="미션 내 모든 에픽 완료 처리"
                  >
                    ✓ 에픽 일괄 완료
                  </button>
                );
              }}
              onBulkCompleteIssues={handleTriggerEpicIssuesComplete}
            />
          )}
        </div>
      </div>

      {/* Modals Orchestrator */}
      <IssueManagerModals
        sprintModalOpen={sprintModalOpen}
        setSprintModalOpen={setSprintModalOpen}
        missionModalOpen={missionModalOpen}
        setMissionModalOpen={setMissionModalOpen}
        epicModalOpen={epicModalOpen}
        setEpicModalOpen={setEpicModalOpen}
        editMission={editMission}
        setEditMission={setEditMission}
        issueModalEpicId={issueModalEpicId}
        setIssueModalEpicId={setIssueModalEpicId}
        editEpic={editEpic}
        setEditEpic={setEditEpic}
        editSprint={editSprint}
        setEditSprint={setEditSprint}
        completeSprintTarget={completeSprintTarget}
        setCompleteSprintTarget={setCompleteSprintTarget}
        sprints={sprints}
      />

      {/* Bulk action floating bar */}
      {bulkSelectedEpics.size > 0 && (
        <BulkActionBar
          selectedCount={bulkSelectedEpics.size}
          sprints={sprints}
          onClear={() => setBulkSelectedEpics(new Set())}
          onUpdateSprint={(sprintId) =>
            bulkUpdateEpics.mutate({
              epicIds: Array.from(bulkSelectedEpics),
              sprintId,
            })
          }
          onUpdateStatus={(status) =>
            bulkUpdateEpics.mutate({
              epicIds: Array.from(bulkSelectedEpics),
              status,
            })
          }
          isPending={bulkUpdateEpics.isPending}
        />
      )}

      {bulkActionTarget && (
        <ConfirmBulkActionModal
          isOpen={!!bulkActionTarget}
          onClose={() => setBulkActionTarget(null)}
          onConfirm={handleConfirmBulkAction}
          title={bulkActionTarget.type === 'epic' ? '에픽 하위 이슈 일괄 완료' : '미션 하위 에픽 일괄 완료'}
          description={
            bulkActionTarget.type === 'epic'
              ? '에픽 하위의 다음 미완료 이슈들을 모두 완료(Finished) 처리하시겠습니까?'
              : '미션 하위의 다음 미완료 에픽들을 모두 완료(Completed) 처리하시겠습니까?'
          }
          items={bulkActionTarget.items}
          confirmText="일괄 완료"
          isPending={bulkCompleteIssues.isPending || bulkCompleteMissionEpics.isPending}
        />
      )}
    </div>
  );
}
