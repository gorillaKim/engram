import { describe, it, expect } from 'vitest';
import { sortHistoryIssues, paginateHistoryIssues, calculateHistoryStats } from './historyHelper';
import type { Issue } from '../ipc/types';

// Mock Issue 데이터 생성 헬퍼
function createMockIssue(id: number, title: string, priority: 'critical' | 'high' | 'medium' | 'low', updatedAt: string): Issue {
  return {
    id,
    epic_id: 1,
    mission_id: null,
    sprint_id: 1,
    title,
    description: '',
    goal: '',
    status: 'finished',
    priority,
    created_at: '2026-05-20 00:00:00',
    updated_at: updatedAt,
  };
}

describe('historyHelper 테스트 (TDD)', () => {
  const mockIssues: Issue[] = [
    createMockIssue(1, 'Apple Issue', 'medium', '2026-05-20 10:00:00'),
    createMockIssue(3, 'Banana Issue', 'critical', '2026-05-20 09:00:00'),
    createMockIssue(2, 'Cherry Issue', 'low', '2026-05-20 11:00:00'),
    createMockIssue(4, 'Durian Issue', 'high', '2026-05-20 08:30:00'),
  ];

  describe('sortHistoryIssues', () => {
    it('ID 기준으로 오름차순 및 내림차순 정렬이 되어야 한다', () => {
      const asc = sortHistoryIssues(mockIssues, 'id', 'asc');
      expect(asc[0].id).toBe(1);
      expect(asc[3].id).toBe(4);

      const desc = sortHistoryIssues(mockIssues, 'id', 'desc');
      expect(desc[0].id).toBe(4);
      expect(desc[3].id).toBe(1);
    });

    it('제목 기준으로 오름차순 및 내림차순 정렬이 되어야 한다', () => {
      const asc = sortHistoryIssues(mockIssues, 'title', 'asc');
      expect(asc[0].title).toBe('Apple Issue');
      expect(asc[3].title).toBe('Durian Issue');

      const desc = sortHistoryIssues(mockIssues, 'title', 'desc');
      expect(desc[0].title).toBe('Durian Issue');
      expect(desc[3].title).toBe('Apple Issue');
    });

    it('완료일(updated_at) 기준으로 오름차순 및 내림차순 정렬이 되어야 한다', () => {
      const asc = sortHistoryIssues(mockIssues, 'updated_at', 'asc');
      expect(asc[0].id).toBe(4); // 08:30:00
      expect(asc[3].id).toBe(2); // 11:00:00

      const desc = sortHistoryIssues(mockIssues, 'updated_at', 'desc');
      expect(desc[0].id).toBe(2); // 11:00:00
      expect(desc[3].id).toBe(4); // 08:30:00
    });

    it('우선순위(priority) 기준으로 가중치 정렬이 되어야 한다 (critical > high > medium > low)', () => {
      const asc = sortHistoryIssues(mockIssues, 'priority', 'asc'); // 낮은 우선순위부터
      expect(asc[0].priority).toBe('low');
      expect(asc[3].priority).toBe('critical');

      const desc = sortHistoryIssues(mockIssues, 'priority', 'desc'); // 높은 우선순위부터
      expect(desc[0].priority).toBe('critical');
      expect(desc[3].priority).toBe('low');
    });
  });

  describe('paginateHistoryIssues', () => {
    it('지정한 visibleCount 개수만큼 이슈를 반환해야 한다', () => {
      const paginated = paginateHistoryIssues(mockIssues, 2);
      expect(paginated.length).toBe(2);
      expect(paginated[0].id).toBe(1);
      expect(paginated[1].id).toBe(3);

      const all = paginateHistoryIssues(mockIssues, 10);
      expect(all.length).toBe(mockIssues.length);
    });
  });

  describe('calculateHistoryStats', () => {
    it('이슈 리스트의 총 개수 및 우선순위별 분포 통계를 정확히 집계해야 한다', () => {
      const stats = calculateHistoryStats(mockIssues);
      expect(stats.totalCount).toBe(4);
      expect(stats.priorityCounts.critical).toBe(1);
      expect(stats.priorityCounts.high).toBe(1);
      expect(stats.priorityCounts.medium).toBe(1);
      expect(stats.priorityCounts.low).toBe(1);

      const emptyStats = calculateHistoryStats([]);
      expect(emptyStats.totalCount).toBe(0);
      expect(emptyStats.priorityCounts.critical).toBe(0);
    });
  });
});
