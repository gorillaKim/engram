import { useState, useEffect } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { epicCreate, missionList } from '../ipc/invoke';
import type { Mission } from '../ipc/types';

interface Props {
  open: boolean;
  onClose: () => void;
  defaultProjectKey?: string;
}

export function CreateEpicModal({ open, onClose, defaultProjectKey }: Props) {
  const qc = useQueryClient();

  const [projectKey, setProjectKey] = useState('');
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [selectedMissionId, setSelectedMissionId] = useState<number | null>(null);

  const { data: missions = [] } = useQuery<Mission[]>({
    queryKey: ['missionList'],
    queryFn: () => missionList(null, false),
    enabled: open,
  });

  const activeMissions = missions.filter((m) => m.status === 'active');

  useEffect(() => {
    if (open) {
      setProjectKey(defaultProjectKey ?? '');
      setTitle('');
      setDescription('');
      setSelectedMissionId(null);
    }
  }, [open, defaultProjectKey]);

  const create = useMutation({
    mutationFn: () =>
      epicCreate({
        project_key: projectKey.trim(),
        title: title.trim(),
        description: description.trim() || undefined,
        mission_id: selectedMissionId,
      }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['epicList'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      toast.success('에픽이 생성되었습니다');
      onClose();
    },
    onError: (err) => toast.error(`에픽 생성 실패: ${err}`),
  });

  if (!open) return null;

  const canSubmit = title.trim().length > 0 && projectKey.trim().length > 0;
  const inputCls = 'w-full text-sm border border-slate-200 rounded-md px-3 py-2 bg-white focus:outline-none focus:ring-2 focus:ring-indigo-500/30 focus:border-indigo-400';

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
      onClick={(e) => { if (e.target === e.currentTarget) onClose(); }}
    >
      <div className="bg-white rounded-xl shadow-2xl w-full max-w-md p-6 flex flex-col gap-5">
        <div className="flex items-center justify-between">
          <h2 className="text-sm font-bold text-slate-800">새 에픽 생성</h2>
          <button onClick={onClose} className="text-slate-400 hover:text-slate-600 text-lg leading-none">×</button>
        </div>

        <div className="flex flex-col gap-4">
          <div className="flex flex-col gap-1">
            <label className="text-xs font-medium text-slate-600">미션</label>
            <select
              value={selectedMissionId ?? ''}
              onChange={(e) => setSelectedMissionId(e.target.value ? Number(e.target.value) : null)}
              className={inputCls}
            >
              <option value="">미션 없음</option>
              {activeMissions.map((m) => (
                <option key={m.id} value={m.id}>{m.title}</option>
              ))}
            </select>
          </div>

          <div className="flex flex-col gap-1">
            <label className="text-xs font-medium text-slate-600">
              프로젝트 키 <span className="text-red-400">*</span>
            </label>
            <input
              value={projectKey}
              onChange={(e) => setProjectKey(e.target.value)}
              placeholder="예: engram, xpert-da-web"
              className={inputCls}
            />
            <p className="text-xs text-slate-400">에픽은 sprint-agnostic 카테고리입니다.</p>
          </div>

          <div className="flex flex-col gap-1">
            <label className="text-xs font-medium text-slate-600">
              에픽 이름 <span className="text-red-400">*</span>
            </label>
            <input
              autoFocus
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="에픽 이름"
              className={inputCls}
            />
          </div>

          <div className="flex flex-col gap-1">
            <label className="text-xs font-medium text-slate-600">설명</label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              rows={3}
              placeholder="에픽 설명 (선택)"
              className={`${inputCls} resize-y`}
            />
          </div>
        </div>

        <div className="flex gap-2 justify-end pt-1 border-t border-slate-100">
          <button
            type="button"
            onClick={onClose}
            className="px-4 py-2 text-sm rounded-md border border-slate-200 text-slate-600 hover:bg-slate-50"
          >
            취소
          </button>
          <button
            type="button"
            disabled={!canSubmit || create.isPending}
            onClick={() => create.mutate()}
            className="px-4 py-2 text-sm rounded-md bg-indigo-600 text-white hover:bg-indigo-700 disabled:opacity-50 font-medium"
          >
            {create.isPending ? '생성 중…' : '생성'}
          </button>
        </div>
      </div>
    </div>
  );
}
