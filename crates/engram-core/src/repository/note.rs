use crate::models::history::{CreateHistoryInput, EntityType};
use crate::models::note::*;
use crate::models::PaginatedResponse;
use crate::models::{OutputMode, CoreResponse};
use crate::{Db, Error, Result};

impl Db {
    pub async fn note_add(&self, input: CreateNoteInput) -> Result<Note> {
        let author = input.author.unwrap_or_else(|| "agent".to_string());
        let nt = serde_json::to_value(&input.note_type).unwrap().as_str().unwrap().to_string();

        // scope 자동 판정: 명시되지 않으면 task_id 가 있으면 task, 아니면 issue
        let scope = input.scope.unwrap_or_else(|| {
            if input.task_id.is_some() { NoteScope::Task } else { NoteScope::Issue }
        });
        let scope_str = serde_json::to_value(scope).unwrap().as_str().unwrap().to_string();

        // scope 별 검증 + 컬럼 채우기 결정
        let (issue_id_db, scope_target_id_db, project_key_db) = match scope {
            NoteScope::Issue => {
                if input.issue_id <= 0 {
                    return Err(Error::Validation(
                        r#"{"scope":"issue","expected_fields":["issue_id"],"message":"issue scope 는 issue_id (>0) 가 필수입니다"}"#.to_string()
                    ));
                }
                (Some(input.issue_id), input.scope_target_id.or(Some(input.issue_id)), None)
            }
            NoteScope::Task => {
                if input.issue_id <= 0 || input.task_id.is_none() {
                    return Err(Error::Validation(
                        r#"{"scope":"task","expected_fields":["issue_id","task_id"],"message":"task scope 는 issue_id (>0) 와 task_id 둘 다 필수입니다"}"#.to_string()
                    ));
                }
                (Some(input.issue_id), input.scope_target_id.or(input.task_id), None)
            }
            NoteScope::Project => {
                let pk = input.project_key.clone()
                    .ok_or_else(|| Error::Validation(
                        r#"{"scope":"project","expected_fields":["project_key"],"message":"project scope 는 project_key 가 필수입니다"}"#.to_string()
                    ))?;
                (None, None, Some(pk))
            }
            NoteScope::Sprint => {
                let target = input.scope_target_id
                    .ok_or_else(|| Error::Validation(
                        r#"{"scope":"sprint","expected_fields":["scope_target_id"],"message":"sprint scope 는 scope_target_id (sprint id) 가 필수입니다"}"#.to_string()
                    ))?;
                (None, Some(target), None)
            }
            NoteScope::Epic => {
                let target = input.scope_target_id
                    .ok_or_else(|| Error::Validation(
                        r#"{"scope":"epic","expected_fields":["scope_target_id"],"message":"epic scope 는 scope_target_id (epic id) 가 필수입니다"}"#.to_string()
                    ))?;
                (None, Some(target), None)
            }
        };


        // RETURNING * — WAL 가시성 회피.
        sqlx::query_as::<_, Note>(
            "INSERT INTO notes (issue_id, task_id, note_type, summary, detail, author, agent_id, scope, scope_target_id, project_key) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             RETURNING id, issue_id, task_id, note_type, summary, detail, author, agent_id, resolved, scope, scope_target_id, project_key, created_at, resolved_at, updated_at",
        )
        .bind(issue_id_db)
        .bind(input.task_id)
        .bind(&nt)
        .bind(&input.summary)
        .bind(&input.detail)
        .bind(&author)
        .bind(&input.agent_id)
        .bind(&scope_str)
        .bind(scope_target_id_db)
        .bind(&project_key_db)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn note_get(&self, id: i64, compact: bool) -> Result<Note> {
        let select_fields = if compact {
            "id, issue_id, task_id, note_type, summary, NULL AS detail, author, agent_id, resolved, scope, scope_target_id, project_key, created_at, resolved_at, updated_at"
        } else {
            "id, issue_id, task_id, note_type, summary, detail, author, agent_id, resolved, scope, scope_target_id, project_key, created_at, resolved_at, updated_at"
        };
        sqlx::query_as::<_, Note>(&format!(
            "SELECT {} FROM notes WHERE id = ?", select_fields
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| Error::NotFound(format!("note:{id}")))
    }

    /// session_restore용: summary만 반환 (detail 제외)
    pub async fn note_summaries(
        &self,
        issue_id: i64,
        include_resolved: bool,
    ) -> Result<Vec<NoteSummary>> {
        #[derive(sqlx::FromRow)]
        struct SummaryRow {
            id: i64,
            note_type: NoteType,
            summary: String,
            task_id: Option<i64>,
            resolved: bool,
        }

        let mut sql = "SELECT id, note_type, summary, task_id, resolved FROM notes WHERE issue_id = ?".to_string();
        if !include_resolved {
            sql.push_str(" AND resolved = 0");
        }
        sql.push_str(" ORDER BY created_at ASC");

        let rows = sqlx::query_as::<_, SummaryRow>(&sql)
            .bind(issue_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(|r| NoteSummary {
            id: r.id,
            note_type: r.note_type,
            summary: r.summary,
            task_id: r.task_id,
            resolved: r.resolved,
        }).collect())
    }

    pub async fn note_list(
        &self,
        issue_id: Option<i64>,
        task_id: Option<i64>,
        note_type: Option<NoteType>,
        note_types: Option<Vec<NoteType>>,
        include_resolved: bool,
        include_detail: bool,
        project_key: Option<&str>,
        sprint_id: Option<i64>,
        epic_id: Option<i64>,
        rollup: Option<bool>,
        limit: Option<i64>,
        offset: Option<i64>,
        compact: Option<bool>,
        updated_after: Option<String>,
    ) -> Result<PaginatedResponse<Note>> {
        let select_fields = if include_detail {
            "n.detail"
        } else if compact.unwrap_or(false) {
            "CASE WHEN n.detail IS NOT NULL THEN SUBSTR(n.detail, 1, 200) ELSE NULL END AS detail"
        } else {
            "NULL AS detail"
        };
        let mut from_where = "FROM notes n \
             LEFT JOIN issues i ON n.issue_id = i.id \
             LEFT JOIN epics ie ON i.epic_id = ie.id \
             LEFT JOIN epics ee ON (n.scope = 'epic' AND n.scope_target_id = ee.id) \
             WHERE 1=1".to_string();

        if issue_id.is_some()  { from_where.push_str(" AND n.issue_id = ?"); }
        if task_id.is_some()   { from_where.push_str(" AND n.task_id = ?"); }
        if note_type.is_some() { from_where.push_str(" AND n.note_type = ?"); }
        if let Some(ref types) = note_types {
            if !types.is_empty() {
                let placeholders = types.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                from_where.push_str(&format!(" AND n.note_type IN ({})", placeholders));
            }
        }
        if !include_resolved   { from_where.push_str(" AND n.resolved = 0"); }
        if project_key.is_some() {
            from_where.push_str(" AND COALESCE(n.project_key, ie.project_key, ee.project_key) = ?");
        }
        if sprint_id.is_some() {
            from_where.push_str(" AND COALESCE(CASE WHEN n.scope = 'sprint' THEN n.scope_target_id ELSE NULL END, ie.sprint_id, ee.sprint_id) = ?");
        }
        if let Some(eid) = epic_id {
            if rollup.unwrap_or(false) {
                from_where.push_str(" AND ( (n.scope = 'epic' AND n.scope_target_id = ?) OR (n.issue_id IN (SELECT id FROM issues WHERE epic_id = ?)) OR (n.task_id IN (SELECT id FROM tasks WHERE issue_id IN (SELECT id FROM issues WHERE epic_id = ?))) )");
            } else {
                from_where.push_str(" AND n.scope = 'epic' AND n.scope_target_id = ?");
            }
        }
        if updated_after.is_some() {
            from_where.push_str(" AND n.updated_at > ?");
        }

        // 1) total count
        let count_sql = format!("SELECT COUNT(*) {from_where}");
        let mut count_q = sqlx::query_scalar::<_, i64>(&count_sql);

        // 2) items
        let (lim, off) = crate::repository::apply_pagination(limit, offset);
        let sql = format!(
            "SELECT n.id, n.issue_id, n.task_id, n.note_type, n.summary, {}, n.author, n.agent_id, n.resolved, n.scope, n.scope_target_id, n.project_key, n.created_at, n.resolved_at, n.updated_at
             {from_where} ORDER BY n.created_at DESC LIMIT ? OFFSET ?",
            select_fields
        );
        let mut q = sqlx::query_as::<_, Note>(&sql);

        // 바인딩
        if let Some(i) = issue_id {
            count_q = count_q.bind(i);
            q = q.bind(i);
        }
        if let Some(t) = task_id {
            count_q = count_q.bind(t);
            q = q.bind(t);
        }
        if let Some(nt) = note_type {
            let ntv = serde_json::to_value(&nt).unwrap().as_str().unwrap().to_string();
            count_q = count_q.bind(ntv.clone());
            q = q.bind(ntv);
        }
        if let Some(ref types) = note_types {
            for nt in types {
                let ntv = serde_json::to_value(nt).unwrap().as_str().unwrap().to_string();
                count_q = count_q.bind(ntv.clone());
                q = q.bind(ntv);
            }
        }
        if let Some(pk) = project_key {
            let pks = pk.to_string();
            count_q = count_q.bind(pks.clone());
            q = q.bind(pks);
        }
        if let Some(sid) = sprint_id {
            count_q = count_q.bind(sid);
            q = q.bind(sid);
        }
        if let Some(eid) = epic_id {
            if rollup.unwrap_or(false) {
                count_q = count_q.bind(eid).bind(eid).bind(eid);
                q = q.bind(eid).bind(eid).bind(eid);
            } else {
                count_q = count_q.bind(eid);
                q = q.bind(eid);
            }
        }
        if let Some(ref ua) = updated_after {
            let uas = ua.to_string();
            count_q = count_q.bind(uas.clone());
            q = q.bind(uas);
        }

        // q 에만 pagination 바인딩
        q = q.bind(lim).bind(off);

        let total = count_q.fetch_one(&self.pool).await.unwrap_or(0);
        let items = q.fetch_all(&self.pool).await?;
        let has_more = (off + items.len() as i64) < total;

        Ok(PaginatedResponse { items, total, has_more })
    }

    pub async fn note_list_mode(
        &self,
        issue_id: Option<i64>,
        task_id: Option<i64>,
        note_type: Option<NoteType>,
        note_types: Option<Vec<NoteType>>,
        include_resolved: bool,
        include_detail: bool,
        project_key: Option<&str>,
        sprint_id: Option<i64>,
        epic_id: Option<i64>,
        rollup: Option<bool>,
        limit: Option<i64>,
        offset: Option<i64>,
        mode: OutputMode,
        updated_after: Option<String>,
    ) -> Result<CoreResponse<PaginatedResponse<Note>>> {
        let is_ref = matches!(mode, OutputMode::Ref);
        let is_agent = matches!(mode, OutputMode::Agent);
        let is_compact = matches!(mode, OutputMode::Compact) || is_agent || is_ref;
        let paginated = self.note_list(
            issue_id,
            task_id,
            note_type,
            note_types,
            include_resolved,
            if is_compact { false } else { include_detail },
            project_key,
            sprint_id,
            epic_id,
            rollup,
            limit,
            offset,
            if is_compact { Some(true) } else { None },
            updated_after,
        ).await?;

        if is_ref {
            // Ref 모드: #id type summary 한 줄씩 — 토큰 최소화 인덱스 전용
            let mut out = String::new();
            out.push_str("=== NOTE REF LIST ===\n");
            if paginated.items.is_empty() {
                out.push_str("- None\n");
            } else {
                for note in &paginated.items {
                    let type_val = serde_json::to_value(&note.note_type).unwrap();
                    let type_str = type_val.as_str().unwrap_or("general");
                    out.push_str(&format!("#{} [{}] {}\n", note.id, type_str, note.summary));
                }
            }
            out.push_str(&format!("Total: {} | Has More: {}\n", paginated.total, paginated.has_more));
            out.push_str("====================");
            Ok(CoreResponse::Text(out))
        } else if is_agent {
            let mut out = String::new();
            out.push_str("=== NOTE LIST ===\n");
            if paginated.items.is_empty() {
                out.push_str("- None\n");
            } else {
                for note in &paginated.items {
                    let type_val = serde_json::to_value(&note.note_type).unwrap();
                    let type_str = type_val.as_str().unwrap_or("general");
                    let resolved_mark = if note.resolved { "[Resolved] " } else { "" };
                    out.push_str(&format!(
                        "- #{} ({}{}): {}\n",
                        note.id, resolved_mark, type_str, note.summary
                    ));
                }
            }
            out.push_str(&format!("Total: {} | Has More: {}\n", paginated.total, paginated.has_more));
            out.push_str("=================");
            Ok(CoreResponse::Text(out))
        } else {
            Ok(CoreResponse::Json(paginated))
        }
    }

    pub async fn note_resolve(&self, id: i64, changed_by: &str) -> Result<Note> {
        sqlx::query("UPDATE notes SET resolved = 1, resolved_at = datetime('now'), updated_at = datetime('now') WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        let _ = self.history_record(CreateHistoryInput {
            entity_type: EntityType::Note,
            entity_id: id,
            field: "resolved".to_string(),
            old_value: Some("false".to_string()),
            new_value: Some("true".to_string()),
            changed_by: changed_by.to_string(),
        }).await;
        self.note_get(id, false).await
    }

    pub async fn note_add_bulk(&self, inputs: Vec<CreateNoteInput>) -> Result<Vec<Note>> {
        let mut tx = self.pool.begin().await?;
        let mut notes = Vec::new();
        for input in inputs {
            let author = input.author.clone().unwrap_or_else(|| "agent".to_string());
            let nt = serde_json::to_value(&input.note_type)
                .ok()
                .and_then(|v| v.as_str().map(String::from))
                .ok_or_else(|| Error::Validation("note_type 직렬화 실패".to_string()))?;

            // scope 자동 판정
            let scope = input.scope.unwrap_or_else(|| {
                if input.task_id.is_some() { NoteScope::Task } else { NoteScope::Issue }
            });
            let scope_str = serde_json::to_value(scope)
                .ok()
                .and_then(|v| v.as_str().map(String::from))
                .ok_or_else(|| Error::Validation("scope 직렬화 실패".to_string()))?;

            // scope 별 검증 + 컬럼 채우기 결정
            let (issue_id_db, scope_target_id_db, project_key_db) = match scope {
                NoteScope::Issue => {
                    if input.issue_id <= 0 {
                        return Err(Error::Validation(
                            r#"{"scope":"issue","expected_fields":["issue_id"],"message":"issue scope 는 issue_id (>0) 가 필수입니다"}"#.to_string()
                        ));
                    }
                    (Some(input.issue_id), input.scope_target_id.or(Some(input.issue_id)), None)
                }
                NoteScope::Task => {
                    if input.issue_id <= 0 || input.task_id.is_none() {
                        return Err(Error::Validation(
                            r#"{"scope":"task","expected_fields":["issue_id","task_id"],"message":"task scope 는 issue_id (>0) 와 task_id 둘 다 필수입니다"}"#.to_string()
                        ));
                    }
                    (Some(input.issue_id), input.scope_target_id.or(input.task_id), None)
                }
                NoteScope::Project => {
                    let pk = input.project_key.clone()
                        .ok_or_else(|| Error::Validation(
                            r#"{"scope":"project","expected_fields":["project_key"],"message":"project scope 는 project_key 가 필수입니다"}"#.to_string()
                        ))?;
                    (None, None, Some(pk))
                }
                NoteScope::Sprint => {
                    let target = input.scope_target_id
                        .ok_or_else(|| Error::Validation(
                            r#"{"scope":"sprint","expected_fields":["scope_target_id"],"message":"sprint scope 는 scope_target_id (sprint id) 가 필수입니다"}"#.to_string()
                        ))?;
                    (None, Some(target), None)
                }
                NoteScope::Epic => {
                    let target = input.scope_target_id
                        .ok_or_else(|| Error::Validation(
                            r#"{"scope":"epic","expected_fields":["scope_target_id"],"message":"epic scope 는 scope_target_id (epic id) 가 필수입니다"}"#.to_string()
                        ))?;
                    (None, Some(target), None)
                }
            };

            let note = sqlx::query_as::<_, Note>(
                "INSERT INTO notes (issue_id, task_id, note_type, summary, detail, author, agent_id, scope, scope_target_id, project_key) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                 RETURNING id, issue_id, task_id, note_type, summary, detail, author, agent_id, resolved, scope, scope_target_id, project_key, created_at, resolved_at, updated_at",
            )
            .bind(issue_id_db)
            .bind(input.task_id)
            .bind(&nt)
            .bind(&input.summary)
            .bind(&input.detail)
            .bind(&author)
            .bind(&input.agent_id)
            .bind(&scope_str)
            .bind(scope_target_id_db)
            .bind(&project_key_db)
            .fetch_one(&mut *tx)
            .await?;

            notes.push(note);
        }
        tx.commit().await?;
        Ok(notes)
    }

    pub async fn note_get_batch(&self, ids: &[i64], compact: bool, mode: OutputMode) -> Result<CoreResponse<Vec<Note>>> {
        let select_fields = if compact {
            "id, issue_id, task_id, note_type, summary, NULL AS detail, author, agent_id, resolved, scope, scope_target_id, project_key, created_at, resolved_at, updated_at"
        } else {
            "id, issue_id, task_id, note_type, summary, detail, author, agent_id, resolved, scope, scope_target_id, project_key, created_at, resolved_at, updated_at"
        };
        if ids.is_empty() {
            return Ok(if matches!(mode, OutputMode::Agent) {
                CoreResponse::Text("=== NOTE BATCH ===\nNo notes specified.\n====================".to_string())
            } else {
                CoreResponse::Json(Vec::new())
            });
        }
        let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "SELECT {select_fields} FROM notes WHERE id IN ({placeholders})"
        );
        let mut q = sqlx::query_as::<_, Note>(&sql);
        for id in ids { q = q.bind(id); }
        let notes = q.fetch_all(&self.pool).await?;

        if matches!(mode, OutputMode::Agent) {
            let mut out = String::new();
            out.push_str("=== NOTE BATCH ===\n");
            for note in &notes {
                out.push_str(&format_agent_note_text(note));
                out.push_str("\n\n");
            }
            out.push_str("====================");
            Ok(CoreResponse::Text(out))
        } else {
            Ok(CoreResponse::Json(notes))
        }
    }
}

fn format_agent_note_text(note: &Note) -> String {
    let mut out = String::new();
    out.push_str(&format!("note #{}\n", note.id));
    if let Some(iid) = note.issue_id { out.push_str(&format!("issue: {}\n", iid)); }
    if let Some(tid) = note.task_id { out.push_str(&format!("task: {}\n", tid)); }
    let note_type_str = serde_json::to_value(&note.note_type).unwrap().as_str().unwrap_or("context").to_string();
    out.push_str(&format!("type: {}\n", note_type_str));
    out.push_str(&format!("resolved: {}\n", note.resolved));
    out.push_str(&format!("created: {}\n", note.created_at));
    out.push_str(&format!("updated: {}\n", note.updated_at));
    out.push_str(&format!("summary: {}\n", note.summary));
    if let Some(ref detail) = note.detail {
        out.push_str(&format!("detail: {}\n", detail));
    }
    out
}
