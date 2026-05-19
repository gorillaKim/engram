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
  epicMap?: Map<number, string>;
  onCreateIssue?: () => void;
}

export function KanbanColumn({ status, issues, onIssueClick, expansionIds, epicMap, onCreateIssue }: Props) {
  const { setNodeRef, isOver } = useDroppable({ id: status });
  const isDemo = status === 'demo';
  const isCancelled = status === 'cancelled';
  const isRequired = status === 'required';

  return (
    <div
      ref={setNodeRef}
      className={`flex flex-col min-h-[400px] rounded-xl p-4 transition-all duration-200 border-2 ${
        isOver ? 'bg-indigo-50/50 border-indigo-300 ring-4 ring-indigo-50' :
        isDemo ? 'bg-amber-50/40 border-amber-100' :
        isCancelled ? 'bg-slate-100/50 border-transparent opacity-60' : 
        'bg-slate-100/40 border-transparent'
      }`}
    >
      <div className="flex items-center justify-between mb-4 px-1">
        <div className="flex items-center gap-2">
          <span className="text-[11px] font-bold uppercase tracking-widest text-slate-500">
            {LABELS[status]}
          </span>
          <span className={`text-[11px] font-bold px-2 py-0.5 rounded-full ${
            issues.length > 0 ? 'bg-slate-200 text-slate-700' : 'bg-slate-100 text-slate-400'
          }`}>
            {issues.length}
          </span>
          {isDemo && issues.length > 0 && (
            <span className="text-[10px] font-bold bg-amber-200 text-amber-900 rounded px-2 py-0.5 animate-pulse">
              검토 대기
            </span>
          )}
        </div>
        {isRequired && onCreateIssue && (
          <button
            type="button"
            onClick={onCreateIssue}
            title="이슈 생성"
            className="w-6 h-6 flex items-center justify-center text-slate-400 hover:text-indigo-600 hover:bg-white hover:shadow-sm rounded-full transition-all"
          >
            <span className="text-lg font-light leading-none">+</span>
          </button>
        )}
      </div>

      <div className="flex flex-col gap-3 flex-1">
        {issues.length === 0 ? (
          <div className="flex-1 flex flex-col items-center justify-center border-2 border-dashed border-slate-200/60 rounded-lg p-6 grayscale opacity-40">
             <div className="text-2xl mb-2">📥</div>
             <p className="text-[11px] font-medium text-slate-400 text-center">이슈 없음</p>
          </div>
        ) : (
          issues.map((issue) => (
            <IssueCard
              key={issue.id}
              issue={issue}
              onClick={onIssueClick}
              scopeExpanded={expansionIds?.has(issue.id)}
              epicTitle={epicMap?.get(issue.epic_id)}
            />
          ))
        )}
      </div>
    </div>
  );
}
