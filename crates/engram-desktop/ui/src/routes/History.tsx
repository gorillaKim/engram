import { useState, useEffect, useMemo } from 'react';
import { useQuery } from '@tanstack/react-query';
import { useIssues } from '../hooks/useIssues';
import { useSprints } from '../hooks/useSprints';
import { useEpics } from '../hooks/useEpics';
import { useDebounce } from '../hooks/useDebounce';
import { PriorityBadge } from '../components/PriorityBadge';
import { useUIStore } from '../store/ui';
import { missionList } from '../ipc/invoke';
import type { Mission } from '../ipc/types';
import { sortHistoryIssues, paginateHistoryIssues, calculateHistoryStats, SortKey, SortOrder } from '../utils/historyHelper';

export function History() {
  const { selectIssue } = useUIStore();
  const [viewMode, setViewMode] = useState<'sprint' | 'epic' | 'mission'>('sprint');
  const [selectedId, setSelectedId] = useState<number | null>(null);

  // 정렬 및 페이징 상태
  const [sortKey, setSortKey] = useState<SortKey>('updated_at');
  const [sortOrder, setSortOrder] = useState<SortOrder>('desc');
  const [visibleCount, setVisibleCount] = useState<number>(30);

  const { data: sprints } = useSprints();
  const { data: epics } = useEpics();
  const { data: missions } = useQuery<Mission[]>({
    queryKey: ['missionList', 'all'],
    queryFn: () => missionList(null, true),
  });

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
    } else if (viewMode === 'mission' && missions && missions.length > 0) {
      if (selectedId === null || !missions.find(m => m.id === selectedId)) {
        setSelectedId(missions[0].id);
      }
    }
  }, [viewMode, sprints, epics, missions, selectedId]);

  // 필터나 모드 변경 시 페이징 수 초기화
  useEffect(() => {
    setVisibleCount(30);
  }, [viewMode, selectedId]);

  // 전체 이슈 조회 (상태 필터 없음) — finished/cancelled 표시 + 진척률 계산에 모두 사용
  const totalFilter = useMemo(() => ({
    ...(viewMode === 'sprint'
      ? { sprint_id: selectedId }
      : viewMode === 'epic'
      ? { epic_id: selectedId ?? undefined }
      : {}),
  }), [viewMode, selectedId]);

  const { data: allIssues, isLoading } = useIssues(totalFilter);

  const handleModeChange = (mode: 'sprint' | 'epic' | 'mission') => {
    setViewMode(mode);
    setSelectedId(null);
  };

  const [searchQuery, setSearchQuery] = useState('');
  const debouncedQuery = useDebounce(searchQuery);

  const finishedIssues = useMemo(() => {
    const raw = (allIssues ?? []).filter(
      (i) => i.status === 'finished' || i.status === 'cancelled',
    );
    if (viewMode === 'mission' && selectedId != null) {
      return raw.filter((i) => i.mission_id === selectedId);
    }
    return raw;
  }, [allIssues, viewMode, selectedId]);

  // 통계 계산
  const stats = useMemo(() => calculateHistoryStats(finishedIssues), [finishedIssues]);
  
  const missionScopedAllIssues = useMemo(() => {
    if (viewMode === 'mission' && selectedId != null) {
      return (allIssues ?? []).filter((i) => i.mission_id === selectedId);
    }
    return allIssues ?? [];
  }, [allIssues, viewMode, selectedId]);

  const progressPercent = useMemo(() => {
    if (missionScopedAllIssues.length === 0) return 0;
    const finishedCount = missionScopedAllIssues.filter(i => i.status === 'finished').length;
    return Math.round((finishedCount / missionScopedAllIssues.length) * 100);
  }, [missionScopedAllIssues]);

  const totalIssueCount = missionScopedAllIssues.length;
  const finishedIssueCount = missionScopedAllIssues.filter(i => i.status === 'finished').length;

  const filteredIssues = useMemo(() => {
    const q = debouncedQuery.trim().toLowerCase();
    if (!q) return finishedIssues;
    if (q.startsWith('#')) {
      const id = parseInt(q.slice(1));
      return isNaN(id) ? [] : finishedIssues.filter((i) => i.id === id);
    }
    return finishedIssues.filter((i) => i.title.toLowerCase().includes(q));
  }, [finishedIssues, debouncedQuery]);

  // 정렬 및 페이징 계산 적용
  const sortedIssues = useMemo(() => {
    return sortHistoryIssues(filteredIssues, sortKey, sortOrder);
  }, [filteredIssues, sortKey, sortOrder]);

  const paginatedIssues = useMemo(() => {
    return paginateHistoryIssues(sortedIssues, visibleCount);
  }, [sortedIssues, visibleCount]);

  const handleSort = (key: SortKey) => {
    if (sortKey === key) {
      setSortOrder(sortOrder === 'asc' ? 'desc' : 'asc');
    } else {
      setSortKey(key);
      setSortOrder('desc');
    }
  };

  const renderSortIndicator = (key: SortKey) => {
    if (sortKey !== key) return <span className="text-slate-300 ml-1">⇅</span>;
    return sortOrder === 'asc' ? <span className="text-indigo-600 ml-1">▲</span> : <span className="text-indigo-600 ml-1">▼</span>;
  };

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
              onClick={() => handleModeChange('mission')}
              className={`text-xs px-3 py-1.5 rounded-md font-medium transition-all ${
                viewMode === 'mission'
                  ? 'bg-white text-violet-600 shadow-sm ring-1 ring-slate-200'
                  : 'text-slate-500 hover:text-slate-700'
              }`}
            >
              미션별
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
            ) : viewMode === 'mission' ? (
              missions?.map(m => (
                <option key={m.id} value={m.id}>
                  {m.title} {m.status === 'active' ? '' : `(${m.status})`}
                </option>
              ))
            ) : (
              epics?.map(e => (
                <option key={e.id} value={e.id}>{e.title}</option>
              ))
            )}
          </select>

          <input
            type="text"
            placeholder="#ID 또는 제목 검색…"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="text-sm border border-slate-200 rounded-lg px-3 py-1.5 bg-white shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/20 min-w-[180px]"
          />

          <span className="text-sm text-slate-500 font-medium whitespace-nowrap">
            총 {filteredIssues.length}건
          </span>
        </div>
      </div>

      {/* 테이블 및 통계 영역 — 스크롤 */}
      <div className="flex-1 overflow-auto p-6 space-y-6">
        {/* 통계 미니 대시보드 */}
        {selectedId !== null && !isLoading && (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6 bg-white border border-slate-200 rounded-xl p-5 shadow-sm">
            {/* 완료 진척률 */}
            <div className="flex flex-col justify-between space-y-3">
              <div>
                <h3 className="text-xs font-semibold uppercase tracking-wider text-slate-400">
                  {viewMode === 'sprint' ? '스프린트' : viewMode === 'mission' ? '미션' : '에픽'} 완료율
                </h3>
                <div className="flex items-baseline gap-2 mt-1">
                  <span className="text-2xl font-bold text-slate-800">{progressPercent}%</span>
                  <span className="text-xs text-slate-500">
                    ({finishedIssueCount} / {totalIssueCount} 완료)
                  </span>
                </div>
              </div>
              <div className="w-full bg-slate-100 rounded-full h-3 overflow-hidden">
                <div
                  className="bg-indigo-600 h-full rounded-full transition-all duration-500 ease-out"
                  style={{ width: `${progressPercent}%` }}
                />
              </div>
            </div>

            {/* 해결 이슈 우선순위 분포 */}
            <div className="flex flex-col space-y-3">
              <h3 className="text-xs font-semibold uppercase tracking-wider text-slate-400">
                해결 이슈 우선순위 분포
              </h3>
              <div className="grid grid-cols-4 gap-2">
                {[
                  { key: 'critical', label: '긴급', color: 'bg-red-500 text-red-700 bg-red-50' },
                  { key: 'high', label: '높음', color: 'bg-orange-500 text-orange-700 bg-orange-50' },
                  { key: 'medium', label: '보통', color: 'bg-amber-500 text-amber-700 bg-amber-50' },
                  { key: 'low', label: '낮음', color: 'bg-sky-500 text-sky-700 bg-sky-50' },
                ].map((item) => {
                  const count = stats.priorityCounts[item.key as keyof typeof stats.priorityCounts] ?? 0;
                  const ratio = stats.totalCount > 0 ? Math.round((count / stats.totalCount) * 100) : 0;
                  return (
                    <div key={item.key} className={`p-2.5 rounded-lg border border-slate-100 flex flex-col items-center justify-between text-center`}>
                      <span className="text-[10px] font-semibold text-slate-500 mb-0.5">{item.label}</span>
                      <span className="text-base font-bold text-slate-800">{count}</span>
                      <span className="text-[9px] text-slate-400 font-mono mt-0.5">{ratio}%</span>
                    </div>
                  );
                })}
              </div>
            </div>
          </div>
        )}

        {isLoading ? (
          <div className="flex items-center justify-center h-40 text-slate-400">
            히스토리 로딩 중…
          </div>
        ) : filteredIssues.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-40 bg-white rounded-xl border-2 border-dashed border-slate-200 p-12 text-slate-400">
            <span className="text-4xl mb-3">📦</span>
            <p>
              {searchQuery.trim()
                ? `"${searchQuery.trim()}" 에 일치하는 이슈가 없습니다.`
                : `해당 ${viewMode === 'sprint' ? '스프린트' : viewMode === 'mission' ? '미션' : '에픽'}에 완료된 이슈가 없습니다.`}
            </p>
          </div>
        ) : (
          <div className="space-y-4">
            <div className="bg-white rounded-xl border border-slate-200 shadow-sm overflow-hidden">
              <table className="w-full text-sm text-left table-fixed">
                <thead className="bg-slate-50 border-b border-slate-200 text-[11px] font-bold text-slate-500 uppercase tracking-wider sticky top-0 z-10">
                  <tr>
                    <th className="px-6 py-3 w-20 cursor-pointer hover:bg-slate-100 transition-colors select-none" onClick={() => handleSort('id')}>
                      ID {renderSortIndicator('id')}
                    </th>
                    <th className="px-6 py-3 cursor-pointer hover:bg-slate-100 transition-colors select-none" onClick={() => handleSort('title')}>
                      제목 {renderSortIndicator('title')}
                    </th>
                    <th className="px-6 py-3 w-48 text-slate-500">에픽</th>
                    <th className="px-6 py-3 w-20 text-slate-500">상태</th>
                    <th className="px-6 py-3 w-28 cursor-pointer hover:bg-slate-100 transition-colors select-none" onClick={() => handleSort('priority')}>
                      우선순위 {renderSortIndicator('priority')}
                    </th>
                    <th className="px-6 py-3 w-40 cursor-pointer hover:bg-slate-100 transition-colors select-none" onClick={() => handleSort('updated_at')}>
                      완료/취소일 {renderSortIndicator('updated_at')}
                    </th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-100">
                  {paginatedIssues.map((issue) => {
                    const epicTitle = epics?.find((e) => e.id === issue.epic_id)?.title ?? '-';
                    return (
                      <tr
                        key={issue.id}
                        className="hover:bg-slate-50/80 cursor-pointer transition-colors"
                        onClick={() => selectIssue(issue.id)}
                      >
                        <td className="px-6 py-4 text-slate-400 font-mono">#{issue.id}</td>
                        <td className="px-6 py-4 truncate">
                          <span className="font-medium text-slate-800">{issue.title}</span>
                        </td>
                        <td className="px-6 py-4 text-slate-500 text-xs truncate">
                          {epicTitle}
                        </td>
                        <td className="px-6 py-4">
                          {issue.status === 'cancelled' ? (
                            <span className="inline-flex items-center px-2 py-0.5 rounded-full text-[10px] font-semibold bg-slate-100 text-slate-500">취소</span>
                          ) : (
                            <span className="inline-flex items-center px-2 py-0.5 rounded-full text-[10px] font-semibold bg-emerald-50 text-emerald-700">완료</span>
                          )}
                        </td>
                        <td className="px-6 py-4">
                          <PriorityBadge priority={issue.priority} />
                        </td>
                        <td className="px-6 py-4 text-slate-500 text-xs" title={issue.updated_at}>
                          {new Date(issue.updated_at).toLocaleDateString()}
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>

            {/* 더 보기 버튼 */}
            {sortedIssues.length > visibleCount && (
              <div className="flex justify-center mt-4">
                <button
                  onClick={() => setVisibleCount((prev) => prev + 30)}
                  className="px-6 py-2 bg-white border border-slate-200 hover:bg-slate-50 text-slate-600 hover:text-slate-800 text-xs font-semibold rounded-lg shadow-sm transition-all"
                >
                  더 보기 ({sortedIssues.length - visibleCount}개 남음)
                </button>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

