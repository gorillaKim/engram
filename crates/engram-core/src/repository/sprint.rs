use crate::models::history::{CreateHistoryInput, EntityType};
use crate::models::sprint::*;
use crate::{Db, Error, Result};

impl Db {
    pub async fn sprint_create(&self, input: CreateSprintInput) -> Result<Sprint> {
        let id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO sprints (name, goal, start_date, end_date) VALUES (?, ?, ?, ?) RETURNING id",
        )
        .bind(&input.name)
        .bind(&input.goal)
        .bind(&input.start_date)
        .bind(&input.end_date)
        .fetch_one(&self.pool)
        .await?;
        self.sprint_get(id).await
    }

    pub async fn sprint_get(&self, id: i64) -> Result<Sprint> {
        sqlx::query_as::<_, Sprint>(
            "SELECT id, name, goal, status, start_date, end_date, created_at, updated_at FROM sprints WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| Error::NotFound(format!("sprint:{id}")))
    }

    pub async fn sprint_list(&self, _status: Option<SprintStatus>) -> Result<Vec<Sprint>> {
        sqlx::query_as::<_, Sprint>(
            "SELECT id, name, goal, status, start_date, end_date, created_at, updated_at FROM sprints ORDER BY id DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn sprint_current(&self) -> Result<Option<Sprint>> {
        sqlx::query_as::<_, Sprint>(
            "SELECT id, name, goal, status, start_date, end_date, created_at, updated_at FROM sprints WHERE status = 'active' ORDER BY id DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn sprint_update(&self, id: i64, input: UpdateSprintInput, changed_by: &str) -> Result<Sprint> {
        if let Some(status) = &input.status {
            let s = serde_json::to_value(status).unwrap().as_str().unwrap().to_string();
            sqlx::query("UPDATE sprints SET status = ?, updated_at = datetime('now') WHERE id = ?")
                .bind(&s)
                .bind(id)
                .execute(&self.pool)
                .await?;
            let _ = self.history_record(CreateHistoryInput {
                entity_type: EntityType::Sprint,
                entity_id: id,
                field: "status".to_string(),
                old_value: None,
                new_value: Some(s),
                changed_by: changed_by.to_string(),
            }).await;
        }
        if let Some(name) = &input.name {
            sqlx::query("UPDATE sprints SET name = ?, updated_at = datetime('now') WHERE id = ?")
                .bind(name)
                .bind(id)
                .execute(&self.pool)
                .await?;
            let _ = self.history_record(CreateHistoryInput {
                entity_type: EntityType::Sprint,
                entity_id: id,
                field: "name".to_string(),
                old_value: None,
                new_value: Some(name.clone()),
                changed_by: changed_by.to_string(),
            }).await;
        }
        self.sprint_get(id).await
    }
}
