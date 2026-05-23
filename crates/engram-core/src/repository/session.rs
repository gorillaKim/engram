use serde::{Deserialize, Serialize};
use crate::models::{Epic, Issue, Note, NoteSummary, Task, MissionSummary};
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
    /// Issue IDs where agent_discovered tasks > 50% (structured form of scope-expansion warnings)
    pub scope_expansion_ids: Vec<i64>,
    /// 현재 working 상태에서 점유 중인 에이전트 목록. 리더 에이전트가 spawn 결정 시 참조.
    #[serde(default)]
    pub active_workers: Vec<ActiveWorker>,
    /// Broadcast scope 의 unresolved caveat/decision 노트 (project / 활성 sprint / 활성 epic).
    /// 어느 이슈로 session_restore 를 호출해도 같은 광역 공지가 모든 에이전트에 전파된다.
    #[serde(default)]
    pub active_caveats: Vec<Note>,
    /// 현재 active 상태인 미션 목록 (완료/취소 미션 제외).
    /// project_key 지정 시 해당 프로젝트의 에픽을 가진 미션만 포함.
    #[serde(default)]
    pub active_missions: Vec<MissionSummary>,
}

/// `assigned_agent` 가 NULL 이 아닌 working 이슈의 점유 정보.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ActiveWorker {
    pub issue_id: i64,
    pub issue_title: String,
    pub agent_id: String,
    pub project_key: String,
    /// working 상태 진입 시각 (issues.updated_at). lease 만료 추적용.
    pub since: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpicSnapshot {
    pub epic: Epic,
    pub active_issues: Vec<IssueSnapshot>,
    pub progress: EpicProgress,
    /// compact=true 시 active_issues 대신 채워진다. compact=false 시 None.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_issues_compact: Option<Vec<IssueSnapshotCompact>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueSnapshot {
    pub issue: Issue,
    pub active_notes: Vec<NoteSummary>,
    pub current_task: Option<Task>,
    pub blocked_by: Vec<i64>, // blocker issue ids
}

/// compact 모드 이슈 요약 — notes/tasks 를 count 로만 반환해 페이로드를 줄인다.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueSnapshotCompact {
    pub issue: Issue,
    pub task_count: i64,
    pub note_count: i64,
    pub blocked_by_ids: Vec<i64>,
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

