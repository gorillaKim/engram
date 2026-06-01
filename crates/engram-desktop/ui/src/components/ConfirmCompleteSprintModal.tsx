import { useState } from 'react';
import type { Sprint, Issue } from '../ipc/types';
import { useMutation, useQueryClient, useQuery } from '@tanstack/react-query';
import { epicSetSprint, sprintUpdate, issueList } from '../ipc/invoke';
import { getUnfinishedIssues, getUnfinishedEpics } from '../utils/sprintCompleteHelper';
import { toast } from 'sonner';
import { BaseModal } from './BaseModal';

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

        const epicIds = getUnfinishedEpics(issues);
        const promises = epicIds.map((epicId) =>
          epicSetSprint(epicId, targetSprintId)
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

  const inputCls = 'w-full text-xs border border-slate-700 rounded px-2 py-1.5 focus:outline-none focus:border-blue-500 bg-slate-800 text-white';

  return (
    <BaseModal open={isOpen} onClose={onClose} title="스프린트 완료" maxWidth="max-w-md">
      <div className="space-y-4">
        <p className="text-xs text-slate-400">
          스프린트를 완료하고 남은 미완료 일감을 관리합니다.
        </p>

        <div className="bg-slate-800 border border-slate-700 rounded-lg p-3 text-xs text-slate-300">
          <span className="font-semibold text-slate-200">대상 스프린트:</span> {sprint.name}
        </div>

        {isLoadingIssues ? (
          <div className="text-xs text-slate-400 text-center py-4">이슈 정보를 불러오는 중…</div>
        ) : (
          <>
            {unfinishedIssues.length > 0 ? (
              <div className="space-y-3">
                <div className="text-xs font-semibold text-rose-400 flex items-center gap-1.5">
                  ⚠️ 완료되지 않은 이슈 {unfinishedIssues.length}개가 존재합니다.
                </div>

                <div className="max-h-28 overflow-y-auto border border-slate-700 rounded-lg bg-slate-800/40 p-2 space-y-1.5">
                  {unfinishedIssues.map((issue) => (
                    <div key={issue.id} className="flex items-center justify-between text-[11px] text-slate-400">
                      <span className="truncate max-w-[280px]">· {issue.title}</span>
                      <span className="font-mono text-slate-500">#{issue.id}</span>
                    </div>
                  ))}
                </div>

                <div className="border border-slate-700 rounded-lg p-3 bg-slate-800/20 space-y-2">
                  <label className="text-[11px] font-semibold text-slate-400 uppercase tracking-wider block mb-1">
                    미완료 이슈 이관 대상 선택
                  </label>

                  <div className="space-y-2">
                    <label className="flex items-center gap-2 text-xs text-slate-300 cursor-pointer">
                      <input
                        type="radio"
                        name="transferTarget"
                        value="backlog"
                        checked={transferTarget === 'backlog'}
                        onChange={() => setTransferTarget('backlog')}
                        className="text-indigo-600 focus:ring-indigo-500/20 bg-slate-800 border-slate-700"
                      />
                      백로그로 이동
                    </label>

                    <label className="flex items-center gap-2 text-xs text-slate-300 cursor-pointer">
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
                        className="text-indigo-600 focus:ring-indigo-500/20 bg-slate-800 border-slate-700"
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
                            className={inputCls}
                          >
                            {otherActiveSprints.map((s) => (
                              <option key={s.id} value={s.id} className="bg-slate-900">
                                {s.name}
                              </option>
                            ))}
                          </select>
                        ) : (
                          <div className="text-[11px] text-slate-500">
                            이관 가능한 다른 활성/기획 단계의 스프린트가 없습니다.
                          </div>
                        )}
                      </div>
                    )}
                  </div>
                </div>
              </div>
            ) : (
              <div className="text-xs text-emerald-400 font-semibold flex items-center gap-1.5 py-2">
                ✅ 모든 이슈가 정상적으로 완료되었습니다.
              </div>
            )}
          </>
        )}
      </div>

      <div className="flex justify-end gap-2 mt-6 pt-4 border-t border-slate-800">
        <button
          type="button"
          onClick={onClose}
          disabled={completeMutation.isPending}
          className="px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white text-sm rounded-lg"
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
          className="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white text-sm rounded-lg disabled:opacity-50 font-medium"
        >
          {completeMutation.isPending ? '처리 중…' : '스프린트 완료'}
        </button>
      </div>
    </BaseModal>
  );
}
