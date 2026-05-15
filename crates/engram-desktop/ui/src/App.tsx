import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { Toaster } from 'sonner';
import { KanbanBoard } from './components/KanbanBoard';
import { IssueDetail } from './routes/IssueDetail';
import { McpManager } from './routes/McpManager';
import { useUIStore } from './store/ui';

const queryClient = new QueryClient();

function AppContent() {
  const { selectedIssueId, view, setView } = useUIStore();

  return (
    <div className="flex flex-col h-screen bg-white">
      <header className="border-b border-slate-200 px-6 py-3 flex items-center gap-3 flex-shrink-0">
        <span className="font-bold text-slate-800 text-lg">Engram</span>
        <div className="flex items-center gap-2 ml-2">
          <button
            onClick={() => setView('board')}
            className={`text-sm px-3 py-1 rounded ${view === 'board' ? 'bg-indigo-600 text-white' : 'text-slate-600 hover:bg-slate-100'}`}
          >
            칸반
          </button>
          <button
            onClick={() => setView('mcp')}
            className={`text-sm px-3 py-1 rounded ${view === 'mcp' ? 'bg-indigo-600 text-white' : 'text-slate-600 hover:bg-slate-100'}`}
          >
            MCP 서버
          </button>
        </div>
      </header>
      <main className="flex-1 overflow-hidden">
        {view === 'board' ? <KanbanBoard /> : <McpManager />}
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
