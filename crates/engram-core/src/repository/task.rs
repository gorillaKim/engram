use crate::models::history::{CreateHistoryInput, EntityType};
use crate::models::task::*;
use crate::models::{OutputMode, CoreResponse};
use crate::{Db, Error, Result};

impl Db {
    pub async fn task_create(&self, input: CreateTaskInput) -> Result<Task> {
        let ord = self.next_ord(input.issue_id, input.after_task_id).await?;
        let source = input.source.unwrap_or(TaskSource::Planned);
        let sv = serde_json::to_value(&source).unwrap().as_str().unwrap().to_string();
        // RETURNING * — INSERT 후 다른 풀 커넥션이 행을 못 보는 WAL 가시성 이슈 회피.
        sqlx::query_as::<_, Task>(
            "INSERT INTO tasks (issue_id, title, description, goal, ord, source) VALUES (?, ?, ?, ?, ?, ?)
             RETURNING id, issue_id, title, description, goal, status, ord, source, created_at, updated_at",
        )
        .bind(input.issue_id)
        .bind(&input.title)
        .bind(&input.description)
        .bind(&input.goal)
        .bind(ord)
        .bind(&sv)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn task_get(&self, id: i64) -> Result<Task> {
        sqlx::query_as::<_, Task>(
            "SELECT id, issue_id, title, description, goal, status, ord, source, created_at, updated_at FROM tasks WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| Error::NotFound(format!("task:{id}")))
    }

    pub async fn task_list(&self, issue_id: i64, status: Option<TaskStatus>) -> Result<Vec<Task>> {
        if let Some(st) = status {
            let sv = serde_json::to_value(&st).unwrap().as_str().unwrap().to_string();
            sqlx::query_as::<_, Task>(
                "SELECT id, issue_id, title, description, goal, status, ord, source, created_at, updated_at FROM tasks WHERE issue_id = ? AND status = ? ORDER BY ord ASC",
            )
            .bind(issue_id)
            .bind(sv)
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
        } else {
            sqlx::query_as::<_, Task>(
                "SELECT id, issue_id, title, description, goal, status, ord, source, created_at, updated_at FROM tasks WHERE issue_id = ? ORDER BY ord ASC",
            )
            .bind(issue_id)
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
        }
    }

    pub async fn task_list_mode(&self, issue_id: i64, status: Option<TaskStatus>, mode: OutputMode) -> Result<CoreResponse<Vec<Task>>> {
        let mut tasks = self.task_list(issue_id, status).await?;
        let compact = matches!(mode, OutputMode::Compact) || matches!(mode, OutputMode::Agent);
        if compact {
            for task in &mut tasks {
                if let Some(ref desc) = task.description {
                    if desc.chars().count() > 200 {
                        let mut truncated: String = desc.chars().take(200).collect();
                        truncated.push_str("...");
                        task.description = Some(truncated);
                    }
                }
                if let Some(ref goal) = task.goal {
                    if goal.chars().count() > 200 {
                        let mut truncated: String = goal.chars().take(200).collect();
                        truncated.push_str("...");
                        task.goal = Some(truncated);
                    }
                }
            }
        }

        if matches!(mode, OutputMode::Agent) {
            let mut out = String::new();
            out.push_str("=== TASK LIST ===\n");
            if tasks.is_empty() {
                out.push_str("- None\n");
            } else {
                for task in &tasks {
                    let status_val = serde_json::to_value(&task.status).unwrap();
                    let status_str = status_val.as_str().unwrap_or("required");
                    let source_val = serde_json::to_value(&task.source).unwrap();
                    let source_str = source_val.as_str().unwrap_or("planned");
                    out.push_str(&format!(
                        "- #{} ({}, source:{}): {}\n",
                        task.id, status_str, source_str, task.title
                    ));
                }
            }
            out.push_str("==================");
            Ok(CoreResponse::Text(out))
        } else {
            Ok(CoreResponse::Json(tasks))
        }
    }

    /// 태스크 삭제.
    /// FK: task_tests 는 CASCADE 로 같이 삭제, notes.task_id 는 SET NULL.
    pub async fn task_delete(&self, id: i64) -> Result<()> {
        self.task_get(id).await?; // 존재 확인
        sqlx::query("DELETE FROM tasks WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn task_update(&self, id: i64, input: UpdateTaskInput, changed_by: &str) -> Result<Task> {
        if let Some(ref status) = input.status {
            let sv = serde_json::to_value(status).unwrap().as_str().unwrap().to_string();
            sqlx::query("UPDATE tasks SET status = ?, updated_at = datetime('now') WHERE id = ?")
                .bind(&sv).bind(id).execute(&self.pool).await?;
            let _ = self.history_record(CreateHistoryInput {
                entity_type: EntityType::Task,
                entity_id: id,
                field: "status".to_string(),
                old_value: None,
                new_value: Some(sv),
                changed_by: changed_by.to_string(),
            }).await;
        }
        if let Some(ref title) = input.title {
            sqlx::query("UPDATE tasks SET title = ?, updated_at = datetime('now') WHERE id = ?")
                .bind(title).bind(id).execute(&self.pool).await?;
            let _ = self.history_record(CreateHistoryInput {
                entity_type: EntityType::Task,
                entity_id: id,
                field: "title".to_string(),
                old_value: None,
                new_value: Some(title.clone()),
                changed_by: changed_by.to_string(),
            }).await;
        }
        if let Some(ref desc) = input.description {
            sqlx::query("UPDATE tasks SET description = ?, updated_at = datetime('now') WHERE id = ?")
                .bind(desc).bind(id).execute(&self.pool).await?;
        }
        if let Some(ref goal) = input.goal {
            sqlx::query("UPDATE tasks SET goal = ?, updated_at = datetime('now') WHERE id = ?")
                .bind(goal).bind(id).execute(&self.pool).await?;
        }
        self.task_get(id).await
    }

    pub async fn task_next(
        &self,
        project_key: Option<&str>,
        issue_id: Option<i64>,
    ) -> Result<Option<NextTask>> {
        let mut sql = r#"
            SELECT t.id as task_id, t.title as task_title,
                   i.id as issue_id, i.title as issue_title,
                   e.id as epic_id, e.title as epic_title, e.project_key,
                   CASE i.priority WHEN 'critical' THEN 0 WHEN 'high' THEN 1 WHEN 'medium' THEN 2 ELSE 3 END as priority_ord,
                   CASE i.status WHEN 'working' THEN 0 WHEN 'ready' THEN 1 ELSE 2 END as status_ord
            FROM tasks t
            JOIN issues i ON t.issue_id = i.id
            JOIN epics e ON i.epic_id = e.id
            WHERE t.status = 'ready'
            AND i.status IN ('ready', 'working')
            AND NOT EXISTS (
                SELECT 1 FROM issue_links il
                JOIN issues bi ON il.source_id = bi.id
                WHERE il.target_id = i.id AND il.link_type = 'blocks'
                  AND bi.status NOT IN ('finished','cancelled')
            )
        "#.to_string();

        if project_key.is_some() { sql.push_str(" AND e.project_key = ?"); }
        if issue_id.is_some()    { sql.push_str(" AND i.id = ?"); }
        sql.push_str(" ORDER BY priority_ord ASC, status_ord ASC, i.created_at ASC LIMIT 1");

        #[derive(sqlx::FromRow)]
        struct Row {
            task_id: i64, task_title: String,
            issue_id: i64, issue_title: String,
            epic_id: i64, epic_title: String, project_key: String,
            priority_ord: i64, status_ord: i64,
        }

        let mut q = sqlx::query_as::<_, Row>(&sql);
        if let Some(p) = project_key { q = q.bind(p); }
        if let Some(i) = issue_id    { q = q.bind(i); }

        Ok(q.fetch_optional(&self.pool).await?.map(|r| NextTask {
            task_id: r.task_id, task_title: r.task_title,
            issue_id: r.issue_id, issue_title: r.issue_title,
            epic_id: r.epic_id, epic_title: r.epic_title, project_key: r.project_key,
            reason: format!("priority_ord:{} + status_ord:{}", r.priority_ord, r.status_ord),
        }))
    }

    pub async fn task_next_mode(
        &self,
        project_key: Option<&str>,
        issue_id: Option<i64>,
        mode: OutputMode,
    ) -> Result<CoreResponse<Option<NextTask>>> {
        let next = self.task_next(project_key, issue_id).await?;
        if matches!(mode, OutputMode::Agent) {
            let mut out = String::new();
            out.push_str("=== NEXT TASK ===\n");
            match &next {
                None => {
                    out.push_str("No ready tasks available.\n");
                }
                Some(nt) => {
                    out.push_str(&format!("Task ID: #{}\n", nt.task_id));
                    out.push_str(&format!("Task Title: {}\n", nt.task_title));
                    out.push_str(&format!("Issue ID: #{}\n", nt.issue_id));
                    out.push_str(&format!("Issue Title: {}\n", nt.issue_title));
                    out.push_str(&format!("Epic ID: #{}\n", nt.epic_id));
                    out.push_str(&format!("Epic Title: {}\n", nt.epic_title));
                    out.push_str(&format!("Project Key: {}\n", nt.project_key));
                    out.push_str(&format!("Reason: {}\n", nt.reason));
                }
            }
            out.push_str("=================");
            Ok(CoreResponse::Text(out))
        } else {
            Ok(CoreResponse::Json(next))
        }
    }

    async fn next_ord(&self, issue_id: i64, after_task_id: Option<i64>) -> Result<f64> {
        match after_task_id {
            None => {
                let max: Option<f64> = sqlx::query_scalar(
                    "SELECT MAX(ord) FROM tasks WHERE issue_id = ?",
                )
                .bind(issue_id)
                .fetch_one(&self.pool)
                .await?;
                Ok(max.unwrap_or(0.0) + 1.0)
            }
            Some(after_id) => {
                let after: f64 = sqlx::query_scalar("SELECT ord FROM tasks WHERE id = ?")
                    .bind(after_id)
                    .fetch_one(&self.pool)
                    .await?;
                let next: Option<f64> = sqlx::query_scalar(
                    "SELECT MIN(ord) FROM tasks WHERE issue_id = ? AND ord > ?",
                )
                .bind(issue_id).bind(after)
                .fetch_one(&self.pool)
                .await?;
                Ok(match next {
                    Some(n) => (after + n) / 2.0,
                    None    => after + 1.0,
                })
            }
        }
    }
}
