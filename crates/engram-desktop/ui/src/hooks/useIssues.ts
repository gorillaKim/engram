import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import type { Issue, IssueFilter } from '../ipc/types';

export function useIssues(filter: IssueFilter) {
  return useQuery({
    queryKey: ['issues', filter],
    queryFn: async () => {
      const issues = await invoke<Issue[]>('issue_list', { filter });
      return issues;
    },
  });
}
