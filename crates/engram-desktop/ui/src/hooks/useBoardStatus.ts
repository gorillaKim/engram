import { useQuery } from '@tanstack/react-query';
import { boardStatus } from '../ipc/invoke';

export const useBoardStatus = (projectKey?: string | null) =>
  useQuery({
    queryKey: ['boardStatus', projectKey ?? 'all'],
    queryFn: () => boardStatus(projectKey ?? undefined),
    refetchInterval: 5000,
  });
