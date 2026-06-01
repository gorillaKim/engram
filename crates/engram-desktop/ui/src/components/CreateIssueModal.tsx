import { useState, useEffect, useMemo } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { epicList, issueCreate, missionList } from '../ipc/invoke';
import type { Epic, IssuePriority, Mission } from '../ipc/types';
import { BaseModal } from './BaseModal';

interface Props {
  open: boolean;
  onClose: () => void;
  projectKey?: string;
  defaultEpicId?: number;
}

const PRIORITIES: { value: IssuePriority; label: string }[] = [
  { value: 'critical', label: '긴급' },
  { value: 'high',     label: '높음' },
  { value: 'medium',   label: '보통' },
  { value: 'low',      label: '낮음' },
];

export function CreateIssueModal({
  open, onClose, projectKey, defaultEpicId,
}: Props) {
  const qc = useQueryClient();

  const { data: allEpics = [] } = useQuery<Epic[]>({
    queryKey: ['epicList', projectKey],
    queryFn: () => epicList(projectKey),
    enabled: open,
  });

  const { data: missions = [] } = useQuery<Mission[]>({
    queryKey: ['missionList'],
    queryFn: () => missionList(false),
    enabled: open,
  });

  const activeMissions = useMemo(() => missions.filter((m) => m.status === 'active'), [missions]);

  const [selectedMissionId, setSelectedMissionId] = useState<number | null>(null);
  const [epicId, setEpicId] = useState<number | ''>('');
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [priority, setPriority] = useState<IssuePriority>('medium');

  const filteredEpics = useMemo(() => {
    if (selectedMissionId === null) return allEpics;
    return allEpics.filter((e) => e.mission_id === selectedMissionId);
  }, [selectedMissionId, allEpics]);

  useEffect(() => {
    if (selectedMissionId !== null) setEpicId('');
  }, [selectedMissionId]);

  useEffect(() => {
    if (open) {
      setSelectedMissionId(null);
      setEpicId(defaultEpicId ?? (allEpics[0]?.id ?? ''));
      setTitle('');
      setDescription('');
      setPriority('medium');
    }
  }, [open, defaultEpicId, allEpics]);

  const create = useMutation({
    mutationFn: () =>
      issueCreate({
        epic_id: epicId as number,
        title: title.trim(),
        description: description.trim() || undefined,
        priority,
      }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      qc.invalidateQueries({ queryKey: ['issueList'] });
      qc.invalidateQueries({ queryKey: ['issueListBacklog'] });
      toast.success('이슈가 생성되었습니다');
      onClose();
    },
    onError: (err) => toast.error(`이슈 생성 실패: ${err}`),
  });

  const canSubmit = title.trim().length > 0 && typeof epicId === 'number';
  const inputCls = 'w-full text-sm border border-slate-700 rounded-md px-3 py-2 bg-slate-800 text-white focus:outline-none focus:border-blue-500';

  return (
    <BaseModal open={open} onClose={onClose} title="새 이슈 생성" maxWidth="max-w-md">
      <div className="flex flex-col gap-4">
        <div className="flex flex-col gap-1">
          <label className="text-xs font-semibold text-slate-400">미션</label>
          <select
            value={selectedMissionId ?? ''}
            onChange={(e) => setSelectedMissionId(e.target.value ? Number(e.target.value) : null)}
            className={inputCls}
          >
            <option value="" className="bg-slate-900">미션 없음 (전체 에픽 표시)</option>
            {activeMissions.map((m) => (
              <option key={m.id} value={m.id} className="bg-slate-900">{m.title}</option>
            ))}
          </select>
        </div>

        <div className="flex flex-col gap-1">
          <label className="text-xs font-semibold text-slate-400">에픽</label>
          <select
            value={epicId}
            onChange={(e) => setEpicId(e.target.value === '' ? '' : Number(e.target.value))}
            className={inputCls}
          >
            <option value="" className="bg-slate-900">에픽 선택…</option>
            {filteredEpics.map((epic: Epic) => (
              <option key={epic.id} value={epic.id} className="bg-slate-900">
                [{epic.project_key}] {epic.title}
              </option>
            ))}
          </select>
        </div>

        <div className="text-xs text-slate-400 bg-slate-800/40 border border-slate-700/60 rounded-md p-2 flex items-center justify-between">
          <span>ℹ️ 스프린트는 미션 결합에 따라 결정됩니다.</span>
        </div>

        <div className="flex flex-col gap-1">
          <label className="text-xs font-semibold text-slate-400">
            제목 <span className="text-red-400">*</span>
          </label>
          <input
            autoFocus
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="이슈 제목"
            className={inputCls}
          />
        </div>

        <div className="flex flex-col gap-1">
          <label className="text-xs font-semibold text-slate-400">설명</label>
          <textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            rows={3}
            placeholder="이슈 설명 (선택)"
            className={`${inputCls} resize-y`}
          />
        </div>

        <div className="flex flex-col gap-1">
          <label className="text-xs font-semibold text-slate-400">우선순위</label>
          <select
            value={priority}
            onChange={(e) => setPriority(e.target.value as IssuePriority)}
            className={inputCls}
          >
            {PRIORITIES.map((p) => (
              <option key={p.value} value={p.value} className="bg-slate-900">{p.label}</option>
            ))}
          </select>
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
