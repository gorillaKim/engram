-- migrations/0003_issue_sprint.sql
-- Purpose: 이슈에 sprint_id 컬럼 추가 (이슈가 스프린트에 직접 소속됨)
--
-- epics.sprint_id 는 남겨 두되 애플리케이션 코드는 이를 무시한다.
-- (SQLite 는 DROP TABLE 시 FK 참조 테이블을 삭제할 수 없음)
--
-- ON DELETE 정책: 스프린트 삭제 시 이슈의 sprint_id 를 NULL 로 (백로그 이동).

-- 1) 이슈에 sprint_id 추가 (nullable — null = 백로그)
ALTER TABLE issues ADD COLUMN sprint_id INTEGER REFERENCES sprints(id) ON DELETE SET NULL;

-- 2) 기존 데이터 백필: 이슈는 자기 에픽의 sprint_id 를 물려받는다
UPDATE issues
SET sprint_id = (SELECT e.sprint_id FROM epics e WHERE e.id = issues.epic_id);

-- 인덱스 (스프린트별 이슈 조회 빈도가 높음)
CREATE INDEX IF NOT EXISTS idx_issues_sprint_id ON issues(sprint_id);
