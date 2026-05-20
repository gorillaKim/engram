import { describe, it, expect } from 'vitest';
import { toggleIssueSelection, toggleAllIssuesInEpic } from './bulkHelper';

describe('bulkHelper 테스트 (TDD)', () => {
  it('단일 이슈 선택 토글이 정상 동작해야 한다', () => {
    let selected: number[] = [1, 2];
    
    // 추가
    selected = toggleIssueSelection(selected, 3);
    expect(selected).toEqual([1, 2, 3]);

    // 제거
    selected = toggleIssueSelection(selected, 2);
    expect(selected).toEqual([1, 3]);
  });

  it('에픽 내 일괄 선택/해제가 정상 동작해야 한다', () => {
    const epicIssues = [10, 20, 30];
    let selected: number[] = [1, 10];

    // 일괄 추가 (중복 없이)
    selected = toggleAllIssuesInEpic(selected, epicIssues, true);
    expect(selected.sort()).toEqual([1, 10, 20, 30].sort());

    // 일괄 제거
    selected = toggleAllIssuesInEpic(selected, epicIssues, false);
    expect(selected).toEqual([1]);
  });
});
