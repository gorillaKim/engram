import { describe, it, expect } from 'vitest';
import { getUnfinishedIssues } from './sprintCompleteHelper';
import type { Issue } from '../ipc/types';

describe('sprintCompleteHelper 테스트 (TDD)', () => {
  const mockIssues: Partial<Issue>[] = [
    { id: 101, title: '이슈 1', status: 'ready' },
    { id: 102, title: '이슈 2', status: 'finished' },
    { id: 103, title: '이슈 3', status: 'working' },
    { id: 104, title: '이슈 4', status: 'cancelled' },
  ];

  it('finished 와 cancelled 가 아닌 미완료 이슈만 잘 추출해야 한다', () => {
    const result = getUnfinishedIssues(mockIssues as Issue[]);
    expect(result.length).toBe(2);
    expect(result.find(i => i.id === 101)).toBeDefined();
    expect(result.find(i => i.id === 103)).toBeDefined();
    expect(result.find(i => i.id === 102)).toBeUndefined();
    expect(result.find(i => i.id === 104)).toBeUndefined();
  });
});
