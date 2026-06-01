import { Handle, Position, type NodeProps, type Node } from '@xyflow/react';
import type { Mission } from '../../ipc/types';
import { StatusBadge } from '../StatusBadge';

export type MissionNodeData = {
  label: string;
  progress_rate: number;
  status: string;
  missionData: Mission;
  onDoubleClickMission: (mission: Mission) => void;
  [key: string]: unknown;
};

export type MissionFlowNode = Node<MissionNodeData, 'mission'>;

export function MissionNode({ data }: NodeProps<MissionFlowNode>) {
  return (
    <div
      className="relative w-52 rounded-xl border-2 border-indigo-400 bg-white shadow-lg p-3 flex flex-col gap-2 cursor-pointer hover:border-indigo-600 hover:shadow-xl transition-all"
      onDoubleClick={() => data.onDoubleClickMission(data.missionData)}
      title="더블클릭으로 수정"
    >
      <Handle type="source" position={Position.Right} style={{ background: '#a78bfa', border: 'none' }} />
      <div className="flex items-center justify-between gap-1">
        <span className="text-xs font-bold text-indigo-700 uppercase tracking-wide">Mission</span>
        <StatusBadge status={data.status} />
      </div>
      <p className="text-sm font-semibold text-slate-800 leading-tight line-clamp-2">
        {data.label}
      </p>

      <div className="flex flex-col gap-1 border-t border-slate-100 pt-2">
        <div className="flex justify-between text-[10px] text-slate-500">
          <span>진행률</span>
          <span className="font-mono font-semibold text-indigo-600">{data.progress_rate}%</span>
        </div>
        <div className="h-1.5 w-full rounded-full bg-slate-200 overflow-hidden">
          <div
            className="h-full rounded-full bg-indigo-500 transition-all duration-300"
            style={{ width: `${data.progress_rate}%` }}
          />
        </div>
      </div>
    </div>
  );
}
