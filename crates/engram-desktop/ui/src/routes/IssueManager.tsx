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
import { CreateSprintModal } from '../components/CreateSprintModal';
import { CreateEpicModal } from '../components/CreateEpicModal';
import { CreateIssueModal } from '../components/CreateIssueModal';
import { EditEpicModal } from '../components/EditEpicModal';
import { EditSprintModal } from '../components/EditSprintModal';
import { ConfirmCompleteSprintModal } from '../components/ConfirmCompleteSprintModal';
import { MissionModal } from '../components/MissionModal';
import type { Sprint, Epic, Issue, Mission } from '../ipc/types';
import { clampSidebarWidth } from '../utils/sidebarHelper';
import { toggleAllEpics } from '../utils/epicHelper';
import { filterFinishedIssues } from '../utils/issueFilterHelper';
import { MissionHierarchy } from '../components/MissionHierarchy';
import { BulkActionBar } from '../components/BulkActionBar';
import { StatusBadge } from '../components/StatusBadge';

// ── Sprint sidebar ──────────────────────────────────────────────────────────

/** 사이드바에서 "백로그(스프린트 미지정)" 를 선택했을 때 selectedSprintId 로 사용하는 sentinel */
const BACKLOG_ID = 0;

function BacklogItem({
  selected, onClick, count,
}: {
  selected: boolean;
  onClick: () => void;
  count?: number;
}) {
  return (
    <div
      onClick={onClick}
      className={`p-3 rounded-lg cursor-pointer mb-1 ${selected ? 'bg-indigo-50 border border-indigo-200' : 'hover:bg-slate-50'}`}
    >
      <div className="flex items-center justify-between gap-2">
        <span className="text-xs font-semibold px-2 py-0.5 rounded-full bg-slate-200 text-slate-600">백로그</span>
        {count != null && <span className="text-xs text-slate-400">{count}개</span>}
      </div>
      <p className="text-sm font-medium text-slate-800 mt-1">스프린트 미지정</p>
      <p className="text-xs text-slate-400 mt-0.5">아직 스프린트에 들어가지 않은 이슈</p>
    </div>
  );
}

function SprintItem({
  sprint, selected, onClick, onActivate, onComplete, onDelete, onEdit,
}: {
  sprint: Sprint;
  selected: boolean;
  onClick: () => void;
  onActivate: () => void;
  onComplete: () => void;
  onDelete: () => void;
  onEdit: () => void;
}) {
  const [confirmDelete, setConfirmDelete] = useState(false);

  useEffect(() => {
    if (!confirmDelete) return;
    const t = setTimeout(() => setConfirmDelete(false), 3000);
    return () => clearTimeout(t);
  }, [confirmDelete]);

  return (
    <div
      onClick={onClick}
      className={`p-3 rounded-lg cursor-pointer mb-1 ${selected ? 'bg-indigo-50 border border-indigo-200' : 'hover:bg-slate-50'}`}
    >
      <div className="flex items-center justify-between gap-2">
        <StatusBadge status={sprint.status} type="sprint" variant="ko" />
        <div className="flex items-center gap-1">
          {sprint.status === 'planning' && (
            <button
              type="button"
              onClick={(e) => { e.stopPropagation(); onActivate(); }}
              className="text-xs px-2 py-0.5 bg-indigo-600 hover:bg-indigo-500 text-white rounded"
            >
              활성화
            </button>
          )}
          {sprint.status === 'active' && (
            <button
              type="button"
              onClick={(e) => { e.stopPropagation(); onComplete(); }}
              className="text-xs px-2 py-0.5 bg-green-600 hover:bg-green-500 text-white rounded"
            >
              완료
            </button>
          )}
          <button
            type="button"
            onClick={(e) => { e.stopPropagation(); onEdit(); }}
            title="스프린트 수정"
            className="text-xs px-1.5 py-0.5 text-slate-400 hover:text-slate-700"
          >
            ✎
          </button>
          {confirmDelete ? (
            <button
              type="button"
              onClick={(e) => { e.stopPropagation(); setConfirmDelete(false); onDelete(); }}
              className="text-xs px-2 py-0.5 bg-red-600 hover:bg-red-500 text-white rounded"
            >
              삭제 확인
            </button>
          ) : (
            <button
              type="button"
              onClick={(e) => { e.stopPropagation(); setConfirmDelete(true); }}
              title="스프린트 삭제"
              className="text-xs px-1.5 py-0.5 text-slate-400 hover:text-red-600"
            >
              ✕
            </button>
          )}
        </div>
      </div>
      <p className="text-sm font-medium text-slate-800 mt-1 truncate">{sprint.name}</p>
      {sprint.goal && <p className="text-xs text-slate-400 mt-0.5 truncate">{sprint.goal}</p>}
      {(sprint.start_date || sprint.end_date) && (
        <p className="text-xs text-slate-400 mt-0.5">
          {sprint.start_date ?? '?'} ~ {sprint.end_date ?? '?'}
        </p>
      )}
    </div>
  );
}


