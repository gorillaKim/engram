-- migrations/0008_missions.sql
-- Purpose: M6 Mission 레이어 도입 — missions 테이블, epics/issues에 mission_id, history entity_type 확장, 기존 데이터 백필

-- 1단계: missions 테이블 생성
CREATE TABLE IF NOT EXISTS missions (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    jira_key    TEXT    UNIQUE,          -- Nullable: NULL 다중 허용(SQLite UNIQUE는 NULL을 별개 값으로 취급), 비-NULL 시만 충돌 검사
    title       TEXT    NOT NULL,
    description TEXT,
    status      TEXT    NOT NULL DEFAULT 'active'
                CHECK(status IN ('active','completed','cancelled')),
    sprint_id   INTEGER REFERENCES sprints(id) ON DELETE SET NULL,   -- NULL = 백로그
    created_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- 2단계: 기존 테이블 컬럼 추가
ALTER TABLE epics  ADD COLUMN mission_id INTEGER REFERENCES missions(id) ON DELETE SET NULL;
ALTER TABLE issues ADD COLUMN mission_id INTEGER REFERENCES missions(id) ON DELETE SET NULL;

-- 3단계: 인덱스 추가
CREATE INDEX IF NOT EXISTS idx_missions_sprint   ON missions(sprint_id);
CREATE INDEX IF NOT EXISTS idx_missions_jira_key ON missions(jira_key);
CREATE INDEX IF NOT EXISTS idx_epics_mission     ON epics(mission_id);
CREATE INDEX IF NOT EXISTS idx_issues_mission    ON issues(mission_id);

-- 4단계: history 테이블 entity_type CHECK 제약 확장 ('mission' 추가)
-- SQLite는 ALTER TABLE로 CHECK 제약 수정 불가 → rename 패턴 사용
-- DROP TABLE 시 idx_history_entity 인덱스는 자동 삭제됨 (별도 DROP INDEX 불필요)
CREATE TABLE IF NOT EXISTS history_new (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_type TEXT    NOT NULL CHECK(entity_type IN ('sprint','epic','issue','task','note','mission')),
    entity_id   INTEGER NOT NULL,
    field       TEXT    NOT NULL,
    old_value   TEXT,
    new_value   TEXT,
    changed_by  TEXT    NOT NULL DEFAULT 'agent',
    created_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);
INSERT INTO history_new SELECT * FROM history;
DROP TABLE history;
ALTER TABLE history_new RENAME TO history;

-- history 인덱스 재생성 (DROP TABLE 시 자동 삭제된 것)
CREATE INDEX IF NOT EXISTS idx_history_entity ON history(entity_type, entity_id);

-- 5단계: 백필 — 기존 에픽 project_key 별 placeholder mission 생성
-- 기존 mission_id가 NULL인 에픽의 project_key를 title로 하는 미션을 삽입
INSERT INTO missions(title, status)
    SELECT DISTINCT project_key, 'active'
    FROM epics
    WHERE project_key IS NOT NULL;

-- 에픽의 mission_id를 project_key 일치하는 미션으로 채움
UPDATE epics
    SET mission_id = (SELECT id FROM missions WHERE title = epics.project_key)
    WHERE mission_id IS NULL AND project_key IS NOT NULL;

-- 이슈의 mission_id를 부모 에픽의 mission_id에서 상속
UPDATE issues
    SET mission_id = (SELECT mission_id FROM epics WHERE epics.id = issues.epic_id)
    WHERE mission_id IS NULL;
