import { useEffect, useState } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { Toaster } from 'sonner';
import { KanbanBoard } from './components/KanbanBoard';
import { History } from './routes/History';
import { IssueDetail } from './routes/IssueDetail';
import { EpicDetail } from './routes/EpicDetail';
import { MissionDetail } from './routes/MissionDetail';
import { IssueManager } from './routes/IssueManager';
import { McpManager } from './routes/McpManager';
import { MissionsBoard } from './routes/MissionsBoard';
import { Settings } from './routes/Settings';
import { Guide } from './routes/Guide';
import { useUIStore } from './store/ui';
import {
  checkForUpdates,
  downloadAndInstall,
  relaunchApp,
} from './services/updateManager';
import type { Update } from './services/updateManager';

const queryClient = new QueryClient();

type ModalState = 'prompt' | 'downloading' | 'installed';

function UpdateModal({
  update,
  onClose,
}: {
  update: Update;
  onClose: () => void;
}) {
  const [state, setState] = useState<ModalState>('prompt');
  const [progress, setProgress] = useState(0);

  useEffect(() => {
    const onInstalled = () => setState('installed');
    window.addEventListener('update:installed', onInstalled);
    return () => window.removeEventListener('update:installed', onInstalled);
  }, []);

  async function handleInstall() {
    setState('downloading');
    await downloadAndInstall((pct) => setProgress(pct));
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
      <div className="w-96 rounded-xl border border-slate-700 bg-slate-800 p-6 shadow-2xl flex flex-col gap-4">
        <h2 className="text-lg font-semibold text-slate-100">업데이트 사용 가능</h2>
        <p className="text-sm text-slate-300">
          새 버전{' '}
          <span className="font-mono text-slate-100">v{update.version}</span>이
          출시되었습니다.
        </p>
        {update.body && (
          <p className="text-xs text-slate-400 line-clamp-4 whitespace-pre-wrap">
            {update.body}
          </p>
        )}

        {state === 'downloading' && (
          <div className="flex flex-col gap-1">
            <div className="flex justify-between text-xs text-slate-400">
              <span>다운로드 중…</span>
              <span className="font-mono">{progress}%</span>
            </div>
            <div className="h-1.5 w-full rounded-full bg-slate-700 overflow-hidden">
              <div
                className="h-full rounded-full bg-blue-500 transition-all duration-200"
                style={{ width: `${progress}%` }}
              />
            </div>
          </div>
        )}

        <div className="flex gap-2 justify-end pt-1">
          {state === 'prompt' && (
            <>
              <button
                onClick={onClose}
                className="px-4 py-1.5 text-sm rounded-md text-slate-400 hover:text-slate-200 transition-colors"
              >
                나중에
              </button>
              <button
                onClick={handleInstall}
                className="px-4 py-1.5 text-sm rounded-md bg-blue-600 text-white hover:bg-blue-500 transition-colors"
              >
                지금 설치
              </button>
            </>
          )}
          {state === 'installed' && (
            <button
              onClick={relaunchApp}
              className="px-4 py-1.5 text-sm rounded-md bg-emerald-600 text-white hover:bg-emerald-500 transition-colors"
            >
              재시작
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

function AppContent() {
  const { selectedIssueId, selectedEpicId, selectedMissionId, view, setView, selectProject, popPanel } = useUIStore();
  const [pendingUpdate, setPendingUpdate] = useState<Update | null>(null);

  useEffect(() => {
    checkForUpdates({ silent: true }).then((update) => {
      if (update) setPendingUpdate(update);
    });
  }, []);

  // Esc key down global listener to pop drawers in LIFO order
  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === 'Escape') {
        const activeEl = document.activeElement;
        if (
          activeEl &&
          (activeEl.tagName === 'INPUT' ||
            activeEl.tagName === 'TEXTAREA' ||
            activeEl.getAttribute('contenteditable') === 'true')
        ) {
          (activeEl as HTMLElement).blur();
          return;
        }
        popPanel();
      }
    }
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [popPanel]);

  // URL 쿼리 스트링의 view 값을 기반으로 초기 탭 상태 설정
  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const viewParam = params.get('view');
    if (viewParam && ['board', 'missions', 'issues', 'history', 'mcp', 'settings', 'guide'].includes(viewParam)) {
      setView(viewParam as any);
    }
  }, [setView]);

  const handleSetView = (key: 'board' | 'missions' | 'issues' | 'history' | 'mcp' | 'settings' | 'guide') => {
    setView(key);
    selectProject(null); // 다른 탭으로 가거나 탭을 클릭할 때 프로젝트 포커스 해제
    const params = new URLSearchParams(window.location.search);
    params.set('view', key);
    
    // issues 탭이 아닐 때는 다른 상세 필터 쿼리들을 정리하여 주소창을 깔끔하게 유지할 수 있음
    if (key !== 'issues') {
      params.delete('sprint');
      params.delete('missions');
      params.delete('epics');
      params.delete('statuses');
      params.delete('priorities');
      params.delete('agents');
      params.delete('q');
    }
    
    const newSearch = params.toString();
    const newUrl = `${window.location.pathname}${newSearch ? '?' + newSearch : ''}`;
    window.history.replaceState(null, '', newUrl);
  };

  return (
    <div className="flex flex-col h-screen bg-slate-50/50">
      <header className="border-b border-slate-200 px-6 py-3 flex items-center justify-between bg-white flex-shrink-0 z-30 relative overflow-x-auto">
        <div className="flex items-center gap-4">
          <span className="font-bold text-slate-900 text-xl tracking-tight">Engram</span>
          
          <nav className="flex items-center p-1 bg-slate-100 rounded-lg ml-4">
            {([
              { key: 'board',    label: '칸반보드' },
              { key: 'missions', label: 'Missions' },
              { key: 'issues',   label: '이슈관리' },
              { key: 'history',  label: '히스토리' },
              { key: 'mcp',      label: 'MCP 서버' },
              { key: 'guide',    label: '사용 가이드' },
              { key: 'settings', label: '설정' },
            ] as const).map(({ key, label }) => (
              <button
                key={key}
                onClick={() => handleSetView(key)}
                className={`text-sm px-4 py-1.5 rounded-md font-medium transition-all ${
                  view === key
                    ? 'bg-white text-indigo-600 shadow-sm ring-1 ring-slate-200'
                    : 'text-slate-500 hover:text-slate-700'
                }`}
              >
                {label}
              </button>
            ))}
          </nav>
        </div>

        <div className="flex items-center gap-2">
          {/* 우측 도구 영역 - 향후 추가 가능 */}
        </div>
      </header>
      {/* Container holding Main and Side Panels */}
      <div className="flex-1 flex overflow-hidden min-h-0 relative">
        {/* Main Content Area */}
        <main className="flex-1 min-w-0 h-full min-h-0 flex flex-col overflow-hidden">
          {view === 'board' && <KanbanBoard />}
          {view === 'history' && <History />}
          {view === 'issues' && <IssueManager />}
          {view === 'mcp' && <McpManager />}
          {view === 'missions' && <MissionsBoard />}
          {view === 'guide' && <Guide />}
          {view === 'settings' && <Settings />}
        </main>
      </div>

      {/* Floating Overlay Drawers: All views use the same overlay modal style */}
      {(selectedIssueId != null || selectedEpicId != null || selectedMissionId != null) && (
        <div className="fixed inset-0 z-40 flex justify-end pointer-events-none">
          {/* Backdrop overlay */}
          <div
            className="absolute inset-0 bg-slate-900/20 backdrop-blur-[2px] pointer-events-auto transition-opacity duration-300"
            onClick={() => {
              useUIStore.getState().selectIssue(null);
              useUIStore.getState().selectEpic(null);
              useUIStore.getState().selectMission(null);
            }}
          />
          {/* Cascading drawers floating over content */}
          <div className="relative flex flex-row-reverse items-stretch h-full gap-4 p-4 pointer-events-none z-10 overflow-x-auto max-w-full">
            {selectedIssueId != null && (
              <div className="pointer-events-auto h-full shadow-2xl rounded-2xl bg-white border border-slate-100 animate-slide-in w-[460px] flex-shrink-0 overflow-hidden">
                <IssueDetail />
              </div>
            )}
            {selectedEpicId != null && (
              <div className="pointer-events-auto h-full shadow-2xl rounded-2xl bg-white border border-slate-100 animate-slide-in w-[460px] flex-shrink-0 overflow-hidden">
                <EpicDetail />
              </div>
            )}
            {selectedMissionId != null && (
              <div className="pointer-events-auto h-full shadow-2xl rounded-2xl bg-white border border-slate-100 animate-slide-in w-[460px] flex-shrink-0 overflow-hidden">
                <MissionDetail />
              </div>
            )}
          </div>
        </div>
      )}

      {pendingUpdate && (
        <UpdateModal update={pendingUpdate} onClose={() => setPendingUpdate(null)} />
      )}
    </div>
  );
}

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <AppContent />
      <Toaster position="bottom-right" richColors />
    </QueryClientProvider>
  );
}
