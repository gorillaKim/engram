import { useDraggable } from '@dnd-kit/core';
import type { Issue } from '../ipc/types';
import { PriorityBadge } from './PriorityBadge';
import { CopyableId } from './CopyableId';
import { PromptButton } from './PromptButton';
import { parseUTCDate } from '../utils/date';

function relativeTime(iso: string): string {
  const diff = Date.now() - parseUTCDate(iso).getTime();
  if (diff < 0) return '방금';
  const mins = Math.floor(diff / 60_000);
  if (mins < 1) return '방금';
  if (mins < 60) return `${mins}분 전`;
  const hrs = Math.floor(mins / 60);
  if (hrs < 24) return `${hrs}시간 전`;
  const days = Math.floor(hrs / 24);
  if (days < 30) return `${days}일 전`;
  return `${Math.floor(days / 30)}개월 전`;
}

interface Props {
  issue: Issue;
  epicTitle?: string;
  scopeExpanded?: boolean;
  onClick?: (id: number) => void;
}

const cardClass = "bg-white rounded-lg shadow-sm hover:shadow-md border border-slate-200 p-4 space-y-2.5 transition-all hover:-translate-y-0.5 touch-none";

function CardContent({ issue, epicTitle, scopeExpanded }: Pick<Props, 'issue' | 'epicTitle' | 'scopeExpanded'>) {
  return (
    <>
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

      {/* 에픽 배지 (있는 경우 별도 행으로 개행되어 깔끔하게 표시) */}
      {epicTitle && (
        <div className="pt-0.5">
          <span className="inline-flex items-center gap-1.5 max-w-full bg-indigo-50/80 text-indigo-700 border border-indigo-200/60 px-2 py-0.5 rounded-md text-[10.5px] font-medium">
            <span className="w-1.5 h-1.5 rounded-full bg-indigo-400 shrink-0" />
            <span className="truncate min-w-0">{epicTitle}</span>
          </span>
        </div>
      )}

      {/* 하위 메타 행: 티켓 ID 복사, Prompt 버튼, 에이전트, 수정 시각 */}
      <div className="flex items-center justify-between gap-2 pt-1 border-t border-slate-100/60">
        <div className="flex items-center gap-2 min-w-0 flex-wrap shrink-0">
          <CopyableId type="issue" id={issue.id} prefix="#" className="text-[11px] font-medium text-slate-400 shrink-0" />
          <PromptButton type="issue" id={issue.id} title={issue.title} goal={issue.goal} size="xs" className="shrink-0" />
          {issue.assigned_agent && (
            <span className="inline-flex items-center gap-0.5 bg-violet-50 text-violet-600 border border-violet-200 px-1.5 py-0.5 rounded text-[10px] shrink-0" title={issue.assigned_agent}>
              🤖
            </span>
          )}
        </div>
        <span className="text-[10px] text-slate-400 shrink-0">{relativeTime(issue.updated_at)}</span>
      </div>
    </>
  );
}

export function IssueCardView({ issue, epicTitle, scopeExpanded }: Omit<Props, 'onClick'>) {
  return (
    <div className={cardClass}>
      <CardContent issue={issue} epicTitle={epicTitle} scopeExpanded={scopeExpanded} />
    </div>
  );
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
      className={`${cardClass} cursor-grab active:cursor-grabbing`}
      onClick={(e) => {
        if (!isDragging) onClick?.(issue.id);
        e.stopPropagation();
      }}
    >
      <CardContent issue={issue} epicTitle={epicTitle} scopeExpanded={scopeExpanded} />
    </div>
  );
}
