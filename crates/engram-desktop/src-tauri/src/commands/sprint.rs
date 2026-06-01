use crate::commands::parse;
use engram_core::{
    Db,
    models::sprint::{Sprint, CreateSprintInput, UpdateSprintInput, SprintStatus},
};
use std::sync::Arc;
use tauri::State;

// ── Inner functions (testable without Tauri context) ──────────────────────────

pub async fn do_sprint_current(db: &Db) -> engram_core::Result<Option<Sprint>> {
    db.sprint_current().await
}

pub async fn do_sprint_list(db: &Db) -> engram_core::Result<Vec<Sprint>> {
    db.sprint_list(None).await
}

pub async fn do_sprint_create(
    db: &Db,
    name: String,
    goal: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> engram_core::Result<Sprint> {
    db.sprint_create(CreateSprintInput { name, goal, start_date, end_date }).await
}

pub async fn do_sprint_delete(db: &Db, id: i64) -> engram_core::Result<()> {
    db.sprint_delete(id).await
}

// ── Tauri command wrappers ────────────────────────────────────────────────────

#[tauri::command(rename_all = "snake_case")]
pub async fn sprint_current(db: State<'_, Arc<Db>>) -> Result<Option<Sprint>, String> {
    do_sprint_current(&db).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn sprint_list(db: State<'_, Arc<Db>>) -> Result<Vec<Sprint>, String> {
    do_sprint_list(&db).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn sprint_create(
    db: State<'_, Arc<Db>>,
    name: String,
    goal: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Sprint, String> {
    do_sprint_create(&db, name, goal, start_date, end_date).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn sprint_update(
    db: State<'_, Arc<Db>>,
    id: i64,
    name: Option<String>,
    goal: Option<String>,
    status: Option<String>,
) -> Result<Sprint, String> {
    let status_parsed = if let Some(s) = status {
        Some(parse::<SprintStatus>(&s).map_err(|e| e.to_string())?)
    } else {
        None
    };
    db.sprint_update(id, UpdateSprintInput { name, goal, status: status_parsed, ..Default::default() }, "user")
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn sprint_delete(
    db: State<'_, Arc<Db>>,
    id: i64,
) -> Result<(), String> {
    do_sprint_delete(&db, id).await.map_err(|e| e.to_string())
}
