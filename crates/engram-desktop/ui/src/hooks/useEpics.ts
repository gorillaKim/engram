import { useQuery } from '@tanstack/react-query';
import { epicList } from '../ipc/invoke';

export function useEpics(projectKey?: string, includeCompleted?: boolean) {
  return useQuery({
    queryKey: ['epics', projectKey, includeCompleted],
    queryFn: () => epicList(projectKey, includeCompleted),
  });
}
