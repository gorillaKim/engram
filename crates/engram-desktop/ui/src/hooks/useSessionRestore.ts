import { useQuery } from '@tanstack/react-query';
import { sessionRestore } from '../ipc/invoke';

export const useSessionRestore = (projectKey?: string) =>
  useQuery({
    queryKey: ['sessionRestore', projectKey],
    queryFn: () => sessionRestore(projectKey),
    staleTime: 10_000,
    refetchInterval: 30_000,
  });
