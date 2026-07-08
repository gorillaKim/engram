import type { NotificationEntry } from '../store/notification';
import { BellOff } from 'lucide-react';

interface Props {
  entries: NotificationEntry[];
}

function timeAgo(ts: number): string {
  const diff = (Date.now() - ts) / 1000;
  if (diff < 60) return '방금';
  if (diff < 3600) return `${Math.floor(diff / 60)}분 전`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}시간 전`;
  return `${Math.floor(diff / 86400)}일 전`;
}

export function TrayNotificationList({ entries }: Props) {
  if (entries.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-7 gap-2 text-white/20 bg-white/[0.01] border border-dashed border-white/[0.05] rounded-xl">
        <BellOff className="h-6 w-6 stroke-[1.2] opacity-80" />
        <p className="text-[11px] font-medium tracking-wide">최근 수신된 알림이 없습니다</p>
      </div>
    );
  }

  return (
    <ul className="flex flex-col gap-1 -mx-1">
      {entries.slice(0, 8).map((e) => (
        <li 
          key={e.id} 
          className="relative flex gap-3 text-xs py-2 px-2.5 rounded-xl hover:bg-white/[0.03] border border-transparent hover:border-white/[0.04] transition-all duration-200 group cursor-default"
        >
          {/* 호버 시 좌측 보더 포인트 라인 */}
          <span className="absolute left-0 top-1/2 -translate-y-1/2 w-[3px] h-0 bg-sky-400 group-hover:h-3.5 transition-all duration-200 rounded-full" />
          
          <span className="shrink-0 text-white/30 w-11 font-medium text-[10px] pt-0.5 tabular-nums text-right">
            {timeAgo(e.ts)}
          </span>
          <span className="text-white/60 group-hover:text-white/85 transition-colors leading-relaxed">
            <span className="font-semibold text-white/80 group-hover:text-white/95">{e.title}</span>
            <span className="mx-1 text-white/25">·</span>
            {e.body}
          </span>
        </li>
      ))}
    </ul>
  );
}

