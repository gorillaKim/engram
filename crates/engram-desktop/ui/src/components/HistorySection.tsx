import { useQuery } from '@tanstack/react-query';
import { historyList } from '../ipc/invoke';
import type { HistoryEntry } from '../ipc/types';

const ACTOR_LABEL: Record<string, string> = {
  user: '사용자',
  agent: 'AI 에이전트',
};

interface Props {
  entityType: 'issue' | 'epic' | 'task' | 'sprint' | 'note';
  entityId: number;
}

export function HistorySection({ entityType, entityId }: Props) {
  const { data: entries = [] } = useQuery({
    queryKey: ['history', entityType, entityId],
    queryFn: () => historyList(entityType, entityId),
  });

  return (
    <section>
      <h3 className="text-xs font-semibold text-slate-400 uppercase tracking-wider mb-2">
        변경 이력 ({entries.length})
      </h3>

      {entries.length === 0 && (
        <p className="text-xs text-slate-400">이력 없음</p>
      )}

      <ul className="space-y-1">
        {entries.slice().reverse().map((h: HistoryEntry) => {
          const actor = ACTOR_LABEL[h.changed_by] ?? h.changed_by;
          return (
            <li key={h.id} className="text-xs text-slate-600 flex items-baseline gap-2">
              <span className="text-slate-400 shrink-0">{h.created_at.slice(0, 16).replace('T', ' ')}</span>
              <span className="font-medium text-slate-700">{actor}</span>
              <span className="text-slate-500">
                {h.field}: {h.old_value ? <span className="line-through opacity-60">{h.old_value}</span> : <span className="opacity-60">∅</span>}
                {' → '}
                <span className="text-slate-800">{h.new_value ?? '∅'}</span>
              </span>
            </li>
          );
        })}
      </ul>
    </section>
  );
}
