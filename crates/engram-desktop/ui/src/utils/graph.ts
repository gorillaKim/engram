import { Position, type Node, type Edge } from '@xyflow/react';
import type { MissionTree, Sprint, Epic, Issue, EpicWithIssues } from '../ipc/types';
import type { MissionFlowNode } from '../components/flow/MissionNode';
import type { EpicFlowNode } from '../components/flow/EpicNode';
import type { IssueFlowNode } from '../components/flow/IssueNode';

const COL_GAP = 240;
const ISSUE_H = 64;    // approximate issue card height
const ISSUE_GAP = 8;   // vertical gap between issue nodes
const EPIC_H = 80;     // approximate epic card height
const EPIC_GAP = 20;   // vertical gap between epic groups
const MISSION_H = 110; // approximate mission card height

export function buildGraph(
  tree: MissionTree,
  sprints: Sprint[],
  onDoubleClickIssue: (issueId: number) => void,
  onDoubleClickEpic: (epic: Epic) => void,
  onDoubleClickMission: (mission: any) => void,
): {
  nodes: Node[];
  edges: Edge[];
} {
  const nodes: Node[] = [];
  const edges: Edge[] = [];

  const missionId = `mission-${tree.mission.id}`;

  // compute progress_rate from issues
  const allIssues = tree.epics.flatMap((e) => e.issues);
  const total = allIssues.length;
  const finished = allIssues.filter((i) => i.status === 'finished').length;
  const progressRate = total > 0 ? Math.round((finished / total) * 100) : 0;

  // First pass: compute cumulative Y positions based on actual block heights
  let curY = 0;
  const epicLayouts: { epicY: number; issueStartY: number; blockH: number }[] = [];

  for (const ew of tree.epics) {
    const n = ew.issues.length;
    const issueBlockH = n > 0 ? n * ISSUE_H + (n - 1) * ISSUE_GAP : ISSUE_H;
    const epicCenterY = curY + Math.max(0, issueBlockH / 2 - EPIC_H / 2);
    epicLayouts.push({ epicY: epicCenterY, issueStartY: curY, blockH: issueBlockH });
    curY += issueBlockH + EPIC_GAP;
  }

  const totalH = Math.max(curY - EPIC_GAP, MISSION_H + 40); // 40px buffer for sprint dropdown
  const missionY = totalH / 2 - MISSION_H / 2;

  // Mission node — vertically centered relative to all epics
  const missionNode: MissionFlowNode = {
    id: missionId,
    type: 'mission',
    position: { x: 0, y: missionY },
    data: {
      label: tree.mission.title,
      progress_rate: progressRate,
      status: tree.mission.status,
      missionData: tree.mission,
      onDoubleClickMission,
    },
    sourcePosition: Position.Right,
    targetPosition: Position.Left,
  };
  nodes.push(missionNode);

  // Epics — column 1, Issues — column 2
  tree.epics.forEach((ew: EpicWithIssues, ei: number) => {
    const epicId = `epic-${ew.epic.id}`;
    const { epicY, issueStartY } = epicLayouts[ei];

    const epicNode: EpicFlowNode = {
      id: epicId,
      type: 'epic',
      position: { x: COL_GAP, y: epicY },
      data: {
        label: ew.epic.title,
        status: ew.epic.status,
        project_key: ew.epic.project_key,
        epicData: ew.epic,
        sprints,
        onDoubleClickEpic,
      },
      sourcePosition: Position.Right,
      targetPosition: Position.Left,
    };
    nodes.push(epicNode as any);

    edges.push({
      id: `e-${missionId}-${epicId}`,
      source: missionId,
      target: epicId,
      type: 'smoothstep',
      style: { stroke: '#a78bfa', strokeWidth: 1.5 },
    });

    ew.issues.forEach((issue: Issue, ii: number) => {
      const issueId = `issue-${issue.id}`;

      const issueNode: IssueFlowNode = {
        id: issueId,
        type: 'issue',
        position: { x: COL_GAP * 2, y: issueStartY + ii * (ISSUE_H + ISSUE_GAP) },
        data: {
          label: issue.title,
          status: issue.status,
          priority: issue.priority,
          issueId: issue.id,
          onDoubleClickIssue,
        },
        sourcePosition: Position.Right,
        targetPosition: Position.Left,
      };
      nodes.push(issueNode as any);

      edges.push({
        id: `e-${epicId}-${issueId}`,
        source: epicId,
        target: issueId,
        type: 'smoothstep',
        style: { stroke: '#c4b5fd', strokeWidth: 1 },
      });
    });
  });

  return { nodes, edges };
}
