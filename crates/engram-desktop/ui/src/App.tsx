import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { Toaster } from 'sonner';
import { KanbanBoard } from './components/KanbanBoard';
import { IssueDetail } from './routes/IssueDetail';
import { useUIStore } from './store/ui';

const queryClient = new QueryClient();

function AppContent() {
  const { selectedIssueId } = useUIStore();

  return (
    <div className="flex flex-col h-screen bg-white">
      <header className="border-b border-slate-200 px-6 py-3 flex items-center gap-3 flex-shrink-0">
        <span className="font-bold text-slate-800 text-lg">Engram</span>
        <span className="text-xs bg-slate-100 text-slate-500 rounded px-2 py-0.5">Kanban</span>
      </header>
      <main className="flex-1 overflow-hidden">
        <KanbanBoard />
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
