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
} from '../ipc/invoke';
import { useUIStore } from '../store/ui';
import type { Sprint, Epic, Issue, Mission } from '../ipc/types';
import { toggleAllEpics } from '../utils/epicHelper';
import { MissionHierarchy } from '../components/MissionHierarchy';
import { BulkActionBar } from '../components/BulkActionBar';
import { SprintSidebar } from '../components/SprintSidebar';
import { IssueFilterPanel } from '../components/IssueFilterPanel';
import { IssueManagerModals } from '../components/IssueManagerModals';
import { useGroupedMissions } from '../hooks/useGroupedMissions';

const BACKLOG_ID = 0;

export function IssueManager() {
  const { selectedSprintId, selectSprint, selectIssue, setView } = useUIStore();
  const qc = useQueryClient();

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

  // 미션 / 에픽 필터 상태
  const [selectedMissionIds, setSelectedMissionIds] = useState<number[]>([]);
  const [selectedEpicIds, setSelectedEpicIds] = useState<number[]>([]);
  const [filterOpen, setFilterOpen] = useState(false);

  // 모달 상태
  const [sprintModalOpen, setSprintModalOpen] = useState(false);
  const [missionModalOpen, setMissionModalOpen] = useState(false);
  const [epicModalOpen, setEpicModalOpen] = useState(false);
  const [editMission, setEditMission] = useState<Mission | null>(null);
  const [issueModalEpicId, setIssueModalEpicId] = useState<number | null>(null);
  const [editEpic, setEditEpic] = useState<Epic | null>(null);
  const [editSprint, setEditSprint] = useState<Sprint | null>(null);
  const [completeSprintTarget, setCompleteSprintTarget] = useState<Sprint | null>(null);
  
  const [searchQuery, setSearchQuery] = useState('');
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
    queryFn: () => missionList(false),
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
  });

  // 스프린트 전환 시 검색어, 필터 초기화
  useEffect(() => {
    setSearchQuery('');
    setSelectedMissionIds([]);
    setSelectedEpicIds([]);
    setFilterOpen(false);
    setBulkSelectedEpics(new Set());
  }, [selectedSprintId]);

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
        onSelectSprint={selectSprint}
        onActivateSprint={(id) => activateSprint.mutate(id)}
        onCompleteSprint={setCompleteSprintTarget}
        onDeleteSprint={(id) => deleteSprint.mutate(id)}
        onEditSprint={setEditSprint}
        onAddSprint={() => setSprintModalOpen(true)}
      >
        {(filteredMissionsForFilter.length > 0 || filterAvailableEpics.length > 0) && (
          <div className="mx-2 mb-2">
            <button
              type="button"
              onClick={() => setFilterOpen((v) => !v)}
              className="w-full flex items-center justify-between px-2 py-1 text-[11px] text-slate-500 hover:text-slate-700 hover:bg-slate-100 rounded-md transition-colors"
            >
              <span className="flex items-center gap-1.5">
                <span>필터</span>
                {(selectedMissionIds.length > 0 || selectedEpicIds.length > 0) && (
                  <span className="bg-indigo-500 text-white text-[9px] rounded-full px-1.5 font-bold">
                    {selectedMissionIds.length + selectedEpicIds.length}
                  </span>
                )}
              </span>
              <span className="text-[9px]">{filterOpen ? '▲' : '▼'}</span>
            </button>
            {filterOpen && (
              <IssueFilterPanel
                filteredMissions={filteredMissionsForFilter}
                availableEpics={filterAvailableEpics}
                selectedMissionIds={selectedMissionIds}
                setSelectedMissionIds={setSelectedMissionIds}
                selectedEpicIds={selectedEpicIds}
                setSelectedEpicIds={setSelectedEpicIds}
              />
            )}
          </div>
        )}
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
              onEpicEdit={setEditEpic}
              onMissionEdit={(mission) => {
                setEditMission(mission);
                setMissionModalOpen(true);
              }}
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
    </div>
  );
}
