import type { StalledIssueBrief } from '../ipc/types';

interface Props {
  issues: StalledIssueBrief[];
  onIssueClick?: (id: number) => void;
}

function formatElapsed(secs: number | null): string {
  if (secs == null) return '기록 없음';
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  if (h > 0) return `${h}시간 ${m}분 전`;
  return `${m}분 전`;
}

export function StalledWorkingBanner({ issues, onIssueClick }: Props) {
  if (issues.length === 0) return null;

  return (
    <div className="rounded-lg border border-red-500/30 bg-red-500/10 px-4 py-3">
      <div className="flex items-center gap-2 mb-2">
        <span className="text-red-400 text-sm font-semibold">⚠ 작업중단 의심 이슈 {issues.length}개</span>
        <span className="text-xs text-white/40">히스토리 갱신이 오래된 working 이슈입니다</span>
      </div>
      <div className="flex flex-wrap gap-2">
        {issues.map((issue) => (
          <button
            key={issue.id}
            type="button"
            onClick={() => onIssueClick?.(issue.id)}
            className="flex items-center gap-1.5 px-2 py-1 rounded bg-red-500/20 hover:bg-red-500/30 text-xs text-red-300 transition-colors cursor-default"
          >
            <span className="font-mono text-red-400/70">#{issue.id}</span>
            <span className="max-w-[160px] truncate">{issue.title}</span>
            <span className="text-red-400/50 ml-1">{formatElapsed(issue.secs_since_activity)}</span>
          </button>
        ))}
      </div>
    </div>
  );
}
