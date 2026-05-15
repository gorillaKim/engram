import { useMemo } from 'react';
import {
  ReactFlow,
  Background,
  Controls,
  type Node,
  type Edge,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import type { BlockingGraph } from '../ipc/types';

interface Props {
  graph: BlockingGraph;
  focusIssueId: number;
  issueTitles: Map<number, string>;
}

export function BlockingGraphView({ graph, focusIssueId, issueTitles }: Props) {
  const { nodes, edges } = useMemo(() => {
    // Collect all edges (consecutive pairs in each chain)
    const edgeSet = new Set<string>();
    const allEdges: Edge[] = [];
    for (const chain of graph.chains) {
      for (let i = 0; i < chain.length - 1; i++) {
        const src = chain[i];
        const tgt = chain[i + 1];
        const key = `${src}-${tgt}`;
        if (!edgeSet.has(key)) {
          edgeSet.add(key);
          allEdges.push({
            id: key,
            source: String(src),
            target: String(tgt),
            animated: graph.has_cycle,
            style: graph.has_cycle ? { stroke: '#ef4444' } : { stroke: '#94a3b8' },
          });
        }
      }
    }

    // Only include nodes within 1 hop of focusIssueId
    const neighbors = new Set<number>([focusIssueId]);
    for (const e of allEdges) {
      const src = parseInt(e.source, 10);
      const tgt = parseInt(e.target, 10);
      if (src === focusIssueId || tgt === focusIssueId) {
        neighbors.add(src);
        neighbors.add(tgt);
      }
    }

    const filteredEdges = allEdges.filter(
      (e) => neighbors.has(parseInt(e.source, 10)) && neighbors.has(parseInt(e.target, 10))
    );

    // Layout: blockers on left, focus in center, blocked-by on right
    const blockers = filteredEdges
      .filter((e) => parseInt(e.target, 10) === focusIssueId)
      .map((e) => parseInt(e.source, 10));
    const blocked = filteredEdges
      .filter((e) => parseInt(e.source, 10) === focusIssueId)
      .map((e) => parseInt(e.target, 10));

    const allNodes: Node[] = [];

    blockers.forEach((id, i) => {
      allNodes.push({
        id: String(id),
        position: { x: 0, y: i * 80 },
        data: { label: `#${id} ${issueTitles.get(id) ?? ''}`.trim() },
        style: { background: '#fee2e2', border: '1px solid #fca5a5', fontSize: 11, maxWidth: 140 },
      });
    });

    allNodes.push({
      id: String(focusIssueId),
      position: { x: 200, y: (Math.max(blockers.length, blocked.length) - 1) * 40 },
      data: { label: `#${focusIssueId} ${issueTitles.get(focusIssueId) ?? ''}`.trim() },
      style: { background: '#e0e7ff', border: '2px solid #6366f1', fontWeight: 700, fontSize: 11, maxWidth: 140 },
    });

    blocked.forEach((id, i) => {
      allNodes.push({
        id: String(id),
        position: { x: 400, y: i * 80 },
        data: { label: `#${id} ${issueTitles.get(id) ?? ''}`.trim() },
        style: { background: '#fef9c3', border: '1px solid #fde047', fontSize: 11, maxWidth: 140 },
      });
    });

    return { nodes: allNodes, edges: filteredEdges };
  }, [graph, focusIssueId, issueTitles]);

  if (nodes.length <= 1 && edges.length === 0) {
    return <p className="text-xs text-slate-400 py-2">블로킹 관계 없음</p>;
  }

  return (
    <div className="w-full h-48 rounded-md border border-slate-200 overflow-hidden">
      <ReactFlow
        nodes={nodes}
        edges={edges}
        fitView
        nodesDraggable={false}
        nodesConnectable={false}
        elementsSelectable={false}
        zoomOnScroll={false}
        panOnDrag={false}
      >
        <Background gap={16} color="#f1f5f9" />
        <Controls showInteractive={false} />
      </ReactFlow>
    </div>
  );
}
