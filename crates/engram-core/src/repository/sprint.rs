use crate::models::history::{CreateHistoryInput, EntityType};
use crate::models::sprint::*;
use crate::{Db, Error, Result};

impl Db {
    pub async fn sprint_create(&self, input: CreateSprintInput) -> Result<Sprint> {
        sqlx::query_as::<_, Sprint>(
            "INSERT INTO sprints (name, goal, start_date, end_date) VALUES (?, ?, ?, ?)
             RETURNING id, name, goal, status, start_date, end_date, created_at, updated_at",
        )
        .bind(&input.name)
        .bind(&input.goal)
        .bind(&input.start_date)
        .bind(&input.end_date)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
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

            // 단일 활성 스프린트 보장: active 로 전환 시, 기존 active 들을 planning 으로 강등
            if s == "active" {
                let mut tx = self.pool.begin().await?;
                let demoted: Vec<i64> = sqlx::query_scalar(
                    "SELECT id FROM sprints WHERE status = 'active' AND id != ?",
                )
                .bind(id)
                .fetch_all(&mut *tx)
                .await?;
                if !demoted.is_empty() {
                    sqlx::query(
                        "UPDATE sprints SET status = 'planning', updated_at = datetime('now')
                         WHERE status = 'active' AND id != ?",
                    )
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
                }
                sqlx::query("UPDATE sprints SET status = ?, updated_at = datetime('now') WHERE id = ?")
                    .bind(&s)
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
                tx.commit().await?;

                for demoted_id in demoted {
                    let _ = self.history_record(CreateHistoryInput {
                        entity_type: EntityType::Sprint,
                        entity_id: demoted_id,
                        field: "status".to_string(),
                        old_value: Some("active".to_string()),
                        new_value: Some("planning".to_string()),
                        changed_by: changed_by.to_string(),
                    }).await;
                }
            } else {
                sqlx::query("UPDATE sprints SET status = ?, updated_at = datetime('now') WHERE id = ?")
                    .bind(&s)
                    .bind(id)
                    .execute(&self.pool)
                    .await?;
            }

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

    /// 스프린트를 삭제한다.
    /// FK 가 `ON DELETE SET NULL` 이므로 소속 이슈는 자동으로 백로그로 이동한다.
    /// 사용자에게는 사이드바의 2-클릭 confirm 으로 사고를 막는다.
    pub async fn sprint_delete(&self, id: i64) -> Result<()> {
        self.sprint_get(id).await?; // 존재 확인 (없으면 NotFound)

        sqlx::query("DELETE FROM sprints WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
