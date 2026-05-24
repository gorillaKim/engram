import type { TrayBoardSummary } from '../ipc/types';

interface Props {
  summary: TrayBoardSummary | null;
}

export function TraySummary({ summary }: Props) {
  const inbox = summary?.inbox ?? 0;
  const demo = summary?.demo_review ?? 0;
  const working = summary?.working ?? 0;

  return (
    <div className="flex items-center gap-5 text-sm">
      <span className="flex items-center gap-1.5">
        <span className="text-base">📦</span>
        <span className={`font-semibold ${inbox > 0 ? 'text-white/90' : 'text-white/40'}`}>{inbox}</span>
        <span className="text-white/40 text-xs">승인대기</span>
      </span>

      <span className={`flex items-center gap-1.5 ${demo > 0 ? 'text-amber-400' : ''}`}>
        <span className="text-base">👀</span>
        <span className={`font-semibold ${demo > 0 ? 'text-amber-400' : 'text-white/40'}`}>{demo}</span>
        <span className="text-white/40 text-xs">검토대기</span>
      </span>

      {working > 0 && (
        <span className="flex items-center gap-1.5 text-emerald-400">
          <span className="relative flex h-2 w-2">
            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75" />
            <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-500" />
          </span>
          <span className="font-semibold">{working}</span>
          <span className="text-xs text-white/40">작업중</span>
        </span>
      )}
    </div>
  );
}
