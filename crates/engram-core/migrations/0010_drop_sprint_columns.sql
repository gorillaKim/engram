-- migrations/0010_drop_sprint_columns.sql
-- requires SQLite >= 3.35
-- Purpose: ADR-0013 — Mission.sprint_id SSOT 확정 후 deprecated 컬럼 제거.

-- 1) 인덱스 먼저 제거 (DROP COLUMN 전 의존성 정리)
DROP INDEX IF EXISTS idx_issues_sprint_id;

-- 2) SQLite 3.35+ DROP COLUMN
ALTER TABLE issues DROP COLUMN sprint_id;
ALTER TABLE epics  DROP COLUMN sprint_id;
