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

    /// 에픽을 삭제한다. 하위 이슈/태스크/노트/링크까지 모두 cascade 삭제한다.
    ///
    /// 스키마상 `issues.epic_id ON DELETE RESTRICT`, `tasks.issue_id ON DELETE RESTRICT`
    /// 라서 단순 DELETE 는 막힌다. 트랜잭션 내에서 task_tests → tasks → notes/links → issues → epic
    /// 순으로 명시 삭제한다.
    pub async fn epic_delete(&self, id: i64, changed_by: &str) -> Result<()> {
        let epic = self.epic_get(id).await?; // 존재 확인

        let mut tx = self.pool.begin().await?;

        // 1) 하위 이슈들의 task_tests
        sqlx::query(
            "DELETE FROM task_tests WHERE task_id IN (\
                 SELECT t.id FROM tasks t \
                 JOIN issues i ON i.id = t.issue_id \
                 WHERE i.epic_id = ?\
             )",
        )
        .bind(id)
        .execute(&mut *tx)
        .await?;

        // 2) 하위 이슈들의 태스크
        sqlx::query(
            "DELETE FROM tasks WHERE issue_id IN (SELECT id FROM issues WHERE epic_id = ?)",
        )
        .bind(id)
        .execute(&mut *tx)
        .await?;

        // 3) 하위 이슈들의 노트
        sqlx::query(
            "DELETE FROM notes WHERE issue_id IN (SELECT id FROM issues WHERE epic_id = ?)",
        )
        .bind(id)
        .execute(&mut *tx)
        .await?;

        // 4) 하위 이슈들의 링크 (source 또는 target 어느 쪽이든)
        sqlx::query(
            "DELETE FROM issue_links \
             WHERE source_id IN (SELECT id FROM issues WHERE epic_id = ?) \
                OR target_id IN (SELECT id FROM issues WHERE epic_id = ?)",
        )
        .bind(id)
        .bind(id)
        .execute(&mut *tx)
        .await?;

        // 5) 하위 이슈 삭제
        sqlx::query("DELETE FROM issues WHERE epic_id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        // 6) 에픽 자체 삭제
        sqlx::query("DELETE FROM epics WHERE id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        // 7) history 기록 (deletion marker)
        sqlx::query(
            "INSERT INTO history (entity_type, entity_id, field, old_value, new_value, changed_by) VALUES ('epic', ?, 'deleted', ?, NULL, ?)",
        )
        .bind(id)
        .bind(epic.title)
        .bind(changed_by)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
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
