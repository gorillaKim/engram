import type { Mission, Epic, Issue, Sprint } from '../ipc/types';
import { EpicRow } from './EpicRow';
import { StatusBadge } from './StatusBadge';
import { CopyableId } from './CopyableId';
import { PromptButton } from './PromptButton';

interface GroupedMission {
  mission: Mission | null;
  epics: {
    epic: Epic;
    issues: Issue[];
  }[];
}

interface MissionHierarchyProps {
  groupedMissions: GroupedMission[];
  sprints: Sprint[];
  expandedMissions: Record<string, boolean>;
  onToggleMission: (key: string) => void;
  expandedEpics: Record<number, boolean>;
  onToggleEpic: (id: number) => void;
  onIssueClick: (id: number) => void;
  readOnly?: boolean;
  showEpicCheckboxes?: boolean;
  bulkSelectedEpics?: Set<number>;
  onEpicCheck?: (epicId: number, checked: boolean) => void;
  onEpicEdit?: (epic: Epic) => void;
  onMissionEdit?: (mission: Mission) => void;
  renderMissionActions?: (mission: Mission | null) => React.ReactNode;
  renderIssueExtra?: (issue: Issue) => React.ReactNode;
  onIssueStatusChange?: (issueId: number, status: string) => void;
  onIssuePriorityChange?: (issueId: number, priority: any) => void;
  onBulkCompleteIssues?: (epicId: number) => void;
}

export function MissionHierarchy({
  groupedMissions,
  sprints,
  expandedMissions,
  onToggleMission,
  expandedEpics,
  onToggleEpic,
  onIssueClick,
  readOnly = false,
  showEpicCheckboxes = false,
  bulkSelectedEpics = new Set(),
  onEpicCheck,
  onEpicEdit,
  onMissionEdit,
  renderMissionActions,
  renderIssueExtra,
  onIssueStatusChange,
  onIssuePriorityChange,
  onBulkCompleteIssues,
}: MissionHierarchyProps) {
  return (
    <div className="space-y-4">
      {groupedMissions.map((gm) => {
        const mKey = gm.mission ? `mission-${gm.mission.id}` : 'unclassified';
        const isMissionExpanded = expandedMissions[mKey] !== false;
        const missionIssuesCount = gm.epics.reduce((acc, curr) => acc + curr.issues.length, 0);

        return (
          <div
            key={mKey}
            className={`transition-all duration-200 ${
              readOnly 
                ? 'border border-slate-200/80 rounded-2xl bg-white shadow-sm' 
                : 'mb-6'
            }`}
          >
            {/* Mission Header */}
            {gm.mission ? (
              <div 
                onClick={() => onToggleMission(mKey)}
                className={`flex items-center justify-between border shadow-sm select-none hover:bg-slate-100/10 transition-all ${
                  readOnly 
                    ? 'bg-violet-50/40 border-b border-slate-100 rounded-t-2xl rounded-b-none cursor-pointer px-5 py-3.5'
                    : 'bg-violet-50/70 border-violet-100/80 rounded-xl px-4 py-2.5 mb-3 cursor-pointer'
                }`}
              >
                <div className="flex items-center gap-2.5 min-w-0 flex-1">
                  <span className="text-violet-400 text-xs w-4 flex-shrink-0 text-center font-bold">
                    {isMissionExpanded ? '▼' : '▶'}
                  </span>
                  <span className="text-[10px] font-bold text-violet-600 uppercase bg-violet-100 px-1.5 py-0.5 rounded tracking-wider flex-shrink-0">
                    Mission
                  </span>
                  <CopyableId type="mission" id={gm.mission.id} prefix="#" className="text-xs font-bold text-violet-500 flex-shrink-0" />
                  <PromptButton type="mission" id={gm.mission.id} title={gm.mission.title} size="xs" />
                  <h3 className="text-sm font-bold text-slate-800 truncate" title={gm.mission.title}>
                    {gm.mission.title}
                  </h3>
                  <StatusBadge status={gm.mission.status} type="mission" />
                  {readOnly && (
                    <span className="text-xs text-slate-400 bg-slate-100 px-2.5 py-0.5 rounded-full font-semibold flex-shrink-0">
                      완료 {missionIssuesCount}개
                    </span>
                  )}
                  {!readOnly && onMissionEdit && (
                    <button
                      type="button"
                      onClick={(e) => {
                        e.stopPropagation();
                        onMissionEdit(gm.mission!);
                      }}
                      title="미션 수정"
                      className="text-xs px-1.5 py-1 text-slate-400 hover:text-slate-700 transition-colors"
                    >
                      ✎
                    </button>
                  )}
                </div>

                {renderMissionActions && (
                  <div onClick={(e) => e.stopPropagation()} className="flex-shrink-0">
                    {renderMissionActions(gm.mission)}
                  </div>
                )}
              </div>
            ) : (
              <div 
                onClick={() => onToggleMission(mKey)}
                className={`flex items-center gap-2 border shadow-sm select-none transition-all ${
                  readOnly
                    ? 'bg-slate-50 border-b border-slate-200/60 rounded-t-2xl rounded-b-none cursor-pointer px-5 py-3.5'
                    : 'bg-slate-100/80 border-slate-200/60 rounded-xl px-4 py-2.5 mb-3 cursor-pointer'
                }`}
              >
                <span className="text-slate-400 text-xs w-4 flex-shrink-0 text-center font-bold">
                  {isMissionExpanded ? '▼' : '▶'}
                </span>
                <span className="text-[10px] font-bold text-slate-600 uppercase bg-slate-200 px-1.5 py-0.5 rounded tracking-wider flex-shrink-0">
                  System
                </span>
                <h3 className={`text-sm font-bold select-none truncate ${readOnly ? 'text-slate-600' : 'text-slate-600'}`}>
                  미분류 (지정 미션 없음)
                </h3>
                {readOnly && (
                  <span className="text-xs text-slate-400 bg-slate-100 px-2.5 py-0.5 rounded-full font-semibold flex-shrink-0">
                    완료 {missionIssuesCount}개
                  </span>
                )}
              </div>
            )}

            {/* Epics list under this Mission */}
            {isMissionExpanded && (
              <div 
                className={`flex flex-col gap-1.5 ${
                  readOnly 
                    ? 'p-4 space-y-3 bg-slate-50/20 border-t border-slate-100' 
                    : 'pl-2 border-l-2 border-slate-100/80'
                }`}
              >
                {gm.epics.map(({ epic, issues }) => (
                  <EpicRow
                    key={epic.id}
                    epic={epic}
                    issues={issues}
                    sprints={sprints}
                    onIssueClick={onIssueClick}
                    onEdit={onEpicEdit}
                    expanded={expandedEpics[epic.id] !== false}
                    onToggle={() => onToggleEpic(epic.id)}
                    readOnly={readOnly}
                    showCheckbox={showEpicCheckboxes}
                    checked={bulkSelectedEpics.has(epic.id)}
                    onCheck={onEpicCheck ? (c) => onEpicCheck(epic.id, c) : undefined}
                    renderIssueExtra={renderIssueExtra}
                    onIssueStatusChange={onIssueStatusChange}
                    onIssuePriorityChange={onIssuePriorityChange}
                    onBulkCompleteIssues={onBulkCompleteIssues}
                  />
                ))}
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}
