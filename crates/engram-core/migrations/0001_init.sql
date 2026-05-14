-- Engram v0.1 — 초기 스키마
-- 단일 중앙 DB: 모든 프로젝트를 하나의 engram.db에서 관리
-- 프로젝트 구분: epics.project_key 컬럼

-- PRAGMAs are set in connection options (repository/mod.rs), not here
-- WAL mode, busy_timeout, foreign_keys are all handled at connect time

-- 스프린트 (시간 단위 관리)
CREATE TABLE IF NOT EXISTS sprints (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL,
    goal        TEXT,
    status      TEXT    NOT NULL DEFAULT 'planning'
                CHECK(status IN ('planning','active','completed','cancelled')),
    start_date  TEXT,
    end_date    TEXT,
    created_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- 에픽 (프로젝트 단위 작업 묶음)
-- project_key 로 프로젝트 구분 (별도 DB 파일 대신 단일 DB 내 필터링)
CREATE TABLE IF NOT EXISTS epics (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    sprint_id   INTEGER NOT NULL REFERENCES sprints(id) ON DELETE RESTRICT,
    project_key TEXT    NOT NULL,
    title       TEXT    NOT NULL,
    description TEXT,
    status      TEXT    NOT NULL DEFAULT 'active'
                CHECK(status IN ('active','completed','cancelled')),
    created_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- 이슈 (에픽 하위 구체적 작업 단위)
CREATE TABLE IF NOT EXISTS issues (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    epic_id     INTEGER NOT NULL REFERENCES epics(id) ON DELETE RESTRICT,
    title       TEXT    NOT NULL,
    description TEXT,
    status      TEXT    NOT NULL DEFAULT 'draft'
                CHECK(status IN ('draft','approved','todo','in_progress','in_review','done','cancelled')),
    priority    TEXT    NOT NULL DEFAULT 'medium'
                CHECK(priority IN ('critical','high','medium','low')),
    created_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- 이슈 간 관계 (단방향 저장)
-- ⚠️ blocked_by는 저장하지 않음: "A blocks B"만 저장하고
--    "B blocked_by A"는 WHERE target_id=B AND link_type='blocks'로 도출
CREATE TABLE IF NOT EXISTS issue_links (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id   INTEGER NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    target_id   INTEGER NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    link_type   TEXT    NOT NULL
                CHECK(link_type IN ('blocks','relates_to','duplicates')),
    created_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    UNIQUE(source_id, target_id, link_type)
);

-- 태스크 (이슈 하위 세부 실행 항목)
CREATE TABLE IF NOT EXISTS tasks (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id    INTEGER NOT NULL REFERENCES issues(id) ON DELETE RESTRICT,
    title       TEXT    NOT NULL,
    description TEXT,
    status      TEXT    NOT NULL DEFAULT 'todo'
                CHECK(status IN ('todo','in_progress','done','skipped')),
    ord         REAL    NOT NULL,   -- fractional index (order는 SQL 예약어 → ord)
    source      TEXT    NOT NULL DEFAULT 'planned'
                CHECK(source IN ('planned','agent_discovered','user_added')),
    created_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- Typed Notes (2단계 로딩: summary 항상 / detail 요청 시)
CREATE TABLE IF NOT EXISTS notes (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id    INTEGER NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    task_id     INTEGER REFERENCES tasks(id) ON DELETE SET NULL,
    note_type   TEXT    NOT NULL
                CHECK(note_type IN ('caveat','decision','discovery','blocker_detail','context','reference')),
    summary     TEXT    NOT NULL,   -- 한 줄 요약 (session_restore에서 항상 로드)
    detail      TEXT,               -- 상세 내용 (note_get 호출 시만 로드)
    author      TEXT    NOT NULL DEFAULT 'agent',
    resolved    INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    resolved_at TEXT
);

-- 이력 추적 (모든 상태 변경 자동 기록)
CREATE TABLE IF NOT EXISTS history (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_type TEXT    NOT NULL CHECK(entity_type IN ('sprint','epic','issue','task','note')),
    entity_id   INTEGER NOT NULL,
    field       TEXT    NOT NULL,
    old_value   TEXT,
    new_value   TEXT,
    changed_by  TEXT    NOT NULL DEFAULT 'agent',
    created_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- 인덱스
CREATE INDEX IF NOT EXISTS idx_epics_sprint    ON epics(sprint_id, status);
CREATE INDEX IF NOT EXISTS idx_epics_project   ON epics(project_key, status);
CREATE INDEX IF NOT EXISTS idx_issues_epic     ON issues(epic_id, status);
CREATE INDEX IF NOT EXISTS idx_tasks_issue     ON tasks(issue_id, ord);
CREATE INDEX IF NOT EXISTS idx_notes_issue     ON notes(issue_id, resolved);
CREATE INDEX IF NOT EXISTS idx_notes_type      ON notes(issue_id, note_type);
CREATE INDEX IF NOT EXISTS idx_links_source    ON issue_links(source_id);
CREATE INDEX IF NOT EXISTS idx_links_target    ON issue_links(target_id);
CREATE INDEX IF NOT EXISTS idx_history_entity  ON history(entity_type, entity_id);
