import React, { useState, useEffect } from 'react';
import { useQuery } from '@tanstack/react-query';
import type { Sprint, SprintProgress } from '../ipc/types';
import { sprintProgressList } from '../ipc/invoke';
import { StatusBadge } from './StatusBadge';
import { clampSidebarWidth } from '../utils/sidebarHelper';

const BACKLOG_ID = 0;

function BacklogItem({
  selected, onClick, count,
}: {
  selected: boolean;
  onClick: () => void;
  count?: number;
}) {
  return (
    <div
      onClick={onClick}
      className={`p-3 rounded-lg cursor-pointer mb-1 ${selected ? 'bg-indigo-50 border border-indigo-200' : 'hover:bg-slate-50'}`}
    >
      <div className="flex items-center justify-between gap-2">
        <span className="text-xs font-semibold px-2 py-0.5 rounded-full bg-slate-200 text-slate-600">백로그</span>
        {count != null && <span className="text-xs text-slate-400">{count}개</span>}
      </div>
      <p className="text-sm font-medium text-slate-800 mt-1">스프린트 미지정</p>
      <p className="text-xs text-slate-400 mt-0.5">아직 스프린트에 들어가지 않은 이슈</p>
    </div>
  );
}

function SprintItem({
  sprint, progress, selected, onClick, onActivate, onComplete, onDelete, onEdit,
}: {
  sprint: Sprint;
  progress?: SprintProgress;
  selected: boolean;
  onClick: () => void;
  onActivate: () => void;
  onComplete: () => void;
  onDelete: () => void;
  onEdit: () => void;
}) {
  const [confirmDelete, setConfirmDelete] = useState(false);

  useEffect(() => {
    if (!confirmDelete) return;
    const t = setTimeout(() => setConfirmDelete(false), 3000);
    return () => clearTimeout(t);
  }, [confirmDelete]);

  const completionRate = progress && progress.total_issues > 0
    ? Math.round((progress.completed_issues / progress.total_issues) * 100)
    : 0;

  return (
    <div
      onClick={onClick}
      className={`p-3 rounded-lg cursor-pointer mb-1 ${selected ? 'bg-indigo-50 border border-indigo-200' : 'hover:bg-slate-50'}`}
    >
      <div className="flex items-center justify-between gap-2">
        <StatusBadge status={sprint.status} type="sprint" variant="ko" />
        <div className="flex items-center gap-1">
          {sprint.status === 'planning' && (
            <button
              type="button"
              onClick={(e) => { e.stopPropagation(); onActivate(); }}
              className="text-xs px-2 py-0.5 bg-indigo-600 hover:bg-indigo-500 text-white rounded"
            >
              활성화
            </button>
          )}
          {sprint.status === 'active' && (
            <button
              type="button"
              onClick={(e) => { e.stopPropagation(); onComplete(); }}
              className="text-xs px-2 py-0.5 bg-green-600 hover:bg-green-500 text-white rounded"
            >
              완료
            </button>
          )}
          <button
            type="button"
            onClick={(e) => { e.stopPropagation(); onEdit(); }}
            title="스프린트 수정"
            className="text-xs px-1.5 py-0.5 text-slate-400 hover:text-slate-700"
          >
            ✎
          </button>
          {confirmDelete ? (
            <button
              type="button"
              onClick={(e) => { e.stopPropagation(); setConfirmDelete(false); onDelete(); }}
              className="text-xs px-2 py-0.5 bg-red-600 hover:bg-red-500 text-white rounded"
            >
              삭제 확인
            </button>
          ) : (
            <button
              type="button"
              onClick={(e) => { e.stopPropagation(); setConfirmDelete(true); }}
              title="스프린트 삭제"
              className="text-xs px-1.5 py-0.5 text-slate-400 hover:text-red-600"
            >
              ✕
            </button>
          )}
        </div>
      </div>
      <p className="text-sm font-medium text-slate-800 mt-1 truncate">{sprint.name}</p>
      {sprint.goal && <p className="text-xs text-slate-400 mt-0.5 truncate">{sprint.goal}</p>}
      {(sprint.start_date || sprint.end_date) && (
        <p className="text-xs text-slate-400 mt-0.5">
          {sprint.start_date ?? '?'} ~ {sprint.end_date ?? '?'}
        </p>
      )}

      {/* Progress Bar */}
      {progress && progress.total_issues > 0 && (
        <div className="mt-2.5 flex flex-col gap-1 animate-fade-in">
          <div className="flex justify-between items-center text-[10px] text-slate-400 font-semibold">
            <span>완료율 {completionRate}%</span>
            <span>{progress.completed_issues}/{progress.total_issues}개</span>
          </div>
          <div className="h-1 w-full bg-slate-200/80 rounded-full overflow-hidden">
            <div
              className="h-full bg-indigo-500 rounded-full transition-all duration-300"
              style={{ width: `${completionRate}%` }}
            />
          </div>
        </div>
      )}
    </div>
  );
}

interface SprintSidebarProps {
  sprints: Sprint[];
  backlogCount: number;
  selectedSprintId: number | null;
  onSelectSprint: (id: number | null) => void;
  onActivateSprint: (id: number) => void;
  onCompleteSprint: (sprint: Sprint) => void;
  onDeleteSprint: (id: number) => void;
  onEditSprint: (sprint: Sprint) => void;
  onAddSprint: () => void;
  children?: React.ReactNode;
}

