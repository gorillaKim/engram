use crate::models::history::{CreateHistoryInput, EntityType};
use crate::models::note::*;
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
                    return Err(Error::Validation("issue scope 는 issue_id (>0) 가 필수입니다".to_string()));
                }
                (Some(input.issue_id), input.scope_target_id.or(Some(input.issue_id)), None)
            }
            NoteScope::Task => {
                if input.issue_id <= 0 || input.task_id.is_none() {
                    return Err(Error::Validation("task scope 는 issue_id (>0) 와 task_id 둘 다 필수입니다".to_string()));
                }
                (Some(input.issue_id), input.scope_target_id.or(input.task_id), None)
            }
            NoteScope::Project => {
                let pk = input.project_key.clone()
                    .ok_or_else(|| Error::Validation("project scope 는 project_key 가 필수입니다".to_string()))?;
                (None, None, Some(pk))
            }
            NoteScope::Sprint => {
                let target = input.scope_target_id
                    .ok_or_else(|| Error::Validation("sprint scope 는 scope_target_id (sprint id) 가 필수입니다".to_string()))?;
                (None, Some(target), None)
            }
            NoteScope::Epic => {
                let target = input.scope_target_id
                    .ok_or_else(|| Error::Validation("epic scope 는 scope_target_id (epic id) 가 필수입니다".to_string()))?;
                (None, Some(target), None)
            }
        };

        // RETURNING * — WAL 가시성 회피.
        sqlx::query_as::<_, Note>(
            "INSERT INTO notes (issue_id, task_id, note_type, summary, detail, author, agent_id, scope, scope_target_id, project_key) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             RETURNING id, issue_id, task_id, note_type, summary, detail, author, agent_id, resolved, scope, scope_target_id, project_key, created_at, resolved_at",
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
            "id, issue_id, task_id, note_type, summary, NULL AS detail, author, agent_id, resolved, scope, scope_target_id, project_key, created_at, resolved_at"
        } else {
            "id, issue_id, task_id, note_type, summary, detail, author, agent_id, resolved, scope, scope_target_id, project_key, created_at, resolved_at"
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
        include_resolved: bool,
        include_detail: bool,
    ) -> Result<Vec<Note>> {
        let select_fields = if include_detail {
            "id, issue_id, task_id, note_type, summary, detail, author, agent_id, resolved, scope, scope_target_id, project_key, created_at, resolved_at"
        } else {
            "id, issue_id, task_id, note_type, summary, NULL AS detail, author, agent_id, resolved, scope, scope_target_id, project_key, created_at, resolved_at"
        };
        let mut sql = format!("SELECT {} FROM notes WHERE 1=1", select_fields);
        if issue_id.is_some()  { sql.push_str(" AND issue_id = ?"); }
        if task_id.is_some()   { sql.push_str(" AND task_id = ?"); }
        if note_type.is_some() { sql.push_str(" AND note_type = ?"); }
        if !include_resolved   { sql.push_str(" AND resolved = 0"); }
        sql.push_str(" ORDER BY created_at DESC");

        let mut q = sqlx::query_as::<_, Note>(&sql);
        if let Some(i) = issue_id { q = q.bind(i); }
        if let Some(t) = task_id  { q = q.bind(t); }
        if let Some(nt) = note_type {
            let ntv = serde_json::to_value(&nt).unwrap().as_str().unwrap().to_string();
            q = q.bind(ntv);
        }
        q.fetch_all(&self.pool).await.map_err(Into::into)
    }

    pub async fn note_resolve(&self, id: i64, changed_by: &str) -> Result<Note> {
        sqlx::query("UPDATE notes SET resolved = 1, resolved_at = datetime('now') WHERE id = ?")
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
            let nt = serde_json::to_value(&input.note_type).unwrap().as_str().unwrap().to_string();

            // scope 자동 판정
            let scope = input.scope.unwrap_or_else(|| {
                if input.task_id.is_some() { NoteScope::Task } else { NoteScope::Issue }
            });
            let scope_str = serde_json::to_value(scope).unwrap().as_str().unwrap().to_string();

            // scope 별 검증 + 컬럼 채우기 결정
            let (issue_id_db, scope_target_id_db, project_key_db) = match scope {
                NoteScope::Issue => {
                    if input.issue_id <= 0 {
                        return Err(Error::Validation("issue scope 는 issue_id (>0) 가 필수입니다".to_string()));
                    }
                    (Some(input.issue_id), input.scope_target_id.or(Some(input.issue_id)), None)
                }
                NoteScope::Task => {
                    if input.issue_id <= 0 || input.task_id.is_none() {
                        return Err(Error::Validation("task scope 는 issue_id (>0) 와 task_id 둘 다 필수입니다".to_string()));
                    }
                    (Some(input.issue_id), input.scope_target_id.or(input.task_id), None)
                }
                NoteScope::Project => {
                    let pk = input.project_key.clone()
                        .ok_or_else(|| Error::Validation("project scope 는 project_key 가 필수입니다".to_string()))?;
                    (None, None, Some(pk))
                }
                NoteScope::Sprint => {
                    let target = input.scope_target_id
                        .ok_or_else(|| Error::Validation("sprint scope 는 scope_target_id (sprint id) 가 필수입니다".to_string()))?;
                    (None, Some(target), None)
                }
                NoteScope::Epic => {
                    let target = input.scope_target_id
                        .ok_or_else(|| Error::Validation("epic scope 는 scope_target_id (epic id) 가 필수입니다".to_string()))?;
                    (None, Some(target), None)
                }
            };

            let note = sqlx::query_as::<_, Note>(
                "INSERT INTO notes (issue_id, task_id, note_type, summary, detail, author, agent_id, scope, scope_target_id, project_key) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                 RETURNING id, issue_id, task_id, note_type, summary, detail, author, agent_id, resolved, scope, scope_target_id, project_key, created_at, resolved_at",
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
}
