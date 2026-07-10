import { useState, useRef, useEffect, useMemo } from 'react';
import type { IssuePriority, IssueProjectBoard, Mission, Epic } from '../ipc/types';
import type { BoardFilters } from '../store/ui';

interface Props {
  boards: IssueProjectBoard[];
  filters: BoardFilters;
  hideFinished: boolean;
  onToggleHideFinished: () => void;
  showCancelled: boolean;
  onToggleShowCancelled: () => void;
  onChange: (f: Partial<BoardFilters>) => void;
  onReset: () => void;
  missions?: Mission[];
  epics?: Epic[];
}

const PRIORITIES: IssuePriority[] = ['critical', 'high', 'medium', 'low'];
const PRIORITY_LABEL: Record<IssuePriority, string> = {
  critical: '긴급', high: '높음', medium: '중간', low: '낮음',
};
const PRIORITY_ACTIVE: Record<IssuePriority, string> = {
  critical: 'bg-red-100 text-red-700 border-red-300',
  high: 'bg-orange-100 text-orange-700 border-orange-300',
  medium: 'bg-yellow-100 text-yellow-700 border-yellow-300',
  low: 'bg-slate-100 text-slate-600 border-slate-300',
};

function toggle<T>(arr: T[], val: T): T[] {
  return arr.includes(val) ? arr.filter((x) => x !== val) : [...arr, val];
}

interface DropdownProps {
  label: string;
  count: number;
  activeColor: string;
  children: React.ReactNode;
}

function FilterDropdown({ label, count, activeColor, children }: DropdownProps) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, [open]);

  const isActive = count > 0;

  return (
    <div className="relative" ref={ref}>
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        className={`flex items-center gap-1.5 text-xs px-2.5 py-1 rounded-full border transition-colors ${
          isActive
            ? `${activeColor} font-medium`
            : 'bg-white text-slate-600 border-slate-200 hover:border-slate-300 hover:bg-slate-50'
        }`}
      >
        {label}
        {isActive && (
          <span className="text-[10px] bg-white/60 rounded-full px-1 font-bold min-w-[16px] text-center">
            {count}
          </span>
        )}
        <span className="text-[9px] opacity-60">▾</span>
      </button>

      {open && (
        <div className="absolute top-full left-0 mt-1.5 bg-white border border-slate-200 rounded-xl shadow-lg z-50 min-w-[180px] max-h-[260px] overflow-y-auto py-1.5">
          {children}
        </div>
      )}
    </div>
  );
}

function DropdownItem({
  label, checked, onChange,
}: { label: string; checked: boolean; onChange: () => void }) {
  return (
    <label className="flex items-center gap-2.5 px-3 py-1.5 hover:bg-slate-50 cursor-pointer select-none">
      <input
        type="checkbox"
        checked={checked}
        onChange={onChange}
        className="rounded border-slate-300 text-indigo-600 w-3 h-3"
      />
      <span className="text-xs text-slate-700 truncate">{label}</span>
    </label>
  );
}

