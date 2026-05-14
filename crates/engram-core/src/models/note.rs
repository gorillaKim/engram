use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Note {
    pub id: i64,
    pub issue_id: i64,
    pub task_id: Option<i64>,
    pub note_type: NoteType,
    pub summary: String,
    pub detail: Option<String>, // note_get(id) 호출 시만 로드
    pub author: String,
    pub resolved: bool,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

/// session_restore 시 summary만 반환 — 토큰 절약 핵심
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteSummary {
    pub id: i64,
    pub note_type: NoteType,
    pub summary: String,
    pub task_id: Option<i64>,
    pub resolved: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum NoteType {
    Caveat,         // 주의사항/함정 — 같은 실수 반복 방지
    Decision,       // 의사결정 기록 — "왜 이렇게 했지?" 추적
    Discovery,      // 작업 중 발견한 사실
    BlockerDetail,  // 블로커 상세 원인
    Context,        // 다음 세션 인수인계 (session_restore 자동 로드)
    Reference,      // 외부 링크/문서 참조
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNoteInput {
    pub issue_id: i64,
    pub task_id: Option<i64>,
    pub note_type: NoteType,
    pub summary: String,
    pub detail: Option<String>,
    pub author: Option<String>,
}
