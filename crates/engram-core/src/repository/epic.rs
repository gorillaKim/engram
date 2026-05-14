use crate::models::epic::*;
use crate::{Db, Error, Result};

impl Db {
    pub async fn epic_create(&self, input: CreateEpicInput) -> Result<Epic> {
        let id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO epics (sprint_id, project_key, title, description) VALUES (?, ?, ?, ?) RETURNING id",
        )
        .bind(input.sprint_id)
        .bind(&input.project_key)
        .bind(&input.title)
        .bind(&input.description)
        .fetch_one(&self.pool)
        .await?;
        self.epic_get(id).await
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

    pub async fn epic_update(&self, id: i64, input: UpdateEpicInput) -> Result<Epic> {
        if let Some(title) = &input.title {
            sqlx::query("UPDATE epics SET title = ?, updated_at = datetime('now') WHERE id = ?")
                .bind(title).bind(id).execute(&self.pool).await?;
        }
        if let Some(status) = &input.status {
            let s = serde_json::to_value(status).unwrap().as_str().unwrap().to_string();
            sqlx::query("UPDATE epics SET status = ?, updated_at = datetime('now') WHERE id = ?")
                .bind(s).bind(id).execute(&self.pool).await?;
        }
        self.epic_get(id).await
    }
}
