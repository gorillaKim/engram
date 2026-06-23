use crate::models::history::{CreateHistoryInput, EntityType};
use crate::models::task_test::TaskTest;
use crate::{Db, Error, Result};

impl Db {
    pub async fn task_test_add(&self, task_id: i64, label: String) -> Result<TaskTest> {
        // RETURNING * — INSERT 후 별도 SELECT 시 WAL 풀의 가시성 지연으로 실패 가능.
        sqlx::query_as::<_, TaskTest>(
            "INSERT INTO task_tests (task_id, label) VALUES (?, ?)
             RETURNING id, task_id, label, checked, created_at, checked_at",
        )
        .bind(task_id)
        .bind(&label)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn task_test_add_bulk(&self, task_id: i64, labels: Vec<String>) -> Result<Vec<TaskTest>> {
        if labels.is_empty() {
            return Ok(vec![]);
        }
        let mut tx = self.pool.begin().await?;
        let mut result = Vec::with_capacity(labels.len());
        for label in &labels {
            let row: TaskTest = sqlx::query_as::<_, TaskTest>(
                "INSERT INTO task_tests (task_id, label) VALUES (?, ?)
                 RETURNING id, task_id, label, checked, created_at, checked_at",
            )
            .bind(task_id)
            .bind(label)
            .fetch_one(&mut *tx)
            .await?;
            result.push(row);
        }
        tx.commit().await?;
        Ok(result)
    }

    pub async fn task_test_list(&self, task_id: Option<i64>, issue_id: Option<i64>) -> Result<Vec<TaskTest>> {
        if let Some(tid) = task_id {
            sqlx::query_as::<_, TaskTest>(
                "SELECT id, task_id, label, checked, created_at, checked_at FROM task_tests WHERE task_id = ? ORDER BY id ASC",
            )
            .bind(tid)
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
        } else if let Some(iid) = issue_id {
            sqlx::query_as::<_, TaskTest>(
                "SELECT tt.id, tt.task_id, tt.label, tt.checked, tt.created_at, tt.checked_at FROM task_tests tt JOIN tasks t ON tt.task_id = t.id WHERE t.issue_id = ? ORDER BY t.ord ASC, tt.id ASC",
            )
            .bind(iid)
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
        } else {
            Err(Error::Validation("task_id 또는 issue_id 중 최소 하나는 지정해야 합니다.".to_string()))
        }
    }

    pub async fn task_test_check(&self, id: i64, changed_by: &str) -> Result<TaskTest> {
        sqlx::query(
            "UPDATE task_tests SET checked = 1, checked_at = datetime('now') WHERE id = ?",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        let tt = self.task_test_get(id).await?;
        let _ = self.history_record(CreateHistoryInput {
            entity_type: EntityType::Task,
            entity_id:   tt.task_id,
            field:        "task_test.checked".to_string(),
            old_value:    Some("false".to_string()),
            new_value:    Some("true".to_string()),
            changed_by:   changed_by.to_string(),
        }).await;
        Ok(tt)
    }

    pub async fn task_test_check_bulk(&self, ids: Vec<i64>, changed_by: &str) -> Result<Vec<TaskTest>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let mut tx = self.pool.begin().await?;
        for &id in &ids {
            sqlx::query(
                "UPDATE task_tests SET checked = 1, checked_at = datetime('now') WHERE id = ?",
            )
            .bind(id)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;

        let mut result = Vec::with_capacity(ids.len());
        for id in ids {
            let tt = self.task_test_get(id).await?;
            let _ = self.history_record(CreateHistoryInput {
                entity_type: EntityType::Task,
                entity_id:   tt.task_id,
                field:        "task_test.checked".to_string(),
                old_value:    Some("false".to_string()),
                new_value:    Some("true".to_string()),
                changed_by:   changed_by.to_string(),
            }).await;
            result.push(tt);
        }
        Ok(result)
    }

    pub async fn task_test_uncheck(&self, id: i64, changed_by: &str) -> Result<TaskTest> {
        sqlx::query(
            "UPDATE task_tests SET checked = 0, checked_at = NULL WHERE id = ?",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        let tt = self.task_test_get(id).await?;
        let _ = self.history_record(CreateHistoryInput {
            entity_type: EntityType::Task,
            entity_id:   tt.task_id,
            field:        "task_test.checked".to_string(),
            old_value:    Some("true".to_string()),
            new_value:    Some("false".to_string()),
            changed_by:   changed_by.to_string(),
        }).await;
        Ok(tt)
    }

    pub async fn task_test_remove(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM task_tests WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn task_test_get(&self, id: i64) -> Result<TaskTest> {
        sqlx::query_as::<_, TaskTest>(
            "SELECT id, task_id, label, checked, created_at, checked_at FROM task_tests WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| Error::NotFound(format!("task_test:{id}")))
    }
}
