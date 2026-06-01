use crate::commands::parse;
use engram_core::{
    Db,
    models::mission::{
        Mission, CreateMissionInput, UpdateMissionInput, MissionFilter,
        MissionStatus, MissionProgress, MissionTree,
    },
};
use std::sync::Arc;
use tauri::State;

// ── Inner functions (testable without Tauri context) ──────────────────────────

pub async fn do_mission_list(
    db: &Db,
    include_completed: Option<bool>,
) -> engram_core::Result<Vec<Mission>> {
    let filter = MissionFilter {
        status: None,
        include_completed: include_completed.unwrap_or(false),
        project_key: None,
        sprint_id: None,
    };
    db.mission_list(filter).await
}

pub async fn do_mission_create(
    db: &Db,
    title: String,
    description: Option<String>,
    jira_key: Option<String>,
) -> engram_core::Result<Mission> {
    db.mission_create(CreateMissionInput { title, description, jira_key }).await
}

pub async fn do_mission_get(db: &Db, id: i64) -> engram_core::Result<Mission> {
    db.mission_get(id).await
}

pub async fn do_mission_update(
    db: &Db,
    id: i64,
    title: Option<String>,
    description: Option<String>,
    jira_key: Option<String>,
    status: Option<MissionStatus>,
) -> engram_core::Result<Mission> {
    let input = UpdateMissionInput { title, description, jira_key, status };
    db.mission_update(id, input, "user").await
}

pub async fn do_mission_delete(db: &Db, id: i64) -> engram_core::Result<()> {
    db.mission_delete(id).await
}

pub async fn do_mission_get_progress(db: &Db, id: i64) -> engram_core::Result<MissionProgress> {
    db.mission_progress_query(id).await
}

pub async fn do_mission_get_tree(db: &Db, id: i64) -> engram_core::Result<MissionTree> {
    db.mission_get_tree(id).await
}

// ── Tauri command wrappers ────────────────────────────────────────────────────

#[tauri::command(rename_all = "snake_case")]
pub async fn mission_list(
    db: State<'_, Arc<Db>>,
    include_completed: Option<bool>,
) -> Result<Vec<Mission>, String> {
    do_mission_list(&db, include_completed).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn mission_create(
    db: State<'_, Arc<Db>>,
    title: String,
    description: Option<String>,
    jira_key: Option<String>,
) -> Result<Mission, String> {
    do_mission_create(&db, title, description, jira_key).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn mission_get(
    db: State<'_, Arc<Db>>,
    id: i64,
) -> Result<Mission, String> {
    do_mission_get(&db, id).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn mission_update(
    db: State<'_, Arc<Db>>,
    id: i64,
    title: Option<String>,
    description: Option<String>,
    jira_key: Option<String>,
    status: Option<String>,
) -> Result<Mission, String> {
    let status_parsed = if let Some(s) = status {
        Some(parse::<MissionStatus>(&s).map_err(|e| e.to_string())?)
    } else {
        None
    };
    do_mission_update(&db, id, title, description, jira_key, status_parsed)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn mission_delete(
    db: State<'_, Arc<Db>>,
    id: i64,
) -> Result<(), String> {
    do_mission_delete(&db, id).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn mission_get_progress(
    db: State<'_, Arc<Db>>,
    id: i64,
) -> Result<MissionProgress, String> {
    do_mission_get_progress(&db, id).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn mission_get_tree(
    db: State<'_, Arc<Db>>,
    id: i64,
) -> Result<MissionTree, String> {
    do_mission_get_tree(&db, id).await.map_err(|e| e.to_string())
}
