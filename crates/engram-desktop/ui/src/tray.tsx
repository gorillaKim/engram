import React, { useEffect, useState } from 'react';
import ReactDOM from 'react-dom/client';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
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
    const unlistenRequired = listen<{ id: number; title: string }>('tray://new_required', (e) => {
      useNotificationStore.getState().add({ id: `req:${e.payload.id}`, title: '승인 대기', body: `#${e.payload.id} ${e.payload.title}` });
    });
    const unlistenDemo = listen<{ id: number; title: string }>('tray://entered_demo', (e) => {
      useNotificationStore.getState().add({ id: `demo:${e.payload.id}`, title: '검토 대기', body: `#${e.payload.id} ${e.payload.title}` });
    });
    const unlistenBlocker = listen<{ count: number }>('tray://new_blocker', (e) => {
      useNotificationStore.getState().add({ id: `blocker:${Date.now()}`, title: '새 블로커', body: `${e.payload.count}개 이슈가 블로킹됨` });
    });

    return () => {
      void unlistenSummary.then(fn => fn());
      void unlistenRequired.then(fn => fn());
      void unlistenDemo.then(fn => fn());
      void unlistenBlocker.then(fn => fn());
    };
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const totalAlerts = (summary?.inbox ?? 0) + (summary?.demo_review ?? 0) + (summary?.blockers ?? 0);

  return (
    <div className="select-none">
      <div className="bg-[#2c2c2e]/90 backdrop-blur-[30px] rounded-[12px] overflow-hidden">

        {/* 헤더 */}
        <div className="px-4 pt-4 pb-3 flex items-baseline justify-between">
          <h1 className="text-[15px] font-semibold text-white/90">Engram</h1>
          <span className={`text-[12px] ${totalAlerts > 0 ? 'text-amber-400' : 'text-white/30'}`}>
            {totalAlerts > 0 ? `${totalAlerts}건 주의` : '이상 없음'}
          </span>
        </div>

        <div className="h-px bg-white/[0.08]" />

        {/* 요약 통계 */}
        <div className="px-4 py-3">
          <TraySummary summary={summary} />
        </div>

        <div className="h-px bg-white/[0.08]" />

        {/* 최근 알림 */}
        <div className="px-4 py-3">
          <p className="text-[11px] font-medium text-white/40 uppercase tracking-wider mb-2">최근 알림</p>
          <TrayNotificationList entries={log} />
        </div>

        <div className="h-px bg-white/[0.08]" />

        {/* MCP 서버 */}
        <div className="px-4 py-3">
          <p className="text-[11px] font-medium text-white/40 uppercase tracking-wider mb-2">MCP 서버</p>
          <TrayMcpStatus />
        </div>

        <div className="h-px bg-white/[0.08]" />

        {/* 푸터 */}
        <div className="px-4 py-3">
          <button
            onClick={() => void invoke('show_main_window')}
            className="text-[13px] text-[#4c9ff8] hover:text-[#6db3ff] active:text-[#3a8ee6] transition-colors cursor-default"
          >
            보드 열기...
          </button>
        </div>

      </div>
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
