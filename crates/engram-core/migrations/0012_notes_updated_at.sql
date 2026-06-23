-- migrations/0012_notes_updated_at.sql
-- Purpose: notes 테이블에 updated_at 컬럼 추가
-- NOTE: SQLite 의 ALTER TABLE ADD COLUMN 은 비상수 기본값(괄호 표현식, CURRENT_TIMESTAMP 등)을
--       허용하지 않는다("Cannot add a column with non-constant default"). 따라서 상수 기본값('')으로
--       컬럼을 추가한 뒤 기존 행을 created_at 으로 백필한다. 최종 NOT NULL DEFAULT (datetime('now'))
--       스키마는 0013 에서 notes 테이블 재생성 시 확정된다.

ALTER TABLE notes ADD COLUMN updated_at TEXT NOT NULL DEFAULT '';
UPDATE notes SET updated_at = created_at WHERE updated_at = '';
