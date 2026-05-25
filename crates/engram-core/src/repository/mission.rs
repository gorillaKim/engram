use crate::models::epic::Epic;
use crate::models::history::{CreateHistoryInput, EntityType};
use crate::models::issue::Issue;
use crate::models::mission::*;
use crate::{Db, Error, Result};

impl Db {
    pub async fn mission_create(&self, input: CreateMissionInput) -> Result<Mission> {
        sqlx::query_as::<_, Mission>(
            "INSERT INTO missions (jira_key, title, description) VALUES (?, ?, ?) \
             RETURNING id, jira_key, title, description, status, created_at, updated_at",
        )
        .bind(&input.jira_key)
        .bind(&input.title)
        .bind(&input.description)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed") {
                Error::Validation(format!(
                    "jira_key '{}' already exists",
                    input.jira_key.as_deref().unwrap_or("")
                ))
            } else {
                Error::Db(e)
            }
        })
    }

    pub async fn mission_get(&self, id: i64) -> Result<Mission> {
        sqlx::query_as::<_, Mission>(
            "SELECT id, jira_key, title, description, status, created_at, updated_at \
             FROM missions WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| Error::NotFound(format!("mission:{id}")))
    }

    pub async fn mission_list(&self, filter: MissionFilter) -> Result<Vec<Mission>> {
        let mut sql = "SELECT id, jira_key, title, description, status, created_at, updated_at \
                       FROM missions WHERE 1=1"
            .to_string();

        if filter.status.is_some() {
            sql.push_str(" AND status = ?");
        } else if !filter.include_completed {
            sql.push_str(" AND status = 'active'");
        }
        sql.push_str(" ORDER BY id DESC");

        let mut q = sqlx::query_as::<_, Mission>(&sql);
        if let Some(s) = filter.status {
            let sv = serde_json::to_value(&s).unwrap().as_str().unwrap().to_string();
            q = q.bind(sv);
        }
        q.fetch_all(&self.pool).await.map_err(Into::into)
    }

    pub async fn mission_update(
        &self,
        id: i64,
        input: UpdateMissionInput,
        changed_by: &str,
    ) -> Result<Mission> {
        let _current = self.mission_get(id).await?;

        if let Some(ref title) = input.title {
            sqlx::query(
                "UPDATE missions SET title = ?, updated_at = datetime('now') WHERE id = ?",
            )
            .bind(title)
            .bind(id)
            .execute(&self.pool)
            .await?;
            let _ = self
                .history_record(CreateHistoryInput {
                    entity_type: EntityType::Mission,
                    entity_id: id,
                    field: "title".to_string(),
                    old_value: None,
                    new_value: Some(title.clone()),
                    changed_by: changed_by.to_string(),
                })
                .await;
        }

        if let Some(ref desc) = input.description {
            sqlx::query(
                "UPDATE missions SET description = ?, updated_at = datetime('now') WHERE id = ?",
            )
            .bind(desc)
            .bind(id)
            .execute(&self.pool)
            .await?;
            let _ = self
                .history_record(CreateHistoryInput {
                    entity_type: EntityType::Mission,
                    entity_id: id,
                    field: "description".to_string(),
                    old_value: None,
                    new_value: Some(desc.clone()),
                    changed_by: changed_by.to_string(),
                })
                .await;
        }

        if let Some(ref jira_key) = input.jira_key {
            sqlx::query(
                "UPDATE missions SET jira_key = ?, updated_at = datetime('now') WHERE id = ?",
            )
            .bind(jira_key)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                if e.to_string().contains("UNIQUE constraint failed") {
                    Error::Validation(format!("jira_key '{jira_key}' already exists"))
                } else {
                    Error::Db(e)
                }
            })?;
            let _ = self
                .history_record(CreateHistoryInput {
                    entity_type: EntityType::Mission,
                    entity_id: id,
                    field: "jira_key".to_string(),
                    old_value: None,
                    new_value: Some(jira_key.clone()),
                    changed_by: changed_by.to_string(),
                })
                .await;
        }

        if let Some(ref status) = input.status {
            let sv = serde_json::to_value(status).unwrap().as_str().unwrap().to_string();
            sqlx::query(
                "UPDATE missions SET status = ?, updated_at = datetime('now') WHERE id = ?",
            )
            .bind(&sv)
            .bind(id)
            .execute(&self.pool)
            .await?;
            let _ = self
                .history_record(CreateHistoryInput {
                    entity_type: EntityType::Mission,
                    entity_id: id,
                    field: "status".to_string(),
                    old_value: None,
                    new_value: Some(sv),
                    changed_by: changed_by.to_string(),
                })
                .await;
        }

        self.mission_get(id).await
    }

    /// 미션별 종합 진척도를 집계한다.
    ///
    /// 이슈는 `epic.mission_id` 를 통해 mission 에 귀속된다 (Issue 는 mission_id 컬럼이 없음).
    pub async fn mission_progress_query(&self, id: i64) -> Result<MissionProgress> {
        let _mission = self.mission_get(id).await?;

        let row = sqlx::query_as::<_, MissionProgress>(
            "SELECT
                m.id,
                m.title,
                (SELECT COUNT(*) FROM epics WHERE mission_id = m.id) AS epics_count,
                COUNT(i.id)                                                          AS issues_count,
                COUNT(CASE WHEN i.status IN ('required','ready') THEN 1 END)        AS todo_issues,
                COUNT(CASE WHEN i.status = 'working'            THEN 1 END)         AS working_issues,
                COUNT(CASE WHEN i.status = 'demo'               THEN 1 END)         AS demo_issues,
                COUNT(CASE WHEN i.status = 'finished'           THEN 1 END)         AS finished_issues,
                COUNT(CASE WHEN i.status = 'cancelled'          THEN 1 END)         AS cancelled_issues,
                CASE WHEN COUNT(i.id) = 0 THEN 0.0
                     ELSE CAST(COUNT(CASE WHEN i.status = 'finished' THEN 1 END) AS REAL)
                          / CAST(COUNT(i.id) AS REAL)
                END                                                                  AS progress_rate
             FROM missions m
             LEFT JOIN epics e ON e.mission_id = m.id
             LEFT JOIN issues i ON i.epic_id = e.id
             WHERE m.id = ?
             GROUP BY m.id",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| Error::NotFound(format!("mission:{id}")))?;

        Ok(row)
    }

    pub async fn mission_get_by_jira_key(&self, jira_key: &str) -> Result<Mission> {
        sqlx::query_as::<_, Mission>(
            "SELECT id, jira_key, title, description, status, created_at, updated_at \
             FROM missions WHERE jira_key = ?",
        )
        .bind(jira_key)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| Error::NotFound(format!("mission with jira_key:{jira_key}")))
    }

    /// 미션의 계층 트리를 반환한다: Mission → Vec<EpicWithIssues>.
    ///
    /// 한 mission 산하의 epic 들이 서로 다른 sprint 에 속할 수 있다 (ADR-0014).
    pub async fn mission_get_tree(&self, id: i64) -> Result<MissionTree> {
        let mission = self.mission_get(id).await?;

        let epics = sqlx::query_as::<_, Epic>(
            "SELECT id, project_key, mission_id, sprint_id, title, description, status, created_at, updated_at \
             FROM epics WHERE mission_id = ? ORDER BY id ASC",
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await?;

        // mission 산하 모든 이슈를 epic JOIN 으로 조회 (mission_id / sprint_id 는 epic 에서 derive).
        let all_issues = sqlx::query_as::<_, Issue>(
            "SELECT i.id, i.epic_id, e.mission_id AS mission_id, e.sprint_id AS sprint_id, \
                    i.title, i.description, i.goal, i.status, i.priority, \
                    i.assigned_agent, i.created_at, i.updated_at \
             FROM issues i \
             JOIN epics e ON i.epic_id = e.id \
             WHERE e.mission_id = ? ORDER BY i.id ASC",
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await?;

        let epics_with_issues: Vec<EpicWithIssues> = epics
            .into_iter()
            .map(|epic| {
                let issues: Vec<Issue> = all_issues
                    .iter()
                    .filter(|i| i.epic_id == epic.id)
                    .cloned()
                    .collect();
                EpicWithIssues { epic, issues }
            })
            .collect();

        Ok(MissionTree {
            mission,
            epics: epics_with_issues,
        })
    }

    pub async fn mission_delete(&self, id: i64) -> Result<()> {
        let _mission = self.mission_get(id).await?;

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM epics WHERE mission_id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        if count > 0 {
            return Err(Error::Validation(format!(
                "mission:{id} has {count} child epics, cannot delete"
            )));
        }

        sqlx::query("DELETE FROM missions WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Db;

    async fn setup_db() -> Db {
        Db::open_in_memory().await.expect("open in-memory db")
    }

    fn make_input(title: &str) -> CreateMissionInput {
        CreateMissionInput {
            title: title.to_string(),
            description: None,
            jira_key: None,
        }
    }

    #[tokio::test]
    async fn test_mission_create_and_get() {
        let db = setup_db().await;
        let m = db.mission_create(make_input("M1")).await.unwrap();
        assert_eq!(m.title, "M1");
        assert_eq!(m.status, MissionStatus::Active);
        assert!(m.jira_key.is_none());
    }

    #[tokio::test]
    async fn test_mission_create_duplicate_jira_key_fails() {
        let db = setup_db().await;
        db.mission_create(CreateMissionInput {
            title: "M1".to_string(),
            description: None,
            jira_key: Some("PROJ-1".to_string()),
        })
        .await
        .unwrap();

        let result = db
            .mission_create(CreateMissionInput {
                title: "M2".to_string(),
                description: None,
                jira_key: Some("PROJ-1".to_string()),
            })
            .await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("PROJ-1"), "{err_msg}");
    }

    #[tokio::test]
    async fn test_mission_get_not_found() {
        let db = setup_db().await;
        let result = db.mission_get(9999).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("mission:9999"), "{err_msg}");
    }

    #[tokio::test]
    async fn test_mission_list_active_only_default() {
        let db = setup_db().await;
        db.mission_create(make_input("Active")).await.unwrap();
        let m2 = db.mission_create(make_input("Completed")).await.unwrap();
        db.mission_update(
            m2.id,
            UpdateMissionInput {
                status: Some(MissionStatus::Completed),
                ..Default::default()
            },
            "test",
        )
        .await
        .unwrap();

        let list = db.mission_list(MissionFilter::default()).await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].title, "Active");
    }

    #[tokio::test]
    async fn test_mission_list_include_completed() {
        let db = setup_db().await;
        db.mission_create(make_input("Active")).await.unwrap();
        let m2 = db.mission_create(make_input("Completed")).await.unwrap();
        db.mission_update(
            m2.id,
            UpdateMissionInput {
                status: Some(MissionStatus::Completed),
                ..Default::default()
            },
            "test",
        )
        .await
        .unwrap();

        let list = db
            .mission_list(MissionFilter {
                include_completed: true,
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn test_mission_update_status_recorded_in_history() {
        let db = setup_db().await;
        let m = db.mission_create(make_input("M1")).await.unwrap();

        let updated = db
            .mission_update(
                m.id,
                UpdateMissionInput {
                    status: Some(MissionStatus::Completed),
                    title: Some("M1 Updated".to_string()),
                    ..Default::default()
                },
                "agent-test",
            )
            .await
            .unwrap();

        assert_eq!(updated.status, MissionStatus::Completed);
        assert_eq!(updated.title, "M1 Updated");

        let history = db
            .history_list(crate::models::EntityType::Mission, m.id)
            .await
            .unwrap();
        assert!(history.iter().any(|h| h.field == "status"));
        assert!(history.iter().any(|h| h.field == "title"));
    }

    #[tokio::test]
    async fn test_mission_delete_blocked_by_child_epic() {
        let db = setup_db().await;
        let m = db.mission_create(make_input("M1")).await.unwrap();

        sqlx::query("INSERT INTO epics (project_key, mission_id, title) VALUES ('test', ?, 'E1')")
            .bind(m.id)
            .execute(&db.pool)
            .await
            .unwrap();

        let result = db.mission_delete(m.id).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("child epics"), "{err_msg}");
    }

    #[tokio::test]
    async fn test_mission_delete_success() {
        let db = setup_db().await;
        let m = db.mission_create(make_input("ToDelete")).await.unwrap();
        db.mission_delete(m.id).await.unwrap();
        assert!(db.mission_get(m.id).await.is_err());
    }

    #[tokio::test]
    async fn test_mission_progress_query_empty() {
        let db = setup_db().await;
        let m = db.mission_create(make_input("M")).await.unwrap();

        let p = db.mission_progress_query(m.id).await.unwrap();
        assert_eq!(p.id, m.id);
        assert_eq!(p.title, "M");
        assert_eq!(p.issues_count, 0);
        assert_eq!(p.epics_count, 0);
        assert_eq!(p.progress_rate, 0.0);
    }

    #[tokio::test]
    async fn test_mission_progress_query_calculates_correctly() {
        let db = setup_db().await;
        let m = db.mission_create(make_input("M")).await.unwrap();

        sqlx::query("INSERT INTO epics (project_key, mission_id, title) VALUES ('test', ?, 'E1')")
            .bind(m.id)
            .execute(&db.pool)
            .await
            .unwrap();

        let epic_id: i64 = sqlx::query_scalar("SELECT id FROM epics WHERE mission_id = ?")
            .bind(m.id)
            .fetch_one(&db.pool)
            .await
            .unwrap();

        for (status, title) in [
            ("finished", "I1"),
            ("working", "I2"),
            ("required", "I3"),
            ("demo", "I4"),
            ("cancelled", "I5"),
        ] {
            sqlx::query(
                "INSERT INTO issues (epic_id, title, status, priority) \
                 VALUES (?, ?, ?, 'medium')",
            )
            .bind(epic_id)
            .bind(title)
            .bind(status)
            .execute(&db.pool)
            .await
            .unwrap();
        }

        let p = db.mission_progress_query(m.id).await.unwrap();
        assert_eq!(p.issues_count, 5);
        assert_eq!(p.epics_count, 1);
        assert_eq!(p.finished_issues, 1);
        assert_eq!(p.working_issues, 1);
        assert_eq!(p.todo_issues, 1);
        assert_eq!(p.demo_issues, 1);
        assert_eq!(p.cancelled_issues, 1);
        assert!((p.progress_rate - 0.2).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_mission_progress_query_not_found() {
        let db = setup_db().await;
        let result = db.mission_progress_query(9999).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mission_get_by_jira_key() {
        let db = setup_db().await;
        let m = db
            .mission_create(CreateMissionInput {
                title: "JiraKeyMission".to_string(),
                description: None,
                jira_key: Some("PROJ-42".to_string()),
            })
            .await
            .unwrap();

        let found = db.mission_get_by_jira_key("PROJ-42").await.unwrap();
        assert_eq!(found.id, m.id);

        assert!(db.mission_get_by_jira_key("PROJ-NONEXISTENT").await.is_err());
    }

    #[tokio::test]
    async fn test_mission_get_tree_groups_correctly() {
        let db = setup_db().await;
        let m = db.mission_create(make_input("Tree Test")).await.unwrap();

        sqlx::query("INSERT INTO epics (project_key, mission_id, title) VALUES ('p1', ?, 'E1')")
            .bind(m.id)
            .execute(&db.pool)
            .await
            .unwrap();

        let epic_id: i64 = sqlx::query_scalar("SELECT id FROM epics WHERE mission_id = ?")
            .bind(m.id)
            .fetch_one(&db.pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO issues (epic_id, title, priority) VALUES (?, 'I1', 'medium')")
            .bind(epic_id)
            .execute(&db.pool)
            .await
            .unwrap();

        let tree = db.mission_get_tree(m.id).await.unwrap();
        assert_eq!(tree.mission.id, m.id);
        assert_eq!(tree.epics.len(), 1);
        assert_eq!(tree.epics[0].issues.len(), 1);
    }

    #[tokio::test]
    async fn test_mission_get_tree_multiple_epics() {
        let db = setup_db().await;
        let m = db
            .mission_create(make_input("Multi Epic Mission"))
            .await
            .unwrap();

        for title in ["Epic A", "Epic B"] {
            sqlx::query(
                "INSERT INTO epics (project_key, mission_id, title) VALUES ('p1', ?, ?)",
            )
            .bind(m.id)
            .bind(title)
            .execute(&db.pool)
            .await
            .unwrap();
        }

        let epic_ids: Vec<i64> = sqlx::query_scalar(
            "SELECT id FROM epics WHERE mission_id = ? ORDER BY id ASC",
        )
        .bind(m.id)
        .fetch_all(&db.pool)
        .await
        .unwrap();

        for title in ["Issue A1", "Issue A2"] {
            sqlx::query("INSERT INTO issues (epic_id, title, priority) VALUES (?, ?, 'medium')")
                .bind(epic_ids[0])
                .bind(title)
                .execute(&db.pool)
                .await
                .unwrap();
        }
        sqlx::query("INSERT INTO issues (epic_id, title, priority) VALUES (?, 'Issue B1', 'medium')")
            .bind(epic_ids[1])
            .execute(&db.pool)
            .await
            .unwrap();

        let tree = db.mission_get_tree(m.id).await.unwrap();
        assert_eq!(tree.epics.len(), 2);
        assert_eq!(tree.epics[0].issues.len(), 2);
        assert_eq!(tree.epics[1].issues.len(), 1);
    }

    #[tokio::test]
    async fn test_mission_get_tree_not_found() {
        let db = setup_db().await;
        assert!(db.mission_get_tree(9999).await.is_err());
    }
}
