import { describe, it, expect } from 'vitest';
import { clampSidebarWidth } from './sidebarHelper';

describe('sidebarHelper 테스트 (TDD)', () => {
  it('최솟값(160)과 최댓값(450) 사이의 값은 그대로 반환해야 한다', () => {
    expect(clampSidebarWidth(200)).toBe(200);
    expect(clampSidebarWidth(300)).toBe(300);
  });

  it('160 미만의 값은 160으로 클램프 처리해야 한다', () => {
    expect(clampSidebarWidth(100)).toBe(160);
    expect(clampSidebarWidth(0)).toBe(160);
    expect(clampSidebarWidth(-50)).toBe(160);
  });

  it('450 초과의 값은 450으로 클램프 처리해야 한다', () => {
    expect(clampSidebarWidth(500)).toBe(450);
    expect(clampSidebarWidth(1000)).toBe(450);
  });

  it('커스텀 최소/최대 값을 지정했을 때 알맞게 처리해야 한다', () => {
    expect(clampSidebarWidth(150, 180, 400)).toBe(180);
    expect(clampSidebarWidth(450, 180, 400)).toBe(400);
  });
});
