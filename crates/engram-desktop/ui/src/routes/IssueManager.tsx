import { useState, useEffect } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { sprintList, sprintUpdate, epicList, epicSetSprint, issueList } from '../ipc/invoke';
import { useUIStore } from '../store/ui';
import { CreateSprintModal } from '../components/CreateSprintModal';
import { CreateEpicModal } from '../components/CreateEpicModal';
import { CreateIssueModal } from '../components/CreateIssueModal';
import { PriorityBadge } from '../components/PriorityBadge';
import type { Sprint, Epic, Issue, SprintStatus } from '../ipc/types';

// ── Sprint sidebar ──────────────────────────────────────────────────────────

const STATUS_LABEL: Record<SprintStatus, string> = {
  planning: '계획',
  active: '활성',
  completed: '완료',
};

const STATUS_CLS: Record<SprintStatus, string> = {
  planning: 'bg-yellow-100 text-yellow-700',
  active: 'bg-green-100 text-green-700',
  completed: 'bg-slate-100 text-slate-500',
};

function SprintItem({
  sprint, selected, onClick, onActivate,
}: {
  sprint: Sprint;
  selected: boolean;
  onClick: () => void;
  onActivate: () => void;
}) {
  return (
    <div
      onClick={onClick}
      className={`p-3 rounded-lg cursor-pointer mb-1 ${selected ? 'bg-indigo-50 border border-indigo-200' : 'hover:bg-slate-50'}`}
    >
      <div className="flex items-center justify-between gap-2">
        <span className={`text-xs font-semibold px-2 py-0.5 rounded-full ${STATUS_CLS[sprint.status]}`}>
          {STATUS_LABEL[sprint.status]}
        </span>
        {sprint.status === 'planning' && (
          <button
            type="button"
            onClick={(e) => { e.stopPropagation(); onActivate(); }}
            className="text-xs px-2 py-0.5 bg-indigo-600 hover:bg-indigo-500 text-white rounded"
          >
            활성화
          </button>
        )}
      </div>
      <p className="text-sm font-medium text-slate-800 mt-1 truncate">{sprint.name}</p>
      {sprint.goal && <p className="text-xs text-slate-400 mt-0.5 truncate">{sprint.goal}</p>}
      {(sprint.start_date || sprint.end_date) && (
        <p className="text-xs text-slate-400 mt-0.5">
          {sprint.start_date ?? '?'} ~ {sprint.end_date ?? '?'}
        </p>
      )}
    </div>
  );
}

// ── Issue status badge ──────────────────────────────────────────────────────

const ISSUE_STATUS_CLS: Record<string, string> = {
  required: 'bg-slate-100 text-slate-500',
  ready:    'bg-blue-100 text-blue-600',
  working:  'bg-indigo-100 text-indigo-600',
  demo:     'bg-amber-100 text-amber-700',
  finished: 'bg-green-100 text-green-700',
  cancelled:'bg-red-50 text-red-400',
};

const ISSUE_STATUS_LABEL: Record<string, string> = {
  required: '필요',
  ready:    '준비',
  working:  '진행',
  demo:     '검토',
  finished: '완료',
  cancelled:'취소',
};

// ── Epic row ────────────────────────────────────────────────────────────────

