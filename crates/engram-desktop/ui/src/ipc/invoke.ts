import { invoke } from '@tauri-apps/api/core';
import type {
  SessionSnapshot, IssueBoardStatus, Issue, Epic, Sprint,
  IssueFilter, Task, Note, CreateNoteInput, BlockingGraph,
  SupervisorStatusSnapshot, CallRecord, LogLine, SprintProgress,
  CreateEpicInput, CreateIssueInput, CreateTaskInput,
  IssueLink, LinkType, EpicStatus, HistoryEntry, CreateSprintInput,
  Mission, MissionProgress, MissionTree,
  CreateMissionInput, UpdateMissionInput, McpToolDefinition,
  RetrospectiveWithItems, Retrospective, RetroActionItem,
  CreateRetrospectiveInput, CreateRetroActionItemInput,
  UpdateRetrospectiveInput, UpdateRetroActionItemInput,
} from './types';

export const sessionRestore = (project_key?: string) =>
  invoke<SessionSnapshot>('session_restore', { project_key: project_key ?? null });

export const boardStatus = (project_key?: string) =>
  invoke<IssueBoardStatus>('board_status', { project_key: project_key ?? null });

export const issueList = (filter: IssueFilter) =>
  invoke<Issue[]>('issue_list', { filter });

export const issueGet = (id: number) =>
  invoke<Issue>('issue_get', { id });

export const issueSetStatus = (id: number, status: string) =>
  invoke<Issue>('issue_set_status', { id, status });

export const issueSetPriority = (id: number, priority: string) =>
  invoke<Issue>('issue_set_priority', { id, priority });

export const issueUpdate = (
  id: number,
  input: {
    title?: string;
    description?: string | null;
    goal?: string | null;
    epic_id?: number | null;
  },
) =>
  invoke<Issue>('issue_update', {
    id,
    title: input.title ?? null,
    description: input.description ?? null,
    goal: input.goal ?? null,
    epic_id: input.epic_id ?? null,
  });

export const epicGet = (id: number) =>
  invoke<Epic>('epic_get', { id });

export const epicList = (project_key?: string, include_completed?: boolean) =>
  invoke<Epic[]>('epic_list', {
    project_key: project_key ?? null,
    include_completed: include_completed ?? null,
  });

/** 에픽의 sprint_id 를 변경. null 을 넘기면 백로그로 이동. 산하 이슈가 자동 상속 (ADR-0014). */
export const epicSetSprint = (epic_id: number, sprint_id: number | null) =>
  invoke<Epic>('epic_set_sprint', { epic_id, sprint_id });

export const sprintCurrent = () =>
  invoke<Sprint | null>('sprint_current');

export const sprintList = () =>
  invoke<Sprint[]>('sprint_list');

export const sprintProgressList = () =>
  invoke<SprintProgress[]>('sprint_progress_list');

export const sprintCreate = (input: CreateSprintInput) =>
  invoke<Sprint>('sprint_create', {
    name: input.name,
    goal: input.goal ?? null,
    start_date: input.start_date ?? null,
    end_date: input.end_date ?? null,
  });

export const sprintUpdate = (id: number, status?: string, name?: string, goal?: string) =>
  invoke<Sprint>('sprint_update', {
    id,
    name: name ?? null,
    goal: goal ?? null,
    status: status ?? null,
  });

export const sprintDelete = (id: number) =>
  invoke<void>('sprint_delete', { id });

export const taskList = (issue_id: number) =>
  invoke<Task[]>('task_list', { issue_id });

export const taskSetStatus = (id: number, status: string) =>
  invoke<Task>('task_set_status', { id, status });

export const noteList = (issue_id?: number | null, epic_id?: number | null, mission_id?: number | null) =>
  invoke<Note[]>('note_list', {
    issue_id: issue_id ?? null,
    epic_id: epic_id ?? null,
    mission_id: mission_id ?? null,
  });

export const noteGet = (id: number) =>
  invoke<Note>('note_get', { id });

export const noteAdd = (input: CreateNoteInput) =>
  invoke<Note>('note_add', { input });

export const noteResolve = (id: number) =>
  invoke<Note>('note_resolve', { id });

export const blockedIssuesGraph = (project_key: string) =>
  invoke<BlockingGraph>('blocked_issues_graph', { project_key });

export const blockingGraphForIssue = (issue_id: number) =>
  invoke<BlockingGraph>('blocking_graph_for_issue', { issue_id });

// ── MCP Supervisor ────────────────────────────────────────────────────────────

export const mcpStatus = () =>
  invoke<SupervisorStatusSnapshot>('mcp_status');

export const mcpStart = (port: number) =>
  invoke<SupervisorStatusSnapshot>('mcp_start', { port });

export const mcpStop = () =>
  invoke<SupervisorStatusSnapshot>('mcp_stop');

export const mcpRestart = (port: number) =>
  invoke<SupervisorStatusSnapshot>('mcp_restart', { port });

export const mcpRecentCalls = () =>
  invoke<CallRecord[]>('mcp_recent_calls');

export const mcpRecentLogs = () =>
  invoke<LogLine[]>('mcp_recent_logs');

export const mcpSetAutostart = (on: boolean) =>
  invoke<void>('mcp_set_autostart', { on });

// ── Activity Settings ─────────────────────────────────────────────────────────

export const getActivitySettings = () =>
  invoke<import('./types').ActivitySettings>('get_activity_settings');

export const setActivitySettings = (warn_minutes: number, stall_minutes: number) =>
  invoke<void>('set_activity_settings', { warn_minutes, stall_minutes });

