import { useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { mcpRestart, mcpStop, mcpStatus } from '../ipc/invoke';
import { useQuery } from '@tanstack/react-query';
import { listen } from '@tauri-apps/api/event';
import type { SupervisorStatusSnapshot } from '../ipc/types';
import { useEffect } from 'react';
import { RefreshCw, Square } from 'lucide-react';

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
    <div className="flex items-center justify-between text-xs py-0.5">
      <div className="flex items-center gap-2">
        <span 
          className={`w-2 h-2 rounded-full flex-shrink-0 transition-all duration-300 ${
            status?.running 
              ? 'bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.6)] animate-pulse' 
              : 'bg-white/20'
          }`} 
        />
        <span className="text-white/80 font-semibold tracking-wide">MCP</span>
        {status?.running && (
          <span className="text-[10px] text-white/40 bg-white/[0.04] border border-white/[0.04] rounded-md px-1.5 py-0.5 font-mono">
            {status.port} · {status.call_count} calls
          </span>
        )}
      </div>
      <div className="flex gap-1.5">
        <button
          onClick={() => restartMut.mutate()}
          disabled={restartMut.isPending}
          className="p-1.5 rounded-lg bg-white/[0.04] hover:bg-white/[0.08] active:bg-white/[0.02] text-white/50 hover:text-white/80 disabled:opacity-20 border border-white/[0.05] transition-all duration-200"
          title="MCP 재시작"
        >
          <RefreshCw className={`h-3.5 w-3.5 ${restartMut.isPending ? 'animate-spin' : ''}`} />
        </button>
        <button
          onClick={() => stopMut.mutate()}
          disabled={!status?.running || stopMut.isPending}
          className="p-1.5 rounded-lg bg-white/[0.04] hover:bg-white/[0.08] active:bg-white/[0.02] text-white/50 hover:text-white/80 disabled:opacity-20 border border-white/[0.05] transition-all duration-200"
          title="MCP 정지"
        >
          <Square className="h-3.5 w-3.5 fill-current" />
        </button>
      </div>
    </div>
  );
}

