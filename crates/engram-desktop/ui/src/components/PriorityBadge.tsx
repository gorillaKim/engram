import type { IssuePriority } from '../ipc/types';

const colors: Record<IssuePriority, string> = {
  critical: 'bg-red-500',
  high: 'bg-orange-500',
  medium: 'bg-amber-400',
  low: 'bg-slate-400',
};

export function PriorityBadge({ priority }: { priority: IssuePriority }) {
  return <span className={`inline-block w-2 h-2 rounded-full flex-shrink-0 ${colors[priority]}`} title={priority} />;
}
