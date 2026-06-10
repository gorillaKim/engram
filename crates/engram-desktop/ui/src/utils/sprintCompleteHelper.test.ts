import { describe, it, expect } from 'vitest';
import type { Epic, Issue } from '../ipc/types';
import { classifyEpics, getUnfinishedIssuesForEpic } from './sprintCompleteHelper';

describe('classifyEpics', () => {
  const createMockEpic = (id: number): Epic => ({
    id,
    project_key: 'test',
    mission_id: null,
    sprint_id: 1,
    title: `Epic #${id}`,
    description: null,
    status: 'active',
    created_at: '',
    updated_at: '',
  });

  const createMockIssue = (id: number, epicId: number, status: any): Issue => ({
    id,
    epic_id: epicId,
    mission_id: null,
    sprint_id: 1,
    title: `Issue #${id}`,
    description: null,
    goal: null,
    status,
    priority: 'medium',
    assigned_agent: null,
    created_at: '',
    updated_at: '',
  });

  it('하위 이슈가 모두 완료(finished/cancelled)된 에픽은 toComplete에 분류되어야 한다', () => {
    const epic1 = createMockEpic(1);
    const epic2 = createMockEpic(2);

    const issue1 = createMockIssue(101, 1, 'finished');
    const issue2 = createMockIssue(102, 1, 'cancelled');
    const issue3 = createMockIssue(103, 2, 'finished');

    const epics = [epic1, epic2];
    const issues = [issue1, issue2, issue3];

    const result = classifyEpics(epics, issues);

    expect(result.toComplete.map(e => e.id)).toContain(1);
    expect(result.toComplete.map(e => e.id)).toContain(2);
    expect(result.toTransfer).toHaveLength(0);
  });

  it('하위 이슈 중 미완료 이슈가 있는 에픽은 toTransfer에 분류되어야 한다', () => {
    const epic = createMockEpic(1);
    const issue1 = createMockIssue(101, 1, 'finished');
    const issue2 = createMockIssue(102, 1, 'working'); // 미완료

    const epics = [epic];
    const issues = [issue1, issue2];

    const result = classifyEpics(epics, issues);

    expect(result.toTransfer.map(e => e.id)).toContain(1);
    expect(result.toComplete).toHaveLength(0);
  });

  it('하위 이슈가 없는 빈 에픽은 toTransfer에 분류되어야 한다', () => {
    const epic = createMockEpic(1);

    const epics = [epic];
    const issues: Issue[] = [];

    const result = classifyEpics(epics, issues);

    expect(result.toTransfer.map(e => e.id)).toContain(1);
    expect(result.toComplete).toHaveLength(0);
  });
});

describe('getUnfinishedIssuesForEpic', () => {
  const createMockIssue = (id: number, epicId: number, status: any): Issue => ({
    id,
    epic_id: epicId,
    mission_id: null,
    sprint_id: 1,
    title: `Issue #${id}`,
    description: null,
    goal: null,
    status,
    priority: 'medium',
    assigned_agent: null,
    created_at: '',
    updated_at: '',
  });

  it('해당 에픽에 속한 이슈 중 미완료(finished/cancelled가 아닌) 이슈만 반환해야 한다', () => {
    const issue1 = createMockIssue(101, 1, 'ready');
    const issue2 = createMockIssue(102, 1, 'finished');
    const issue3 = createMockIssue(103, 1, 'working');
    const issue4 = createMockIssue(104, 2, 'ready'); // 다른 에픽

    const issues = [issue1, issue2, issue3, issue4];
    const result = getUnfinishedIssuesForEpic(1, issues);

    expect(result.map(i => i.id)).toEqual([101, 103]);
  });
});
