-- migrations/0013_add_evaluation_note_type.sql
-- Purpose: notes.note_type CHECK 제약에 'evaluation' 추가 및 기존 [EVALUATION] reference 노트 백필

CREATE TABLE notes_new (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id        INTEGER REFERENCES issues(id) ON DELETE CASCADE,
    task_id         INTEGER REFERENCES tasks(id) ON DELETE SET NULL,
    note_type       TEXT    NOT NULL
                    CHECK(note_type IN ('caveat','decision','discovery','blocker_detail','context','reference','comment','evaluation')),
    summary         TEXT    NOT NULL,
    detail          TEXT,
    author          TEXT    NOT NULL DEFAULT 'agent',
    agent_id        TEXT,
    resolved        INTEGER NOT NULL DEFAULT 0,
    scope           TEXT    NOT NULL DEFAULT 'issue'
                    CHECK(scope IN ('project','sprint','epic','issue','task')),
    scope_target_id INTEGER,
    project_key     TEXT,
    created_at      TEXT    NOT NULL DEFAULT (datetime('now')),
    resolved_at     TEXT,
    updated_at      TEXT    NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO notes_new (id, issue_id, task_id, note_type, summary, detail, author, agent_id, resolved, scope, scope_target_id, project_key, created_at, resolved_at, updated_at)
SELECT id, issue_id, task_id, note_type, summary, detail, author, agent_id, resolved, scope, scope_target_id, project_key, created_at, resolved_at, updated_at FROM notes;

DROP TABLE notes;
ALTER TABLE notes_new RENAME TO notes;

-- 기존 인덱스 재생성
CREATE INDEX IF NOT EXISTS idx_notes_issue        ON notes(issue_id, resolved);
CREATE INDEX IF NOT EXISTS idx_notes_type         ON notes(issue_id, note_type);
CREATE INDEX IF NOT EXISTS idx_notes_agent_id     ON notes(agent_id) WHERE agent_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_notes_scope        ON notes(scope, scope_target_id, resolved);
CREATE INDEX IF NOT EXISTS idx_notes_project_scope ON notes(scope, project_key, resolved) WHERE scope = 'project';

-- 기존 [EVALUATION] reference 노트 백필
UPDATE notes SET note_type = 'evaluation' WHERE note_type = 'reference' AND summary LIKE '[EVALUATION]%';
