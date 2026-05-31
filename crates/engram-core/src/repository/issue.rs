use crate::models::history::{CreateHistoryInput, EntityType};
use crate::models::issue::*;
use crate::models::PaginatedResponse;
use crate::{Db, Error, Result};

const DESCRIPTION_EXCERPT_CHARS: usize = 100;

/// Issue 조회용 공통 SELECT 컬럼 — mission_id 와 sprint_id 는 Epic JOIN 으로 derive.
fn issue_select_columns(compact: bool) -> &'static str {
    if compact {
        "i.id, i.epic_id, e.mission_id AS mission_id, e.sprint_id AS sprint_id, \
         i.title, CASE WHEN i.description IS NOT NULL THEN SUBSTR(i.description, 1, 200) ELSE NULL END AS description, \
         CASE WHEN i.goal IS NOT NULL THEN SUBSTR(i.goal, 1, 200) ELSE NULL END AS goal, \
         i.status, i.priority, i.assigned_agent, i.created_at, i.updated_at, \
         NULL AS note_count, NULL AS task_count"
    } else {
        "i.id, i.epic_id, e.mission_id AS mission_id, e.sprint_id AS sprint_id, \
         i.title, i.description, i.goal, i.status, i.priority, \
         i.assigned_agent, i.created_at, i.updated_at, \
         NULL AS note_count, NULL AS task_count"
    }
}

