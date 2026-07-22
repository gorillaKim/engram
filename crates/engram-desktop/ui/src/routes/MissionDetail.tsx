import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { missionGet, missionUpdate, missionDelete, epicList, epicUpdate, issueList } from '../ipc/invoke';
import { useUIStore } from '../store/ui';
import type { MissionStatus, Epic, Issue } from '../ipc/types';
import { NoteList } from '../components/NoteList';
import { CopyableId } from '../components/CopyableId';
import { PromptButton } from '../components/PromptButton';
import { BaseModal } from '../components/BaseModal';
import { Markdown } from '../components/Markdown';

export function MissionDetail() {
  const { selectedMissionId, selectMission, selectEpic } = useUIStore();
  const qc = useQueryClient();

  const { data: mission, isLoading } = useQuery({
    queryKey: ['mission', selectedMissionId],
    queryFn: () => missionGet(selectedMissionId!),
    enabled: selectedMissionId != null,
  });

  // 하위 에픽 리스트 조회
  const { data: epics = [] } = useQuery<Epic[]>({
    queryKey: ['epicList', 'all'],
    queryFn: () => epicList(undefined, true),
    enabled: selectedMissionId != null,
  });
  const missionEpics = epics.filter((e) => e.mission_id === selectedMissionId);

  // 미션 하위 모든 이슈 조회 (삭제 예정 목록용)
  const { data: missionIssues = [] } = useQuery<Issue[]>({
    queryKey: ['issueList', 'mission', selectedMissionId],
    queryFn: () => issueList({ mission_id: selectedMissionId } as any),
    enabled: selectedMissionId != null,
  });

  const [editingField, setEditingField] = useState<'title' | 'description' | null>(null);
  const [draftValue, setDraftValue] = useState('');
  const [deleteModalOpen, setDeleteModalOpen] = useState(false);

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
      qc.invalidateQueries({ queryKey: ['epicList'] });
      qc.invalidateQueries({ queryKey: ['issueList'] });
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      toast.success('미션과 하위 항목들이 삭제되었습니다.');
      setDeleteModalOpen(false);
      selectMission(null);
    },
    onError: (err) => toast.error(`삭제 실패: ${err}`),
  });

  const bulkCompleteEpicsMutation = useMutation({
    mutationFn: async (epicIds: number[]) => {
      const promises = epicIds.map((id) => epicUpdate(id, { status: 'completed' }));
      return Promise.all(promises);
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['epicList'] });
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      toast.success('모든 하위 에픽이 완료되었습니다.');
    },
    onError: (err) => toast.error(`에픽 일괄 완료 실패: ${err}`),
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
    deleteMissionMutation.mutate();
  };

  const handleClose = () => {
    selectMission(null);
  };

  const unfinishedEpics = missionEpics.filter((e) => e.status !== 'completed' && e.status !== 'cancelled');

  if (selectedMissionId == null) return null;

  return (
    <div className="relative w-full h-full bg-white flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between px-6 py-4 border-b border-slate-100 flex-shrink-0">
        <div className="flex items-center gap-2">
          <CopyableId type="mission" id={selectedMissionId} prefix="MISSION #" className="text-xs font-bold text-slate-500" />
        </div>
        <div className="flex items-center gap-2">
          {mission && (
            <PromptButton type="mission" id={mission.id} title={mission.title} size="xs" />
          )}
          <button
            onClick={handleClose}
            className="text-slate-400 hover:text-slate-600 p-1 text-sm font-medium transition-colors"
          >
            ✕ 닫기
          </button>
        </div>
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
                className="text-xs text-slate-600 hover:bg-slate-50 p-2.5 -m-2.5 rounded-lg cursor-pointer font-medium min-h-16 border border-slate-100/50 bg-slate-50/20 leading-relaxed"
              >
                {mission.description ? (
                  <Markdown>{mission.description}</Markdown>
                ) : (
                  '상세 설명이 없습니다. 클릭해서 설명 추가…'
                )}
              </div>
            )}
          </div>

          {/* Child Epics List Section */}
          <div className="border-t border-slate-100 pt-5 space-y-3">
            <div className="flex items-center justify-between">
              <label className="text-[10px] font-bold text-slate-400 uppercase tracking-wider pl-0.5">하위 에픽 ({missionEpics.length})</label>
              {unfinishedEpics.length > 0 && (
                <button
                  type="button"
                  onClick={() => bulkCompleteEpicsMutation.mutate(unfinishedEpics.map(e => e.id))}
                  disabled={bulkCompleteEpicsMutation.isPending}
                  className="text-[10px] font-bold text-violet-600 hover:text-violet-800 disabled:opacity-50 transition-colors"
                >
                  ✓ 에픽 일괄 완료
                </button>
              )}
            </div>
            {missionEpics.length === 0 ? (
              <div className="text-xs text-slate-400 py-2 pl-0.5">연결된 에픽이 없습니다.</div>
            ) : (
              <div className="space-y-1.5">
                {missionEpics.map((epic) => (
                  <div
                    key={epic.id}
                    onClick={() => selectEpic(epic.id)}
                    className="flex items-center justify-between p-2.5 rounded-lg border border-slate-100 hover:border-indigo-100 hover:bg-indigo-50/20 cursor-pointer transition-all"
                  >
                    <div className="flex flex-col gap-0.5 min-w-0 flex-1 mr-3">
                      <span className="text-xs font-semibold text-slate-700 truncate">{epic.title}</span>
                      <div className="flex items-center gap-2">
                        <CopyableId type="epic" id={epic.id} prefix="#" className="text-[10px] text-slate-400 font-mono" />
                        <span className="text-[10px] text-slate-400">{epic.project_key}</span>
                      </div>
                    </div>
                    <div className="flex items-center gap-2 flex-shrink-0">
                      <PromptButton type="epic" id={epic.id} title={epic.title} size="xs" />
                      <span className={`text-[10px] font-bold px-2 py-0.5 rounded-full whitespace-nowrap ${
                        epic.status === 'completed'
                          ? 'bg-emerald-50 text-emerald-600'
                          : epic.status === 'cancelled'
                          ? 'bg-slate-100 text-slate-500'
                          : 'bg-indigo-50 text-indigo-600'
                      }`}>
                        {epic.status === 'completed' ? '완료' : epic.status === 'cancelled' ? '취소' : '활성'}
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* Notes Section */}
          <div className="border-t border-slate-100 pt-5">
            <NoteList missionId={mission.id} />
          </div>

          {/* Footer actions */}
          <div className="flex flex-col gap-2 pt-6 border-t border-slate-100">
            <button
              onClick={() => setDeleteModalOpen(true)}
              className="w-full py-2 text-sm rounded-md bg-red-50 hover:bg-red-100 text-red-600 hover:text-red-700 font-semibold transition-colors"
            >
              미션 삭제
            </button>
          </div>
        </div>
      )}

      {/* Delete Warning Modal */}
      <BaseModal
        open={deleteModalOpen}
        onClose={() => setDeleteModalOpen(false)}
        title="미션 삭제"
      >
        <div className="space-y-4">
          <div className="bg-red-950/20 border border-red-800/40 rounded-lg p-4 space-y-1.5">
            <h4 className="text-sm font-bold text-red-400">⚠️ 경고: 일괄 삭제 안내</h4>
            <p className="text-xs text-slate-300 leading-relaxed">
              이 미션을 삭제하면 하위에 속한 에픽 및 모든 이슈, 태스크, 노드가 **영구히 일괄 삭제**됩니다.
            </p>
          </div>

          {missionEpics.length > 0 && (
            <div className="space-y-2">
              <label className="text-[10px] font-bold text-slate-400 uppercase tracking-wider block">
                삭제될 에픽 ({missionEpics.length}개)
              </label>
              <div className="max-h-24 overflow-y-auto border border-slate-700 rounded-lg bg-slate-800/40 p-2 space-y-1">
                {missionEpics.map((epic) => (
                  <div key={epic.id} className="text-[11px] text-slate-400 truncate">
                    · {epic.title} <span className="text-[9px] text-slate-500">({epic.project_key})</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {missionIssues.length > 0 && (
            <div className="space-y-2">
              <label className="text-[10px] font-bold text-slate-400 uppercase tracking-wider block">
                삭제될 하위 이슈 ({missionIssues.length}개)
              </label>
              <div className="max-h-28 overflow-y-auto border border-slate-700 rounded-lg bg-slate-800/40 p-2 space-y-1">
                {missionIssues.map((issue) => (
                  <div key={issue.id} className="text-[11px] text-slate-400 truncate flex justify-between">
                    <span>· {issue.title}</span>
                    <span className="text-[9px] text-slate-500 font-mono">#{issue.id}</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          <div className="flex justify-end gap-2 pt-4 border-t border-slate-800">
            <button
              onClick={() => setDeleteModalOpen(false)}
              className="px-4 py-2 border border-slate-700 hover:bg-slate-800 text-slate-300 text-xs font-semibold rounded-lg"
            >
              취소
            </button>
            <button
              onClick={handleDelete}
              disabled={deleteMissionMutation.isPending}
              className="px-4 py-2 bg-red-600 hover:bg-red-500 text-white text-xs font-semibold rounded-lg disabled:opacity-50 shadow-sm"
            >
              {deleteMissionMutation.isPending ? '삭제 중…' : '확인 및 삭제'}
            </button>
          </div>
        </div>
      </BaseModal>
    </div>
  );
}
