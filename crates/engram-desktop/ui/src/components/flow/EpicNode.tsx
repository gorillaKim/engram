import { Handle, Position, type NodeProps, type Node } from '@xyflow/react';
import type { Epic, Sprint } from '../../ipc/types';
import { StatusBadge } from '../StatusBadge';

export type EpicNodeData = {
  label: string;
  status: string;
  project_key: string;
  epicData: Epic;
  sprints: Sprint[];
  onDoubleClickEpic: (epic: Epic) => void;
  [key: string]: unknown;
};

export type EpicFlowNode = Node<EpicNodeData, 'epic'>;

export function EpicNode({ data }: NodeProps<EpicFlowNode>) {
  const epic = data.epicData;
  const sprints = (data.sprints as Sprint[]) ?? [];
  const sprintName = epic.sprint_id
    ? (sprints.find((s) => s.id === epic.sprint_id)?.name ?? `Sprint #${epic.sprint_id}`)
    : '백로그';

  return (
    <div
      className="relative w-44 rounded-lg border border-violet-300 bg-violet-50 shadow p-2.5 flex flex-col gap-1.5 cursor-pointer hover:border-violet-500 hover:shadow-md transition-all"
      onDoubleClick={() => data.onDoubleClickEpic(data.epicData)}
      title="더블클릭으로 수정"
    >
      <Handle type="target" position={Position.Left} style={{ background: '#a78bfa', border: 'none' }} />
      <Handle type="source" position={Position.Right} style={{ background: '#c4b5fd', border: 'none' }} />
      <div className="flex items-center justify-between gap-1">
        <span className="text-[10px] font-bold text-violet-600 uppercase tracking-wide">Epic</span>
        <StatusBadge status={data.status} />
      </div>
      <p className="text-xs font-semibold text-slate-800 leading-tight line-clamp-2">
        {data.label}
      </p>
      <div className="flex items-center justify-between gap-2 mt-1">
        <span className="text-[10px] text-slate-400 font-mono">{data.project_key}</span>
        <span className={`text-[9px] px-1.5 py-0.5 rounded font-medium ${epic.sprint_id ? 'bg-indigo-100 text-indigo-700 border border-indigo-200/30' : 'bg-slate-200 text-slate-600'}`}>
          {sprintName}
        </span>
      </div>
    </div>
  );
}
