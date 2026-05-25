import { useState } from 'react';
import type { Sprint } from '../ipc/types';

interface BulkActionBarProps {
  selectedCount: number;
  sprints: Sprint[];
  onClear: () => void;
  onUpdateSprint: (sprintId: number | null) => void;
  onUpdateStatus: (status: 'active' | 'completed' | 'cancelled') => void;
  isPending: boolean;
}

export function BulkActionBar({
  selectedCount,
  sprints,
  onClear,
  onUpdateSprint,
  onUpdateStatus,
  isPending,
}: BulkActionBarProps) {
  const [sprintTarget, setSprintTarget] = useState<string>('none');
  const [statusTarget, setStatusTarget] = useState<string>('none');

  // 활성 및 계획 스프린트만 노출
  const activeAndPlanningSprints = sprints.filter(
    (s) => s.status === 'active' || s.status === 'planning'
  );

  const handleSprintSubmit = () => {
    if (sprintTarget === 'none') return;
    if (sprintTarget === 'backlog') {
      onUpdateSprint(null);
    } else {
      onUpdateSprint(parseInt(sprintTarget, 10));
    }
    setSprintTarget('none');
  };

  const handleStatusSubmit = () => {
    if (statusTarget === 'none') return;
    onUpdateStatus(statusTarget as 'active' | 'completed' | 'cancelled');
    setStatusTarget('none');
  };

  return (
    <div className="fixed bottom-6 left-1/2 transform -translate-x-1/2 bg-slate-900/90 text-white backdrop-blur-md px-6 py-4 rounded-2xl shadow-2xl border border-slate-800 z-50 flex flex-wrap items-center gap-5 animate-in slide-in-from-bottom-5 duration-200 max-w-[90%] sm:max-w-none">
      <div className="flex items-center gap-2">
        <span className="w-2.5 h-2.5 bg-indigo-500 rounded-full animate-pulse" />
        <span className="text-xs font-bold text-slate-200">
          에픽 <strong className="text-indigo-400 font-extrabold text-sm">{selectedCount}</strong>개 선택됨
        </span>
      </div>

      <div className="h-4 w-px bg-slate-800 hidden sm:block" />

      {/* 스프린트 변경 */}
      <div className="flex items-center gap-1.5">
        <select
          value={sprintTarget}
          onChange={(e) => setSprintTarget(e.target.value)}
          disabled={isPending}
          className="text-xs bg-slate-800 border border-slate-700 rounded-lg px-2 py-1.5 text-slate-200 focus:outline-none focus:ring-1 focus:ring-indigo-500 max-w-[140px] truncate"
        >
          <option value="none">스프린트 일괄 변경…</option>
          <option value="backlog">백로그 (지정 안함)</option>
          {activeAndPlanningSprints.map((s) => (
            <option key={s.id} value={s.id}>
              {s.name}
            </option>
          ))}
        </select>
        <button
          type="button"
          onClick={handleSprintSubmit}
          disabled={isPending || sprintTarget === 'none'}
          className="text-xs px-2.5 py-1.5 bg-indigo-600 hover:bg-indigo-500 disabled:opacity-40 text-white rounded-lg font-semibold transition-colors whitespace-nowrap"
        >
          적용
        </button>
      </div>

      {/* 상태 변경 */}
      <div className="flex items-center gap-1.5">
        <select
          value={statusTarget}
          onChange={(e) => setStatusTarget(e.target.value)}
          disabled={isPending}
          className="text-xs bg-slate-800 border border-slate-700 rounded-lg px-2 py-1.5 text-slate-200 focus:outline-none focus:ring-1 focus:ring-indigo-500"
        >
          <option value="none">상태 일괄 변경…</option>
          <option value="active">Active</option>
          <option value="completed">Completed</option>
          <option value="cancelled">Cancelled</option>
        </select>
        <button
          type="button"
          onClick={handleStatusSubmit}
          disabled={isPending || statusTarget === 'none'}
          className="text-xs px-2.5 py-1.5 bg-indigo-600 hover:bg-indigo-500 disabled:opacity-40 text-white rounded-lg font-semibold transition-colors whitespace-nowrap"
        >
          적용
        </button>
      </div>

      <div className="h-4 w-px bg-slate-800" />

      <div className="flex items-center gap-2">
        <button
          type="button"
          onClick={onClear}
          disabled={isPending}
          className="text-xs px-2 py-1.5 text-slate-400 hover:text-slate-200 font-semibold transition-colors whitespace-nowrap"
        >
          선택 해제
        </button>
      </div>
    </div>
  );
}
