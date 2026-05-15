export type IssueStatus = 'required' | 'ready' | 'working' | 'demo' | 'finished' | 'cancelled';
export type IssuePriority = 'critical' | 'high' | 'medium' | 'low';
export type EpicStatus = 'active' | 'completed' | 'cancelled';
export type SprintStatus = 'planning' | 'active' | 'completed';
export type TaskStatus = 'required' | 'ready' | 'working' | 'demo' | 'finished' | 'cancelled';
export type TaskSource = 'planned' | 'agent_discovered' | 'user_added';
export type NoteType = 'caveat' | 'decision' | 'discovery' | 'blocker_detail' | 'context' | 'reference';

export interface Issue {
  id: number;
  epic_id: number;
  title: string;
  description: string | null;
  goal: string | null;
  status: IssueStatus;
  priority: IssuePriority;
  created_at: string;
  updated_at: string;
}

export interface Epic {
  id: number;
  sprint_id: number;
  project_key: string;
  title: string;
  description: string | null;
  status: EpicStatus;
  created_at: string;
  updated_at: string;
}

export interface Sprint {
  id: number;
  name: string;
  goal: string | null;
  status: SprintStatus;
  start_date: string | null;
  end_date: string | null;
  created_at: string;
  updated_at: string;
}

export interface Task {
  id: number;
  issue_id: number;
  title: string;
  description: string | null;
  goal: string | null;
  status: TaskStatus;
  ord: number;
  source: TaskSource;
  created_at: string;
  updated_at: string;
}

export interface Note {
  id: number;
  issue_id: number;
  task_id: number | null;
  note_type: NoteType;
  summary: string;
  detail: string | null;
  author: string;
  resolved: boolean;
  created_at: string;
  resolved_at: string | null;
}

export interface NoteSummary {
  id: number;
  note_type: NoteType;
  summary: string;
  task_id: number | null;
  resolved: boolean;
}

export interface CreateNoteInput {
  issue_id: number;
  task_id?: number | null;
  note_type: NoteType;
  summary: string;
  detail?: string | null;
  author?: string | null;
}

export interface BlockingGraph {
  chains: number[][];
  leaf_blockers: number[];
  has_cycle: boolean;
}

// ── Session restore ───────────────────────────────────────────────────────────

export interface EpicProgress {
  done: number;
  in_progress: number;
  todo: number;
  total: number;
}

export interface IssueBrief {
  id: number;
  title: string;
  epic_id: number;
  created_at: string;
}

export interface NextTask {
  task_id: number;
  task_title: string;
  issue_id: number;
  issue_title: string;
  epic_id: number;
  epic_title: string;
  project_key: string;
  reason: string;
}

export interface IssueSnapshot {
  issue: Issue;
  active_notes: NoteSummary[];
  current_task: Task | null;
  blocked_by: number[];
}

export interface EpicSnapshot {
  epic: Epic;
  active_issues: IssueSnapshot[];
  progress: EpicProgress;
}

export interface SessionSnapshot {
  sprint_id: number;
  sprint_name: string;
  sprint_goal: string | null;
  project_key: string | null;
  active_epics: EpicSnapshot[];
  next_action: NextTask | null;
  pending_drafts: IssueBrief[];
  warnings: string[];
}

// ── Board (Kanban UI) ─────────────────────────────────────────────────────────

export interface IssueProjectBoard {
  project_key: string;
  required: Issue[];
  ready: Issue[];
  working: Issue[];
  demo: Issue[];
  finished: Issue[];
}

export interface IssueBoardStatus {
  sprint_id: number;
  sprint_name: string;
  project_key: string | null;
  boards: IssueProjectBoard[];
}

export interface IssueFilter {
  epic_id?: number;
  project_key?: string;
  status?: IssueStatus;
  priority?: IssuePriority;
}
