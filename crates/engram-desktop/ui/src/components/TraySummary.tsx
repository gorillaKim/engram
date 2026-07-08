import type { TrayBoardSummary, TrayStallEntry } from '../ipc/types';
import { Inbox, Eye, Activity, AlertTriangle } from 'lucide-react';

interface Props {
  summary: TrayBoardSummary | null;
}

const WORKING_STATE_CONFIG = {
  active:  { color: 'text-emerald-400', dot: 'bg-emerald-500', pingColor: 'bg-emerald-400', label: '작업중',   animate: true  },
  pending: { color: 'text-sky-400',     dot: 'bg-sky-500',     pingColor: 'bg-sky-400',     label: '작업예상', animate: false },
  stalled: { color: 'text-rose-400',     dot: 'bg-rose-500',     pingColor: 'bg-rose-400',     label: '작업중단', animate: false },
  none:    { color: 'text-white/30',    dot: 'bg-white/30',    pingColor: 'bg-white/30',    label: '작업중',   animate: false },
} as const;

export function TraySummary({ summary }: Props) {
  const inbox = summary?.inbox ?? 0;
  const demo = summary?.demo_review ?? 0;
  const working = summary?.working ?? 0;
  const state = summary?.working_state ?? 'none';
  const stalledIssues: TrayStallEntry[] = summary?.stalled_issues ?? [];
  const stalledTotal = summary?.stalled_total ?? 0;
  const cfg = WORKING_STATE_CONFIG[state] ?? WORKING_STATE_CONFIG.none;

  return (
    <div className="space-y-3">
      {/* 3열 대시보드 미니 카드 그리드 */}
      <div className="grid grid-cols-3 gap-2">
        {/* 승인 대기 */}
        <div className="bg-white/[0.02] border border-white/[0.04] rounded-xl p-2.5 flex flex-col items-center justify-center gap-1 transition-all duration-200 hover:bg-white/[0.04]">
          <Inbox className={`h-4 w-4 ${inbox > 0 ? 'text-sky-400' : 'text-white/20'}`} />
          <span className={`text-base font-bold tabular-nums ${inbox > 0 ? 'text-white/95' : 'text-white/30'}`}>
            {inbox}
          </span>
          <span className="text-[10px] text-white/40 font-medium">승인대기</span>
        </div>

        {/* 검토 대기 */}
        <div className="bg-white/[0.02] border border-white/[0.04] rounded-xl p-2.5 flex flex-col items-center justify-center gap-1 transition-all duration-200 hover:bg-white/[0.04]">
          <Eye className={`h-4 w-4 ${demo > 0 ? 'text-amber-400 animate-pulse' : 'text-white/20'}`} />
          <span className={`text-base font-bold tabular-nums ${demo > 0 ? 'text-amber-400' : 'text-white/30'}`}>
            {demo}
          </span>
          <span className="text-[10px] text-white/40 font-medium">검토대기</span>
        </div>

        {/* 진행 중 */}
        <div className="bg-white/[0.02] border border-white/[0.04] rounded-xl p-2.5 flex flex-col items-center justify-center gap-1 transition-all duration-200 hover:bg-white/[0.04]">
          <div className="relative flex items-center justify-center">
            <Activity className={`h-4 w-4 ${working > 0 ? cfg.color : 'text-white/20'}`} />
            {working > 0 && cfg.animate && (
              <span className="absolute -top-0.5 -right-0.5 flex h-1.5 w-1.5">
                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75" />
                <span className="relative inline-flex rounded-full h-1.5 w-1.5 bg-emerald-500" />
              </span>
            )}
          </div>
          <span className={`text-base font-bold tabular-nums ${working > 0 ? 'text-white/95' : 'text-white/30'}`}>
            {working}
          </span>
          <span className="text-[10px] text-white/40 font-medium">
            {working > 0 ? cfg.label : '진행중'}
          </span>
        </div>
      </div>

      {/* 작업 중단 의심 이슈 상세 */}
      {state === 'stalled' && stalledIssues.length > 0 && (
        <div className="bg-rose-500/[0.04] border border-rose-500/10 rounded-xl p-3 space-y-2">
          <div className="flex items-center gap-1.5 text-rose-400 text-xs font-semibold">
            <AlertTriangle className="h-3.5 w-3.5 shrink-0" />
            <span>작업 중단 의심 이슈 ({stalledTotal})</span>
          </div>
          <div className="space-y-1.5">
            {stalledIssues.map((s) => (
              <div key={s.id} className="flex items-start gap-1.5 text-[11px] text-rose-400/80 leading-normal">
                <span className="text-rose-500/60 shrink-0">•</span>
                <span className="truncate hover:text-rose-300 transition-colors cursor-default">
                  #{s.id} {s.title}
                </span>
              </div>
            ))}
            {stalledTotal > stalledIssues.length && (
              <div className="text-[10px] text-white/35 pl-3">
                그 외 {stalledTotal - stalledIssues.length}개 더 있음
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

