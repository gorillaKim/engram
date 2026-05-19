import { useState, useEffect } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { sprintUpdate } from '../ipc/invoke';
import type { Sprint } from '../ipc/types';

interface Props {
  sprint: Sprint | null;
  onClose: () => void;
}

export function EditSprintModal({ sprint, onClose }: Props) {
  const qc = useQueryClient();
  const [name, setName] = useState('');
  const [goal, setGoal] = useState('');

  useEffect(() => {
    if (sprint) {
      setName(sprint.name);
      setGoal(sprint.goal ?? '');
    }
  }, [sprint]);

  const update = useMutation({
    mutationFn: () => {
      if (!sprint) throw new Error('no sprint');
      return sprintUpdate(
        sprint.id,
        undefined,
        name.trim() !== sprint.name ? name.trim() : undefined,
        goal.trim() !== (sprint.goal ?? '') ? (goal.trim() || undefined) : undefined,
      );
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['sprintList'] });
      qc.invalidateQueries({ queryKey: ['sprintCurrent'] });
      toast.success('스프린트가 수정되었습니다');
      onClose();
    },
    onError: (err) => toast.error(`스프린트 수정 실패: ${err}`),
  });

  if (!sprint) return null;

  const canSubmit = name.trim().length > 0;

  return (
    <div
      className="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
      onClick={(e) => { if (e.target === e.currentTarget) onClose(); }}
    >
      <div className="bg-slate-900 border border-slate-700 rounded-lg p-6 w-full max-w-md mx-4">
        <h3 className="text-lg font-semibold text-white mb-4">스프린트 수정</h3>

        <div className="space-y-4">
          <div>
            <label className="block text-xs font-semibold text-slate-400 uppercase tracking-wider mb-1">
              스프린트 이름 *
            </label>
            <input
              autoFocus
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white text-sm focus:outline-none focus:border-blue-500"
            />
          </div>

          <div>
            <label className="block text-xs font-semibold text-slate-400 uppercase tracking-wider mb-1">
              목표 (선택)
            </label>
            <textarea
              value={goal}
              onChange={(e) => setGoal(e.target.value)}
              rows={3}
              placeholder="이번 스프린트의 목표를 입력하세요"
              className="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white text-sm focus:outline-none focus:border-blue-500 resize-none"
            />
          </div>

          <p className="text-xs text-slate-500">상태: {sprint.status} · #{sprint.id}</p>
        </div>

        <div className="flex justify-end gap-2 mt-6">
          <button
            type="button"
            onClick={onClose}
            className="px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white text-sm rounded-lg"
          >
            취소
          </button>
          <button
            type="button"
            disabled={!canSubmit || update.isPending}
            onClick={() => update.mutate()}
            className="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white text-sm rounded-lg disabled:opacity-50"
          >
            {update.isPending ? '저장 중…' : '저장'}
          </button>
        </div>
      </div>
    </div>
  );
}
