import { useState, useEffect } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { epicUpdate, epicDelete, missionList, sprintList, issueList, issueSetStatus } from '../ipc/invoke';
import type { Epic, EpicStatus, Issue } from '../ipc/types';
import { BaseModal } from './BaseModal';
import { ConfirmBulkActionModal } from './ConfirmBulkActionModal';
import { getUnfinishedIssuesForEpic } from '../utils/sprintCompleteHelper';

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
  const [missionIdInput, setMissionIdInput] = useState<number | null>(null);
  const [sprintIdInput, setSprintIdInput] = useState<number | null>(null);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [confirmBulkIssuesOpen, setConfirmBulkIssuesOpen] = useState(false);

  const { data: issues = [] } = useQuery<Issue[]>({
    queryKey: ['issueList', 'epic', epic?.id],
    queryFn: () => issueList({ epic_id: epic?.id } as any),
    enabled: epic != null,
  });

  const unfinishedIssues = getUnfinishedIssuesForEpic(epic?.id ?? 0, issues);

  const bulkCompleteIssues = useMutation({
    mutationFn: async (issueIds: number[]) => {
      const promises = issueIds.map(id => issueSetStatus(id, 'finished'));
      return Promise.all(promises);
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['issueList'] });
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      toast.success('에픽 하위의 모든 미완료 이슈가 완료되었습니다');
      setConfirmBulkIssuesOpen(false);
    },
    onError: (e) => toast.error(`이슈 일괄 완료 실패: ${e}`),
  });

  const { data: missions = [] } = useQuery({
    queryKey: ['missionList'],
    queryFn: () => missionList(true),
    enabled: epic != null,
  });

  const { data: sprints = [] } = useQuery({
    queryKey: ['sprintList'],
    queryFn: sprintList,
    enabled: epic != null,
  });

  useEffect(() => {
    if (epic) {
      setTitle(epic.title);
      setDescription(epic.description ?? '');
      setStatus(epic.status);
      setMissionIdInput(epic.mission_id);
      setSprintIdInput(epic.sprint_id);
      setConfirmDelete(false);
    }
  }, [epic]);

  const update = useMutation({
    mutationFn: () => {
      if (!epic) throw new Error('no epic');
      const isSprintChanged = sprintIdInput !== epic.sprint_id;
      return epicUpdate(epic.id, {
        title: title.trim() !== epic.title ? title.trim() : undefined,
        description: description.trim() !== (epic.description ?? '')
          ? (description.trim() || null)
          : undefined,
        status: status !== epic.status ? status : undefined,
        mission_id: missionIdInput !== epic.mission_id ? missionIdInput : undefined,
        sprint_id: isSprintChanged ? sprintIdInput : undefined,
        update_sprint_id: isSprintChanged ? true : undefined,
      });
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['epicList'] });
      qc.invalidateQueries({ queryKey: ['epicListBacklog'] });
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      qc.invalidateQueries({ queryKey: ['issueList'] });
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

  const canSubmit = title.trim().length > 0;
  const inputCls = 'w-full text-sm border border-slate-700 rounded-md px-3 py-2 bg-slate-800 text-white focus:outline-none focus:border-blue-500';

  return (
    <BaseModal open={epic != null} onClose={onClose} title="에픽 수정" maxWidth="max-w-md">
      {epic && (
        <div className="flex flex-col gap-4">
          {/* Title */}
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

          {/* Status */}
          <div className="flex flex-col gap-1">
            <label className="text-xs font-semibold text-slate-400">상태</label>
            <select
              value={status}
              onChange={(e) => setStatus(e.target.value as EpicStatus)}
              className={inputCls}
            >
              {STATUS_OPTIONS.map((opt) => (
                <option key={opt.value} value={opt.value} className="bg-slate-900">{opt.label}</option>
              ))}
            </select>
          </div>

          {/* Mission */}
          <div className="flex flex-col gap-1">
            <label className="text-xs font-semibold text-slate-400">미션</label>
            <select
              value={missionIdInput ?? ''}
              onChange={(e) => setMissionIdInput(e.target.value ? Number(e.target.value) : null)}
              className={inputCls}
            >
              <option value="" className="bg-slate-900">(미지정)</option>
              {missions.map((m) => (
                <option key={m.id} value={m.id} className="bg-slate-900">{m.title}</option>
              ))}
            </select>
          </div>

          {/* Sprint */}
          <div className="flex flex-col gap-1">
            <label className="text-xs font-semibold text-slate-400">스프린트</label>
            <select
              value={sprintIdInput ?? ''}
              onChange={(e) => setSprintIdInput(e.target.value ? Number(e.target.value) : null)}
              className={inputCls}
            >
              <option value="" className="bg-slate-900">백로그 (스프린트 미지정)</option>
              {sprints.map((s) => (
                <option key={s.id} value={s.id} className="bg-slate-900">{s.name}</option>
              ))}
            </select>
          </div>

          {/* Description */}
          <div className="flex flex-col gap-1">
            <label className="text-xs font-semibold text-slate-400">설명</label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="에픽 설명 (선택)"
              rows={3}
              className={`${inputCls} resize-y`}
            />
          </div>

          {/* Meta */}
          <p className="text-xs text-slate-500">프로젝트: {epic.project_key} · #{epic.id}</p>

          {/* Footer */}
          <div className="flex items-center justify-between pt-4 border-t border-slate-800 mt-2">
            {confirmDelete ? (
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
              <div className="flex items-center">
                <button
                  type="button"
                  onClick={handleDelete}
                  disabled={remove.isPending}
                  className="px-3 py-2 text-xs rounded-lg border border-red-500/50 hover:bg-red-950/20 text-red-400 disabled:opacity-50"
                >
                  에픽 삭제
                </button>
                {unfinishedIssues.length > 0 && (
                  <button
                    type="button"
                    onClick={() => setConfirmBulkIssuesOpen(true)}
                    className="px-3 py-2 text-xs rounded-lg border border-emerald-500/50 hover:bg-emerald-950/20 text-emerald-400 font-medium ml-2 transition-colors"
                  >
                    하위 이슈 일괄 완료
                  </button>
                )}
              </div>
            )}
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
      )}
      {confirmBulkIssuesOpen && epic && (
        <ConfirmBulkActionModal
          isOpen={confirmBulkIssuesOpen}
          onClose={() => setConfirmBulkIssuesOpen(false)}
          onConfirm={() => bulkCompleteIssues.mutate(unfinishedIssues.map(i => i.id))}
          title="에픽 하위 이슈 일괄 완료"
          description="에픽 하위의 다음 미완료 이슈들을 모두 완료(Finished) 처리하시겠습니까?"
          items={unfinishedIssues.map(i => ({ id: i.id, title: i.title }))}
          confirmText="일괄 완료"
          isPending={bulkCompleteIssues.isPending}
        />
      )}
    </BaseModal>
  );
}
