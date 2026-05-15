import { invoke } from '@tauri-apps/api/core';
import type {
  SessionSnapshot, IssueBoardStatus, Issue, Epic, Sprint,
  IssueFilter, Task, Note, CreateNoteInput, BlockingGraph,
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

export const epicList = (project_key?: string) =>
  invoke<Epic[]>('epic_list', { project_key: project_key ?? null });

export const sprintCurrent = () =>
  invoke<Sprint | null>('sprint_current');

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
