-- migrations/0011_epic_owns_sprint.sql
-- requires SQLite >= 3.35
-- Purpose: ADR-0014 — Sprint SSOT 를 Mission 에서 Epic 으로 이관.
--   * Mission 은 sprint-agnostic 한 thematic grouping 으로 축소.
--   * Epic 이 sprint_id 를 직접 보유한다 (실제 SSOT).
--   * Issue 의 mission_id 컬럼은 제거되고 Epic 을 통해 derive 한다.
--   * Issue.sprint_id 는 응답 호환성을 위해 derived (LEFT JOIN epics.sprint_id) 로 유지.

-- 1) epics.sprint_id 컬럼 추가
ALTER TABLE epics ADD COLUMN sprint_id INTEGER REFERENCES sprints(id) ON DELETE SET NULL;

-- 2) 인덱스 추가
CREATE INDEX IF NOT EXISTS idx_epics_sprint ON epics(sprint_id);

-- 3) 백필 — 각 epic 의 sprint_id 를 소속 mission.sprint_id 로 채운다.
UPDATE epics
SET sprint_id = (SELECT m.sprint_id FROM missions m WHERE m.id = epics.mission_id)
WHERE mission_id IS NOT NULL;

-- 4) 더 이상 쓰지 않는 인덱스 정리
DROP INDEX IF EXISTS idx_missions_sprint;
DROP INDEX IF EXISTS idx_issues_mission;

-- 5) 컬럼 제거 (SQLite 3.35+)
ALTER TABLE missions DROP COLUMN sprint_id;
ALTER TABLE issues   DROP COLUMN mission_id;
