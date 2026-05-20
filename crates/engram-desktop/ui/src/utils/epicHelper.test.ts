import { describe, it, expect } from 'vitest';
import { toggleAllEpics } from './epicHelper';

describe('epicHelper 테스트 (TDD)', () => {
  const mockEpicIds = [1, 2, 3, 4];

  it('모든 에픽을 일괄 펼침(true) 상태로 설정할 수 있어야 한다', () => {
    const map = toggleAllEpics(mockEpicIds, true);
    expect(map[1]).toBe(true);
    expect(map[2]).toBe(true);
    expect(map[3]).toBe(true);
    expect(map[4]).toBe(true);
  });

  it('모든 에픽을 일괄 접힘(false) 상태로 설정할 수 있어야 한다', () => {
    const map = toggleAllEpics(mockEpicIds, false);
    expect(map[1]).toBe(false);
    expect(map[2]).toBe(false);
    expect(map[3]).toBe(false);
    expect(map[4]).toBe(false);
  });

  it('빈 에픽 리스트에 대해 빈 객체를 리턴해야 한다', () => {
    const map = toggleAllEpics([], true);
    expect(Object.keys(map).length).toBe(0);
  });
});
