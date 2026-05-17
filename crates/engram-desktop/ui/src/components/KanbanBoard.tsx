import { useState } from 'react';
import { DndContext, DragEndEvent, DragOverlay, PointerSensor, useSensor, useSensors } from '@dnd-kit/core';
import { KanbanColumn } from './KanbanColumn';
import { IssueCard } from './IssueCard';
import { FilterBar } from './FilterBar';
import { ScopeExpansionBanner } from './ScopeExpansionBanner';
import { CreateIssueModal } from './CreateIssueModal';
import { CreateEpicModal } from './CreateEpicModal';
import { CreateSprintModal } from './CreateSprintModal';
import { useBoardStatus } from '../hooks/useBoardStatus';
import { useIssueDnd } from '../hooks/useIssueDnd';
import { useSessionRestore } from '../hooks/useSessionRestore';
import { useUIStore } from '../store/ui';
import type { Issue, IssueStatus, IssueProjectBoard } from '../ipc/types';

type BoardColumn = 'required' | 'ready' | 'working' | 'demo' | 'finished' | 'cancelled';
const STANDARD_COLUMNS: BoardColumn[] = ['required', 'ready', 'working', 'demo', 'finished'];

export function KanbanBoard() {
  const {
    selectedProjectKey, selectIssue,
    hideFinished, toggleHideFinished,
    showCancelled, toggleShowCancelled,
    boardFilters, setBoardFilters, resetBoardFilters,
  } = useUIStore();

  const { data, isLoading, error } = useBoardStatus(selectedProjectKey ?? undefined);
  const { data: session } = useSessionRestore(selectedProjectKey ?? undefined);
  const dnd = useIssueDnd(selectedProjectKey ?? undefined);

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

  // Apply client-side filters
  const filteredBoards = applyFilters(boards, boardFilters);

  const visibleColumns: BoardColumn[] = [
    ...STANDARD_COLUMNS.filter((c) => !(hideFinished && c === 'finished')),
    ...(showCancelled ? (['cancelled'] as BoardColumn[]) : []),
  ];

  function handleDragEnd(event: DragEndEvent) {
    setActiveIssue(null);
    const { active, over } = event;
    if (!over) return;
    const issueId = active.id as number;
    const toStatus = over.id as IssueStatus;
    const fromStatus = (active.data.current as { issue: Issue })?.issue.status;
    if (fromStatus === toStatus) return;
    dnd.mutate({ id: issueId, status: toStatus });
  }

  return (
    <DndContext
      sensors={sensors}
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
        />

        {/* Scope expansion banner */}
        {warnings.length > 0 && (
          <ScopeExpansionBanner
            warnings={warnings}
            onIssueClick={selectIssue}
          />
        )}

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
          <div key={board.project_key}>
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
            <div className={`grid gap-3`} style={{ gridTemplateColumns: `repeat(${visibleColumns.length}, minmax(0, 1fr))` }}>
              {visibleColumns.map((status) => (
                <KanbanColumn
                  key={status}
                  status={status}
                  issues={(board as unknown as Record<string, Issue[]>)[status] ?? []}
                  onIssueClick={(id) => selectIssue(id)}
                  expansionIds={expansionIds}
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
            <IssueCard issue={activeIssue} scopeExpanded={expansionIds.has(activeIssue.id)} />
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

  // Priority filter (applied per column)
  if (filters.priorities.length > 0) {
    result = result.map((board) => {
      const filterIssues = (issues: Issue[]) =>
        issues.filter((i) => filters.priorities.includes(i.priority));
      return {
        ...board,
        required: filterIssues(board.required),
        ready: filterIssues(board.ready),
        working: filterIssues(board.working),
        demo: filterIssues(board.demo),
        finished: filterIssues(board.finished),
        cancelled: filterIssues(board.cancelled ?? []),
      };
    });
  }

  return result;
}