interface FilterPanelProps {
  filteredMissions: Mission[];
  availableEpics: Epic[];
  selectedMissionIds: number[];
  setSelectedMissionIds: React.Dispatch<React.SetStateAction<number[]>>;
  selectedEpicIds: number[];
  setSelectedEpicIds: React.Dispatch<React.SetStateAction<number[]>>;
}

function FilterPanel({
  filteredMissions,
  availableEpics,
  selectedMissionIds,
  setSelectedMissionIds,
  selectedEpicIds,
  setSelectedEpicIds,
}: FilterPanelProps) {
  return (
    <div className="mt-1 px-2 py-2 bg-slate-50 rounded-lg border border-slate-100 flex flex-col gap-2">
      {(filteredMissions.length > 0 || selectedMissionIds.length > 0) && (
        <div>
          <p className="text-[10px] font-semibold text-slate-400 uppercase tracking-wider mb-1">미션</p>
          <div className="flex flex-wrap gap-1">
            <button
              type="button"
              onClick={() => setSelectedMissionIds([])}
              className={`text-[11px] px-2 py-0.5 rounded-full border transition-colors ${
                selectedMissionIds.length === 0
                  ? 'bg-violet-100 text-violet-700 border-violet-300 font-medium'
                  : 'bg-white text-slate-500 border-slate-200 hover:border-violet-200'
              }`}
            >
              전체
            </button>
            <button
              type="button"
              onClick={() =>
                setSelectedMissionIds((prev) =>
                  prev.includes(0) ? prev.filter((id) => id !== 0) : [...prev, 0]
                )
              }
              className={`text-[11px] px-2 py-0.5 rounded-full border transition-colors ${
                selectedMissionIds.includes(0)
                  ? 'bg-violet-100 text-violet-700 border-violet-300 font-medium'
                  : 'bg-white text-slate-500 border-slate-200 hover:border-violet-200'
              }`}
            >
              미분류
            </button>
            {filteredMissions.map((m) => (
              <button
                key={m.id}
                type="button"
                title={m.title}
                onClick={() =>
                  setSelectedMissionIds((prev) =>
                    prev.includes(m.id) ? prev.filter((id) => id !== m.id) : [...prev, m.id]
                  )
                }
                className={`text-[11px] px-2 py-0.5 rounded-full border transition-colors max-w-[120px] truncate ${
                  selectedMissionIds.includes(m.id)
                    ? 'bg-violet-100 text-violet-700 border-violet-300 font-medium'
                    : 'bg-white text-slate-500 border-slate-200 hover:border-violet-200'
                }`}
              >
                {m.title}
              </button>
            ))}
          </div>
        </div>
      )}
      {availableEpics.length > 0 && (
        <div>
          <p className="text-[10px] font-semibold text-slate-400 uppercase tracking-wider mb-1">에픽</p>
          <div className="flex flex-wrap gap-1">
            {availableEpics.map((epic) => (
              <button
                key={epic.id}
                type="button"
                title={epic.title}
                onClick={() =>
                  setSelectedEpicIds((prev) =>
                    prev.includes(epic.id) ? prev.filter((id) => id !== epic.id) : [...prev, epic.id]
                  )
                }
                className={`text-[11px] px-2 py-0.5 rounded-full border transition-colors max-w-[130px] truncate ${
                  selectedEpicIds.includes(epic.id)
                    ? 'bg-indigo-100 text-indigo-700 border-indigo-300 font-medium'
                    : 'bg-white text-slate-500 border-slate-200 hover:border-indigo-200'
                }`}
              >
                {epic.title}
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

// ── Main ────────────────────────────────────────────────────────────────────

export function IssueManager() {
  const { selectedSprintId, selectSprint, selectIssue, setView } = useUIStore();
  const qc = useQueryClient();

  // 사이드바 가로 조절 상태
  const [sidebarWidth, setSidebarWidth] = useState<number>(() => {
    const saved = localStorage.getItem('engram_sidebar_width');
    return saved ? parseInt(saved, 10) : 224;
  });

  // 미션 접기/펼치기 맵 상태
  const [missionExpandedMap, setMissionExpandedMap] = useState<Record<string, boolean>>({});
  const toggleMission = (key: string) => {
    setMissionExpandedMap((prev) => ({ ...prev, [key]: !prev[key] }));
  };

  // 에픽 접기/펼치기 맵 상태
  const [epicExpandedMap, setEpicExpandedMap] = useState<Record<number, boolean>>({});

  // 에픽 일괄 선택 상태
  const [bulkSelectedEpics, setBulkSelectedEpics] = useState<Set<number>>(new Set());

  // 에픽 일괄 변경 Mutation
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

  // 완료된 이슈 숨기기 토글 상태 (기본값 true)
  const [hideFinished, setHideFinished] = useState(true);
  // 완료된 에픽 숨기기 토글 상태 (기본값 true)
  const [hideFinishedEpics, setHideFinishedEpics] = useState(true);

  // 완료된 스프린트 아코디언 상태
  const [showPastSprints, setShowPastSprints] = useState(false);

  // 미션 필터 상태 — 비어 있으면 전체 선택
  const [selectedMissionIds, setSelectedMissionIds] = useState<number[]>([]);

  // 에픽 필터 상태 — 비어 있으면 전체 선택
  const [selectedEpicIds, setSelectedEpicIds] = useState<number[]>([]);

  // 필터 패널 접기/펼치기 — 기본 닫힘
  const [filterOpen, setFilterOpen] = useState(false);

  const handleMouseDown = (e: React.MouseEvent) => {
    e.preventDefault();
    const startX = e.clientX;
    const startWidth = sidebarWidth;

    const handleMouseMove = (moveEvent: MouseEvent) => {
      const deltaX = moveEvent.clientX - startX;
      setSidebarWidth(clampSidebarWidth(startWidth + deltaX));
    };

    const handleMouseUp = () => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
    };

    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
  };

  useEffect(() => {
    localStorage.setItem('engram_sidebar_width', sidebarWidth.toString());
  }, [sidebarWidth]);

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

  // 현재 뷰의 이슈 — 스프린트 선택 시 sprint_id 필터, 백로그 선택 시 backlog_only.
  const { data: issuesInView = [], isLoading: issuesLoading } = useQuery<Issue[]>({
    queryKey: ['issueList', isBacklog ? 'backlog' : selectedSprintId],
    queryFn: () => isBacklog
      ? issueList({ backlog_only: true } as any)
      : issueList({ sprint_id: selectedSprintId } as any),
    enabled: selectedSprintId != null,
  });

  // 백로그 카운트 (사이드바 표시용) — 항상 별도 쿼리.
  const { data: backlogIssues = [] } = useQuery<Issue[]>({
    queryKey: ['issueList', 'backlog'],
    queryFn: () => issueList({ backlog_only: true } as any),
    refetchInterval: 30_000,
  });

  // 모든 에픽 (완료/취소 포함 조회용 lookup).
  const { data: allEpics = [], isLoading: epicsLoading } = useQuery<Epic[]>({
    queryKey: ['epicList'],
    queryFn: () => epicList(undefined, true),
    refetchInterval: 30_000,
  });

  // 전체 활성 미션 목록 (sprint_id 무관).
  const { data: missions = [], isLoading: missionsLoading } = useQuery<Mission[]>({
    queryKey: ['missionList'],
    queryFn: () => missionList(false),
  });

  const loading = issuesLoading || epicsLoading || missionsLoading;

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
    const epicsByMission = new Map<number | null, { epic: Epic; issues: Issue[] }[]>();
    
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
    
    const result: { mission: Mission | null; epics: { epic: Epic; issues: Issue[] }[] }[] = [];
    
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
    const leftoverEpics: { epic: Epic; issues: Issue[] }[] = [];
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

  // 스프린트 전환 시 검색어, 선택 이슈, 미션/에픽 필터 초기화 + 필터 패널 닫기 + 일괄 선택 상태 초기화
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

    // 1. 현재 스프린트에 속한 미션의 ID들
    const sprintMissionIds = new Set(
      missions
        .filter((m) => {
          if (selectedSprintId === BACKLOG_ID) {
            return m.sprint_id === null;
          }
          return m.sprint_id === selectedSprintId;
        })
        .map((m) => m.id)
    );

    // 2. 현재 스프린트 내 이슈들이 속한 에픽 ID들 (완료 여부 무관)
    const issueEpicIds = new Set(issuesInView.map((i) => i.epic_id));

    // 3. 미션 소속이 해당 스프린트이거나, 해당 스프린트 내 이슈가 있는 에픽들
    return allEpics.filter((epic) => {
      if (epic.mission_id !== null && sprintMissionIds.has(epic.mission_id)) {
        return true;
      }
      if (issueEpicIds.has(epic.id)) {
        return true;
      }
      return false;
    });
  }, [allEpics, missions, issuesInView, selectedSprintId]);

  // 필터 패널에서 사용할 에픽 목록 수집용 헬퍼 (미션 필터에 따라 동적으로 에픽 목록을 채우기 위함)
  const filterAvailableEpics = useMemo(() => {
    if (selectedMissionIds.length === 0) {
      return sprintEpics;
    }
    return sprintEpics.filter((epic) => {
      if (epic.mission_id === null) {
        return selectedMissionIds.includes(0);
      }
      return selectedMissionIds.includes(epic.mission_id);
    });
  }, [sprintEpics, selectedMissionIds]);

  // 미션 선택 변경 시 에픽 필터 초기화
  useEffect(() => {
    setSelectedEpicIds([]);
  }, [selectedMissionIds]);

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

  const selectedSprint = isBacklog ? null : sprints.find((s) => s.id === selectedSprintId);

  const activeSprints = useMemo(() => sprints.filter(s => s.status !== 'completed' && s.status !== 'cancelled'), [sprints]);
  const pastSprints = useMemo(() => sprints.filter(s => s.status === 'completed' || s.status === 'cancelled'), [sprints]);

  return (
    <div className="flex h-full overflow-hidden">
      {/* ── Sprint sidebar ── */}
      <div 
        className="flex-shrink-0 border-r border-slate-200 flex flex-col bg-slate-50"
        style={{ width: sidebarWidth }}
      >
        <div className="flex items-center justify-between px-4 py-3 border-b border-slate-200">
          <span className="text-xs font-semibold text-slate-500 uppercase tracking-wider">스프린트</span>
          <button
            type="button"
            onClick={() => setSprintModalOpen(true)}
            className="text-xs px-2 py-1 bg-slate-200 hover:bg-slate-300 text-slate-700 rounded"
          >
            + 추가
          </button>
        </div>

        <div className="flex-1 overflow-y-auto p-2">
          <BacklogItem
            selected={isBacklog}
            onClick={() => selectSprint(BACKLOG_ID)}
            count={backlogIssues.length}
          />
          {isBacklog && (filteredMissionsForFilter.length > 0 || filterAvailableEpics.length > 0) && (
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
                <FilterPanel
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
          {activeSprints.length === 0 && (
            <p className="text-xs text-slate-400 text-center mt-4">활성 스프린트가 없습니다</p>
          )}
          {activeSprints.map((sprint) => (
            <div key={sprint.id}>
              <SprintItem
                sprint={sprint}
                selected={sprint.id === selectedSprintId}
                onClick={() => selectSprint(sprint.id)}
                onActivate={() => activateSprint.mutate(sprint.id)}
                onComplete={() => setCompleteSprintTarget(sprint)}
                onDelete={() => deleteSprint.mutate(sprint.id)}
                onEdit={() => setEditSprint(sprint)}
              />
              {/* 미션 + 에픽 필터 (콜랩스) */}
              {sprint.id === selectedSprintId && (filteredMissionsForFilter.length > 0 || filterAvailableEpics.length > 0) && (
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
                    <div className="mt-1 px-2 py-2 bg-slate-50 rounded-lg border border-slate-100 flex flex-col gap-2">
                      {missions.length > 0 && (
                        <div>
                          <p className="text-[10px] font-semibold text-slate-400 uppercase tracking-wider mb-1">미션</p>
                          <div className="flex flex-wrap gap-1">
                            <button
                              type="button"
                              onClick={() => setSelectedMissionIds([])}
                              className={`text-[11px] px-2 py-0.5 rounded-full border transition-colors ${
                                selectedMissionIds.length === 0
                                  ? 'bg-violet-100 text-violet-700 border-violet-300 font-medium'
                                  : 'bg-white text-slate-500 border-slate-200 hover:border-violet-200'
                              }`}
                            >
                              전체
                            </button>
                            <button
                              type="button"
                              onClick={() =>
                                setSelectedMissionIds((prev) =>
                                  prev.includes(0) ? prev.filter((id) => id !== 0) : [...prev, 0]
                                )
                              }
                              className={`text-[11px] px-2 py-0.5 rounded-full border transition-colors ${
                                selectedMissionIds.includes(0)
                                  ? 'bg-violet-100 text-violet-700 border-violet-300 font-medium'
                                  : 'bg-white text-slate-500 border-slate-200 hover:border-violet-200'
                              }`}
                            >
                              미분류
                            </button>
                            {missions.map((m) => (
                              <button
                                key={m.id}
                                type="button"
                                title={m.title}
                                onClick={() =>
                                  setSelectedMissionIds((prev) =>
                                    prev.includes(m.id) ? prev.filter((id) => id !== m.id) : [...prev, m.id]
                                  )
                                }
                                className={`text-[11px] px-2 py-0.5 rounded-full border transition-colors max-w-[120px] truncate ${
                                  selectedMissionIds.includes(m.id)
                                    ? 'bg-violet-100 text-violet-700 border-violet-300 font-medium'
                                    : 'bg-white text-slate-500 border-slate-200 hover:border-violet-200'
                                }`}
                              >
                                {m.title}
                              </button>
                            ))}
                          </div>
                        </div>
                      )}
                      {filterAvailableEpics.length > 0 && (
                        <div>
                          <p className="text-[10px] font-semibold text-slate-400 uppercase tracking-wider mb-1">에픽</p>
                          <div className="flex flex-wrap gap-1">
                            {filterAvailableEpics.map((epic) => (
                              <button
                                key={epic.id}
                                type="button"
                                title={epic.title}
                                onClick={() =>
                                  setSelectedEpicIds((prev) =>
                                    prev.includes(epic.id) ? prev.filter((id) => id !== epic.id) : [...prev, epic.id]
                                  )
                                }
                                className={`text-[11px] px-2 py-0.5 rounded-full border transition-colors max-w-[130px] truncate ${
                                  selectedEpicIds.includes(epic.id)
                                    ? 'bg-indigo-100 text-indigo-700 border-indigo-300 font-medium'
                                    : 'bg-white text-slate-500 border-slate-200 hover:border-indigo-200'
                                }`}
                              >
                                {epic.title}
                              </button>
                            ))}
                          </div>
                        </div>
                      )}
                    </div>
                  )}
                </div>
              )}
            </div>
          ))}

          {/* 완료된 스프린트 아코디언 */}
          {pastSprints.length > 0 && (
            <div className="mt-4 border-t border-slate-200/60 pt-3">
              <button
                type="button"
                onClick={() => setShowPastSprints(!showPastSprints)}
                className="w-full px-3 py-1.5 flex items-center justify-between text-xs font-semibold text-slate-400 hover:text-slate-600 hover:bg-slate-100 rounded-md transition-colors"
              >
                <span>완료된 스프린트 ({pastSprints.length})</span>
                <span>{showPastSprints ? '▼' : '▶'}</span>
              </button>
              
              {showPastSprints && (
                <div className="mt-1 px-1">
                  {pastSprints.map((sprint) => (
                    <div key={sprint.id}>
                      <SprintItem
                        sprint={sprint}
                        selected={sprint.id === selectedSprintId}
                        onClick={() => selectSprint(sprint.id)}
                        onActivate={() => activateSprint.mutate(sprint.id)}
                        onComplete={() => setCompleteSprintTarget(sprint)}
                        onDelete={() => deleteSprint.mutate(sprint.id)}
                        onEdit={() => setEditSprint(sprint)}
                      />
                      {sprint.id === selectedSprintId && (filteredMissionsForFilter.length > 0 || filterAvailableEpics.length > 0) && (
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
                            <div className="mt-1 px-2 py-2 bg-slate-50 rounded-lg border border-slate-100 flex flex-col gap-2">
                              {missions.length > 0 && (
                                <div>
                                  <p className="text-[10px] font-semibold text-slate-400 uppercase tracking-wider mb-1">미션</p>
                                  <div className="flex flex-wrap gap-1">
                                    <button
                                      type="button"
                                      onClick={() => setSelectedMissionIds([])}
                                      className={`text-[11px] px-2 py-0.5 rounded-full border transition-colors ${
                                        selectedMissionIds.length === 0
                                          ? 'bg-violet-100 text-violet-700 border-violet-300 font-medium'
                                          : 'bg-white text-slate-500 border-slate-200 hover:border-violet-200'
                                      }`}
                                    >
                                      전체
                                    </button>
                                    <button
                                      type="button"
                                      onClick={() =>
                                        setSelectedMissionIds((prev) =>
                                          prev.includes(0) ? prev.filter((id) => id !== 0) : [...prev, 0]
                                        )
                                      }
                                      className={`text-[11px] px-2 py-0.5 rounded-full border transition-colors ${
                                        selectedMissionIds.includes(0)
                                          ? 'bg-violet-100 text-violet-700 border-violet-300 font-medium'
                                          : 'bg-white text-slate-500 border-slate-200 hover:border-violet-200'
                                      }`}
                                    >
                                      미분류
                                    </button>
                                    {missions.map((m) => (
                                      <button
                                        key={m.id}
                                        type="button"
                                        title={m.title}
                                        onClick={() =>
                                          setSelectedMissionIds((prev) =>
                                            prev.includes(m.id) ? prev.filter((id) => id !== m.id) : [...prev, m.id]
                                          )
                                        }
                                        className={`text-[11px] px-2 py-0.5 rounded-full border transition-colors max-w-[120px] truncate ${
                                          selectedMissionIds.includes(m.id)
                                            ? 'bg-violet-100 text-violet-700 border-violet-300 font-medium'
                                            : 'bg-white text-slate-500 border-slate-200 hover:border-violet-200'
                                        }`}
                                      >
                                        {m.title}
                                      </button>
                                    ))}
                                  </div>
                                </div>
                              )}
                              {filterAvailableEpics.length > 0 && (
                                <div>
                                  <p className="text-[10px] font-semibold text-slate-400 uppercase tracking-wider mb-1">에픽</p>
                                  <div className="flex flex-wrap gap-1">
                                    {filterAvailableEpics.map((epic) => (
                                      <button
                                        key={epic.id}
                                        type="button"
                                        title={epic.title}
                                        onClick={() =>
                                          setSelectedEpicIds((prev) =>
                                            prev.includes(epic.id) ? prev.filter((id) => id !== epic.id) : [...prev, epic.id]
                                          )
                                        }
                                        className={`text-[11px] px-2 py-0.5 rounded-full border transition-colors max-w-[130px] truncate ${
                                          selectedEpicIds.includes(epic.id)
                                            ? 'bg-indigo-100 text-indigo-700 border-indigo-300 font-medium'
                                            : 'bg-white text-slate-500 border-slate-200 hover:border-indigo-200'
                                        }`}
                                      >
                                        {epic.title}
                                      </button>
                                    ))}
                                  </div>
                                </div>
                              )}
                            </div>
                          )}
                        </div>
                      )}
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>
      </div>

      {/* Resize Handle */}
      <div
        onMouseDown={handleMouseDown}
        className="w-[3px] hover:w-[6px] hover:bg-indigo-300 active:bg-indigo-500 cursor-col-resize flex-shrink-0 transition-all duration-150 z-30"
        style={{ cursor: 'col-resize' }}
      />

      {/* ── Main content ── */}
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
            <button
              type="button"
              onClick={() => {
                setEditMission(null);
                setMissionModalOpen(true);
              }}
              className="text-xs sm:text-sm px-2 sm:px-2.5 py-1.5 bg-violet-100 hover:bg-violet-200 text-violet-700 rounded-md shrink min-w-0 max-w-[80px] md:max-w-none whitespace-nowrap flex items-center justify-center"
            >
              <span className="truncate min-w-0">+ 새 미션</span>
            </button>
            <button
              type="button"
              onClick={() => setEpicModalOpen(true)}
              className="text-xs sm:text-sm px-2 sm:px-2.5 py-1.5 bg-slate-200 hover:bg-slate-300 text-slate-700 rounded-md shrink min-w-0 max-w-[80px] md:max-w-none whitespace-nowrap flex items-center justify-center"
            >
              <span className="truncate min-w-0">+ 새 에픽</span>
            </button>
            {allEpics.length > 0 && (
              <button
                type="button"
                onClick={() => setIssueModalEpicId(allEpics[0].id)}
                className="text-xs sm:text-sm px-2 sm:px-2.5 py-1.5 bg-indigo-600 hover:bg-indigo-50 text-white rounded-md shrink min-w-0 max-w-[80px] md:max-w-none whitespace-nowrap flex items-center justify-center"
              >
                <span className="truncate min-w-0">+ 새 이슈</span>
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
        <div className="flex-1 overflow-y-auto p-6">
          {selectedSprint && (selectedSprint.status === 'completed' || selectedSprint.status === 'cancelled') && (
            <div className="mb-6 px-4 py-3 bg-indigo-50/80 border border-indigo-100 rounded-xl flex items-center justify-between text-xs text-indigo-700">
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

      {/* Modals */}
      <MissionModal
        open={missionModalOpen}
        onClose={() => {
          setMissionModalOpen(false);
          setEditMission(null);
        }}
        mission={editMission ?? undefined}
      />
      <CreateSprintModal
        open={sprintModalOpen}
        onClose={() => setSprintModalOpen(false)}
      />
      <CreateEpicModal
        open={epicModalOpen}
        onClose={() => setEpicModalOpen(false)}
      />
      <CreateIssueModal
        open={issueModalEpicId != null}
        onClose={() => setIssueModalEpicId(null)}
        defaultEpicId={issueModalEpicId ?? undefined}
      />
      <EditEpicModal
        epic={editEpic}
        onClose={() => setEditEpic(null)}
      />
      <EditSprintModal
        sprint={editSprint}
        onClose={() => setEditSprint(null)}
      />
      {completeSprintTarget && (
        <ConfirmCompleteSprintModal
          isOpen={!!completeSprintTarget}
          onClose={() => setCompleteSprintTarget(null)}
          sprint={completeSprintTarget}
          sprints={sprints}
        />
      )}

      {/* ── Bulk action floating bar ── */}
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
