import { useMemo } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { issueGet, issueSetStatus, blockedIssuesGraph } from '../ipc/invoke';
import { TaskChecklist } from '../components/TaskChecklist';
import { NoteList } from '../components/NoteList';
import { PriorityBadge } from '../components/PriorityBadge';
import { BlockingGraphView } from '../components/BlockingGraph';
import { useUIStore } from '../store/ui';
import { useBoardStatus } from '../hooks/useBoardStatus';
import type { Issue } from '../ipc/types';

export function IssueDetail() {
  const { selectedIssueId, selectedProjectKey, selectIssue } = useUIStore();
  const qc = useQueryClient();

  const { data: issue } = useQuery({
    queryKey: ['issue', selectedIssueId],
    queryFn: () => issueGet(selectedIssueId!),
    enabled: selectedIssueId != null,
  });

  const { data: graphData } = useQuery({
    queryKey: ['blockingGraph', selectedProjectKey],
    queryFn: () => blockedIssuesGraph(selectedProjectKey!),
    enabled: selectedIssueId != null && selectedProjectKey != null,
    staleTime: 10_000,
  });

  // Build issue title map from board data for graph node labels
  const { data: boardData } = useBoardStatus(selectedProjectKey ?? undefined);
  const issueTitles = useIssueTitleMap(boardData?.boards ?? []);

  const transition = useMutation({
    mutationFn: (status: string) => issueSetStatus(selectedIssueId!, status),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['issue', selectedIssueId] });
    },
    onError: (err) => toast.error(`전이 실패: ${err}`),
  });

  if (!selectedIssueId) return null;

  return (
    <div
      className="fixed inset-0 z-40 flex justify-end bg-black/30"
      onClick={(e) => { if (e.target === e.currentTarget) selectIssue(null); }}
    >
      <div className="relative w-full max-w-lg bg-white shadow-2xl flex flex-col h-full overflow-y-auto">
        {/* Header */}
        <div className="flex items-start justify-between p-5 border-b border-slate-200 sticky top-0 bg-white z-10">
          <div className="flex items-center gap-2 min-w-0">
            {issue && <PriorityBadge priority={issue.priority} />}
            <h2 className="text-base font-semibold text-slate-800 truncate">
              {issue ? `#${issue.id} ${issue.title}` : '…'}
            </h2>
          </div>
          <button
            onClick={() => selectIssue(null)}
            className="ml-3 shrink-0 text-slate-400 hover:text-slate-600 text-lg leading-none"
          >
            ×
          </button>
        </div>

        {issue && (
          <div className="p-5 flex flex-col gap-6">
            {/* Status badge */}
            <div className="flex items-center gap-2">
              <span className="text-xs text-slate-500">상태</span>
              <span className="text-xs font-medium bg-slate-100 rounded px-2 py-0.5 capitalize">
                {issue.status}
              </span>
            </div>

            {/* Goal */}
            {issue.goal && (
              <div className="bg-indigo-50 rounded-md p-3 text-sm text-indigo-800">
                <span className="font-semibold">목표: </span>{issue.goal}
              </div>
            )}

            {/* Description */}
            {issue.description && (
              <p className="text-sm text-slate-600 whitespace-pre-wrap">{issue.description}</p>
            )}

            {/* Blocking Graph */}
            {graphData && (
              <section>
                <h3 className="text-xs font-semibold text-slate-400 uppercase tracking-wider mb-2">
                  블로킹 관계
                </h3>
                <BlockingGraphView
                  graph={graphData}
                  focusIssueId={issue.id}
                  issueTitles={issueTitles}
                />
              </section>
            )}

            {/* Tasks */}
            <TaskChecklist issueId={issue.id} />

            {/* Notes */}
            <NoteList issueId={issue.id} />

            {/* Footer actions */}
            <div className="flex flex-col gap-2 pt-2 border-t border-slate-100">
              {issue.status === 'demo' && (
                <>
                  <button
                    onClick={() => transition.mutate('working')}
                    disabled={transition.isPending}
                    className="w-full py-2 text-sm rounded-md border border-slate-300 text-slate-700 hover:bg-slate-50 disabled:opacity-50"
                  >
                    Working 으로 되돌리기
                  </button>
                  <button
                    onClick={() => transition.mutate('finished')}
                    disabled={transition.isPending}
                    className="w-full py-2 text-sm rounded-md bg-indigo-600 text-white hover:bg-indigo-700 disabled:opacity-50"
                  >
                    완료로 표시 (Finished)
                  </button>
                </>
              )}
              <button
                onClick={() => transition.mutate('cancelled')}
                disabled={transition.isPending}
                className="w-full py-2 text-sm rounded-md border border-red-200 text-red-600 hover:bg-red-50 disabled:opacity-50"
              >
                취소 (Cancelled)
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

function useIssueTitleMap(boards: { project_key: string; required: Issue[]; ready: Issue[]; working: Issue[]; demo: Issue[]; finished: Issue[] }[]): Map<number, string> {
  return useMemo(() => {
    const map = new Map<number, string>();
    for (const board of boards) {
      for (const col of [board.required, board.ready, board.working, board.demo, board.finished]) {
        for (const issue of col) {
          map.set(issue.id, issue.title);
        }
      }
    }
    return map;
  }, [boards]);
}
