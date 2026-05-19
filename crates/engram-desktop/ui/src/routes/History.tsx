import { useState, useEffect } from 'react';
import { useIssues } from '../hooks/useIssues';
import { useSprints } from '../hooks/useSprints';
import { useEpics } from '../hooks/useEpics';
import { PriorityBadge } from '../components/PriorityBadge';
import { useUIStore } from '../store/ui';

export function History() {
  const { selectIssue } = useUIStore();
  const [viewMode, setViewMode] = useState<'sprint' | 'epic'>('sprint');
  const [selectedId, setSelectedId] = useState<number | null>(null);

  const { data: sprints } = useSprints();
  const { data: epics } = useEpics();

  // Set default selection when data loads or mode changes
  useEffect(() => {
    if (viewMode === 'sprint' && sprints && sprints.length > 0) {
      if (selectedId === null || !sprints.find(s => s.id === selectedId)) {
        const active = sprints.find(s => s.status === 'active');
        const completed = sprints.filter(s => s.status === 'completed');
        setSelectedId(active?.id ?? completed[completed.length - 1]?.id ?? sprints[0].id);
      }
    } else if (viewMode === 'epic' && epics && epics.length > 0) {
      if (selectedId === null || !epics.find(e => e.id === selectedId)) {
        setSelectedId(epics[0].id);
      }
    }
  }, [viewMode, sprints, epics, selectedId]);

  const filter = {
    status: 'finished' as const,
    ...(viewMode === 'sprint' ? { sprint_id: selectedId } : { epic_id: selectedId ?? undefined }),
  };

  const { data: issues, isLoading } = useIssues(filter);

  const handleModeChange = (mode: 'sprint' | 'epic') => {
    setViewMode(mode);
    setSelectedId(null);
  };

  const finishedIssues = issues ?? [];

  return (
    <div className="flex flex-col h-full bg-slate-50/30">
      {/* 헤더 — 고정 */}
      <div className="flex-shrink-0 px-6 py-4 border-b border-slate-200 bg-white flex items-center justify-between flex-wrap gap-4">
        <h1 className="text-xl font-bold text-slate-800">완료 히스토리</h1>

        <div className="flex items-center gap-3 flex-wrap">
          <nav className="flex items-center p-1 bg-slate-100 rounded-lg">
            <button
              onClick={() => handleModeChange('sprint')}
              className={`text-xs px-3 py-1.5 rounded-md font-medium transition-all ${
                viewMode === 'sprint'
                  ? 'bg-white text-indigo-600 shadow-sm ring-1 ring-slate-200'
                  : 'text-slate-500 hover:text-slate-700'
              }`}
            >
              스프린트별
            </button>
            <button
              onClick={() => handleModeChange('epic')}
              className={`text-xs px-3 py-1.5 rounded-md font-medium transition-all ${
                viewMode === 'epic'
                  ? 'bg-white text-indigo-600 shadow-sm ring-1 ring-slate-200'
                  : 'text-slate-500 hover:text-slate-700'
              }`}
            >
              에픽별
            </button>
          </nav>

          <select
            className="text-sm border border-slate-200 rounded-lg px-3 py-1.5 bg-white shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/20 min-w-[160px]"
            value={selectedId ?? ''}
            onChange={(e) => setSelectedId(Number(e.target.value))}
          >
            <option value="" disabled>선택하세요</option>
            {viewMode === 'sprint' ? (
              sprints?.map(s => (
                <option key={s.id} value={s.id}>
                  {s.name} {s.status === 'active' ? '(활성)' : ''}
                </option>
              ))
            ) : (
              epics?.map(e => (
                <option key={e.id} value={e.id}>{e.title}</option>
              ))
            )}
          </select>

          <span className="text-sm text-slate-500 font-medium whitespace-nowrap">
            총 {finishedIssues.length}건
          </span>
        </div>
      </div>

      {/* 테이블 영역 — 스크롤 */}
      <div className="flex-1 overflow-auto p-6">
        {isLoading ? (
          <div className="flex items-center justify-center h-40 text-slate-400">
            히스토리 로딩 중…
          </div>
        ) : finishedIssues.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-40 bg-white rounded-xl border-2 border-dashed border-slate-200 p-12 text-slate-400">
            <span className="text-4xl mb-3">📦</span>
            <p>해당 {viewMode === 'sprint' ? '스프린트' : '에픽'}에 완료된 이슈가 없습니다.</p>
          </div>
        ) : (
          <div className="bg-white rounded-xl border border-slate-200 shadow-sm">
            <table className="w-full text-sm text-left">
              <thead className="bg-slate-50 border-b border-slate-200 text-[11px] font-bold text-slate-500 uppercase tracking-wider sticky top-0">
                <tr>
                  <th className="px-6 py-3 w-16">ID</th>
                  <th className="px-6 py-3">제목</th>
                  <th className="px-6 py-3 w-24">우선순위</th>
                  <th className="px-6 py-3 w-40">완료일</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100">
                {finishedIssues.map((issue) => (
                  <tr
                    key={issue.id}
                    className="hover:bg-slate-50/80 cursor-pointer transition-colors"
                    onClick={() => selectIssue(issue.id)}
                  >
                    <td className="px-6 py-4 text-slate-400 font-mono">#{issue.id}</td>
                    <td className="px-6 py-4">
                      <span className="font-medium text-slate-800">{issue.title}</span>
                    </td>
                    <td className="px-6 py-4">
                      <PriorityBadge priority={issue.priority} />
                    </td>
                    <td className="px-6 py-4 text-slate-500">
                      {new Date(issue.updated_at).toLocaleDateString()}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}
