-- 0015_retrospectives.sql

-- 회고 본문 및 컨텍스트
CREATE TABLE IF NOT EXISTS retrospectives (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    project_key TEXT    NOT NULL,
    title       TEXT    NOT NULL,
    content     TEXT    NOT NULL,
    sprint_id   INTEGER REFERENCES sprints(id) ON DELETE SET NULL,
    mission_id  INTEGER REFERENCES missions(id) ON DELETE SET NULL,
    epic_id     INTEGER REFERENCES epics(id) ON DELETE SET NULL,
    agent_id    TEXT,
    created_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- 회고 액션 아이템
CREATE TABLE IF NOT EXISTS retro_action_items (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    retro_id        INTEGER NOT NULL REFERENCES retrospectives(id) ON DELETE CASCADE,
    title           TEXT    NOT NULL,
    description     TEXT,
    status          TEXT    NOT NULL DEFAULT 'open'
                    CHECK(status IN ('open', 'completed', 'cancelled')),
    linked_issue_id INTEGER REFERENCES issues(id) ON DELETE SET NULL,
    linked_note_id  INTEGER REFERENCES notes(id) ON DELETE SET NULL,
    ord             REAL    NOT NULL DEFAULT 1.0,
    created_at      TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_retrospectives_project ON retrospectives(project_key);
CREATE INDEX IF NOT EXISTS idx_retrospectives_sprint ON retrospectives(sprint_id);
CREATE INDEX IF NOT EXISTS idx_retro_action_items_retro ON retro_action_items(retro_id);
