import { useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { mcpRestart, mcpStop, mcpStatus } from '../ipc/invoke';
import { useQuery } from '@tanstack/react-query';
import { listen } from '@tauri-apps/api/event';
import type { SupervisorStatusSnapshot } from '../ipc/types';
import { useEffect } from 'react';

export function TrayMcpStatus() {
  const qc = useQueryClient();
  const { data: status } = useQuery({
    queryKey: ['mcpStatus'],
    queryFn: mcpStatus,
    staleTime: Infinity,
  });

  useEffect(() => {
    const unlisten = listen<SupervisorStatusSnapshot>('mcp://status', (e) => {
      qc.setQueryData(['mcpStatus'], e.payload);
    });
    return () => { unlisten.then(fn => fn()); };
  }, [qc]);

  const mutOpts = {
    onSuccess: (snap: SupervisorStatusSnapshot) => qc.setQueryData(['mcpStatus'], snap),
    onError: (err: unknown) => toast.error(`${err}`),
  };
  const restartMut = useMutation({
    mutationFn: () => mcpRestart(status?.port ?? 3456),
    ...mutOpts,
  });
  const stopMut = useMutation({ mutationFn: mcpStop, ...mutOpts });

  return (
    <div className="flex items-center justify-between text-xs">
      <div className="flex items-center gap-1.5">
        <span className={`w-2 h-2 rounded-full ${status?.running ? 'bg-green-500' : 'bg-slate-400'}`} />
        <span className="text-slate-700 font-medium">MCP</span>
        {status?.running && (
          <span className="text-slate-400">:{status.port} · {status.call_count}calls</span>
        )}
      </div>
      <div className="flex gap-1">
        <button
          onClick={() => restartMut.mutate()}
          disabled={restartMut.isPending}
          className="px-1.5 py-0.5 rounded bg-slate-100 hover:bg-slate-200 text-slate-600 disabled:opacity-40"
          title="재시작"
        >
          ↻
        </button>
        <button
          onClick={() => stopMut.mutate()}
          disabled={!status?.running || stopMut.isPending}
          className="px-1.5 py-0.5 rounded bg-slate-100 hover:bg-slate-200 text-slate-600 disabled:opacity-40"
          title="정지"
        >
          ■
        </button>
      </div>
    </div>
  );
}
