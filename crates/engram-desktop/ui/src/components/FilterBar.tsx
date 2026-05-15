import type { IssuePriority, IssueProjectBoard } from '../ipc/types';
import type { BoardFilters } from '../store/ui';

interface Props {
  boards: IssueProjectBoard[];
  filters: BoardFilters;
  hideFinished: boolean;
  onToggleHideFinished: () => void;
  onChange: (f: Partial<BoardFilters>) => void;
  onReset: () => void;
}

const PRIORITIES: IssuePriority[] = ['critical', 'high', 'medium', 'low'];
const PRIORITY_LABEL: Record<IssuePriority, string> = {
  critical: '긴급', high: '높음', medium: '중간', low: '낮음',
};
const PRIORITY_COLOR: Record<IssuePriority, string> = {
  critical: 'bg-red-100 text-red-700 border-red-300',
  high: 'bg-orange-100 text-orange-700 border-orange-300',
  medium: 'bg-yellow-100 text-yellow-700 border-yellow-300',
  low: 'bg-slate-100 text-slate-600 border-slate-300',
};

function toggle<T>(arr: T[], val: T): T[] {
  return arr.includes(val) ? arr.filter((x) => x !== val) : [...arr, val];
}

export function FilterBar({ boards, filters, hideFinished, onToggleHideFinished, onChange, onReset }: Props) {
  const allProjects = boards.map((b) => b.project_key);
  const hasActiveFilters = filters.projects.length > 0 || filters.priorities.length > 0;

  return (
    <div className="flex flex-wrap items-center gap-3 text-sm">
      {/* Hide finished toggle */}
      <label className="flex items-center gap-1.5 text-slate-600 cursor-pointer select-none">
        <input
          type="checkbox"
          checked={hideFinished}
          onChange={onToggleHideFinished}
          className="rounded border-slate-300 text-indigo-600"
        />
        완료 숨기기
      </label>

      {/* Project filter (only shown when multiple projects) */}
      {allProjects.length > 1 && (
        <div className="flex items-center gap-1.5">
          <span className="text-slate-400 text-xs">프로젝트</span>
          {allProjects.map((key) => (
            <button
              key={key}
              onClick={() => onChange({ projects: toggle(filters.projects, key) })}
              className={`px-2 py-0.5 rounded border text-xs transition-colors ${
                filters.projects.includes(key)
                  ? 'bg-indigo-100 text-indigo-700 border-indigo-300'
                  : 'bg-white text-slate-600 border-slate-200 hover:border-indigo-300'
              }`}
            >
              {key}
            </button>
          ))}
        </div>
      )}

      {/* Priority filter */}
      <div className="flex items-center gap-1.5">
        <span className="text-slate-400 text-xs">우선순위</span>
        {PRIORITIES.map((p) => (
          <button
            key={p}
            onClick={() => onChange({ priorities: toggle(filters.priorities, p) })}
            className={`px-2 py-0.5 rounded border text-xs transition-colors ${
              filters.priorities.includes(p)
                ? PRIORITY_COLOR[p]
                : 'bg-white text-slate-400 border-slate-200 hover:border-slate-300'
            }`}
          >
            {PRIORITY_LABEL[p]}
          </button>
        ))}
      </div>

      {/* Reset */}
      {hasActiveFilters && (
        <button
          onClick={onReset}
          className="text-xs text-indigo-600 hover:text-indigo-800 underline"
        >
          필터 초기화
        </button>
      )}
    </div>
  );
}
