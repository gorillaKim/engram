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

/* ── Standalone column header (sticky 헤더 행에서 사용) ── */
interface HeaderProps {
  status: BoardColumn;
  issueCount: number;
  onCreateIssue?: () => void;
  onBulkFinish?: () => void;
}

export function KanbanColumnHeader({ status, issueCount, onCreateIssue, onBulkFinish }: HeaderProps) {
  const isDemo = status === 'demo';
  const isRequired = status === 'required';

  return (
    <div className={`flex items-center justify-between px-4 py-2.5 rounded-lg ${
      isDemo ? 'bg-amber-50/80 border border-amber-100' : 'bg-slate-100/60'
    }`}>
      <div className="flex items-center gap-2 flex-wrap">
        <span className="text-[11px] font-bold uppercase tracking-widest text-slate-500">
          {LABELS[status]}
        </span>
        <span className={`text-[11px] font-bold px-2 py-0.5 rounded-full ${
          issueCount > 0 ? 'bg-slate-200 text-slate-700' : 'bg-slate-100 text-slate-400'
        }`}>
          {issueCount}
        </span>
        {isDemo && issueCount > 0 && (
          <span className="text-[10px] font-bold bg-amber-200 text-amber-900 rounded px-2 py-0.5 animate-pulse">
            검토 대기
          </span>
        )}
      </div>
      <div className="flex items-center gap-1">
        {isDemo && issueCount > 0 && onBulkFinish && (
          <button
            type="button"
            onClick={(e) => { e.stopPropagation(); onBulkFinish(); }}
            title="보이는 DEMO 이슈 모두 완료 처리"
            className="text-[10px] font-semibold px-2 py-1 bg-emerald-600 hover:bg-emerald-500 text-white rounded-md transition-colors flex items-center gap-1"
          >
            <span>✓</span>
            <span>일괄 완료</span>
          </button>
        )}
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
    </div>
  );
}

/* ── Column body (droppable + cards) ── */
interface Props {
  projectKey: string;
  status: BoardColumn;
  issues: Issue[];
  onIssueClick?: (id: number) => void;
  expansionIds?: Set<number>;
  epicMap?: Map<number, string>;
}

export function KanbanColumn({ projectKey, status, issues, onIssueClick, expansionIds, epicMap }: Props) {
  const { setNodeRef, isOver } = useDroppable({ id: `${projectKey}-${status}` });
  const isDemo = status === 'demo';
  const isCancelled = status === 'cancelled';

  return (
    <div
      ref={setNodeRef}
      className={`flex flex-col min-h-[300px] min-w-[280px] shrink-0 rounded-xl p-4 pt-3 transition-all duration-200 border-2 ${
        isOver ? 'bg-indigo-50/50 border-indigo-300 ring-4 ring-indigo-50' :
        isDemo ? 'bg-amber-50/40 border-amber-100' :
        isCancelled ? 'bg-slate-100/50 border-transparent opacity-60' : 
        'bg-slate-100/40 border-transparent'
      }`}
    >
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
