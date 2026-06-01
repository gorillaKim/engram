use engram_core::{
    Db,
    repository::session::{SessionSnapshot, IssueBoardStatus},
};
use std::sync::Arc;
use tauri::State;

// ── Inner functions (testable without Tauri context) ──────────────────────────

pub async fn do_session_restore(
    db: &Db,
    project_key: Option<&str>,
) -> engram_core::Result<SessionSnapshot> {
    let stall = crate::settings::load().unwrap_or_default().activity.stall_minutes;
    db.session_restore(project_key, false, stall, None).await
}

pub async fn do_board_status(
    db: &Db,
    project_key: Option<&str>,
) -> engram_core::Result<IssueBoardStatus> {
    let stall = crate::settings::load().unwrap_or_default().activity.stall_minutes;
    db.board_issues_query(project_key, stall).await
}

// ── Tauri command wrappers ────────────────────────────────────────────────────

#[tauri::command(rename_all = "snake_case")]
pub async fn session_restore(
    db: State<'_, Arc<Db>>,
    project_key: Option<String>,
) -> Result<SessionSnapshot, String> {
    do_session_restore(&db, project_key.as_deref()).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn board_status(
    db: State<'_, Arc<Db>>,
    project_key: Option<String>,
) -> Result<IssueBoardStatus, String> {
    do_board_status(&db, project_key.as_deref()).await.map_err(|e| e.to_string())
}