impl Db {
    pub async fn issue_create(&self, input: CreateIssueInput) -> Result<Issue> {
        let priority = input.priority.unwrap_or(IssuePriority::Medium);
        let pval = serde_json::to_value(&priority).unwrap().as_str().unwrap().to_string();

        // epic 존재 확인 + Sprint/Mission derive 는 RETURNING 절의 JOIN 으로 처리.
        sqlx::query_as::<_, Issue>(
            "INSERT INTO issues (epic_id, title, description, goal, priority) VALUES (?, ?, ?, ?, ?) \
             RETURNING id, epic_id, \
                       (SELECT mission_id FROM epics WHERE id = epic_id) AS mission_id, \
                       (SELECT sprint_id  FROM epics WHERE id = epic_id) AS sprint_id, \
                       title, description, goal, status, priority, assigned_agent, created_at, updated_at",
        )
        .bind(input.epic_id)
        .bind(&input.title)
        .bind(&input.description)
        .bind(&input.goal)
        .bind(&pval)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn issue_get(&self, id: i64, compact: bool) -> Result<Issue> {
        let cols = issue_select_columns(compact);
        sqlx::query_as::<_, Issue>(&format!(
            "SELECT {cols} FROM issues i JOIN epics e ON i.epic_id = e.id WHERE i.id = ?"
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| Error::NotFound(format!("issue:{id}")))
    }

    pub async fn issue_list(&self, filter: IssueFilter) -> Result<PaginatedResponse<Issue>> {
        let is_compact = filter.compact.unwrap_or(false);
        let cols = if is_compact {
            "i.id, i.epic_id, e.mission_id AS mission_id, e.sprint_id AS sprint_id, \
             i.title, CASE WHEN i.description IS NOT NULL THEN SUBSTR(i.description, 1, 200) ELSE NULL END AS description, \
             CASE WHEN i.goal IS NOT NULL THEN SUBSTR(i.goal, 1, 200) ELSE NULL END AS goal, \
             i.status, i.priority, i.assigned_agent, i.created_at, i.updated_at, \
             COUNT(DISTINCT n.id) AS note_count, COUNT(DISTINCT t.id) AS task_count"
        } else {
            "i.id, i.epic_id, e.mission_id AS mission_id, e.sprint_id AS sprint_id, \
             i.title, i.description, i.goal, i.status, i.priority, \
             i.assigned_agent, i.created_at, i.updated_at, \
             NULL AS note_count, NULL AS task_count"
        };

        let mut from_clause = "FROM issues i JOIN epics e ON i.epic_id = e.id".to_string();
        if is_compact {
            from_clause.push_str(" LEFT JOIN notes n ON i.id = n.issue_id LEFT JOIN tasks t ON i.id = t.issue_id");
        }

        let mut where_clause = String::new();
        if filter.epic_id.is_some()     { where_clause.push_str(" AND i.epic_id = ?"); }
        if filter.mission_id.is_some()  { where_clause.push_str(" AND e.mission_id = ?"); }
        if filter.backlog_only {
            where_clause.push_str(" AND e.sprint_id IS NULL");
        } else if filter.sprint_id.is_some() {
            where_clause.push_str(" AND e.sprint_id = ?");
        }
        if filter.project_key.is_some() {
            where_clause.push_str(" AND e.project_key = ?");
        }

        let mut target_statuses = Vec::new();
        if let Some(s) = filter.status {
            target_statuses.push(s);
        }
        if let Some(ss) = filter.statuses {
            for s in ss {
                if !target_statuses.contains(&s) {
                    target_statuses.push(s);
                }
            }
        }

        if !target_statuses.is_empty() {
            let placeholders = target_statuses.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            where_clause.push_str(&format!(" AND i.status IN ({})", placeholders));
        }

        // 1) total count 쿼리 실행
        let count_sql = format!(
            "SELECT COUNT(DISTINCT i.id) FROM issues i JOIN epics e ON i.epic_id = e.id WHERE 1=1 {where_clause}"
        );
        let mut count_q = sqlx::query_scalar::<_, i64>(&count_sql);

        // 2) items 쿼리 실행
        let group_by = if is_compact { " GROUP BY i.id, e.mission_id, e.sprint_id" } else { "" };
        let (lim, off) = crate::repository::apply_pagination(filter.limit, filter.offset);
        let sql = format!(
            "SELECT {cols} {from_clause} WHERE 1=1 {where_clause} {group_by} ORDER BY i.id DESC LIMIT ? OFFSET ?"
        );
        let mut q = sqlx::query_as::<_, Issue>(&sql);

        // 바인딩
        if let Some(e) = filter.epic_id {
            count_q = count_q.bind(e);
            q = q.bind(e);
        }
        if let Some(m) = filter.mission_id {
            count_q = count_q.bind(m);
            q = q.bind(m);
        }
        if !filter.backlog_only {
            if let Some(s) = filter.sprint_id {
                count_q = count_q.bind(s);
                q = q.bind(s);
            }
        }
        if let Some(p) = filter.project_key {
            count_q = count_q.bind(p.clone());
            q = q.bind(p);
        }
        for s in target_statuses {
            let sv = serde_json::to_value(&s).unwrap().as_str().unwrap().to_string();
            count_q = count_q.bind(sv.clone());
            q = q.bind(sv);
        }

        // q 에만 pagination 바인딩
        q = q.bind(lim).bind(off);

        let total = count_q.fetch_one(&self.pool).await.unwrap_or(0);
        let items = q.fetch_all(&self.pool).await?;
        let has_more = (off + items.len() as i64) < total;

        Ok(PaginatedResponse { items, total, has_more })
    }

    pub async fn issue_update(&self, id: i64, input: UpdateIssueInput, changed_by: &str) -> Result<Issue> {
        if matches!(input.status.as_ref(), Some(IssueStatus::Finished) | Some(IssueStatus::Cancelled))
            && changed_by != "user"
        {
            return Err(Error::Validation(format!(
                "finished/cancelled 전이는 사용자(agent_id=\"user\")만 가능합니다 (현재 호출자: {}). agent_demo_gate 규칙 참조",
                changed_by
            )));
        }
        if let Some(ref new_status) = input.status {
            let current = self.issue_get(id, false).await?;
            if !current.status.can_transition_to(new_status) {
                return Err(crate::Error::Conflict(
                    format!("{:?} → {:?}", current.status, new_status)
                ));
            }
            if matches!(new_status, IssueStatus::Working | IssueStatus::Demo | IssueStatus::Finished) {
                let blockers = self.active_blockers_for(id).await?;
                if !blockers.is_empty() {
                    let list = blockers.iter().map(|b| format!("#{}", b)).collect::<Vec<_>>().join(", ");
                    return Err(crate::Error::Conflict(format!(
                        "이슈 #{}은(는) 블로킹 이슈 [{}]이(가) demo 또는 finished 상태가 될 때까지 작업이 불가합니다.",
                        id, list
                    )));
                }
            }
            let old_v = serde_json::to_value(&current.status).unwrap().as_str().unwrap().to_string();
            let new_v = serde_json::to_value(new_status).unwrap().as_str().unwrap().to_string();
            let clear_assignment = current.status == IssueStatus::Working && new_status != &IssueStatus::Working;
            if clear_assignment {
                sqlx::query("UPDATE issues SET status = ?, assigned_agent = NULL, updated_at = datetime('now') WHERE id = ?")
                    .bind(&new_v).bind(id).execute(&self.pool).await?;
            } else {
                sqlx::query("UPDATE issues SET status = ?, updated_at = datetime('now') WHERE id = ?")
                    .bind(&new_v).bind(id).execute(&self.pool).await?;
            }
            let _ = self.history_record(CreateHistoryInput {
                entity_type: EntityType::Issue,
                entity_id: id,
                field: "status".to_string(),
                old_value: Some(old_v),
                new_value: Some(new_v),
                changed_by: changed_by.to_string(),
            }).await;
        }
        if let Some(ref p) = input.priority {
            let pv = serde_json::to_value(p).unwrap().as_str().unwrap().to_string();
            sqlx::query("UPDATE issues SET priority = ?, updated_at = datetime('now') WHERE id = ?")
                .bind(&pv).bind(id).execute(&self.pool).await?;
            let _ = self.history_record(CreateHistoryInput {
                entity_type: EntityType::Issue,
                entity_id: id,
                field: "priority".to_string(),
                old_value: None,
                new_value: Some(pv),
                changed_by: changed_by.to_string(),
            }).await;
        }
        if let Some(ref title) = input.title {
            sqlx::query("UPDATE issues SET title = ?, updated_at = datetime('now') WHERE id = ?")
                .bind(title).bind(id).execute(&self.pool).await?;
            let _ = self.history_record(CreateHistoryInput {
                entity_type: EntityType::Issue,
                entity_id: id,
                field: "title".to_string(),
                old_value: None,
                new_value: Some(title.clone()),
                changed_by: changed_by.to_string(),
            }).await;
        }
        if let Some(ref desc) = input.description {
            sqlx::query("UPDATE issues SET description = ?, updated_at = datetime('now') WHERE id = ?")
                .bind(desc).bind(id).execute(&self.pool).await?;
            let _ = self.history_record(CreateHistoryInput {
                entity_type: EntityType::Issue,
                entity_id: id,
                field: "description".to_string(),
                old_value: None,
                new_value: Some(desc.clone()),
                changed_by: changed_by.to_string(),
            }).await;
        }
        if let Some(ref goal) = input.goal {
            sqlx::query("UPDATE issues SET goal = ?, updated_at = datetime('now') WHERE id = ?")
                .bind(goal).bind(id).execute(&self.pool).await?;
            let _ = self.history_record(CreateHistoryInput {
                entity_type: EntityType::Issue,
                entity_id: id,
                field: "goal".to_string(),
                old_value: None,
                new_value: Some(goal.clone()),
                changed_by: changed_by.to_string(),
            }).await;
        }
        // 이슈를 다른 epic 으로 이동 — mission/sprint 는 자동으로 epic 의 값에 따라온다.
        if let Some(new_epic_id) = input.epic_id {
            sqlx::query("UPDATE issues SET epic_id = ?, updated_at = datetime('now') WHERE id = ?")
                .bind(new_epic_id).bind(id).execute(&self.pool).await?;
            let _ = self.history_record(CreateHistoryInput {
                entity_type: EntityType::Issue,
                entity_id: id,
                field: "epic_id".to_string(),
                old_value: None,
                new_value: Some(new_epic_id.to_string()),
                changed_by: changed_by.to_string(),
            }).await;
        }
        self.issue_get(id, false).await
    }

    pub async fn issue_link(&self, source_id: i64, target_id: i64, link_type: LinkType) -> Result<IssueLink> {
        let lt = serde_json::to_value(&link_type).unwrap().as_str().unwrap().to_string();
        sqlx::query_as::<_, IssueLink>(
            "INSERT INTO issue_links (source_id, target_id, link_type) VALUES (?, ?, ?)
             RETURNING id, source_id, target_id, link_type, created_at",
        )
        .bind(source_id).bind(target_id).bind(&lt)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn issue_unlink(&self, link_id: i64) -> Result<()> {
        sqlx::query("DELETE FROM issue_links WHERE id = ?")
            .bind(link_id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn issue_claim(&self, id: i64, agent_id: &str) -> Result<Issue> {
        let current = self.issue_get(id, false).await?;

        let blockers = self.active_blockers_for(id).await?;
        if !blockers.is_empty() {
            let list = blockers.iter().map(|b| format!("#{}", b)).collect::<Vec<_>>().join(", ");
            return Err(crate::Error::Conflict(format!(
                "이슈 #{}은(는) 블로킹 이슈 [{}]이(가) demo 또는 finished 상태가 될 때까지 작업이 불가합니다.",
                id, list
            )));
        }

        let result = sqlx::query(
            "UPDATE issues \
             SET status='working', assigned_agent = ?, updated_at = datetime('now') \
             WHERE id = ? \
               AND status IN ('ready','working') \
               AND (assigned_agent IS NULL OR assigned_agent = ?)",
        )
        .bind(agent_id)
        .bind(id)
        .bind(agent_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(Error::Conflict(format!(
                "issue:{id} is already held by another agent (current: status={:?}, assigned_agent={:?})",
                current.status, current.assigned_agent
            )));
        }

        if current.status != IssueStatus::Working {
            let _ = self.history_record(CreateHistoryInput {
                entity_type: EntityType::Issue,
                entity_id: id,
                field: "status".to_string(),
                old_value: Some(serde_json::to_value(&current.status).unwrap().as_str().unwrap().to_string()),
                new_value: Some("working".to_string()),
                changed_by: agent_id.to_string(),
            }).await;
        }

        self.issue_get(id, false).await
    }

    pub async fn issue_release(&self, id: i64, transition_to: IssueStatus, agent_id: &str, force: bool) -> Result<Issue> {
        let current = self.issue_get(id, false).await?;

        if !force {
            if let Some(holder) = current.assigned_agent.as_deref() {
                if holder != agent_id {
                    return Err(Error::Conflict(format!(
                        "issue:{id} is held by '{holder}', cannot be released by '{agent_id}' (use force=true to override)"
                    )));
                }
            }
        }

        let new_v = serde_json::to_value(&transition_to).unwrap().as_str().unwrap().to_string();
        sqlx::query(
            "UPDATE issues SET status = ?, assigned_agent = NULL, updated_at = datetime('now') WHERE id = ?",
        )
        .bind(&new_v)
        .bind(id)
        .execute(&self.pool)
        .await?;

        if current.status != transition_to {
            let _ = self.history_record(CreateHistoryInput {
                entity_type: EntityType::Issue,
                entity_id: id,
                field: "status".to_string(),
                old_value: Some(serde_json::to_value(&current.status).unwrap().as_str().unwrap().to_string()),
                new_value: Some(new_v),
                changed_by: agent_id.to_string(),
            }).await;
        }

        self.issue_get(id, false).await
    }

    pub async fn issue_finish(&self, id: i64, changed_by: &str) -> Result<Issue> {
        if changed_by != "user" {
            return Err(Error::Validation("issue_finish 는 사용자 전용입니다".to_string()));
        }

        let current = self.issue_get(id, false).await?;
        if current.status != IssueStatus::Demo {
            return Err(Error::Conflict(format!(
                "demo 상태의 이슈만 finished 로 전이할 수 있습니다 (현재 상태: {:?})",
                current.status
            )));
        }

        sqlx::query("UPDATE issues SET status = 'finished', assigned_agent = NULL, updated_at = datetime('now') WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        let _ = self.history_record(CreateHistoryInput {
            entity_type: EntityType::Issue,
            entity_id: id,
            field: "status".to_string(),
            old_value: Some("demo".to_string()),
            new_value: Some("finished".to_string()),
            changed_by: changed_by.to_string(),
        }).await;

        self.issue_get(id, false).await
    }

    pub async fn issue_cancel(&self, id: i64, reason: &str, changed_by: &str) -> Result<Issue> {
        if changed_by != "user" {
            return Err(Error::Validation("issue_cancel 은 사용자 전용입니다".to_string()));
        }

        let current = self.issue_get(id, false).await?;
        if current.status == IssueStatus::Finished {
            return Err(Error::Conflict("이미 finished 로 종결된 이슈는 cancelled 로 전이할 수 없습니다".to_string()));
        }

        let old_status_str = serde_json::to_value(&current.status).unwrap().as_str().unwrap().to_string();

        sqlx::query("UPDATE issues SET status = 'cancelled', assigned_agent = NULL, updated_at = datetime('now') WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        let _ = self.history_record(CreateHistoryInput {
            entity_type: EntityType::Issue,
            entity_id: id,
            field: "status".to_string(),
            old_value: Some(old_status_str),
            new_value: Some("cancelled".to_string()),
            changed_by: changed_by.to_string(),
        }).await;

        let _ = self.history_record(CreateHistoryInput {
            entity_type: EntityType::Issue,
            entity_id: id,
            field: "cancel_reason".to_string(),
            old_value: None,
            new_value: Some(reason.to_string()),
            changed_by: changed_by.to_string(),
        }).await;

        self.issue_get(id, false).await
    }

    pub async fn issue_delete(&self, id: i64, changed_by: &str) -> Result<()> {
        let issue = self.issue_get(id, false).await?;
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            "DELETE FROM task_tests WHERE task_id IN (SELECT id FROM tasks WHERE issue_id = ?)",
        )
        .bind(id)
        .execute(&mut *tx)
        .await?;

        sqlx::query("DELETE FROM tasks WHERE issue_id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        sqlx::query("DELETE FROM notes WHERE issue_id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        sqlx::query("DELETE FROM issue_links WHERE source_id = ? OR target_id = ?")
            .bind(id)
            .bind(id)
            .execute(&mut *tx)
            .await?;

        sqlx::query("DELETE FROM issues WHERE id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            "INSERT INTO history (entity_type, entity_id, field, old_value, new_value, changed_by) VALUES ('issue', ?, 'deleted', ?, NULL, ?)",
        )
        .bind(id)
        .bind(issue.title)
        .bind(changed_by)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn issue_blocked_by(&self, issue_id: i64) -> Result<Vec<IssueLink>> {
        sqlx::query_as::<_, IssueLink>(
            "SELECT id, source_id, target_id, link_type, created_at FROM issue_links WHERE target_id = ? AND link_type = 'blocks'",
        )
        .bind(issue_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn stalled_issues(
        &self,
        project_key: Option<&str>,
        status: IssueStatus,
        threshold_minutes: i64,
    ) -> Result<Vec<StalledIssue>> {
        let status_v = serde_json::to_value(&status).unwrap().as_str().unwrap().to_string();

        let mut sql = String::from(
            "SELECT \
                i.id AS id, \
                i.title AS title, \
                e.project_key AS project_key, \
                i.status AS status, \
                i.priority AS priority, \
                COALESCE(MAX(h.created_at), i.updated_at) AS entered_status_at, \
                CAST((julianday('now') - julianday(COALESCE(MAX(h.created_at), i.updated_at))) * 24 * 60 AS INTEGER) AS minutes_in_status \
             FROM issues i \
             JOIN epics e ON e.id = i.epic_id \
             LEFT JOIN history h \
                ON h.entity_type = 'issue' AND h.entity_id = i.id \
               AND h.field = 'status' AND h.new_value = ? \
             WHERE i.status = ?",
        );
        if project_key.is_some() {
            sql.push_str(" AND e.project_key = ?");
        }
        sql.push_str(" GROUP BY i.id, i.title, e.project_key, i.status, i.priority, i.updated_at");
        sql.push_str(" HAVING minutes_in_status >= ?");
        sql.push_str(" ORDER BY minutes_in_status DESC");

        let mut q = sqlx::query_as::<_, StalledIssue>(&sql)
            .bind(&status_v)
            .bind(&status_v);
        if let Some(pk) = project_key {
            q = q.bind(pk);
        }
        q = q.bind(threshold_minutes);
        q.fetch_all(&self.pool).await.map_err(Into::into)
    }

    pub async fn issue_links_for(&self, issue_id: i64) -> Result<Vec<IssueLink>> {
        sqlx::query_as::<_, IssueLink>(
            "SELECT id, source_id, target_id, link_type, created_at FROM issue_links WHERE source_id = ? OR target_id = ? ORDER BY id ASC",
        )
        .bind(issue_id)
        .bind(issue_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn active_blockers_for(&self, issue_id: i64) -> Result<Vec<i64>> {
        sqlx::query_scalar::<_, i64>(
            "SELECT il.source_id              FROM issue_links il              JOIN issues i ON il.source_id = i.id              WHERE il.target_id = ?                AND il.link_type = 'blocks'                AND i.status NOT IN ('demo', 'finished', 'cancelled')",
        )
        .bind(issue_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Error::Db)
    }

    pub async fn planning_review_queue(
        &self,
        project_key: &str,
        sprint_id: Option<i64>,
        statuses: Option<Vec<IssueStatus>>,
    ) -> Result<IssuePlanningSnapshot> {
        let sprint = match sprint_id {
            Some(sid) => Some(self.sprint_get(sid).await?),
            None => self.sprint_current().await?,
        };

        let sid = sprint.as_ref().map(|s| s.id);
        let sname = sprint.as_ref().map(|s| s.name.clone());

        let cols = issue_select_columns(false);
        let mut sql = format!(
            "SELECT {cols} FROM issues i JOIN epics e ON i.epic_id = e.id WHERE e.project_key = ?"
        );

        if sid.is_some() {
            sql.push_str(" AND e.sprint_id = ?");
        } else {
            sql.push_str(" AND e.sprint_id IS NULL");
        }

        sql.push_str(" ORDER BY i.id DESC");

        let mut q = sqlx::query_as::<_, Issue>(&sql).bind(project_key);
        if let Some(s) = sid {
            q = q.bind(s);
        }

        let db_issues = q.fetch_all(&self.pool).await?;

        let mut items = Vec::new();
        for issue in db_issues {
            if let Some(ref s_list) = statuses {
                if !s_list.contains(&issue.status) {
                    continue;
                }
            }

            let description_excerpt = issue.description.as_ref().map(|d| {
                let mut iter = d.chars();
                let head: String = iter.by_ref().take(DESCRIPTION_EXCERPT_CHARS).collect();
                if iter.next().is_some() {
                    format!("{head}...")
                } else {
                    head
                }
            });

            let blockers = sqlx::query_scalar::<_, i64>(
                "SELECT source_id FROM issue_links WHERE target_id = ? AND link_type = 'blocks'"
            )
            .bind(issue.id)
            .fetch_all(&self.pool)
            .await?;

            let existing_context_note_count = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM notes WHERE issue_id = ? AND note_type = 'context'"
            )
            .bind(issue.id)
            .fetch_one(&self.pool)
            .await?;

            items.push(IssuePlanningItem {
                id: issue.id,
                title: issue.title,
                status: issue.status,
                priority: issue.priority,
                description_excerpt,
                blockers,
                existing_context_note_count,
                updated_at: issue.updated_at,
            });
        }

        Ok(IssuePlanningSnapshot {
            sprint_id: sid,
            sprint_name: sname,
            issues: items,
        })
    }

    pub async fn issue_bulk_update(
        &self,
        ids: Vec<i64>,
        input: BulkUpdateInput,
        changed_by: &str,
    ) -> Result<BulkUpdateResult> {
        let mut succeeded = Vec::new();
        let mut failed = Vec::new();

        for id in ids {
            let update_input = UpdateIssueInput {
                status: input.status.clone(),
                priority: input.priority.clone(),
                title: None,
                description: None,
                goal: None,
                epic_id: None,
            };
            match self.issue_update(id, update_input, changed_by).await {
                Ok(issue) => succeeded.push(issue),
                Err(e) => failed.push(BulkUpdateFailedItem {
                    id,
                    error: e.to_string(),
                }),
            }
        }

        Ok(BulkUpdateResult { succeeded, failed })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Db;
    use crate::models::epic::CreateEpicInput;
    use crate::models::mission::CreateMissionInput;
    use crate::models::sprint::CreateSprintInput;

    async fn setup_db() -> Db {
        Db::open_in_memory().await.expect("open in-memory db")
    }

    #[tokio::test]
    async fn test_issue_list_filtered_by_mission() {
        let db = setup_db().await;

        let sprint = db.sprint_create(CreateSprintInput {
            name: "S1".to_string(),
            goal: None,
            start_date: None,
            end_date: None,
        }).await.unwrap();

        let mission_a = db.mission_create(CreateMissionInput {
            title: "Mission A".to_string(),
            description: None,
            jira_key: None,
        }).await.unwrap();
        let mission_b = db.mission_create(CreateMissionInput {
            title: "Mission B".to_string(),
            description: None,
            jira_key: None,
        }).await.unwrap();

        let epic_a = db.epic_create(CreateEpicInput {
            mission_id: Some(mission_a.id),
            sprint_id: Some(sprint.id),
            project_key: "proj".to_string(),
            title: "Epic A".to_string(),
            description: None,
        }).await.unwrap();
        let epic_b = db.epic_create(CreateEpicInput {
            mission_id: Some(mission_b.id),
            sprint_id: Some(sprint.id),
            project_key: "proj".to_string(),
            title: "Epic B".to_string(),
            description: None,
        }).await.unwrap();

        let issue_a1 = db.issue_create(CreateIssueInput {
            epic_id: epic_a.id,
            title: "Issue A1".to_string(),
            description: None,
            goal: None,
            priority: None,
        }).await.unwrap();
        let issue_a2 = db.issue_create(CreateIssueInput {
            epic_id: epic_a.id,
            title: "Issue A2".to_string(),
            description: None,
            goal: None,
            priority: None,
        }).await.unwrap();
        let issue_b1 = db.issue_create(CreateIssueInput {
            epic_id: epic_b.id,
            title: "Issue B1".to_string(),
            description: None,
            goal: None,
            priority: None,
        }).await.unwrap();

        // mission_id 가 Epic 에서 derive 되는지 확인
        assert_eq!(issue_a1.mission_id, Some(mission_a.id));
        assert_eq!(issue_a2.mission_id, Some(mission_a.id));
        assert_eq!(issue_b1.mission_id, Some(mission_b.id));

        let result_a = db.issue_list(IssueFilter {
            mission_id: Some(mission_a.id),
            ..Default::default()
        }).await.unwrap();
        let ids_a: Vec<i64> = result_a.items.iter().map(|i| i.id).collect();
        assert!(ids_a.contains(&issue_a1.id));
        assert!(ids_a.contains(&issue_a2.id));
        assert!(!ids_a.contains(&issue_b1.id));
        assert_eq!(result_a.items.len(), 2);

        let result_b = db.issue_list(IssueFilter {
            mission_id: Some(mission_b.id),
            ..Default::default()
        }).await.unwrap();
        let ids_b: Vec<i64> = result_b.items.iter().map(|i| i.id).collect();
        assert!(ids_b.contains(&issue_b1.id));
        assert!(!ids_b.contains(&issue_a1.id));
        assert_eq!(result_b.items.len(), 1);
    }

    /// Helper: 테스트용 이슈를 required 상태로 생성
    async fn create_test_issue(db: &Db) -> crate::models::issue::Issue {
        let sprint = db.sprint_create(crate::models::sprint::CreateSprintInput {
            name: "S-dg".to_string(),
            goal: None,
            start_date: None,
            end_date: None,
        }).await.unwrap();
        let mission = db.mission_create(crate::models::mission::CreateMissionInput {
            title: "M-dg".to_string(),
            description: None,
            jira_key: None,
        }).await.unwrap();
        let epic = db.epic_create(crate::models::epic::CreateEpicInput {
            mission_id: Some(mission.id),
            sprint_id: Some(sprint.id),
            project_key: "dg".to_string(),
            title: "E-dg".to_string(),
            description: None,
        }).await.unwrap();
        db.issue_create(CreateIssueInput {
            epic_id: epic.id,
            title: "DG Test Issue".to_string(),
            description: None,
            goal: None,
            priority: None,
        }).await.unwrap()
    }

    /// Helper: required → ready → working → demo 단계별 전이 (user 권한)
    async fn advance_to_demo(db: &Db, id: i64) {
        db.issue_update(id, UpdateIssueInput { status: Some(IssueStatus::Ready), ..Default::default() }, "user").await.unwrap();
        db.issue_update(id, UpdateIssueInput { status: Some(IssueStatus::Working), ..Default::default() }, "user").await.unwrap();
        db.issue_update(id, UpdateIssueInput { status: Some(IssueStatus::Demo), ..Default::default() }, "user").await.unwrap();
    }

    /// [demo gate] agent 가 finished 로 직접 전이 시도 → 거부
    #[tokio::test]
    async fn test_demo_gate_agent_cannot_set_finished() {
        let db = setup_db().await;
        let issue = create_test_issue(&db).await;
        advance_to_demo(&db, issue.id).await;

        let result = db.issue_update(
            issue.id,
            UpdateIssueInput { status: Some(IssueStatus::Finished), ..Default::default() },
            "agent@some-bot",
        ).await;

        assert!(result.is_err(), "agent 의 finished 직접 전이는 거부되어야 한다");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("finished/cancelled") || err_msg.contains("user"),
            "에러 메시지에 demo gate 설명이 포함되어야 한다: {err_msg}"
        );

        // DB 에 실제로 반영되지 않았는지 확인
        let reloaded = db.issue_get(issue.id, false).await.unwrap();
        assert_eq!(reloaded.status, IssueStatus::Demo, "상태는 여전히 demo 이어야 한다");
    }

    /// [demo gate] agent 가 cancelled 로 직접 전이 시도 → 거부
    #[tokio::test]
    async fn test_demo_gate_agent_cannot_set_cancelled() {
        let db = setup_db().await;
        let issue = create_test_issue(&db).await;
        advance_to_demo(&db, issue.id).await;

        let result = db.issue_update(
            issue.id,
            UpdateIssueInput { status: Some(IssueStatus::Cancelled), ..Default::default() },
            "main@some-agent-id",
        ).await;

        assert!(result.is_err(), "agent 의 cancelled 직접 전이는 거부되어야 한다");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("finished/cancelled") || err_msg.contains("user"),
            "에러 메시지에 demo gate 설명이 포함되어야 한다: {err_msg}"
        );

        let reloaded = db.issue_get(issue.id, false).await.unwrap();
        assert_eq!(reloaded.status, IssueStatus::Demo, "상태는 여전히 demo 이어야 한다");
    }

    /// [demo gate] 정상 경로: user 가 demo → finished 전이 → 허용
    #[tokio::test]
    async fn test_demo_gate_user_can_finish() {
        let db = setup_db().await;
        let issue = create_test_issue(&db).await;
        advance_to_demo(&db, issue.id).await;

        let result = db.issue_update(
            issue.id,
            UpdateIssueInput { status: Some(IssueStatus::Finished), ..Default::default() },
            "user",
        ).await;

        assert!(result.is_ok(), "user 의 demo→finished 전이는 허용되어야 한다: {:?}", result.err());
        let updated = result.unwrap();
        assert_eq!(updated.status, IssueStatus::Finished);
    }
}
