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
    return <p className="text-xs text-white/30">최근 알림 없음</p>;
  }
  return (
    <ul className="flex flex-col gap-1.5">
      {entries.slice(0, 5).map((e) => (
        <li key={e.id} className="flex gap-2 text-xs">
          <span className="shrink-0 text-white/30 w-14">{timeAgo(e.ts)}</span>
          <span className="text-white/70">
            <span className="font-medium text-white/85">{e.title}</span>{' '}{e.body}
          </span>
        </li>
      ))}
    </ul>
  );
}
