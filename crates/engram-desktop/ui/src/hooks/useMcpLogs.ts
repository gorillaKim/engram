import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { mcpRecentLogs } from '../ipc/invoke';
import type { LogLine } from '../ipc/types';

export function useMcpLogs() {
  const [logs, setLogs] = useState<LogLine[]>([]);

  useEffect(() => {
    // 1) 초기 버퍼 로그 로드
    mcpRecentLogs()
      .then((initialLogs) => {
        setLogs(initialLogs.slice(-100));
      })
      .catch((err) => console.error("최근 MCP 로그 로드 실패:", err));

    // 2) 실시간 이벤트 수신
    const unlisten = listen<LogLine>('mcp://log', (e) => {
      setLogs(prev => [...prev, e.payload].slice(-100));
    });
    return () => { unlisten.then(fn => fn()); };
  }, []);

  return logs;
}
