import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { DndContext, DragEndEvent, DragOverlay, MeasuringStrategy, PointerSensor, closestCenter, pointerWithin, rectIntersection, useSensor, useSensors, type CollisionDetection } from '@dnd-kit/core';
import { toast } from 'sonner';
import { KanbanColumn, KanbanColumnHeader } from './KanbanColumn';
import { IssueCardView } from './IssueCard';
import { FilterBar } from './FilterBar';
import { ScopeExpansionBanner } from './ScopeExpansionBanner';
import { StalledWorkingBanner } from './StalledWorkingBanner';
import { CreateIssueModal } from './CreateIssueModal';
import { CreateEpicModal } from './CreateEpicModal';
import { CreateSprintModal } from './CreateSprintModal';
import { BulkFinishConfirmModal } from './BulkFinishConfirmModal';
import { useBoardStatus } from '../hooks/useBoardStatus';
import { useIssueDnd } from '../hooks/useIssueDnd';
import { useSessionRestore } from '../hooks/useSessionRestore';
import { useEpics } from '../hooks/useEpics';
import { useUIStore } from '../store/ui';
import { missionList, issueSetStatus, sprintList } from '../ipc/invoke';
import { BOARD_COLUMNS, type Issue, type IssueStatus, type IssueProjectBoard, type Mission, type Sprint } from '../ipc/types';
import { ConfirmCompleteSprintModal } from './ConfirmCompleteSprintModal';

type BoardColumn = IssueStatus;
const STANDARD_COLUMNS = BOARD_COLUMNS.filter((c) => c !== 'cancelled');

const COLUMN_LABELS: Record<BoardColumn, string> = {
  required: 'Required',
  ready: 'Ready',
  working: 'Working',
  demo: 'Demo',
  finished: 'Finished',
  cancelled: 'Cancelled',
};

const customCollisionDetection: CollisionDetection = (args) => {
  const pointerCollisions = pointerWithin(args);
  if (pointerCollisions.length > 0) {
    return pointerCollisions;
  }
  const rectCollisions = rectIntersection(args);
  if (rectCollisions.length > 0) {
    return rectCollisions;
  }
  return closestCenter(args);
};

