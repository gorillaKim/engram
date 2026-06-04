import React from 'react';
import type { Mission, Epic } from '../ipc/types';

interface IssueFilterPanelProps {
  filteredMissions: Mission[];
  availableEpics: Epic[];
  selectedMissionIds: number[];
  setSelectedMissionIds: React.Dispatch<React.SetStateAction<number[]>>;
  selectedEpicIds: number[];
  setSelectedEpicIds: React.Dispatch<React.SetStateAction<number[]>>;
  
  // 신규 필터 props
  selectedStatuses: string[];
  setSelectedStatuses: React.Dispatch<React.SetStateAction<string[]>>;
  selectedPriorities: string[];
  setSelectedPriorities: React.Dispatch<React.SetStateAction<string[]>>;
  selectedAgents: string[];
  setSelectedAgents: React.Dispatch<React.SetStateAction<string[]>>;
  availableAgents: string[];
  onClose?: () => void;
}

const STATUS_LABELS: Record<string, string> = {
  required: 'Required',
  ready: 'Ready',
  working: 'Working',
  demo: 'Demo',
  finished: 'Finished',
  cancelled: 'Cancelled',
};

const PRIORITY_LABELS: Record<string, string> = {
  critical: 'Critical',
  high: 'High',
  medium: 'Medium',
  low: 'Low',
};

