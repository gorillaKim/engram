use crate::models::issue::*;
use crate::{Db, Error, Result};

impl Db {
    pub async fn issue_create(&self, input: CreateIssueInput) -> Result<Issue> {
        let priority = input.priority.unwrap_or(IssuePriority::Medium);
        let pval = serde_json::to_value(&priority).unwrap().as_str().unwrap().to_string();
        let id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO issues (epic_id, title, description, goal, priority) VALUES (?, ?, ?, ?, ?) RETURNING id",
        )
        .bind(input.epic_id)
        .bind(&input.title)
        .bind(&input.description)
        .bind(&input.goal)
        .bind(&pval)
        .fetch_one(&self.pool)
        .await?;
        self.issue_get(id).await
    }

    pub async fn issue_get(&self, id: i64) -> Result<Issue> {
        sqlx::query_as::<_, Issue>(
            "SELECT id, epic_id, title, description, goal, status, priority, created_at, updated_at FROM issues WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| Error::NotFound(format!("issue:{id}")))
    }

    pub async fn issue_list(&self, filter: IssueFilter) -> Result<Vec<Issue>> {
        let mut sql = "SELECT id, epic_id, title, description, goal, status, priority, created_at, updated_at FROM issues i WHERE 1=1".to_string();
        if filter.epic_id.is_some()     { sql.push_str(" AND i.epic_id = ?"); }
        if filter.project_key.is_some() {
            sql.push_str(" AND EXISTS (SELECT 1 FROM epics e WHERE e.id = i.epic_id AND e.project_key = ?)");
        }
        if filter.status.is_some() { sql.push_str(" AND i.status = ?"); }
        sql.push_str(" ORDER BY i.id DESC");

        let mut q = sqlx::query_as::<_, Issue>(&sql);
        if let Some(e) = filter.epic_id     { q = q.bind(e); }
        if let Some(p) = filter.project_key { q = q.bind(p); }
        if let Some(s) = filter.status {
            let sv = serde_json::to_value(&s).unwrap().as_str().unwrap().to_string();
            q = q.bind(sv);
        }
        q.fetch_all(&self.pool).await.map_err(Into::into)
    }

    pub async fn issue_update(&self, id: i64, input: UpdateIssueInput) -> Result<Issue> {
        if let Some(ref new_status) = input.status {
            let current = self.issue_get(id).await?;
            if !current.status.can_transition_to(new_status) {
                return Err(crate::Error::InvalidTransition(
                    format!("{:?} → {:?}", current.status, new_status)
                ));
            }
            let sv = serde_json::to_value(new_status).unwrap().as_str().unwrap().to_string();
            sqlx::query("UPDATE issues SET status = ?, updated_at = datetime('now') WHERE id = ?")
                .bind(sv).bind(id).execute(&self.pool).await?;
        }
        if let Some(ref p) = input.priority {
            let pv = serde_json::to_value(p).unwrap().as_str().unwrap().to_string();
            sqlx::query("UPDATE issues SET priority = ?, updated_at = datetime('now') WHERE id = ?")
                .bind(pv).bind(id).execute(&self.pool).await?;
        }
        if let Some(ref title) = input.title {
            sqlx::query("UPDATE issues SET title = ?, updated_at = datetime('now') WHERE id = ?")
                .bind(title).bind(id).execute(&self.pool).await?;
        }
        if let Some(ref desc) = input.description {
            sqlx::query("UPDATE issues SET description = ?, updated_at = datetime('now') WHERE id = ?")
                .bind(desc).bind(id).execute(&self.pool).await?;
        }
        if let Some(ref goal) = input.goal {
            sqlx::query("UPDATE issues SET goal = ?, updated_at = datetime('now') WHERE id = ?")
                .bind(goal).bind(id).execute(&self.pool).await?;
        }
        self.issue_get(id).await
    }

    pub async fn issue_link(&self, source_id: i64, target_id: i64, link_type: LinkType) -> Result<IssueLink> {
        let lt = serde_json::to_value(&link_type).unwrap().as_str().unwrap().to_string();
        let id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO issue_links (source_id, target_id, link_type) VALUES (?, ?, ?) RETURNING id",
        )
        .bind(source_id).bind(target_id).bind(&lt)
        .fetch_one(&self.pool)
        .await?;

        sqlx::query_as::<_, IssueLink>(
            "SELECT id, source_id, target_id, link_type, created_at FROM issue_links WHERE id = ?",
        )
        .bind(id)
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
}
