use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Task {
    pub id: i64,
    pub issue_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub goal: Option<String>,
    pub status: TaskStatus,
    pub ord: f64,    // fractional index (order는 SQL 예약어)
    pub source: TaskSource,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Required,
    Ready,
    Working,
    Demo,
    Finished,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TaskSource {
    Planned,
    AgentDiscovered,
    UserAdded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskInput {
    pub issue_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub goal: Option<String>,
    pub after_task_id: Option<i64>, // None이면 마지막에 추가
    pub source: Option<TaskSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateTaskInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub goal: Option<String>,
    pub status: Option<TaskStatus>,
}

/// task_next 반환값 — Agent가 선택 근거를 인지할 수 있도록 reason 포함
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NextTask {
    pub task_id: i64,
    pub task_title: String,
    pub issue_id: i64,
    pub issue_title: String,
    pub epic_id: i64,
    pub epic_title: String,
    pub project_key: String,
    pub reason: String, // 예: "priority:high + no_blockers"
}
