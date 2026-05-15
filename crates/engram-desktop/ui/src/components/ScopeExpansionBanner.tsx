import { useState } from 'react';

interface Props {
  warnings: string[];
  onIssueClick?: (id: number) => void;
}

export function ScopeExpansionBanner({ warnings, onIssueClick }: Props) {
  const [expanded, setExpanded] = useState(false);

  const scopeWarnings = warnings.filter((w) => w.startsWith('스코프 팽창 감지'));
  if (scopeWarnings.length === 0) return null;

  return (
    <div className="rounded-md border border-amber-200 bg-amber-50 px-4 py-2.5 text-sm">
      <div className="flex items-center justify-between">
        <span className="font-medium text-amber-800">
          ⚠ 스코프 팽창 감지 ({scopeWarnings.length}건)
        </span>
        <button
          onClick={() => setExpanded((v) => !v)}
          className="text-xs text-amber-600 hover:text-amber-800"
        >
          {expanded ? '접기' : '상세 보기'}
        </button>
      </div>

      {expanded && (
        <ul className="mt-2 space-y-1">
          {scopeWarnings.map((w, i) => {
            // Extract issue ID to make it clickable
            const match = w.match(/이슈 #(\d+)/);
            const issueId = match ? parseInt(match[1], 10) : null;
            return (
              <li key={i} className="text-amber-700 text-xs flex items-start gap-1.5">
                <span className="shrink-0">•</span>
                <span>
                  {issueId && onIssueClick ? (
                    <>
                      <button
                        onClick={() => onIssueClick(issueId)}
                        className="font-semibold underline hover:text-amber-900"
                      >
                        #{issueId}
                      </button>{' '}
                      {w.replace(/이슈 #\d+/, '').trimStart()}
                    </>
                  ) : (
                    w
                  )}
                </span>
              </li>
            );
          })}
        </ul>
      )}
    </div>
  );
}
