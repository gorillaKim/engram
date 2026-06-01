import { useState, useEffect } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { sprintCreate } from '../ipc/invoke';
import { BaseModal } from './BaseModal';

interface Props {
  open: boolean;
  onClose: () => void;
}

export function CreateSprintModal({ open, onClose }: Props) {
  const qc = useQueryClient();

  const [name, setName] = useState('');
  const [goal, setGoal] = useState('');
  const [startDate, setStartDate] = useState('');
  const [endDate, setEndDate] = useState('');

  useEffect(() => {
    if (open) {
      setName('');
      setGoal('');
      setStartDate('');
      setEndDate('');
    }
  }, [open]);

  const create = useMutation({
    mutationFn: () =>
      sprintCreate({
        name: name.trim(),
        goal: goal.trim() || undefined,
        start_date: startDate || undefined,
        end_date: endDate || undefined,
      }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['sprintCurrent'] });
      qc.invalidateQueries({ queryKey: ['sprintList'] });
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      toast.success('스프린트가 생성되었습니다');
      onClose();
    },
    onError: (err) => toast.error(`스프린트 생성 실패: ${err}`),
  });

  const canSubmit = name.trim().length > 0;

  return (
    <BaseModal open={open} onClose={onClose} title="새 스프린트 생성" maxWidth="max-w-md">
      <div className="space-y-4">
        <div>
          <label className="block text-xs font-semibold text-slate-400 uppercase tracking-wider mb-1">스프린트 이름 *</label>
          <input
            autoFocus
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="예: Sprint 6"
            className="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white text-sm focus:outline-none focus:border-blue-500"
          />
        </div>

        <div>
          <label className="block text-xs font-semibold text-slate-400 uppercase tracking-wider mb-1">목표</label>
          <textarea
            value={goal}
            onChange={(e) => setGoal(e.target.value)}
            rows={2}
            placeholder="이번 스프린트의 목표 (선택)"
            className="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white text-sm focus:outline-none focus:border-blue-500"
          />
        </div>

        <div className="flex gap-3">
          <div className="flex-1">
            <label className="block text-xs font-semibold text-slate-400 uppercase tracking-wider mb-1">시작일</label>
            <input
              type="date"
              value={startDate}
              onChange={(e) => setStartDate(e.target.value)}
              className="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white text-sm focus:outline-none focus:border-blue-500"
            />
          </div>
          <div className="flex-1">
            <label className="block text-xs font-semibold text-slate-400 uppercase tracking-wider mb-1">종료일</label>
            <input
              type="date"
              value={endDate}
              onChange={(e) => setEndDate(e.target.value)}
              className="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white text-sm focus:outline-none focus:border-blue-500"
            />
          </div>
        </div>
      </div>

      <div className="flex gap-2 justify-end mt-6">
        <button
          type="button"
          onClick={onClose}
          className="px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white text-sm rounded-lg"
        >
          취소
        </button>
        <button
          type="button"
          disabled={!canSubmit || create.isPending}
          onClick={() => create.mutate()}
          className="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white text-sm rounded-lg disabled:opacity-50"
        >
          {create.isPending ? '생성 중…' : '생성'}
        </button>
      </div>
    </BaseModal>
  );
}
