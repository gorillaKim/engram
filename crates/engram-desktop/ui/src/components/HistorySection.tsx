import { useState, useCallback } from 'react';
import { useQuery } from '@tanstack/react-query';
import { historyList } from '../ipc/invoke';
import type { HistoryEntry } from '../ipc/types';
import { Markdown } from './Markdown';

const ACTOR_LABEL: Record<string, string> = {
  user: '사용자',
  agent: 'AI 에이전트',
};

const PAGE = 10;

/** 마크다운 렌더가 의미 있는 필드 목록 */
const MD_FIELDS = new Set(['description', 'summary', 'body', 'content', 'note', 'comment']);

function isMarkdownField(field: string) {
  return MD_FIELDS.has(field.toLowerCase());
}

interface HistoryValueProps {
  field: string;
  oldValue?: string | null;
  newValue?: string | null;
  expanded: boolean;
  onToggle: () => void;
}

function HistoryValue({ field, oldValue, newValue, expanded, onToggle }: HistoryValueProps) {
  const useMd = isMarkdownField(field);

  if (useMd) {
    // 마크다운 필드: 구분선으로 before/after 블록 분리
    return (
      <div
        className="cursor-pointer space-y-1.5"
        onClick={onToggle}
      >
        {oldValue ? (
          <div className={`opacity-50 line-through-block ${expanded ? '' : 'line-clamp-2'}`}>
            <div className="text-[10px] text-slate-400 font-medium mb-0.5">이전</div>
            <div className={`overflow-hidden ${expanded ? '' : 'max-h-10'}`}>
              <Markdown className="[&_*]:text-[11px] [&_*]:leading-relaxed [&_p]:mb-0.5 opacity-60 line-through decoration-slate-400">
                {oldValue}
              </Markdown>
            </div>
          </div>
        ) : (
          <span className="text-slate-400 text-[11px]">∅</span>
        )}
        <div className="flex items-center gap-1">
          <span className="text-slate-300 text-[10px]">→</span>
        </div>
        <div>
          <div className="text-[10px] text-slate-400 font-medium mb-0.5">이후</div>
          <div className={`overflow-hidden ${expanded ? '' : 'max-h-14'}`}>
            {newValue ? (
              <Markdown className="[&_*]:text-[11px] [&_*]:leading-relaxed [&_p]:mb-0.5">
                {newValue}
              </Markdown>
            ) : (
              <span className="text-slate-400 text-[11px]">∅</span>
            )}
          </div>
        </div>
      </div>
    );
  }

  // 단순 필드: 인라인 before → after
  return (
    <div
      className={`text-slate-600 leading-relaxed cursor-pointer hover:text-slate-800 transition-colors ${expanded ? '' : 'line-clamp-2'}`}
      onClick={onToggle}
    >
      {oldValue ? (
        <span className="line-through text-slate-400">{oldValue}</span>
      ) : (
        <span className="text-slate-400">∅</span>
      )}
      <span className="mx-1 text-slate-400">→</span>
      <span className="text-slate-800 font-medium">{newValue ?? '∅'}</span>
    </div>
  );
}

interface Props {
  entityType: 'issue' | 'epic' | 'task' | 'sprint' | 'note';
  entityId: number;
}

export function HistorySection({ entityType, entityId }: Props) {
  const { data: entries = [] } = useQuery({
    queryKey: ['history', entityType, entityId],
    queryFn: () => historyList(entityType, entityId),
  });

  const [visible, setVisible] = useState(PAGE);
  const [expanded, setExpanded] = useState<Set<number>>(new Set());

  const toggleExpand = useCallback((id: number) => {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }, []);

  // 최신순 정렬
  const sorted = entries.slice().reverse();
  const shown = sorted.slice(0, visible);
  const hasMore = visible < sorted.length;

  return (
    <section>
      <h3 className="text-xs font-semibold text-slate-400 uppercase tracking-wider mb-2">
        변경 이력 ({entries.length})
      </h3>

      {entries.length === 0 && (
        <p className="text-xs text-slate-400">이력 없음</p>
      )}

      <ul className="space-y-1.5">
        {shown.map((h: HistoryEntry) => {
          const isAgent = h.changed_by === 'agent';
          const actor = ACTOR_LABEL[h.changed_by] ?? h.changed_by;
          const isExpanded = expanded.has(h.id);
          return (
            <li key={h.id} className="rounded-lg border border-slate-100 bg-slate-50/60 px-3 py-2 text-xs">
              {/* Meta row */}
              <div className="flex items-center justify-between gap-2 mb-1.5">
                <span className="text-slate-400 font-mono tabular-nums">
                  {h.created_at.slice(0, 16).replace('T', ' ')}
                </span>
                <span className={`shrink-0 rounded-full px-2 py-0.5 text-[10px] font-semibold ${
                  isAgent
                    ? 'bg-indigo-100 text-indigo-600'
                    : 'bg-slate-200 text-slate-600'
                }`}>
                  {actor}
                </span>
              </div>

              {/* Field badge + value */}
              <div className="flex items-start gap-2">
                <span className="shrink-0 mt-0.5 rounded px-1.5 py-0.5 text-[10px] font-semibold bg-slate-200 text-slate-500 uppercase tracking-wide">
                  {h.field}
                </span>
                <div className="min-w-0 flex-1">
                  <HistoryValue
                    field={h.field}
                    oldValue={h.old_value}
                    newValue={h.new_value}
                    expanded={isExpanded}
                    onToggle={() => toggleExpand(h.id)}
                  />
                  <button
                    type="button"
                    onClick={() => toggleExpand(h.id)}
                    className="mt-1 text-indigo-400 hover:text-indigo-600 text-[10px] font-medium"
                  >
                    {isExpanded ? '접기 ▲' : '펼치기 ▼'}
                  </button>
                </div>
              </div>
            </li>
          );
        })}
      </ul>

      {hasMore && (
        <button
          type="button"
          onClick={() => setVisible((v) => v + PAGE)}
          className="mt-2 text-xs text-indigo-600 hover:text-indigo-800 hover:underline"
        >
          더보기 ({sorted.length - visible}건 남음)
        </button>
      )}
    </section>
  );
}
