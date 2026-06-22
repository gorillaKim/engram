use engram_core::{
    Db,
    models::note::{Note, CreateNoteInput},
};
use std::sync::Arc;
use tauri::State;

// ── Inner functions (testable without Tauri context) ──────────────────────────

pub async fn do_note_list(db: &Db, issue_id: i64) -> engram_core::Result<Vec<Note>> {
    let res = db.note_list(Some(issue_id), None, None, None, false, true, None, None, None, None, None, None, None, None).await?;
    Ok(res.items)
}

pub async fn do_note_get(db: &Db, id: i64) -> engram_core::Result<Note> {
    db.note_get(id, false).await
}

pub async fn do_note_add(db: &Db, input: CreateNoteInput) -> engram_core::Result<Note> {
    db.note_add(input).await
}

pub async fn do_note_resolve(db: &Db, id: i64) -> engram_core::Result<Note> {
    db.note_resolve(id, "user").await
}

// ── Tauri command wrappers ────────────────────────────────────────────────────

#[tauri::command(rename_all = "snake_case")]
pub async fn note_list(db: State<'_, Arc<Db>>, issue_id: i64) -> Result<Vec<Note>, String> {
    do_note_list(&db, issue_id).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn note_get(db: State<'_, Arc<Db>>, id: i64) -> Result<Note, String> {
    do_note_get(&db, id).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn note_add(
    db: State<'_, Arc<Db>>,
    input: CreateNoteInput,
) -> Result<Note, String> {
    do_note_add(&db, input).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn note_resolve(db: State<'_, Arc<Db>>, id: i64) -> Result<Note, String> {
    do_note_resolve(&db, id).await.map_err(|e| e.to_string())
}
