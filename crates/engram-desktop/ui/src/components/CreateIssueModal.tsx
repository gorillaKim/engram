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

const PRIORITIES: IssuePriority[] = ['critical', 'high', 'medium', 'low'];

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
  const [sprintId, setSprintId] = useState<number | null>(null); // null = 백로그
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [priority, setPriority] = useState<IssuePriority>('medium');

  // 미션 선택에 따라 에픽 필터링
  const filteredEpics = useMemo(() => {
    if (selectedMissionId === null) return allEpics;
    return allEpics.filter((e) => e.mission_id === selectedMissionId);
  }, [selectedMissionId, allEpics]);

  // 미션 변경 시 에픽 선택 초기화
  useEffect(() => {
    if (selectedMissionId !== null) {
      setEpicId('');
    }
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

  return (
    <div
      className="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
      onClick={(e) => { if (e.target === e.currentTarget) onClose(); }}
    >
      <div className="bg-slate-900 border border-slate-700 rounded-lg p-6 w-full max-w-md mx-4">
        <h3 className="text-lg font-semibold text-white mb-4">새 이슈 생성</h3>

        <div className="space-y-4">
          <div>
            <label className="block text-xs font-semibold text-slate-400 uppercase tracking-wider mb-1">미션</label>
            <select
              value={selectedMissionId ?? ''}
              onChange={(e) => setSelectedMissionId(e.target.value ? Number(e.target.value) : null)}
              className="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white text-sm focus:outline-none focus:border-blue-500"
            >
              <option value="">미션 없음 (전체 에픽 표시)</option>
              {activeMissions.map((m) => (
                <option key={m.id} value={m.id}>{m.title}</option>
              ))}
            </select>
          </div>

          <div>
            <label className="block text-xs font-semibold text-slate-400 uppercase tracking-wider mb-1">에픽</label>
            <select
              value={epicId}
              onChange={(e) => setEpicId(e.target.value === '' ? '' : Number(e.target.value))}
              className="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white text-sm focus:outline-none focus:border-blue-500"
            >
              <option value="">에픽 선택...</option>
              {filteredEpics.map((epic: Epic) => (
                <option key={epic.id} value={epic.id}>
                  [{epic.project_key}] {epic.title}
                </option>
              ))}
            </select>
          </div>

          <div>
            <label className="block text-xs font-semibold text-slate-400 uppercase tracking-wider mb-1">스프린트</label>
            <select
              value={sprintId ?? ''}
              onChange={(e) => setSprintId(e.target.value === '' ? null : Number(e.target.value))}
              className="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white text-sm focus:outline-none focus:border-blue-500"
            >
              <option value="">백로그 (스프린트 미지정)</option>
              {selectableSprints.map((s) => (
                <option key={s.id} value={s.id}>
                  {s.name} {s.status === 'active' ? '· 활성' : s.status === 'completed' ? '· 완료' : '· 계획'}
                </option>
              ))}
            </select>
          </div>

          <div>
            <label className="block text-xs font-semibold text-slate-400 uppercase tracking-wider mb-1">제목 *</label>
            <input
              autoFocus
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="이슈 제목"
              className="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white text-sm focus:outline-none focus:border-blue-500"
            />
          </div>

          <div>
            <label className="block text-xs font-semibold text-slate-400 uppercase tracking-wider mb-1">설명</label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              rows={3}
              placeholder="이슈 설명 (선택)"
              className="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white text-sm focus:outline-none focus:border-blue-500"
            />
          </div>

          <div>
            <label className="block text-xs font-semibold text-slate-400 uppercase tracking-wider mb-1">우선순위</label>
            <select
              value={priority}
              onChange={(e) => setPriority(e.target.value as IssuePriority)}
              className="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white text-sm focus:outline-none focus:border-blue-500"
            >
              {PRIORITIES.map((p) => (
                <option key={p} value={p}>{p}</option>
              ))}
            </select>
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
      </div>
    </div>
  );
}
