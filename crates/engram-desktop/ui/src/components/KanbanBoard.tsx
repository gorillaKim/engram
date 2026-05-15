import { useState } from 'react';
import { DndContext, DragEndEvent, DragOverlay, PointerSensor, useSensor, useSensors } from '@dnd-kit/core';
import { KanbanColumn } from './KanbanColumn';
import { IssueCard } from './IssueCard';
import { useBoardStatus } from '../hooks/useBoardStatus';
import { useIssueDnd } from '../hooks/useIssueDnd';
import { useUIStore } from '../store/ui';
import type { Issue, IssueStatus } from '../ipc/types';

type BoardColumn = 'required' | 'ready' | 'working' | 'demo' | 'finished';
const COLUMNS: BoardColumn[] = ['required', 'ready', 'working', 'demo', 'finished'];

export function KanbanBoard() {
  const { selectedProjectKey, selectIssue, hideFinished, toggleHideFinished } = useUIStore();
  const { data, isLoading, error } = useBoardStatus(selectedProjectKey ?? undefined);
  const dnd = useIssueDnd(selectedProjectKey ?? undefined);

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 5 } })
  );

  const [activeIssue, setActiveIssue] = useState<Issue | null>(null);

  if (isLoading) return <div className="p-8 text-slate-400">Loading board…</div>;
  if (error)    return <div className="p-8 text-red-500">Error loading board</div>;

  const boards = data?.boards ?? [];
  const visibleColumns = hideFinished ? COLUMNS.filter((c) => c !== 'finished') : COLUMNS;

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
      <div className="flex flex-col gap-6 p-6 overflow-auto h-full">
        {/* Filter bar */}
        <div className="flex items-center gap-4">
          <label className="flex items-center gap-1.5 text-sm text-slate-600 cursor-pointer select-none">
            <input
              type="checkbox"
              checked={hideFinished}
              onChange={toggleHideFinished}
              className="rounded border-slate-300 text-indigo-600"
            />
            완료 숨기기
          </label>
        </div>

        {boards.length === 0 && (
          <div className="text-slate-400 text-center mt-20">이슈가 없습니다. CLI로 이슈를 생성하세요.</div>
        )}

        {boards.map((board) => (
          <div key={board.project_key}>
            <h2 className="text-base font-semibold text-slate-700 mb-3">{board.project_key}</h2>
            <div className={`grid gap-3 ${visibleColumns.length === 5 ? 'grid-cols-5' : 'grid-cols-4'}`}>
              {visibleColumns.map((status) => (
                <KanbanColumn
                  key={status}
                  status={status}
                  issues={board[status]}
                  onIssueClick={(id) => selectIssue(id)}
                />
              ))}
            </div>
          </div>
        ))}
      </div>

      <DragOverlay>
        {activeIssue && (
          <div className="rotate-2 scale-105">
            <IssueCard issue={activeIssue} />
          </div>
        )}
      </DragOverlay>
    </DndContext>
  );
}
