import { useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { issueSetStatus } from '../ipc/invoke';
import type { IssueBoardStatus, IssueStatus } from '../ipc/types';

export function useIssueDnd(projectKey?: string) {
  const qc = useQueryClient();
  const queryKey = ['boardStatus', projectKey];

  return useMutation({
    mutationFn: ({ id, status }: { id: number; status: IssueStatus }) =>
      issueSetStatus(id, status),

    onMutate: async ({ id, status }) => {
      await qc.cancelQueries({ queryKey });
      const prev = qc.getQueryData<IssueBoardStatus>(queryKey);

      qc.setQueryData<IssueBoardStatus>(queryKey, (old) => {
        if (!old) return old;
        const cols = ['required', 'ready', 'working', 'demo', 'finished'] as const;
        return {
          ...old,
          boards: old.boards.map((board) => {
            const next = { ...board };
            let issue = null;
            for (const col of cols) {
              const idx = next[col].findIndex((i) => i.id === id);
              if (idx !== -1) {
                next[col] = [...next[col]];
                [issue] = next[col].splice(idx, 1);
              }
            }
            if (issue && status !== 'cancelled') {
              const col = status as 'required' | 'ready' | 'working' | 'demo' | 'finished';
              next[col] = [...next[col], { ...issue, status }];
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

    onSettled: () => qc.invalidateQueries({ queryKey }),
  });
}
