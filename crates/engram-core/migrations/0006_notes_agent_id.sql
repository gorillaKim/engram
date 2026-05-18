-- migrations/0006_notes_agent_id.sql
-- Purpose: notes.agent_id (TEXT NULL) 컬럼 추가.
--
-- 기존 author 는 역할 버킷('agent'|'user') 으로 굳어졌으므로, 인스턴스 식별을
-- 별도 컬럼으로 분리한다. 멀티 LLM (claude / codex / gemini 동시 운영) 환경에서
-- 어느 워커가 어떤 노트를 남겼는지 추적 가능해진다.
--
-- ADR-0009 의 agent_id 1급 시민화 원칙을 note 영역까지 확장.

ALTER TABLE notes ADD COLUMN agent_id TEXT;

CREATE INDEX IF NOT EXISTS idx_notes_agent_id
    ON notes(agent_id)
    WHERE agent_id IS NOT NULL;
