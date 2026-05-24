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
  const [confirmDelete, setConfirmDelete] = useState(false);

  useEffect(() => {
    if (epic) {
      setTitle(epic.title);
      setDescription(epic.description ?? '');
      setStatus(epic.status);
      setConfirmDelete(false);
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
    if (!confirmDelete) { setConfirmDelete(true); return; }
    remove.mutate();
  };

  if (!epic) return null;

  const canSubmit = title.trim().length > 0;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
      onClick={(e) => { if (e.target === e.currentTarget) onClose(); }}
    >
      <div className="bg-white rounded-xl shadow-2xl w-full max-w-md p-6 flex flex-col gap-5">
        {/* Header */}
        <div className="flex items-center justify-between">
          <h2 className="text-sm font-bold text-slate-800">에픽 수정</h2>
          <button onClick={onClose} className="text-slate-400 hover:text-slate-600 text-lg leading-none">×</button>
        </div>

        <div className="flex flex-col gap-4">
          {/* Title */}
          <div className="flex flex-col gap-1">
            <label className="text-xs font-medium text-slate-600">
              에픽 이름 <span className="text-red-400">*</span>
            </label>
            <input
              autoFocus
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="에픽 이름"
              className="text-sm border border-slate-200 rounded-md px-3 py-2 focus:outline-none focus:ring-2 focus:ring-indigo-500/30 focus:border-indigo-400"
            />
          </div>

          {/* Status */}
          <div className="flex flex-col gap-1">
            <label className="text-xs font-medium text-slate-600">상태</label>
            <select
              value={status}
              onChange={(e) => setStatus(e.target.value as EpicStatus)}
              className="text-sm border border-slate-200 rounded-md px-3 py-2 bg-white focus:outline-none focus:ring-2 focus:ring-indigo-500/30 focus:border-indigo-400"
            >
              {STATUS_OPTIONS.map((opt) => (
                <option key={opt.value} value={opt.value}>{opt.label}</option>
              ))}
            </select>
          </div>

          {/* Description */}
          <div className="flex flex-col gap-1">
            <label className="text-xs font-medium text-slate-600">설명</label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="에픽 설명 (선택)"
              rows={3}
              className="text-sm border border-slate-200 rounded-md px-3 py-2 resize-y focus:outline-none focus:ring-2 focus:ring-indigo-500/30 focus:border-indigo-400"
            />
          </div>

          {/* Meta */}
          <p className="text-xs text-slate-400">프로젝트: {epic.project_key} · #{epic.id}</p>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between pt-1 border-t border-slate-100">
          {confirmDelete ? (
            <div className="flex items-center gap-2">
              <span className="text-xs text-red-600 font-medium">정말 삭제하시겠습니까?</span>
              <button
                type="button"
                onClick={handleDelete}
                disabled={remove.isPending}
                className="px-3 py-1.5 text-xs rounded-md bg-red-600 text-white hover:bg-red-700 disabled:opacity-50"
              >
                {remove.isPending ? '삭제 중…' : '확인'}
              </button>
              <button
                type="button"
                onClick={() => setConfirmDelete(false)}
                className="px-3 py-1.5 text-xs rounded-md border border-slate-200 text-slate-600 hover:bg-slate-50"
              >
                취소
              </button>
            </div>
          ) : (
            <button
              type="button"
              onClick={handleDelete}
              disabled={remove.isPending}
              className="px-3 py-2 text-xs rounded-md border border-red-200 text-red-600 hover:bg-red-50 disabled:opacity-50"
            >
              에픽 삭제
            </button>
          )}
          <div className="flex gap-2">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-sm rounded-md border border-slate-200 text-slate-600 hover:bg-slate-50"
            >
              취소
            </button>
            <button
              type="button"
              disabled={!canSubmit || update.isPending}
              onClick={() => update.mutate()}
              className="px-4 py-2 text-sm rounded-md bg-indigo-600 text-white hover:bg-indigo-700 disabled:opacity-50 font-medium"
            >
              {update.isPending ? '저장 중…' : '저장'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