export function SprintSidebar({
  sprints,
  backlogCount,
  selectedSprintId,
  onSelectSprint,
  onActivateSprint,
  onCompleteSprint,
  onDeleteSprint,
  onEditSprint,
  onAddSprint,
  children,
}: SprintSidebarProps) {
  const [sidebarWidth, setSidebarWidth] = useState<number>(() => {
    const saved = localStorage.getItem('engram_sidebar_width');
    return saved ? parseInt(saved, 10) : 224;
  });

  const [showPastSprints, setShowPastSprints] = useState(false);

  // 스프린트별 하위 이슈 진행률 로드
  const { data: progressList = [] } = useQuery<SprintProgress[]>({
    queryKey: ['sprintsProgress'],
    queryFn: sprintProgressList,
    refetchInterval: 10_000,
  });

  const handleMouseDown = (e: React.MouseEvent) => {
    e.preventDefault();
    const startX = e.clientX;
    const startWidth = sidebarWidth;

    const handleMouseMove = (moveEvent: MouseEvent) => {
      const deltaX = moveEvent.clientX - startX;
      setSidebarWidth(clampSidebarWidth(startWidth + deltaX));
    };

    const handleMouseUp = () => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
    };

    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
  };

  useEffect(() => {
    localStorage.setItem('engram_sidebar_width', sidebarWidth.toString());
  }, [sidebarWidth]);

  const isBacklog = selectedSprintId === BACKLOG_ID;
  const activeSprints = sprints.filter(s => s.status !== 'completed' && s.status !== 'cancelled');
  const pastSprints = sprints.filter(s => s.status === 'completed' || s.status === 'cancelled');

  return (
    <>
      <div
        className="flex-shrink-0 border-r border-slate-200 flex flex-col bg-slate-50"
        style={{ width: sidebarWidth }}
      >
        <div className="flex items-center justify-between px-4 py-3 border-b border-slate-200">
          <span className="text-xs font-semibold text-slate-500 uppercase tracking-wider">스프린트</span>
          <button
            type="button"
            onClick={onAddSprint}
            className="text-xs px-2 py-1 bg-slate-200 hover:bg-slate-300 text-slate-700 rounded flex items-center gap-1 transition-all hover:scale-105 active:scale-95"
          >
            + 추가
          </button>
        </div>

        <div className="flex-1 overflow-y-auto p-2">
          <BacklogItem
            selected={isBacklog}
            onClick={() => onSelectSprint(BACKLOG_ID)}
            count={backlogCount}
          />
          {isBacklog && children}

          {activeSprints.length === 0 && (
            <p className="text-xs text-slate-400 text-center mt-4">활성 스프린트가 없습니다</p>
          )}

          {activeSprints.map((sprint) => {
            const progress = progressList.find((p) => p.sprint_id === sprint.id);
            return (
              <div key={sprint.id}>
                <SprintItem
                  sprint={sprint}
                  progress={progress}
                  selected={sprint.id === selectedSprintId}
                  onClick={() => onSelectSprint(sprint.id)}
                  onActivate={() => onActivateSprint(sprint.id)}
                  onComplete={() => onCompleteSprint(sprint)}
                  onDelete={() => onDeleteSprint(sprint.id)}
                  onEdit={() => onEditSprint(sprint)}
                />
                {sprint.id === selectedSprintId && children}
              </div>
            );
          })}

          {/* 완료된 스프린트 아코디언 */}
          {pastSprints.length > 0 && (
            <div className="mt-4 border-t border-slate-200/60 pt-3">
              <button
                type="button"
                onClick={() => setShowPastSprints(!showPastSprints)}
                className="w-full px-3 py-1.5 flex items-center justify-between text-xs font-semibold text-slate-400 hover:text-slate-600 hover:bg-slate-100 rounded-md transition-colors"
              >
                <span>완료된 스프린트 ({pastSprints.length})</span>
                <span>{showPastSprints ? '▼' : '▶'}</span>
              </button>

              {showPastSprints && (
                <div className="mt-1 px-1">
                  {pastSprints.map((sprint) => {
                    const progress = progressList.find((p) => p.sprint_id === sprint.id);
                    return (
                      <div key={sprint.id}>
                        <SprintItem
                          sprint={sprint}
                          progress={progress}
                          selected={sprint.id === selectedSprintId}
                          onClick={() => onSelectSprint(sprint.id)}
                          onActivate={() => onActivateSprint(sprint.id)}
                          onComplete={() => onCompleteSprint(sprint)}
                          onDelete={() => onDeleteSprint(sprint.id)}
                          onEdit={() => onEditSprint(sprint)}
                        />
                        {sprint.id === selectedSprintId && children}
                      </div>
                    );
                  })}
                </div>
              )}
            </div>
          )}
        </div>
      </div>

      {/* Resize Handle */}
      <div
        onMouseDown={handleMouseDown}
        className="w-[3px] hover:w-[6px] hover:bg-indigo-300 active:bg-indigo-500 cursor-col-resize flex-shrink-0 transition-all duration-150 z-30"
        style={{ cursor: 'col-resize' }}
      />
    </>
  );
}
