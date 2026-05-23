use crate::models::history::{CreateHistoryInput, EntityType};
use crate::models::epic::*;
use crate::{Db, Error, Result};

impl Db {
    pub async fn epic_create(&self, input: CreateEpicInput) -> Result<Epic> {
        sqlx::query_as::<_, Epic>(
            "INSERT INTO epics (project_key, mission_id, title, description) VALUES (?, ?, ?, ?)
             RETURNING id, project_key, mission_id, title, description, status, created_at, updated_at",
        )
        .bind(&input.project_key)
        .bind(input.mission_id)
        .bind(&input.title)
        .bind(&input.description)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn epic_get(&self, id: i64) -> Result<Epic> {
        sqlx::query_as::<_, Epic>(
            "SELECT id, project_key, mission_id, title, description, status, created_at, updated_at FROM epics WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| Error::NotFound(format!("epic:{id}")))
    }

    /// 에픽 목록. 에픽은 sprint-agnostic 이므로 project_key 와 status 로만 필터한다.
    ///
    /// `include_completed = false` (기본): status != 'completed' 인 에픽만 반환.
    /// `include_completed = true`: 전체 반환.
    pub async fn epic_list(
        &self,
        project_key: Option<&str>,
        include_completed: bool,
    ) -> Result<Vec<Epic>> {
        let mut sql = "SELECT id, project_key, mission_id, title, description, status, created_at, updated_at FROM epics WHERE 1=1".to_string();
        if project_key.is_some() { sql.push_str(" AND project_key = ?"); }
        if !include_completed { sql.push_str(" AND status != 'completed'"); }
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

    /// 에픽을 수정한다.
    ///
    /// 반환값: `(Epic, cascade_updated, cascade_skipped)`
    /// - `cascade_updated`: mission_id cascade 시 실제 업데이트된 이슈 수
    /// - `cascade_skipped`: working/demo 상태로 보호되어 cascade 에서 제외된 이슈 수
    /// cascade 가 발생하지 않으면 둘 다 0.
    pub async fn epic_update(&self, id: i64, input: UpdateEpicInput, changed_by: &str) -> Result<(Epic, i64, i64)> {
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

        let mut cascade_updated: i64 = 0;
        let mut cascade_skipped: i64 = 0;

        if let Some(mid) = input.mission_id {
            let mut tx = self.pool.begin().await?;

            // 에픽 mission_id 업데이트
            sqlx::query(
                "UPDATE epics SET mission_id = ?, updated_at = datetime('now') WHERE id = ?",
            )
            .bind(mid)
            .bind(id)
            .execute(&mut *tx)
            .await?;

            // cascade: 하위 이슈 mission_id 일괄 갱신 (기본 true)
            // working/demo 상태 이슈는 에이전트 컨텍스트 충돌 방지를 위해 제외
            if input.cascade_issues {
                let affected = sqlx::query(
                    "UPDATE issues SET mission_id = ?, updated_at = datetime('now') \
                     WHERE epic_id = ? AND status NOT IN ('working', 'demo')",
                )
                .bind(mid)
                .bind(id)
                .execute(&mut *tx)
                .await?
                .rows_affected() as i64;

                // working/demo 이슈 수 — 보호된 이슈
                let skipped = sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM issues WHERE epic_id = ? AND status IN ('working', 'demo')",
                )
                .bind(id)
                .fetch_one(&mut *tx)
                .await?;

                cascade_updated = affected;
                cascade_skipped = skipped;

                if affected > 0 || skipped > 0 {
                    // cascade 감사 기록 (updated/skipped 포함)
                    sqlx::query(
                        "INSERT INTO history (entity_type, entity_id, field, old_value, new_value, changed_by) \
                         VALUES ('epic', ?, 'mission_id_cascade', NULL, ?, ?)",
                    )
                    .bind(id)
                    .bind(format!("{}:updated={},skipped={}", mid, affected, skipped))
                    .bind("cascade")
                    .execute(&mut *tx)
                    .await?;
                }
            }

            tx.commit().await?;

            // epic history 기록 (트랜잭션 외부에서 개별 기록)
            let _ = self.history_record(CreateHistoryInput {
                entity_type: EntityType::Epic,
                entity_id: id,
                field: "mission_id".to_string(),
                old_value: None,
                new_value: Some(mid.to_string()),
                changed_by: changed_by.to_string(),
            }).await;
        }

        let epic = self.epic_get(id).await?;
        Ok((epic, cascade_updated, cascade_skipped))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Db,
        models::{
            epic::{CreateEpicInput, EpicStatus, UpdateEpicInput},
            issue::{CreateIssueInput, UpdateIssueInput, IssueStatus},
            mission::CreateMissionInput,
        },
    };

    async fn setup() -> Db {
        Db::open_in_memory().await.unwrap()
    }

    #[tokio::test]
    async fn test_epic_list_excludes_completed_by_default() {
        let db = setup().await;

        let mission = db.mission_create(CreateMissionInput {
            title: "M".to_string(), description: None, jira_key: None, sprint_id: None,
        }).await.unwrap();

        let e1 = db.epic_create(CreateEpicInput {
            project_key: "p".to_string(), mission_id: Some(mission.id),
            title: "Active Epic".to_string(), description: None,
        }).await.unwrap();
        let e2 = db.epic_create(CreateEpicInput {
            project_key: "p".to_string(), mission_id: Some(mission.id),
            title: "Completed Epic".to_string(), description: None,
        }).await.unwrap();

        // e2를 completed 상태로 변경
        db.epic_update(e2.id, UpdateEpicInput {
            status: Some(EpicStatus::Completed), ..Default::default()
        }, "test").await.unwrap();

        // 기본(include_completed=false): completed 제외
        let default_list = db.epic_list(None, false).await.unwrap();
        assert_eq!(default_list.len(), 1, "기본값은 completed 제외, 1건이어야 함");
        assert_eq!(default_list[0].id, e1.id, "active 에픽만 반환");

        // include_completed=true: 전체 반환
        let full_list = db.epic_list(None, true).await.unwrap();
        assert_eq!(full_list.len(), 2, "include_completed=true이면 2건 반환");
    }

    #[tokio::test]
    async fn test_epic_list_include_completed_with_project_filter() {
        let db = setup().await;

        let mission = db.mission_create(CreateMissionInput {
            title: "M".to_string(), description: None, jira_key: None, sprint_id: None,
        }).await.unwrap();

        db.epic_create(CreateEpicInput {
            project_key: "proj-a".to_string(), mission_id: Some(mission.id),
            title: "A Active".to_string(), description: None,
        }).await.unwrap();
        let e2 = db.epic_create(CreateEpicInput {
            project_key: "proj-a".to_string(), mission_id: Some(mission.id),
            title: "A Completed".to_string(), description: None,
        }).await.unwrap();
        db.epic_create(CreateEpicInput {
            project_key: "proj-b".to_string(), mission_id: Some(mission.id),
            title: "B Active".to_string(), description: None,
        }).await.unwrap();

        db.epic_update(e2.id, UpdateEpicInput {
            status: Some(EpicStatus::Completed), ..Default::default()
        }, "test").await.unwrap();

        // proj-a, include_completed=false → 1건
        let filtered = db.epic_list(Some("proj-a"), false).await.unwrap();
        assert_eq!(filtered.len(), 1, "proj-a active만 1건");

        // proj-a, include_completed=true → 2건
        let all = db.epic_list(Some("proj-a"), true).await.unwrap();
        assert_eq!(all.len(), 2, "proj-a 전체 2건");
    }

    #[tokio::test]
    async fn test_epic_cascade_skips_working_issues() {
        let db = setup().await;

        // mission A, B 생성
        let mission_a = db.mission_create(CreateMissionInput {
            title: "Mission A".to_string(),
            description: None,
            jira_key: None,
            sprint_id: None,
        }).await.unwrap();

        let mission_b = db.mission_create(CreateMissionInput {
            title: "Mission B".to_string(),
            description: None,
            jira_key: None,
            sprint_id: None,
        }).await.unwrap();

        // epic 생성 (mission_id = A)
        let epic = db.epic_create(CreateEpicInput {
            project_key: "test-proj".to_string(),
            mission_id: Some(mission_a.id),
            title: "Test Epic".to_string(),
            description: None,
        }).await.unwrap();

        // issue 1 생성 (mission_id = A)
        let issue1 = db.issue_create(CreateIssueInput {
            epic_id: epic.id,
            sprint_id: None,
            mission_id: Some(mission_a.id),
            title: "Issue 1 (will be working)".to_string(),
            description: None,
            goal: None,
            priority: None,
        }).await.unwrap();

        // issue 2 생성 (mission_id = A)
        let issue2 = db.issue_create(CreateIssueInput {
            epic_id: epic.id,
            sprint_id: None,
            mission_id: Some(mission_a.id),
            title: "Issue 2 (will be cascaded)".to_string(),
            description: None,
            goal: None,
            priority: None,
        }).await.unwrap();

        // issue 3: demo 상태 (cascade 제외 확인용)
        let issue3 = db.issue_create(CreateIssueInput {
            epic_id: epic.id,
            sprint_id: None,
            mission_id: Some(mission_a.id),
            title: "Issue 3 (will be demo)".to_string(),
            description: None,
            goal: None,
            priority: None,
        }).await.unwrap();

        // issue 1 → working 상태로 전환 (ready → working은 issue_claim 경유)
        db.issue_update(issue1.id, UpdateIssueInput {
            status: Some(IssueStatus::Ready),
            ..Default::default()
        }, "agent").await.unwrap();
        db.issue_claim(issue1.id, "test-agent").await.unwrap();

        // issue 3 → demo 상태로 전환
        db.issue_update(issue3.id, UpdateIssueInput {
            status: Some(IssueStatus::Ready),
            ..Default::default()
        }, "agent").await.unwrap();
        db.issue_claim(issue3.id, "test-agent").await.unwrap();
        db.issue_release(issue3.id, IssueStatus::Demo, "test-agent", false).await.unwrap();

        // epic_update(mission_id = B, cascade_issues = true)
        let (updated_epic, cascade_updated, cascade_skipped) = db.epic_update(
            epic.id,
            UpdateEpicInput {
                mission_id: Some(mission_b.id),
                cascade_issues: true,
                ..Default::default()
            },
            "agent",
        ).await.unwrap();

        // epic 자체는 mission B로 변경
        assert_eq!(updated_epic.mission_id, Some(mission_b.id), "epic mission_id는 B여야 함");

        // cascade 카운트 검증
        assert_eq!(cascade_updated, 1, "issue 2만 cascade 업데이트 되어야 함");
        assert_eq!(cascade_skipped, 2, "issue 1(working), issue 3(demo) 2건이 보호되어야 함");

        // issue 1 (working): mission_id = A 유지
        let i1 = db.issue_get(issue1.id, false).await.unwrap();
        assert_eq!(i1.mission_id, Some(mission_a.id), "working 이슈는 mission_id A 유지");

        // issue 2 (required): mission_id = B 로 변경
        let i2 = db.issue_get(issue2.id, false).await.unwrap();
        assert_eq!(i2.mission_id, Some(mission_b.id), "required 이슈는 mission_id B로 변경");

        // issue 3 (demo): mission_id = A 유지
        let i3 = db.issue_get(issue3.id, false).await.unwrap();
        assert_eq!(i3.mission_id, Some(mission_a.id), "demo 이슈는 mission_id A 유지");
    }

    #[tokio::test]
    async fn test_epic_cascade_false_skips_all() {
        let db = setup().await;

        let mission_a = db.mission_create(CreateMissionInput {
            title: "Mission A".to_string(),
            description: None,
            jira_key: None,
            sprint_id: None,
        }).await.unwrap();

        let mission_b = db.mission_create(CreateMissionInput {
            title: "Mission B".to_string(),
            description: None,
            jira_key: None,
            sprint_id: None,
        }).await.unwrap();

        let epic = db.epic_create(CreateEpicInput {
            project_key: "test-proj".to_string(),
            mission_id: Some(mission_a.id),
            title: "Test Epic".to_string(),
            description: None,
        }).await.unwrap();

        let issue = db.issue_create(CreateIssueInput {
            epic_id: epic.id,
            sprint_id: None,
            mission_id: Some(mission_a.id),
            title: "Issue 1".to_string(),
            description: None,
            goal: None,
            priority: None,
        }).await.unwrap();

        // cascade_issues = false: 하위 이슈 mission_id 변경 없음
        let (updated_epic, cascade_updated, cascade_skipped) = db.epic_update(
            epic.id,
            UpdateEpicInput {
                mission_id: Some(mission_b.id),
                cascade_issues: false,
                ..Default::default()
            },
            "agent",
        ).await.unwrap();

        assert_eq!(updated_epic.mission_id, Some(mission_b.id));
        assert_eq!(cascade_updated, 0, "cascade=false이므로 0");
        assert_eq!(cascade_skipped, 0, "cascade=false이므로 0");

        // 이슈 mission_id는 A 유지
        let i = db.issue_get(issue.id, false).await.unwrap();
        assert_eq!(i.mission_id, Some(mission_a.id), "cascade=false이면 이슈 mission_id 유지");
    }
}
