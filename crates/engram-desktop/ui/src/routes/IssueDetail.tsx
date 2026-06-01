import { useMemo, useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { issueGet, issueSetStatus, issueSetPriority, issueUpdate, blockingGraphForIssue, issueDelete, missionGet } from '../ipc/invoke';
import { TaskChecklist } from '../components/TaskChecklist';
import { NoteList } from '../components/NoteList';
import { PriorityBadge } from '../components/PriorityBadge';
import { StatusBadge } from '../components/StatusBadge';
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

  const { data: epics = [] } = useEpics(undefined, true);
  const epic = useMemo(() => issue ? epics.find((e) => e.id === issue.epic_id) : undefined, [epics, issue]);
  const targetProjectKey = selectedProjectKey || epic?.project_key;
  const [epicOpen, setEpicOpen] = useState(false);
  const [missionOpen, setMissionOpen] = useState(false);
  const [editingField, setEditingField] = useState<'title' | 'description' | 'goal' | null>(null);

  const { data: mission } = useQuery({
    queryKey: ['mission', issue?.mission_id],
    queryFn: () => missionGet(issue!.mission_id!),
    enabled: issue?.mission_id != null,
    staleTime: 30_000,
  });
  const [draftValue, setDraftValue] = useState('');
  const [confirmDelete, setConfirmDelete] = useState(false);

  const { data: graphData, isLoading: graphLoading } = useQuery({
    queryKey: ['blockingGraph', selectedIssueId],
    queryFn: () => blockingGraphForIssue(selectedIssueId!),
    enabled: selectedIssueId != null,
    staleTime: 10_000,
  });

  // Build issue title map from board data for graph node labels
  const { data: boardData } = useBoardStatus(targetProjectKey ?? null);
  const issueTitles = useIssueTitleMap(boardData?.boards ?? []);

  const transition = useMutation({
    mutationFn: (status: string) => issueSetStatus(selectedIssueId!, status),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['issue', selectedIssueId] });
    },
    onError: (err) => toast.error(`전이 실패: ${err}`),
  });

  const updateField = useMutation({
    mutationFn: (input: { title?: string; description?: string | null; goal?: string | null }) =>
      issueUpdate(selectedIssueId!, input),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['issue', selectedIssueId] });
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['issueList'] });
      setEditingField(null);
    },
    onError: (err) => toast.error(`수정 실패: ${err}`),
  });

  const updatePriority = useMutation({
    mutationFn: (priority: string) => issueSetPriority(selectedIssueId!, priority),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['issue', selectedIssueId] });
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
    },
    onError: (err) => toast.error(`우선순위 변경 실패: ${err}`),
  });

  const startEdit = (field: 'title' | 'description' | 'goal', current: string) => {
    setEditingField(field);
    setDraftValue(current);
  };

  const saveEdit = () => {
    if (!editingField || !issue) return;
    updateField.mutate({ [editingField]: draftValue });
  };

  const cancelEdit = () => setEditingField(null);

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
    if (!confirmDelete) { setConfirmDelete(true); return; }
    remove.mutate();
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
          <div className="flex items-start gap-2 min-w-0 flex-1">
            {issue && <PriorityBadge priority={issue.priority} />}
            <div className="flex-1 min-w-0">
              {editingField === 'title' ? (
                <input
                  autoFocus
                  value={draftValue}
                  onChange={(e) => setDraftValue(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') saveEdit();
                    if (e.key === 'Escape') cancelEdit();
                  }}
                  onBlur={saveEdit}
                  className="w-full text-base font-semibold text-slate-800 border border-indigo-300 rounded px-2 py-0.5 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 bg-white"
                />
              ) : (
                <div
                  onClick={() => issue && startEdit('title', issue.title)}
                  className="group flex items-center justify-between gap-1 w-full text-base font-semibold text-slate-800 border border-transparent rounded px-2 py-0.5 cursor-pointer hover:bg-slate-50 hover:border-slate-200 transition-all duration-200"
                >
                  <h2 className="truncate flex-1">
                    {issue ? `#${issue.id} ${issue.title}` : '…'}
                  </h2>
                  {issue && (
                    <button
                      className="opacity-0 group-hover:opacity-100 text-slate-400 hover:text-slate-600 text-xs shrink-0 transition-opacity"
                      title="제목 편집"
                    >
                      ✎
                    </button>
                  )}
                </div>
              )}
            </div>
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
            {/* Status + Priority */}
            <div className="flex items-center gap-4 flex-wrap">
              <div className="flex items-center gap-2">
                <span className="text-xs text-slate-500">상태</span>
                <StatusBadge status={issue.status} type="issue" />
              </div>
              <div className="flex items-center gap-2">
                <span className="text-xs text-slate-500">우선순위</span>
                <select
                  value={issue.priority}
                  disabled={updatePriority.isPending}
                  onChange={(e) => updatePriority.mutate(e.target.value)}
                  className="text-xs border border-slate-200 rounded px-2 py-0.5 bg-white focus:outline-none focus:ring-1 focus:ring-indigo-400 disabled:opacity-50"
                >
                  <option value="critical">긴급</option>
                  <option value="high">높음</option>
                  <option value="medium">보통</option>
                  <option value="low">낮음</option>
                </select>
              </div>
            </div>

            {/* Mission (collapsible, default closed) */}
            {mission && (
              <div className="rounded-lg border border-violet-100 bg-violet-50/60 overflow-hidden">
                <button
                  type="button"
                  onClick={() => setMissionOpen((v) => !v)}
                  className="w-full flex items-center gap-2 px-3 py-2 hover:bg-violet-100/50 transition-colors text-left"
                >
                  <span className="w-2 h-2 rounded-full bg-violet-400 flex-shrink-0" />
                  <span className="text-xs font-semibold text-violet-700 uppercase tracking-wide">미션</span>
                  <span className="flex-1 text-xs text-violet-800 font-medium truncate">{mission.title}</span>
                  {mission.jira_key && (
                    <span className="text-[10px] font-mono text-violet-400 flex-shrink-0">{mission.jira_key}</span>
                  )}
                  <StatusBadge status={mission.status} type="mission" />
                  <span className="text-violet-400 text-xs ml-1">{missionOpen ? '▲' : '▼'}</span>
                </button>
                {missionOpen && (
                  <div className="px-3 pb-3 border-t border-violet-100 pt-2">
                    {mission.description ? (
                      <div className="text-xs text-violet-700/80">
                        <Markdown>{mission.description}</Markdown>
                      </div>
                    ) : (
                      <p className="text-xs text-violet-400 italic">설명 없음</p>
                    )}
                  </div>
                )}
              </div>
            )}

            {/* Epic (collapsible, default closed) */}
            {epic && (
              <div className="rounded-lg border border-indigo-100 bg-indigo-50/60 overflow-hidden">
                <button
                  type="button"
                  onClick={() => setEpicOpen((v) => !v)}
                  className="w-full flex items-center gap-2 px-3 py-2 hover:bg-indigo-100/50 transition-colors text-left"
                >
                  <span className="w-2 h-2 rounded-full bg-indigo-400 flex-shrink-0" />
                  <span className="text-xs font-semibold text-indigo-700 uppercase tracking-wide">에픽</span>
                  <span className="flex-1 text-xs text-indigo-800 font-medium truncate">{epic.title}</span>
                  <StatusBadge status={epic.status} type="epic" />
                  <span className="text-indigo-400 text-xs ml-1">{epicOpen ? '▲' : '▼'}</span>
                </button>
                {epicOpen && (
                  <div className="px-3 pb-3 border-t border-indigo-100 pt-2">
                    {epic.description ? (
                      <div className="text-xs text-indigo-700/80">
                        <Markdown>{epic.description}</Markdown>
                      </div>
                    ) : (
                      <p className="text-xs text-indigo-400 italic">설명 없음</p>
                    )}
                  </div>
                )}
              </div>
            )}

            {/* Goal */}
            {editingField === 'goal' ? (
              <div className="relative bg-amber-50 rounded-md p-3 border border-amber-300 shadow-sm min-h-[96px] flex flex-col">
                <div className="flex items-center justify-between mb-1">
                  <p className="text-xs font-semibold text-amber-700 uppercase tracking-wide">목표 편집</p>
                </div>
                <div className="relative flex-1 flex flex-col">
                  <textarea
                    autoFocus
                    value={draftValue}
                    onChange={(e) => setDraftValue(e.target.value)}
                    className="w-full text-sm border-0 bg-transparent rounded p-0 pb-10 focus:outline-none focus:ring-0 resize-none text-amber-900"
                    style={{ minHeight: '64px' }}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
                        saveEdit();
                      }
                      if (e.key === 'Escape') cancelEdit();
                    }}
                  />
                  <div className="absolute bottom-0 right-0 flex gap-1.5 z-10 bg-amber-50/90 pt-1 pl-2">
                    <button
                      onClick={saveEdit}
                      disabled={updateField.isPending}
                      className="text-[11px] px-2 py-1 bg-amber-600 hover:bg-amber-500 text-white rounded font-medium disabled:opacity-50 transition-colors"
                    >
                      {updateField.isPending ? '저장 중…' : '저장 (⌘↵)'}
                    </button>
                    <button onClick={cancelEdit} className="text-[11px] px-2 py-1 bg-amber-100 hover:bg-amber-200 text-amber-800 rounded font-medium transition-colors">
                      취소
                    </button>
                  </div>
                </div>
              </div>
            ) : (
              <div
                onClick={() => startEdit('goal', issue.goal ?? '')}
                className="group relative bg-amber-50 rounded-md p-3 border border-amber-100 hover:border-amber-300/80 cursor-pointer transition-all duration-200 min-h-[96px] flex flex-col justify-between"
              >
                <div className="flex items-center justify-between mb-1">
                  <p className="text-xs font-semibold text-amber-700 uppercase tracking-wide">목표</p>
                  <span className="opacity-0 group-hover:opacity-100 text-xs text-amber-500 transition-opacity">✎ 편집</span>
                </div>
                <div className="flex-1 text-sm text-amber-900">
                  {issue.goal ? (
                    <Markdown>{issue.goal}</Markdown>
                  ) : (
                    <span className="text-amber-500/70 italic">+ 목표 추가</span>
                  )}
                </div>
              </div>
            )}

            {/* Description */}
            {editingField === 'description' ? (
              <div className="relative bg-slate-50 rounded-md p-3 border border-indigo-300 shadow-sm min-h-[140px] flex flex-col">
                <div className="flex items-center justify-between mb-2">
                  <p className="text-xs font-semibold text-slate-500 uppercase tracking-wide">설명 편집</p>
                </div>
                <div className="relative flex-1 flex flex-col">
                  <textarea
                    autoFocus
                    value={draftValue}
                    onChange={(e) => setDraftValue(e.target.value)}
                    className="w-full text-sm border-0 bg-transparent rounded p-0 pb-10 focus:outline-none focus:ring-0 resize-none text-slate-800"
                    style={{ minHeight: '108px' }}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
                        saveEdit();
                      }
                      if (e.key === 'Escape') cancelEdit();
                    }}
                  />
                  <div className="absolute bottom-0 right-0 flex gap-1.5 z-10 bg-slate-50/90 pt-1 pl-2">
                    <button
                      onClick={saveEdit}
                      disabled={updateField.isPending}
                      className="text-[11px] px-2 py-1 bg-indigo-600 hover:bg-indigo-500 text-white rounded font-medium disabled:opacity-50 transition-colors"
                    >
                      {updateField.isPending ? '저장 중…' : '저장 (⌘↵)'}
                    </button>
                    <button onClick={cancelEdit} className="text-[11px] px-2 py-1 bg-slate-200 hover:bg-slate-300 text-slate-700 rounded font-medium transition-colors">
                      취소
                    </button>
                  </div>
                </div>
              </div>
            ) : (
              <div
                onClick={() => startEdit('description', issue.description ?? '')}
                className="group relative bg-slate-50 rounded-md p-3 border border-slate-100 hover:border-slate-300/80 cursor-pointer transition-all duration-200 min-h-[140px] flex flex-col justify-between"
              >
                <div className="flex items-center justify-between mb-2">
                  <p className="text-xs font-semibold text-slate-500 uppercase tracking-wide">설명</p>
                  <span className="opacity-0 group-hover:opacity-100 text-xs text-slate-400 transition-opacity">✎ 편집</span>
                </div>
                <div className="flex-1 text-sm text-slate-800">
                  {issue.description ? (
                    <Markdown>{issue.description}</Markdown>
                  ) : (
                    <span className="text-slate-400 hover:text-indigo-500 italic">+ 설명 추가</span>
                  )}
                </div>
              </div>
            )}

            {/* Blocking Graph */}
            <section>
              <h3 className="text-xs font-semibold text-slate-400 uppercase tracking-wider mb-2">
                블로킹 관계
              </h3>
              {graphLoading ? (
                <p className="text-xs text-slate-400 py-2 animate-pulse">그래프 로딩 중…</p>
              ) : graphData ? (
                <BlockingGraphView
                  graph={graphData}
                  focusIssueId={issue.id}
                  issueTitles={issueTitles}
                  onSelectIssue={selectIssue}
                />
              ) : (
                <p className="text-xs text-slate-400 py-2">블로킹 관계 없음</p>
              )}
            </section>

            {/* Tasks */}
            <TaskChecklist issueId={issue.id} />

            {/* Issue links */}
            <IssueLinkSection issueId={issue.id} projectKey={targetProjectKey ?? undefined} />

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
              {confirmDelete ? (
                <div className="flex items-center gap-2">
                  <span className="text-xs text-red-600 font-medium flex-1">정말 영구 삭제하시겠습니까?</span>
                  <button
                    onClick={handleDelete}
                    disabled={remove.isPending}
                    className="px-3 py-1.5 text-xs rounded-md bg-red-600 text-white hover:bg-red-700 disabled:opacity-50"
                  >
                    {remove.isPending ? '삭제 중…' : '확인'}
                  </button>
                  <button
                    onClick={() => setConfirmDelete(false)}
                    className="px-3 py-1.5 text-xs rounded-md border border-slate-200 text-slate-600 hover:bg-slate-50"
                  >
                    취소
                  </button>
                </div>
              ) : (
                <button
                  onClick={handleDelete}
                  disabled={remove.isPending}
                  className="w-full py-2 text-sm rounded-md bg-red-600 text-white hover:bg-red-700 disabled:opacity-50"
                >
                  영구 삭제
                </button>
              )}
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
