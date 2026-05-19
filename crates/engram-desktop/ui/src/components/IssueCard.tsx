import { useDraggable } from '@dnd-kit/core';
import type { Issue } from '../ipc/types';
import { PriorityBadge } from './PriorityBadge';

interface Props {
  issue: Issue;
  epicTitle?: string;
  scopeExpanded?: boolean;
  onClick?: (id: number) => void;
}

export function IssueCard({ issue, epicTitle, scopeExpanded, onClick }: Props) {
  const { attributes, listeners, setNodeRef, isDragging } = useDraggable({
    id: issue.id,
    data: { issue },
  });

  return (
    <div
      ref={setNodeRef}
      {...listeners}
      {...attributes}
      style={{ opacity: isDragging ? 0.4 : 1 }}
      className="bg-white rounded-lg shadow-sm hover:shadow-md border border-slate-200 p-4 cursor-grab active:cursor-grabbing space-y-2.5 transition-all hover:-translate-y-0.5 touch-none"
      onClick={(e) => {
        if (!isDragging) onClick?.(issue.id);
        e.stopPropagation();
      }}
    >
      <div className="flex items-start justify-between gap-2">
        <span className="text-sm font-semibold text-slate-800 leading-snug line-clamp-2">
          {issue.title}
        </span>
        <PriorityBadge priority={issue.priority} />
      </div>

      {scopeExpanded && (
        <div className="flex items-center gap-1.5 px-2 py-1 bg-amber-50 border border-amber-200 rounded text-[11px] font-medium text-amber-700">
          <span className="text-sm leading-none">⚠</span>
          <span>스코프 팽창 감지</span>
        </div>
      )}

      <div className="flex items-center justify-between gap-2 pt-1 min-w-0">
        <span className="text-[11px] font-medium text-slate-400 flex-shrink-0">#{issue.id}</span>
        {epicTitle && (
          <span className="inline-flex items-center gap-1 min-w-0 max-w-[140px] bg-indigo-50 text-indigo-600 border border-indigo-200 px-2 py-0.5 rounded-full">
            <span className="w-1.5 h-1.5 rounded-full bg-indigo-400 flex-shrink-0" />
            <span className="text-[10px] font-medium truncate min-w-0">{epicTitle}</span>
          </span>
        )}
      </div>
    </div>
  );
}
