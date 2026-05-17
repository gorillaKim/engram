use crate::models::history::{CreateHistoryInput, EntityType};
use crate::models::epic::*;
use crate::{Db, Error, Result};

impl Db {
    pub async fn epic_create(&self, input: CreateEpicInput) -> Result<Epic> {
        sqlx::query_as::<_, Epic>(
            "INSERT INTO epics (sprint_id, project_key, title, description) VALUES (?, ?, ?, ?)
             RETURNING id, sprint_id, project_key, title, description, status, created_at, updated_at",
        )
        .bind(input.sprint_id)
        .bind(&input.project_key)
        .bind(&input.title)
        .bind(&input.description)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn epic_get(&self, id: i64) -> Result<Epic> {
        sqlx::query_as::<_, Epic>(
            "SELECT id, sprint_id, project_key, title, description, status, created_at, updated_at FROM epics WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| Error::NotFound(format!("epic:{id}")))
    }

    pub async fn epic_list(
        &self,
        sprint_id: Option<i64>,
        project_key: Option<&str>,
        _status: Option<EpicStatus>,
    ) -> Result<Vec<Epic>> {
        let mut sql = "SELECT id, sprint_id, project_key, title, description, status, created_at, updated_at FROM epics WHERE 1=1".to_string();
        if sprint_id.is_some()   { sql.push_str(" AND sprint_id = ?"); }
        if project_key.is_some() { sql.push_str(" AND project_key = ?"); }
        sql.push_str(" ORDER BY id DESC");

        let mut q = sqlx::query_as::<_, Epic>(&sql);
        if let Some(s) = sprint_id   { q = q.bind(s); }
        if let Some(p) = project_key { q = q.bind(p); }
        q.fetch_all(&self.pool).await.map_err(Into::into)
    }

    pub async fn epic_update(&self, id: i64, input: UpdateEpicInput, changed_by: &str) -> Result<Epic> {
        if let Some(sprint_id) = input.sprint_id {
            sqlx::query("UPDATE epics SET sprint_id = ?, updated_at = datetime('now') WHERE id = ?")
                .bind(sprint_id).bind(id).execute(&self.pool).await?;
            let _ = self.history_record(CreateHistoryInput {
                entity_type: EntityType::Epic,
                entity_id: id,
                field: "sprint_id".to_string(),
                old_value: None,
                new_value: Some(sprint_id.to_string()),
                changed_by: changed_by.to_string(),
            }).await;
        }
        if let Some(title) = &input.title {
            sqlx::query("UPDATE epics SET title = ?, updated_at = datetime('now') WHERE id = ?")
                .bind(title).bind(id).execute(&self.pool).await?;
            let _ = self.history_record(CreateHistoryInput {
                entity_type: EntityType::Epic,
                entity_id: id,
                field: "title".to_string(),
                old_value: None,
                new_value: Some(title.clone()),
                changed_by: changed_by.to_string(),
            }).await;
        }
        if let Some(status) = &input.status {
            let s = serde_json::to_value(status).unwrap().as_str().unwrap().to_string();
            sqlx::query("UPDATE epics SET status = ?, updated_at = datetime('now') WHERE id = ?")
                .bind(&s).bind(id).execute(&self.pool).await?;
            let _ = self.history_record(CreateHistoryInput {
                entity_type: EntityType::Epic,
                entity_id: id,
                field: "status".to_string(),
                old_value: None,
                new_value: Some(s),
                changed_by: changed_by.to_string(),
            }).await;
        }
        self.epic_get(id).await
    }
}
