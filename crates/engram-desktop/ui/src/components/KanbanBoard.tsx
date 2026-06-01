import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { DndContext, DragEndEvent, DragOverlay, MeasuringStrategy, PointerSensor, closestCenter, pointerWithin, rectIntersection, useSensor, useSensors, type CollisionDetection } from '@dnd-kit/core';
import { KanbanColumn } from './KanbanColumn';
import { IssueCardView } from './IssueCard';
import { FilterBar } from './FilterBar';
import { ScopeExpansionBanner } from './ScopeExpansionBanner';
import { StalledWorkingBanner } from './StalledWorkingBanner';
import { CreateIssueModal } from './CreateIssueModal';
import { CreateEpicModal } from './CreateEpicModal';
import { CreateSprintModal } from './CreateSprintModal';
import { useBoardStatus } from '../hooks/useBoardStatus';
import { useIssueDnd } from '../hooks/useIssueDnd';
import { useSessionRestore } from '../hooks/useSessionRestore';
import { useEpics } from '../hooks/useEpics';
import { useUIStore } from '../store/ui';
import { missionList } from '../ipc/invoke';
import { BOARD_COLUMNS, type Issue, type IssueStatus, type IssueProjectBoard, type Mission } from '../ipc/types';

type BoardColumn = IssueStatus;
const STANDARD_COLUMNS = BOARD_COLUMNS.filter((c) => c !== 'cancelled');

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
  const dnd = useIssueDnd(null);

  const epicMap = new Map(epics.map((e) => [e.id, e.title]));

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 5 } })
  );

  const [activeIssue, setActiveIssue] = useState<Issue | null>(null);
  const [issueModalProject, setIssueModalProject] = useState<string | null>(null);
  const [epicModalProject, setEpicModalProject] = useState<string | null>(null);
  const [sprintModalOpen, setSprintModalOpen] = useState(false);

  if (isLoading) return <div className="p-8 text-slate-400">Loading board…</div>;
  if (error)    return <div className="p-8 text-red-500">Error loading board</div>;

  const boards = data?.boards ?? [];
  const warnings = session?.warnings ?? [];
  const expansionIds = new Set<number>(session?.scope_expansion_ids ?? []);
  const stalledIssues = data?.stalled_issues ?? [];

  // Apply client-side filters
  let filteredBoards = applyFilters(boards, boardFilters);
  if (selectedProjectKey) {
    filteredBoards = filteredBoards.filter((b) => b.project_key === selectedProjectKey);
  }

  const visibleColumns: BoardColumn[] = [
    ...STANDARD_COLUMNS.filter((c) => !(hideFinished && c === 'finished')),
    ...(showCancelled ? (['cancelled'] as BoardColumn[]) : []),
  ];

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
      <div className="flex flex-col gap-4 p-6 overflow-auto h-full">
        {/* Filter bar */}
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

        <div className="flex items-center justify-between">
          <span className="text-xs text-slate-400">{filteredBoards.length} 프로젝트</span>
          <div className="flex gap-2">
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

        {filteredBoards.map((board) => (
          <div key={board.project_key} className="overflow-x-auto pb-4 shrink-0">
            <div className="flex items-center justify-between mb-3">
              <h2 className="text-base font-semibold text-slate-700">{board.project_key}</h2>
              <button
                type="button"
                onClick={() => setIssueModalProject(board.project_key)}
                className="text-xs px-2 py-1 bg-slate-100 hover:bg-slate-200 text-slate-700 rounded"
              >
                + 이슈 추가
              </button>
            </div>
            <div className={`grid gap-3`} style={{ gridTemplateColumns: `repeat(${visibleColumns.length}, minmax(280px, 1fr))` }}>
              {visibleColumns.map((status) => (
                <KanbanColumn
                  key={status}
                  projectKey={board.project_key}
                  status={status}
                  issues={(board as unknown as Record<string, Issue[]>)[status] ?? []}
                  onIssueClick={(id) => selectIssue(id)}
                  expansionIds={expansionIds}
                  epicMap={epicMap}
                  onCreateIssue={status === 'required' ? () => setIssueModalProject(board.project_key) : undefined}
                />
              ))}
            </div>
          </div>
        ))}
      </div>

      <CreateSprintModal
        open={sprintModalOpen}
        onClose={() => setSprintModalOpen(false)}
      />
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
