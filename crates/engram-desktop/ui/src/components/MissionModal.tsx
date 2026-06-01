import { useEffect, useState } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { missionCreate, missionUpdate, missionDelete } from '../ipc/invoke';
import type { Mission, MissionStatus } from '../ipc/types';
import { BaseModal } from './BaseModal';

interface Props {
  open: boolean;
  onClose: () => void;
  /** 편집 모드: 기존 미션 전달 시 수정, 미전달 시 생성 */
  mission?: Mission;
}

export function MissionModal({ open, onClose, mission }: Props) {
  const qc = useQueryClient();
  const isEdit = mission != null;

  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [jiraKey, setJiraKey] = useState('');
  const [status, setStatus] = useState<MissionStatus>('active');
  const [confirmDelete, setConfirmDelete] = useState(false);

  useEffect(() => {
    if (!open) {
      setConfirmDelete(false);
      return;
    }
    if (mission) {
      setTitle(mission.title);
      setDescription(mission.description ?? '');
      setJiraKey(mission.jira_key ?? '');
      setStatus(mission.status);
    } else {
      setTitle('');
      setDescription('');
      setJiraKey('');
      setStatus('active');
    }
  }, [open, mission]);

  const create = useMutation({
    mutationFn: () =>
      missionCreate({
        title: title.trim(),
        description: description.trim() || null,
        jira_key: jiraKey.trim() || null,
      }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['missionList'] });
      toast.success('미션이 생성되었습니다');
      onClose();
    },
    onError: (err) => toast.error(`생성 실패: ${err}`),
  });

  const update = useMutation({
    mutationFn: () =>
      missionUpdate(mission!.id, {
        title: title.trim() || null,
        description: description.trim() || null,
        jira_key: jiraKey.trim() || null,
        status,
      }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['missionList'] });
      qc.invalidateQueries({ queryKey: ['mission', mission!.id] });
      toast.success('미션이 수정되었습니다');
      onClose();
    },
    onError: (err) => toast.error(`수정 실패: ${err}`),
  });

  const remove = useMutation({
    mutationFn: () => missionDelete(mission!.id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['missionList'] });
      toast.success('미션이 삭제되었습니다');
      onClose();
    },
    onError: (err) => toast.error(`삭제 실패: ${err}`),
  });

  const handleDelete = () => {
    if (!mission) return;
    if (!confirmDelete) {
      setConfirmDelete(true);
      return;
    }
    remove.mutate();
  };

  const isPending = create.isPending || update.isPending || remove.isPending;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!title.trim()) return;
    isEdit ? update.mutate() : create.mutate();
  };

  const inputCls = 'w-full text-sm border border-slate-700 rounded-md px-3 py-2 bg-slate-800 text-white focus:outline-none focus:border-blue-500';

  return (
    <BaseModal open={open} onClose={onClose} title={isEdit ? '미션 수정' : '미션 생성'} maxWidth="max-w-md">
      <form onSubmit={handleSubmit} className="flex flex-col gap-4">
        {/* Title */}
        <div className="flex flex-col gap-1">
          <label className="text-xs font-semibold text-slate-400">
            제목 <span className="text-red-400">*</span>
          </label>
          <input
            autoFocus
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="미션 제목"
            required
            className={inputCls}
          />
        </div>

        {/* Jira Key */}
        <div className="flex flex-col gap-1">
          <label className="text-xs font-semibold text-slate-400">Jira Key</label>
          <input
            value={jiraKey}
            onChange={(e) => setJiraKey(e.target.value)}
            placeholder="예: M6, PROJ-123 (선택)"
            className={`${inputCls} font-mono`}
          />
        </div>

        {/* Status (edit only) */}
        {isEdit && (
          <div className="flex flex-col gap-1">
            <label className="text-xs font-semibold text-slate-400">상태</label>
            <select
              value={status}
              onChange={(e) => setStatus(e.target.value as MissionStatus)}
              className={inputCls}
            >
              <option value="active" className="bg-slate-900">Active</option>
              <option value="completed" className="bg-slate-900">Completed</option>
              <option value="cancelled" className="bg-slate-900">Cancelled</option>
            </select>
          </div>
        )}

        {/* Description */}
        <div className="flex flex-col gap-1">
          <label className="text-xs font-semibold text-slate-400">설명</label>
          <textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="미션 설명 (선택)"
            rows={3}
            className={`${inputCls} resize-y`}
          />
        </div>

        <div className="flex items-center justify-between pt-4 border-t border-slate-800 mt-2">
          {isEdit ? (
            confirmDelete ? (
              <div className="flex items-center gap-2">
                <span className="text-xs text-red-400 font-medium">정말 삭제하시겠습니까?</span>
                <button
                  type="button"
                  onClick={handleDelete}
                  disabled={remove.isPending}
                  className="px-3 py-1.5 text-xs rounded-lg bg-red-600 hover:bg-red-500 text-white disabled:opacity-50"
                >
                  {remove.isPending ? '삭제 중…' : '확인'}
                </button>
                <button
                  type="button"
                  onClick={() => setConfirmDelete(false)}
                  className="px-3 py-1.5 text-xs rounded-lg bg-slate-700 hover:bg-slate-600 text-white"
                >
                  취소
                </button>
              </div>
            ) : (
              <button
                type="button"
                onClick={handleDelete}
                disabled={remove.isPending}
                className="px-3 py-2 text-xs rounded-lg border border-red-500/50 hover:bg-red-950/20 text-red-400 disabled:opacity-50"
              >
                미션 삭제
              </button>
            )
          ) : <span />}
          <div className="flex gap-2">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white text-sm rounded-lg"
            >
              취소
            </button>
            <button
              type="submit"
              disabled={isPending || !title.trim()}
              className="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white text-sm rounded-lg disabled:opacity-50"
            >
              {isPending ? (isEdit ? '저장 중…' : '생성 중…') : (isEdit ? '저장' : '생성')}
            </button>
          </div>
        </div>
      </form>
    </BaseModal>
  );
}
