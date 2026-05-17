-- migrations/0002_epic_sprint_nullable.sql
-- Purpose: epics.sprint_id 를 NULL 허용으로 변경 (백로그 지원: 스프린트 미지정 상태)
-- SQLite 는 ALTER COLUMN 을 지원하지 않으므로 테이블을 재생성한다.
-- DROP/RENAME 시 issues.epic_id FK 는 테이블명으로 다시 바인딩되므로 무결성은 유지된다.

CREATE TABLE epics_new (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    sprint_id   INTEGER REFERENCES sprints(id) ON DELETE RESTRICT,
    project_key TEXT    NOT NULL,
    title       TEXT    NOT NULL,
    description TEXT,
    status      TEXT    NOT NULL DEFAULT 'active'
                CHECK(status IN ('active','completed','cancelled')),
    created_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO epics_new (id, sprint_id, project_key, title, description, status, created_at, updated_at)
SELECT id, sprint_id, project_key, title, description, status, created_at, updated_at FROM epics;

DROP TABLE epics;
ALTER TABLE epics_new RENAME TO epics;
