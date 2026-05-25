import { describe, it, expect } from 'vitest';
import { getUnfinishedIssues, getUnfinishedEpics } from './sprintCompleteHelper';
import type { Issue } from '../ipc/types';

describe('sprintCompleteHelper 테스트 (TDD)', () => {
  const mockIssues: Partial<Issue>[] = [
    { id: 101, title: '이슈 1', status: 'ready', epic_id: 10 },
    { id: 102, title: '이슈 2', status: 'finished', epic_id: 10 },
    { id: 103, title: '이슈 3', status: 'working', epic_id: 20 },
    { id: 104, title: '이슈 4', status: 'cancelled', epic_id: 30 },
    { id: 105, title: '이슈 5', status: 'ready', epic_id: 20 }, // 20번 에픽 중복
  ];

  it('finished 와 cancelled 가 아닌 미완료 이슈만 잘 추출해야 한다', () => {
    const result = getUnfinishedIssues(mockIssues as Issue[]);
    expect(result.length).toBe(3); // 101, 103, 105
    expect(result.find(i => i.id === 101)).toBeDefined();
    expect(result.find(i => i.id === 103)).toBeDefined();
    expect(result.find(i => i.id === 105)).toBeDefined();
  });

  it('미완료 이슈들로부터 고유한 epic_id들을 추출해야 한다', () => {
    const result = getUnfinishedEpics(mockIssues as Issue[]);
    // 미완료 이슈(101, 103, 105)의 epic_id는 [10, 20, 20] -> 고유 결과 [10, 20]
    expect(result.length).toBe(2);
    expect(result).toContain(10);
    expect(result).toContain(20);
    expect(result).not.toContain(30); // 104는 cancelled이므로 이관 대상 아님
  });
});
