import { useQuery } from '@tanstack/react-query';
import { mcpGetToolDefinitions } from '../ipc/invoke';

export function useMcpTools() {
  return useQuery({
    queryKey: ['mcpTools'],
    queryFn: mcpGetToolDefinitions,
    staleTime: Infinity,
  });
}
