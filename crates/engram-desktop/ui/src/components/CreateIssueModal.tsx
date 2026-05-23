import { useState, useEffect, useMemo } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { epicList, issueCreate, missionList, sprintList } from '../ipc/invoke';
import type { Epic, IssuePriority, Mission, Sprint } from '../ipc/types';

interface Props {
  open: boolean;
  onClose: () => void;
  projectKey?: string;
  defaultEpicId?: number;
  /** 기본 선택 스프린트. null 이면 백로그로 기본 선택 */
  defaultSprintId?: number | null;
}

const PRIORITIES: { value: IssuePriority; label: string }[] = [
  { value: 'critical', label: '긴급' },
  { value: 'high',     label: '높음' },
  { value: 'medium',   label: '보통' },
  { value: 'low',      label: '낮음' },
];

export function CreateIssueModal({
  open, onClose, projectKey, defaultEpicId, defaultSprintId,
}: Props) {
  const qc = useQueryClient();

  const { data: allEpics = [] } = useQuery<Epic[]>({
    queryKey: ['epicList', projectKey],
    queryFn: () => epicList(projectKey),
    enabled: open,
  });

  const { data: sprints = [] } = useQuery<Sprint[]>({
    queryKey: ['sprintList'],
    queryFn: sprintList,
    enabled: open,
  });

  const { data: missions = [] } = useQuery<Mission[]>({
    queryKey: ['missionList'],
    queryFn: () => missionList(null, false),
    enabled: open,
  });

  const activeMissions = useMemo(() => missions.filter((m) => m.status === 'active'), [missions]);

  const [selectedMissionId, setSelectedMissionId] = useState<number | null>(null);
  const [epicId, setEpicId] = useState<number | ''>('');
  const [sprintId, setSprintId] = useState<number | null>(null);
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
      setSprintId(defaultSprintId ?? null);
      setTitle('');
      setDescription('');
      setPriority('medium');
    }
  }, [open, defaultEpicId, defaultSprintId, allEpics]);

  const create = useMutation({
    mutationFn: () =>
      issueCreate({
        epic_id: epicId as number,
        sprint_id: sprintId,
        mission_id: selectedMissionId,
        title: title.trim(),
        description: description.trim() || undefined,
        priority,
      }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      qc.invalidateQueries({ queryKey: ['issueList'] });
      qc.invalidateQueries({ queryKey: ['issueListBacklog'] });
      toast.success(sprintId == null ? '이슈가 백로그에 추가되었습니다' : '이슈가 생성되었습니다');
      onClose();
    },
    onError: (err) => toast.error(`이슈 생성 실패: ${err}`),
  });

  if (!open) return null;

  const canSubmit = title.trim().length > 0 && typeof epicId === 'number';
  const selectableSprints = sprints.filter((s) => s.status !== 'cancelled');
  const inputCls = 'w-full text-sm border border-slate-200 rounded-md px-3 py-2 bg-white focus:outline-none focus:ring-2 focus:ring-indigo-500/30 focus:border-indigo-400';

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
      onClick={(e) => { if (e.target === e.currentTarget) onClose(); }}
    >
      <div className="bg-white rounded-xl shadow-2xl w-full max-w-md p-6 flex flex-col gap-5 max-h-[90vh] overflow-y-auto">
        <div className="flex items-center justify-between">
          <h2 className="text-sm font-bold text-slate-800">새 이슈 생성</h2>
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
              <option value="">미션 없음 (전체 에픽 표시)</option>
              {activeMissions.map((m) => (
                <option key={m.id} value={m.id}>{m.title}</option>
              ))}
            </select>
          </div>

          <div className="flex flex-col gap-1">
            <label className="text-xs font-medium text-slate-600">에픽</label>
            <select
              value={epicId}
              onChange={(e) => setEpicId(e.target.value === '' ? '' : Number(e.target.value))}
              className={inputCls}
            >
              <option value="">에픽 선택…</option>
              {filteredEpics.map((epic: Epic) => (
                <option key={epic.id} value={epic.id}>
                  [{epic.project_key}] {epic.title}
                </option>
              ))}
            </select>
          </div>

          <div className="flex flex-col gap-1">
            <label className="text-xs font-medium text-slate-600">스프린트</label>
            <select
              value={sprintId ?? ''}
              onChange={(e) => setSprintId(e.target.value === '' ? null : Number(e.target.value))}
              className={inputCls}
            >
              <option value="">백로그 (스프린트 미지정)</option>
              {selectableSprints.map((s) => (
                <option key={s.id} value={s.id}>
                  {s.name}{s.status === 'active' ? ' · 활성' : s.status === 'completed' ? ' · 완료' : ' · 계획'}
                </option>
              ))}
            </select>
          </div>

          <div className="flex flex-col gap-1">
            <label className="text-xs font-medium text-slate-600">
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
            <label className="text-xs font-medium text-slate-600">설명</label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              rows={3}
              placeholder="이슈 설명 (선택)"
              className={`${inputCls} resize-y`}
            />
          </div>

          <div className="flex flex-col gap-1">
            <label className="text-xs font-medium text-slate-600">우선순위</label>
            <select
              value={priority}
              onChange={(e) => setPriority(e.target.value as IssuePriority)}
              className={inputCls}
            >
              {PRIORITIES.map((p) => (
                <option key={p.value} value={p.value}>{p.label}</option>
              ))}
            </select>
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
