import { useMemo } from 'react';
import {
  ReactFlow,
  Background,
  Controls,
  MarkerType,
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
          const strokeColor = graph.has_cycle ? '#ef4444' : '#94a3b8';
          allEdges.push({
            id: key,
            source: String(src),
            target: String(tgt),
            animated: graph.has_cycle,
            style: { stroke: strokeColor },
            markerEnd: {
              type: MarkerType.ArrowClosed,
              color: strokeColor,
            },
          });
        }
      }
    }

    // BFS forward: focusIssueId가 전이적으로 블로킹하는 노드들 (오른쪽)
    const distForward = new Map<number, number>([[focusIssueId, 0]]);
    const fwdQ = [focusIssueId];
    while (fwdQ.length > 0) {
      const cur = fwdQ.shift()!;
      for (const e of allEdges) {
        const src = parseInt(e.source, 10);
        const tgt = parseInt(e.target, 10);
        if (src === cur && !distForward.has(tgt)) {
          distForward.set(tgt, distForward.get(cur)! + 1);
          fwdQ.push(tgt);
        }
      }
    }

    // BFS backward: focusIssueId를 전이적으로 블로킹하는 노드들 (왼쪽)
    const distBackward = new Map<number, number>([[focusIssueId, 0]]);
    const bwdQ = [focusIssueId];
    while (bwdQ.length > 0) {
      const cur = bwdQ.shift()!;
      for (const e of allEdges) {
        const src = parseInt(e.source, 10);
        const tgt = parseInt(e.target, 10);
        if (tgt === cur && !distBackward.has(src)) {
          distBackward.set(src, distBackward.get(cur)! + 1);
          bwdQ.push(src);
        }
      }
    }

    const allNodeIds = new Set<number>([...distForward.keys(), ...distBackward.keys()]);
    const filteredEdges = allEdges.filter(
      (e) => allNodeIds.has(parseInt(e.source, 10)) && allNodeIds.has(parseInt(e.target, 10))
    );

    // 레이어: 왼쪽(blockers) → focus → 오른쪽(blocked), 전이적 체인 모두 표시
    const maxBack = Math.max(0, ...Array.from(distBackward.values()));
    const getLayer = (id: number): number => {
      if (id === focusIssueId) return maxBack;
      if (distBackward.get(id)! > 0 && !distForward.has(id)) return maxBack - distBackward.get(id)!;
      return maxBack + (distForward.get(id) ?? 0);
    };

    const layerGroups = new Map<number, number[]>();
    for (const id of allNodeIds) {
      const layer = getLayer(id);
      if (!layerGroups.has(layer)) layerGroups.set(layer, []);
      layerGroups.get(layer)!.push(id);
    }

    const allNodes: Node[] = [];
    for (const [layer, ids] of layerGroups.entries()) {
      ids.forEach((id, i) => {
        const isFocus = id === focusIssueId;
        const isBlocker = (distBackward.get(id) ?? 0) > 0 && !distForward.has(id);
        allNodes.push({
          id: String(id),
          position: { x: layer * 200, y: i * 90 },
          data: { label: `#${id} ${issueTitles.get(id) ?? ''}`.trim() },
          style: isFocus
            ? { background: '#e0e7ff', border: '2px solid #6366f1', fontWeight: 700, fontSize: 11, maxWidth: 140 }
            : isBlocker
            ? { background: '#fee2e2', border: '1px solid #fca5a5', fontSize: 11, maxWidth: 140 }
            : { background: '#fef9c3', border: '1px solid #fde047', fontSize: 11, maxWidth: 140 },
        });
      });
    }

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
        panOnDrag={true}
      >
        <Background gap={16} color="#f1f5f9" />
        <Controls showInteractive={false} />
      </ReactFlow>
    </div>
  );
}
