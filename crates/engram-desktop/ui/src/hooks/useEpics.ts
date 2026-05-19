import { useQuery } from '@tanstack/react-query';
import { epicList } from '../ipc/invoke';

export function useEpics(projectKey?: string) {
  return useQuery({
    queryKey: ['epics', projectKey],
    queryFn: () => epicList(projectKey),
  });
}
