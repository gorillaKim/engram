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

    /// 특정 에이전트(또는 'user')가 남긴 변경 이력. 최근 순.
    /// 멀티 에이전트 환경에서 "누가 무엇을 했는지" 추적용.
    pub async fn history_by_agent(&self, agent_id: &str, limit: i64) -> Result<Vec<History>> {
        sqlx::query_as::<_, History>(
            "SELECT id, entity_type, entity_id, field, old_value, new_value, changed_by, created_at \
             FROM history WHERE changed_by = ? ORDER BY created_at DESC LIMIT ?",
        )
        .bind(agent_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }

    /// working 이슈(및 그 하위 태스크)의 가장 최근 히스토리로부터 경과 초 반환.
    /// 히스토리가 전혀 없으면 None.
    pub async fn history_last_activity_secs_for_issues(&self, issue_ids: &[i64]) -> Result<Option<i64>> {
        if issue_ids.is_empty() {
            return Ok(None);
        }
        let ph = issue_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let sql = format!(
            "SELECT CAST(strftime('%s', 'now') AS INTEGER) - CAST(strftime('%s', MAX(created_at)) AS INTEGER) \
             FROM history \
             WHERE (entity_type = 'issue' AND entity_id IN ({ph})) \
                OR (entity_type = 'task'  AND entity_id IN (SELECT id FROM tasks WHERE issue_id IN ({ph})))",
        );
        let mut q = sqlx::query_scalar::<_, Option<i64>>(&sql);
        for id in issue_ids { q = q.bind(*id); }
        for id in issue_ids { q = q.bind(*id); }
        q.fetch_one(&self.pool).await.map_err(Into::into)
    }

    /// 최근 N분 이내의 모든 변경 이력. since_minutes=None 이면 전체 최근 limit 건.
    /// 멀티 에이전트 활동 모니터링 / 사후 디버깅용.
    pub async fn history_recent(&self, limit: i64, since_minutes: Option<i64>) -> Result<Vec<History>> {
        match since_minutes {
            Some(mins) => sqlx::query_as::<_, History>(
                "SELECT id, entity_type, entity_id, field, old_value, new_value, changed_by, created_at \
                 FROM history \
                 WHERE created_at >= datetime('now', ? || ' minutes') \
                 ORDER BY created_at DESC LIMIT ?",
            )
            .bind(format!("-{}", mins))
            .bind(limit),
            None => sqlx::query_as::<_, History>(
                "SELECT id, entity_type, entity_id, field, old_value, new_value, changed_by, created_at \
                 FROM history ORDER BY created_at DESC LIMIT ?",
            )
            .bind(limit),
        }
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }
}
