import { Handle, Position, type NodeProps, type Node } from '@xyflow/react';
import { StatusBadge } from '../StatusBadge';

export type IssueNodeData = {
  label: string;
  status: string;
  priority: string;
  issueId: number;
  onDoubleClickIssue: (issueId: number) => void;
  [key: string]: unknown;
};

export type IssueFlowNode = Node<IssueNodeData, 'issue'>;

const PRIORITY_DOT: Record<string, string> = {
  critical: 'bg-red-500',
  high: 'bg-orange-400',
  medium: 'bg-yellow-400',
  low: 'bg-slate-300',
};

export function IssueNode({ data }: NodeProps<IssueFlowNode>) {
  const dot = PRIORITY_DOT[data.priority] ?? 'bg-slate-300';
  return (
    <div
      className="relative w-36 rounded-md border border-slate-200 bg-white shadow-sm p-2 flex flex-col gap-1 cursor-pointer hover:border-indigo-400 hover:shadow-md transition-all"
      onDoubleClick={() => data.onDoubleClickIssue(data.issueId)}
      title="더블클릭으로 상세 보기"
    >
      <Handle type="target" position={Position.Left} style={{ background: '#c4b5fd', border: 'none' }} />
      <div className="flex items-center gap-1.5">
        <span className={`w-1.5 h-1.5 rounded-full flex-shrink-0 ${dot}`} />
        <StatusBadge status={data.status} />
      </div>
      <p className="text-[11px] font-medium text-slate-700 leading-tight line-clamp-2">
        {data.label}
      </p>
    </div>
  );
}
