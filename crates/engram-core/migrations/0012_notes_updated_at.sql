-- migrations/0012_notes_updated_at.sql
-- Purpose: notes 테이블에 updated_at 컬럼 추가

ALTER TABLE notes ADD COLUMN updated_at TEXT NOT NULL DEFAULT (datetime('now'));
