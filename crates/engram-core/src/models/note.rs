use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Note {
    pub id: i64,
    /// scope='issue'/'task' 일 때만 NOT NULL. broadcast scope (project/sprint/epic) 에선 NULL.
    pub issue_id: Option<i64>,
    pub task_id: Option<i64>,
    pub note_type: NoteType,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>, // note_get(id) 호출 시만 로드
    pub author: String,
    /// 작성 에이전트 인스턴스 식별자 (예: "claude-opus@sess-abc").
    /// `author` 는 역할 버킷('agent'|'user'), `agent_id` 는 인스턴스 — 두 축을 분리.
    #[serde(default)]
    pub agent_id: Option<String>,
    pub resolved: bool,
    /// 노트 적용 범위. 'issue'/'task' 가 기본. 'project'/'sprint'/'epic' 은 broadcast.
    #[serde(default = "default_note_scope")]
    pub scope: NoteScope,
    /// scope 별 대상 ID — scope='project' 일 때는 NULL, 그 외에는 해당 entity 의 id.
    #[serde(default)]
    pub scope_target_id: Option<i64>,
    /// scope='project' 일 때만 의미. 그 외 NULL.
    #[serde(default)]
    pub project_key: Option<String>,
    pub created_at: String,
    pub resolved_at: Option<String>,
    pub updated_at: String,
}

fn default_note_scope() -> NoteScope { NoteScope::Issue }

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum NoteScope {
    Project,
    Sprint,
    Epic,
    Issue,
    Task,
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
    Comment,        // 사용자/에이전트 간 코멘트 — 데스크톱 CommentSection 으로 노출
    Evaluation,     // 평가/회고 피드백
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNoteInput {
    /// scope='issue'/'task' 일 때 필수 (>0), broadcast scope (project/sprint/epic) 에선 0 또는 무시됨.
    /// 하위 호환을 위해 i64 유지 — `scope` 가 broadcast 이면 NULL 로 저장된다.
    pub issue_id: i64,
    pub task_id: Option<i64>,
    pub note_type: NoteType,
    pub summary: String,
    pub detail: Option<String>,
    pub author: Option<String>,
    /// 작성 에이전트 인스턴스 식별자. `author` 와 별도 — `author` 는 역할(agent|user),
    /// `agent_id` 는 인스턴스(claude-opus@A 등). 멀티 LLM 환경에서 누가 남겼는지 추적용.
    #[serde(default)]
    pub agent_id: Option<String>,
    /// 노트 scope. 생략 시 issue (issue_id 필수) 또는 task (task_id 필수) 자동 판정.
    #[serde(default)]
    pub scope: Option<NoteScope>,
    /// scope 별 대상 ID. project scope 면 None, 그 외엔 대상 entity id.
    #[serde(default)]
    pub scope_target_id: Option<i64>,
    /// project scope 일 때 필수.
    #[serde(default)]
    pub project_key: Option<String>,
}
