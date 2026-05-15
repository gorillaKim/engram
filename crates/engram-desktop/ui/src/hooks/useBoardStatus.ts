import { useQuery } from '@tanstack/react-query';
import { boardStatus } from '../ipc/invoke';

export const useBoardStatus = (projectKey?: string) =>
  useQuery({
    queryKey: ['boardStatus', projectKey],
    queryFn: () => boardStatus(projectKey),
    refetchInterval: 5000,
  });
