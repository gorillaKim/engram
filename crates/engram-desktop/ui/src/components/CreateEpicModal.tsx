import { useState, useEffect } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { epicCreate, missionList, sprintList } from '../ipc/invoke';
import type { Mission, Sprint } from '../ipc/types';
import { BaseModal } from './BaseModal';

interface Props {
  open: boolean;
  onClose: () => void;
  defaultProjectKey?: string;
  defaultMissionId?: number;
}

export function CreateEpicModal({ open, onClose, defaultProjectKey, defaultMissionId }: Props) {
  const qc = useQueryClient();

  const [projectKey, setProjectKey] = useState('');
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [selectedMissionId, setSelectedMissionId] = useState<number | null>(null);
  const [selectedSprintId, setSelectedSprintId] = useState<number | null>(null);

  const { data: missions = [] } = useQuery<Mission[]>({
    queryKey: ['missionList'],
    queryFn: () => missionList(true),
    enabled: open,
  });

  const { data: sprints = [] } = useQuery<Sprint[]>({
    queryKey: ['sprintList'],
    queryFn: sprintList,
    enabled: open,
  });

  const activeMissions = missions.filter((m) => m.status === 'active');

  useEffect(() => {
    if (open) {
      setProjectKey(defaultProjectKey ?? '');
      setTitle('');
      setDescription('');
      setSelectedMissionId(defaultMissionId ?? null);
      setSelectedSprintId(null);
    }
  }, [open, defaultProjectKey, defaultMissionId]);

  const create = useMutation({
    mutationFn: () =>
      epicCreate({
        project_key: projectKey.trim(),
        title: title.trim(),
        description: description.trim() || undefined,
        mission_id: selectedMissionId,
        sprint_id: selectedSprintId,
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

  const canSubmit = title.trim().length > 0 && projectKey.trim().length > 0;
  const inputCls = 'w-full text-sm border border-slate-700 rounded-md px-3 py-2 bg-slate-800 text-white focus:outline-none focus:border-blue-500';

  return (
    <BaseModal open={open} onClose={onClose} title="새 에픽 생성" maxWidth="max-w-md">
      <div className="flex flex-col gap-4">
        <div className="flex flex-col gap-1">
          <label className="text-xs font-semibold text-slate-400">미션</label>
          <select
            value={selectedMissionId ?? ''}
            onChange={(e) => setSelectedMissionId(e.target.value ? Number(e.target.value) : null)}
            className={inputCls}
          >
            <option value="" className="bg-slate-900">미션 없음</option>
            {activeMissions.map((m) => (
              <option key={m.id} value={m.id} className="bg-slate-900">{m.title}</option>
            ))}
          </select>
        </div>

        <div className="flex flex-col gap-1">
          <label className="text-xs font-semibold text-slate-400">스프린트</label>
          <select
            value={selectedSprintId ?? ''}
            onChange={(e) => setSelectedSprintId(e.target.value ? Number(e.target.value) : null)}
            className={inputCls}
          >
            <option value="" className="bg-slate-900">백로그 (스프린트 미지정)</option>
            {sprints.map((s) => (
              <option key={s.id} value={s.id} className="bg-slate-900">{s.name}</option>
            ))}
          </select>
        </div>

        <div className="flex flex-col gap-1">
          <label className="text-xs font-semibold text-slate-400">
            프로젝트 키 <span className="text-red-400">*</span>
          </label>
          <input
            value={projectKey}
            onChange={(e) => setProjectKey(e.target.value)}
            placeholder="예: engram, xpert-da-web"
            className={inputCls}
          />
          <p className="text-[10px] text-slate-500">에픽이 sprint 를 보유합니다 (ADR-0014).</p>
        </div>

        <div className="flex flex-col gap-1">
          <label className="text-xs font-semibold text-slate-400">
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
          <label className="text-xs font-semibold text-slate-400">설명</label>
          <textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            rows={3}
            placeholder="에픽 설명 (선택)"
            className={`${inputCls} resize-y`}
          />
        </div>
      </div>

      <div className="flex gap-2 justify-end pt-4 border-t border-slate-800 mt-6">
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
