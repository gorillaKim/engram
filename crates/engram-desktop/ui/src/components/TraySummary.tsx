import type { TrayBoardSummary } from '../ipc/types';

interface Props {
  summary: TrayBoardSummary | null;
}

const WORKING_STATE_CONFIG = {
  active:  { color: 'text-emerald-400', dot: 'bg-emerald-500', pingColor: 'bg-emerald-400', label: '작업중',   animate: true  },
  pending: { color: 'text-sky-400',     dot: 'bg-sky-500',     pingColor: 'bg-sky-400',     label: '작업예상', animate: false },
  stalled: { color: 'text-red-400',     dot: 'bg-red-500',     pingColor: 'bg-red-400',     label: '작업중단', animate: false },
  none:    { color: 'text-white/40',    dot: 'bg-white/40',    pingColor: 'bg-white/40',    label: '작업중',   animate: false },
} as const;

export function TraySummary({ summary }: Props) {
  const inbox = summary?.inbox ?? 0;
  const demo = summary?.demo_review ?? 0;
  const working = summary?.working ?? 0;
  const state = summary?.working_state ?? 'none';
  const cfg = WORKING_STATE_CONFIG[state] ?? WORKING_STATE_CONFIG.none;

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
        <span className={`flex items-center gap-1.5 ${cfg.color}`}>
          <span className="relative flex h-2 w-2">
            {cfg.animate && (
              <span className={`animate-ping absolute inline-flex h-full w-full rounded-full ${cfg.pingColor} opacity-75`} />
            )}
            <span className={`relative inline-flex rounded-full h-2 w-2 ${cfg.dot}`} />
          </span>
          <span className="font-semibold">{working}</span>
          <span className="text-xs text-white/40">{cfg.label}</span>
        </span>
      )}
    </div>
  );
}
