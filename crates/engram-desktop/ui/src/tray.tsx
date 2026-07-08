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
import { Activity, AlertCircle, AlertOctagon, Clock, CheckCircle2, ExternalLink } from 'lucide-react';

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

  const totalAlerts = (summary?.inbox ?? 0) + (summary?.demo_review ?? 0);
  const working = summary?.working ?? 0;
  const workingState = summary?.working_state ?? 'none';

  // 상태 배너 카드 설정 (ATK 디자인 참고)
  let bannerConfig = {
    bg: 'bg-slate-500/10 border-slate-500/20 text-slate-300 hover:bg-slate-500/15',
    icon: <CheckCircle2 className="h-4 w-4 shrink-0 text-slate-400" />,
    text: '모든 프로세스 정상 작동 중'
  };

  if (working > 0) {
    if (workingState === 'active') {
      bannerConfig = {
        bg: 'bg-emerald-500/10 border-emerald-500/20 text-emerald-400 hover:bg-emerald-500/15',
        icon: <Activity className="h-4 w-4 shrink-0 text-emerald-400 animate-pulse" />,
        text: `${working}개 이슈 작업 진행 중`
      };
    } else if (workingState === 'stalled') {
      bannerConfig = {
        bg: 'bg-rose-500/10 border-rose-500/20 text-rose-400 hover:bg-rose-500/15',
        icon: <AlertOctagon className="h-4 w-4 shrink-0 text-rose-400 animate-bounce" />,
        text: `${working}개 이슈 작업 중단 의심`
      };
    } else {
      bannerConfig = {
        bg: 'bg-sky-500/10 border-sky-500/20 text-sky-400 hover:bg-sky-500/15',
        icon: <Clock className="h-4 w-4 shrink-0 text-sky-400" />,
        text: `${working}개 이슈 작업 예정`
      };
    }
  } else if (totalAlerts > 0) {
    bannerConfig = {
      bg: 'bg-amber-500/10 border-amber-500/20 text-amber-400 hover:bg-amber-500/15',
      icon: <AlertCircle className="h-4 w-4 shrink-0 text-amber-400" />,
      text: `${totalAlerts}건의 승인/검토 대기 중`
    };
  }

  return (
    <div className="select-none h-screen flex flex-col p-1.5">
      <div className="bg-slate-950/90 backdrop-blur-xl rounded-2xl overflow-hidden flex flex-col h-full border border-white/[0.08] shadow-[0_12px_40px_rgba(0,0,0,0.5)]">

        {/* 헤더 - 고정 */}
        <div className="flex-none px-4 pt-4 pb-3">
          <div className="flex items-center justify-between">
            <h1 className="text-[15px] font-bold text-white/95 tracking-wide">Engram</h1>
            <span className="flex items-center gap-1.5 text-[11px] font-semibold text-emerald-400">
              <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-emerald-400" />
              LIVE
            </span>
          </div>
        </div>

        {/* 상태 배너 영역 */}
        <div className="flex-none px-4 pb-3">
          <button
            onClick={() => void invoke('show_main_window')}
            className={`w-full flex items-center gap-2.5 rounded-xl border ${bannerConfig.bg} px-3 py-2 text-left text-xs font-medium transition-all duration-200 active:scale-[0.98] cursor-default`}
          >
            {bannerConfig.icon}
            <span className="truncate">{bannerConfig.text}</span>
          </button>
        </div>

        <div className="h-px bg-white/[0.08] mx-4" />

        {/* 중앙 콘텐츠 - 스크롤 */}
        <div className="flex-1 overflow-y-auto custom-scrollbar">
          {/* 요약 통계 */}
          <div className="px-4 py-3 hover:bg-white/[0.02] transition-colors">
            <TraySummary summary={summary} />
          </div>

          <div className="h-px bg-white/[0.08] mx-4" />

          {/* 최근 알림 */}
          <div className="px-4 py-3">
            <p className="text-[11px] font-semibold text-white/40 uppercase tracking-wider mb-2">최근 알림</p>
            <TrayNotificationList entries={log} />
          </div>

          <div className="h-px bg-white/[0.08] mx-4" />

          {/* MCP 서버 */}
          <div className="px-4 py-3">
            <p className="text-[11px] font-semibold text-white/40 uppercase tracking-wider mb-2">MCP 서버</p>
            <TrayMcpStatus />
          </div>
        </div>

        {/* 푸터 - 고정 */}
        <div className="flex-none bg-slate-950/40 px-4 py-3">
          <div className="h-px bg-white/[0.08] -mx-4 mb-3" />
          <button
            onClick={() => void invoke('show_main_window')}
            className="w-full flex items-center justify-between text-[13px] font-medium text-sky-400 hover:text-sky-300 bg-white/[0.03] hover:bg-white/[0.06] active:bg-white/[0.01] border border-white/[0.06] rounded-xl px-3.5 py-2 transition-all cursor-default group"
          >
            <span>보드 열기</span>
            <ExternalLink className="h-3.5 w-3.5 opacity-60 group-hover:opacity-100 group-hover:translate-x-0.5 group-hover:-translate-y-0.5 transition-all duration-200" />
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

