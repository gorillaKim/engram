use engram_core::{
    Db,
    models::{
        issue::Issue,
        retrospective::{
            CreateRetroActionItemInput, CreateRetrospectiveInput, RetroActionItem,
            Retrospective, RetrospectiveWithItems, UpdateRetroActionItemInput,
            UpdateRetrospectiveInput,
        },
    },
};
use std::sync::Arc;
use tauri::State;

// ── Inner functions (testable without Tauri context) ──────────────────────────

pub async fn do_retrospective_list(
    db: &Db,
    project_key: Option<String>,
    sprint_id: Option<i64>,
    limit: Option<u32>,
) -> engram_core::Result<Vec<RetrospectiveWithItems>> {
    let limit = limit.unwrap_or(50);
    let retros = db.retrospective_list(project_key.as_deref(), sprint_id, limit).await?;
    let mut result = Vec::with_capacity(retros.len());
    for r in retros {
        if let Ok(full) = db.retrospective_get(r.id).await {
            result.push(full);
        } else {
            result.push(RetrospectiveWithItems {
                retro: r,
                action_items: vec![],
            });
        }
    }
    Ok(result)
}

pub async fn do_retrospective_get(
    db: &Db,
    id: i64,
) -> engram_core::Result<RetrospectiveWithItems> {
    db.retrospective_get(id).await
}

pub async fn do_retrospective_create(
    db: &Db,
    input: CreateRetrospectiveInput,
) -> engram_core::Result<RetrospectiveWithItems> {
    db.retrospective_create(input).await
}

pub async fn do_retrospective_update(
    db: &Db,
    id: i64,
    input: UpdateRetrospectiveInput,
) -> engram_core::Result<Retrospective> {
    db.retrospective_update(id, input, Some("user")).await
}

pub async fn do_retrospective_delete(db: &Db, id: i64) -> engram_core::Result<()> {
    db.retrospective_delete(id).await
}

pub async fn do_retro_action_item_create(
    db: &Db,
    retro_id: i64,
    input: CreateRetroActionItemInput,
) -> engram_core::Result<RetroActionItem> {
    db.retro_action_item_create(retro_id, input).await
}

pub async fn do_retro_action_item_update(
    db: &Db,
    id: i64,
    input: UpdateRetroActionItemInput,
) -> engram_core::Result<RetroActionItem> {
    db.retro_action_item_update(id, input).await
}

pub async fn do_retro_action_item_delete(db: &Db, id: i64) -> engram_core::Result<()> {
    db.retro_action_item_delete(id).await
}

pub async fn do_retro_action_item_convert_to_issue(
    db: &Db,
    id: i64,
    agent_id: Option<String>,
) -> engram_core::Result<Issue> {
    db.retro_action_item_convert_to_issue(id, agent_id.as_deref()).await
}

// ── Tauri command wrappers ────────────────────────────────────────────────────

#[tauri::command(rename_all = "snake_case")]
pub async fn retrospective_list(
    db: State<'_, Arc<Db>>,
    project_key: Option<String>,
    sprint_id: Option<i64>,
    limit: Option<u32>,
) -> Result<Vec<RetrospectiveWithItems>, String> {
    do_retrospective_list(&db, project_key, sprint_id, limit)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn retrospective_get(
    db: State<'_, Arc<Db>>,
    id: i64,
) -> Result<RetrospectiveWithItems, String> {
    do_retrospective_get(&db, id).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn retrospective_create(
    db: State<'_, Arc<Db>>,
    input: CreateRetrospectiveInput,
) -> Result<RetrospectiveWithItems, String> {
    do_retrospective_create(&db, input).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn retrospective_update(
    db: State<'_, Arc<Db>>,
    id: i64,
    input: UpdateRetrospectiveInput,
) -> Result<Retrospective, String> {
    do_retrospective_update(&db, id, input).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn retrospective_delete(
    db: State<'_, Arc<Db>>,
    id: i64,
) -> Result<(), String> {
    do_retrospective_delete(&db, id).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn retro_action_item_create(
    db: State<'_, Arc<Db>>,
    retro_id: i64,
    input: CreateRetroActionItemInput,
) -> Result<RetroActionItem, String> {
    do_retro_action_item_create(&db, retro_id, input).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn retro_action_item_update(
    db: State<'_, Arc<Db>>,
    id: i64,
    input: UpdateRetroActionItemInput,
) -> Result<RetroActionItem, String> {
    do_retro_action_item_update(&db, id, input).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn retro_action_item_delete(
    db: State<'_, Arc<Db>>,
    id: i64,
) -> Result<(), String> {
    do_retro_action_item_delete(&db, id).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn retro_action_item_convert_to_issue(
    db: State<'_, Arc<Db>>,
    id: i64,
    agent_id: Option<String>,
) -> Result<Issue, String> {
    do_retro_action_item_convert_to_issue(&db, id, agent_id).await.map_err(|e| e.to_string())
}
