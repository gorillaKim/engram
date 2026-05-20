import { describe, it, expect } from 'vitest';
import { filterFinishedIssues } from './issueFilterHelper';
import type { Issue } from '../ipc/types';

describe('issueFilterHelper 테스트 (TDD)', () => {
  const mockIssues: Partial<Issue>[] = [
    { id: 1, title: '이슈 1', status: 'working' },
    { id: 2, title: '이슈 2', status: 'finished' },
    { id: 3, title: '이슈 3', status: 'cancelled' },
    { id: 4, title: '이슈 4', status: 'ready' },
  ];

  it('hide가 true일 때 finished 및 cancelled 이슈를 가려야 한다', () => {
    const result = filterFinishedIssues(mockIssues as Issue[], true);
    expect(result.length).toBe(2);
    expect(result.find(i => i.id === 1)).toBeDefined();
    expect(result.find(i => i.id === 4)).toBeDefined();
    expect(result.find(i => i.id === 2)).toBeUndefined();
    expect(result.find(i => i.id === 3)).toBeUndefined();
  });

  it('hide가 false일 때 모든 이슈가 그대로 노출되어야 한다', () => {
    const result = filterFinishedIssues(mockIssues as Issue[], false);
    expect(result.length).toBe(4);
  });
});
