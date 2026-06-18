use crate::models::epic::*;
use crate::models::history::{CreateHistoryInput, EntityType};
use crate::models::{OutputMode, CoreResponse};
use crate::{Db, Error, Result};

const EPIC_COLS: &str = "id, project_key, mission_id, sprint_id, title, description, status, created_at, updated_at";

impl Db {
    pub async fn epic_create(&self, input: CreateEpicInput) -> Result<Epic> {
        sqlx::query_as::<_, Epic>(
            &format!(
                "INSERT INTO epics (project_key, mission_id, sprint_id, title, description) VALUES (?, ?, ?, ?, ?) \
                 RETURNING {EPIC_COLS}"
            ),
        )
        .bind(&input.project_key)
        .bind(input.mission_id)
        .bind(input.sprint_id)
        .bind(&input.title)
        .bind(&input.description)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn epic_get(&self, id: i64) -> Result<Epic> {
        sqlx::query_as::<_, Epic>(&format!("SELECT {EPIC_COLS} FROM epics WHERE id = ?"))
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| Error::NotFound(format!("epic:{id}")))
    }

    pub async fn epic_get_mode(&self, id: i64, mode: OutputMode) -> Result<CoreResponse<Epic>> {
        let epic = self.epic_get(id).await?;
        if matches!(mode, OutputMode::Agent) {
            Ok(CoreResponse::Text(format_agent_epic_text(&epic)))
        } else {
            Ok(CoreResponse::Json(epic))
        }
    }

    /// 에픽 목록. project_key, status, sprint_id 로 필터.
    pub async fn epic_list(
        &self,
        project_key: Option<&str>,
        include_completed: bool,
    ) -> Result<Vec<Epic>> {
        self.epic_list_filtered(project_key, include_completed, None, false).await
    }

    pub async fn epic_list_mode(
        &self,
        project_key: Option<&str>,
        include_completed: bool,
        mode: OutputMode,
    ) -> Result<CoreResponse<Vec<Epic>>> {
        let mut epics = self.epic_list(project_key, include_completed).await?;
        let compact = matches!(mode, OutputMode::Compact) || matches!(mode, OutputMode::Agent);
        if compact {
            for epic in &mut epics {
                if let Some(ref desc) = epic.description {
                    if desc.chars().count() > 200 {
                        let mut truncated: String = desc.chars().take(200).collect();
                        truncated.push_str("...");
                        epic.description = Some(truncated);
                    }
                }
            }
        }

        if matches!(mode, OutputMode::Agent) {
            let mut out = String::new();
            out.push_str("=== EPIC LIST ===\n");
            if epics.is_empty() {
                out.push_str("- None\n");
            } else {
                for epic in &epics {
                    let status_val = serde_json::to_value(&epic.status).unwrap();
                    let status_str = status_val.as_str().unwrap_or("active");
                    out.push_str(&format!("- #{} ({}): {}\n", epic.id, status_str, epic.title));
                }
            }
            out.push_str("==================");
            Ok(CoreResponse::Text(out))
        } else {
            Ok(CoreResponse::Json(epics))
        }
    }

    /// 에픽 목록 (sprint 필터 포함).
    ///
    /// - `sprint_id = Some(id)`: 해당 sprint 의 에픽만.
    /// - `backlog_only = true`: sprint_id IS NULL 인 에픽만 (백로그). `sprint_id` 와 동시 지정 시 backlog_only 우선.
    pub async fn epic_list_filtered(
        &self,
        project_key: Option<&str>,
        include_completed: bool,
        sprint_id: Option<i64>,
        backlog_only: bool,
    ) -> Result<Vec<Epic>> {
        let mut sql = format!("SELECT {EPIC_COLS} FROM epics WHERE 1=1");
        if project_key.is_some() { sql.push_str(" AND project_key = ?"); }
        if !include_completed { sql.push_str(" AND status != 'completed'"); }
        if backlog_only {
            sql.push_str(" AND sprint_id IS NULL");
        } else if sprint_id.is_some() {
            sql.push_str(" AND sprint_id = ?");
        }
        sql.push_str(" ORDER BY id DESC");

        let mut q = sqlx::query_as::<_, Epic>(&sql);
        if let Some(p) = project_key { q = q.bind(p); }
        if !backlog_only {
            if let Some(s) = sprint_id { q = q.bind(s); }
        }
        q.fetch_all(&self.pool).await.map_err(Into::into)
    }

