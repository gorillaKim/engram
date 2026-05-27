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

const RESOLVED_STATUSES = new Set(['finished', 'cancelled']);

export function BlockingGraphView({ graph, focusIssueId, issueTitles }: Props) {
  const { nodes, edges } = useMemo(() => {
    const nodeStatus = (id: number): string =>
      graph.node_statuses[String(id)] ?? 'unknown';

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

          // 양쪽 중 하나라도 resolved면 해소된 엣지
          const srcResolved = RESOLVED_STATUSES.has(nodeStatus(src));
          const tgtResolved = RESOLVED_STATUSES.has(nodeStatus(tgt));
          const edgeResolved = srcResolved || tgtResolved;

          const strokeColor = graph.has_cycle
            ? '#ef4444'
            : edgeResolved
            ? '#cbd5e1' // slate-300 — 해소된 관계는 흐리게
            : '#94a3b8';

          allEdges.push({
            id: key,
            source: String(src),
            target: String(tgt),
            animated: graph.has_cycle,
            style: {
              stroke: strokeColor,
              strokeDasharray: edgeResolved ? '6 3' : undefined,
              opacity: edgeResolved ? 0.5 : 1,
            },
            label: edgeResolved ? '✓ 해소' : undefined,
            labelStyle: edgeResolved
              ? { fontSize: 9, fill: '#94a3b8', fontWeight: 600 }
              : undefined,
            labelBgStyle: edgeResolved
              ? { fill: '#f8fafc', fillOpacity: 0.9 }
              : undefined,
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
        const status = nodeStatus(id);
        const isFinished = status === 'finished';
        const isCancelled = status === 'cancelled';
        const isResolved = isFinished || isCancelled;
        const isBlocker = (distBackward.get(id) ?? 0) > 0 && !distForward.has(id);

        // 상태 아이콘
        const statusIcon = isFinished ? '✅ ' : isCancelled ? '⛔ ' : '';
        const title = issueTitles.get(id) ?? '';
        const label = `${statusIcon}#${id} ${title}`.trim();

        let style: React.CSSProperties;
        if (isFocus) {
          style = {
            background: '#e0e7ff', border: '2px solid #6366f1',
            fontWeight: 700, fontSize: 11, maxWidth: 160,
          };
        } else if (isFinished) {
          style = {
            background: '#f0fdf4', border: '1px dashed #86efac',
            fontSize: 11, maxWidth: 160, opacity: 0.7,
            textDecoration: 'line-through', color: '#64748b',
          };
        } else if (isCancelled) {
          style = {
            background: '#fef2f2', border: '1px dashed #fca5a5',
            fontSize: 11, maxWidth: 160, opacity: 0.6,
            textDecoration: 'line-through', color: '#94a3b8',
          };
        } else if (isBlocker) {
          style = {
            background: '#fee2e2', border: '1px solid #fca5a5',
            fontSize: 11, maxWidth: 160,
          };
        } else {
          style = {
            background: '#fef9c3', border: '1px solid #fde047',
            fontSize: 11, maxWidth: 160,
          };
        }

        // 해소된 노드에 포커스가 아닌 경우에만 resolved 시각화 적용
        if (isResolved && isFocus) {
          style = { ...style, opacity: 1, textDecoration: 'none' };
        }

        allNodes.push({
          id: String(id),
          position: { x: layer * 200, y: i * 90 },
          data: { label },
          style,
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
