use crate::{
    models::{
        issue::{CreateIssueInput, Issue},
        note::{CreateNoteInput, NoteType},
        retrospective::*,
    },
    Db, Result,
};

impl Db {
    pub async fn retrospective_create(
        &self,
        input: CreateRetrospectiveInput,
    ) -> Result<RetrospectiveWithItems> {
        let retro = sqlx::query_as::<_, Retrospective>(
            r#"INSERT INTO retrospectives (project_key, title, content, sprint_id, mission_id, epic_id, agent_id)
               VALUES (?, ?, ?, ?, ?, ?, ?)
               RETURNING id, project_key, title, content, sprint_id, mission_id, epic_id, agent_id, created_at, updated_at"#,
        )
        .bind(&input.project_key)
        .bind(&input.title)
        .bind(&input.content)
        .bind(input.sprint_id)
        .bind(input.mission_id)
        .bind(input.epic_id)
        .bind(&input.agent_id)
        .fetch_one(&self.pool)
        .await?;

        let mut items = Vec::new();
        if let Some(action_items) = input.action_items {
            for (idx, item_input) in action_items.into_iter().enumerate() {
                let ord = item_input.ord.unwrap_or((idx + 1) as f64);
                let item = sqlx::query_as::<_, RetroActionItem>(
                    r#"INSERT INTO retro_action_items (retro_id, title, description, linked_issue_id, linked_note_id, ord)
                       VALUES (?, ?, ?, ?, ?, ?)
                       RETURNING id, retro_id, title, description, status, linked_issue_id, linked_note_id, ord, created_at, updated_at"#,
                )
                .bind(retro.id)
                .bind(&item_input.title)
                .bind(&item_input.description)
                .bind(item_input.linked_issue_id)
                .bind(item_input.linked_note_id)
                .bind(ord)
                .fetch_one(&self.pool)
                .await?;
                items.push(item);
            }
        }

        Ok(RetrospectiveWithItems { retro, action_items: items })
    }

