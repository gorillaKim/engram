use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Sprint {
    pub id: i64,
    pub name: String,
    pub goal: Option<String>,
    pub status: SprintStatus,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SprintStatus {
    Planning,
    Active,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSprintInput {
    pub name: String,
    pub goal: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateSprintInput {
    pub name: Option<String>,
    pub goal: Option<String>,
    pub status: Option<SprintStatus>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

impl SprintStatus {
    pub const ALL: &'static [&'static str] = &["planning", "active", "completed", "cancelled"];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sprint_status_all_match() {
        for &s in SprintStatus::ALL {
            let status: SprintStatus = serde_json::from_str(&format!("\"{}\"", s))
                .expect(&format!("failed to deserialize status: {}", s));
            let serialized = serde_json::to_string(&status).unwrap();
            assert_eq!(serialized, format!("\"{}\"", s));
        }
    }
}
