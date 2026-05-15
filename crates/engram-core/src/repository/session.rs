use serde::{Deserialize, Serialize};
use crate::models::{Epic, Issue, NoteSummary, Task};
use crate::{Db, Result};

/// session_restore 전체 응답 구조
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub sprint_id: i64,
    pub sprint_name: String,
    pub sprint_goal: Option<String>,
    pub project_key: Option<String>,
    pub active_epics: Vec<EpicSnapshot>,
    pub next_action: Option<crate::models::task::NextTask>,
    pub pending_drafts: Vec<IssueBrief>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpicSnapshot {
    pub epic: Epic,
    pub active_issues: Vec<IssueSnapshot>,
    pub progress: EpicProgress,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueSnapshot {
    pub issue: Issue,
    pub active_notes: Vec<NoteSummary>,
    pub current_task: Option<Task>,
    pub blocked_by: Vec<i64>, // blocker issue ids
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpicProgress {
    pub done: u32,
    pub in_progress: u32,
    pub todo: u32,
    pub total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueBrief {
    pub id: i64,
    pub title: String,
    pub epic_id: i64,
    pub created_at: String,
}

/// session_end 체크리스트 응답
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEndResult {
    pub warnings: Vec<String>,
    pub in_progress_tasks: Vec<TaskBrief>,
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskBrief {
    pub id: i64,
    pub title: String,
    pub issue_id: i64,
    pub issue_title: String,
}

impl Db {
    /// 세션 복원 — 현재 active sprint + project_key 기준 에픽/이슈 조회
    pub async fn session_restore(
        &self,
        project_key: Option<&str>,
    ) -> Result<SessionSnapshot> {
        let sprint = self.sprint_current().await?;
        let Some(sprint) = sprint else {
            return Ok(SessionSnapshot {
                sprint_id: 0,
                sprint_name: "활성 스프린트 없음".to_string(),
                sprint_goal: None,
                project_key: project_key.map(String::from),
                active_epics: vec![],
                next_action: None,
                pending_drafts: vec![],
                warnings: vec!["활성 스프린트가 없습니다. sprint_create로 시작하세요.".to_string()],
            });
        };

        // 에픽 목록 조회 (project_key 필터)
        let epics = self.epic_list(Some(sprint.id), project_key, None).await?;

        let mut active_epics = Vec::new();
        let mut pending_drafts = Vec::new();

        for epic in epics {
            let issues = self.issue_list(crate::models::issue::IssueFilter {
                epic_id: Some(epic.id),
                ..Default::default()
            }).await?;

            let mut active_issues = Vec::new();
            let (mut done, mut in_prog, mut todo_cnt, total) =
                (0u32, 0u32, 0u32, issues.len() as u32);

            for issue in &issues {
                use crate::models::issue::IssueStatus;
                match &issue.status {
                    IssueStatus::Finished => done += 1,
                    IssueStatus::Working | IssueStatus::Demo => in_prog += 1,
                    IssueStatus::Ready => todo_cnt += 1,
                    IssueStatus::Required => {
                        pending_drafts.push(IssueBrief {
                            id: issue.id,
                            title: issue.title.clone(),
                            epic_id: epic.id,
                            created_at: issue.created_at.clone(),
                        });
                        continue;
                    }
                    IssueStatus::Cancelled => continue,
                }

                // 활성 이슈의 note summaries + current task 조회
                let active_notes = self.note_summaries(issue.id, false).await?;
                let tasks = self.task_list(issue.id, None).await?;
                let current_task = tasks.into_iter()
                    .find(|t| t.status == crate::models::task::TaskStatus::Ready);
                let blocked_by = self.issue_blocked_by(issue.id).await?
                    .into_iter().map(|l| l.source_id).collect();

                active_issues.push(IssueSnapshot {
                    issue: issue.clone(),
                    active_notes,
                    current_task,
                    blocked_by,
                });
            }

            active_epics.push(EpicSnapshot {
                epic,
                active_issues,
                progress: EpicProgress { done, in_progress: in_prog, todo: todo_cnt, total },
            });
        }

        let next_action = self.task_next(project_key, None).await?;

        let mut warnings = Vec::new();
        if !pending_drafts.is_empty() {
            warnings.push(format!(
                "미승인 draft 이슈 {}건이 있습니다. 승인 또는 취소가 필요합니다.",
                pending_drafts.len()
            ));
        }

        Ok(SessionSnapshot {
            sprint_id: sprint.id,
            sprint_name: sprint.name,
            sprint_goal: sprint.goal,
            project_key: project_key.map(String::from),
            active_epics,
            next_action,
            pending_drafts,
            warnings,
        })
    }

    /// 세션 종료 체크리스트 — context note 누락 경고
    pub async fn session_end(
        &self,
        project_key: Option<&str>,
    ) -> Result<SessionEndResult> {
        let mut warnings = Vec::new();
        let mut in_progress_tasks = Vec::new();

        // in_progress 태스크 조회 (동적 project_key 필터)
        #[derive(sqlx::FromRow)]
        struct TaskRow {
            id: i64,
            task_title: String,
            issue_id: i64,
            issue_title: String,
        }

        let mut sql = r#"SELECT t.id, t.title as task_title,
            i.id as issue_id, i.title as issue_title
            FROM tasks t
            JOIN issues i ON t.issue_id = i.id
            JOIN epics e ON i.epic_id = e.id
            WHERE t.status = 'working'"#.to_string();

        if project_key.is_some() {
            sql.push_str(" AND e.project_key = ?");
        }

        let mut q = sqlx::query_as::<_, TaskRow>(&sql);
        if let Some(pk) = project_key { q = q.bind(pk); }
        let rows = q.fetch_all(&self.pool).await?;

        for r in rows {
            // context note 있는지 확인
            let has_context: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM notes WHERE issue_id = ? AND note_type = 'context' AND resolved = 0",
            )
            .bind(r.issue_id)
            .fetch_one(&self.pool)
            .await?;

            if has_context == 0 {
                warnings.push(format!(
                    "이슈 '{}': context note가 없습니다. 다음 세션 인수인계를 위해 기록을 남겨주세요.",
                    r.issue_title
                ));
            }

            in_progress_tasks.push(TaskBrief {
                id: r.id,
                title: r.task_title,
                issue_id: r.issue_id,
                issue_title: r.issue_title,
            });
        }

        let ok = warnings.is_empty();
        Ok(SessionEndResult { warnings, in_progress_tasks, ok })
    }
}
