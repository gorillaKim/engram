import { useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { issueSetStatus } from '../ipc/invoke';
import type { IssueBoardStatus, IssueStatus } from '../ipc/types';

export function useIssueDnd(projectKey?: string | null) {
  const qc = useQueryClient();
  const queryKey = ['boardStatus', projectKey ?? 'all'];

  return useMutation({
    mutationFn: ({ id, status }: { id: number; status: IssueStatus }) =>
      issueSetStatus(id, status),

    onMutate: async ({ id, status }) => {
      await qc.cancelQueries({ queryKey });
      const prev = qc.getQueryData<IssueBoardStatus>(queryKey);

      qc.setQueryData<IssueBoardStatus>(queryKey, (old) => {
        if (!old) return old;
        const cols = ['required', 'ready', 'working', 'demo', 'finished', 'cancelled'] as const;
        return {
          ...old,
          boards: old.boards.map((board) => {
            const next = { ...board };
            let issue = null;
            for (const col of cols) {
              const list = next[col] ?? [];
              const idx = list.findIndex((i) => i.id === id);
              if (idx !== -1) {
                next[col] = [...list];
                [issue] = next[col].splice(idx, 1);
              }
            }
            if (issue) {
              const col = status as typeof cols[number];
              next[col] = [...(next[col] ?? []), { ...issue, status }];
            }
            return next;
          }),
        };
      });

      return { prev };
    },

    onError: (err, _vars, ctx) => {
      if (ctx?.prev) qc.setQueryData(queryKey, ctx.prev);
      toast.error(`상태 변경 실패: ${err}`);
    },

    // 성공 시 background 재검증만 — optimistic update 가 이미 적용된 화면 위에
    // 곧이어 invalidate 가 refetch 를 트리거하면 잠시 빈 상태가 보일 수 있어
    // refetchType: 'none' 으로 즉시 refetch 를 막고 다음 refetchInterval(5s) 에 동기화.
    onSettled: () => qc.invalidateQueries({ queryKey, refetchType: 'none' }),
  });
}
