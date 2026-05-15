use crate::models::history::{CreateHistoryInput, EntityType};
use crate::models::note::*;
use crate::{Db, Error, Result};

impl Db {
    pub async fn note_add(&self, input: CreateNoteInput) -> Result<Note> {
        let author = input.author.unwrap_or_else(|| "agent".to_string());
        let nt = serde_json::to_value(&input.note_type).unwrap().as_str().unwrap().to_string();
        let id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO notes (issue_id, task_id, note_type, summary, detail, author) VALUES (?, ?, ?, ?, ?, ?) RETURNING id",
        )
        .bind(input.issue_id)
        .bind(input.task_id)
        .bind(&nt)
        .bind(&input.summary)
        .bind(&input.detail)
        .bind(&author)
        .fetch_one(&self.pool)
        .await?;
        self.note_get(id).await
    }

    pub async fn note_get(&self, id: i64) -> Result<Note> {
        sqlx::query_as::<_, Note>(
            "SELECT id, issue_id, task_id, note_type, summary, detail, author, resolved, created_at, resolved_at FROM notes WHERE id = ?",
        )
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
    ) -> Result<Vec<Note>> {
        let mut sql = "SELECT id, issue_id, task_id, note_type, summary, detail, author, resolved, created_at, resolved_at FROM notes WHERE 1=1".to_string();
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

    pub async fn note_resolve(&self, id: i64) -> Result<Note> {
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
            changed_by: "agent".to_string(),
        }).await;
        self.note_get(id).await
    }
}