export function KanbanBoard() {
  const {
    selectedProjectKey, selectIssue,
    hideFinished, toggleHideFinished,
    showCancelled, toggleShowCancelled,
    boardFilters, setBoardFilters, resetBoardFilters,
  } = useUIStore();

  const { data, isLoading, error } = useBoardStatus(null);
  const { data: session } = useSessionRestore(undefined);
  const { data: epics = [] } = useEpics(undefined);
  const { data: missions = [] } = useQuery<Mission[]>({
    queryKey: ['missionList'],
    queryFn: () => missionList(true),
  }); // ADR-0014: Issue.mission_id 는 Epic 에서 derive 된 값. 백엔드 JOIN 결과로 일관성 보장.
  const { data: sprints = [] } = useQuery<Sprint[]>({
    queryKey: ['sprintList'],
    queryFn: sprintList,
    refetchInterval: 30_000,
  });
  const dnd = useIssueDnd(null);

  const epicMap = new Map(epics.map((e) => [e.id, e.title]));

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 5 } })
  );

  const [activeIssue, setActiveIssue] = useState<Issue | null>(null);
  const [issueModalProject, setIssueModalProject] = useState<string | null>(null);
  const [epicModalProject, setEpicModalProject] = useState<string | null>(null);
  const [sprintModalOpen, setSprintModalOpen] = useState(false);
  const [bulkFinishTarget, setBulkFinishTarget] = useState<{ projectKey: string; issues: Issue[] } | null>(null);
  const [completeSprintTarget, setCompleteSprintTarget] = useState<Sprint | null>(null);

  const queryClient = useQueryClient();
  const bulkFinishMutation = useMutation({
    mutationFn: async (issues: Issue[]) => {
      const results = await Promise.allSettled(
        issues.map((i) => issueSetStatus(i.id, 'finished')),
      );
      const failed = results.filter((r) => r.status === 'rejected');
      if (failed.length > 0) {
        throw new Error(`${failed.length}건 실패`);
      }
      return results;
    },
    onSuccess: (_data, vars) => {
      toast.success(`${vars.length}건 완료 처리됨`);
      queryClient.invalidateQueries({ queryKey: ['boardStatus'] });
      setBulkFinishTarget(null);
    },
    onError: (err) => {
      toast.error(`일괄 완료 실패: ${err.message}`);
      queryClient.invalidateQueries({ queryKey: ['boardStatus'] });
    },
  });

  if (isLoading) return <div className="p-8 text-slate-400">Loading board…</div>;
  if (error)    return <div className="p-8 text-red-500">Error loading board</div>;

  const boards = data?.boards ?? [];
  const warnings = session?.warnings ?? [];
  const expansionIds = new Set<number>(session?.scope_expansion_ids ?? []);
  const stalledIssues = data?.stalled_issues ?? [];
  const activeSprint = sprints.find((s) => s.id === data?.sprint_id && s.status === 'active');

  // Apply client-side filters
  let filteredBoards = applyFilters(boards, boardFilters);
  if (selectedProjectKey) {
    filteredBoards = filteredBoards.filter((b) => b.project_key === selectedProjectKey);
  }

  const visibleColumns: BoardColumn[] = [
    ...STANDARD_COLUMNS.filter((c) => !(hideFinished && c === 'finished')),
    ...(showCancelled ? (['cancelled'] as BoardColumn[]) : []),
  ];

  // visible 컬럼에 이슈가 하나도 없는 빈 보드 제거 ("완료 숨기기" 버그 수정 포함)
  filteredBoards = filteredBoards.filter((board) =>
    visibleColumns.some((col) => ((board as unknown as Record<string, Issue[]>)[col] ?? []).length > 0)
  );

  function handleDragEnd(event: DragEndEvent) {
    setActiveIssue(null);
    const { active, over } = event;
    if (!over) return;
    const issueId = active.id as number;

    const activeIssueData = active.data.current as { issue: Issue } | undefined;
    if (!activeIssueData) return;

    let toStatus: IssueStatus | null = null;
    let targetProjectKey: string | null = null;

    if (typeof over.id === 'string') {
      const parts = over.id.split('-');
      toStatus = parts.pop() as IssueStatus;
      targetProjectKey = parts.join('-');
    } else {
      const overIssue = (over.data.current as { issue: Issue } | undefined)?.issue;
      if (overIssue) {
        toStatus = overIssue.status;
      }
    }

    if (!toStatus) return;

    // 다른 프로젝트 보드의 컬럼에 드랍하는 것을 차단
    if (targetProjectKey) {
      const issueEpic = epics.find((e) => e.id === activeIssueData.issue.epic_id);
      if (issueEpic && issueEpic.project_key !== targetProjectKey) {
        return;
      }
    }

    const fromStatus = activeIssueData.issue.status;
    if (fromStatus === toStatus) return;
    dnd.mutate({ id: issueId, status: toStatus });
  }

  return (
    <DndContext
      sensors={sensors}
      collisionDetection={customCollisionDetection}
      measuring={{ droppable: { strategy: MeasuringStrategy.Always } }}
      onDragStart={(e) => setActiveIssue((e.active.data.current as { issue: Issue })?.issue ?? null)}
      onDragEnd={handleDragEnd}
      onDragCancel={() => setActiveIssue(null)}
    >
      {/* Outer: full height flex column — no scroll here */}
      <div className="flex flex-col h-full overflow-hidden">

        {/* ── Sticky Filter Bar (never scrolls) ── */}
        <div className="flex-shrink-0 bg-white border-b border-slate-200 px-6 pt-4 pb-3 z-20">
          <FilterBar
            boards={boards}
            filters={boardFilters}
            hideFinished={hideFinished}
            onToggleHideFinished={toggleHideFinished}
            showCancelled={showCancelled}
            onToggleShowCancelled={toggleShowCancelled}
            onChange={setBoardFilters}
            onReset={resetBoardFilters}
            missions={missions}
            epics={epics}
          />
        </div>

        {/* ── Sticky Stats + Action Buttons Bar ── */}
        <div className="flex-shrink-0 bg-white border-b border-slate-100 px-6 py-2.5 flex items-center justify-between flex-wrap gap-2 z-10">
          <div className="flex items-center gap-3 flex-wrap">
            <span className="text-xs text-slate-400 font-medium">{filteredBoards.length} 프로젝트</span>
            <div className="h-3 w-px bg-slate-200" />
            {visibleColumns.map((col) => {
              const count = filteredBoards.reduce(
                (sum, b) => sum + ((b as unknown as Record<string, Issue[]>)[col]?.length ?? 0), 0
              );
              return (
                <span key={col} className="text-[10px] text-slate-500">
                  <span className="font-semibold">{COLUMN_LABELS[col]}</span>{' '}
                  <span className={count > 0 ? 'text-slate-700 font-bold' : ''}>{count}</span>
                </span>
              );
            })}
          </div>
          <div className="flex gap-2">
            {activeSprint && (
              <button
                type="button"
                onClick={() => setCompleteSprintTarget(activeSprint)}
                className="px-3 py-1.5 bg-emerald-600 hover:bg-emerald-500 text-white text-xs font-semibold rounded-md shadow-sm transition-all hover:scale-105 active:scale-95 flex items-center justify-center gap-0.5"
                title="현재 활성화된 스프린트 완료 처리"
              >
                스프린트 완료
              </button>
            )}
            <button
              type="button"
              onClick={() => setSprintModalOpen(true)}
              className="px-3 py-1.5 bg-slate-600 hover:bg-slate-500 text-white text-xs rounded-md"
            >
              + 새 스프린트
            </button>
            <button
              type="button"
              onClick={() => setEpicModalProject(selectedProjectKey ?? '')}
              className="px-3 py-1.5 bg-indigo-600 hover:bg-indigo-500 text-white text-xs rounded-md"
            >
              + 새 에픽
            </button>
          </div>
        </div>

        {/* ── Single scrollable content area ── */}
        <div className="flex-1 overflow-y-auto overflow-x-hidden min-h-0">
          <div className="p-6 flex flex-col gap-4 w-full">

            {/* Scope expansion banner */}
            {warnings.length > 0 && (
              <ScopeExpansionBanner
                warnings={warnings}
                onIssueClick={selectIssue}
              />
            )}

            {/* Stalled working banner */}
            <StalledWorkingBanner
              issues={stalledIssues}
              onIssueClick={selectIssue}
            />

            {filteredBoards.length === 0 && (
              <div className="text-slate-400 text-center mt-20">이슈가 없습니다. CLI로 이슈를 생성하세요.</div>
            )}

        {filteredBoards.map((board) => (
          <div key={board.project_key} className="pb-4 shrink-0 flex flex-col">
            {/* 프로젝트명 영역 (가로 고정) */}
            <div className="flex items-center justify-between mb-3 py-1 flex-shrink-0">
              <h2 className="text-base font-semibold text-slate-700">{board.project_key}</h2>
              <button
                type="button"
                onClick={() => setIssueModalProject(board.project_key)}
                className="text-xs px-2 py-1 bg-slate-100 hover:bg-slate-200 text-slate-700 rounded"
              >
                + 이슈 추가
              </button>
            </div>

            {/* 가로 스크롤 영역 (컬럼 헤더 & 카드 그리드) */}
            <div className="overflow-x-auto pb-4 -mx-6 px-6">
              <div className="min-w-max flex flex-col gap-3">
                {/* Column Headers */}
                <div className={`grid gap-3`} style={{ gridTemplateColumns: `repeat(${visibleColumns.length}, 300px)` }}>
                  {visibleColumns.map((status) => {
                    const issues = (board as unknown as Record<string, Issue[]>)[status] ?? [];
                    return (
                      <KanbanColumnHeader
                        key={status}
                        status={status}
                        issueCount={issues.length}
                        onCreateIssue={status === 'required' ? () => setIssueModalProject(board.project_key) : undefined}
                        onBulkFinish={status === 'demo' ? () => {
                          const demoIssues = (board as unknown as Record<string, Issue[]>)['demo'] ?? [];
                          if (demoIssues.length > 0) {
                            setBulkFinishTarget({ projectKey: board.project_key, issues: demoIssues });
                          }
                        } : undefined}
                      />
                    );
                  })}
                </div>

                {/* Cards grid */}
                <div className={`grid gap-3`} style={{ gridTemplateColumns: `repeat(${visibleColumns.length}, 300px)` }}>
                  {visibleColumns.map((status) => (
                    <KanbanColumn
                      key={status}
                      projectKey={board.project_key}
                      status={status}
                      issues={(board as unknown as Record<string, Issue[]>)[status] ?? []}
                      onIssueClick={(id) => selectIssue(id)}
                      expansionIds={expansionIds}
                      epicMap={epicMap}
                    />
                  ))}
                </div>
              </div>
            </div>
          </div>
            ))}
          </div>{/* end p-6 content */}
        </div>{/* end overflow-y-auto scroll area */}
      </div>{/* end outer h-full flex col */}

      <CreateSprintModal
        open={sprintModalOpen}
        onClose={() => setSprintModalOpen(false)}
      />
      {completeSprintTarget && (
        <ConfirmCompleteSprintModal
          isOpen={!!completeSprintTarget}
          onClose={() => setCompleteSprintTarget(null)}
          sprint={completeSprintTarget}
          sprints={sprints}
        />
      )}
      <CreateIssueModal
        open={issueModalProject !== null}
        onClose={() => setIssueModalProject(null)}
        projectKey={issueModalProject ?? undefined}
      />
      <CreateEpicModal
        open={epicModalProject !== null}
        onClose={() => setEpicModalProject(null)}
        defaultProjectKey={epicModalProject ?? undefined}
      />
      <BulkFinishConfirmModal
        open={bulkFinishTarget !== null}
        issues={bulkFinishTarget?.issues ?? []}
        projectKey={bulkFinishTarget?.projectKey ?? ''}
        onConfirm={() => {
          if (bulkFinishTarget) {
            bulkFinishMutation.mutate(bulkFinishTarget.issues);
          }
        }}
        onCancel={() => setBulkFinishTarget(null)}
        isPending={bulkFinishMutation.isPending}
      />

      <DragOverlay>
        {activeIssue && (
          <div className="rotate-2 scale-105">
            <IssueCardView issue={activeIssue} scopeExpanded={expansionIds.has(activeIssue.id)} />
          </div>
        )}
      </DragOverlay>
    </DndContext>
  );
}