    /// 에픽을 삭제한다. 하위 이슈/태스크/노트/링크까지 cascade 삭제.
    pub async fn epic_delete(&self, id: i64, changed_by: &str) -> Result<()> {
        let epic = self.epic_get(id).await?;
        let mut tx = self.pool.begin().await?;

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

        sqlx::query(
            "DELETE FROM tasks WHERE issue_id IN (SELECT id FROM issues WHERE epic_id = ?)",
        )
        .bind(id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "DELETE FROM notes WHERE issue_id IN (SELECT id FROM issues WHERE epic_id = ?)",
        )
        .bind(id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "DELETE FROM issue_links \
             WHERE source_id IN (SELECT id FROM issues WHERE epic_id = ?) \
                OR target_id IN (SELECT id FROM issues WHERE epic_id = ?)",
        )
        .bind(id)
        .bind(id)
        .execute(&mut *tx)
        .await?;

        sqlx::query("DELETE FROM issues WHERE epic_id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        sqlx::query("DELETE FROM epics WHERE id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;

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
        let _current = self.epic_get(id).await?;

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

        if let Some(mid) = input.mission_id {
            sqlx::query(
                "UPDATE epics SET mission_id = ?, updated_at = datetime('now') WHERE id = ?",
            )
            .bind(mid)
            .bind(id)
            .execute(&self.pool)
            .await?;
            let _ = self.history_record(CreateHistoryInput {
                entity_type: EntityType::Epic,
                entity_id: id,
                field: "mission_id".to_string(),
                old_value: None,
                new_value: Some(mid.to_string()),
                changed_by: changed_by.to_string(),
            }).await;
        }

        if input.update_sprint_id {
            self.epic_set_sprint_internal(id, input.sprint_id, changed_by).await?;
        }

        self.epic_get(id).await
    }

    /// 에픽의 스프린트 소속을 변경한다. None 이면 백로그.
    ///
    /// 산하 모든 이슈는 epic.sprint_id 를 derive 하므로 자동으로 따라온다.
    /// history 에 sprint_id 변경 기록.
    pub async fn epic_set_sprint(
        &self,
        epic_id: i64,
        sprint_id: Option<i64>,
        changed_by: &str,
    ) -> Result<Epic> {
        let _current = self.epic_get(epic_id).await?;
        self.epic_set_sprint_internal(epic_id, sprint_id, changed_by).await?;
        self.epic_get(epic_id).await
    }

    async fn epic_set_sprint_internal(
        &self,
        epic_id: i64,
        sprint_id: Option<i64>,
        changed_by: &str,
    ) -> Result<()> {
        let current_sid: Option<i64> = sqlx::query_scalar("SELECT sprint_id FROM epics WHERE id = ?")
            .bind(epic_id)
            .fetch_one(&self.pool)
            .await?;

        sqlx::query(
            "UPDATE epics SET sprint_id = ?, updated_at = datetime('now') WHERE id = ?",
        )
        .bind(sprint_id)
        .bind(epic_id)
        .execute(&self.pool)
        .await?;

        if current_sid != sprint_id {
            let old_v = current_sid.map(|s| s.to_string()).unwrap_or_else(|| "null".to_string());
            let new_v = sprint_id.map(|s| s.to_string()).unwrap_or_else(|| "null".to_string());
            let _ = self.history_record(CreateHistoryInput {
                entity_type: EntityType::Epic,
                entity_id: epic_id,
                field: "sprint_id".to_string(),
                old_value: Some(old_v),
                new_value: Some(new_v),
                changed_by: changed_by.to_string(),
            }).await;
        }
        Ok(())
    }
}

fn format_agent_epic_text(epic: &Epic) -> String {
    let mut out = String::new();
    out.push_str("=== EPIC SPECIFICATION ===\n");
    out.push_str(&format!("ID: #{}\n", epic.id));
    out.push_str(&format!("Project Key: {}\n", epic.project_key));
    out.push_str(&format!("Mission ID: {}\n", epic.mission_id.map(|id| id.to_string()).unwrap_or_else(|| "None".to_string())));
    out.push_str(&format!("Sprint ID: {}\n", epic.sprint_id.map(|id| id.to_string()).unwrap_or_else(|| "None".to_string())));
    out.push_str(&format!("Title: {}\n", epic.title));
    let status_val = serde_json::to_value(&epic.status).unwrap();
    let status_str = status_val.as_str().unwrap_or("active");
    out.push_str(&format!("Status: {}\n", status_str));
    out.push_str(&format!("Created At: {}\n", epic.created_at));
    out.push_str(&format!("Updated At: {}\n", epic.updated_at));
    out.push_str("\n[Description]\n");
    out.push_str(epic.description.as_deref().unwrap_or("None"));
    out.push_str("\n==========================");
    out
}

#[cfg(test)]
mod tests {
    use crate::{
        Db,
        models::{
            epic::{CreateEpicInput, EpicStatus, UpdateEpicInput},
            issue::{CreateIssueInput, IssueStatus, UpdateIssueInput},
            mission::CreateMissionInput,
            sprint::CreateSprintInput,
        },
    };

