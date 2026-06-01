use crate::commands::parse;
use engram_core::{
    Db,
    models::epic::{Epic, CreateEpicInput, EpicStatus, UpdateEpicInput},
};
use std::sync::Arc;
use tauri::State;

// ── Inner functions (testable without Tauri context) ──────────────────────────

pub async fn do_epic_list(
    db: &Db,
    project_key: Option<&str>,
    include_completed: bool,
) -> engram_core::Result<Vec<Epic>> {
    db.epic_list(project_key, include_completed).await
}

pub async fn do_epic_create(
    db: &Db,
    project_key: String,
    mission_id: Option<i64>,
    sprint_id: Option<i64>,
    title: String,
    description: Option<String>,
) -> engram_core::Result<Epic> {
    db.epic_create(CreateEpicInput { project_key, mission_id, sprint_id, title, description }).await
}

pub async fn do_epic_set_status(
    db: &Db,
    id: i64,
    status: &str,
) -> engram_core::Result<Epic> {
    let parsed: EpicStatus = parse(status)?;
    db.epic_update(id, UpdateEpicInput { status: Some(parsed), ..Default::default() }, "user").await
}

pub async fn do_epic_set_sprint(
    db: &Db,
    epic_id: i64,
    sprint_id: Option<i64>,
) -> engram_core::Result<Epic> {
    db.epic_set_sprint(epic_id, sprint_id, "user").await
}

// ── Tauri command wrappers ────────────────────────────────────────────────────

#[tauri::command(rename_all = "snake_case")]
pub async fn epic_list(
    db: State<'_, Arc<Db>>,
    project_key: Option<String>,
    include_completed: Option<bool>,
) -> Result<Vec<Epic>, String> {
    do_epic_list(&db, project_key.as_deref(), include_completed.unwrap_or(false)).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn epic_set_sprint(
    db: State<'_, Arc<Db>>,
    epic_id: i64,
    sprint_id: Option<i64>,
) -> Result<Epic, String> {
    do_epic_set_sprint(&db, epic_id, sprint_id).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn epic_create(
    db: State<'_, Arc<Db>>,
    project_key: String,
    mission_id: Option<i64>,
    sprint_id: Option<i64>,
    title: String,
    description: Option<String>,
) -> Result<Epic, String> {
    do_epic_create(&db, project_key, mission_id, sprint_id, title, description).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn epic_set_status(
    db: State<'_, Arc<Db>>,
    id: i64,
    status: String,
) -> Result<Epic, String> {
    do_epic_set_status(&db, id, &status).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn epic_update(
    db: State<'_, Arc<Db>>,
    id: i64,
    title: Option<String>,
    description: Option<String>,
    status: Option<String>,
    mission_id: Option<i64>,
    sprint_id: Option<i64>,
    update_sprint_id: Option<bool>,
) -> Result<Epic, String> {
    let status_parsed = if let Some(s) = status {
        Some(parse::<EpicStatus>(&s).map_err(|e| e.to_string())?)
    } else {
        None
    };
    db.epic_update(
        id,
        UpdateEpicInput {
            title,
            description,
            status: status_parsed,
            mission_id,
            sprint_id,
            update_sprint_id: update_sprint_id.unwrap_or(false),
        },
        "user",
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn epic_delete(
    db: State<'_, Arc<Db>>,
    id: i64,
) -> Result<(), String> {
    db.epic_delete(id, "user").await.map_err(|e| e.to_string())
}
