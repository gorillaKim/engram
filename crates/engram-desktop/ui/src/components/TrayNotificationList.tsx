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
    return <p className="text-xs text-slate-400 px-1">최근 알림 없음</p>;
  }
  return (
    <ul className="flex flex-col gap-1">
      {entries.slice(0, 5).map((e) => (
        <li key={e.id} className="flex gap-2 text-xs">
          <span className="shrink-0 text-slate-400 w-16">{timeAgo(e.ts)}</span>
          <span className="text-slate-700">
            <span className="font-medium">{e.title}</span> {e.body}
          </span>
        </li>
      ))}
    </ul>
  );
}
