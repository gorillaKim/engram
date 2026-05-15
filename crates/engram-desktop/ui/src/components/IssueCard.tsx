import { useDraggable } from '@dnd-kit/core';
import type { Issue } from '../ipc/types';
import { PriorityBadge } from './PriorityBadge';

interface Props {
  issue: Issue;
  epicTitle?: string;
  onClick?: (id: number) => void;
}

export function IssueCard({ issue, epicTitle, onClick }: Props) {
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
        // suppress click when drag distance was ≥ 5px (dnd-kit handles activation)
        if (!isDragging) onClick?.(issue.id);
        e.stopPropagation();
      }}
    >
      <div className="flex items-center gap-1.5">
        <PriorityBadge priority={issue.priority} />
        <span className="text-sm font-medium text-slate-800 line-clamp-2">{issue.title}</span>
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
