import { useMemo } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { issueGet, issueSetStatus, blockedIssuesGraph, issueDelete } from '../ipc/invoke';
import { TaskChecklist } from '../components/TaskChecklist';
import { NoteList } from '../components/NoteList';
import { PriorityBadge } from '../components/PriorityBadge';
import { BlockingGraphView } from '../components/BlockingGraph';
import { IssueLinkSection } from '../components/IssueLinkSection';
import { CommentSection } from '../components/CommentSection';
import { HistorySection } from '../components/HistorySection';
import { Markdown } from '../components/Markdown';
import { useUIStore } from '../store/ui';
import { useBoardStatus } from '../hooks/useBoardStatus';
import { useEpics } from '../hooks/useEpics';
import type { Issue } from '../ipc/types';

export function IssueDetail() {
  const { selectedIssueId, selectedProjectKey, selectIssue } = useUIStore();
  const qc = useQueryClient();

  const { data: issue } = useQuery({
    queryKey: ['issue', selectedIssueId],
    queryFn: () => issueGet(selectedIssueId!),
    enabled: selectedIssueId != null,
  });

  const { data: epics = [] } = useEpics(selectedProjectKey ?? undefined);
  const epic = useMemo(() => issue ? epics.find((e) => e.id === issue.epic_id) : undefined, [epics, issue]);

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

  const remove = useMutation({
    mutationFn: () => issueDelete(selectedIssueId!),
    onSuccess: () => {
      // 캐시 무효화 — 보드/세션/이슈 목록 모두 갱신
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      qc.invalidateQueries({ queryKey: ['blockingGraph'] });
      qc.removeQueries({ queryKey: ['issue', selectedIssueId] });
      toast.success('이슈가 삭제되었습니다');
      selectIssue(null);
    },
    onError: (err) => toast.error(`삭제 실패: ${err}`),
  });

  const handleDelete = () => {
    if (!issue) return;
    const ok = window.confirm(
      `정말 이슈 "#${issue.id} ${issue.title}" 를 삭제하시겠습니까?\n` +
      `하위 태스크/노트/링크가 모두 함께 삭제되며 되돌릴 수 없습니다.`,
    );
    if (ok) remove.mutate();
  };

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

            {/* Epic */}
            {epic && (
              <div className="rounded-lg border border-indigo-100 bg-indigo-50/60 p-3">
                <div className="flex items-center gap-2 mb-1">
                  <span className="w-2 h-2 rounded-full bg-indigo-400 flex-shrink-0" />
                  <span className="text-xs font-semibold text-indigo-700 uppercase tracking-wide">에픽</span>
                  <span className={`ml-auto text-[10px] font-medium px-1.5 py-0.5 rounded-full ${
                    epic.status === 'active' ? 'bg-emerald-100 text-emerald-700' :
                    epic.status === 'completed' ? 'bg-slate-100 text-slate-500' :
                    'bg-red-100 text-red-600'
                  }`}>{epic.status}</span>
                </div>
                <p className="text-sm font-semibold text-indigo-800 mb-1">{epic.title}</p>
                {epic.description && (
                  <div className="text-xs text-indigo-700/80">
                    <Markdown>{epic.description}</Markdown>
                  </div>
                )}
              </div>
            )}

            {/* Goal */}
            {issue.goal && (
              <div className="bg-amber-50 rounded-md p-3 border border-amber-100">
                <p className="text-xs font-semibold text-amber-700 uppercase tracking-wide mb-1">목표</p>
                <Markdown>{issue.goal}</Markdown>
              </div>
            )}

            {/* Description */}
            {issue.description && (
              <div className="bg-slate-50 rounded-md p-3 border border-slate-100">
                <p className="text-xs font-semibold text-slate-500 uppercase tracking-wide mb-2">설명</p>
                <Markdown>{issue.description}</Markdown>
              </div>
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

            {/* Issue links */}
            <IssueLinkSection issueId={issue.id} />

            {/* Notes (non-context) */}
            <NoteList issueId={issue.id} />

            {/* Comments (context notes) */}
            <CommentSection issueId={issue.id} />

            {/* History */}
            <HistorySection entityType="issue" entityId={issue.id} />

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
              <button
                onClick={handleDelete}
                disabled={remove.isPending}
                className="w-full py-2 text-sm rounded-md bg-red-600 text-white hover:bg-red-700 disabled:opacity-50"
              >
                {remove.isPending ? '삭제 중…' : '영구 삭제'}
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
