import React, { useEffect, useState } from 'react';
import ReactDOM from 'react-dom/client';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { listen } from '@tauri-apps/api/event';
import { Toaster } from 'sonner';
import './index.css';
import { TraySummary } from './components/TraySummary';
import { TrayNotificationList } from './components/TrayNotificationList';
import { TrayMcpStatus } from './components/TrayMcpStatus';
import { useNotificationStore } from './store/notification';
import type { TrayBoardSummary } from './ipc/types';

const qc = new QueryClient();

function TrayApp() {
  const [summary, setSummary] = useState<TrayBoardSummary | null>(null);
  const { log } = useNotificationStore();

  useEffect(() => {
    const unlistenSummary = listen<TrayBoardSummary>('tray://summary', (e) => {
      setSummary(e.payload);
    });

    // Listen for notification events to build in-app log
    const unlistenRequired = listen<{ id: number; title: string }>('tray://new_required', (e) => {
      useNotificationStore.getState().add({ id: `req:${e.payload.id}`, title: '🆕 승인 대기', body: `#${e.payload.id} ${e.payload.title}` });
    });
    const unlistenDemo = listen<{ id: number; title: string }>('tray://entered_demo', (e) => {
      useNotificationStore.getState().add({ id: `demo:${e.payload.id}`, title: '👀 검토 대기', body: `#${e.payload.id} ${e.payload.title}` });
    });
    const unlistenBlocker = listen<{ count: number }>('tray://new_blocker', (e) => {
      useNotificationStore.getState().add({ id: `blocker:${Date.now()}`, title: '🚫 새 블로커', body: `${e.payload.count}개 이슈가 블로킹됨` });
    });

    return () => {
      void unlistenSummary.then(fn => fn());
      void unlistenRequired.then(fn => fn());
      void unlistenDemo.then(fn => fn());
      void unlistenBlocker.then(fn => fn());
    };
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <div className="bg-white/95 backdrop-blur-sm rounded-2xl shadow-2xl border border-slate-200 p-4 min-h-screen flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <h1 className="text-sm font-bold text-slate-800">Engram</h1>
      </div>

      <TraySummary summary={summary} />

      <section>
        <h2 className="text-xs font-semibold text-slate-400 uppercase tracking-wider mb-2">최근 알림</h2>
        <TrayNotificationList entries={log} />
      </section>

      <hr className="border-slate-100" />

      <section>
        <h2 className="text-xs font-semibold text-slate-400 uppercase tracking-wider mb-2">MCP 서버</h2>
        <TrayMcpStatus />
      </section>
    </div>
  );
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <QueryClientProvider client={qc}>
      <TrayApp />
      <Toaster position="top-right" />
    </QueryClientProvider>
  </React.StrictMode>
);
