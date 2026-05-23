use crate::models::epic::Epic;
use crate::models::history::{CreateHistoryInput, EntityType};
use crate::models::issue::Issue;
use crate::models::mission::*;
use crate::{Db, Error, Result};

impl Db {
    pub async fn mission_create(&self, input: CreateMissionInput) -> Result<Mission> {
        let id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO missions (jira_key, title, description, sprint_id) VALUES (?, ?, ?, ?) RETURNING id",
        )
        .bind(&input.jira_key)
        .bind(&input.title)
        .bind(&input.description)
        .bind(input.sprint_id)
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
        })?;
        self.mission_get(id).await
    }

    pub async fn mission_get(&self, id: i64) -> Result<Mission> {
        sqlx::query_as::<_, Mission>(
            "SELECT id, jira_key, title, description, status, sprint_id, created_at, updated_at \
             FROM missions WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| Error::NotFound(format!("mission:{id}")))
    }

    pub async fn mission_list(&self, filter: MissionFilter) -> Result<Vec<Mission>> {
        let mut sql = "SELECT id, jira_key, title, description, status, sprint_id, created_at, updated_at \
                       FROM missions WHERE 1=1"
            .to_string();

        if filter.sprint_id.is_some() {
            sql.push_str(" AND sprint_id = ?");
        }
        if filter.status.is_some() {
            sql.push_str(" AND status = ?");
        } else if !filter.include_completed {
            sql.push_str(" AND status = 'active'");
        }
        sql.push_str(" ORDER BY id DESC");

        let mut q = sqlx::query_as::<_, Mission>(&sql);
        if let Some(s) = filter.sprint_id {
            q = q.bind(s);
        }
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
        // 존재 확인
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

        if input.sprint_id.is_some() {
            // sprint_id 는 None 으로의 명시 변경도 지원하므로 Option<Option<i64>> 패턴이 이상적이나
            // UpdateMissionInput 스펙 유지를 위해 Some 이 들어온 경우만 업데이트한다.
            let sid = input.sprint_id;
            sqlx::query(
                "UPDATE missions SET sprint_id = ?, updated_at = datetime('now') WHERE id = ?",
            )
            .bind(sid)
            .bind(id)
            .execute(&self.pool)
            .await?;
            let _ = self
                .history_record(CreateHistoryInput {
                    entity_type: EntityType::Mission,
                    entity_id: id,
                    field: "sprint_id".to_string(),
                    old_value: None,
                    new_value: sid.map(|s| s.to_string()),
                    changed_by: changed_by.to_string(),
                })
                .await;
        }

        self.mission_get(id).await
    }

    /// 미션별 종합 진척도를 집계하여 반환한다.
    ///
    /// - `issues.mission_id` 기준으로 직접 집계 (epic 경유 불필요).
    /// - 이슈가 0건이면 모든 카운터는 0, `progress_rate` 는 0.0.
    /// - `todo_issues` = required + ready, `progress_rate` = finished / total.
    pub async fn mission_progress_query(&self, id: i64) -> Result<MissionProgress> {
        use crate::models::mission::MissionProgress;

        // 존재 확인 — NotFound를 명시적으로 반환하기 위해
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
             LEFT JOIN issues i ON i.mission_id = m.id
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
            "SELECT id, jira_key, title, description, status, sprint_id, created_at, updated_at \
             FROM missions WHERE jira_key = ?",
        )
        .bind(jira_key)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| Error::NotFound(format!("mission with jira_key:{jira_key}")))
    }

    /// 미션의 계층 트리를 반환한다: Mission → Vec<EpicWithIssues>.
    ///
    /// 3회 SELECT + 앱 레이어 그룹핑으로 N+1 없음:
    /// 1. mission_get(id)
    /// 2. epics WHERE mission_id = id
    /// 3. issues WHERE mission_id = id
    pub async fn mission_get_tree(&self, id: i64) -> Result<MissionTree> {
        let mission = self.mission_get(id).await?;

        // sprint_name 조회 (sprint_id 있을 때만)
        let sprint_name = if let Some(sid) = mission.sprint_id {
            sqlx::query_scalar::<_, Option<String>>("SELECT name FROM sprints WHERE id = ?")
                .bind(sid)
                .fetch_optional(&self.pool)
                .await?
                .flatten()
        } else {
            None
        };

        // mission_id로 에픽 목록 조회
        let epics = sqlx::query_as::<_, Epic>(
            "SELECT id, project_key, mission_id, title, description, status, created_at, updated_at \
             FROM epics WHERE mission_id = ? ORDER BY id ASC",
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await?;

        // mission_id로 이슈 전체 조회
        let all_issues = sqlx::query_as::<_, Issue>(
            "SELECT id, epic_id, mission_id, sprint_id, title, description, goal, status, priority, \
             assigned_agent, created_at, updated_at \
             FROM issues WHERE mission_id = ? ORDER BY id ASC",
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await?;

        // 앱 레이어에서 epic별 그룹핑
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
            sprint_name,
        })
    }

    /// 미션의 스프린트 소속을 변경하거나 백로그(None)로 내린다.
    ///
    /// - completed 미션에는 변경 불가 → `Error::Validation`
    /// - 존재하지 않는 mission_id → `Error::NotFound`
    /// - history 에 `sprint_id` 변경 기록
    pub async fn mission_set_sprint(
        &self,
        mission_id: i64,
        sprint_id: Option<i64>,
        changed_by: &str,
    ) -> Result<Mission> {
        let current = self
            .mission_get(mission_id)
            .await
            .map_err(|_| Error::NotFound(format!("mission:{mission_id}")))?;

        if current.status == MissionStatus::Completed {
            return Err(Error::Validation(
                "completed mission cannot change sprint".to_string(),
            ));
        }

        let old_value = current
            .sprint_id
            .map(|s| s.to_string())
            .unwrap_or_else(|| "null".to_string());
        let new_value = sprint_id
            .map(|s| s.to_string())
            .unwrap_or_else(|| "null".to_string());

        sqlx::query(
            "UPDATE missions SET sprint_id = ?, updated_at = datetime('now') WHERE id = ?",
        )
        .bind(sprint_id)
        .bind(mission_id)
        .execute(&self.pool)
        .await?;

        let _ = self
            .history_record(CreateHistoryInput {
                entity_type: EntityType::Mission,
                entity_id: mission_id,
                field: "sprint_id".to_string(),
                old_value: Some(old_value),
                new_value: Some(new_value),
                changed_by: changed_by.to_string(),
            })
            .await;

        self.mission_get(mission_id).await
    }

    pub async fn mission_delete(&self, id: i64) -> Result<()> {
        // 존재 확인
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

    #[tokio::test]
    async fn test_mission_create_and_get() {
        let db = setup_db().await;
        let m = db
            .mission_create(CreateMissionInput {
                title: "M1".to_string(),
                description: None,
                jira_key: None,
                sprint_id: None,
            })
            .await
            .unwrap();
        assert_eq!(m.title, "M1");
        assert_eq!(m.status, MissionStatus::Active);
        assert!(m.jira_key.is_none());
        assert!(m.sprint_id.is_none());
    }

    #[tokio::test]
    async fn test_mission_create_duplicate_jira_key_fails() {
        let db = setup_db().await;
        db.mission_create(CreateMissionInput {
            title: "M1".to_string(),
            description: None,
            jira_key: Some("PROJ-1".to_string()),
            sprint_id: None,
        })
        .await
        .unwrap();

        let result = db
            .mission_create(CreateMissionInput {
                title: "M2".to_string(),
                description: None,
                jira_key: Some("PROJ-1".to_string()),
                sprint_id: None,
            })
            .await;
        assert!(
            result.is_err(),
            "중복 jira_key로 생성 시 에러가 발생해야 함"
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("PROJ-1"),
            "에러 메시지에 jira_key 포함: {err_msg}"
        );
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
        db.mission_create(CreateMissionInput {
            title: "Active".to_string(),
            description: None,
            jira_key: None,
            sprint_id: None,
        })
        .await
        .unwrap();
        let m2 = db
            .mission_create(CreateMissionInput {
                title: "Completed".to_string(),
                description: None,
                jira_key: None,
                sprint_id: None,
            })
            .await
            .unwrap();
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
        assert_eq!(list.len(), 1, "기본값은 active only이므로 1건만 반환");
        assert_eq!(list[0].title, "Active");
    }

    #[tokio::test]
    async fn test_mission_list_include_completed() {
        let db = setup_db().await;
        db.mission_create(CreateMissionInput {
            title: "Active".to_string(),
            description: None,
            jira_key: None,
            sprint_id: None,
        })
        .await
        .unwrap();
        let m2 = db
            .mission_create(CreateMissionInput {
                title: "Completed".to_string(),
                description: None,
                jira_key: None,
                sprint_id: None,
            })
            .await
            .unwrap();
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
        assert_eq!(list.len(), 2, "include_completed=true이면 2건 반환");
    }

    #[tokio::test]
    async fn test_mission_update_status_recorded_in_history() {
        let db = setup_db().await;
        let m = db
            .mission_create(CreateMissionInput {
                title: "M1".to_string(),
                description: None,
                jira_key: None,
                sprint_id: None,
            })
            .await
            .unwrap();

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
        assert!(
            history.iter().any(|h| h.field == "status"),
            "status 변경이 history에 기록되어야 함"
        );
        assert!(
            history.iter().any(|h| h.field == "title"),
            "title 변경이 history에 기록되어야 함"
        );
    }

    #[tokio::test]
    async fn test_mission_delete_blocked_by_child_epic() {
        let db = setup_db().await;
        let m = db
            .mission_create(CreateMissionInput {
                title: "M1".to_string(),
                description: None,
                jira_key: None,
                sprint_id: None,
            })
            .await
            .unwrap();

        // epic_create에 mission_id를 넣으려면 직접 INSERT 사용
        sqlx::query(
            "INSERT INTO epics (project_key, mission_id, title) VALUES ('test', ?, 'E1')",
        )
        .bind(m.id)
        .execute(&db.pool)
        .await
        .unwrap();

        let result = db.mission_delete(m.id).await;
        assert!(result.is_err(), "하위 epic이 있으면 mission_delete가 실패해야 함");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("child epics"),
            "에러 메시지에 'child epics' 포함: {err_msg}"
        );
    }

    #[tokio::test]
    async fn test_mission_delete_success() {
        let db = setup_db().await;
        let m = db
            .mission_create(CreateMissionInput {
                title: "ToDelete".to_string(),
                description: None,
                jira_key: None,
                sprint_id: None,
            })
            .await
            .unwrap();

        db.mission_delete(m.id).await.unwrap();

        let result = db.mission_get(m.id).await;
        assert!(result.is_err(), "삭제 후 조회 시 NotFound 에러");
    }

    #[tokio::test]
    async fn test_mission_progress_query_empty() {
        let db = setup_db().await;
        let m = db
            .mission_create(CreateMissionInput {
                title: "M".to_string(),
                description: None,
                jira_key: None,
                sprint_id: None,
            })
            .await
            .unwrap();

        let p = db.mission_progress_query(m.id).await.unwrap();
        assert_eq!(p.id, m.id);
        assert_eq!(p.title, "M");
        assert_eq!(p.issues_count, 0, "이슈 없음");
        assert_eq!(p.epics_count, 0, "에픽 없음");
        assert_eq!(p.todo_issues, 0);
        assert_eq!(p.working_issues, 0);
        assert_eq!(p.demo_issues, 0);
        assert_eq!(p.finished_issues, 0);
        assert_eq!(p.cancelled_issues, 0);
        assert_eq!(p.progress_rate, 0.0, "이슈 없으면 0.0");
    }

    #[tokio::test]
    async fn test_mission_progress_query_calculates_correctly() {
        let db = setup_db().await;
        let m = db
            .mission_create(CreateMissionInput {
                title: "M".to_string(),
                description: None,
                jira_key: None,
                sprint_id: None,
            })
            .await
            .unwrap();

        // epic 직접 INSERT (epic_create API에는 mission_id 파라미터 없음)
        sqlx::query(
            "INSERT INTO epics (project_key, mission_id, title) VALUES ('test', ?, 'E1')",
        )
        .bind(m.id)
        .execute(&db.pool)
        .await
        .unwrap();

        let epic_id: i64 =
            sqlx::query_scalar("SELECT id FROM epics WHERE mission_id = ?")
                .bind(m.id)
                .fetch_one(&db.pool)
                .await
                .unwrap();

        // 이슈 5개: 1 finished, 1 working, 1 required, 1 demo, 1 cancelled
        for (status, title) in [
            ("finished", "I1"),
            ("working", "I2"),
            ("required", "I3"),
            ("demo", "I4"),
            ("cancelled", "I5"),
        ] {
            sqlx::query(
                "INSERT INTO issues (epic_id, mission_id, title, status, priority) \
                 VALUES (?, ?, ?, ?, 'medium')",
            )
            .bind(epic_id)
            .bind(m.id)
            .bind(title)
            .bind(status)
            .execute(&db.pool)
            .await
            .unwrap();
        }

        let p = db.mission_progress_query(m.id).await.unwrap();
        assert_eq!(p.issues_count, 5, "전체 이슈 5건");
        assert_eq!(p.epics_count, 1, "에픽 1건");
        assert_eq!(p.finished_issues, 1);
        assert_eq!(p.working_issues, 1);
        assert_eq!(p.todo_issues, 1, "required만 (ready 없음)");
        assert_eq!(p.demo_issues, 1);
        assert_eq!(p.cancelled_issues, 1);
        // progress_rate = 1 finished / 5 total = 0.2
        assert!(
            (p.progress_rate - 0.2).abs() < 0.001,
            "progress_rate = 1/5 = 0.2, got {}",
            p.progress_rate
        );
    }

    #[tokio::test]
    async fn test_mission_progress_query_not_found() {
        let db = setup_db().await;
        let result = db.mission_progress_query(9999).await;
        assert!(result.is_err(), "존재하지 않는 mission_id → NotFound");
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("mission:9999"), "{err_msg}");
    }

    #[tokio::test]
    async fn test_mission_get_by_jira_key() {
        let db = setup_db().await;
        let m = db
            .mission_create(CreateMissionInput {
                title: "JiraKeyMission".to_string(),
                description: None,
                jira_key: Some("PROJ-42".to_string()),
                sprint_id: None,
            })
            .await
            .unwrap();

        let found = db.mission_get_by_jira_key("PROJ-42").await.unwrap();
        assert_eq!(found.id, m.id, "jira_key로 조회한 mission id가 일치해야 함");

        let not_found = db.mission_get_by_jira_key("PROJ-NONEXISTENT").await;
        assert!(not_found.is_err(), "존재하지 않는 jira_key → NotFound");
    }

    #[tokio::test]
    async fn test_mission_get_tree_groups_correctly() {
        let db = setup_db().await;
        let m = db
            .mission_create(CreateMissionInput {
                title: "Tree Test".to_string(),
                description: None,
                jira_key: None,
                sprint_id: None,
            })
            .await
            .unwrap();

        // epic 직접 INSERT (epic_create API에는 mission_id 파라미터 없음)
        sqlx::query(
            "INSERT INTO epics (project_key, mission_id, title) VALUES ('p1', ?, 'E1')",
        )
        .bind(m.id)
        .execute(&db.pool)
        .await
        .unwrap();

        let epic_id: i64 = sqlx::query_scalar("SELECT id FROM epics WHERE mission_id = ?")
            .bind(m.id)
            .fetch_one(&db.pool)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO issues (epic_id, mission_id, title, priority) VALUES (?, ?, 'I1', 'medium')",
        )
        .bind(epic_id)
        .bind(m.id)
        .execute(&db.pool)
        .await
        .unwrap();

        let tree = db.mission_get_tree(m.id).await.unwrap();
        assert_eq!(tree.mission.id, m.id, "mission id 일치");
        assert_eq!(tree.epics.len(), 1, "에픽 1건");
        assert_eq!(tree.epics[0].issues.len(), 1, "이슈 1건");
        assert_eq!(tree.epics[0].epic.id, epic_id, "epic id 일치");
        assert!(tree.sprint_name.is_none(), "sprint_id 없으면 sprint_name은 None");
    }

    #[tokio::test]
    async fn test_mission_get_tree_multiple_epics() {
        let db = setup_db().await;
        let m = db
            .mission_create(CreateMissionInput {
                title: "Multi Epic Mission".to_string(),
                description: None,
                jira_key: None,
                sprint_id: None,
            })
            .await
            .unwrap();

        // 에픽 2개 추가
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

        let epic_ids: Vec<i64> =
            sqlx::query_scalar("SELECT id FROM epics WHERE mission_id = ? ORDER BY id ASC")
                .bind(m.id)
                .fetch_all(&db.pool)
                .await
                .unwrap();

        // Epic A에 이슈 2개, Epic B에 이슈 1개
        for title in ["Issue A1", "Issue A2"] {
            sqlx::query(
                "INSERT INTO issues (epic_id, mission_id, title, priority) VALUES (?, ?, ?, 'medium')",
            )
            .bind(epic_ids[0])
            .bind(m.id)
            .bind(title)
            .execute(&db.pool)
            .await
            .unwrap();
        }
        sqlx::query(
            "INSERT INTO issues (epic_id, mission_id, title, priority) VALUES (?, ?, 'Issue B1', 'medium')",
        )
        .bind(epic_ids[1])
        .bind(m.id)
        .execute(&db.pool)
        .await
        .unwrap();

        let tree = db.mission_get_tree(m.id).await.unwrap();
        assert_eq!(tree.epics.len(), 2, "에픽 2건");
        assert_eq!(tree.epics[0].issues.len(), 2, "Epic A에 이슈 2건");
        assert_eq!(tree.epics[1].issues.len(), 1, "Epic B에 이슈 1건");
    }

    #[tokio::test]
    async fn test_mission_get_tree_not_found() {
        let db = setup_db().await;
        let result = db.mission_get_tree(9999).await;
        assert!(result.is_err(), "존재하지 않는 id → NotFound");
    }

    #[tokio::test]
    async fn test_mission_set_sprint_moves() {
        let db = setup_db().await;
        // sprint A, sprint B 생성
        let sprint_a = db
            .sprint_create(crate::models::sprint::CreateSprintInput {
                name: "Sprint A".to_string(),
                goal: None,
                start_date: None,
                end_date: None,
            })
            .await
            .unwrap();
        let sprint_b = db
            .sprint_create(crate::models::sprint::CreateSprintInput {
                name: "Sprint B".to_string(),
                goal: None,
                start_date: None,
                end_date: None,
            })
            .await
            .unwrap();

        // mission(sprint_id=A) 생성
        let m = db
            .mission_create(CreateMissionInput {
                title: "Move Me".to_string(),
                description: None,
                jira_key: None,
                sprint_id: Some(sprint_a.id),
            })
            .await
            .unwrap();
        assert_eq!(m.sprint_id, Some(sprint_a.id), "초기 sprint_id = A");

        // sprint B로 이동
        let updated = db
            .mission_set_sprint(m.id, Some(sprint_b.id), "test-agent")
            .await
            .unwrap();
        assert_eq!(
            updated.sprint_id,
            Some(sprint_b.id),
            "mission_set_sprint 후 sprint_id = B"
        );

        // history에 sprint_id 변경이 기록됐는지 확인
        let history = db
            .history_list(crate::models::EntityType::Mission, m.id)
            .await
            .unwrap();
        assert!(
            history.iter().any(|h| h.field == "sprint_id"),
            "sprint_id 변경이 history에 기록되어야 함"
        );
    }

    #[tokio::test]
    async fn test_mission_set_sprint_to_backlog() {
        let db = setup_db().await;
        let sprint_a = db
            .sprint_create(crate::models::sprint::CreateSprintInput {
                name: "Sprint A".to_string(),
                goal: None,
                start_date: None,
                end_date: None,
            })
            .await
            .unwrap();

        let m = db
            .mission_create(CreateMissionInput {
                title: "Backlog Me".to_string(),
                description: None,
                jira_key: None,
                sprint_id: Some(sprint_a.id),
            })
            .await
            .unwrap();

        // None(백로그)으로 이동
        let updated = db
            .mission_set_sprint(m.id, None, "test-agent")
            .await
            .unwrap();
        assert!(
            updated.sprint_id.is_none(),
            "백로그 이동 후 sprint_id = None"
        );
    }

    #[tokio::test]
    async fn test_mission_set_sprint_completed_fails() {
        let db = setup_db().await;
        let sprint_a = db
            .sprint_create(crate::models::sprint::CreateSprintInput {
                name: "Sprint A".to_string(),
                goal: None,
                start_date: None,
                end_date: None,
            })
            .await
            .unwrap();

        let m = db
            .mission_create(CreateMissionInput {
                title: "Completed Mission".to_string(),
                description: None,
                jira_key: None,
                sprint_id: Some(sprint_a.id),
            })
            .await
            .unwrap();

        // completed 상태로 변경
        db.mission_update(
            m.id,
            UpdateMissionInput {
                status: Some(MissionStatus::Completed),
                ..Default::default()
            },
            "test",
        )
        .await
        .unwrap();

        // completed 미션에 스프린트 변경 시도 → Validation 에러
        let result = db.mission_set_sprint(m.id, None, "test-agent").await;
        assert!(result.is_err(), "completed 미션은 sprint 변경 불가");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("completed mission cannot change sprint"),
            "에러 메시지 확인: {err_msg}"
        );
    }

    #[tokio::test]
    async fn test_mission_set_sprint_not_found() {
        let db = setup_db().await;
        let result = db.mission_set_sprint(9999, None, "test-agent").await;
        assert!(result.is_err(), "존재하지 않는 mission_id → NotFound");
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("mission:9999"), "{err_msg}");
    }
}
