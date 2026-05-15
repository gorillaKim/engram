import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import type { LogLine } from '../ipc/types';

export function useMcpLogs() {
  const [logs, setLogs] = useState<LogLine[]>([]);

  useEffect(() => {
    const unlisten = listen<LogLine>('mcp://log', (e) => {
      setLogs(prev => [...prev, e.payload].slice(-100));
    });
    return () => { unlisten.then(fn => fn()); };
  }, []);

  return logs;
}
