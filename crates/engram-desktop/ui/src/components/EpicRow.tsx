import { useState, useEffect } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { PriorityBadge } from './PriorityBadge';
import { StatusBadge } from './StatusBadge';
import type { Sprint, Epic, Issue } from '../ipc/types';
import { epicDelete, issueCreate } from '../ipc/invoke';

interface EpicRowProps {
  epic: Epic;
  issues: Issue[];
  sprints: Sprint[];
  onIssueClick: (id: number) => void;
  onAddIssue?: (epicId: number) => void;
  onEdit?: (epic: Epic) => void;
  expanded: boolean;
  onToggle: () => void;
  readOnly?: boolean;
  showCheckbox?: boolean;
  checked?: boolean;
  onCheck?: (checked: boolean) => void;
  renderIssueExtra?: (issue: Issue) => React.ReactNode;
}

export function EpicRow({
  epic,
  issues,
  sprints,
  onIssueClick,
  onEdit,
  expanded,
  onToggle,
  readOnly = false,
  showCheckbox = false,
  checked = false,
  onCheck,
  renderIssueExtra,
}: EpicRowProps) {
  const qc = useQueryClient();
  const [confirmDeleteEpic, setConfirmDeleteEpic] = useState(false);

  // 빠른 이슈 추가 상태
  const [quickTitle, setQuickTitle] = useState('');
  const [isAdding, setIsAdding] = useState(false);

  useEffect(() => {
    if (!confirmDeleteEpic) return;
    const t = setTimeout(() => setConfirmDeleteEpic(false), 3000);
    return () => clearTimeout(t);
  }, [confirmDeleteEpic]);

  const deleteEpic = useMutation({
    mutationFn: () => epicDelete(epic.id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['epicList'] });
      qc.invalidateQueries({ queryKey: ['issueList'] });
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      toast.success('에픽이 삭제되었습니다');
    },
    onError: (e) => toast.error(`에픽 삭제 실패: ${e}`),
  });

  const addIssueMutation = useMutation({
    mutationFn: (title: string) => issueCreate({
      epic_id: epic.id,
      title,
      priority: 'medium',
    }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['issueList'] });
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      toast.success('이슈가 추가되었습니다');
      setQuickTitle('');
      setIsAdding(false);
    },
    onError: (e) => toast.error(`이슈 추가 실패: ${e}`),
  });

  const handleQuickAddSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!quickTitle.trim()) return;
    addIssueMutation.mutate(quickTitle.trim());
  };

  return (
    <div className="border border-slate-200/80 rounded-xl bg-white shadow-sm overflow-hidden select-none">
      {/* Epic Row Header */}
      <div
        onClick={onToggle}
        className="flex items-center justify-between px-4 py-2.5 bg-slate-50 border-b border-slate-100 hover:bg-slate-100/40 transition-all cursor-pointer animate-in fade-in-20 duration-150"
      >
        <div className="flex items-center gap-2.5 min-w-0 flex-1">
          <span className="text-slate-400 text-[10px] w-3 flex-shrink-0 text-center font-bold">
            {expanded ? '▼' : '▶'}
          </span>

          {showCheckbox && !readOnly && onCheck && (
            <input
              type="checkbox"
              checked={checked}
              onChange={(e) => onCheck(e.target.checked)}
              onClick={(e) => e.stopPropagation()}
              className="w-3.5 h-3.5 rounded border-slate-300 text-indigo-600 focus:ring-indigo-500/20 cursor-pointer flex-shrink-0"
              title="일괄 변경용 선택"
            />
          )}

          <span className="text-[9px] font-bold text-violet-500 uppercase bg-violet-50 border border-violet-200/50 px-1.5 py-0.5 rounded tracking-wider flex-shrink-0">
            Epic
          </span>
          <span className="text-xs font-bold text-slate-800 truncate" title={epic.title}>
            {epic.title === '(에픽 정보 없음)' || epic.title === '(에픽 없음)'
              ? epic.title
              : `[${epic.project_key}] ${epic.title}`}
          </span>
          {epic.sprint_id ? (
            <span className="text-[9px] px-1.5 py-0.5 bg-indigo-50 text-indigo-600 border border-indigo-200/30 rounded font-medium flex-shrink-0">
              {sprints.find((s) => s.id === epic.sprint_id)?.name ?? `Sprint #${epic.sprint_id}`}
            </span>
          ) : (
            <span className="text-[9px] px-1.5 py-0.5 bg-slate-100 text-slate-500 rounded font-medium flex-shrink-0">
              백로그
            </span>
          )}
          <span className="text-xs text-slate-400 font-medium flex-shrink-0">
            {issues.length}개 이슈
          </span>
        </div>

        {!readOnly && (
          <div className="flex items-center gap-1 flex-shrink-0" onClick={(e) => e.stopPropagation()}>
            <button
              type="button"
              onClick={() => setIsAdding(true)}
              className="text-xs px-2 py-1 bg-white border border-slate-200 hover:bg-slate-100 text-slate-600 rounded font-semibold transition-colors"
            >
              + 이슈
            </button>

            {onEdit && (
              <button
                type="button"
                onClick={() => onEdit(epic)}
                title="에픽 수정"
                className="text-xs px-1.5 py-1 text-slate-400 hover:text-slate-700 transition-colors"
              >
                ✎
              </button>
            )}

            {confirmDeleteEpic ? (
              <button
                type="button"
                onClick={() => { setConfirmDeleteEpic(false); deleteEpic.mutate(); }}
                className="text-xs px-2 py-0.5 bg-red-600 hover:bg-red-500 text-white rounded font-medium transition-colors"
              >
                삭제 확인
              </button>
            ) : (
              <button
                type="button"
                onClick={() => setConfirmDeleteEpic(true)}
                title="에픽 삭제"
                className="text-xs px-1.5 py-1 text-slate-400 hover:text-red-600 transition-colors"
              >
                ✕
              </button>
            )}
          </div>
        )}
      </div>

      {/* Epic Row Body (Issues list & Quick add) */}
      {expanded && (
        <div className="divide-y divide-slate-100">
          {issues.map((issue) => (
            <div
              key={issue.id}
              onClick={() => onIssueClick(issue.id)}
              className="flex items-center gap-3 px-4 py-2.5 hover:bg-slate-50 cursor-pointer border-b border-slate-100 last:border-b-0 transition-colors"
            >
              <StatusBadge status={issue.status} type="issue" variant="ko" />
              <PriorityBadge priority={issue.priority} />
              <span className="text-xs text-slate-700 flex-1 truncate font-medium">{issue.title}</span>

              {/* Sprint Badge (read-only) */}
              {issue.sprint_id ? (
                <span className="text-[10px] px-2 py-0.5 bg-indigo-50 text-indigo-600 rounded font-medium max-w-[120px] truncate flex-shrink-0">
                  {sprints.find((s) => s.id === issue.sprint_id)?.name ?? `Sprint #${issue.sprint_id}`}
                </span>
              ) : (
                <span className="text-[10px] px-2 py-0.5 bg-slate-100 text-slate-500 rounded font-medium flex-shrink-0">
                  백로그
                </span>
              )}

              {/* Epic Chip (Link to EditEpicModal) */}
              {!readOnly && onEdit && (
                <button
                  type="button"
                  onClick={(e) => {
                    e.stopPropagation();
                    onEdit(epic);
                  }}
                  className="text-[10px] px-2 py-0.5 bg-violet-100 hover:bg-violet-200 text-violet-700 rounded font-medium truncate max-w-[120px] flex-shrink-0"
                  title={`에픽: ${epic.title} (클릭하여 수정)`}
                >
                  {epic.title}
                </button>
              )}

              <span className="text-xs text-slate-400 font-semibold font-mono flex-shrink-0">#{issue.id}</span>
              
              {/* Extra content (e.g. date for history) */}
              {renderIssueExtra && renderIssueExtra(issue)}
            </div>
          ))}

          {issues.length === 0 && !isAdding && (
            <div className="px-4 py-4 text-xs text-slate-400 text-center select-none font-medium">
              이 에픽에 등록된 이슈가 없습니다.
            </div>
          )}

          {/* Quick add issue form */}
          {!readOnly && isAdding && (
            <form onSubmit={handleQuickAddSubmit} className="p-3 bg-slate-50/50 flex items-center gap-2 border-t border-slate-100">
              <input
                type="text"
                placeholder="새 이슈 제목 입력…"
                value={quickTitle}
                onChange={(e) => setQuickTitle(e.target.value)}
                className="flex-1 text-xs border border-slate-200 rounded px-2.5 py-1.5 focus:outline-none focus:ring-1 focus:ring-indigo-500 bg-white font-medium text-slate-700"
                autoFocus
              />
              <button
                type="submit"
                disabled={addIssueMutation.isPending}
                className="text-xs px-2.5 py-1.5 bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white rounded font-semibold transition-colors"
              >
                {addIssueMutation.isPending ? '추가 중…' : '추가'}
              </button>
              <button
                type="button"
                onClick={() => { setIsAdding(false); setQuickTitle(''); }}
                className="text-xs px-2.5 py-1.5 bg-white border border-slate-200 hover:bg-slate-100 text-slate-600 rounded font-semibold transition-colors"
              >
                취소
              </button>
            </form>
          )}
        </div>
      )}
    </div>
  );
}
