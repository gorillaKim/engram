import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { epicGet, epicUpdate, epicDelete, missionList, sprintList } from '../ipc/invoke';
import { useUIStore } from '../store/ui';
import type { Mission, Sprint, EpicStatus } from '../ipc/types';
import { NoteList } from '../components/NoteList';

export function EpicDetail() {
  const { selectedEpicId, selectEpic } = useUIStore();
  const qc = useQueryClient();

  const { data: epic, isLoading } = useQuery({
    queryKey: ['epic', selectedEpicId],
    queryFn: () => epicGet(selectedEpicId!),
    enabled: selectedEpicId != null,
  });

  const { data: missions = [] } = useQuery<Mission[]>({
    queryKey: ['missionList'],
    queryFn: () => missionList(true),
  });

  const { data: sprints = [] } = useQuery<Sprint[]>({
    queryKey: ['sprintList'],
    queryFn: sprintList,
  });

  const [editingField, setEditingField] = useState<'title' | 'description' | null>(null);
  const [draftValue, setDraftValue] = useState('');
  const [confirmDelete, setConfirmDelete] = useState(false);

  const updateEpicMutation = useMutation({
    mutationFn: (input: {
      title?: string;
      description?: string | null;
      status?: string;
      mission_id?: number | null;
      sprint_id?: number | null;
      update_sprint_id?: boolean;
    }) => epicUpdate(selectedEpicId!, {
      title: input.title,
      description: input.description,
      status: input.status as EpicStatus,
      mission_id: input.mission_id,
      sprint_id: input.sprint_id,
      update_sprint_id: input.update_sprint_id,
    }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['epic', selectedEpicId] });
      qc.invalidateQueries({ queryKey: ['epicList'] });
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      setEditingField(null);
      toast.success('에픽 정보가 수정되었습니다.');
    },
    onError: (err) => toast.error(`수정 실패: ${err}`),
  });

  const deleteEpicMutation = useMutation({
    mutationFn: () => epicDelete(selectedEpicId!),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['epicList'] });
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      toast.success('에픽이 삭제되었습니다.');
      selectEpic(null);
    },
    onError: (err) => toast.error(`삭제 실패: ${err}`),
  });

  const startEdit = (field: 'title' | 'description', current: string) => {
    setEditingField(field);
    setDraftValue(current);
  };

  const saveEdit = () => {
    if (!editingField || !epic) return;
    updateEpicMutation.mutate({ [editingField]: draftValue });
  };

  const handleStatusChange = (status: EpicStatus) => {
    updateEpicMutation.mutate({ status });
  };

  const handleMissionChange = (missionId: number | null) => {
    updateEpicMutation.mutate({ mission_id: missionId });
  };

  const handleSprintChange = (sprintId: number | null) => {
    updateEpicMutation.mutate({ sprint_id: sprintId, update_sprint_id: true });
  };

  const handleDelete = () => {
    if (!confirmDelete) {
      setConfirmDelete(true);
      return;
    }
    deleteEpicMutation.mutate();
  };

  const handleClose = () => {
    selectEpic(null);
  };

  if (selectedEpicId == null) return null;

  return (
    <div className="relative w-full h-full bg-white flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-slate-100 flex-shrink-0">
          <div className="flex items-center gap-2">
            <span className="text-xs font-bold text-slate-400">EPIC #{selectedEpicId}</span>
            {epic && (
              <span className="text-xs font-semibold text-slate-500 font-mono bg-slate-100 px-1.5 py-0.5 rounded">
                {epic.project_key}
              </span>
            )}
          </div>
          <button
            onClick={handleClose}
            className="text-slate-400 hover:text-slate-600 p-1 text-sm font-medium transition-colors"
          >
            ✕ 닫기
          </button>
        </div>

        {isLoading || !epic ? (
          <div className="flex-1 flex items-center justify-center text-slate-400 text-sm">
            에픽 정보를 불러오는 중…
          </div>
        ) : (
          <div className="flex-1 overflow-y-auto p-6 space-y-6">
            {/* Title */}
            <div className="space-y-1">
              <label className="text-[10px] font-bold text-slate-400 uppercase tracking-wider pl-0.5">에픽 제목</label>
              {editingField === 'title' ? (
                <div className="flex gap-2">
                  <input
                    type="text"
                    value={draftValue}
                    onChange={(e) => setDraftValue(e.target.value)}
                    className="flex-1 text-sm border border-slate-200 rounded-lg px-3 py-1.5 focus:outline-none focus:ring-2 focus:ring-indigo-500/20"
                    autoFocus
                  />
                  <button onClick={saveEdit} className="px-3 py-1.5 bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold rounded-lg shadow-sm">저장</button>
                  <button onClick={() => setEditingField(null)} className="px-3 py-1.5 border border-slate-200 text-slate-600 text-xs rounded-lg hover:bg-slate-50">취소</button>
                </div>
              ) : (
                <div
                  onClick={() => startEdit('title', epic.title)}
                  className="text-lg font-bold text-slate-800 hover:bg-slate-50 p-2 -m-2 rounded-lg cursor-pointer transition-colors"
                >
                  {epic.title}
                </div>
              )}
            </div>

            {/* Status Selector */}
            <div className="space-y-1.5">
              <label className="text-[10px] font-bold text-slate-400 uppercase tracking-wider pl-0.5">상태</label>
              <div className="bg-slate-100/80 p-1 rounded-xl flex gap-1 w-full">
                {(['active', 'completed', 'cancelled'] as EpicStatus[]).map((st) => (
                  <button
                    key={st}
                    type="button"
                    onClick={() => handleStatusChange(st)}
                    className={`flex-1 text-xs py-1 px-3 rounded-lg font-semibold transition-all duration-200 ${
                      epic.status === st
                        ? 'bg-white text-indigo-600 shadow-sm ring-1 ring-slate-200/50'
                        : 'text-slate-500 hover:text-slate-700 hover:bg-white/40'
                    }`}
                  >
                    {st === 'active' ? '활성' : st === 'completed' ? '완료' : '취소'}
                  </button>
                ))}
              </div>
            </div>

            {/* Mission Selector */}
            <div className="space-y-1.5">
              <label className="text-[10px] font-bold text-slate-400 uppercase tracking-wider pl-0.5">소속 미션</label>
              <select
                value={epic.mission_id ?? ''}
                onChange={(e) => handleMissionChange(e.target.value ? Number(e.target.value) : null)}
                className="w-full text-sm border border-slate-200 rounded-lg h-10 py-0 pl-3 pr-8 bg-white shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/20 text-slate-700 font-medium"
              >
                <option value="">미션 지정 안 함</option>
                {missions.map((m) => (
                  <option key={m.id} value={m.id}>{m.title}</option>
                ))}
              </select>
            </div>

            {/* Sprint Selector */}
            <div className="space-y-1.5">
              <label className="text-[10px] font-bold text-slate-400 uppercase tracking-wider pl-0.5">소속 스프린트</label>
              <select
                value={epic.sprint_id ?? ''}
                onChange={(e) => handleSprintChange(e.target.value ? Number(e.target.value) : null)}
                className="w-full text-sm border border-slate-200 rounded-lg h-10 py-0 pl-3 pr-8 bg-white shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/20 text-slate-700 font-medium"
              >
                <option value="">백로그 (스프린트 미지정)</option>
                {sprints.map((s) => (
                  <option key={s.id} value={s.id}>{s.name} ({s.status === 'active' ? '진행 중' : s.status === 'planning' ? '계획 중' : '완료됨'})</option>
                ))}
              </select>
            </div>

            {/* Description */}
            <div className="space-y-1.5">
              <label className="text-[10px] font-bold text-slate-400 uppercase tracking-wider pl-0.5">에픽 상세 설명</label>
              {editingField === 'description' ? (
                <div className="flex flex-col gap-2">
                  <textarea
                    value={draftValue}
                    onChange={(e) => setDraftValue(e.target.value)}
                    rows={4}
                    className="w-full text-xs border border-slate-200 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 font-medium"
                    autoFocus
                  />
                  <div className="flex gap-2 justify-end">
                    <button onClick={saveEdit} className="px-3 py-1.5 bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold rounded-lg shadow-sm">저장</button>
                    <button onClick={() => setEditingField(null)} className="px-3 py-1.5 border border-slate-200 text-slate-600 text-xs rounded-lg hover:bg-slate-50">취소</button>
                  </div>
                </div>
              ) : (
                <div
                  onClick={() => startEdit('description', epic.description ?? '')}
                  className="text-xs text-slate-600 hover:bg-slate-50 p-2.5 -m-2.5 rounded-lg cursor-pointer font-medium whitespace-pre-wrap min-h-16 border border-slate-100/50 bg-slate-50/20 leading-relaxed"
                >
                  {epic.description || '상세 설명이 없습니다. 클릭해서 설명 추가…'}
                </div>
              )}
            </div>

            {/* Notes Section */}
            <div className="border-t border-slate-100 pt-5">
              <NoteList epicId={epic.id} />
            </div>

            {/* Footer actions */}
            <div className="flex flex-col gap-2 pt-6 border-t border-slate-100">
              {confirmDelete ? (
                <div className="flex items-center gap-2 bg-red-50 p-3 rounded-lg border border-red-100">
                  <span className="text-[11px] text-red-700 font-bold flex-1">정말 영구 삭제하시겠습니까? 산하 이슈는 유지됩니다.</span>
                  <button
                    onClick={handleDelete}
                    disabled={deleteEpicMutation.isPending}
                    className="px-3 py-1.5 text-xs rounded-md bg-red-600 text-white hover:bg-red-700 disabled:opacity-50 font-semibold"
                  >
                    {deleteEpicMutation.isPending ? '삭제 중…' : '확인'}
                  </button>
                  <button
                    onClick={() => setConfirmDelete(false)}
                    className="px-3 py-1.5 text-xs rounded-md border border-slate-200 bg-white text-slate-600 hover:bg-slate-50"
                  >
                    취소
                  </button>
                </div>
              ) : (
                <button
                  onClick={handleDelete}
                  disabled={deleteEpicMutation.isPending}
                  className="w-full py-2 text-sm rounded-md bg-red-50 hover:bg-red-100 text-red-600 hover:text-red-700 font-semibold transition-colors"
                >
                  에픽 삭제
                </button>
              )}
            </div>
          </div>
        )}
      </div>
  );
}