    pub async fn retrospective_get(&self, id: i64) -> Result<RetrospectiveWithItems> {
        let retro = sqlx::query_as::<_, Retrospective>(
            r#"SELECT id, project_key, title, content, sprint_id, mission_id, epic_id, agent_id, created_at, updated_at
               FROM retrospectives WHERE id = ?"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| crate::Error::NotFound(format!("Retrospective #{id} not found")))?;

        let action_items = sqlx::query_as::<_, RetroActionItem>(
            r#"SELECT id, retro_id, title, description, status, linked_issue_id, linked_note_id, ord, created_at, updated_at
               FROM retro_action_items WHERE retro_id = ? ORDER BY ord ASC, id ASC"#,
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await?;

        Ok(RetrospectiveWithItems { retro, action_items })
    }

    pub async fn retrospective_list(
        &self,
        project_key: Option<&str>,
        sprint_id: Option<i64>,
        limit: u32,
    ) -> Result<Vec<Retrospective>> {
        let limit = limit.min(100) as i64;
        let retros = match (project_key, sprint_id) {
            (Some(pk), Some(sid)) => {
                sqlx::query_as::<_, Retrospective>(
                    r#"SELECT id, project_key, title, content, sprint_id, mission_id, epic_id, agent_id, created_at, updated_at
                       FROM retrospectives WHERE project_key = ? AND sprint_id = ?
                       ORDER BY id DESC LIMIT ?"#,
                )
                .bind(pk)
                .bind(sid)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(pk), None) => {
                sqlx::query_as::<_, Retrospective>(
                    r#"SELECT id, project_key, title, content, sprint_id, mission_id, epic_id, agent_id, created_at, updated_at
                       FROM retrospectives WHERE project_key = ?
                       ORDER BY id DESC LIMIT ?"#,
                )
                .bind(pk)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
            (None, Some(sid)) => {
                sqlx::query_as::<_, Retrospective>(
                    r#"SELECT id, project_key, title, content, sprint_id, mission_id, epic_id, agent_id, created_at, updated_at
                       FROM retrospectives WHERE sprint_id = ?
                       ORDER BY id DESC LIMIT ?"#,
                )
                .bind(sid)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
            (None, None) => {
                sqlx::query_as::<_, Retrospective>(
                    r#"SELECT id, project_key, title, content, sprint_id, mission_id, epic_id, agent_id, created_at, updated_at
                       FROM retrospectives ORDER BY id DESC LIMIT ?"#,
                )
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
        };

        Ok(retros)
    }

    pub async fn retrospective_update(
        &self,
        id: i64,
        input: UpdateRetrospectiveInput,
        _agent_id: Option<&str>,
    ) -> Result<Retrospective> {
        let current = self.retrospective_get(id).await?.retro;

        let title = input.title.unwrap_or(current.title);
        let content = input.content.unwrap_or(current.content);
        let sprint_id = input.sprint_id.or(current.sprint_id);
        let mission_id = input.mission_id.or(current.mission_id);
        let epic_id = input.epic_id.or(current.epic_id);

        let updated = sqlx::query_as::<_, Retrospective>(
            r#"UPDATE retrospectives
               SET title = ?, content = ?, sprint_id = ?, mission_id = ?, epic_id = ?, updated_at = datetime('now')
               WHERE id = ?
               RETURNING id, project_key, title, content, sprint_id, mission_id, epic_id, agent_id, created_at, updated_at"#,
        )
        .bind(title)
        .bind(content)
        .bind(sprint_id)
        .bind(mission_id)
        .bind(epic_id)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated)
    }

    pub async fn retrospective_delete(&self, id: i64) -> Result<()> {
        let res = sqlx::query("DELETE FROM retrospectives WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if res.rows_affected() == 0 {
            return Err(crate::Error::NotFound(format!("Retrospective #{id} not found")));
        }
        Ok(())
    }

    pub async fn retro_action_item_create(
        &self,
        retro_id: i64,
        input: CreateRetroActionItemInput,
    ) -> Result<RetroActionItem> {
        let ord = input.ord.unwrap_or(1.0);
        let item = sqlx::query_as::<_, RetroActionItem>(
            r#"INSERT INTO retro_action_items (retro_id, title, description, linked_issue_id, linked_note_id, ord)
               VALUES (?, ?, ?, ?, ?, ?)
               RETURNING id, retro_id, title, description, status, linked_issue_id, linked_note_id, ord, created_at, updated_at"#,
        )
        .bind(retro_id)
        .bind(&input.title)
        .bind(&input.description)
        .bind(input.linked_issue_id)
        .bind(input.linked_note_id)
        .bind(ord)
        .fetch_one(&self.pool)
        .await?;

        Ok(item)
    }

    pub async fn retro_action_item_update(
        &self,
        id: i64,
        input: UpdateRetroActionItemInput,
    ) -> Result<RetroActionItem> {
        let current = sqlx::query_as::<_, RetroActionItem>(
            "SELECT id, retro_id, title, description, status, linked_issue_id, linked_note_id, ord, created_at, updated_at FROM retro_action_items WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| crate::Error::NotFound(format!("RetroActionItem #{id} not found")))?;

        let title = input.title.unwrap_or(current.title);
        let description = input.description.or(current.description);
        let status = input.status.unwrap_or(current.status);
        let linked_issue_id = input.linked_issue_id.or(current.linked_issue_id);
        let linked_note_id = input.linked_note_id.or(current.linked_note_id);
        let ord = input.ord.unwrap_or(current.ord);

        let updated = sqlx::query_as::<_, RetroActionItem>(
            r#"UPDATE retro_action_items
               SET title = ?, description = ?, status = ?, linked_issue_id = ?, linked_note_id = ?, ord = ?, updated_at = datetime('now')
               WHERE id = ?
               RETURNING id, retro_id, title, description, status, linked_issue_id, linked_note_id, ord, created_at, updated_at"#,
        )
        .bind(title)
        .bind(description)
        .bind(status)
        .bind(linked_issue_id)
        .bind(linked_note_id)
        .bind(ord)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated)
    }

    pub async fn retro_action_item_delete(&self, id: i64) -> Result<()> {
        let res = sqlx::query("DELETE FROM retro_action_items WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if res.rows_affected() == 0 {
            return Err(crate::Error::NotFound(format!("RetroActionItem #{id} not found")));
        }
        Ok(())
    }

    /// retro-{{sprint_name}} 미션과 Retrospective Action Items 에픽을 확보합니다.
    pub async fn ensure_retro_mission_and_epic(
        &self,
        project_key: &str,
        sprint_name: &str,
    ) -> Result<(i64, i64)> {
        let mission_title = format!("retro-{sprint_name}");

        // 1. Mission 확보
        let mission = match sqlx::query_as::<_, crate::models::mission::Mission>(
            "SELECT id, title, description, jira_key, status, created_at, updated_at FROM missions WHERE title = ?",
        )
        .bind(&mission_title)
        .fetch_optional(&self.pool)
        .await? {
            Some(m) => m,
            None => {
                self.mission_create(crate::models::mission::CreateMissionInput {
                    title: mission_title.clone(),
                    description: Some(format!("{sprint_name} 회고 액션 아이템 전용 미션")),
                    jira_key: None,
                }).await?
            }
        };

        // 2. Epic 확보 (Retrospective Action Items)
        let epic_title = "Retrospective Action Items";
        let epic = match sqlx::query_as::<_, crate::models::epic::Epic>(
            "SELECT id, sprint_id, mission_id, project_key, title, description, status, created_at, updated_at FROM epics WHERE mission_id = ? AND title = ?",
        )
        .bind(mission.id)
        .bind(epic_title)
        .fetch_optional(&self.pool)
        .await? {
            Some(e) => e,
            None => {
                let sprint_id = self.sprint_current().await.ok().flatten().map(|s| s.id);
                self.epic_create(crate::models::epic::CreateEpicInput {
                    project_key: project_key.to_string(),
                    mission_id: Some(mission.id),
                    sprint_id,
                    title: epic_title.to_string(),
                    description: Some("회고에서 생성된 액션 아이템 이슈 집합".to_string()),
                }).await?
            }
        };

        Ok((mission.id, epic.id))
    }

    /// 액션 아이템 1개를 이슈로 변환하여 생성하고 연결합니다.
    pub async fn retro_action_item_convert_to_issue(
        &self,
        item_id: i64,
        agent_id: Option<&str>,
    ) -> Result<Issue> {
        let item = sqlx::query_as::<_, RetroActionItem>(
            "SELECT id, retro_id, title, description, status, linked_issue_id, linked_note_id, ord, created_at, updated_at FROM retro_action_items WHERE id = ?",
        )
        .bind(item_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| crate::Error::NotFound(format!("RetroActionItem #{item_id} not found")))?;

        if let Some(existing_issue_id) = item.linked_issue_id {
            return self.issue_get(existing_issue_id, false).await;
        }

        let retro_with_items = self.retrospective_get(item.retro_id).await?;
        let retro = retro_with_items.retro;

        let sprint_name = if let Some(sid) = retro.sprint_id {
            self.sprint_get(sid).await.map(|s| s.name).unwrap_or_else(|_| "sprint".to_string())
        } else {
            retro.title.clone()
        };

        let (_mission_id, epic_id) = self.ensure_retro_mission_and_epic(&retro.project_key, &sprint_name).await?;

        let issue = self.issue_create(CreateIssueInput {
            epic_id,
            title: item.title.clone(),
            description: item.description.clone(),
            goal: Some(format!("회고 Retrospective #{0} 액션아이템에서 자동 변환됨", retro.id)),
            priority: None,
        }).await?;

        // 액션아이템에 linked_issue_id 연결
        sqlx::query("UPDATE retro_action_items SET linked_issue_id = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(issue.id)
            .bind(item_id)
            .execute(&self.pool)
            .await?;

        // 생성된 이슈에 context note 부착
        self.note_add(CreateNoteInput {
            issue_id: issue.id,
            task_id: None,
            note_type: NoteType::Context,
            summary: format!("회고 #{0} Action Item에서 자동 생성됨", retro.id),
            detail: Some(format!("원문 액션 제목: {0}", item.title)),
            author: agent_id.map(|s| s.to_string()),
            agent_id: agent_id.map(|s| s.to_string()),
            scope: None,
            scope_target_id: None,
            project_key: Some(retro.project_key.clone()),
        }).await?;

        Ok(issue)
    }

    /// 회고 내 아직 이슈가 연결되지 않은 모든 액션 아이템을 일괄 이슈로 변환합니다.
    pub async fn retrospective_bulk_convert_actions_to_issues(
        &self,
        retro_id: i64,
        agent_id: Option<&str>,
    ) -> Result<Vec<Issue>> {
        let retro_with_items = self.retrospective_get(retro_id).await?;
        let mut created_issues = Vec::new();

        for item in retro_with_items.action_items {
            if item.linked_issue_id.is_none() {
                let issue = self.retro_action_item_convert_to_issue(item.id, agent_id).await?;
                created_issues.push(issue);
            }
        }

        Ok(created_issues)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup() -> Db {
        Db::open_in_memory().await.unwrap()
    }

    #[tokio::test]
    async fn test_retrospective_crud_and_conversion() {
        let db = setup().await;

        let retro_res = db.retrospective_create(CreateRetrospectiveInput {
            project_key: "engram".to_string(),
            title: "Sprint 1 Retro".to_string(),
            content: "KPT content".to_string(),
            sprint_id: None,
            mission_id: None,
            epic_id: None,
            agent_id: Some("agent-1".to_string()),
            action_items: Some(vec![
                CreateRetroActionItemInput {
                    title: "Action 1".to_string(),
                    description: Some("Desc 1".to_string()),
                    linked_issue_id: None,
                    linked_note_id: None,
                    ord: Some(1.0),
                },
                CreateRetroActionItemInput {
                    title: "Action 2".to_string(),
                    description: None,
                    linked_issue_id: None,
                    linked_note_id: None,
                    ord: Some(2.0),
                },
            ]),
        }).await.unwrap();

        assert_eq!(retro_res.retro.title, "Sprint 1 Retro");
        assert_eq!(retro_res.action_items.len(), 2);

        // 개별 변환 테스트
        let action1_id = retro_res.action_items[0].id;
        let issue1 = db.retro_action_item_convert_to_issue(action1_id, Some("agent-1")).await.unwrap();
        assert_eq!(issue1.title, "Action 1");

        // 변환 후 조회시 linked_issue_id 연결 확인
        let get_res = db.retrospective_get(retro_res.retro.id).await.unwrap();
        assert_eq!(get_res.action_items[0].linked_issue_id, Some(issue1.id));

        // 일괄 변환 테스트 (나머지 1개 변환)
        let bulk_issues = db.retrospective_bulk_convert_actions_to_issues(retro_res.retro.id, Some("agent-1")).await.unwrap();
        assert_eq!(bulk_issues.len(), 1);
        assert_eq!(bulk_issues[0].title, "Action 2");
    }
}
