import { useState, useEffect } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { epicUpdate, epicDelete } from '../ipc/invoke';
import type { Epic, EpicStatus } from '../ipc/types';

const STATUS_OPTIONS: { value: EpicStatus; label: string }[] = [
  { value: 'active',    label: '진행' },
  { value: 'completed', label: '완료' },
  { value: 'cancelled', label: '취소' },
];

interface Props {
  epic: Epic | null;
  onClose: () => void;
}

export function EditEpicModal({ epic, onClose }: Props) {
  const qc = useQueryClient();
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [status, setStatus] = useState<EpicStatus>('active');

  useEffect(() => {
    if (epic) {
      setTitle(epic.title);
      setDescription(epic.description ?? '');
      setStatus(epic.status);
    }
  }, [epic]);

  const update = useMutation({
    mutationFn: () => {
      if (!epic) throw new Error('no epic');
      return epicUpdate(epic.id, {
        title: title.trim() !== epic.title ? title.trim() : undefined,
        description: description.trim() !== (epic.description ?? '')
          ? (description.trim() || null)
          : undefined,
        status: status !== epic.status ? status : undefined,
      });
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['epicList'] });
      qc.invalidateQueries({ queryKey: ['epicListBacklog'] });
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      toast.success('에픽이 수정되었습니다');
      onClose();
    },
    onError: (err) => toast.error(`에픽 수정 실패: ${err}`),
  });

  const remove = useMutation({
    mutationFn: () => {
      if (!epic) throw new Error('no epic');
      return epicDelete(epic.id);
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['epicList'] });
      qc.invalidateQueries({ queryKey: ['epicListBacklog'] });
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      qc.invalidateQueries({ queryKey: ['blockingGraph'] });
      toast.success('에픽이 하위 이슈와 함께 삭제되었습니다');
      onClose();
    },
    onError: (err) => toast.error(`에픽 삭제 실패: ${err}`),
  });

  const handleDelete = () => {
    if (!epic) return;
    const ok = window.confirm(
      `정말 에픽 "${epic.title}" 을 삭제하시겠습니까?\n` +
      `하위 이슈/태스크/노트/링크가 모두 함께 삭제되며 되돌릴 수 없습니다.`,
    );
    if (ok) remove.mutate();
  };

  if (!epic) return null;

  const canSubmit = title.trim().length > 0;

  return (
    <div
      className="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
      onClick={(e) => { if (e.target === e.currentTarget) onClose(); }}
    >
      <div className="bg-slate-900 border border-slate-700 rounded-lg p-6 w-full max-w-md mx-4">
        <h3 className="text-lg font-semibold text-white mb-4">에픽 수정</h3>

        <div className="space-y-4">
          <div>
            <label className="block text-xs font-semibold text-slate-400 uppercase tracking-wider mb-1">에픽 이름 *</label>
            <input
              autoFocus
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              className="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white text-sm focus:outline-none focus:border-blue-500"
            />
          </div>

          <div>
            <label className="block text-xs font-semibold text-slate-400 uppercase tracking-wider mb-1">상태</label>
            <select
              value={status}
              onChange={(e) => setStatus(e.target.value as EpicStatus)}
              className="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white text-sm focus:outline-none focus:border-blue-500"
            >
              {STATUS_OPTIONS.map((opt) => (
                <option key={opt.value} value={opt.value}>{opt.label}</option>
              ))}
            </select>
          </div>

          <div>
            <label className="block text-xs font-semibold text-slate-400 uppercase tracking-wider mb-1">설명</label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              rows={3}
              className="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white text-sm focus:outline-none focus:border-blue-500"
            />
          </div>

          <p className="text-xs text-slate-500">프로젝트 키: {epic.project_key} · #{epic.id}</p>
        </div>

        <div className="flex items-center justify-between mt-6">
          <button
            type="button"
            onClick={handleDelete}
            disabled={remove.isPending}
            className="px-3 py-2 bg-red-600 hover:bg-red-500 text-white text-xs rounded-lg disabled:opacity-50"
            title="에픽과 하위 이슈/태스크/노트를 모두 삭제합니다 (비가역)"
          >
            {remove.isPending ? '삭제 중…' : '에픽 삭제'}
          </button>
          <div className="flex gap-2">
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
    </div>
  );
}
