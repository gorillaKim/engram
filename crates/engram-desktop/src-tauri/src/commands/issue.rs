use crate::commands::parse;
use engram_core::{
    Db,
    models::issue::{
        Issue, IssueFilter, IssueLink, IssueStatus, IssuePriority,
        CreateIssueInput, LinkType, UpdateIssueInput,
    },
    repository::blocking::BlockingGraph,
};
use std::sync::Arc;
use tauri::State;

// ── Inner functions (testable without Tauri context) ──────────────────────────

pub async fn do_issue_list(
    db: &Db,
    filter: IssueFilter,
) -> engram_core::Result<Vec<Issue>> {
    let res = db.issue_list(filter).await?;
    Ok(res.items)
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

pub async fn do_issue_create(
    db: &Db,
    epic_id: i64,
    title: String,
    description: Option<String>,
    goal: Option<String>,
    priority: Option<String>,
) -> engram_core::Result<Issue> {
    let priority_parsed = if let Some(p) = priority {
        Some(parse::<IssuePriority>(&p)?)
    } else {
        None
    };
    db.issue_create(CreateIssueInput {
        epic_id,
        title,
        description,
        goal,
        priority: priority_parsed,
    })
    .await
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

// ── Tauri command wrappers ────────────────────────────────────────────────────

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
pub async fn issue_delete(
    db: State<'_, Arc<Db>>,
    id: i64,
) -> Result<(), String> {
    db.issue_delete(id, "user").await.map_err(|e| e.to_string())
}
