import { useState } from 'react';
import type { Sprint, IssuePriority } from '../ipc/types';
import { useQueryClient } from '@tanstack/react-query';
import { issueSetSprint, issueSetPriority, issueDelete } from '../ipc/invoke';
import { toast } from 'sonner';

interface BulkActionBarProps {
  selectedIds: number[];
  onClearSelection: () => void;
  sprints: Sprint[];
}

export function BulkActionBar({
  selectedIds,
  onClearSelection,
  sprints,
}: BulkActionBarProps) {
  const qc = useQueryClient();
  const [isPending, setIsPending] = useState(false);

  // 스프린트 일괄 변경
  const handleBulkSprintChange = async (sprintIdStr: string) => {
    if (selectedIds.length === 0) return;
    setIsPending(true);
    try {
      const sprintId = sprintIdStr === '' ? null : Number(sprintIdStr);
      const promises = selectedIds.map((id) => issueSetSprint(id, sprintId));
      await Promise.all(promises);
      qc.invalidateQueries({ queryKey: ['issueList'] });
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      toast.success(`${selectedIds.length}개 이슈의 스프린트를 변경했습니다.`);
      onClearSelection();
    } catch (err) {
      toast.error(`일괄 변경 중 오류가 발생했습니다: ${err}`);
    } finally {
      setIsPending(false);
    }
  };

  // 우선순위 일괄 변경
  const handleBulkPriorityChange = async (priority: IssuePriority) => {
    if (selectedIds.length === 0) return;
    setIsPending(true);
    try {
      const promises = selectedIds.map((id) => issueSetPriority(id, priority));
      await Promise.all(promises);
      qc.invalidateQueries({ queryKey: ['issueList'] });
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      toast.success(`${selectedIds.length}개 이슈의 우선순위를 변경했습니다.`);
      onClearSelection();
    } catch (err) {
      toast.error(`일괄 변경 중 오류가 발생했습니다: ${err}`);
    } finally {
      setIsPending(false);
    }
  };

  // 일괄 삭제
  const handleBulkDelete = async () => {
    if (selectedIds.length === 0) return;
    const ok = window.confirm(`선택한 ${selectedIds.length}개의 이슈를 정말로 삭제하시겠습니까?\n이 작업은 되돌릴 수 없습니다.`);
    if (!ok) return;

    setIsPending(true);
    try {
      const promises = selectedIds.map((id) => issueDelete(id));
      await Promise.all(promises);
      qc.invalidateQueries({ queryKey: ['issueList'] });
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      toast.success(`${selectedIds.length}개 이슈를 삭제했습니다.`);
      onClearSelection();
    } catch (err) {
      toast.error(`일괄 삭제 중 오류가 발생했습니다: ${err}`);
    } finally {
      setIsPending(false);
    }
  };

  if (selectedIds.length === 0) return null;

  return (
    <div className="fixed bottom-6 left-1/2 -translate-x-1/2 bg-white/95 backdrop-blur-md border border-slate-200/80 shadow-2xl rounded-2xl px-6 py-3.5 flex items-center gap-6 z-40 animate-in slide-in-from-bottom-8 duration-200">
      <div className="flex items-center gap-2">
        <span className="bg-indigo-100 text-indigo-700 font-bold text-xs px-2.5 py-1 rounded-full">
          {selectedIds.length}
        </span>
        <span className="text-xs font-semibold text-slate-600">개 선택됨</span>
        <button
          type="button"
          onClick={onClearSelection}
          className="text-xs text-slate-400 hover:text-slate-600 font-medium underline"
        >
          선택 해제
        </button>
      </div>

      <div className="w-px h-5 bg-slate-200" />

      <div className="flex items-center gap-4">
        {/* 스프린트 일괄 변경 드롭다운 */}
        <div className="flex items-center gap-1.5">
          <span className="text-xs text-slate-400 font-medium">스프린트:</span>
          <select
            disabled={isPending}
            defaultValue="placeholder"
            onChange={(e) => {
              if (e.target.value !== 'placeholder') {
                handleBulkSprintChange(e.target.value);
              }
            }}
            className="text-xs border border-slate-200 bg-white hover:border-slate-300 rounded px-2.5 py-1 focus:outline-none focus:ring-2 focus:ring-indigo-500/20"
          >
            <option value="placeholder" disabled>이동 선택…</option>
            <option value="">백로그</option>
            {sprints.map((s) => (
              <option key={s.id} value={s.id}>
                {s.name}
              </option>
            ))}
          </select>
        </div>

        {/* 우선순위 일괄 변경 드롭다운 */}
        <div className="flex items-center gap-1.5">
          <span className="text-xs text-slate-400 font-medium">우선순위:</span>
          <select
            disabled={isPending}
            defaultValue="placeholder"
            onChange={(e) => {
              if (e.target.value !== 'placeholder') {
                handleBulkPriorityChange(e.target.value as IssuePriority);
              }
            }}
            className="text-xs border border-slate-200 bg-white hover:border-slate-300 rounded px-2.5 py-1 focus:outline-none focus:ring-2 focus:ring-indigo-500/20"
          >
            <option value="placeholder" disabled>변경 선택…</option>
            <option value="low">Low</option>
            <option value="medium">Medium</option>
            <option value="high">High</option>
            <option value="critical">Critical</option>
          </select>
        </div>

        {/* 일괄 삭제 버튼 */}
        <button
          type="button"
          onClick={handleBulkDelete}
          disabled={isPending}
          className="text-xs px-3 py-1 bg-rose-50 hover:bg-rose-100 text-rose-600 font-semibold rounded transition-colors disabled:opacity-50"
        >
          {isPending ? '삭제 중…' : '일괄 삭제'}
        </button>
      </div>
    </div>
  );
}