export const getPromptSettings = () =>
  invoke<import('./types').PromptSettings>('get_prompt_settings');

export const setPromptSettings = (input: {
  issue_template?: string;
  epic_template?: string;
  mission_template?: string;
}) =>
  invoke<void>('set_prompt_settings', {
    issue_template: input.issue_template ?? null,
    epic_template: input.epic_template ?? null,
    mission_template: input.mission_template ?? null,
  });

// ── Dashboard CRUD ────────────────────────────────────────────────────────────

export const epicCreate = (input: CreateEpicInput) =>
  invoke<Epic>('epic_create', {
    project_key: input.project_key,
    title: input.title,
    description: input.description ?? null,
    mission_id: input.mission_id ?? null,
    sprint_id: input.sprint_id ?? null,
  });

export const issueCreate = (input: CreateIssueInput) =>
  invoke<Issue>('issue_create', {
    epic_id: input.epic_id,
    title: input.title,
    description: input.description ?? null,
    goal: input.goal ?? null,
    priority: input.priority ?? null,
  });

export const taskCreate = (input: CreateTaskInput) =>
  invoke<Task>('task_create', { issue_id: input.issue_id, title: input.title });

export const taskDelete = (id: number) =>
  invoke<void>('task_delete', { id });

export const issueLink = (source_id: number, target_id: number, link_type: LinkType) =>
  invoke<IssueLink>('issue_link', { source_id, target_id, link_type });

export const issueUnlink = (link_id: number) =>
  invoke<void>('issue_unlink', { link_id });

export const issueLinks = (issue_id: number) =>
  invoke<IssueLink[]>('issue_links', { issue_id });

export const epicSetStatus = (id: number, status: EpicStatus) =>
  invoke<Epic>('epic_set_status', { id, status });

export const epicUpdate = (
  id: number,
  input: {
    title?: string;
    description?: string | null;
    status?: EpicStatus;
    mission_id?: number | null;
    sprint_id?: number | null;
    update_sprint_id?: boolean;
  },
) =>
  invoke<Epic>('epic_update', {
    id,
    title: input.title ?? null,
    description: input.description ?? null,
    status: input.status ?? null,
    mission_id: input.mission_id !== undefined ? input.mission_id : null,
    sprint_id: input.sprint_id !== undefined ? input.sprint_id : null,
    update_sprint_id: input.update_sprint_id ?? null,
  });

export const epicDelete = (id: number) =>
  invoke<void>('epic_delete', { id });

export const issueDelete = (id: number) =>
  invoke<void>('issue_delete', { id });

export const historyList = (entity_type: string, entity_id: number) =>
  invoke<HistoryEntry[]>('history_list', { entity_type, entity_id });

// ── Mission IPC ───────────────────────────────────────────────────────────────

export const missionList = (include_completed?: boolean) =>
  invoke<Mission[]>('mission_list', {
    include_completed: include_completed ?? null,
  });

export const missionCreate = (input: CreateMissionInput) =>
  invoke<Mission>('mission_create', {
    title: input.title,
    description: input.description ?? null,
    jira_key: input.jira_key ?? null,
  });

export const missionGet = (id: number) =>
  invoke<Mission>('mission_get', { id });

export const missionUpdate = (id: number, input: UpdateMissionInput) =>
  invoke<Mission>('mission_update', {
    id,
    title: input.title ?? null,
    description: input.description ?? null,
    jira_key: input.jira_key ?? null,
    status: input.status ?? null,
  });

export const missionDelete = (id: number) =>
  invoke<void>('mission_delete', { id });

export const missionGetProgress = (id: number) =>
  invoke<MissionProgress>('mission_get_progress', { id });

export const missionGetTree = (id: number) =>
  invoke<MissionTree>('mission_get_tree', { id });

// ── App lifecycle ─────────────────────────────────────────────────────────────

export const getAppVersion = () =>
  invoke<string>('get_app_version');

export const relaunchApp = () =>
  invoke<void>('relaunch_app');

export const mcpGetToolDefinitions = () =>
  invoke<McpToolDefinition[]>('mcp_get_tool_definitions');

// ── Retrospective ──────────────────────────────────────────────────────────────

export const retrospectiveList = (project_key?: string, sprint_id?: number, limit?: number) =>
  invoke<RetrospectiveWithItems[]>('retrospective_list', {
    project_key: project_key ?? null,
    sprint_id: sprint_id ?? null,
    limit: limit ?? null,
  });

export const retrospectiveGet = (id: number) =>
  invoke<RetrospectiveWithItems>('retrospective_get', { id });

export const retrospectiveCreate = (input: CreateRetrospectiveInput) =>
  invoke<RetrospectiveWithItems>('retrospective_create', { input });

export const retrospectiveUpdate = (id: number, input: UpdateRetrospectiveInput) =>
  invoke<Retrospective>('retrospective_update', { id, input });

export const retrospectiveDelete = (id: number) =>
  invoke<void>('retrospective_delete', { id });

export const retroActionItemCreate = (retro_id: number, input: CreateRetroActionItemInput) =>
  invoke<RetroActionItem>('retro_action_item_create', { retro_id, input });

export const retroActionItemUpdate = (id: number, input: UpdateRetroActionItemInput) =>
  invoke<RetroActionItem>('retro_action_item_update', { id, input });

export const retroActionItemDelete = (id: number) =>
  invoke<void>('retro_action_item_delete', { id });

export const retroActionItemConvertToIssue = (id: number, agent_id?: string) =>
  invoke<Issue>('retro_action_item_convert_to_issue', { id, agent_id: agent_id ?? null });
