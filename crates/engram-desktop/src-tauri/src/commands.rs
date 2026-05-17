use crate::mcp_supervisor::{McpSupervisor, SupervisorStatusSnapshot};
use crate::settings;
use engram_core::{
    Db,
    models::{
        epic::Epic,
        issue::{Issue, IssueFilter, IssueStatus, IssuePriority, UpdateIssueInput},
        note::{Note, CreateNoteInput},
        sprint::Sprint,
        task::{Task, TaskStatus, UpdateTaskInput},
    },
    repository::session::{SessionSnapshot, IssueBoardStatus},
    repository::blocking::BlockingGraph,
};
use engram_mcp::http::CallRecord;
use std::sync::Arc;
use tauri::{Manager, State};

// ── Parse helpers ─────────────────────────────────────────────────────────────

fn parse<T: serde::de::DeserializeOwned>(s: &str) -> engram_core::Result<T> {
    serde_json::from_value(serde_json::Value::String(s.to_string()))
        .map_err(|_| engram_core::Error::Validation(format!("unknown value: {s}")))
}

// ── Inner functions (testable without Tauri context) ──────────────────────────

pub async fn do_session_restore(
    db: &Db,
    project_key: Option<&str>,
) -> engram_core::Result<SessionSnapshot> {
    db.session_restore(project_key).await
}

pub async fn do_board_status(
    db: &Db,
    project_key: Option<&str>,
) -> engram_core::Result<IssueBoardStatus> {
    db.board_issues_query(project_key).await
}

pub async fn do_issue_list(
    db: &Db,
    filter: IssueFilter,
) -> engram_core::Result<Vec<Issue>> {
    db.issue_list(filter).await
}

pub async fn do_issue_get(db: &Db, id: i64) -> engram_core::Result<Issue> {
    db.issue_get(id).await
}

pub async fn do_issue_set_status(
    db: &Db,
    id: i64,
    status: &str,
) -> engram_core::Result<Issue> {
    let parsed: IssueStatus = parse(status)?;
    db.issue_update(id, UpdateIssueInput { status: Some(parsed), ..Default::default() }, "user").await
}

pub async fn do_issue_set_priority(
    db: &Db,
    id: i64,
    priority: &str,
) -> engram_core::Result<Issue> {
    let parsed: IssuePriority = parse(priority)?;
    db.issue_update(id, UpdateIssueInput { priority: Some(parsed), ..Default::default() }, "user").await
}

pub async fn do_epic_list(
    db: &Db,
    project_key: Option<&str>,
) -> engram_core::Result<Vec<Epic>> {
    db.epic_list(None, project_key, None).await
}

pub async fn do_sprint_current(db: &Db) -> engram_core::Result<Option<Sprint>> {
    db.sprint_current().await
}

pub async fn do_task_list(db: &Db, issue_id: i64) -> engram_core::Result<Vec<Task>> {
    db.task_list(issue_id, None).await
}

pub async fn do_task_set_status(
    db: &Db,
    id: i64,
    status: &str,
) -> engram_core::Result<Task> {
    let parsed: TaskStatus = parse(status)?;
    db.task_update(id, UpdateTaskInput { status: Some(parsed), ..Default::default() }, "user").await
}

pub async fn do_note_list(db: &Db, issue_id: i64) -> engram_core::Result<Vec<Note>> {
    db.note_list(Some(issue_id), None, None, false).await
}

pub async fn do_note_get(db: &Db, id: i64) -> engram_core::Result<Note> {
    db.note_get(id).await
}

pub async fn do_note_add(db: &Db, input: CreateNoteInput) -> engram_core::Result<Note> {
    db.note_add(input).await
}

pub async fn do_note_resolve(db: &Db, id: i64) -> engram_core::Result<Note> {
    db.note_resolve(id, "user").await
}

pub async fn do_blocked_issues_graph(
    db: &Db,
    project_key: &str,
) -> engram_core::Result<BlockingGraph> {
    db.blocked_issues_graph(project_key).await
}

// ── Tauri command wrappers ────────────────────────────────────────────────────

