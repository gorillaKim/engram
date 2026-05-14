use crate::models::history::*;
use crate::{Db, Result};

impl Db {
    pub async fn history_record(&self, input: CreateHistoryInput) -> Result<History> {
        let et = serde_json::to_value(&input.entity_type).unwrap().as_str().unwrap().to_string();
        let id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO history (entity_type, entity_id, field, old_value, new_value, changed_by) VALUES (?, ?, ?, ?, ?, ?) RETURNING id",
        )
        .bind(&et)
        .bind(input.entity_id)
        .bind(&input.field)
        .bind(&input.old_value)
        .bind(&input.new_value)
        .bind(&input.changed_by)
        .fetch_one(&self.pool)
        .await?;

        sqlx::query_as::<_, History>(
            "SELECT id, entity_type, entity_id, field, old_value, new_value, changed_by, created_at FROM history WHERE id = ?",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn history_list(&self, entity_type: EntityType, entity_id: i64) -> Result<Vec<History>> {
        let et = serde_json::to_value(&entity_type).unwrap().as_str().unwrap().to_string();
        sqlx::query_as::<_, History>(
            "SELECT id, entity_type, entity_id, field, old_value, new_value, changed_by, created_at FROM history WHERE entity_type = ? AND entity_id = ? ORDER BY created_at ASC",
        )
        .bind(&et)
        .bind(entity_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }
}
