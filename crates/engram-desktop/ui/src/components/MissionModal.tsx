import { useEffect, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { missionCreate, missionUpdate, missionDelete, sprintList } from '../ipc/invoke';
import type { Mission, MissionStatus } from '../ipc/types';

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
  const [sprintId, setSprintId] = useState<number | null>(null);

  const { data: sprints = [] } = useQuery({
    queryKey: ['sprintList'],
    queryFn: sprintList,
    enabled: open,
  });

  useEffect(() => {
    if (!open) return;
    if (mission) {
      setTitle(mission.title);
      setDescription(mission.description ?? '');
      setJiraKey(mission.jira_key ?? '');
      setStatus(mission.status);
      setSprintId(mission.sprint_id ?? null);
    } else {
      setTitle('');
      setDescription('');
      setJiraKey('');
      setStatus('active');
      setSprintId(null);
    }
  }, [open, mission]);

  const create = useMutation({
    mutationFn: () =>
      missionCreate({
        title: title.trim(),
        description: description.trim() || null,
        jira_key: jiraKey.trim() || null,
        sprint_id: sprintId,
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
        sprint_id: sprintId,
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
    const ok = window.confirm(
      `정말 미션 "${mission.title}" 을 삭제하시겠습니까?\n` +
      `하위 에픽/이슈 연결이 모두 해제되며 되돌릴 수 없습니다.`,
    );
    if (ok) remove.mutate();
  };

  const isPending = create.isPending || update.isPending || remove.isPending;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!title.trim()) return;
    isEdit ? update.mutate() : create.mutate();
  };

  if (!open) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
      onClick={(e) => { if (e.target === e.currentTarget) onClose(); }}
    >
      <div className="bg-white rounded-xl shadow-2xl w-full max-w-md p-6 flex flex-col gap-5">
        <div className="flex items-center justify-between">
          <h2 className="text-sm font-bold text-slate-800">
            {isEdit ? '미션 수정' : '미션 생성'}
          </h2>
          <button onClick={onClose} className="text-slate-400 hover:text-slate-600 text-lg leading-none">×</button>
        </div>

        <form onSubmit={handleSubmit} className="flex flex-col gap-4">
          {/* Title */}
          <div className="flex flex-col gap-1">
            <label className="text-xs font-medium text-slate-600">
              제목 <span className="text-red-400">*</span>
            </label>
            <input
              autoFocus
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="미션 제목"
              required
              className="text-sm border border-slate-200 rounded-md px-3 py-2 focus:outline-none focus:ring-2 focus:ring-violet-500/30 focus:border-violet-400"
            />
          </div>

          {/* Jira Key */}
          <div className="flex flex-col gap-1">
            <label className="text-xs font-medium text-slate-600">Jira Key</label>
            <input
              value={jiraKey}
              onChange={(e) => setJiraKey(e.target.value)}
              placeholder="예: M6, PROJ-123 (선택)"
              className="text-sm border border-slate-200 rounded-md px-3 py-2 font-mono focus:outline-none focus:ring-2 focus:ring-violet-500/30 focus:border-violet-400"
            />
          </div>

          {/* Sprint */}
          <div className="flex flex-col gap-1">
            <label className="text-xs font-medium text-slate-600">스프린트</label>
            <select
              value={sprintId ?? ''}
              onChange={(e) => setSprintId(e.target.value ? Number(e.target.value) : null)}
              className="text-sm border border-slate-200 rounded-md px-3 py-2 bg-white focus:outline-none focus:ring-2 focus:ring-violet-500/30 focus:border-violet-400"
            >
              <option value="">백로그 (스프린트 미배정)</option>
              {sprints.map((s) => (
                <option key={s.id} value={s.id}>{s.name}</option>
              ))}
            </select>
          </div>

          {/* Status (edit only) */}
          {isEdit && (
            <div className="flex flex-col gap-1">
              <label className="text-xs font-medium text-slate-600">상태</label>
              <select
                value={status}
                onChange={(e) => setStatus(e.target.value as MissionStatus)}
                className="text-sm border border-slate-200 rounded-md px-3 py-2 bg-white focus:outline-none focus:ring-2 focus:ring-violet-500/30 focus:border-violet-400"
              >
                <option value="active">Active</option>
                <option value="completed">Completed</option>
                <option value="cancelled">Cancelled</option>
              </select>
            </div>
          )}

          {/* Description */}
          <div className="flex flex-col gap-1">
            <label className="text-xs font-medium text-slate-600">설명</label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="미션 설명 (선택)"
              rows={3}
              className="text-sm border border-slate-200 rounded-md px-3 py-2 resize-y focus:outline-none focus:ring-2 focus:ring-violet-500/30 focus:border-violet-400"
            />
          </div>

          <div className="flex items-center justify-between pt-1 border-t border-slate-100">
            {isEdit ? (
              <button
                type="button"
                onClick={handleDelete}
                disabled={remove.isPending}
                className="px-3 py-2 text-xs rounded-md border border-red-200 text-red-600 hover:bg-red-50 disabled:opacity-50"
              >
                {remove.isPending ? '삭제 중…' : '미션 삭제'}
              </button>
            ) : <span />}
            <div className="flex gap-2">
              <button
                type="button"
                onClick={onClose}
                className="px-4 py-2 text-sm rounded-md border border-slate-200 text-slate-600 hover:bg-slate-50"
              >
                취소
              </button>
              <button
                type="submit"
                disabled={isPending || !title.trim()}
                className="px-4 py-2 text-sm rounded-md bg-violet-600 text-white hover:bg-violet-700 disabled:opacity-50 font-medium"
              >
                {isPending ? (isEdit ? '저장 중…' : '생성 중…') : (isEdit ? '저장' : '생성')}
              </button>
            </div>
          </div>
        </form>
      </div>
    </div>
  );
}
