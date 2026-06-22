use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Epic {
    pub id: i64,
    pub project_key: String,
    pub mission_id: Option<i64>,
    /// 소속 스프린트. None 이면 백로그. Epic 이 sprint SSOT (ADR-0014).
    pub sprint_id: Option<i64>,
    pub title: String,
    pub description: Option<String>,
    pub status: EpicStatus,
    pub created_at: String,
    pub updated_at: String,
    #[sqlx(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ready_to_complete: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum EpicStatus {
    Active,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEpicInput {
    pub project_key: String,
    pub mission_id: Option<i64>,
    /// 소속 스프린트 (선택). None 이면 백로그.
    pub sprint_id: Option<i64>,
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateEpicInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<EpicStatus>,
    /// 미션 변경. Some(id) 이면 에픽의 mission_id 를 업데이트한다 (sprint 와 무관).
    pub mission_id: Option<i64>,
    /// sprint 변경. update_sprint_id=true 일 때만 동작 (None 도 백로그로 명시 설정 가능).
    pub sprint_id: Option<i64>,
    /// true: sprint_id 필드를 적용한다 (None = 백로그). false 면 sprint_id 값 무시.
    #[serde(default)]
    pub update_sprint_id: bool,
}
