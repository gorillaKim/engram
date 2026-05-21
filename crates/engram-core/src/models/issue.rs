use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Issue {
    pub id: i64,
    pub epic_id: i64,
    /// 소속 스프린트. None 이면 백로그 (Sprint↔Issue 직접 매핑 — ADR-0008 참고).
    pub sprint_id: Option<i64>,
    pub title: String,
    pub description: Option<String>,
    pub goal: Option<String>,
    pub status: IssueStatus,
    pub priority: IssuePriority,
    /// 현재 이슈를 잡고 있는 에이전트 식별자. working 상태일 때만 의미가 있으며,
    /// 다른 상태로 전이하면 NULL 로 비워진다. CAS 기반 `issue_claim` 의 lock 키.
    #[serde(default)]
    pub assigned_agent: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum IssueStatus {
    Required,
    Ready,
    Working,
    Demo,
    Finished,
    Cancelled,
}

impl IssueStatus {
    /// 사용자 / Agent 모두 임의의 상태로 자유롭게 전이할 수 있다.
    /// 권장 흐름은 required → ready → working → (demo →) finished 지만
    /// 칸반 UX 에서 카드를 양방향으로 옮길 수 있어야 하기 때문에 가드를 두지 않는다.
    ///
    /// Agent 가 demo → finished / *→ cancelled 를 호출하지 못하게 막는 책임은
    /// `.claude/rules/agent-demo-gate.md` (워커 에이전트 프롬프트) 가 진다 — 코드 강제 없음.
    pub fn can_transition_to(&self, _next: &IssueStatus) -> bool {
        true
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum IssuePriority {
    Critical,
    High,
    Medium,
    Low,
}

impl IssuePriority {
    /// 우선순위 숫자값 (task_next 정렬에 사용)
    pub fn order_value(&self) -> u8 {
        match self {
            IssuePriority::Critical => 0,
            IssuePriority::High => 1,
            IssuePriority::Medium => 2,
            IssuePriority::Low => 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct IssueLink {
    pub id: i64,
    pub source_id: i64,
    pub target_id: i64,
    pub link_type: LinkType,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum LinkType {
    Blocks,
    RelatesTo,
    Duplicates,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIssueInput {
    pub epic_id: i64,
    /// None 이면 백로그(스프린트 미지정).
    pub sprint_id: Option<i64>,
    pub title: String,
    pub description: Option<String>,
    pub goal: Option<String>,
    pub priority: Option<IssuePriority>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateIssueInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub goal: Option<String>,
    pub status: Option<IssueStatus>,
    pub priority: Option<IssuePriority>,
}

/// issue_list 필터
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IssueFilter {
    pub epic_id: Option<i64>,
    /// 특정 스프린트의 이슈만 (`Some`).
    pub sprint_id: Option<i64>,
    /// `true` 면 백로그(sprint_id IS NULL) 이슈만 — `sprint_id` 필터보다 우선.
    #[serde(default)]
    pub backlog_only: bool,
    pub project_key: Option<String>,
    pub status: Option<IssueStatus>,
    pub priority: Option<IssuePriority>,
}

/// `stalled_issues` 응답 — 특정 상태에서 threshold 분 이상 머문 이슈.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StalledIssue {
    pub id: i64,
    pub title: String,
    pub project_key: String,
    pub status: IssueStatus,
    pub priority: IssuePriority,
    /// 해당 상태로 진입한 시각 (history.created_at).
    /// history 가 없으면 issues.updated_at 으로 폴백.
    pub entered_status_at: String,
    /// 현재까지 그 상태에서 머문 시간(분).
    pub minutes_in_status: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuePlanningSnapshot {
    pub sprint_id: Option<i64>,
    pub sprint_name: Option<String>,
    pub issues: Vec<IssuePlanningItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuePlanningItem {
    pub id: i64,
    pub title: String,
    pub status: IssueStatus,
    pub priority: IssuePriority,
    pub description_excerpt: Option<String>,
    pub blockers: Vec<i64>,
    pub existing_context_note_count: i64,
    pub updated_at: String,
}

