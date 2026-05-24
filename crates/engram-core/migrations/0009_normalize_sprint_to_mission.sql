-- migrations/0009_normalize_sprint_to_mission.sql
-- Purpose: ADR-0013 Mission.sprint_id SSOT 확정 직전 백필.

-- 1) 이슈: mission_id 가 있는 경우 mission.sprint_id 로 갱신
UPDATE issues
SET sprint_id = (SELECT m.sprint_id FROM missions m WHERE m.id = issues.mission_id)
WHERE mission_id IS NOT NULL;

-- 2) 에픽: mission_id 가 있는 경우 mission.sprint_id 로 갱신
UPDATE epics
SET sprint_id = (SELECT m.sprint_id FROM missions m WHERE m.id = epics.mission_id)
WHERE mission_id IS NOT NULL;