/// board_status 응답 구조
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardStatus {
    pub sprint_id: i64,
    pub sprint_name: String,
    pub project_key: Option<String>,
    pub projects: Vec<ProjectBoard>,
    pub blocked_chains: Vec<BlockedChain>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectBoard {
    pub project_key: String,
    pub required: u32,
    pub ready: u32,
    pub working: u32,
    pub demo: u32,
    pub finished: u32,
    pub cancelled: u32,
    pub total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedChain {
    pub blocker_id: i64,
    pub blocker_title: String,
    pub blocked_id: i64,
    pub blocked_title: String,
}

/// Kanban UI 용 — 상태별 Issue 배열 (board_status_query 의 카운트와 별도)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueBoardStatus {
    pub sprint_id: i64,
    pub sprint_name: String,
    pub project_key: Option<String>,
    pub boards: Vec<IssueProjectBoard>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueProjectBoard {
    pub project_key: String,
    pub required: Vec<crate::models::Issue>,
    pub ready: Vec<crate::models::Issue>,
    pub working: Vec<crate::models::Issue>,
    pub demo: Vec<crate::models::Issue>,
    pub finished: Vec<crate::models::Issue>,
    /// 취소된 이슈 — UI 에서 토글 시 표시한다. board 에 포함시키되 기본은 숨김.
    #[serde(default)]
    pub cancelled: Vec<crate::models::Issue>,
}

impl Db {
    /// 세션 복원 — 현재 active sprint + project_key 기준 에픽/이슈 조회.
    /// `compact=true` 이면 per-issue notes/tasks fetch 를 COUNT 쿼리로 대체해 페이로드를 70%+ 줄인다.
    pub async fn session_restore(
        &self,
        project_key: Option<&str>,
        compact: bool,
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
                active_workers: vec![],
                active_caveats: vec![],
                active_missions: vec![],
                warnings: vec!["활성 스프린트가 없습니다. sprint_create로 시작하세요.".to_string()],
                scope_expansion_ids: vec![],
            });
        };

        // 현재 스프린트에 속한 이슈를 조회한 뒤 epic_id 로 그룹핑.
        // (에픽은 sprint-agnostic 카테고리이므로 sprint 로 직접 거를 수 없다 — 이슈 기준.)
        let sprint_issues = self.issue_list(crate::models::issue::IssueFilter {
            sprint_id: Some(sprint.id),
            project_key: project_key.map(String::from),
            ..Default::default()
        }).await?;

        // 이슈를 epic_id 별로 묶는다 (삽입 순서 유지를 위해 IndexMap 대신 Vec<(epic_id, ...)>).
        let mut grouped: Vec<(i64, Vec<crate::models::issue::Issue>)> = Vec::new();
        for issue in sprint_issues {
            if let Some((_, v)) = grouped.iter_mut().find(|(eid, _)| *eid == issue.epic_id) {
                v.push(issue);
            } else {
                grouped.push((issue.epic_id, vec![issue]));
            }
        }

        let mut active_epics = Vec::new();
        let mut pending_drafts = Vec::new();

        for (epic_id, issues) in grouped {
            let epic = self.epic_get(epic_id).await?;

            let (mut done, mut in_prog, mut todo_cnt, total) =
                (0u32, 0u32, 0u32, issues.len() as u32);

            // 먼저 pending_drafts / 카운터를 채우기 위해 이슈 상태를 분류한다.
            // compact 모드와 full 모드 모두 이 분류는 동일하게 수행.
            let mut active_issue_list: Vec<&crate::models::issue::Issue> = Vec::new();
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
                active_issue_list.push(issue);
            }

            let (full_issues, compact_issues) = if compact {
                // compact 모드: COUNT 쿼리로 N+1 제거
                let compact_snaps = if active_issue_list.is_empty() {
                    Vec::new()
                } else {
                    let issue_ids: Vec<i64> = active_issue_list.iter().map(|i| i.id).collect();

                    // bulk task count
                    let placeholders = issue_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                    let task_sql = format!(
                        "SELECT issue_id, COUNT(*) as cnt FROM tasks WHERE issue_id IN ({}) GROUP BY issue_id",
                        placeholders
                    );
                    #[derive(sqlx::FromRow)]
                    struct CountRow { issue_id: i64, cnt: i64 }
                    let mut tq = sqlx::query_as::<_, CountRow>(&task_sql);
                    for id in &issue_ids { tq = tq.bind(id); }
                    let task_counts: std::collections::HashMap<i64, i64> = tq
                        .fetch_all(&self.pool).await.unwrap_or_default()
                        .into_iter().map(|r| (r.issue_id, r.cnt)).collect();

                    // bulk note count
                    let note_sql = format!(
                        "SELECT issue_id, COUNT(*) as cnt FROM notes WHERE issue_id IN ({}) AND resolved = 0 GROUP BY issue_id",
                        placeholders
                    );
                    let mut nq = sqlx::query_as::<_, CountRow>(&note_sql);
                    for id in &issue_ids { nq = nq.bind(id); }
                    let note_counts: std::collections::HashMap<i64, i64> = nq
                        .fetch_all(&self.pool).await.unwrap_or_default()
                        .into_iter().map(|r| (r.issue_id, r.cnt)).collect();

                    // bulk blocked_by: issue_links WHERE target_id IN (...)
                    let links_sql = format!(
                        "SELECT target_id, source_id FROM issue_links WHERE link_type = 'blocks' AND target_id IN ({})",
                        placeholders
                    );
                    #[derive(sqlx::FromRow)]
                    struct LinkRow { target_id: i64, source_id: i64 }
                    let mut lq = sqlx::query_as::<_, LinkRow>(&links_sql);
                    for id in &issue_ids { lq = lq.bind(id); }
                    let link_rows = lq.fetch_all(&self.pool).await.unwrap_or_default();
                    let mut blocked_by_map: std::collections::HashMap<i64, Vec<i64>> = std::collections::HashMap::new();
                    for lr in link_rows {
                        blocked_by_map.entry(lr.target_id).or_default().push(lr.source_id);
                    }

                    active_issue_list.iter().map(|issue| IssueSnapshotCompact {
                        issue: (*issue).clone(),
                        task_count: *task_counts.get(&issue.id).unwrap_or(&0),
                        note_count: *note_counts.get(&issue.id).unwrap_or(&0),
                        blocked_by_ids: blocked_by_map.remove(&issue.id).unwrap_or_default(),
                    }).collect()
                };
                (Vec::new(), Some(compact_snaps))
            } else {
                // full 모드: 기존 N+1 per-issue fetch
                let mut full_snaps = Vec::new();
                for issue in active_issue_list {
                    let active_notes = self.note_summaries(issue.id, false).await?;
                    let tasks = self.task_list(issue.id, None).await?;
                    let current_task = tasks.into_iter()
                        .find(|t| t.status == crate::models::task::TaskStatus::Ready);
                    let blocked_by = self.issue_blocked_by(issue.id).await?
                        .into_iter().map(|l| l.source_id).collect();
                    full_snaps.push(IssueSnapshot {
                        issue: issue.clone(),
                        active_notes,
                        current_task,
                        blocked_by,
                    });
                }
                (full_snaps, None)
            };

            active_epics.push(EpicSnapshot {
                epic,
                active_issues: full_issues,
                active_issues_compact: compact_issues,
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

        // 스코프 팽창 감지: agent_discovered 태스크 비율 > 50% 이슈
        // compact / full 모드 모두 동일하게 처리: issue id + title 만 필요
        for epic_snap in &active_epics {
            // full 모드: active_issues 순회
            let full_iter = epic_snap.active_issues.iter()
                .map(|s| (s.issue.id, s.issue.title.as_str()));
            // compact 모드: active_issues_compact 순회
            let compact_iter = epic_snap.active_issues_compact.as_deref().unwrap_or(&[]).iter()
                .map(|s| (s.issue.id, s.issue.title.as_str()));

            for (issue_id, issue_title) in full_iter.chain(compact_iter) {
                let total: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM tasks WHERE issue_id = ?",
                )
                .bind(issue_id)
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);

                if total == 0 { continue; }

                let discovered: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM tasks WHERE issue_id = ? AND source = 'agent_discovered'",
                )
                .bind(issue_id)
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);

                let rate = discovered * 100 / total;
                if rate > 50 {
                    warnings.push(format!(
                        "스코프 팽창 감지: 이슈 #{} '{}' 태스크의 {}%가 agent_discovered ({}/{}건)",
                        issue_id, issue_title, rate, discovered, total
                    ));
                }
            }
        }

        // Collect scope expansion IDs alongside warning strings
        let scope_expansion_ids: Vec<i64> = warnings.iter()
            .filter(|w| w.contains("스코프 팽창 감지"))
            .filter_map(|w| {
                w.split_once("이슈 #").and_then(|(_, rest)| {
                    rest.split_once(|c: char| !c.is_ascii_digit())
                        .and_then(|(id_str, _)| id_str.parse::<i64>().ok())
                        .or_else(|| rest.parse::<i64>().ok())
                })
            })
            .collect();

        // 현재 working 상태에서 점유 중인 에이전트 조회
        let mut workers_sql = String::from(
            "SELECT i.id AS issue_id, i.title AS issue_title, \
                    i.assigned_agent AS agent_id, e.project_key AS project_key, \
                    i.updated_at AS since \
             FROM issues i \
             JOIN epics e ON e.id = i.epic_id \
             WHERE i.status='working' AND i.assigned_agent IS NOT NULL \
               AND i.sprint_id = ?",
        );
        if project_key.is_some() {
            workers_sql.push_str(" AND e.project_key = ?");
        }
        workers_sql.push_str(" ORDER BY i.updated_at ASC");

        let mut wq = sqlx::query_as::<_, ActiveWorker>(&workers_sql).bind(sprint.id);
        if let Some(pk) = project_key { wq = wq.bind(pk); }
        let active_workers = wq.fetch_all(&self.pool).await.unwrap_or_default();

        // Broadcast scope caveat 조회 — project / 활성 sprint / 활성 epic 의 unresolved 노트.
        // 어떤 이슈로 session_restore 를 호출해도 같은 광역 공지가 노출된다.
        let active_epic_ids: Vec<i64> = active_epics.iter().map(|e| e.epic.id).collect();
        let active_caveats: Vec<Note> = {
            let placeholders = if active_epic_ids.is_empty() {
                "NULL".to_string()
            } else {
                active_epic_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",")
            };
            let mut sql = String::from(
                "SELECT id, issue_id, task_id, note_type, summary, detail, author, agent_id, resolved, scope, scope_target_id, project_key, created_at, resolved_at \
                 FROM notes WHERE resolved = 0 AND ("
            );
            sql.push_str("(scope = 'sprint' AND scope_target_id = ?)");
            sql.push_str(&format!(" OR (scope = 'epic' AND scope_target_id IN ({}))", placeholders));
            if project_key.is_some() {
                sql.push_str(" OR (scope = 'project' AND project_key = ?)");
            }
            sql.push_str(") ORDER BY created_at DESC");

            let mut cq = sqlx::query_as::<_, Note>(&sql).bind(sprint.id);
            for eid in &active_epic_ids { cq = cq.bind(eid); }
            if let Some(pk) = project_key { cq = cq.bind(pk); }
            cq.fetch_all(&self.pool).await.unwrap_or_default()
        };

        // active 미션 목록 조회 — project_key 있으면 해당 프로젝트 에픽을 가진 미션만 포함
        let active_missions: Vec<MissionSummary> = {
            #[derive(sqlx::FromRow)]
            struct MissionSummaryRaw {
                id: i64,
                title: String,
                status: crate::models::MissionStatus,
                progress_rate: Option<f64>,
                epic_count: i64,
            }

            let mut msql = String::from(
                "SELECT \
                    m.id, m.title, m.status, \
                    COUNT(DISTINCT e.id) as epic_count, \
                    CAST( \
                        COUNT(CASE WHEN i.status = 'finished' THEN 1 END) AS REAL \
                    ) / NULLIF(COUNT(i.id), 0) as progress_rate \
                 FROM missions m \
                 LEFT JOIN epics e ON e.mission_id = m.id \
                 LEFT JOIN issues i ON i.mission_id = m.id \
                 WHERE m.status = 'active'",
            );
            if project_key.is_some() {
                msql.push_str(" AND EXISTS ( \
                    SELECT 1 FROM epics ep \
                    WHERE ep.mission_id = m.id AND ep.project_key = ? \
                )");
            }
            msql.push_str(" GROUP BY m.id ORDER BY m.id");

            let mut mq = sqlx::query_as::<_, MissionSummaryRaw>(&msql);
            if let Some(pk) = project_key { mq = mq.bind(pk); }
            let rows = mq.fetch_all(&self.pool).await.unwrap_or_default();

            rows.into_iter().map(|r| MissionSummary {
                id: r.id,
                title: r.title,
                status: r.status,
                progress_rate: r.progress_rate.unwrap_or(0.0),
                epic_count: r.epic_count,
            }).collect()
        };

        Ok(SessionSnapshot {
            sprint_id: sprint.id,
            sprint_name: sprint.name,
            sprint_goal: sprint.goal,
            project_key: project_key.map(String::from),
            active_epics,
            next_action,
            pending_drafts,
            warnings,
            scope_expansion_ids,
            active_workers,
            active_caveats,
            active_missions,
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

    /// 보드 전체 현황 — 프로젝트별 이슈 상태 집계 + 블로킹 체인
    pub async fn board_status_query(&self, project_key: Option<&str>) -> Result<BoardStatus> {
        let sprint = self.sprint_current().await?;
        let Some(sprint) = sprint else {
            return Ok(BoardStatus {
                sprint_id: 0,
                sprint_name: "활성 스프린트 없음".to_string(),
                project_key: project_key.map(String::from),
                projects: vec![],
                blocked_chains: vec![],
            });
        };

        // 프로젝트별 이슈 상태 집계
        let mut sql = r#"
            SELECT e.project_key,
                SUM(CASE WHEN i.status = 'required' THEN 1 ELSE 0 END) as required,
                SUM(CASE WHEN i.status = 'ready' THEN 1 ELSE 0 END) as ready,
                SUM(CASE WHEN i.status = 'working' THEN 1 ELSE 0 END) as working,
                SUM(CASE WHEN i.status = 'demo' THEN 1 ELSE 0 END) as demo,
                SUM(CASE WHEN i.status = 'finished' THEN 1 ELSE 0 END) as finished,
                SUM(CASE WHEN i.status = 'cancelled' THEN 1 ELSE 0 END) as cancelled,
                COUNT(*) as total
            FROM issues i
            JOIN epics e ON i.epic_id = e.id
            WHERE i.sprint_id = ?
        "#.to_string();
        if project_key.is_some() {
            sql.push_str(" AND e.project_key = ?");
        }
        sql.push_str(" GROUP BY e.project_key ORDER BY e.project_key");

        #[derive(sqlx::FromRow)]
        struct ProjRow {
            project_key: String,
            required: i64,
            ready: i64,
            working: i64,
            demo: i64,
            finished: i64,
            cancelled: i64,
            total: i64,
        }

        let mut q = sqlx::query_as::<_, ProjRow>(&sql).bind(sprint.id);
        if let Some(pk) = project_key {
            q = q.bind(pk);
        }
        let proj_rows = q.fetch_all(&self.pool).await?;

        let projects = proj_rows.into_iter().map(|r| ProjectBoard {
            project_key: r.project_key,
            required: r.required as u32,
            ready: r.ready as u32,
            working: r.working as u32,
            demo: r.demo as u32,
            finished: r.finished as u32,
            cancelled: r.cancelled as u32,
            total: r.total as u32,
        }).collect();

        // 블로킹 체인 조회 — 미완료 blocker 만 포함
        let mut bsql = r#"
            SELECT il.source_id as blocker_id, bi.title as blocker_title,
                   il.target_id as blocked_id, ti.title as blocked_title
            FROM issue_links il
            JOIN issues bi ON il.source_id = bi.id
            JOIN issues ti ON il.target_id = ti.id
            JOIN epics be ON bi.epic_id = be.id
            JOIN epics te ON ti.epic_id = te.id
            WHERE il.link_type = 'blocks'
              AND bi.sprint_id = ?
              AND bi.status NOT IN ('finished', 'cancelled')
        "#.to_string();
        if project_key.is_some() {
            bsql.push_str(" AND be.project_key = ?");
        }

        #[derive(sqlx::FromRow)]
        struct ChainRow {
            blocker_id: i64,
            blocker_title: String,
            blocked_id: i64,
            blocked_title: String,
        }

        let mut bq = sqlx::query_as::<_, ChainRow>(&bsql).bind(sprint.id);
        if let Some(pk) = project_key {
            bq = bq.bind(pk);
        }
        let chain_rows = bq.fetch_all(&self.pool).await?;

        let blocked_chains = chain_rows.into_iter().map(|r| BlockedChain {
            blocker_id: r.blocker_id,
            blocker_title: r.blocker_title,
            blocked_id: r.blocked_id,
            blocked_title: r.blocked_title,
        }).collect();

        Ok(BoardStatus {
            sprint_id: sprint.id,
            sprint_name: sprint.name,
            project_key: project_key.map(String::from),
            projects,
            blocked_chains,
        })
    }

    /// Kanban UI 용 — 상태별 Issue 배열을 프로젝트별로 반환
    pub async fn board_issues_query(
        &self,
        project_key: Option<&str>,
    ) -> Result<IssueBoardStatus> {
        let sprint = self.sprint_current().await?;
        let Some(sprint) = sprint else {
            return Ok(IssueBoardStatus {
                sprint_id: 0,
                sprint_name: "활성 스프린트 없음".to_string(),
                project_key: project_key.map(String::from),
                boards: vec![],
            });
        };

        let mut sql = r#"
            SELECT i.id, i.epic_id, i.mission_id, i.sprint_id, i.title, i.description, i.goal,
                   i.status, i.priority, i.assigned_agent, i.created_at, i.updated_at,
                   e.project_key as proj
            FROM issues i
            JOIN epics e ON i.epic_id = e.id
            WHERE i.sprint_id = ?
        "#.to_string();
        if project_key.is_some() {
            sql.push_str(" AND e.project_key = ?");
        }
        sql.push_str(" ORDER BY e.project_key, i.priority, i.id");

        #[derive(sqlx::FromRow)]
        struct IssueRow {
            id: i64,
            epic_id: i64,
            mission_id: Option<i64>,
            sprint_id: Option<i64>,
            title: String,
            description: Option<String>,
            goal: Option<String>,
            status: crate::models::issue::IssueStatus,
            priority: crate::models::issue::IssuePriority,
            assigned_agent: Option<String>,
            created_at: String,
            updated_at: String,
            proj: String,
        }

        let mut q = sqlx::query_as::<_, IssueRow>(&sql).bind(sprint.id);
        if let Some(pk) = project_key {
            q = q.bind(pk);
        }
        let rows = q.fetch_all(&self.pool).await?;

        // Group by project_key — insertion-order Vec
        let mut boards: Vec<IssueProjectBoard> = vec![];
        for r in rows {
            let board = match boards.iter_mut().find(|b| b.project_key == r.proj) {
                Some(b) => b,
                None => {
                    boards.push(IssueProjectBoard {
                        project_key: r.proj.clone(),
                        required: vec![],
                        ready: vec![],
                        working: vec![],
                        demo: vec![],
                        finished: vec![],
                        cancelled: vec![],
                    });
                    boards.last_mut().unwrap()
                }
            };
            let issue = crate::models::Issue {
                id: r.id,
                epic_id: r.epic_id,
                mission_id: r.mission_id,
                sprint_id: r.sprint_id,
                title: r.title,
                description: r.description,
                goal: r.goal,
                status: r.status.clone(),
                priority: r.priority,
                assigned_agent: r.assigned_agent,
                created_at: r.created_at,
                updated_at: r.updated_at,
            };
            use crate::models::issue::IssueStatus::*;
            match r.status {
                Required  => board.required.push(issue),
                Ready     => board.ready.push(issue),
                Working   => board.working.push(issue),
                Demo      => board.demo.push(issue),
                Finished  => board.finished.push(issue),
                Cancelled => board.cancelled.push(issue),
            }
        }

        Ok(IssueBoardStatus {
            sprint_id: sprint.id,
            sprint_name: sprint.name.clone(),
            project_key: project_key.map(String::from),
            boards,
        })
    }
}