function EpicRow({
  epic, sprints, onIssueClick, onAddIssue,
}: {
  epic: Epic;
  sprints: Sprint[];
  onIssueClick: (id: number) => void;
  onAddIssue: () => void;
}) {
  const qc = useQueryClient();
  const [expanded, setExpanded] = useState(true);

  const { data: issues = [] } = useQuery<Issue[]>({
    queryKey: ['issueList', { epic_id: epic.id }],
    queryFn: () => issueList({ epic_id: epic.id }),
    staleTime: 10_000,
  });

  const moveSprint = useMutation({
    mutationFn: (sprint_id: number) => epicSetSprint(epic.id, sprint_id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['epicList'] });
      toast.success('에픽 스프린트가 변경되었습니다');
    },
    onError: (e) => toast.error(`변경 실패: ${e}`),
  });

  const epicStatusCls: Record<string, string> = {
    active: 'bg-blue-100 text-blue-600',
    completed: 'bg-green-100 text-green-700',
    cancelled: 'bg-red-50 text-red-400',
  };

  return (
    <div className="mb-3 border border-slate-200 rounded-lg overflow-hidden">
      {/* Epic header */}
      <div className="flex items-center gap-2 px-3 py-2 bg-slate-50 border-b border-slate-200">
        <button
          type="button"
          onClick={() => setExpanded((v) => !v)}
          className="text-slate-400 hover:text-slate-600 text-xs w-4"
        >
          {expanded ? '▼' : '▶'}
        </button>

        <span className={`text-xs px-2 py-0.5 rounded-full font-medium ${epicStatusCls[epic.status] ?? ''}`}>
          {epic.status}
        </span>

        <span className="text-sm font-semibold text-slate-800 flex-1 min-w-0 truncate">
          {epic.title}
        </span>

        <span className="text-xs text-slate-400">{issues.length}개 이슈</span>

        {/* Sprint change select */}
        <select
          value={epic.sprint_id}
          onChange={(e) => {
            const newId = Number(e.target.value);
            if (newId !== epic.sprint_id) moveSprint.mutate(newId);
          }}
          onClick={(e) => e.stopPropagation()}
          className="text-xs px-2 py-1 bg-white border border-slate-200 rounded text-slate-600"
        >
          {sprints.map((s) => (
            <option key={s.id} value={s.id}>{s.name}</option>
          ))}
        </select>

        <button
          type="button"
          onClick={onAddIssue}
          className="text-xs px-2 py-1 bg-white border border-slate-200 hover:bg-slate-100 text-slate-600 rounded"
        >
          + 이슈
        </button>
      </div>

      {/* Issue list */}
      {expanded && (
        <div>
          {issues.length === 0 ? (
            <p className="text-xs text-slate-400 py-3 px-4">이슈가 없습니다</p>
          ) : (
            issues.map((issue) => (
              <div
                key={issue.id}
                onClick={() => onIssueClick(issue.id)}
                className="flex items-center gap-3 px-4 py-2 hover:bg-slate-50 cursor-pointer border-b border-slate-100 last:border-b-0"
              >
                <span className={`text-xs px-2 py-0.5 rounded-full ${ISSUE_STATUS_CLS[issue.status] ?? ''}`}>
                  {ISSUE_STATUS_LABEL[issue.status] ?? issue.status}
                </span>
                <PriorityBadge priority={issue.priority} />
                <span className="text-sm text-slate-700 flex-1 truncate">{issue.title}</span>
                <span className="text-xs text-slate-400">#{issue.id}</span>
              </div>
            ))
          )}
        </div>
      )}
    </div>
  );
}

// ── Main ────────────────────────────────────────────────────────────────────

