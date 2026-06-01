import { useState, useMemo } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useIssues } from '../hooks/useIssues';
import { useSprints } from '../hooks/useSprints';
import { useEpics } from '../hooks/useEpics';
import { useDebounce } from '../hooks/useDebounce';
import { useUIStore } from '../store/ui';
import { missionList, missionUpdate } from '../ipc/invoke';
import type { Mission } from '../ipc/types';
import { groupIssuesByMissionAndEpic } from '../utils/historyHelper';
import { toast } from 'sonner';
import { MissionHierarchy } from '../components/MissionHierarchy';

export function History() {
  const { selectIssue } = useUIStore();
  const qc = useQueryClient();

  // 필터 상태
  const [dateFrom, setDateFrom] = useState<string>('');
  const [dateTo, setDateTo] = useState<string>('');
  const [searchQuery, setSearchQuery] = useState<string>('');
  const debouncedQuery = useDebounce(searchQuery);

  // Collapse 상태 제어 (key: missionId 혹은 'unclassified')
  const [expandedMissions, setExpandedMissions] = useState<Record<string, boolean>>({});
  // Collapse 상태 제어 (key: epicId)
  const [expandedEpics, setExpandedEpics] = useState<Record<number, boolean>>({});

  // 스프린트, 에픽, 미션 데이터 로드
  const { data: sprints = [] } = useSprints();
  const { data: epics = [] } = useEpics(undefined, true);
  const { data: missions = [] } = useQuery<Mission[]>({
    queryKey: ['missionList', 'all'],
    queryFn: () => missionList(true), // 완료된 미션도 히스토리를 위해 전체 로드
  });

  // 모든 이슈 로드 (필터 없이 로드한 후 프론트에서 완료/취소 상태 필터링)
  const { data: allIssues = [], isLoading } = useIssues({});

  // 미션 활성화 복구 뮤테이션
  const restoreMissionMutation = useMutation({
    mutationFn: (id: number) => missionUpdate(id, { status: 'active' }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['missionList'] });
      qc.invalidateQueries({ queryKey: ['epicList'] });
      qc.invalidateQueries({ queryKey: ['issueList'] });
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      toast.success('미션이 활성화 상태로 복구되었습니다.');
    },
    onError: (e) => toast.error(`미션 복구 실패: ${e}`),
  });

  const toggleMission = (key: string) => {
    setExpandedMissions((prev) => ({ ...prev, [key]: !prev[key] }));
  };

  const toggleEpic = (id: number) => {
    setExpandedEpics((prev) => ({ ...prev, [id]: !prev[id] }));
  };

  // 1. 필터링된 완료/취소 이슈 목록 계산
  const filteredIssues = useMemo(() => {
    return allIssues.filter((issue) => {
      // 완료 또는 취소 상태만 히스토리에 포함
      if (issue.status !== 'finished' && issue.status !== 'cancelled') {
        return false;
      }

      // 날짜 필터링
      if (issue.updated_at) {
        const issueDate = new Date(issue.updated_at).toISOString().split('T')[0];
        if (dateFrom && issueDate < dateFrom) return false;
        if (dateTo && issueDate > dateTo) return false;
      } else if (dateFrom || dateTo) {
        return false;
      }

      // 검색어 필터링
      const q = debouncedQuery.trim().toLowerCase();
      if (q) {
        if (q.startsWith('#')) {
          const idNum = parseInt(q.slice(1), 10);
          return issue.id === idNum;
        }
        return issue.title.toLowerCase().includes(q);
      }

      return true;
    });
  }, [allIssues, dateFrom, dateTo, debouncedQuery]);

  // 2. 미션 > 에픽 > 이슈 계층 구조 생성
  const groupedMissions = useMemo(() => {
    return groupIssuesByMissionAndEpic(filteredIssues, epics, missions);
  }, [filteredIssues, epics, missions]);

  // 완료 이슈 총 개수
  const totalCount = filteredIssues.length;

  return (
    <div className="flex flex-col h-full bg-slate-50/30 overflow-hidden">
      {/* 헤더 필터 영역 */}
      <div className="flex-shrink-0 px-6 py-4 border-b border-slate-200 bg-white flex flex-col gap-4 shadow-sm z-10">
        <div className="flex items-center justify-between">
          <h1 className="text-xl font-bold text-slate-800">완료 히스토리</h1>
          <span className="text-sm font-semibold text-slate-500 bg-slate-100 px-3 py-1 rounded-full">
            총 {totalCount}건의 해결된 이슈
          </span>
        </div>

        <div className="flex items-center gap-4 flex-wrap">
          {/* 날짜 범위 필터 */}
          <div className="flex items-center gap-2">
            <span className="text-xs font-bold text-slate-400 uppercase tracking-wider">완료 기간</span>
            <input
              type="date"
              value={dateFrom}
              onChange={(e) => setDateFrom(e.target.value)}
              className="text-xs border border-slate-200 rounded-lg px-2.5 py-1.5 bg-white shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/20 text-slate-700"
            />
            <span className="text-slate-300">~</span>
            <input
              type="date"
              value={dateTo}
              onChange={(e) => setDateTo(e.target.value)}
              className="text-xs border border-slate-200 rounded-lg px-2.5 py-1.5 bg-white shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/20 text-slate-700"
            />
            {(dateFrom || dateTo) && (
              <button
                type="button"
                onClick={() => { setDateFrom(''); setDateTo(''); }}
                className="text-[11px] text-red-500 hover:text-red-700 hover:underline font-semibold"
              >
                날짜 초기화
              </button>
            )}
          </div>

          {/* 검색어 필터 */}
          <div className="flex-1 min-w-[200px]">
            <input
              type="text"
              placeholder="#ID 또는 이슈 제목 검색…"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full text-xs border border-slate-200 rounded-lg px-3 py-1.5 bg-white shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/20 text-slate-700 font-medium"
            />
          </div>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-6 space-y-4">
        {isLoading ? (
          <div className="flex items-center justify-center h-40 text-slate-400 text-sm">
            히스토리 데이터를 불러오는 중…
          </div>
        ) : totalCount === 0 ? (
          <div className="flex flex-col items-center justify-center h-40 bg-white rounded-2xl border-2 border-dashed border-slate-200 p-12 text-slate-400">
            <span className="text-4xl mb-3">📦</span>
            <p className="text-sm font-medium">조건에 맞는 완료된 이슈가 없습니다.</p>
          </div>
        ) : (
          <MissionHierarchy
            groupedMissions={groupedMissions}
            sprints={sprints}
            expandedMissions={expandedMissions}
            onToggleMission={toggleMission}
            expandedEpics={expandedEpics}
            onToggleEpic={toggleEpic}
            onIssueClick={selectIssue}
            readOnly={true}
            renderMissionActions={(mission) => {
              if (!mission) return null;
              return (
                <div className="flex items-center gap-3">
                  <span className={`text-[10px] font-bold px-2 py-0.5 rounded border ${
                    mission.status === 'completed'
                      ? 'bg-emerald-50 text-emerald-700 border-emerald-200'
                      : 'bg-red-50 text-red-600 border-red-200'
                  }`}>
                    {mission.status}
                  </span>
                  {(mission.status === 'completed' || mission.status === 'cancelled') && (
                    <button
                      type="button"
                      onClick={() => restoreMissionMutation.mutate(mission.id)}
                      disabled={restoreMissionMutation.isPending}
                      className="text-xs px-2.5 py-1 bg-indigo-600 hover:bg-indigo-500 text-white rounded font-medium shadow-sm transition-colors"
                    >
                      {restoreMissionMutation.isPending ? '복구 중…' : 'Active로 복구'}
                    </button>
                  )}
                </div>
              );
            }}
            renderIssueExtra={(issue) => (
              <span className="text-slate-400 text-[11px] w-24 flex-shrink-0 text-right font-medium" title={issue.updated_at}>
                {issue.updated_at ? new Date(issue.updated_at).toLocaleDateString() : ''}
              </span>
            )}
          />
        )}
      </div>
    </div>
  );
}