export function IssueFilterPanel({
  filteredMissions,
  availableEpics,
  selectedMissionIds,
  setSelectedMissionIds,
  selectedEpicIds,
  setSelectedEpicIds,
  selectedStatuses,
  setSelectedStatuses,
  selectedPriorities,
  setSelectedPriorities,
  selectedAgents,
  setSelectedAgents,
  availableAgents,
  onClose,
}: IssueFilterPanelProps) {
  const hasAnyFilter =
    selectedMissionIds.length > 0 ||
    selectedEpicIds.length > 0 ||
    selectedStatuses.length > 0 ||
    selectedPriorities.length > 0 ||
    selectedAgents.length > 0;

  const handleReset = () => {
    setSelectedMissionIds([]);
    setSelectedEpicIds([]);
    setSelectedStatuses([]);
    setSelectedPriorities([]);
    setSelectedAgents([]);
  };

  return (
    <div className="bg-slate-50 border-b border-slate-200 px-6 py-4 flex flex-col md:flex-row md:items-start justify-between gap-4 select-none flex-shrink-0 shadow-sm">
      <div className="flex flex-wrap gap-x-8 gap-y-4 flex-1">
        {/* 미션 필터 */}
        {(filteredMissions.length > 0 || selectedMissionIds.length > 0) && (
          <div className="flex flex-col gap-1.5 max-w-[280px]">
            <p className="text-[10px] font-bold text-slate-400 uppercase tracking-wider">미션</p>
            <div className="flex flex-wrap gap-1 max-h-[110px] overflow-y-auto pr-1">
              <button
                type="button"
                onClick={() => setSelectedMissionIds([])}
                className={`text-[11px] px-2 py-0.5 rounded-full border transition-all hover:scale-105 active:scale-95 ${
                  selectedMissionIds.length === 0
                    ? 'bg-violet-100 text-violet-700 border-violet-300 font-medium'
                    : 'bg-white text-slate-500 border-slate-200 hover:border-violet-200'
                }`}
              >
                전체
              </button>
              <button
                type="button"
                onClick={() =>
                  setSelectedMissionIds((prev) =>
                    prev.includes(0) ? prev.filter((id) => id !== 0) : [...prev, 0]
                  )
                }
                className={`text-[11px] px-2 py-0.5 rounded-full border transition-all hover:scale-105 active:scale-95 ${
                  selectedMissionIds.includes(0)
                    ? 'bg-violet-100 text-violet-700 border-violet-300 font-medium'
                    : 'bg-white text-slate-500 border-slate-200 hover:border-violet-200'
                }`}
              >
                미분류
              </button>
              {filteredMissions.map((m) => (
                <button
                  key={m.id}
                  type="button"
                  title={m.title}
                  onClick={() =>
                    setSelectedMissionIds((prev) =>
                      prev.includes(m.id) ? prev.filter((id) => id !== m.id) : [...prev, m.id]
                    )
                  }
                  className={`text-[11px] px-2 py-0.5 rounded-full border transition-all hover:scale-105 active:scale-95 max-w-[120px] truncate ${
                    selectedMissionIds.includes(m.id)
                      ? 'bg-violet-100 text-violet-700 border-violet-300 font-medium'
                      : 'bg-white text-slate-500 border-slate-200 hover:border-violet-200'
                  }`}
                >
                  {m.title}
                </button>
              ))}
            </div>
          </div>
        )}

        {/* 에픽 필터 */}
        {(availableEpics.length > 0 || selectedEpicIds.length > 0) && (
          <div className="flex flex-col gap-1.5 max-w-[320px]">
            <p className="text-[10px] font-bold text-slate-400 uppercase tracking-wider">에픽</p>
            <div className="flex flex-wrap gap-1 max-h-[110px] overflow-y-auto pr-1">
              {availableEpics.map((epic) => (
                <button
                  key={epic.id}
                  type="button"
                  title={epic.title}
                  onClick={() =>
                    setSelectedEpicIds((prev) =>
                      prev.includes(epic.id) ? prev.filter((id) => id !== epic.id) : [...prev, epic.id]
                    )
                  }
                  className={`text-[11px] px-2 py-0.5 rounded-full border transition-all hover:scale-105 active:scale-95 max-w-[130px] truncate ${
                    selectedEpicIds.includes(epic.id)
                      ? 'bg-indigo-100 text-indigo-700 border-indigo-300 font-medium'
                      : 'bg-white text-slate-500 border-slate-200 hover:border-indigo-200'
                  }`}
                >
                  {epic.title}
                </button>
              ))}
            </div>
          </div>
        )}

        {/* 상태 필터 */}
        <div className="flex flex-col gap-1.5 min-w-[140px]">
          <p className="text-[10px] font-bold text-slate-400 uppercase tracking-wider">상태</p>
          <div className="flex flex-wrap gap-1">
            {Object.entries(STATUS_LABELS).map(([status, label]) => {
              const isSelected = selectedStatuses.includes(status);
              return (
                <button
                  key={status}
                  type="button"
                  onClick={() =>
                    setSelectedStatuses((prev) =>
                      prev.includes(status) ? prev.filter((s) => s !== status) : [...prev, status]
                    )
                  }
                  className={`text-[11px] px-2 py-0.5 rounded-full border transition-all hover:scale-105 active:scale-95 ${
                    isSelected
                      ? 'bg-amber-100 text-amber-700 border-amber-300 font-medium'
                      : 'bg-white text-slate-500 border-slate-200 hover:border-amber-200'
                  }`}
                >
                  {label}
                </button>
              );
            })}
          </div>
        </div>

        {/* 우선순위 필터 */}
        <div className="flex flex-col gap-1.5 min-w-[140px]">
          <p className="text-[10px] font-bold text-slate-400 uppercase tracking-wider">우선순위</p>
          <div className="flex flex-wrap gap-1">
            {Object.entries(PRIORITY_LABELS).map(([prio, label]) => {
              const isSelected = selectedPriorities.includes(prio);
              return (
                <button
                  key={prio}
                  type="button"
                  onClick={() =>
                    setSelectedPriorities((prev) =>
                      prev.includes(prio) ? prev.filter((p) => p !== prio) : [...prev, prio]
                    )
                  }
                  className={`text-[11px] px-2 py-0.5 rounded-full border transition-all hover:scale-105 active:scale-95 ${
                    isSelected
                      ? 'bg-rose-100 text-rose-700 border-rose-300 font-medium'
                      : 'bg-white text-slate-500 border-slate-200 hover:border-rose-200'
                  }`}
                >
                  {label}
                </button>
              );
            })}
          </div>
        </div>

        {/* 담당자 필터 */}
        {availableAgents.length > 0 && (
          <div className="flex flex-col gap-1.5 min-w-[140px]">
            <p className="text-[10px] font-bold text-slate-400 uppercase tracking-wider">담당자</p>
            <div className="flex flex-wrap gap-1 max-h-[110px] overflow-y-auto pr-1">
              {availableAgents.map((agent) => {
                const isSelected = selectedAgents.includes(agent);
                const displayName = agent === 'unassigned' ? '미할당' : agent;
                return (
                  <button
                    key={agent}
                    type="button"
                    onClick={() =>
                      setSelectedAgents((prev) =>
                        prev.includes(agent) ? prev.filter((a) => a !== agent) : [...prev, agent]
                      )
                    }
                    className={`text-[11px] px-2 py-0.5 rounded-full border transition-all hover:scale-105 active:scale-95 ${
                      isSelected
                        ? 'bg-emerald-100 text-emerald-700 border-emerald-300 font-medium'
                        : 'bg-white text-slate-500 border-slate-200 hover:border-emerald-200'
                    }`}
                  >
                    {displayName === 'unassigned' ? '미할당' : `🤖 ${displayName}`}
                  </button>
                );
              })}
            </div>
          </div>
        )}
      </div>

      {/* 필터 제어 버튼 영역 (초기화 및 접기) */}
      <div className="flex flex-col items-end gap-2 shrink-0 md:self-stretch justify-between">
        {onClose && (
          <button
            type="button"
            onClick={onClose}
            className="text-xs px-2 py-1 rounded-md text-slate-400 hover:text-slate-600 hover:bg-slate-200/50 transition-all font-semibold flex items-center gap-0.5 active:scale-95"
            title="필터 접기"
          >
            <span>접기</span>
            <span className="text-[9px]">▲</span>
          </button>
        )}
        
        {hasAnyFilter && (
          <button
            type="button"
            onClick={handleReset}
            className="text-xs px-2.5 py-1.5 rounded-lg border border-slate-200 bg-white hover:bg-slate-100 text-slate-500 hover:text-slate-800 transition-all font-semibold flex items-center gap-1 shadow-sm active:scale-95 mt-auto"
            title="모든 필터 초기화"
          >
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={2.5} stroke="currentColor" className="w-3.5 h-3.5 text-slate-400">
              <path strokeLinecap="round" strokeLinejoin="round" d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0 3.181 3.183a8.25 8.25 0 0 0 13.803-3.7M4.031 9.865a8.25 8.25 0 0 1 13.803-3.7l3.181 3.182m0-4.991v4.99" />
            </svg>
            <span>필터 초기화</span>
          </button>
        )}
      </div>
    </div>
  );
}
