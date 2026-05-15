import { useState } from 'react';
import { DndContext, DragEndEvent, DragOverlay, PointerSensor, useSensor, useSensors } from '@dnd-kit/core';
import { KanbanColumn } from './KanbanColumn';
import { IssueCard } from './IssueCard';
import { FilterBar } from './FilterBar';
import { ScopeExpansionBanner } from './ScopeExpansionBanner';
import { useBoardStatus } from '../hooks/useBoardStatus';
import { useIssueDnd } from '../hooks/useIssueDnd';
import { useSessionRestore } from '../hooks/useSessionRestore';
import { useUIStore } from '../store/ui';
import type { Issue, IssueStatus, IssueProjectBoard } from '../ipc/types';

type BoardColumn = 'required' | 'ready' | 'working' | 'demo' | 'finished';
const STANDARD_COLUMNS: BoardColumn[] = ['required', 'ready', 'working', 'demo', 'finished'];

export function KanbanBoard() {
  const {
    selectedProjectKey, selectIssue,
    hideFinished, toggleHideFinished,
    boardFilters, setBoardFilters, resetBoardFilters,
  } = useUIStore();

  const { data, isLoading, error } = useBoardStatus(selectedProjectKey ?? undefined);
  const { data: session } = useSessionRestore(selectedProjectKey ?? undefined);
  const dnd = useIssueDnd(selectedProjectKey ?? undefined);

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 5 } })
  );

  const [activeIssue, setActiveIssue] = useState<Issue | null>(null);

  if (isLoading) return <div className="p-8 text-slate-400">Loading board…</div>;
  if (error)    return <div className="p-8 text-red-500">Error loading board</div>;

  const boards = data?.boards ?? [];
  const warnings = session?.warnings ?? [];
  const expansionIds = new Set<number>(session?.scope_expansion_ids ?? []);

  // Apply client-side filters
  const filteredBoards = applyFilters(boards, boardFilters);

  const visibleColumns: BoardColumn[] = STANDARD_COLUMNS.filter(
    (c) => !(hideFinished && c === 'finished')
  );

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

        {filteredBoards.map((board) => (
          <div key={board.project_key}>
            <h2 className="text-base font-semibold text-slate-700 mb-3">{board.project_key}</h2>
            <div className={`grid gap-3`} style={{ gridTemplateColumns: `repeat(${visibleColumns.length}, minmax(0, 1fr))` }}>
              {visibleColumns.map((status) => (
                <KanbanColumn
                  key={status}
                  status={status}
                  issues={(board as unknown as Record<string, Issue[]>)[status] ?? []}
                  onIssueClick={(id) => selectIssue(id)}
                  expansionIds={expansionIds}
                />
              ))}
            </div>
          </div>
        ))}
      </div>

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
      };
    });
  }

  return result;
}
