import React from 'react';
import type { Mission, Epic } from '../ipc/types';

interface IssueFilterPanelProps {
  filteredMissions: Mission[];
  availableEpics: Epic[];
  selectedMissionIds: number[];
  setSelectedMissionIds: React.Dispatch<React.SetStateAction<number[]>>;
  selectedEpicIds: number[];
  setSelectedEpicIds: React.Dispatch<React.SetStateAction<number[]>>;
}

export function IssueFilterPanel({
  filteredMissions,
  availableEpics,
  selectedMissionIds,
  setSelectedMissionIds,
  selectedEpicIds,
  setSelectedEpicIds,
}: IssueFilterPanelProps) {
  return (
    <div className="mt-1 px-2 py-2 bg-slate-100 rounded-lg border border-slate-200 flex flex-col gap-2">
      {(filteredMissions.length > 0 || selectedMissionIds.length > 0) && (
        <div>
          <p className="text-[10px] font-bold text-slate-400 uppercase tracking-wider mb-1">미션</p>
          <div className="flex flex-wrap gap-1">
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
      {availableEpics.length > 0 && (
        <div>
          <p className="text-[10px] font-bold text-slate-400 uppercase tracking-wider mb-1">에픽</p>
          <div className="flex flex-wrap gap-1">
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
    </div>
  );
}
