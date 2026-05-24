use crate::models::history::{CreateHistoryInput, EntityType};
use crate::models::issue::*;
use crate::{Db, Error, Result};

const DESCRIPTION_EXCERPT_CHARS: usize = 100;

impl Db {
    pub async fn issue_create(&self, input: CreateIssueInput) -> Result<Issue> {
        if input.sprint_id.is_some() {
            return Err(Error::Validation(
                "sprint_id 는 더 이상 직접 지정할 수 없습니다. 이슈의 스프린트는 소속 미션을 통해 결정됩니다.".to_string()
            ));
        }
        let priority = input.priority.unwrap_or(IssuePriority::Medium);
        let pval = serde_json::to_value(&priority).unwrap().as_str().unwrap().to_string();

        // mission_id: 명시 전달이 없으면 부모 epic 에서 자동 상속.
        // epic 에도 mission_id 가 없으면 NULL 저장 (에러 없음).
        let mission_id = if input.mission_id.is_some() {
            input.mission_id
        } else {
            sqlx::query_scalar::<_, Option<i64>>(
                "SELECT mission_id FROM epics WHERE id = ?",
            )
            .bind(input.epic_id)
            .fetch_optional(&self.pool)
            .await?
            .flatten()
        };

        // RETURNING 절에서 (SELECT sprint_id FROM missions WHERE id = mission_id) 로 derived sprint_id 반환
        sqlx::query_as::<_, Issue>(
            "INSERT INTO issues (epic_id, mission_id, title, description, goal, priority) VALUES (?, ?, ?, ?, ?, ?)
             RETURNING id, epic_id, mission_id, (SELECT sprint_id FROM missions WHERE id = mission_id) AS sprint_id, title, description, goal, status, priority, assigned_agent, created_at, updated_at",
        )
        .bind(input.epic_id)
        .bind(mission_id)
        .bind(&input.title)
        .bind(&input.description)
        .bind(&input.goal)
        .bind(&pval)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn issue_get(&self, id: i64, compact: bool) -> Result<Issue> {
        let select_fields = if compact {
            "i.id, i.epic_id, i.mission_id, m.sprint_id AS sprint_id, i.title, NULL AS description, NULL AS goal, i.status, i.priority, i.assigned_agent, i.created_at, i.updated_at"
        } else {
            "i.id, i.epic_id, i.mission_id, m.sprint_id AS sprint_id, i.title, i.description, i.goal, i.status, i.priority, i.assigned_agent, i.created_at, i.updated_at"
        };
        sqlx::query_as::<_, Issue>(&format!(
            "SELECT {} FROM issues i LEFT JOIN missions m ON i.mission_id = m.id WHERE i.id = ?", select_fields
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| Error::NotFound(format!("issue:{id}")))
    }

    pub async fn issue_list(&self, filter: IssueFilter) -> Result<Vec<Issue>> {
        let mut sql = "SELECT i.id, i.epic_id, i.mission_id, m.sprint_id AS sprint_id, i.title, i.description, i.goal, i.status, i.priority, i.assigned_agent, i.created_at, i.updated_at FROM issues i LEFT JOIN missions m ON i.mission_id = m.id WHERE 1=1".to_string();
        if filter.epic_id.is_some()     { sql.push_str(" AND i.epic_id = ?"); }
        if filter.mission_id.is_some()  { sql.push_str(" AND i.mission_id = ?"); }
        if filter.sprint_id.is_some()   { sql.push_str(" AND m.sprint_id = ?"); }
        if filter.backlog_only           { sql.push_str(" AND (i.mission_id IS NULL OR m.sprint_id IS NULL)"); }
        if filter.project_key.is_some() {
            sql.push_str(" AND EXISTS (SELECT 1 FROM epics e WHERE e.id = i.epic_id AND e.project_key = ?)");
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
            sql.push_str(&format!(" AND i.status IN ({})", placeholders));
        }
        sql.push_str(" ORDER BY i.id DESC");

        let mut q = sqlx::query_as::<_, Issue>(&sql);
        if let Some(e) = filter.epic_id     { q = q.bind(e); }
        if let Some(m) = filter.mission_id  { q = q.bind(m); }
        if let Some(s) = filter.sprint_id   { q = q.bind(s); }
        if let Some(p) = filter.project_key { q = q.bind(p); }
        for s in target_statuses {
            let sv = serde_json::to_value(&s).unwrap().as_str().unwrap().to_string();
            q = q.bind(sv);
        }
        q.fetch_all(&self.pool).await.map_err(Into::into)
    }


    /// 이슈의 스프린트 소속을 변경한다. None 을 넘기면 백로그로 이동.
    pub async fn issue_set_sprint(&self, _id: i64, _sprint_id: Option<i64>, _changed_by: &str) -> Result<Issue> {
        Err(Error::Validation(
            "issue_set_sprint 는 deprecated 입니다. mission_set_sprint 또는 issue_update(mission_id=...) 를 사용하세요. ADR-0013 참조".to_string()
        ))
    }

    pub async fn issue_update(&self, id: i64, input: UpdateIssueInput, changed_by: &str) -> Result<Issue> {
        // Demo gate: finished/cancelled 전이는 사용자(agent_id="user")만 가능하다.
        // 에이전트가 규칙 파일 없이 호출하더라도 repository layer에서 코드 레벨로 막힌다.
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
            // blocked 이슈 전환 검증 — working/demo/finished 로 진입 시 활성 블로커 확인
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
            // working 을 벗어나는 모든 전이에서 assigned_agent 를 정리한다
            // (release 도구를 거치지 않는 사용자/agent 의 자유로운 칸반 이동에도 동작하도록).
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
        self.issue_get(id, false).await
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

    /// 이슈를 CAS(Compare-And-Set) 방식으로 점유한다 (멀티 에이전트 race 방지).
    ///
    /// 한 SQL 안에서 `status ∈ {ready, working}` 이고 `assigned_agent` 가 NULL 이거나
    /// 자기 자신일 때만 `working` + `assigned_agent=agent_id` 로 전이한다.
    /// 다른 에이전트가 잡고 있으면 `rows_affected=0` 으로 빠지므로 `Validation` 에러를 던진다.
    ///
    /// 같은 agent_id 가 재호출하면 idempotent (이미 자기가 잡은 working 이슈를 그대로 반환).
    pub async fn issue_claim(&self, id: i64, agent_id: &str) -> Result<Issue> {
        let current = self.issue_get(id, false).await?; // 존재 확인 + 디버그용 컨텍스트

        // claim = working 전환과 동치이므로 활성 블로커 확인
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

        // history 기록 (status 변화가 있었을 때만 기록 — 동일 agent 의 idempotent 재호출은 noise)
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

    /// 이슈 점유를 해제하고 지정 상태로 전이한다. 보통 `ready` (다른 agent 가 픽업 가능)
    /// 또는 `demo` (사용자 검토 대기) 로 전이한다.
    ///
    /// `force=false` 면 ownership 검증 — `agent_id` 가 현재 `assigned_agent` 와 다르면 거부.
    /// `force=true` 면 검증 우회 — 좀비 lease 회수, 사용자 또는 리더가 강제 ready 환원할 때 사용.
    /// history.changed_by 에는 항상 호출자의 `agent_id` 가 기록되므로 force 회수도 감사 가능.
    pub async fn issue_release(&self, id: i64, transition_to: IssueStatus, agent_id: &str, force: bool) -> Result<Issue> {
        let current = self.issue_get(id, false).await?;

        // ownership 검증 (force=false 일 때만)
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

    /// demo 상태의 이슈를 finished 로 전이한다 (사용자 전용).
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

    /// 임의 상태의 이슈를 cancelled 로 전이한다 (사용자 전용).
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

    /// 이슈를 삭제한다. 하위 태스크/노트/링크/태스크테스트도 함께 cascade 삭제.
    ///
    /// 스키마상 `tasks.issue_id ON DELETE RESTRICT` 라서 단순 DELETE 는 막힌다.
    /// 트랜잭션 내에서 task_tests → tasks → notes/links → issues 순으로 명시 삭제한다.
    /// (`notes`, `issue_links` 는 FK CASCADE 이지만 트랜잭션 일관성을 위해 명시적으로 처리)
    pub async fn issue_delete(&self, id: i64, changed_by: &str) -> Result<()> {
        let issue = self.issue_get(id, false).await?; // 존재 확인

        let mut tx = self.pool.begin().await?;

        // 1) 하위 태스크의 task_tests 먼저 제거 (task_tests.task_id CASCADE 지만 명시)
        sqlx::query(
            "DELETE FROM task_tests WHERE task_id IN (SELECT id FROM tasks WHERE issue_id = ?)",
        )
        .bind(id)
        .execute(&mut *tx)
        .await?;

        // 2) 태스크 제거 (RESTRICT 우회를 위해 트랜잭션 내 명시 삭제)
        sqlx::query("DELETE FROM tasks WHERE issue_id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        // 3) 노트 제거 (FK CASCADE 지만 명시)
        sqlx::query("DELETE FROM notes WHERE issue_id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        // 4) 이슈 링크 제거 (양방향)
        sqlx::query("DELETE FROM issue_links WHERE source_id = ? OR target_id = ?")
            .bind(id)
            .bind(id)
            .execute(&mut *tx)
            .await?;

        // 5) 이슈 자체 삭제
        sqlx::query("DELETE FROM issues WHERE id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        // 6) history 기록 (entity 가 사라졌으므로 entity_id 만 남는 deletion marker)
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

    /// 아직 해소되지 않은 블로커 이슈 ID 목록을 반환한다.
    ///
    ///  에서  이고  인 source 를 조회하되,
    /// source 이슈의 status 가  또는  가 아닌 것만 반환한다.
    /// (demo/finished 에 도달한 블로커는 "해소된 것"으로 간주)
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

        let mut sql = r#"
            SELECT i.id, i.epic_id, i.mission_id, m.sprint_id AS sprint_id, i.title, i.description, i.goal, i.status, i.priority, i.assigned_agent, i.created_at, i.updated_at
            FROM issues i
            JOIN epics e ON i.epic_id = e.id
            LEFT JOIN missions m ON i.mission_id = m.id
            WHERE e.project_key = ?
        "#.to_string();

        if sid.is_some() {
            sql.push_str(" AND m.sprint_id = ?");
        } else {
            sql.push_str(" AND (i.mission_id IS NULL OR m.sprint_id IS NULL)");
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

        // 미션 A, B 생성
        let mission_a = db.mission_create(CreateMissionInput {
            title: "Mission A".to_string(),
            description: None,
            jira_key: None,
            sprint_id: Some(sprint.id),
        }).await.unwrap();
        let mission_b = db.mission_create(CreateMissionInput {
            title: "Mission B".to_string(),
            description: None,
            jira_key: None,
            sprint_id: Some(sprint.id),
        }).await.unwrap();

        // 에픽 A (mission_id=A), 에픽 B (mission_id=B)
        let epic_a = db.epic_create(CreateEpicInput {
            mission_id: Some(mission_a.id),
            project_key: "proj".to_string(),
            title: "Epic A".to_string(),
            description: None,
        }).await.unwrap();
        let epic_b = db.epic_create(CreateEpicInput {
            mission_id: Some(mission_b.id),
            project_key: "proj".to_string(),
            title: "Epic B".to_string(),
            description: None,
        }).await.unwrap();

        // 이슈 A1, A2 (epic_a → mission_id 자동 상속)
        let issue_a1 = db.issue_create(CreateIssueInput {
            epic_id: epic_a.id,
            mission_id: None,
            sprint_id: None,
            title: "Issue A1".to_string(),
            description: None,
            goal: None,
            priority: None,
        }).await.unwrap();
        let issue_a2 = db.issue_create(CreateIssueInput {
            epic_id: epic_a.id,
            mission_id: None,
            sprint_id: None,
            title: "Issue A2".to_string(),
            description: None,
            goal: None,
            priority: None,
        }).await.unwrap();

        // 이슈 B1 (epic_b → mission_id 자동 상속)
        let issue_b1 = db.issue_create(CreateIssueInput {
            epic_id: epic_b.id,
            mission_id: None,
            sprint_id: None,
            title: "Issue B1".to_string(),
            description: None,
            goal: None,
            priority: None,
        }).await.unwrap();

        // mission_id 상속 확인
        assert_eq!(issue_a1.mission_id, Some(mission_a.id), "A1은 mission A 상속");
        assert_eq!(issue_a2.mission_id, Some(mission_a.id), "A2은 mission A 상속");
        assert_eq!(issue_b1.mission_id, Some(mission_b.id), "B1은 mission B 상속");

        // mission_a 필터 → A1, A2 만
        let result_a = db.issue_list(IssueFilter {
            mission_id: Some(mission_a.id),
            ..Default::default()
        }).await.unwrap();
        let ids_a: Vec<i64> = result_a.iter().map(|i| i.id).collect();
        assert!(ids_a.contains(&issue_a1.id), "A1이 mission_a 결과에 있어야 함");
        assert!(ids_a.contains(&issue_a2.id), "A2가 mission_a 결과에 있어야 함");
        assert!(!ids_a.contains(&issue_b1.id), "B1은 mission_a 결과에 없어야 함");
        assert_eq!(result_a.len(), 2, "mission A 필터 결과는 2건");

        // mission_b 필터 → B1 만
        let result_b = db.issue_list(IssueFilter {
            mission_id: Some(mission_b.id),
            ..Default::default()
        }).await.unwrap();
        let ids_b: Vec<i64> = result_b.iter().map(|i| i.id).collect();
        assert!(ids_b.contains(&issue_b1.id), "B1이 mission_b 결과에 있어야 함");
        assert!(!ids_b.contains(&issue_a1.id), "A1은 mission_b 결과에 없어야 함");
        assert_eq!(result_b.len(), 1, "mission B 필터 결과는 1건");
    }
}

