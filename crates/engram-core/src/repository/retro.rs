use serde::{Deserialize, Serialize};
use crate::{Db, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetroReport {
    pub sprint_id: i64,
    pub sprint_name: String,
    pub issue_timelines: Vec<IssueTimeline>,
    pub scope_expansions: Vec<ScopeExpansion>,
    pub total_issues: u32,
    pub finished_issues: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueTimeline {
    pub issue_id: i64,
    pub title: String,
    pub transitions: Vec<StatusTransition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusTransition {
    pub field: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub changed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeExpansion {
    pub issue_id: i64,
    pub title: String,
    pub planned: u32,
    pub discovered: u32,
    pub expansion_rate: u32,
}

impl Db {
    pub async fn retro_report(&self, sprint_id: i64) -> Result<RetroReport> {
        let sprint = self.sprint_get(sprint_id).await?;

        let issues = sqlx::query_as::<_, crate::models::issue::Issue>(
            r#"SELECT i.id, i.epic_id, i.sprint_id, i.title, i.description, i.goal, i.status, i.priority, i.assigned_agent, i.created_at, i.updated_at
               FROM issues i
               WHERE i.sprint_id = ?
               ORDER BY i.id ASC"#,
        )
        .bind(sprint_id)
        .fetch_all(&self.pool)
        .await?;

        let total_issues = issues.len() as u32;
        let finished_issues = issues.iter()
            .filter(|i| i.status == crate::models::issue::IssueStatus::Finished)
            .count() as u32;

        let mut issue_timelines = Vec::new();
        let mut scope_expansions = Vec::new();

        for issue in &issues {
            let history = self.history_list(
                crate::models::history::EntityType::Issue,
                issue.id,
            ).await?;

            let transitions = history.into_iter().map(|h| StatusTransition {
                field: h.field,
                old_value: h.old_value,
                new_value: h.new_value,
                changed_at: h.created_at,
            }).collect::<Vec<_>>();

            if !transitions.is_empty() {
                issue_timelines.push(IssueTimeline {
                    issue_id: issue.id,
                    title: issue.title.clone(),
                    transitions,
                });
            }

            let planned: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM tasks WHERE issue_id = ? AND source = 'planned'",
            )
            .bind(issue.id)
            .fetch_one(&self.pool)
            .await?;

            let discovered: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM tasks WHERE issue_id = ? AND source = 'agent_discovered'",
            )
            .bind(issue.id)
            .fetch_one(&self.pool)
            .await?;

            let total_tasks = planned + discovered;
            if total_tasks > 0 {
                let rate = (discovered * 100 / total_tasks) as u32;
                if rate > 0 {
                    scope_expansions.push(ScopeExpansion {
                        issue_id: issue.id,
                        title: issue.title.clone(),
                        planned: planned as u32,
                        discovered: discovered as u32,
                        expansion_rate: rate,
                    });
                }
            }
        }

        Ok(RetroReport {
            sprint_id: sprint.id,
            sprint_name: sprint.name,
            issue_timelines,
            scope_expansions,
            total_issues,
            finished_issues,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        sprint::{CreateSprintInput, UpdateSprintInput, SprintStatus},
        epic::CreateEpicInput,
        issue::{CreateIssueInput, UpdateIssueInput, IssueStatus},
        task::{CreateTaskInput, TaskSource},
    };

    async fn setup() -> Db {
        Db::open_in_memory().await.unwrap()
    }

    #[tokio::test]
    async fn test_retro_report_basic() {
        let db = setup().await;

        let sprint = db.sprint_create(CreateSprintInput {
            name: "Sprint 1".to_string(), goal: Some("목표".to_string()),
            start_date: None, end_date: None,
        }).await.unwrap();
        db.sprint_update(sprint.id, UpdateSprintInput {
            status: Some(SprintStatus::Active), ..Default::default()
        }, "agent").await.unwrap();

        let epic = db.epic_create(CreateEpicInput {
            project_key: "proj".to_string(),
            title: "Epic".to_string(), description: None,
        }).await.unwrap();

        let issue = db.issue_create(CreateIssueInput {
            epic_id: epic.id, sprint_id: Some(sprint.id), title: "Issue 1".to_string(),
            description: None, goal: None, priority: None,
        }).await.unwrap();

        // 상태 전이 발생 (history 기록됨)
        db.issue_update(issue.id, UpdateIssueInput {
            status: Some(IssueStatus::Ready), ..Default::default()
        }, "agent").await.unwrap();

        // scope expansion: 1 planned + 2 discovered
        db.task_create(CreateTaskInput {
            issue_id: issue.id, title: "T1".to_string(),
            description: None, goal: None, after_task_id: None,
            source: Some(TaskSource::Planned),
        }).await.unwrap();
        for i in 0..2 {
            db.task_create(CreateTaskInput {
                issue_id: issue.id, title: format!("D{i}"),
                description: None, goal: None, after_task_id: None,
                source: Some(TaskSource::AgentDiscovered),
            }).await.unwrap();
        }

        let report = db.retro_report(sprint.id).await.unwrap();

        assert_eq!(report.sprint_id, sprint.id);
        assert!(!report.issue_timelines.is_empty(), "타임라인이 있어야 함");
        assert!(!report.scope_expansions.is_empty(), "scope expansion이 있어야 함");

        let expansion = &report.scope_expansions[0];
        assert_eq!(expansion.planned, 1);
        assert_eq!(expansion.discovered, 2);
        assert!(expansion.expansion_rate > 50, "expansion_rate > 50 이어야 함");
    }
}
