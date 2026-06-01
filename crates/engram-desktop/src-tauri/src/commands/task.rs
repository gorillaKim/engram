use crate::commands::parse;
use engram_core::{
    Db,
    models::task::{Task, CreateTaskInput, TaskSource, TaskStatus, UpdateTaskInput},
};
use std::sync::Arc;
use tauri::State;

// ── Inner functions (testable without Tauri context) ──────────────────────────

pub async fn do_task_list(db: &Db, issue_id: i64) -> engram_core::Result<Vec<Task>> {
    db.task_list(issue_id, None).await
}

pub async fn do_task_set_status(
    db: &Db,
    id: i64,
    status: &str,
) -> engram_core::Result<Task> {
    let parsed: TaskStatus = parse(status)?;
    db.task_update(
        id,
        UpdateTaskInput {
            status: Some(parsed),
            ..Default::default()
        },
        "user",
    )
    .await
}

pub async fn do_task_create(
    db: &Db,
    issue_id: i64,
    title: String,
) -> engram_core::Result<Task> {
    db.task_create(CreateTaskInput {
        issue_id,
        title,
        description: None,
        goal: None,
        after_task_id: None,
        source: Some(TaskSource::UserAdded),
    }).await
}

// ── Tauri command wrappers ────────────────────────────────────────────────────

#[tauri::command(rename_all = "snake_case")]
pub async fn task_list(db: State<'_, Arc<Db>>, issue_id: i64) -> Result<Vec<Task>, String> {
    do_task_list(&db, issue_id).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn task_set_status(
    db: State<'_, Arc<Db>>,
    id: i64,
    status: String,
) -> Result<Task, String> {
    do_task_set_status(&db, id, &status).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn task_create(
    db: State<'_, Arc<Db>>,
    issue_id: i64,
    title: String,
) -> Result<Task, String> {
    do_task_create(&db, issue_id, title).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn task_delete(
    db: State<'_, Arc<Db>>,
    id: i64,
) -> Result<(), String> {
    db.task_delete(id).await.map_err(|e| e.to_string())
}