export function IssueManager() {
  const { selectedSprintId, selectSprint, selectIssue } = useUIStore();
  const qc = useQueryClient();

  const [sprintModalOpen, setSprintModalOpen] = useState(false);
  const [epicModalOpen, setEpicModalOpen] = useState(false);
  const [issueModalEpicId, setIssueModalEpicId] = useState<number | null>(null);

  const { data: sprints = [] } = useQuery<Sprint[]>({
    queryKey: ['sprintList'],
    queryFn: sprintList,
    refetchInterval: 30_000,
  });

  // Auto-select active sprint on first load
  useEffect(() => {
    if (sprints.length === 0) return;
    if (selectedSprintId != null && sprints.some((s) => s.id === selectedSprintId)) return;
    const active = sprints.find((s) => s.status === 'active') ?? sprints[0];
    selectSprint(active.id);
  }, [sprints, selectedSprintId, selectSprint]);

  const { data: epics = [] } = useQuery<Epic[]>({
    queryKey: ['epicList', selectedSprintId],
    queryFn: () => epicList(selectedSprintId!),
    enabled: selectedSprintId != null,
  });

  const activateSprint = useMutation({
    mutationFn: (id: number) => sprintUpdate(id, 'active'),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['sprintList'] });
      qc.invalidateQueries({ queryKey: ['sprintCurrent'] });
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      toast.success('스프린트가 활성화되었습니다');
    },
    onError: (e) => toast.error(`활성화 실패: ${e}`),
  });

  const selectedSprint = sprints.find((s) => s.id === selectedSprintId);

  return (
    <div className="flex h-full overflow-hidden">
      {/* ── Sprint sidebar ── */}
      <div className="w-56 flex-shrink-0 border-r border-slate-200 flex flex-col bg-slate-50">
        <div className="flex items-center justify-between px-4 py-3 border-b border-slate-200">
          <span className="text-xs font-semibold text-slate-500 uppercase tracking-wider">스프린트</span>
          <button
            type="button"
            onClick={() => setSprintModalOpen(true)}
            className="text-xs px-2 py-1 bg-slate-200 hover:bg-slate-300 text-slate-700 rounded"
          >
            + 추가
          </button>
        </div>

        <div className="flex-1 overflow-y-auto p-2">
          {sprints.length === 0 && (
            <p className="text-xs text-slate-400 text-center mt-4">스프린트가 없습니다</p>
          )}
          {sprints.map((sprint) => (
            <SprintItem
              key={sprint.id}
              sprint={sprint}
              selected={sprint.id === selectedSprintId}
              onClick={() => selectSprint(sprint.id)}
              onActivate={() => {
                if (confirm(`"${sprint.name}"을 활성화하시겠습니까?\n기존 활성 스프린트가 있으면 대체됩니다.`)) {
                  activateSprint.mutate(sprint.id);
                }
              }}
            />
          ))}
        </div>
      </div>

      {/* ── Main content ── */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-3 border-b border-slate-200 flex-shrink-0">
          <div>
            <h2 className="text-base font-semibold text-slate-800">
              {selectedSprint ? selectedSprint.name : '스프린트를 선택하세요'}
            </h2>
            {selectedSprint?.goal && (
              <p className="text-xs text-slate-400 mt-0.5">{selectedSprint.goal}</p>
            )}
          </div>
          {selectedSprintId != null && (
            <button
              type="button"
              onClick={() => setEpicModalOpen(true)}
              className="text-sm px-3 py-1.5 bg-indigo-600 hover:bg-indigo-500 text-white rounded-md"
            >
              + 새 에픽
            </button>
          )}
        </div>

        {/* Epic + Issue tree */}
        <div className="flex-1 overflow-y-auto p-6">
          {selectedSprintId == null && (
            <p className="text-slate-400 text-center mt-20">왼쪽에서 스프린트를 선택하세요</p>
          )}
          {selectedSprintId != null && epics.length === 0 && (
            <p className="text-slate-400 text-center mt-20">
              에픽이 없습니다. "+ 새 에픽" 버튼으로 추가하세요.
            </p>
          )}
          {epics.map((epic) => (
            <EpicRow
              key={epic.id}
              epic={epic}
              sprints={sprints}
              onIssueClick={selectIssue}
              onAddIssue={() => setIssueModalEpicId(epic.id)}
            />
          ))}
        </div>
      </div>

      {/* Modals */}
      <CreateSprintModal
        open={sprintModalOpen}
        onClose={() => setSprintModalOpen(false)}
      />
      <CreateEpicModal
        open={epicModalOpen}
        onClose={() => setEpicModalOpen(false)}
        sprintId={selectedSprintId ?? undefined}
      />
      <CreateIssueModal
        open={issueModalEpicId != null}
        onClose={() => setIssueModalEpicId(null)}
        defaultEpicId={issueModalEpicId ?? undefined}
      />
    </div>
  );
}
