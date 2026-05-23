use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct History {
    pub id: i64,
    pub entity_type: EntityType,
    pub entity_id: i64,
    pub field: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub changed_by: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Sprint,
    Epic,
    Issue,
    Task,
    Note,
    Mission,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateHistoryInput {
    pub entity_type: EntityType,
    pub entity_id: i64,
    pub field: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub changed_by: String,
}
