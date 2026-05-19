import type { NotificationEntry } from '../store/notification';

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
      <div className="py-2 px-1">
        <p className="text-xs text-white/20 italic">최근 알림 없음</p>
      </div>
    );
  }
  return (
    <ul className="flex flex-col -mx-2">
      {entries.slice(0, 8).map((e) => (
        <li 
          key={e.id} 
          className="flex gap-2 text-xs py-1.5 px-2 rounded-md hover:bg-white/[0.05] transition-colors group cursor-default"
        >
          <span className="shrink-0 text-white/25 w-12 font-medium">{timeAgo(e.ts)}</span>
          <span className="text-white/65 group-hover:text-white/85 transition-colors leading-relaxed">
            <span className="font-semibold text-white/80 group-hover:text-white/95">{e.title}</span>{' '}{e.body}
          </span>
        </li>
      ))}
    </ul>
  );
}
