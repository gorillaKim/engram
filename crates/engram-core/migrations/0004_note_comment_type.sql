-- migrations/0004_note_comment_type.sql
-- Purpose: notes.note_type CHECK 제약에 'comment' 추가.
-- SQLite 는 ALTER 로 CHECK 변경 불가 → 테이블 재생성 패턴 (0002 와 동일).
-- author 컬럼은 이미 0001 에서 존재하므로 변경 없음.

CREATE TABLE notes_new (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id    INTEGER NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    task_id     INTEGER REFERENCES tasks(id) ON DELETE SET NULL,
    note_type   TEXT    NOT NULL
                CHECK(note_type IN ('caveat','decision','discovery','blocker_detail','context','reference','comment')),
    summary     TEXT    NOT NULL,
    detail      TEXT,
    author      TEXT    NOT NULL DEFAULT 'agent',
    resolved    INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    resolved_at TEXT
);

INSERT INTO notes_new (id, issue_id, task_id, note_type, summary, detail, author, resolved, created_at, resolved_at)
SELECT id, issue_id, task_id, note_type, summary, detail, author, resolved, created_at, resolved_at FROM notes;

DROP TABLE notes;
ALTER TABLE notes_new RENAME TO notes;

CREATE INDEX IF NOT EXISTS idx_notes_issue ON notes(issue_id, resolved);
CREATE INDEX IF NOT EXISTS idx_notes_type  ON notes(issue_id, note_type);
