import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { missionGet, missionUpdate, missionDelete } from '../ipc/invoke';
import { useUIStore } from '../store/ui';
import type { MissionStatus } from '../ipc/types';
import { NoteList } from '../components/NoteList';

export function MissionDetail() {
  const { selectedMissionId, selectMission } = useUIStore();
  const qc = useQueryClient();

  const { data: mission, isLoading } = useQuery({
    queryKey: ['mission', selectedMissionId],
    queryFn: () => missionGet(selectedMissionId!),
    enabled: selectedMissionId != null,
  });

  const [editingField, setEditingField] = useState<'title' | 'description' | null>(null);
  const [draftValue, setDraftValue] = useState('');
  const [confirmDelete, setConfirmDelete] = useState(false);

  const updateMissionMutation = useMutation({
    mutationFn: (input: { title?: string; description?: string | null; status?: string }) =>
      missionUpdate(selectedMissionId!, {
        title: input.title,
        description: input.description,
        status: input.status as MissionStatus,
      }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['mission', selectedMissionId] });
      qc.invalidateQueries({ queryKey: ['missionList'] });
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      setEditingField(null);
      toast.success('미션 정보가 수정되었습니다.');
    },
    onError: (err) => toast.error(`수정 실패: ${err}`),
  });

  const deleteMissionMutation = useMutation({
    mutationFn: () => missionDelete(selectedMissionId!),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['missionList'] });
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      toast.success('미션이 삭제되었습니다.');
      selectMission(null);
    },
    onError: (err) => toast.error(`삭제 실패: ${err}`),
  });

  const startEdit = (field: 'title' | 'description', current: string) => {
    setEditingField(field);
    setDraftValue(current);
  };

  const saveEdit = () => {
    if (!editingField || !mission) return;
    updateMissionMutation.mutate({ [editingField]: draftValue });
  };

  const handleStatusChange = (status: MissionStatus) => {
    updateMissionMutation.mutate({ status });
  };

  const handleDelete = () => {
    if (!confirmDelete) {
      setConfirmDelete(true);
      return;
    }
    deleteMissionMutation.mutate();
  };

  const handleClose = () => {
    selectMission(null);
  };

  if (selectedMissionId == null) return null;

  return (
    <div className="relative w-full h-full bg-white flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-slate-100 flex-shrink-0">
          <div className="flex items-center gap-2">
            <span className="text-xs font-bold text-slate-400">MISSION #{selectedMissionId}</span>
          </div>
          <button
            onClick={handleClose}
            className="text-slate-400 hover:text-slate-600 p-1 text-sm font-medium transition-colors"
          >
            ✕ 닫기
          </button>
        </div>

        {isLoading || !mission ? (
          <div className="flex-1 flex items-center justify-center text-slate-400 text-sm">
            미션 정보를 불러오는 중…
          </div>
        ) : (
          <div className="flex-1 overflow-y-auto p-6 space-y-6">
            {/* Title */}
            <div className="space-y-1">
              <label className="text-[10px] font-bold text-slate-400 uppercase tracking-wider pl-0.5">미션명</label>
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
                  onClick={() => startEdit('title', mission.title)}
                  className="text-lg font-bold text-slate-800 hover:bg-slate-50 p-2 -m-2 rounded-lg cursor-pointer transition-colors"
                >
                  {mission.title}
                </div>
              )}
            </div>

            {/* Status Selector */}
            <div className="space-y-1.5">
              <label className="text-[10px] font-bold text-slate-400 uppercase tracking-wider pl-0.5">미션 상태</label>
              <div className="bg-slate-100/80 p-1 rounded-xl flex gap-1 w-full">
                {(['active', 'completed', 'cancelled'] as MissionStatus[]).map((st) => (
                  <button
                    key={st}
                    type="button"
                    onClick={() => handleStatusChange(st)}
                    className={`flex-1 text-xs py-1 px-3 rounded-lg font-semibold transition-all duration-200 ${
                      mission.status === st
                        ? 'bg-white text-indigo-600 shadow-sm ring-1 ring-slate-200/50'
                        : 'text-slate-500 hover:text-slate-700 hover:bg-white/40'
                    }`}
                  >
                    {st === 'active' ? '활성' : st === 'completed' ? '완료' : '취소'}
                  </button>
                ))}
              </div>
            </div>

            {/* Description */}
            <div className="space-y-1.5">
              <label className="text-[10px] font-bold text-slate-400 uppercase tracking-wider pl-0.5">미션 상세 설명</label>
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
                  onClick={() => startEdit('description', mission.description ?? '')}
                  className="text-xs text-slate-600 hover:bg-slate-50 p-2.5 -m-2.5 rounded-lg cursor-pointer font-medium whitespace-pre-wrap min-h-16 border border-slate-100/50 bg-slate-50/20 leading-relaxed"
                >
                  {mission.description || '상세 설명이 없습니다. 클릭해서 설명 추가…'}
                </div>
              )}
            </div>

            {/* Notes Section */}
            <div className="border-t border-slate-100 pt-5">
              <NoteList missionId={mission.id} />
            </div>

            {/* Footer actions */}
            <div className="flex flex-col gap-2 pt-6 border-t border-slate-100">
              {confirmDelete ? (
                <div className="flex items-center gap-2 bg-red-50 p-3 rounded-lg border border-red-100">
                  <span className="text-[11px] text-red-700 font-bold flex-1">정말 영구 삭제하시겠습니까? 연결된 에픽은 미지정으로 전환됩니다.</span>
                  <button
                    onClick={handleDelete}
                    disabled={deleteMissionMutation.isPending}
                    className="px-3 py-1.5 text-xs rounded-md bg-red-600 text-white hover:bg-red-700 disabled:opacity-50 font-semibold"
                  >
                    {deleteMissionMutation.isPending ? '삭제 중…' : '확인'}
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
                  disabled={deleteMissionMutation.isPending}
                  className="w-full py-2 text-sm rounded-md bg-red-50 hover:bg-red-100 text-red-600 hover:text-red-700 font-semibold transition-colors"
                >
                  미션 삭제
                </button>
              )}
            </div>
          </div>
        )}
      </div>
  );
}
