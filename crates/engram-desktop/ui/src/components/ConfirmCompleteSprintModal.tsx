import { useState } from 'react';
import type { Sprint, Issue } from '../ipc/types';
import { useMutation, useQueryClient, useQuery } from '@tanstack/react-query';
import { missionSetSprint, sprintUpdate, issueList } from '../ipc/invoke';
import { getUnfinishedIssues, getUnfinishedMissions } from '../utils/sprintCompleteHelper';
import { toast } from 'sonner';

interface ConfirmCompleteSprintModalProps {
  isOpen: boolean;
  onClose: () => void;
  sprint: Sprint;
  sprints: Sprint[];
}

export function ConfirmCompleteSprintModal({
  isOpen,
  onClose,
  sprint,
  sprints,
}: ConfirmCompleteSprintModalProps) {
  const qc = useQueryClient();
  const [transferTarget, setTransferTarget] = useState<'backlog' | 'sprint'>('backlog');
  const [selectedTargetSprintId, setSelectedTargetSprintId] = useState<number | ''>('');

  // 현재 스프린트에 귀속된 이슈 목록 조회
  const { data: issues = [], isLoading: isLoadingIssues } = useQuery<Issue[]>({
    queryKey: ['issueList', sprint.id],
    queryFn: () => issueList({ sprint_id: sprint.id } as any),
    enabled: isOpen,
  });

  const unfinishedIssues = getUnfinishedIssues(issues);

  // 다른 활성/기획 단계 스프린트 목록
  const otherActiveSprints = sprints.filter(
    (s) => s.id !== sprint.id && s.status !== 'completed' && s.status !== 'cancelled'
  );

  // 일괄 이관 및 스프린트 완료 뮤테이션
  const completeMutation = useMutation({
    mutationFn: async () => {
      // 1. 미완료 이슈 이관 (미션 단위)
      if (unfinishedIssues.length > 0) {
        const targetSprintId =
          transferTarget === 'backlog'
            ? null
            : selectedTargetSprintId === ''
            ? null
            : selectedTargetSprintId;

        const missionIds = getUnfinishedMissions(issues);
        const promises = missionIds.map((missionId) =>
          missionSetSprint(missionId, targetSprintId)
        );
        await Promise.all(promises);
      }

      // 2. 스프린트 상태를 completed로 갱신
      await sprintUpdate(sprint.id, 'completed');
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['sprintList'] });
      qc.invalidateQueries({ queryKey: ['sprintCurrent'] });
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['issueList'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      toast.success(`"${sprint.name}" 스프린트가 완료되었습니다.`);
      onClose();
    },
    onError: (err) => {
      toast.error(`스프린트 완료 중 오류가 발생했습니다: ${err}`);
    },
  });

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-slate-900/40 backdrop-blur-sm flex items-center justify-center z-50 p-4">
      <div className="bg-white rounded-xl shadow-xl w-full max-w-md flex flex-col max-h-[90vh] border border-slate-100 overflow-hidden animate-in fade-in zoom-in-95 duration-150">
        {/* Header */}
        <div className="px-6 py-4 border-b border-slate-100 flex-shrink-0">
          <h3 className="text-base font-semibold text-slate-800">스프린트 완료</h3>
          <p className="text-xs text-slate-400 mt-1">
            스프린트를 완료하고 남은 미완료 일감을 관리합니다.
          </p>
        </div>

        {/* Content */}
        <div className="p-6 overflow-y-auto flex-1 space-y-4">
          <div className="bg-slate-50 border border-slate-100 rounded-lg p-3 text-xs text-slate-600">
            <span className="font-semibold text-slate-700">대상 스프린트:</span> {sprint.name}
          </div>

          {isLoadingIssues ? (
            <div className="text-xs text-slate-400 text-center py-4">이슈 정보를 불러오는 중…</div>
          ) : (
            <>
              {unfinishedIssues.length > 0 ? (
                <div className="space-y-3">
                  <div className="text-xs font-semibold text-rose-600 flex items-center gap-1.5">
                    ⚠️ 완료되지 않은 이슈 {unfinishedIssues.length}개가 존재합니다.
                  </div>

                  <div className="max-h-28 overflow-y-auto border border-slate-100 rounded-lg bg-slate-50/50 p-2 space-y-1.5">
                    {unfinishedIssues.map((issue) => (
                      <div key={issue.id} className="flex items-center justify-between text-[11px] text-slate-500">
                        <span className="truncate max-w-[280px]">· {issue.title}</span>
                        <span className="font-mono text-slate-400">#{issue.id}</span>
                      </div>
                    ))}
                  </div>

                  <div className="border border-slate-100 rounded-lg p-3 bg-white space-y-2">
                    <label className="text-[11px] font-semibold text-slate-500 uppercase tracking-wider block mb-1">
                      미완료 이슈 이관 대상 선택
                    </label>

                    <div className="space-y-2">
                      <label className="flex items-center gap-2 text-xs text-slate-700 cursor-pointer">
                        <input
                          type="radio"
                          name="transferTarget"
                          value="backlog"
                          checked={transferTarget === 'backlog'}
                          onChange={() => setTransferTarget('backlog')}
                          className="text-indigo-600 focus:ring-indigo-500/20"
                        />
                        백로그로 이동
                      </label>

                      <label className="flex items-center gap-2 text-xs text-slate-700 cursor-pointer">
                        <input
                          type="radio"
                          name="transferTarget"
                          value="sprint"
                          checked={transferTarget === 'sprint'}
                          onChange={() => {
                            setTransferTarget('sprint');
                            if (otherActiveSprints.length > 0 && selectedTargetSprintId === '') {
                              setSelectedTargetSprintId(otherActiveSprints[0].id);
                            }
                          }}
                          className="text-indigo-600 focus:ring-indigo-500/20"
                        />
                        다른 스프린트로 이동
                      </label>

                      {transferTarget === 'sprint' && (
                        <div className="pl-6 pt-1">
                          {otherActiveSprints.length > 0 ? (
                            <select
                              value={selectedTargetSprintId}
                              onChange={(e) =>
                                setSelectedTargetSprintId(
                                  e.target.value === '' ? '' : Number(e.target.value)
                                )
                              }
                              className="w-full text-xs border border-slate-200 rounded px-2 py-1.5 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 bg-white"
                            >
                              {otherActiveSprints.map((s) => (
                                <option key={s.id} value={s.id}>
                                  {s.name}
                                </option>
                              ))}
                            </select>
                          ) : (
                            <div className="text-[11px] text-slate-400">
                              이관 가능한 다른 활성/기획 단계의 스프린트가 없습니다.
                            </div>
                          )}
                        </div>
                      )}
                    </div>
                  </div>
                </div>
              ) : (
                <div className="text-xs text-emerald-600 font-semibold flex items-center gap-1.5 py-2">
                  ✅ 모든 이슈가 정상적으로 완료되었습니다.
                </div>
              )}
            </>
          )}
        </div>

        {/* Footer */}
        <div className="px-6 py-3 bg-slate-50 border-t border-slate-100 flex items-center justify-end gap-2 flex-shrink-0">
          <button
            type="button"
            onClick={onClose}
            disabled={completeMutation.isPending}
            className="text-xs px-3 py-2 bg-white border border-slate-200 text-slate-600 hover:bg-slate-100 rounded-md transition-colors"
          >
            취소
          </button>
          <button
            type="button"
            onClick={() => completeMutation.mutate()}
            disabled={
              completeMutation.isPending ||
              isLoadingIssues ||
              (transferTarget === 'sprint' && selectedTargetSprintId === '')
            }
            className="text-xs px-3 py-2 bg-indigo-600 hover:bg-indigo-500 text-white font-semibold rounded-md transition-all disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {completeMutation.isPending ? '처리 중…' : '스프린트 완료'}
          </button>
        </div>
      </div>
    </div>
  );
}
