use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Issue {
    pub id: i64,
    pub epic_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub status: IssueStatus,
    pub priority: IssuePriority,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum IssueStatus {
    Draft,
    Approved,
    Todo,
    InProgress,
    InReview,
    Done,
    Cancelled,
}

impl IssueStatus {
    /// draft → approved → todo → in_progress → in_review → done
    pub fn can_transition_to(&self, next: &IssueStatus) -> bool {
        use IssueStatus::*;
        matches!(
            (self, next),
            (Draft, Approved)
                | (Approved, Todo)
                | (Todo, InProgress)
                | (InProgress, InReview)
                | (InProgress, Done)
                | (InReview, Done)
                | (_, Cancelled)
        )
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
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<IssuePriority>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateIssueInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<IssueStatus>,
    pub priority: Option<IssuePriority>,
}

/// issue_list 필터
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IssueFilter {
    pub epic_id: Option<i64>,
    pub project_key: Option<String>,
    pub status: Option<IssueStatus>,
    pub priority: Option<IssuePriority>,
}
