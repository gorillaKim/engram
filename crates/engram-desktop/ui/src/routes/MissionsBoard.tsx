import { useCallback, useEffect, useMemo, useState } from 'react';
import {
  ReactFlow,
  Background,
  Controls,
  useEdgesState,
  useNodesState,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import type { Mission, MissionTree, Epic, Sprint } from '../ipc/types';
import { missionList, missionGetTree, sprintCurrent, sprintList, epicList } from '../ipc/invoke';
import { useUIStore } from '../store/ui';
import { MissionModal } from '../components/MissionModal';
import { EditEpicModal } from '../components/EditEpicModal';
import { StatusBadge } from '../components/StatusBadge';
import { MissionNode } from '../components/flow/MissionNode';
import { EpicNode } from '../components/flow/EpicNode';
import { IssueNode } from '../components/flow/IssueNode';
import { buildGraph } from '../utils/graph';

const NODE_TYPES = {
  mission: MissionNode,
  epic: EpicNode,
  issue: IssueNode,
};

function FlowCanvas({
  tree,
  sprints,
  onIssueDoubleClick,
  onEpicDoubleClick,
  onMissionDoubleClick,
}: {
  tree: MissionTree;
  sprints: Sprint[];
  onIssueDoubleClick: (issueId: number) => void;
  onEpicDoubleClick: (epic: Epic) => void;
  onMissionDoubleClick: (mission: Mission) => void;
}) {
  const { nodes: initNodes, edges: initEdges } = useMemo(
    () =>
      buildGraph(
        tree,
        sprints,
        onIssueDoubleClick,
        onEpicDoubleClick,
        onMissionDoubleClick,
      ),
    [tree, sprints, onIssueDoubleClick, onEpicDoubleClick, onMissionDoubleClick],
  );
  const [nodes, , onNodesChange] = useNodesState(initNodes);
  const [edges, , onEdgesChange] = useEdgesState(initEdges);

  return (
    <ReactFlow
      nodes={nodes}
      edges={edges}
      onNodesChange={onNodesChange}
      onEdgesChange={onEdgesChange}
      nodeTypes={NODE_TYPES}
      fitView
      fitViewOptions={{ padding: 0.2 }}
      minZoom={0.5} // 이슈 #366: fitView 텍스트 축소 한계값 minZoom 0.5 설정
      maxZoom={2}
      proOptions={{ hideAttribution: true }}
    >
      <Background gap={16} color="#e2e8f0" />
      <Controls showInteractive={false} />
    </ReactFlow>
  );
}

export function MissionsBoard() {
  const [missions, setMissions] = useState<Mission[]>([]);
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [tree, setTree] = useState<MissionTree | null>(null);
  const [loading, setLoading] = useState(true);
  const [treeLoading, setTreeLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [modalOpen, setModalOpen] = useState(false);
  const [editingMission, setEditingMission] = useState<Mission | undefined>(undefined);
  const [editingEpic, setEditingEpic] = useState<Epic | null>(null);

  // 추가 상태
  const [sprints, setSprints] = useState<Sprint[]>([]);
  const [activeSprint, setActiveSprint] = useState<Sprint | null>(null);
  const [epics, setEpics] = useState<Epic[]>([]);
  const [selectedFilterSprintId, setSelectedFilterSprintId] = useState<number | 'all' | 'backlog'>('all');

  const { selectIssue, selectProject } = useUIStore();

  // issue.id → project_key lookup built from current tree
  const issueProjectMap = useMemo<Map<number, string>>(() => {
    const map = new Map<number, string>();
    if (!tree) return map;
    for (const ew of tree.epics) {
      for (const issue of ew.issues) {
        map.set(issue.id, ew.epic.project_key);
      }
    }
    return map;
  }, [tree]);

  const handleIssueDoubleClick = useCallback((issueId: number) => {
    const projectKey = issueProjectMap.get(issueId) ?? null;
    selectProject(projectKey);
    selectIssue(issueId);
  }, [issueProjectMap, selectIssue, selectProject]);

  const handleEpicDoubleClick = useCallback((epic: Epic) => {
    setEditingEpic(epic);
  }, []);

  const handleMissionDoubleClick = useCallback((mission: Mission) => {
    setEditingMission(mission);
    setModalOpen(true);
  }, []);

  // 스프린트 목록, 활성 스프린트 및 에픽 목록 로드
  useEffect(() => {
    sprintList()
      .then(setSprints)
      .catch((e: unknown) => console.error("스프린트 목록 로드 실패:", e));
      
    sprintCurrent()
      .then(setActiveSprint)
      .catch((e: unknown) => console.error("활성 스프린트 로드 실패:", e));

    epicList()
      .then(setEpics)
      .catch((e: unknown) => console.error("에픽 목록 로드 실패:", e));
  }, []);

  const loadTree = useCallback((id: number) => {
    setTreeLoading(true);
    setTree(null);
    missionGetTree(id)
      .then(setTree)
      .catch((e: unknown) => setError(String(e)))
      .finally(() => setTreeLoading(false));
  }, []);

  const reloadMissions = useCallback(() => {
    setLoading(true);
    
    const fetchMissions = async () => {
      const list = await missionList(true);
      setMissions(list);
      setLoading(false);
    };

    fetchMissions().catch((e: unknown) => {
      setError(String(e));
      setLoading(false);
    });
  }, []);

  const filteredMissions = useMemo(() => {
    if (selectedFilterSprintId === 'all') return missions;

    const targetEpics = epics.filter((epic) => {
      if (selectedFilterSprintId === 'backlog') {
        return epic.sprint_id === null;
      }
      return epic.sprint_id === selectedFilterSprintId;
    });

    const validMissionIds = new Set(
      targetEpics
        .map((epic) => epic.mission_id)
        .filter((id): id is number => id !== null)
    );

    return missions.filter((m) => validMissionIds.has(m.id));
  }, [missions, epics, selectedFilterSprintId]);

  useEffect(() => {
    if (filteredMissions.length > 0) {
      if (selectedId === null || !filteredMissions.some((m) => m.id === selectedId)) {
        setSelectedId(filteredMissions[0].id);
      }
    } else {
      setSelectedId(null);
      setTree(null);
    }
  }, [filteredMissions, selectedId]);

  useEffect(() => {
    reloadMissions();
  }, [reloadMissions]);

  const handleModalClose = useCallback(() => {
    setModalOpen(false);
    setEditingMission(undefined);
    reloadMissions();
  }, [reloadMissions]);

  useEffect(() => {
    if (selectedId != null) loadTree(selectedId);
  }, [selectedId, loadTree]);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full text-slate-400 text-sm">
        미션 목록 로딩 중…
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-full text-red-500 text-sm">
        오류: {error}
      </div>
    );
  }

  if (missions.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-3 text-slate-400">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          className="w-12 h-12 text-slate-300"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
          strokeWidth={1.5}
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            d="M9 6.75V15m6-6v8.25m.503 3.498 4.875-2.437c.381-.19.622-.58.622-1.006V4.82c0-.836-.88-1.38-1.628-1.006l-3.869 1.934c-.317.159-.69.159-1.006 0L9.503 3.252a1.125 1.125 0 0 0-1.006 0L3.622 5.689C3.24 5.88 3 6.27 3 6.695V19.18c0 .836.88 1.38 1.628 1.006l3.869-1.934c.317-.159.69-.159 1.006 0l4.994 2.497c.317.158.69.158 1.006 0Z"
          />
        </svg>
        <p className="text-sm font-medium">미션이 없습니다. 먼저 미션을 생성하세요.</p>
      </div>
    );
  }

  return (
    <div className="flex h-full overflow-hidden">
      {/* 좌측 미션 목록 */}
      <aside className="w-56 flex-shrink-0 border-r border-slate-200 bg-white flex flex-col overflow-y-auto">
        <div className="px-4 py-3 border-b border-slate-100 flex items-center justify-between">
          <h2 className="text-xs font-bold text-slate-500 uppercase tracking-wider">Missions</h2>
          <button
            onClick={() => { setEditingMission(undefined); setModalOpen(true); }}
            className="w-5 h-5 flex items-center justify-center rounded text-slate-400 hover:text-violet-600 hover:bg-violet-50 transition-colors text-sm font-bold"
            title="미션 생성"
          >
            +
          </button>
        </div>
        
        {/* 스프린트 필터 */}
        <div className="px-4 py-2 border-b border-slate-100 bg-slate-50/50">
          <label className="text-[10px] font-bold text-slate-400 block mb-1">스프린트 필터</label>
          <select
            value={selectedFilterSprintId}
            onChange={(e) => {
              const val = e.target.value;
              if (val === 'all' || val === 'backlog') {
                setSelectedFilterSprintId(val);
              } else {
                setSelectedFilterSprintId(Number(val));
              }
            }}
            className="w-full text-xs border border-slate-200 rounded px-2 py-1 bg-white focus:outline-none focus:ring-1 focus:ring-indigo-500 font-medium text-slate-700"
          >
            <option value="all">전체 스프린트</option>
            <option value="backlog">백로그 (스프린트 미지정)</option>
            {sprints.map((s) => (
              <option key={s.id} value={s.id}>{s.name}</option>
            ))}
          </select>
        </div>

        <ul className="flex-1 py-2">
          {filteredMissions.map((m) => (
            <li key={m.id} className="group relative">
              <button
                onClick={() => setSelectedId(m.id)}
                className={`w-full text-left px-4 py-2.5 flex flex-col gap-0.5 transition-colors pr-8 ${
                  selectedId === m.id
                    ? 'bg-indigo-50 border-r-2 border-indigo-500'
                    : 'hover:bg-slate-50'
                }`}
              >
                <span className="text-xs font-semibold text-slate-800 line-clamp-2 leading-tight">
                  {m.title}
                </span>
                {m.jira_key && (
                  <span className="text-[10px] font-mono text-slate-400">{m.jira_key}</span>
                )}
                <StatusBadge status={m.status} />
              </button>
              <button
                onClick={(e) => { e.stopPropagation(); setEditingMission(m); setModalOpen(true); }}
                className="absolute right-2 top-2.5 opacity-0 group-hover:opacity-100 text-slate-400 hover:text-violet-600 transition-all text-xs"
                title="미션 수정"
              >
                ✎
              </button>
            </li>
          ))}
        </ul>
      </aside>

      {/* 우측 ReactFlow 트리 */}
      <div className="flex-1 relative overflow-hidden bg-slate-50">
        {treeLoading && (
          <div className="absolute inset-0 flex items-center justify-center bg-slate-50/80 z-10 text-slate-400 text-sm">
            트리 로딩 중…
          </div>
        )}
        {tree && !treeLoading && (
          <div className="w-full h-full">
            <FlowCanvas
              key={tree.mission.id}
              tree={tree}
              sprints={sprints}
              onIssueDoubleClick={handleIssueDoubleClick}
              onEpicDoubleClick={handleEpicDoubleClick}
              onMissionDoubleClick={handleMissionDoubleClick}
            />
          </div>
        )}
        {!tree && !treeLoading && (
          <div className="flex flex-col items-center justify-center h-full gap-2 text-slate-400 text-sm">
            {!activeSprint ? (
              <>
                <svg xmlns="http://www.w3.org/2000/svg" className="w-8 h-8 text-slate-300" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                </svg>
                <p className="font-medium text-slate-500">진행 중인 스프린트가 없습니다.</p>
                <p className="text-[11px] text-slate-400">스프린트를 새로 시작하세요.</p>
              </>
            ) : missions.length === 0 ? (
              <p>표시할 미션이 없습니다.</p>
            ) : (
              <p>미션을 선택하세요.</p>
            )}
          </div>
        )}
      </div>

      <MissionModal
        open={modalOpen}
        onClose={handleModalClose}
        mission={editingMission}
      />
      <EditEpicModal
        epic={editingEpic}
        onClose={() => { setEditingEpic(null); if (selectedId) loadTree(selectedId); }}
      />
    </div>
  );
}
