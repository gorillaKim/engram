import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { mcpRecentCalls } from '../ipc/invoke';
import type { CallRecord } from '../ipc/types';

export function useMcpCalls() {
  const [calls, setCalls] = useState<CallRecord[]>([]);

  useEffect(() => {
    mcpRecentCalls().then(setCalls);
    const unlisten = listen<CallRecord>('mcp://call', (e) => {
      setCalls(prev => [e.payload, ...prev].slice(0, 200));
    });
    return () => { unlisten.then(fn => fn()); };
  }, []);

  return calls;
}
