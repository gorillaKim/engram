-- migrations/0007_notes_scope.sql
-- Purpose: notes 에 broadcast scope 도입.
--
-- 기존 notes.issue_id NOT NULL 을 NULL 허용으로 완화하고, scope (TEXT) +
-- scope_target_id (INTEGER) + project_key (TEXT) 컬럼을 추가한다.
--
-- 의미:
--   scope='issue'   → scope_target_id = issue_id (기존 동작 유지, 백필됨)
--   scope='task'    → scope_target_id = task_id (issue_id 도 함께 채움)
--   scope='epic'    → scope_target_id = epic_id
--   scope='sprint'  → scope_target_id = sprint_id
--   scope='project' → scope_target_id NULL, project_key 필수
--
-- session_restore 가 활성 sprint/epic/project 광역 노트를 함께 노출하여
-- sprint freeze / 프로젝트 정책 같은 공지를 한 번만 등록하면 모든 이슈에 전파된다.
--
-- SQLite 는 ALTER 로 NOT NULL 완화 불가 → 0002/0004 패턴(테이블 재생성).

CREATE TABLE notes_new (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id    INTEGER REFERENCES issues(id) ON DELETE CASCADE,
    task_id     INTEGER REFERENCES tasks(id) ON DELETE SET NULL,
    note_type   TEXT    NOT NULL
                CHECK(note_type IN ('caveat','decision','discovery','blocker_detail','context','reference','comment')),
    summary     TEXT    NOT NULL,
    detail      TEXT,
    author      TEXT    NOT NULL DEFAULT 'agent',
    agent_id    TEXT,
    resolved    INTEGER NOT NULL DEFAULT 0,
    -- broadcast scope:
    scope           TEXT    NOT NULL DEFAULT 'issue'
                    CHECK(scope IN ('project','sprint','epic','issue','task')),
    scope_target_id INTEGER,
    project_key     TEXT,
    created_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    resolved_at TEXT
);

-- 기존 데이터는 모두 scope='issue', scope_target_id=issue_id 로 백필
INSERT INTO notes_new (id, issue_id, task_id, note_type, summary, detail, author, agent_id, resolved, scope, scope_target_id, project_key, created_at, resolved_at)
SELECT id, issue_id, task_id, note_type, summary, detail, author, agent_id, resolved, 'issue', issue_id, NULL, created_at, resolved_at
FROM notes;

DROP TABLE notes;
ALTER TABLE notes_new RENAME TO notes;

-- 기존 인덱스 재생성
CREATE INDEX IF NOT EXISTS idx_notes_issue        ON notes(issue_id, resolved);
CREATE INDEX IF NOT EXISTS idx_notes_type         ON notes(issue_id, note_type);
CREATE INDEX IF NOT EXISTS idx_notes_agent_id     ON notes(agent_id) WHERE agent_id IS NOT NULL;

-- broadcast scope 조회용 인덱스
CREATE INDEX IF NOT EXISTS idx_notes_scope        ON notes(scope, scope_target_id, resolved);
CREATE INDEX IF NOT EXISTS idx_notes_project_scope ON notes(scope, project_key, resolved) WHERE scope = 'project';