export function FilterBar({
  boards, filters, hideFinished, onToggleHideFinished,
  showCancelled, onToggleShowCancelled, onChange, onReset,
  missions = [], epics = [],
}: Props) {
  const allProjects = boards.map((b) => b.project_key);

  // 1. 현재 보드(스프린트)에 존재하는 모든 이슈 수집
  const allIssuesInBoard = useMemo(() => {
    const list: any[] = [];
    const columns = ['required', 'ready', 'working', 'demo', 'finished', 'cancelled'];
    for (const board of boards) {
      for (const col of columns) {
        const issues = (board as any)[col] ?? [];
        list.push(...issues);
      }
    }
    return list;
  }, [boards]);

  // 2. 이슈들과 연결된 활성 epic_id와 mission_id 수집
  const activeEpicIds = useMemo(() => {
    return new Set(allIssuesInBoard.map((i: any) => i.epic_id));
  }, [allIssuesInBoard]);

  const activeMissionIds = useMemo(() => {
    const set = new Set<number>();
    for (const issue of allIssuesInBoard) {
      if (issue.mission_id != null) {
        set.add(issue.mission_id);
      }
    }
    return set;
  }, [allIssuesInBoard]);

  // 3. 미션 및 에픽 목록을 활성 ID로 필터링 (현재 스프린트에 속한 항목만 남김)
  const sprintMissions = useMemo(() => {
    return missions.filter((m: Mission) => activeMissionIds.has(m.id));
  }, [missions, activeMissionIds]);

  const sprintEpics = useMemo(() => {
    return epics.filter((e: Epic) => activeEpicIds.has(e.id));
  }, [epics, activeEpicIds]);

  // 4. 미션 필터가 활성화된 경우 해당 미션 소속 에픽만 표시
  const visibleEpics = filters.missionIds.length > 0
    ? sprintEpics.filter((e: Epic) => e.mission_id != null && filters.missionIds.includes(e.mission_id))
    : sprintEpics;

  const hasActiveFilters =
    filters.projects.length > 0 ||
    filters.priorities.length > 0 ||
    filters.missionIds.length > 0 ||
    filters.epicIds.length > 0;

  return (
    <div className="flex items-center gap-2 flex-wrap text-sm">
      {/* 완료/취소 토글 */}
      <label className="flex items-center gap-1.5 text-xs text-slate-600 cursor-pointer select-none">
        <input
          type="checkbox"
          checked={hideFinished}
          onChange={onToggleHideFinished}
          className="rounded border-slate-300 text-indigo-600"
        />
        완료 숨기기
      </label>
      <label className="flex items-center gap-1.5 text-xs text-slate-600 cursor-pointer select-none">
        <input
          type="checkbox"
          checked={showCancelled}
          onChange={onToggleShowCancelled}
          className="rounded border-slate-300 text-indigo-600"
        />
        취소 보기
      </label>

      <div className="h-4 w-px bg-slate-200" />

      {/* 프로젝트 드롭다운 */}
      {allProjects.length > 0 && (
        <FilterDropdown
          label="프로젝트"
          count={filters.projects.length}
          activeColor="bg-indigo-100 text-indigo-700 border-indigo-300"
        >
          {allProjects.map((key) => (
            <DropdownItem
              key={key}
              label={key}
              checked={filters.projects.includes(key)}
              onChange={() => onChange({ projects: toggle(filters.projects, key) })}
            />
          ))}
        </FilterDropdown>
      )}

      {/* 미션 드롭다운 */}
      {sprintMissions.length > 0 && (
        <FilterDropdown
          label="미션"
          count={filters.missionIds.length}
          activeColor="bg-violet-100 text-violet-700 border-violet-300"
        >
          {sprintMissions.map((m: Mission) => (
            <DropdownItem
              key={m.id}
              label={m.title}
              checked={filters.missionIds.includes(m.id)}
              onChange={() => onChange({ missionIds: toggle(filters.missionIds, m.id), epicIds: [] })}
            />
          ))}
        </FilterDropdown>
      )}

      {/* 에픽 드롭다운 */}
      {visibleEpics.length > 0 && (
        <FilterDropdown
          label="에픽"
          count={filters.epicIds.length}
          activeColor="bg-sky-100 text-sky-700 border-sky-300"
        >
          {visibleEpics.map((e: Epic) => (
            <DropdownItem
              key={e.id}
              label={e.title}
              checked={filters.epicIds.includes(e.id)}
              onChange={() => onChange({ epicIds: toggle(filters.epicIds, e.id) })}
            />
          ))}
        </FilterDropdown>
      )}

      <div className="h-4 w-px bg-slate-200" />

      {/* 우선순위 칩 */}
      {PRIORITIES.map((p) => (
        <button
          key={p}
          type="button"
          onClick={() => onChange({ priorities: toggle(filters.priorities, p) })}
          className={`px-2 py-0.5 rounded-full border text-xs transition-colors ${
            filters.priorities.includes(p)
              ? PRIORITY_ACTIVE[p]
              : 'bg-white text-slate-400 border-slate-200 hover:border-slate-300'
          }`}
        >
          {PRIORITY_LABEL[p]}
        </button>
      ))}

      {/* 초기화 */}
      {hasActiveFilters && (
        <button
          type="button"
          onClick={onReset}
          className="text-xs text-slate-400 hover:text-red-500 transition-colors"
        >
          ✕ 초기화
        </button>
      )}
    </div>
  );
}
