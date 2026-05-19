import { useState, useEffect, useMemo } from 'react';
import { useDebounce } from '../hooks/useDebounce';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import {
  sprintList, sprintUpdate, sprintDelete,
  epicList, epicDelete,
  issueList, issueSetSprint,
} from '../ipc/invoke';
import { useUIStore } from '../store/ui';
import { CreateSprintModal } from '../components/CreateSprintModal';
import { CreateEpicModal } from '../components/CreateEpicModal';
import { CreateIssueModal } from '../components/CreateIssueModal';
import { EditEpicModal } from '../components/EditEpicModal';
import { EditSprintModal } from '../components/EditSprintModal';
import { PriorityBadge } from '../components/PriorityBadge';
import type { Sprint, Epic, Issue, SprintStatus } from '../ipc/types';

// ── Sprint sidebar ──────────────────────────────────────────────────────────

/** 사이드바에서 "백로그(스프린트 미지정)" 를 선택했을 때 selectedSprintId 로 사용하는 sentinel */
const BACKLOG_ID = 0;

const STATUS_LABEL: Record<SprintStatus, string> = {
  planning: '계획',
  active: '활성',
  completed: '완료',
  cancelled: '취소',
};

const STATUS_CLS: Record<SprintStatus, string> = {
  planning: 'bg-yellow-100 text-yellow-700',
  active: 'bg-green-100 text-green-700',
  completed: 'bg-slate-100 text-slate-500',
  cancelled: 'bg-red-50 text-red-400',
};

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
  sprint, selected, onClick, onActivate, onComplete, onDelete, onEdit,
}: {
  sprint: Sprint;
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

  return (
    <div
      onClick={onClick}
      className={`p-3 rounded-lg cursor-pointer mb-1 ${selected ? 'bg-indigo-50 border border-indigo-200' : 'hover:bg-slate-50'}`}
    >
      <div className="flex items-center justify-between gap-2">
        <span className={`text-xs font-semibold px-2 py-0.5 rounded-full ${STATUS_CLS[sprint.status]}`}>
          {STATUS_LABEL[sprint.status]}
        </span>
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

// ── Epic row (groups issues belonging to one epic, in current sprint or backlog) ─

function EpicRow({
  epic, issues, sprints, onIssueClick, onAddIssue, onEdit,
}: {
  epic: Epic;
  issues: Issue[];
  sprints: Sprint[];
  onIssueClick: (id: number) => void;
  onAddIssue: () => void;
  onEdit: (epic: Epic) => void;
}) {
  const qc = useQueryClient();
  const [expanded, setExpanded] = useState(true);
  const [confirmDeleteEpic, setConfirmDeleteEpic] = useState(false);

  useEffect(() => {
    if (!confirmDeleteEpic) return;
    const t = setTimeout(() => setConfirmDeleteEpic(false), 3000);
    return () => clearTimeout(t);
  }, [confirmDeleteEpic]);

  const moveIssueSprint = useMutation({
    mutationFn: ({ issueId, sprintId }: { issueId: number; sprintId: number | null }) =>
      issueSetSprint(issueId, sprintId),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['issueList'] });
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      toast.success('이슈 스프린트가 변경되었습니다');
    },
    onError: (e) => toast.error(`변경 실패: ${e}`),
  });

  const deleteEpic = useMutation({
    mutationFn: () => epicDelete(epic.id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['epicList'] });
      qc.invalidateQueries({ queryKey: ['issueList'] });
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      toast.success('에픽이 삭제되었습니다');
    },
    onError: (e) => toast.error(`에픽 삭제 실패: ${e}`),
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
          [{epic.project_key}] {epic.title}
        </span>

        <span className="text-xs text-slate-400">{issues.length}개 이슈</span>

        <button
          type="button"
          onClick={onAddIssue}
          className="text-xs px-2 py-1 bg-white border border-slate-200 hover:bg-slate-100 text-slate-600 rounded"
        >
          + 이슈
        </button>

        <button
          type="button"
          onClick={() => onEdit(epic)}
          title="에픽 수정"
          className="text-xs px-1.5 py-1 text-slate-400 hover:text-slate-700"
        >
          ✎
        </button>

        {confirmDeleteEpic ? (
          <button
            type="button"
            onClick={() => { setConfirmDeleteEpic(false); deleteEpic.mutate(); }}
            className="text-xs px-2 py-0.5 bg-red-600 hover:bg-red-500 text-white rounded"
          >
            삭제 확인
          </button>
        ) : (
          <button
            type="button"
            onClick={() => setConfirmDeleteEpic(true)}
            title="에픽 삭제"
            className="text-xs px-1.5 py-1 text-slate-400 hover:text-red-600"
          >
            ✕
          </button>
        )}
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

                {/* Sprint move dropdown (per-issue) */}
                <select
                  value={issue.sprint_id ?? ''}
                  onClick={(e) => e.stopPropagation()}
                  onChange={(e) => {
                    const v = e.target.value;
                    const newId = v === '' ? null : Number(v);
                    if (newId !== issue.sprint_id) {
                      moveIssueSprint.mutate({ issueId: issue.id, sprintId: newId });
                    }
                  }}
                  className="text-xs px-2 py-0.5 bg-white border border-slate-200 rounded text-slate-600"
                >
                  <option value="">백로그</option>
                  {sprints.map((s) => (
                    <option key={s.id} value={s.id}>{s.name}</option>
                  ))}
                </select>

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
  const [editEpic, setEditEpic] = useState<Epic | null>(null);
  const [editSprint, setEditSprint] = useState<Sprint | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const debouncedQuery = useDebounce(searchQuery);

  const { data: sprints = [] } = useQuery<Sprint[]>({
    queryKey: ['sprintList'],
    queryFn: sprintList,
    refetchInterval: 30_000,
  });

  const isBacklog = selectedSprintId === BACKLOG_ID;

  useEffect(() => {
    if (sprints.length === 0) return;
    if (selectedSprintId === BACKLOG_ID) return;
    if (selectedSprintId != null && sprints.some((s) => s.id === selectedSprintId)) return;
    const active = sprints.find((s) => s.status === 'active') ?? sprints[0];
    selectSprint(active.id);
  }, [sprints, selectedSprintId, selectSprint]);

  // 현재 뷰의 이슈 — 스프린트 선택 시 sprint_id 필터, 백로그 선택 시 backlog_only.
  const { data: issuesInView = [] } = useQuery<Issue[]>({
    queryKey: ['issueList', isBacklog ? 'backlog' : selectedSprintId],
    queryFn: () => isBacklog
      ? issueList({ backlog_only: true } as any)
      : issueList({ sprint_id: selectedSprintId } as any),
    enabled: selectedSprintId != null,
  });

  // 백로그 카운트 (사이드바 표시용) — 항상 별도 쿼리.
  const { data: backlogIssues = [] } = useQuery<Issue[]>({
    queryKey: ['issueList', 'backlog'],
    queryFn: () => issueList({ backlog_only: true } as any),
    refetchInterval: 30_000,
  });

  // 모든 에픽 (제목/설명 조회용 lookup).
  const { data: allEpics = [] } = useQuery<Epic[]>({
    queryKey: ['epicList'],
    queryFn: () => epicList(),
    refetchInterval: 30_000,
  });

  // 이슈를 epic_id 별로 그룹핑.
  const grouped = useMemo(() => {
    const byEpic = new Map<number, Issue[]>();
    for (const issue of issuesInView) {
      const arr = byEpic.get(issue.epic_id) ?? [];
      arr.push(issue);
      byEpic.set(issue.epic_id, arr);
    }
    const result: { epic: Epic; issues: Issue[] }[] = [];
    for (const [epicId, issues] of byEpic) {
      const epic = allEpics.find((e) => e.id === epicId);
      if (epic) result.push({ epic, issues });
    }
    return result;
  }, [issuesInView, allEpics]);

  // 스프린트 전환 시 검색어 초기화
  useEffect(() => { setSearchQuery(''); }, [selectedSprintId]);

  const filteredGrouped = useMemo(() => {
    const q = debouncedQuery.trim().toLowerCase();
    if (!q) return grouped;
    const isIdSearch = q.startsWith('#');
    const targetId = isIdSearch ? parseInt(q.slice(1)) : NaN;
    return grouped
      .map(({ epic, issues }) => ({
        epic,
        issues: issues.filter((i) =>
          isIdSearch ? i.id === targetId : i.title.toLowerCase().includes(q)
        ),
      }))
      .filter(({ issues }) => issues.length > 0);
  }, [grouped, debouncedQuery]);

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

  const completeSprint = useMutation({
    mutationFn: (id: number) => sprintUpdate(id, 'completed'),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['sprintList'] });
      qc.invalidateQueries({ queryKey: ['sprintCurrent'] });
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      toast.success('스프린트가 완료되었습니다');
    },
    onError: (e) => toast.error(`완료 처리 실패: ${e}`),
  });

  const deleteSprint = useMutation({
    mutationFn: (id: number) => sprintDelete(id),
    onSuccess: (_, deletedId) => {
      if (selectedSprintId === deletedId) selectSprint(null);
      qc.invalidateQueries({ queryKey: ['sprintList'] });
      qc.invalidateQueries({ queryKey: ['sprintCurrent'] });
      qc.invalidateQueries({ queryKey: ['issueList'] });
      qc.invalidateQueries({ queryKey: ['boardStatus'] });
      qc.invalidateQueries({ queryKey: ['sessionRestore'] });
      toast.success('스프린트가 삭제되었습니다');
    },
    onError: (e) => toast.error(`삭제 실패: ${e}`),
  });

  const selectedSprint = isBacklog ? null : sprints.find((s) => s.id === selectedSprintId);

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
          <BacklogItem
            selected={isBacklog}
            onClick={() => selectSprint(BACKLOG_ID)}
            count={backlogIssues.length}
          />
          {sprints.length === 0 && (
            <p className="text-xs text-slate-400 text-center mt-4">스프린트가 없습니다</p>
          )}
          {sprints.map((sprint) => (
            <SprintItem
              key={sprint.id}
              sprint={sprint}
              selected={sprint.id === selectedSprintId}
              onClick={() => selectSprint(sprint.id)}
              onActivate={() => activateSprint.mutate(sprint.id)}
              onComplete={() => completeSprint.mutate(sprint.id)}
              onDelete={() => deleteSprint.mutate(sprint.id)}
              onEdit={() => setEditSprint(sprint)}
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
              {isBacklog ? '백로그' : (selectedSprint ? selectedSprint.name : '스프린트를 선택하세요')}
            </h2>
            {isBacklog && (
              <p className="text-xs text-slate-400 mt-0.5">스프린트에 아직 들어가지 않은 이슈 모음</p>
            )}
            {!isBacklog && selectedSprint?.goal && (
              <p className="text-xs text-slate-400 mt-0.5">{selectedSprint.goal}</p>
            )}
          </div>
          <div className="flex items-center gap-2">
            <input
              type="text"
              placeholder="#ID 또는 이슈 검색…"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="text-sm border border-slate-200 rounded-lg px-3 py-1.5 bg-white focus:outline-none focus:ring-2 focus:ring-indigo-500/20 min-w-[180px]"
            />
            <button
              type="button"
              onClick={() => setEpicModalOpen(true)}
              className="text-sm px-3 py-1.5 bg-slate-200 hover:bg-slate-300 text-slate-700 rounded-md"
            >
              + 새 에픽
            </button>
            {allEpics.length > 0 && (
              <button
                type="button"
                onClick={() => setIssueModalEpicId(allEpics[0].id)}
                className="text-sm px-3 py-1.5 bg-indigo-600 hover:bg-indigo-500 text-white rounded-md"
              >
                + 새 이슈
              </button>
            )}
          </div>
        </div>

        {/* Epic + Issue tree */}
        <div className="flex-1 overflow-y-auto p-6">
          {selectedSprintId == null && (
            <p className="text-slate-400 text-center mt-20">왼쪽에서 스프린트나 백로그를 선택하세요</p>
          )}
          {selectedSprintId != null && filteredGrouped.length === 0 && (
            <p className="text-slate-400 text-center mt-20">
              {debouncedQuery.trim()
                ? `"${debouncedQuery.trim()}" 에 일치하는 이슈가 없습니다.`
                : isBacklog
                  ? '백로그가 비어 있습니다. 새 이슈를 백로그로 추가하세요.'
                  : '이슈가 없습니다. "+ 새 이슈" 로 이 스프린트에 이슈를 추가하세요.'}
            </p>
          )}
          {filteredGrouped.map(({ epic, issues }) => (
            <EpicRow
              key={epic.id}
              epic={epic}
              issues={issues}
              sprints={sprints}
              onIssueClick={selectIssue}
              onAddIssue={() => setIssueModalEpicId(epic.id)}
              onEdit={setEditEpic}
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
      />
      <CreateIssueModal
        open={issueModalEpicId != null}
        onClose={() => setIssueModalEpicId(null)}
        defaultEpicId={issueModalEpicId ?? undefined}
        defaultSprintId={isBacklog ? null : (selectedSprintId ?? null)}
      />
      <EditEpicModal
        epic={editEpic}
        onClose={() => setEditEpic(null)}
      />
      <EditSprintModal
        sprint={editSprint}
        onClose={() => setEditSprint(null)}
      />
    </div>
  );
}
