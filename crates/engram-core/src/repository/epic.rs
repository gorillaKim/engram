use crate::models::history::{CreateHistoryInput, EntityType};
use crate::models::epic::*;
use crate::{Db, Error, Result};

impl Db {
    pub async fn epic_create(&self, input: CreateEpicInput) -> Result<Epic> {
        sqlx::query_as::<_, Epic>(
            "INSERT INTO epics (project_key, title, description) VALUES (?, ?, ?)
             RETURNING id, project_key, title, description, status, created_at, updated_at",
        )
        .bind(&input.project_key)
        .bind(&input.title)
        .bind(&input.description)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn epic_get(&self, id: i64) -> Result<Epic> {
        sqlx::query_as::<_, Epic>(
            "SELECT id, project_key, title, description, status, created_at, updated_at FROM epics WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| Error::NotFound(format!("epic:{id}")))
    }

    /// 에픽 목록. 에픽은 sprint-agnostic 이므로 project_key 와 status 로만 필터한다.
    pub async fn epic_list(
        &self,
        project_key: Option<&str>,
        _status: Option<EpicStatus>,
    ) -> Result<Vec<Epic>> {
        let mut sql = "SELECT id, project_key, title, description, status, created_at, updated_at FROM epics WHERE 1=1".to_string();
        if project_key.is_some() { sql.push_str(" AND project_key = ?"); }
        sql.push_str(" ORDER BY id DESC");

        let mut q = sqlx::query_as::<_, Epic>(&sql);
        if let Some(p) = project_key { q = q.bind(p); }
        q.fetch_all(&self.pool).await.map_err(Into::into)
    }

    /// 에픽을 삭제한다. 이슈가 하나라도 연결되어 있으면 거부한다.
    pub async fn epic_delete(&self, id: i64) -> Result<()> {
        self.epic_get(id).await?; // 존재 확인

        let issue_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM issues WHERE epic_id = ?",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        if issue_count > 0 {
            return Err(Error::Validation(format!(
                "에픽에 이슈가 {issue_count}개 남아 있습니다. 먼저 이슈를 다른 에픽으로 옮기거나 삭제하세요."
            )));
        }

        sqlx::query("DELETE FROM epics WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn epic_update(&self, id: i64, input: UpdateEpicInput, changed_by: &str) -> Result<Epic> {
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
        if let Some(desc) = &input.description {
            sqlx::query("UPDATE epics SET description = ?, updated_at = datetime('now') WHERE id = ?")
                .bind(desc).bind(id).execute(&self.pool).await?;
            let _ = self.history_record(CreateHistoryInput {
                entity_type: EntityType::Epic,
                entity_id: id,
                field: "description".to_string(),
                old_value: None,
                new_value: Some(desc.clone()),
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
