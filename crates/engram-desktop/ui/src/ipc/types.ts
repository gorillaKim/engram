export type IssueStatus = 'required' | 'ready' | 'working' | 'demo' | 'finished' | 'cancelled';
export type IssuePriority = 'critical' | 'high' | 'medium' | 'low';
export type EpicStatus = 'active' | 'completed' | 'cancelled';
export type SprintStatus = 'planning' | 'active' | 'completed' | 'cancelled';
export type TaskStatus = 'required' | 'ready' | 'working' | 'demo' | 'finished' | 'cancelled';
export type TaskSource = 'planned' | 'agent_discovered' | 'user_added';
export type NoteType = 'caveat' | 'decision' | 'discovery' | 'blocker_detail' | 'context' | 'reference' | 'comment';

export interface Issue {
  id: number;
  epic_id: number;
  /** Epic 에서 derive (ADR-0014). */
  mission_id: number | null;
  /** Epic 에서 derive (ADR-0014). null이면 백로그. */
  sprint_id: number | null;
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
  project_key: string;
  mission_id: number | null;
  sprint_id: number | null;
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

export interface CreateSprintInput {
  name: string;
  goal?: string;
  start_date?: string;
  end_date?: string;
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
  node_statuses: Record<string, string>;
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
  scope_expansion_ids: number[];
}

// ── Board (Kanban UI) ─────────────────────────────────────────────────────────

export interface IssueProjectBoard {
  project_key: string;
  required: Issue[];
  ready: Issue[];
  working: Issue[];
  demo: Issue[];
  finished: Issue[];
  /** 취소된 이슈 모음 — 사용자가 "취소 보기" 를 켰을 때 노출 */
  cancelled: Issue[];
}

export interface StalledIssueBrief {
  id: number;
  title: string;
  project_key: string;
  secs_since_activity: number | null;
}

export interface IssueBoardStatus {
  sprint_id: number;
  sprint_name: string;
  project_key: string | null;
  boards: IssueProjectBoard[];
  stalled_issues: StalledIssueBrief[];
}

export interface IssueFilter {
  epic_id?: number;
  sprint_id?: number | null;
  mission_id?: number | null;
  backlog_only?: boolean;
  project_key?: string;
  status?: IssueStatus;
  priority?: IssuePriority;
}

// ── MCP Supervisor ────────────────────────────────────────────────────────────

export interface SupervisorStatusSnapshot {
  running: boolean;
  port: number;
  started_at: string | null;
  uptime_secs: number;
  call_count: number;
  autostart: boolean;
}

export interface CallRecord {
  name: string;
  args_summary: string;
  ok: boolean;
  duration_ms: number;
  ts: string;
  session_id: string | null;
  reason: string | null;
}

export interface LogLine {
  level: string;
  target: string;
  msg: string;
  ts: string;
}

export interface TrayStallEntry {
  id: number;
  title: string;
}

export interface TrayBoardSummary {
  inbox: number;
  demo_review: number;
  working: number;
  /** "active" | "pending" | "stalled" | "none" */
  working_state: 'active' | 'pending' | 'stalled' | 'none';
  stalled_issues: TrayStallEntry[];
  stalled_total: number;
}

export interface ActivitySettings {
  warn_minutes: number;
  stall_minutes: number;
}

// ── Dashboard CRUD ────────────────────────────────────────────────────────────

export interface CreateIssueInput {
  epic_id: number;
  title: string;
  description?: string;
  goal?: string;
  priority?: IssuePriority;
}

export interface CreateEpicInput {
  project_key: string;
  mission_id?: number | null;
  sprint_id?: number | null;
  title: string;
  description?: string;
}

export interface CreateTaskInput {
  issue_id: number;
  title: string;
}

export type LinkType = 'blocks' | 'relates_to' | 'duplicates';

export interface IssueLink {
  id: number;
  source_id: number;
  target_id: number;
  link_type: LinkType;
  created_at: string;
}

// ── Mission ───────────────────────────────────────────────────────────────────

export type MissionStatus = 'active' | 'completed' | 'cancelled';

export interface Mission {
  id: number;
  jira_key: string | null;
  title: string;
  description: string | null;
  status: MissionStatus;
  sprint_id: number | null;
  created_at: string;
  updated_at: string;
}

export interface MissionProgress {
  id: number;
  title: string;
  epics_count: number;
  issues_count: number;
  todo_issues: number;
  working_issues: number;
  demo_issues: number;
  finished_issues: number;
  cancelled_issues: number;
  progress_rate: number;
}

export interface EpicWithIssues {
  epic: Epic;
  issues: Issue[];
}

export interface MissionTree {
  mission: Mission;
  epics: EpicWithIssues[];
  /** missions.sprint_id 로 조회한 sprint.title. sprint_id 없으면 null. */
  sprint_name: string | null;
}

export interface CreateMissionInput {
  title: string;
  description?: string | null;
  jira_key?: string | null;
}

export interface UpdateMissionInput {
  title?: string | null;
  description?: string | null;
  jira_key?: string | null;
  status?: MissionStatus | null;
}

// ── Updater ───────────────────────────────────────────────────────────────────

export interface UpdateInfo {
  version: string;
  currentVersion: string;
  body: string | null;
  date: string | null;
}

export interface UpdateProgress {
  chunkLength: number;
  contentLength: number | null;
}

export interface HistoryEntry {
  id: number;
  entity_type: string;
  entity_id: number;
  field: string;
  old_value: string | null;
  new_value: string | null;
  changed_by: string;
  created_at: string;
}
