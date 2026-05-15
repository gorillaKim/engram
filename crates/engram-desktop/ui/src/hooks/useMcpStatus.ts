import { useEffect } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { listen } from '@tauri-apps/api/event';
import { mcpStatus } from '../ipc/invoke';
import type { SupervisorStatusSnapshot } from '../ipc/types';

export function useMcpStatus() {
  const qc = useQueryClient();
  const query = useQuery({
    queryKey: ['mcpStatus'],
    queryFn: mcpStatus,
    refetchInterval: 10_000,
  });

  useEffect(() => {
    const unlisten = listen<SupervisorStatusSnapshot>('mcp://status', (e) => {
      qc.setQueryData(['mcpStatus'], e.payload);
    });
    return () => { unlisten.then(fn => fn()); };
  }, [qc]);

  return query;
}
