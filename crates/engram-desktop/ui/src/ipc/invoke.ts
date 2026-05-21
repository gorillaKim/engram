import { invoke } from '@tauri-apps/api/core';
import type {
  SessionSnapshot, IssueBoardStatus, Issue, Epic, Sprint,
  IssueFilter, Task, Note, CreateNoteInput, BlockingGraph,
  SupervisorStatusSnapshot, CallRecord,
  CreateEpicInput, CreateIssueInput, CreateTaskInput,
  IssueLink, LinkType, EpicStatus, HistoryEntry, CreateSprintInput,
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
  input: { title?: string; description?: string | null; goal?: string | null },
) =>
  invoke<Issue>('issue_update', {
    id,
    title: input.title ?? null,
    description: input.description ?? null,
    goal: input.goal ?? null,
  });

export const epicList = (project_key?: string) =>
  invoke<Epic[]>('epic_list', { project_key: project_key ?? null });

/** 이슈의 sprint_id 를 변경. null 을 넘기면 백로그로 이동. */
export const issueSetSprint = (id: number, sprint_id: number | null) =>
  invoke<Issue>('issue_set_sprint', { id, sprint_id });

export const sprintCurrent = () =>
  invoke<Sprint | null>('sprint_current');

export const sprintList = () =>
  invoke<Sprint[]>('sprint_list');

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

export const noteList = (issue_id: number) =>
  invoke<Note[]>('note_list', { issue_id });

export const noteGet = (id: number) =>
  invoke<Note>('note_get', { id });

export const noteAdd = (input: CreateNoteInput) =>
  invoke<Note>('note_add', { input });

export const noteResolve = (id: number) =>
  invoke<Note>('note_resolve', { id });

export const blockedIssuesGraph = (project_key: string) =>
  invoke<BlockingGraph>('blocked_issues_graph', { project_key });

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

export const mcpSetAutostart = (on: boolean) =>
  invoke<void>('mcp_set_autostart', { on });

// ── Dashboard CRUD ────────────────────────────────────────────────────────────

export const epicCreate = (input: CreateEpicInput) =>
  invoke<Epic>('epic_create', {
    project_key: input.project_key,
    title: input.title,
    description: input.description ?? null,
  });

export const issueCreate = (input: CreateIssueInput) =>
  invoke<Issue>('issue_create', {
    epic_id: input.epic_id,
    sprint_id: input.sprint_id ?? null,
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
  input: { title?: string; description?: string | null; status?: EpicStatus },
) =>
  invoke<Epic>('epic_update', {
    id,
    title: input.title ?? null,
    description: input.description ?? null,
    status: input.status ?? null,
  });

export const epicDelete = (id: number) =>
  invoke<void>('epic_delete', { id });

export const issueDelete = (id: number) =>
  invoke<void>('issue_delete', { id });

export const historyList = (entity_type: string, entity_id: number) =>
  invoke<HistoryEntry[]>('history_list', { entity_type, entity_id });

// ── App lifecycle ─────────────────────────────────────────────────────────────

export const getAppVersion = () =>
  invoke<string>('get_app_version');

export const relaunchApp = () =>
  invoke<void>('relaunch_app');
