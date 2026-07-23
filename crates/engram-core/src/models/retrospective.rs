use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Retrospective {
    pub id: i64,
    pub project_key: String,
    pub title: String,
    pub content: String,
    pub sprint_id: Option<i64>,
    pub mission_id: Option<i64>,
    pub epic_id: Option<i64>,
    pub agent_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RetroActionItem {
    pub id: i64,
    pub retro_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub linked_issue_id: Option<i64>,
    pub linked_note_id: Option<i64>,
    pub ord: f64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrospectiveWithItems {
    #[serde(flatten)]
    pub retro: Retrospective,
    pub action_items: Vec<RetroActionItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRetrospectiveInput {
    pub project_key: String,
    pub title: String,
    pub content: String,
    pub sprint_id: Option<i64>,
    pub mission_id: Option<i64>,
    pub epic_id: Option<i64>,
    pub agent_id: Option<String>,
    pub action_items: Option<Vec<CreateRetroActionItemInput>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateRetrospectiveInput {
    pub title: Option<String>,
    pub content: Option<String>,
    pub sprint_id: Option<i64>,
    pub mission_id: Option<i64>,
    pub epic_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRetroActionItemInput {
    pub title: String,
    pub description: Option<String>,
    pub linked_issue_id: Option<i64>,
    pub linked_note_id: Option<i64>,
    pub ord: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateRetroActionItemInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub linked_issue_id: Option<i64>,
    pub linked_note_id: Option<i64>,
    pub ord: Option<f64>,
}
