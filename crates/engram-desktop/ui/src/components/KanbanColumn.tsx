import { useDroppable } from '@dnd-kit/core';
import type { Issue } from '../ipc/types';
import { IssueCard } from './IssueCard';

type BoardColumn = 'required' | 'ready' | 'working' | 'demo' | 'finished';

const LABELS: Record<BoardColumn, string> = {
  required: 'Required',
  ready: 'Ready',
  working: 'Working',
  demo: 'Demo',
  finished: 'Finished',
};

interface Props {
  status: BoardColumn;
  issues: Issue[];
  onIssueClick?: (id: number) => void;
}

export function KanbanColumn({ status, issues, onIssueClick }: Props) {
  const { setNodeRef, isOver } = useDroppable({ id: status });
  const isDemo = status === 'demo';

  return (
    <div
      ref={setNodeRef}
      className={`flex flex-col min-h-[200px] rounded-lg p-3 transition-colors ${
        isOver ? 'ring-2 ring-indigo-400' :
        isDemo ? 'bg-amber-50 ring-1 ring-amber-200' : 'bg-slate-50'
      }`}
    >
      <div className="flex items-center justify-between mb-3">
        <span className="text-xs font-semibold uppercase tracking-wider text-slate-500">
          {LABELS[status]}
        </span>
        <span className="text-xs bg-slate-200 text-slate-600 rounded-full px-2 py-0.5">
          {issues.length}
        </span>
        {isDemo && issues.length > 0 && (
          <span className="text-xs bg-amber-100 text-amber-900 rounded px-1.5 py-0.5 ml-1">
            검토대기
          </span>
        )}
      </div>
      <div className="flex flex-col gap-2">
        {issues.map((issue) => (
          <IssueCard key={issue.id} issue={issue} onClick={onIssueClick} />
        ))}
      </div>
    </div>
  );
}
