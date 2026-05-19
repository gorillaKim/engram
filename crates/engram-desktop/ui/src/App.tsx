import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { Toaster } from 'sonner';
import { KanbanBoard } from './components/KanbanBoard';
import { History } from './routes/History';
import { IssueDetail } from './routes/IssueDetail';
import { IssueManager } from './routes/IssueManager';
import { McpManager } from './routes/McpManager';
import { useUIStore } from './store/ui';

const queryClient = new QueryClient();

function AppContent() {
  const { selectedIssueId, view, setView } = useUIStore();

  return (
    <div className="flex flex-col h-screen bg-slate-50/50">
      <header className="border-b border-slate-200 px-6 py-3 flex items-center justify-between bg-white flex-shrink-0 z-10 relative overflow-x-auto">
        <div className="flex items-center gap-4">
          <span className="font-bold text-slate-900 text-xl tracking-tight">Engram</span>
          
          <nav className="flex items-center p-1 bg-slate-100 rounded-lg ml-4">
            <button
              onClick={() => setView('board')}
              className={`text-sm px-4 py-1.5 rounded-md font-medium transition-all ${
                view === 'board' 
                  ? 'bg-white text-indigo-600 shadow-sm ring-1 ring-slate-200' 
                  : 'text-slate-500 hover:text-slate-700'
              }`}
            >
              칸반보드
            </button>
            <button
              onClick={() => setView('history')}
              className={`text-sm px-4 py-1.5 rounded-md font-medium transition-all ${
                view === 'history' 
                  ? 'bg-white text-indigo-600 shadow-sm ring-1 ring-slate-200' 
                  : 'text-slate-500 hover:text-slate-700'
              }`}
            >
              히스토리
            </button>
            <button
              onClick={() => setView('issues')}
              className={`text-sm px-4 py-1.5 rounded-md font-medium transition-all ${
                view === 'issues' 
                  ? 'bg-white text-indigo-600 shadow-sm ring-1 ring-slate-200' 
                  : 'text-slate-500 hover:text-slate-700'
              }`}
            >
              이슈관리
            </button>
            <button
              onClick={() => setView('mcp')}
              className={`text-sm px-4 py-1.5 rounded-md font-medium transition-all ${
                view === 'mcp' 
                  ? 'bg-white text-indigo-600 shadow-sm ring-1 ring-slate-200' 
                  : 'text-slate-500 hover:text-slate-700'
              }`}
            >
              MCP 서버
            </button>
          </nav>
        </div>

        <div className="flex items-center gap-2">
          {/* 우측 도구 영역 - 향후 추가 가능 */}
        </div>
      </header>
      <main className="flex-1 overflow-hidden">
        {view === 'board' && <KanbanBoard />}
        {view === 'history' && <History />}
        {view === 'issues' && <IssueManager />}
        {view === 'mcp' && <McpManager />}
      </main>
      {selectedIssueId != null && <IssueDetail />}
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
