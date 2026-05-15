import type { TrayBoardSummary } from '../ipc/types';

interface Props {
  summary: TrayBoardSummary | null;
}

export function TraySummary({ summary }: Props) {
  const inbox = summary?.inbox ?? 0;
  const demo = summary?.demo_review ?? 0;
  const blockers = summary?.blockers ?? 0;

  return (
    <div className="flex items-center gap-4 px-4 py-2 bg-slate-50 rounded-lg text-sm">
      <span className="flex items-center gap-1 text-slate-600">
        <span className="text-base">📦</span>
        <span className="font-semibold">{inbox}</span>
        <span className="text-slate-400 text-xs">승인대기</span>
      </span>
      <span className={`flex items-center gap-1 ${demo > 0 ? 'text-amber-700' : 'text-slate-600'}`}>
        <span className="text-base">👀</span>
        <span className="font-semibold">{demo}</span>
        <span className="text-xs opacity-70">검토대기</span>
      </span>
      {blockers > 0 && (
        <span className="flex items-center gap-1 text-red-600">
          <span className="text-base">🚫</span>
          <span className="font-semibold">{blockers}</span>
          <span className="text-xs">블로커</span>
        </span>
      )}
    </div>
  );
}
