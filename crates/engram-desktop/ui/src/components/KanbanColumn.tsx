import { useDroppable } from '@dnd-kit/core';
import type { Issue } from '../ipc/types';
import { IssueCard } from './IssueCard';

type BoardColumn = 'required' | 'ready' | 'working' | 'demo' | 'finished' | 'cancelled';

const LABELS: Record<BoardColumn, string> = {
  required: 'Required',
  ready: 'Ready',
  working: 'Working',
  demo: 'Demo',
  finished: 'Finished',
  cancelled: 'Cancelled',
};

interface Props {
  status: BoardColumn;
  issues: Issue[];
  onIssueClick?: (id: number) => void;
  expansionIds?: Set<number>;
  onCreateIssue?: () => void;
}

export function KanbanColumn({ status, issues, onIssueClick, expansionIds, onCreateIssue }: Props) {
  const { setNodeRef, isOver } = useDroppable({ id: status });
  const isDemo = status === 'demo';
  const isCancelled = status === 'cancelled';
  const isRequired = status === 'required';

  return (
    <div
      ref={setNodeRef}
      className={`flex flex-col min-h-[200px] rounded-lg p-3 transition-colors ${
        isOver ? 'ring-2 ring-indigo-400' :
        isDemo ? 'bg-amber-50 ring-1 ring-amber-200' :
        isCancelled ? 'bg-slate-100 opacity-70' : 'bg-slate-50'
      }`}
    >
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-1.5">
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
        {isRequired && onCreateIssue && (
          <button
            type="button"
            onClick={onCreateIssue}
            title="이슈 생성"
            className="text-slate-400 hover:text-indigo-600 text-base leading-none px-1 rounded hover:bg-slate-200"
          >
            +
          </button>
        )}
      </div>
      <div className="flex flex-col gap-2">
        {issues.map((issue) => (
          <IssueCard
            key={issue.id}
            issue={issue}
            onClick={onIssueClick}
            scopeExpanded={expansionIds?.has(issue.id)}
          />
        ))}
      </div>
    </div>
  );
}