    async fn setup() -> Db {
        Db::open_in_memory().await.unwrap()
    }

    fn mission_input(title: &str) -> CreateMissionInput {
        CreateMissionInput { title: title.to_string(), description: None, jira_key: None }
    }

    fn epic_input(project: &str, mission_id: Option<i64>, sprint_id: Option<i64>, title: &str) -> CreateEpicInput {
        CreateEpicInput {
            project_key: project.to_string(),
            mission_id,
            sprint_id,
            title: title.to_string(),
            description: None,
        }
    }

    fn issue_input(epic_id: i64, title: &str) -> CreateIssueInput {
        CreateIssueInput {
            epic_id,
            title: title.to_string(),
            description: None,
            goal: None,
            priority: None,
        }
    }

    #[tokio::test]
    async fn test_epic_list_excludes_completed_by_default() {
        let db = setup().await;

        let mission = db.mission_create(mission_input("M")).await.unwrap();

        let e1 = db.epic_create(epic_input("p", Some(mission.id), None, "Active Epic")).await.unwrap();
        let e2 = db.epic_create(epic_input("p", Some(mission.id), None, "Completed Epic")).await.unwrap();

        db.epic_update(e2.id, UpdateEpicInput {
            status: Some(EpicStatus::Completed), ..Default::default()
        }, "test").await.unwrap();

        let default_list = db.epic_list(None, false).await.unwrap();
        assert_eq!(default_list.len(), 1);
        assert_eq!(default_list[0].id, e1.id);

        let full_list = db.epic_list(None, true).await.unwrap();
        assert_eq!(full_list.len(), 2);
    }

    #[tokio::test]
    async fn test_epic_set_sprint_moves_and_records_history() {
        let db = setup().await;

        let mission = db.mission_create(mission_input("M")).await.unwrap();
        let sprint_a = db.sprint_create(CreateSprintInput {
            name: "A".to_string(), goal: None, start_date: None, end_date: None,
        }).await.unwrap();
        let sprint_b = db.sprint_create(CreateSprintInput {
            name: "B".to_string(), goal: None, start_date: None, end_date: None,
        }).await.unwrap();

        let epic = db.epic_create(epic_input("p", Some(mission.id), Some(sprint_a.id), "E")).await.unwrap();
        assert_eq!(epic.sprint_id, Some(sprint_a.id));

        let moved = db.epic_set_sprint(epic.id, Some(sprint_b.id), "test-agent").await.unwrap();
        assert_eq!(moved.sprint_id, Some(sprint_b.id));

        let history = db.history_list(crate::models::EntityType::Epic, epic.id).await.unwrap();
        assert!(history.iter().any(|h| h.field == "sprint_id"));
    }

    #[tokio::test]
    async fn test_epic_set_sprint_to_backlog() {
        let db = setup().await;
        let mission = db.mission_create(mission_input("M")).await.unwrap();
        let sprint = db.sprint_create(CreateSprintInput {
            name: "S".to_string(), goal: None, start_date: None, end_date: None,
        }).await.unwrap();
        let epic = db.epic_create(epic_input("p", Some(mission.id), Some(sprint.id), "E")).await.unwrap();

        let moved = db.epic_set_sprint(epic.id, None, "test-agent").await.unwrap();
        assert!(moved.sprint_id.is_none());
    }

    #[tokio::test]
    async fn test_epic_sprint_propagates_to_issues() {
        let db = setup().await;
        let mission = db.mission_create(mission_input("M")).await.unwrap();
        let sprint = db.sprint_create(CreateSprintInput {
            name: "S".to_string(), goal: None, start_date: None, end_date: None,
        }).await.unwrap();
        let epic = db.epic_create(epic_input("p", Some(mission.id), Some(sprint.id), "E")).await.unwrap();
        let issue = db.issue_create(issue_input(epic.id, "I")).await.unwrap();

        assert_eq!(issue.sprint_id, Some(sprint.id), "이슈는 epic.sprint_id 를 상속");
        assert_eq!(issue.mission_id, Some(mission.id), "이슈는 epic.mission_id 를 상속");

        // Epic 을 백로그로 이동 → 이슈 sprint_id 도 자동 변경
        db.epic_set_sprint(epic.id, None, "agent").await.unwrap();
        let i2 = db.issue_get(issue.id, false).await.unwrap();
        assert!(i2.sprint_id.is_none(), "epic.sprint_id 변경이 이슈에 즉시 반영");

        // UpdateIssueInput.status 변경은 sprint 상속과 무관
        db.issue_update(issue.id, UpdateIssueInput {
            status: Some(IssueStatus::Ready), ..Default::default()
        }, "agent").await.unwrap();
    }
}
