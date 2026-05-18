-- migrations/0005_issue_assigned_agent.sql
-- Purpose: 이슈에 assigned_agent (TEXT NULL) 컬럼 추가.
--
-- 멀티 에이전트 환경에서 어느 에이전트가 이슈를 잡고 있는지 식별 + working 전이의
-- CAS(Compare-And-Set) lock 키로 사용한다.
--
-- 의미:
--   NULL       — 잡힌 상태 아님
--   '<agent>'  — 해당 에이전트가 working 으로 점유 중
--
-- 이슈가 working 을 벗어나면 assigned_agent 를 NULL 로 비워야 한다
-- (issue_release / issue_update 핸들러에서 처리).

ALTER TABLE issues ADD COLUMN assigned_agent TEXT;

CREATE INDEX IF NOT EXISTS idx_issues_assigned_agent
    ON issues(assigned_agent)
    WHERE assigned_agent IS NOT NULL;
