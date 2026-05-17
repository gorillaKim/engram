use crate::models::history::{CreateHistoryInput, EntityType};
use crate::models::issue::*;
use crate::{Db, Error, Result};

impl Db {
    pub async fn issue_create(&self, input: CreateIssueInput) -> Result<Issue> {
        let priority = input.priority.unwrap_or(IssuePriority::Medium);
        let pval = serde_json::to_value(&priority).unwrap().as_str().unwrap().to_string();
        // RETURNING * 단일 쿼리 — sprint/epic_create 와 동일 패턴.
        sqlx::query_as::<_, Issue>(
            "INSERT INTO issues (epic_id, sprint_id, title, description, goal, priority) VALUES (?, ?, ?, ?, ?, ?)
             RETURNING id, epic_id, sprint_id, title, description, goal, status, priority, created_at, updated_at",
        )
        .bind(input.epic_id)
        .bind(input.sprint_id)
        .bind(&input.title)
        .bind(&input.description)
        .bind(&input.goal)
        .bind(&pval)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn issue_get(&self, id: i64) -> Result<Issue> {
        sqlx::query_as::<_, Issue>(
            "SELECT id, epic_id, sprint_id, title, description, goal, status, priority, created_at, updated_at FROM issues WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| Error::NotFound(format!("issue:{id}")))
    }

    pub async fn issue_list(&self, filter: IssueFilter) -> Result<Vec<Issue>> {
        let mut sql = "SELECT id, epic_id, sprint_id, title, description, goal, status, priority, created_at, updated_at FROM issues i WHERE 1=1".to_string();
        if filter.epic_id.is_some()     { sql.push_str(" AND i.epic_id = ?"); }
        if filter.sprint_id.is_some()   { sql.push_str(" AND i.sprint_id = ?"); }
        if filter.backlog_only           { sql.push_str(" AND i.sprint_id IS NULL"); }
        if filter.project_key.is_some() {
            sql.push_str(" AND EXISTS (SELECT 1 FROM epics e WHERE e.id = i.epic_id AND e.project_key = ?)");
        }
        if filter.status.is_some() { sql.push_str(" AND i.status = ?"); }
        sql.push_str(" ORDER BY i.id DESC");

        let mut q = sqlx::query_as::<_, Issue>(&sql);
        if let Some(e) = filter.epic_id     { q = q.bind(e); }
        if let Some(s) = filter.sprint_id   { q = q.bind(s); }
        if let Some(p) = filter.project_key { q = q.bind(p); }
        if let Some(s) = filter.status {
            let sv = serde_json::to_value(&s).unwrap().as_str().unwrap().to_string();
            q = q.bind(sv);
        }
        q.fetch_all(&self.pool).await.map_err(Into::into)
    }

    /// 이슈의 스프린트 소속을 변경한다. None 을 넘기면 백로그로 이동.
    pub async fn issue_set_sprint(&self, id: i64, sprint_id: Option<i64>, changed_by: &str) -> Result<Issue> {
        let current = self.issue_get(id).await?;
        sqlx::query("UPDATE issues SET sprint_id = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(sprint_id)
            .bind(id)
            .execute(&self.pool)
            .await?;
        let _ = self.history_record(CreateHistoryInput {
            entity_type: EntityType::Issue,
            entity_id: id,
            field: "sprint_id".to_string(),
            old_value: Some(current.sprint_id.map(|s| s.to_string()).unwrap_or_else(|| "null".to_string())),
            new_value: Some(sprint_id.map(|s| s.to_string()).unwrap_or_else(|| "null".to_string())),
            changed_by: changed_by.to_string(),
        }).await;
        self.issue_get(id).await
    }

    pub async fn issue_update(&self, id: i64, input: UpdateIssueInput, changed_by: &str) -> Result<Issue> {
        if let Some(ref new_status) = input.status {
            let current = self.issue_get(id).await?;
            if !current.status.can_transition_to(new_status) {
                return Err(crate::Error::InvalidTransition(
                    format!("{:?} → {:?}", current.status, new_status)
                ));
            }
            let old_v = serde_json::to_value(&current.status).unwrap().as_str().unwrap().to_string();
            let new_v = serde_json::to_value(new_status).unwrap().as_str().unwrap().to_string();
            sqlx::query("UPDATE issues SET status = ?, updated_at = datetime('now') WHERE id = ?")
                .bind(&new_v).bind(id).execute(&self.pool).await?;
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
        self.issue_get(id).await
    }

    pub async fn issue_link(&self, source_id: i64, target_id: i64, link_type: LinkType) -> Result<IssueLink> {
        let lt = serde_json::to_value(&link_type).unwrap().as_str().unwrap().to_string();
        // RETURNING * — WAL 가시성 회피.
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

    pub async fn issue_blocked_by(&self, issue_id: i64) -> Result<Vec<IssueLink>> {
        sqlx::query_as::<_, IssueLink>(
            "SELECT id, source_id, target_id, link_type, created_at FROM issue_links WHERE target_id = ? AND link_type = 'blocks'",
        )
        .bind(issue_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }

    /// 특정 상태에서 `threshold_minutes` 이상 머문 이슈 목록을 반환한다.
    ///
    /// 진입 시각은 `history` 의 `field='status' AND new_value=<status>` 중 가장 최근 레코드의
    /// `created_at` 으로 정의한다. history 가 없으면 `issues.updated_at` 으로 폴백한다.
    ///
    /// 리더 에이전트가 working 상태에서 정체된 이슈를 발견할 때 사용한다.
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
            .bind(&status_v) // h.new_value = ?
            .bind(&status_v); // i.status = ?
        if let Some(pk) = project_key {
            q = q.bind(pk);
        }
        q = q.bind(threshold_minutes);
        q.fetch_all(&self.pool).await.map_err(Into::into)
    }

    /// 이슈가 source 또는 target 인 모든 링크 반환 (이슈 상세 UI 용).
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
}
