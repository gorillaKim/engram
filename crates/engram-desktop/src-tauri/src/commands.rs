use crate::mcp_supervisor::{McpSupervisor, SupervisorStatusSnapshot};
use crate::settings;
use engram_core::{
    Db,
    models::{
        epic::{Epic, CreateEpicInput, EpicStatus, UpdateEpicInput},
        history::{History, EntityType},
        issue::{
            Issue, IssueFilter, IssueLink, IssueStatus, IssuePriority,
            CreateIssueInput, LinkType, UpdateIssueInput,
        },
        mission::{
            Mission, CreateMissionInput, UpdateMissionInput, MissionFilter,
            MissionStatus, MissionProgress, MissionTree,
        },
        note::{Note, CreateNoteInput},
        sprint::{Sprint, CreateSprintInput},
        task::{Task, CreateTaskInput, TaskSource, TaskStatus, UpdateTaskInput},
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

pub async fn do_issue_list(
    db: &Db,
    filter: IssueFilter,
) -> engram_core::Result<Vec<Issue>> {
    db.issue_list(filter).await
}

pub async fn do_issue_get(db: &Db, id: i64) -> engram_core::Result<Issue> {
    db.issue_get(id, false).await
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

pub async fn do_issue_update(
    db: &Db,
    id: i64,
    title: Option<String>,
    description: Option<String>,
    goal: Option<String>,
    epic_id: Option<i64>,
) -> engram_core::Result<Issue> {
    db.issue_update(
        id,
        UpdateIssueInput {
            title,
            description,
            goal,
            epic_id,
            ..Default::default()
        },
        "user",
    )
    .await
}

pub async fn do_epic_list(
    db: &Db,
    project_key: Option<&str>,
    include_completed: bool,
) -> engram_core::Result<Vec<Epic>> {
    db.epic_list(project_key, include_completed).await
}

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
    db.note_list(Some(issue_id), None, None, false, true).await
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

pub async fn do_blocked_issues_graph(
    db: &Db,
    project_key: &str,
) -> engram_core::Result<BlockingGraph> {
    db.blocked_issues_graph(project_key).await
}

pub async fn do_blocking_graph_for_issue(
    db: &Db,
    issue_id: i64,
) -> engram_core::Result<BlockingGraph> {
    db.blocking_graph_for_issue(issue_id).await
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

pub async fn do_issue_create(
    db: &Db,
    epic_id: i64,
    title: String,
    description: Option<String>,
    goal: Option<String>,
    priority: Option<String>,
) -> engram_core::Result<Issue> {
    let parsed_priority = match priority {
        Some(p) => Some(parse::<IssuePriority>(&p)?),
        None => None,
    };
    db.issue_create(CreateIssueInput {
        epic_id, title, description, goal, priority: parsed_priority,
    }).await
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

pub async fn do_issue_link(
    db: &Db,
    source_id: i64,
    target_id: i64,
    link_type: &str,
) -> engram_core::Result<IssueLink> {
    let parsed: LinkType = parse(link_type)?;
    db.issue_link(source_id, target_id, parsed).await
}

pub async fn do_issue_unlink(db: &Db, link_id: i64) -> engram_core::Result<()> {
    db.issue_unlink(link_id).await
}

pub async fn do_issue_links(db: &Db, issue_id: i64) -> engram_core::Result<Vec<IssueLink>> {
    db.issue_links_for(issue_id).await
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

pub async fn do_history_list(
    db: &Db,
    entity_type: &str,
    entity_id: i64,
) -> engram_core::Result<Vec<History>> {
    let parsed: EntityType = parse(entity_type)?;
    db.history_list(parsed, entity_id).await
}

pub async fn do_mission_list(
    db: &Db,
    include_completed: Option<bool>,
) -> engram_core::Result<Vec<Mission>> {
    let filter = MissionFilter {
        status: None,
        include_completed: include_completed.unwrap_or(false),
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

#[tauri::command(rename_all = "snake_case")]
pub async fn issue_list(
    db: State<'_, Arc<Db>>,
    filter: IssueFilter,
) -> Result<Vec<Issue>, String> {
    do_issue_list(&db, filter).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn issue_get(db: State<'_, Arc<Db>>, id: i64) -> Result<Issue, String> {
    do_issue_get(&db, id).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn issue_set_status(
    db: State<'_, Arc<Db>>,
    id: i64,
    status: String,
) -> Result<Issue, String> {
    do_issue_set_status(&db, id, &status).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn issue_set_priority(
    db: State<'_, Arc<Db>>,
    id: i64,
    priority: String,
) -> Result<Issue, String> {
    do_issue_set_priority(&db, id, &priority).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn issue_update(
    db: State<'_, Arc<Db>>,
    id: i64,
    title: Option<String>,
    description: Option<String>,
    goal: Option<String>,
    epic_id: Option<i64>,
) -> Result<Issue, String> {
    do_issue_update(&db, id, title, description, goal, epic_id)
        .await
        .map_err(|e| e.to_string())
}

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
    use engram_core::models::sprint::UpdateSprintInput;
    let status_parsed = if let Some(s) = status {
        Some(parse::<engram_core::models::sprint::SprintStatus>(&s).map_err(|e| e.to_string())?)
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

#[tauri::command(rename_all = "snake_case")]
pub async fn blocked_issues_graph(
    db: State<'_, Arc<Db>>,
    project_key: String,
) -> Result<BlockingGraph, String> {
    do_blocked_issues_graph(&db, &project_key).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn blocking_graph_for_issue(
    db: State<'_, Arc<Db>>,
    issue_id: i64,
) -> Result<BlockingGraph, String> {
    do_blocking_graph_for_issue(&db, issue_id).await.map_err(|e| e.to_string())
}

// ── Dashboard CRUD ────────────────────────────────────────────────────────────

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
pub async fn issue_create(
    db: State<'_, Arc<Db>>,
    epic_id: i64,
    title: String,
    description: Option<String>,
    goal: Option<String>,
    priority: Option<String>,
) -> Result<Issue, String> {
    do_issue_create(&db, epic_id, title, description, goal, priority).await.map_err(|e| e.to_string())
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

#[tauri::command(rename_all = "snake_case")]
pub async fn issue_link(
    db: State<'_, Arc<Db>>,
    source_id: i64,
    target_id: i64,
    link_type: String,
) -> Result<IssueLink, String> {
    do_issue_link(&db, source_id, target_id, &link_type).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn issue_unlink(
    db: State<'_, Arc<Db>>,
    link_id: i64,
) -> Result<(), String> {
    do_issue_unlink(&db, link_id).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn issue_links(
    db: State<'_, Arc<Db>>,
    issue_id: i64,
) -> Result<Vec<IssueLink>, String> {
    do_issue_links(&db, issue_id).await.map_err(|e| e.to_string())
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

#[tauri::command(rename_all = "snake_case")]
pub async fn issue_delete(
    db: State<'_, Arc<Db>>,
    id: i64,
) -> Result<(), String> {
    db.issue_delete(id, "user").await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn history_list(
    db: State<'_, Arc<Db>>,
    entity_type: String,
    entity_id: i64,
) -> Result<Vec<History>, String> {
    do_history_list(&db, &entity_type, entity_id).await.map_err(|e| e.to_string())
}

// ── Mission IPC ───────────────────────────────────────────────────────────────

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


// ── App info / lifecycle ──────────────────────────────────────────────────────

#[tauri::command(rename_all = "snake_case")]
pub fn get_app_version(app: tauri::AppHandle) -> String {
    app.package_info().version.to_string()
}

#[tauri::command(rename_all = "snake_case")]
pub fn relaunch_app(app: tauri::AppHandle) {
    tauri::process::restart(&app.env());
}

// ── MCP Supervisor commands ───────────────────────────────────────────────────

#[tauri::command(rename_all = "snake_case")]
pub async fn mcp_status(
    sup: State<'_, Arc<McpSupervisor>>,
) -> Result<SupervisorStatusSnapshot, String> {
    Ok(sup.status().await)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn mcp_start(
    sup: State<'_, Arc<McpSupervisor>>,
    port: u16,
) -> Result<SupervisorStatusSnapshot, String> {
    if port < 1024 {
        return Err("Port must be 1024 or higher".to_string());
    }
    sup.start(port).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn mcp_stop(
    sup: State<'_, Arc<McpSupervisor>>,
) -> Result<SupervisorStatusSnapshot, String> {
    sup.stop().await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn mcp_restart(
    sup: State<'_, Arc<McpSupervisor>>,
    port: u16,
) -> Result<SupervisorStatusSnapshot, String> {
    if port < 1024 {
        return Err("Port must be 1024 or higher".to_string());
    }
    sup.restart(port).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn mcp_recent_calls(
    sup: State<'_, Arc<McpSupervisor>>,
) -> Result<Vec<CallRecord>, String> {
    Ok(sup.recent_calls().await)
}

#[tauri::command(rename_all = "snake_case")]
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

#[tauri::command(rename_all = "snake_case")]
pub fn get_activity_settings() -> Result<crate::settings::ActivitySettings, String> {
    Ok(settings::load().unwrap_or_default().activity)
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_activity_settings(
    warn_minutes: i64,
    stall_minutes: i64,
) -> Result<(), String> {
    let mut s = settings::load().unwrap_or_default();
    s.activity.warn_minutes = warn_minutes;
    s.activity.stall_minutes = stall_minutes;
    settings::save(&s).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn hide_tray_popover(app: tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("tray_popover") {
        let _ = w.hide();
    }
}

#[tauri::command(rename_all = "snake_case")]
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
        
        let mission = db.mission_create(engram_core::models::mission::CreateMissionInput {
            title: "M1".to_string(),
            description: None,
            jira_key: None,
        }).await.unwrap();

        let epic = db.epic_create(CreateEpicInput {
            project_key: "proj".to_string(),
            mission_id: Some(mission.id),
            sprint_id: Some(sprint.id),
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
    async fn test_issue_set_status_any_transition_allowed() {
        // 사용자가 칸반에서 카드를 양방향으로 자유롭게 옮길 수 있어야 한다 —
        // can_transition_to 는 항상 true. 우회 가드는 워커 에이전트 프롬프트 책임.
        let db = setup().await;
        let (_, issue_id) = seed_issue(&db).await;
        let updated = do_issue_set_status(&db, issue_id, "working").await.unwrap();
        assert_eq!(updated.status, IssueStatus::Working);

        // 역방향도 OK
        let reverted = do_issue_set_status(&db, issue_id, "required").await.unwrap();
        assert_eq!(reverted.status, IssueStatus::Required);
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
            agent_id: None,
            scope: None, scope_target_id: None, project_key: None,
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

    #[tokio::test]
    async fn test_issue_update_title_description_goal() {
        let db = setup().await;
        let (_, issue_id) = seed_issue(&db).await;

        let updated = do_issue_update(
            &db, issue_id,
            Some("Updated Title".to_string()),
            Some("Updated desc".to_string()),
            Some("New goal".to_string()),
            None,
        ).await.unwrap();

        assert_eq!(updated.title, "Updated Title");
        assert_eq!(updated.description, Some("Updated desc".to_string()));
        assert_eq!(updated.goal, Some("New goal".to_string()));
    }

    #[tokio::test]
    async fn test_issue_update_partial_only_description() {
        let db = setup().await;
        let (_, issue_id) = seed_issue(&db).await;

        // None 필드는 기존 값 유지
        let updated = do_issue_update(&db, issue_id, None, Some("desc only".to_string()), None, None).await.unwrap();
        assert_eq!(updated.title, "I1", "title unchanged");
        assert_eq!(updated.description, Some("desc only".to_string()));
        assert_eq!(updated.goal, None, "goal unchanged");
    }

    #[tokio::test]
    async fn test_issue_update_clear_description_with_empty_string() {
        let db = setup().await;
        let (_, issue_id) = seed_issue(&db).await;

        // 설명 설정 후 빈 문자열로 지우기
        do_issue_update(&db, issue_id, None, Some("initial desc".to_string()), None, None).await.unwrap();
        let cleared = do_issue_update(&db, issue_id, None, Some("".to_string()), None, None).await.unwrap();
        // 빈 문자열은 None 으로 저장
        assert!(cleared.description.as_deref().unwrap_or("").is_empty());
    }
}
