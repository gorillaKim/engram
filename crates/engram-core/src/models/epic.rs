use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Epic {
    pub id: i64,
    pub project_key: String,
    pub mission_id: Option<i64>,
    pub title: String,
    pub description: Option<String>,
    pub status: EpicStatus,
    pub created_at: String,
    pub updated_at: String,
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
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateEpicInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<EpicStatus>,
    /// 미션 변경. Some(id)이면 에픽의 mission_id를 업데이트한다.
    /// cascade_issues=true(기본)이면 하위 이슈 mission_id도 함께 갱신.
    pub mission_id: Option<i64>,
    /// true(기본): 하위 이슈 mission_id도 cascade 갱신
    pub cascade_issues: bool,
}
