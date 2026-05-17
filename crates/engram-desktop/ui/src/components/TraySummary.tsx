import type { TrayBoardSummary } from '../ipc/types';

interface Props {
  summary: TrayBoardSummary | null;
}

export function TraySummary({ summary }: Props) {
  const inbox = summary?.inbox ?? 0;
  const demo = summary?.demo_review ?? 0;
  const blockers = summary?.blockers ?? 0;

  return (
    <div className="flex items-center gap-5 text-sm">
      <span className="flex items-center gap-1.5">
        <span className="text-base">📦</span>
        <span className="font-semibold text-white/90">{inbox}</span>
        <span className="text-white/40 text-xs">승인대기</span>
      </span>
      <span className={`flex items-center gap-1.5 ${demo > 0 ? 'text-amber-400' : ''}`}>
        <span className="text-base">👀</span>
        <span className={`font-semibold ${demo > 0 ? 'text-amber-400' : 'text-white/90'}`}>{demo}</span>
        <span className="text-white/40 text-xs">검토대기</span>
      </span>
      {blockers > 0 && (
        <span className="flex items-center gap-1.5 text-red-400">
          <span className="text-base">🚫</span>
          <span className="font-semibold">{blockers}</span>
          <span className="text-xs text-white/40">블로커</span>
        </span>
      )}
    </div>
  );
}
