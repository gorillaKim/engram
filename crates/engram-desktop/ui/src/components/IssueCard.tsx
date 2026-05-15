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
      className="bg-white rounded-md shadow-sm hover:shadow-md border border-slate-200 p-3 cursor-grab active:cursor-grabbing space-y-1.5 touch-none"
      onClick={(e) => {
        if (!isDragging) onClick?.(issue.id);
        e.stopPropagation();
      }}
    >
      <div className="flex items-center gap-1.5">
        <PriorityBadge priority={issue.priority} />
        <span className="text-sm font-medium text-slate-800 line-clamp-2">{issue.title}</span>
        {scopeExpanded && (
          <span className="shrink-0 text-xs bg-amber-100 text-amber-700 border border-amber-300 rounded px-1 py-0.5 leading-none">
            ⚠팽창
          </span>
        )}
      </div>
      <div className="flex items-center justify-between text-xs text-slate-400">
        <span>#{issue.id}</span>
        {epicTitle && (
          <span className="bg-indigo-50 text-indigo-600 px-1.5 py-0.5 rounded truncate max-w-[120px]">
            {epicTitle}
          </span>
        )}
      </div>
    </div>
  );
}