function applyFilters(
  boards: IssueProjectBoard[],
  filters: ReturnType<typeof useUIStore.getState>['boardFilters'],
): IssueProjectBoard[] {
  let result = boards;

  // Project filter
  if (filters.projects.length > 0) {
    result = result.filter((b) => filters.projects.includes(b.project_key));
  }

  // Mission filter
  if (filters.missionIds.length > 0) {
    result = result.map((board) => {
      const f = (issues: Issue[]) =>
        issues.filter((i) => i.mission_id != null && filters.missionIds.includes(i.mission_id)); // ADR-0014: Epic 에서 derive 된 값 사용
      const next = { ...board };
      for (const col of BOARD_COLUMNS) {
        next[col] = f(next[col] ?? []);
      }
      return next;
    });
  }

  // Epic filter
  if (filters.epicIds.length > 0) {
    result = result.map((board) => {
      const f = (issues: Issue[]) =>
        issues.filter((i) => filters.epicIds.includes(i.epic_id));
      const next = { ...board };
      for (const col of BOARD_COLUMNS) {
        next[col] = f(next[col] ?? []);
      }
      return next;
    });
  }

  // Priority filter
  if (filters.priorities.length > 0) {
    result = result.map((board) => {
      const f = (issues: Issue[]) =>
        issues.filter((i) => filters.priorities.includes(i.priority));
      const next = { ...board };
      for (const col of BOARD_COLUMNS) {
        next[col] = f(next[col] ?? []);
      }
      return next;
    });
  }

  return result;
}