#[tauri::command]
pub async fn session_restore(
    db: State<'_, Arc<Db>>,
    project_key: Option<String>,
) -> Result<SessionSnapshot, String> {
    do_session_restore(&db, project_key.as_deref()).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn board_status(
    db: State<'_, Arc<Db>>,
    project_key: Option<String>,
) -> Result<IssueBoardStatus, String> {
    do_board_status(&db, project_key.as_deref()).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn issue_list(
    db: State<'_, Arc<Db>>,
    filter: IssueFilter,
) -> Result<Vec<Issue>, String> {
    do_issue_list(&db, filter).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn issue_get(db: State<'_, Arc<Db>>, id: i64) -> Result<Issue, String> {
    do_issue_get(&db, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn issue_set_status(
    db: State<'_, Arc<Db>>,
    id: i64,
    status: String,
) -> Result<Issue, String> {
    do_issue_set_status(&db, id, &status).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn issue_set_priority(
    db: State<'_, Arc<Db>>,
    id: i64,
    priority: String,
) -> Result<Issue, String> {
    do_issue_set_priority(&db, id, &priority).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn epic_list(
    db: State<'_, Arc<Db>>,
    project_key: Option<String>,
) -> Result<Vec<Epic>, String> {
    do_epic_list(&db, project_key.as_deref()).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sprint_current(db: State<'_, Arc<Db>>) -> Result<Option<Sprint>, String> {
    do_sprint_current(&db).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn task_list(db: State<'_, Arc<Db>>, issue_id: i64) -> Result<Vec<Task>, String> {
    do_task_list(&db, issue_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn task_set_status(
    db: State<'_, Arc<Db>>,
    id: i64,
    status: String,
) -> Result<Task, String> {
    do_task_set_status(&db, id, &status).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn note_list(db: State<'_, Arc<Db>>, issue_id: i64) -> Result<Vec<Note>, String> {
    do_note_list(&db, issue_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn note_get(db: State<'_, Arc<Db>>, id: i64) -> Result<Note, String> {
    do_note_get(&db, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn note_add(
    db: State<'_, Arc<Db>>,
    input: CreateNoteInput,
) -> Result<Note, String> {
    do_note_add(&db, input).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn note_resolve(db: State<'_, Arc<Db>>, id: i64) -> Result<Note, String> {
    do_note_resolve(&db, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn blocked_issues_graph(
    db: State<'_, Arc<Db>>,
    project_key: String,
) -> Result<BlockingGraph, String> {
    do_blocked_issues_graph(&db, &project_key).await.map_err(|e| e.to_string())
}

// ── MCP Supervisor commands ───────────────────────────────────────────────────

#[tauri::command]
pub async fn mcp_status(
    sup: State<'_, Arc<McpSupervisor>>,
) -> Result<SupervisorStatusSnapshot, String> {
    Ok(sup.status().await)
}

#[tauri::command]
pub async fn mcp_start(
    sup: State<'_, Arc<McpSupervisor>>,
    port: u16,
) -> Result<SupervisorStatusSnapshot, String> {
    if port < 1024 {
        return Err("Port must be 1024 or higher".to_string());
    }
    sup.start(port).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mcp_stop(
    sup: State<'_, Arc<McpSupervisor>>,
) -> Result<SupervisorStatusSnapshot, String> {
    sup.stop().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mcp_restart(
    sup: State<'_, Arc<McpSupervisor>>,
    port: u16,
) -> Result<SupervisorStatusSnapshot, String> {
    if port < 1024 {
        return Err("Port must be 1024 or higher".to_string());
    }
    sup.restart(port).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mcp_recent_calls(
    sup: State<'_, Arc<McpSupervisor>>,
) -> Result<Vec<CallRecord>, String> {
    Ok(sup.recent_calls().await)
}

#[tauri::command]
pub async fn mcp_set_autostart(
    sup: State<'_, Arc<McpSupervisor>>,
    on: bool,
) -> Result<(), String> {
    sup.set_autostart(on);
    tokio::task::spawn_blocking(move || settings::set_autostart(on))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn hide_tray_popover(app: tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("tray_popover") {
        let _ = w.hide();
    }
}

#[tauri::command]
pub fn show_main_window(app: tauri::AppHandle) {
    // macOS: activate the app so it comes to front
    #[cfg(target_os = "macos")]
    let _ = app.show();

    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
    if let Some(popover) = app.get_webview_window("tray_popover") {
        let _ = popover.hide();
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use engram_core::{
        Db,
        models::{
            sprint::{CreateSprintInput, UpdateSprintInput, SprintStatus},
            epic::CreateEpicInput,
            issue::CreateIssueInput,
            task::CreateTaskInput,
            note::{CreateNoteInput, NoteType},
        },
    };

    async fn setup() -> Db {
        Db::open_in_memory().await.unwrap()
    }

    async fn seed_issue(db: &Db) -> (i64, i64) {
        let sprint = db.sprint_create(CreateSprintInput {
            name: "S1".to_string(), goal: None, start_date: None, end_date: None,
        }).await.unwrap();
        db.sprint_update(sprint.id, UpdateSprintInput {
            status: Some(SprintStatus::Active), ..Default::default()
        }, "agent").await.unwrap();
        let epic = db.epic_create(CreateEpicInput {
            sprint_id: sprint.id, project_key: "proj".to_string(),
            title: "E1".to_string(), description: None,
        }).await.unwrap();
        let issue = db.issue_create(CreateIssueInput {
            epic_id: epic.id, title: "I1".to_string(),
            description: None, goal: None, priority: None,
        }).await.unwrap();
        (epic.id, issue.id)
    }

    #[tokio::test]
    async fn test_session_restore_command_returns_snapshot() {
        let db = setup().await;
        assert!(do_session_restore(&db, None).await.is_ok());
    }

    #[tokio::test]
    async fn test_board_status_command_returns_board() {
        let db = setup().await;
        assert!(do_board_status(&db, None).await.is_ok());
    }

    #[tokio::test]
    async fn test_issue_list_command_returns_vec() {
        let db = setup().await;
        let (epic_id, _) = seed_issue(&db).await;
        let issues = do_issue_list(&db, IssueFilter { epic_id: Some(epic_id), ..Default::default() }).await.unwrap();
        assert_eq!(issues.len(), 1);
    }

    #[tokio::test]
    async fn test_sprint_current_returns_active_sprint() {
        let db = setup().await;
        assert!(do_sprint_current(&db).await.unwrap().is_none());
        let sprint = db.sprint_create(CreateSprintInput {
            name: "Active".to_string(), goal: None, start_date: None, end_date: None,
        }).await.unwrap();
        db.sprint_update(sprint.id, UpdateSprintInput {
            status: Some(SprintStatus::Active), ..Default::default()
        }, "agent").await.unwrap();
        assert_eq!(do_sprint_current(&db).await.unwrap().unwrap().id, sprint.id);
    }

    #[tokio::test]
    async fn test_issue_set_status_valid_transition() {
        let db = setup().await;
        let (_, issue_id) = seed_issue(&db).await;
        let updated = do_issue_set_status(&db, issue_id, "ready").await.unwrap();
        assert_eq!(updated.status, IssueStatus::Ready);
    }

    #[tokio::test]
    async fn test_issue_set_status_invalid_transition_returns_err() {
        let db = setup().await;
        let (_, issue_id) = seed_issue(&db).await;
        // required → working is not allowed
        let err = do_issue_set_status(&db, issue_id, "working").await.unwrap_err();
        assert!(err.to_string().contains("transition") || err.to_string().contains("Invalid"),
            "expected InvalidTransition error, got: {err}");
    }

    #[tokio::test]
    async fn test_issue_set_status_unknown_value_returns_err() {
        let db = setup().await;
        let (_, issue_id) = seed_issue(&db).await;
        assert!(do_issue_set_status(&db, issue_id, "bogus").await.is_err());
    }

    #[tokio::test]
    async fn test_task_list_and_set_status() {
        let db = setup().await;
        let (_, issue_id) = seed_issue(&db).await;
        db.task_create(CreateTaskInput {
            issue_id, title: "T1".to_string(),
            description: None, goal: None, after_task_id: None, source: None,
        }).await.unwrap();
        let tasks = do_task_list(&db, issue_id).await.unwrap();
        assert_eq!(tasks.len(), 1);

        let updated = do_task_set_status(&db, tasks[0].id, "finished").await.unwrap();
        assert_eq!(updated.status, TaskStatus::Finished);
    }

    #[tokio::test]
    async fn test_note_add_list_get_resolve() {
        let db = setup().await;
        let (_, issue_id) = seed_issue(&db).await;

        let note = do_note_add(&db, CreateNoteInput {
            issue_id, task_id: None,
            note_type: NoteType::Context,
            summary: "context note".to_string(),
            detail: Some("detail text".to_string()),
            author: None,
        }).await.unwrap();
        assert_eq!(note.summary, "context note");

        let notes = do_note_list(&db, issue_id).await.unwrap();
        assert_eq!(notes.len(), 1);

        let fetched = do_note_get(&db, note.id).await.unwrap();
        assert_eq!(fetched.detail, Some("detail text".to_string()));

        let resolved = do_note_resolve(&db, note.id).await.unwrap();
        assert!(resolved.resolved);
    }

    #[tokio::test]
    async fn test_blocked_issues_graph_empty() {
        let db = setup().await;
        let graph = do_blocked_issues_graph(&db, "proj").await.unwrap();
        assert!(graph.chains.is_empty());
        assert!(!graph.has_cycle);
    }
}
