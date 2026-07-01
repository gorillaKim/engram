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
import { parseUTCDate } from '../utils/date';
import { toast } from 'sonner';
import { MissionHierarchy } from '../components/MissionHierarchy';

export function History() {
  const { selectIssue } = useUIStore();
  const qc = useQueryClient();

  // 필터 상태 (디폴트 최근 30일)
  const [dateFrom, setDateFrom] = useState<string>(() => {
    const d = new Date();
    d.setDate(d.getDate() - 30);
    return d.toISOString().split('T')[0];
  });
  const [dateTo, setDateTo] = useState<string>(() => new Date().toISOString().split('T')[0]);
  const [searchQuery, setSearchQuery] = useState<string>('');
  const debouncedQuery = useDebounce(searchQuery);
  const [viewMode, setViewMode] = useState<'hierarchy' | 'timeline'>('timeline');

  // 추가 메타 필터 상태
  const [filterProject, setFilterProject] = useState<string>('all');
  const [filterPriority, setFilterPriority] = useState<string>('all');
  const [filterAgent, setFilterAgent] = useState<string>('all');
  const [filterStatus, setFilterStatus] = useState<string>('all');

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

  // 고유 프로젝트 목록 추출
  const uniqueProjects = useMemo(() => {
    const keys = new Set<string>();
    for (const epic of epics) {
      if (epic.project_key) keys.add(epic.project_key);
    }
    return Array.from(keys);
  }, [epics]);

  // 고유 에이전트 목록 추출
  const uniqueAgents = useMemo(() => {
    const agents = new Set<string>();
    for (const issue of allIssues) {
      if (issue.assigned_agent) agents.add(issue.assigned_agent);
    }
    return Array.from(agents);
  }, [allIssues]);

  // 1. 필터링된 완료/취소 이슈 목록 계산
  const filteredIssues = useMemo(() => {
    return allIssues.filter((issue) => {
      // 완료 또는 취소 상태만 히스토리에 포함
      if (issue.status !== 'finished' && issue.status !== 'cancelled') {
        return false;
      }

      // 1-1. 해결 구분 상태 필터
      if (filterStatus !== 'all' && issue.status !== filterStatus) {
        return false;
      }

      // 1-2. 프로젝트 필터 (이슈의 epic_id에 해당하는 에픽의 project_key 매칭)
      if (filterProject !== 'all') {
        const parentEpic = epics.find((e) => e.id === issue.epic_id);
        if (!parentEpic || parentEpic.project_key !== filterProject) {
          return false;
        }
      }

      // 1-3. 우선순위 필터
      if (filterPriority !== 'all' && issue.priority !== filterPriority) {
        return false;
      }

      // 1-4. 에이전트 필터
      if (filterAgent !== 'all' && issue.assigned_agent !== filterAgent) {
        return false;
      }

      // 1-5. 날짜 필터링
      if (issue.updated_at) {
        const issueDate = parseUTCDate(issue.updated_at).toISOString().split('T')[0];
        if (dateFrom && issueDate < dateFrom) return false;
        if (dateTo && issueDate > dateTo) return false;
      } else if (dateFrom || dateTo) {
        return false;
      }

      // 1-6. 검색어 필터링
      const q = debouncedQuery.trim().toLowerCase();
      if (q) {
        if (q.startsWith('#')) {
          const idNum = parseInt(q.slice(1), 10);
          return issue.id === idNum;
        }
        // 검색 범위를 제목, 설명, 목표, 에이전트명까지 넓게 매칭
        const matchTitle = issue.title.toLowerCase().includes(q);
        const matchDesc = issue.description?.toLowerCase().includes(q) || false;
        const matchGoal = issue.goal?.toLowerCase().includes(q) || false;
        const matchAgent = issue.assigned_agent?.toLowerCase().includes(q) || false;
        return matchTitle || matchDesc || matchGoal || matchAgent;
      }

      return true;
    });
  }, [allIssues, epics, dateFrom, dateTo, debouncedQuery, filterProject, filterPriority, filterAgent, filterStatus]);

  // 2. 미션 > 에픽 > 이슈 계층 구조 생성
  const groupedMissions = useMemo(() => {
    return groupIssuesByMissionAndEpic(filteredIssues, epics, missions);
  }, [filteredIssues, epics, missions]);

  // 3. 날짜별 그룹핑 (타임라인용)
  const timelineGroups = useMemo(() => {
    if (viewMode !== 'timeline') return [];

    const groups: Record<string, typeof filteredIssues> = {};
    for (const issue of filteredIssues) {
      const dateStr = issue.updated_at
        ? parseUTCDate(issue.updated_at).toLocaleDateString('ko-KR', {
            year: 'numeric',
            month: 'long',
            day: 'numeric',
            weekday: 'short',
          })
        : '날짜 정보 없음';
      if (!groups[dateStr]) {
        groups[dateStr] = [];
      }
      groups[dateStr].push(issue);
    }

    return Object.entries(groups).sort((a, b) => {
      if (a[0] === '날짜 정보 없음') return 1;
      if (b[0] === '날짜 정보 없음') return -1;
      const aTime = a[1][0]?.updated_at || '';
      const bTime = b[1][0]?.updated_at || '';
      return bTime.localeCompare(aTime);
    });
  }, [filteredIssues, viewMode]);

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

        <div className="flex flex-col gap-3">
          {/* 첫 번째 줄: 검색어 및 메타 필터들 */}
          <div className="flex items-center gap-3 flex-wrap">
            {/* 검색어 필터 */}
            <div className="flex-1 min-w-[240px]">
              <input
                type="text"
                placeholder="#ID, 제목, 설명, 목표, 에이전트 검색…"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="w-full text-xs border border-slate-200 rounded-lg px-3 py-1.5 bg-white shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/20 text-slate-700 font-medium"
              />
            </div>

            {/* 해결 구분 */}
            <div className="flex items-center gap-1.5">
              <span className="text-[10px] font-bold text-slate-400 uppercase tracking-wider pl-0.5">상태</span>
              <select
                value={filterStatus}
                onChange={(e) => setFilterStatus(e.target.value)}
                className="text-xs border border-slate-200 rounded-lg h-9 py-0 pl-2 pr-6 bg-white shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/20 text-slate-700 font-semibold"
              >
                <option value="all">전체 상태</option>
                <option value="finished">완료 (Finished)</option>
                <option value="cancelled">취소 (Cancelled)</option>
              </select>
            </div>

            {/* 프로젝트 */}
            <div className="flex items-center gap-1.5">
              <span className="text-[10px] font-bold text-slate-400 uppercase tracking-wider pl-0.5">프로젝트</span>
              <select
                value={filterProject}
                onChange={(e) => setFilterProject(e.target.value)}
                className="text-xs border border-slate-200 rounded-lg h-9 py-0 pl-2 pr-6 bg-white shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/20 text-slate-700 font-semibold max-w-[140px]"
              >
                <option value="all">전체 프로젝트</option>
                {uniqueProjects.map((p) => (
                  <option key={p} value={p}>{p}</option>
                ))}
              </select>
            </div>

            {/* 담당 에이전트 */}
            <div className="flex items-center gap-1.5">
              <span className="text-[10px] font-bold text-slate-400 uppercase tracking-wider pl-0.5">에이전트</span>
              <select
                value={filterAgent}
                onChange={(e) => setFilterAgent(e.target.value)}
                className="text-xs border border-slate-200 rounded-lg h-9 py-0 pl-2 pr-6 bg-white shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/20 text-slate-700 font-semibold max-w-[140px]"
              >
                <option value="all">전체 에이전트</option>
                {uniqueAgents.map((a) => (
                  <option key={a} value={a}>{a}</option>
                ))}
              </select>
            </div>

            {/* 우선순위 */}
            <div className="flex items-center gap-1.5">
              <span className="text-[10px] font-bold text-slate-400 uppercase tracking-wider pl-0.5">우선순위</span>
              <select
                value={filterPriority}
                onChange={(e) => setFilterPriority(e.target.value)}
                className="text-xs border border-slate-200 rounded-lg h-9 py-0 pl-2 pr-6 bg-white shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/20 text-slate-700 font-semibold"
              >
                <option value="all">전체 우선순위</option>
                <option value="high">High</option>
                <option value="medium">Medium</option>
                <option value="low">Low</option>
                <option value="none">None</option>
              </select>
            </div>
          </div>

          {/* 두 번째 줄: 날짜 필터 및 퀵 버튼 */}
          <div className="flex items-center justify-between gap-4 flex-wrap pt-2 border-t border-slate-100">
            <div className="flex items-center gap-2">
              <span className="text-xs font-bold text-slate-400 uppercase tracking-wider">완료 기간</span>
              <input
                type="date"
                value={dateFrom}
                onChange={(e) => setDateFrom(e.target.value)}
                className="text-xs border border-slate-200 rounded-lg px-2 h-8 bg-white shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/20 text-slate-700"
              />
              <span className="text-slate-300">~</span>
              <input
                type="date"
                value={dateTo}
                onChange={(e) => setDateTo(e.target.value)}
                className="text-xs border border-slate-200 rounded-lg px-2 h-8 bg-white shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/20 text-slate-700"
              />
              
              {/* 퀵 뱃지 버튼 */}
              <div className="flex items-center gap-1 ml-2">
                <button
                  type="button"
                  onClick={() => {
                    const d = new Date();
                    d.setDate(d.getDate() - 30);
                    setDateFrom(d.toISOString().split('T')[0]);
                    setDateTo(new Date().toISOString().split('T')[0]);
                  }}
                  className="text-[10px] px-2.5 py-1 rounded bg-indigo-50 text-indigo-600 hover:bg-indigo-100 font-semibold transition-colors"
                >
                  최근 30일
                </button>
                <button
                  type="button"
                  onClick={() => {
                    const d = new Date();
                    d.setDate(d.getDate() - 90);
                    setDateFrom(d.toISOString().split('T')[0]);
                    setDateTo(new Date().toISOString().split('T')[0]);
                  }}
                  className="text-[10px] px-2.5 py-1 rounded bg-slate-100 text-slate-600 hover:bg-slate-200 font-semibold transition-colors"
                >
                  최근 90일
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setDateFrom('');
                    setDateTo('');
                  }}
                  className="text-[10px] px-2.5 py-1 rounded bg-slate-100 text-slate-600 hover:bg-slate-200 font-semibold transition-colors"
                >
                  전체 기간
                </button>
              </div>
            </div>

            {/* 필터 전체 초기화 */}
            {(dateFrom || dateTo || filterProject !== 'all' || filterPriority !== 'all' || filterAgent !== 'all' || filterStatus !== 'all' || searchQuery) && (
              <button
                type="button"
                onClick={() => {
                  const d = new Date();
                  d.setDate(d.getDate() - 30);
                  setDateFrom(d.toISOString().split('T')[0]);
                  setDateTo(new Date().toISOString().split('T')[0]);
                  setSearchQuery('');
                  setFilterProject('all');
                  setFilterPriority('all');
                  setFilterAgent('all');
                  setFilterStatus('all');
                }}
                className="text-[11px] text-red-500 hover:text-red-700 hover:underline font-semibold flex items-center gap-1"
              >
                ✕ 필터 전체 초기화
              </button>
            )}
          </div>
        </div>

        {/* 뷰 모드 탭 스위치 */}
        <div className="flex border-b border-slate-100 mt-1">
          <button
            onClick={() => setViewMode('timeline')}
            className={`px-4 py-2 text-xs font-semibold border-b-2 transition-all ${
              viewMode === 'timeline'
                ? 'border-indigo-600 text-indigo-600 font-bold'
                : 'border-transparent text-slate-500 hover:text-slate-700'
            }`}
          >
            타임라인 뷰
          </button>
          <button
            onClick={() => setViewMode('hierarchy')}
            className={`px-4 py-2 text-xs font-semibold border-b-2 transition-all ${
              viewMode === 'hierarchy'
                ? 'border-indigo-600 text-indigo-600 font-bold'
                : 'border-transparent text-slate-500 hover:text-slate-700'
            }`}
          >
            계층형 뷰 (Mission &gt; Epic)
          </button>
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
        ) : viewMode === 'hierarchy' ? (
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
                {issue.updated_at ? parseUTCDate(issue.updated_at).toLocaleDateString() : ''}
              </span>
            )}
          />
        ) : (
          <div className="space-y-6">
            {timelineGroups.map(([dateStr, issues]) => (
              <div key={dateStr} className="space-y-2.5">
                <h3 className="text-xs font-bold text-slate-400 uppercase tracking-wider pl-1">
                  {dateStr}
                </h3>
                <div className="grid gap-2.5">
                  {issues.map((issue) => {
                    const issueEpic = epics.find((e) => e.id === issue.epic_id);
                    const issueMission = issue.mission_id ? missions.find((m) => m.id === issue.mission_id) : undefined;
                    return (
                      <div
                        key={issue.id}
                        onClick={() => selectIssue(issue.id)}
                        className="bg-white border border-slate-200 rounded-xl p-4 shadow-sm hover:shadow-md hover:border-indigo-300 transition-all cursor-pointer flex items-center justify-between gap-4 group"
                      >
                        <div className="flex flex-col gap-1 min-w-0">
                          <div className="flex items-center gap-1.5 flex-wrap">
                            <span className="text-xs font-semibold text-slate-400">#{issue.id}</span>
                            <span className="text-[10px] font-semibold text-slate-500 font-mono bg-slate-100 px-1.5 py-0.5 rounded">
                              {issueEpic ? issueEpic.project_key : 'PROJ'}
                            </span>
                            <span className={`text-[10px] font-bold px-1.5 py-0.2 rounded border uppercase ${
                              issue.status === 'finished'
                                ? 'bg-emerald-50 text-emerald-700 border-emerald-150'
                                : 'bg-red-50 text-red-600 border-red-150'
                            }`}>
                              {issue.status === 'finished' ? '완료' : '취소'}
                            </span>
                            {issueMission && (
                              <span className="text-[10px] font-medium text-violet-600 bg-violet-50 border border-violet-100/50 rounded px-1.5 py-0.2 truncate max-w-[120px]" title={`미션: ${issueMission.title}`}>
                                {issueMission.title}
                              </span>
                            )}
                            {issueEpic && (
                              <span className="text-[10px] font-medium text-sky-600 bg-sky-50 border border-sky-100/50 rounded px-1.5 py-0.2 truncate max-w-[120px]" title={`에픽: ${issueEpic.title}`}>
                                {issueEpic.title}
                              </span>
                            )}
                          </div>
                          <h4 className="text-sm font-semibold text-slate-800 group-hover:text-indigo-600 transition-colors truncate leading-snug">
                            {issue.title}
                          </h4>
                        </div>
                        <div className="text-slate-400 text-xs flex-shrink-0 font-mono">
                          {issue.updated_at ? parseUTCDate(issue.updated_at).toLocaleTimeString('ko-KR', { hour: '2-digit', minute: '2-digit' }) : ''}
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
