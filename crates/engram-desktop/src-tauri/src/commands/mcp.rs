use crate::commands::parse;
use crate::mcp_supervisor::{McpSupervisor, SupervisorStatusSnapshot};
use engram_core::Db;
use engram_core::models::history::{History, EntityType};
use engram_mcp::http::CallRecord;
use std::sync::Arc;
use tauri::{Manager, State};

// ── Inner functions (testable without Tauri context) ──────────────────────────

pub async fn do_history_list(
    db: &Db,
    entity_type: &str,
    entity_id: i64,
) -> engram_core::Result<Vec<History>> {
    let parsed: EntityType = parse(entity_type)?;
    db.history_list(parsed, entity_id).await
}

// ── Tauri command wrappers ────────────────────────────────────────────────────

#[tauri::command(rename_all = "snake_case")]
pub async fn history_list(
    db: State<'_, Arc<Db>>,
    entity_type: String,
    entity_id: i64,
) -> Result<Vec<History>, String> {
    do_history_list(&db, &entity_type, entity_id).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_app_version(app: tauri::AppHandle) -> String {
    app.package_info().version.to_string()
}

#[tauri::command(rename_all = "snake_case")]
pub fn relaunch_app(app: tauri::AppHandle) {
    tauri::process::restart(&app.env());
}

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

#[tauri::command(rename_all = "snake_case")]
pub fn mcp_get_tool_definitions() -> Vec<serde_json::Value> {
    engram_mcp::tools::all_tool_definitions()
}

#[tauri::command(rename_all = "snake_case")]
pub async fn mcp_recent_logs(
    sup: State<'_, Arc<McpSupervisor>>,
) -> Result<Vec<crate::mcp_supervisor::LogLine>, String> {
    Ok(sup.recent_logs().await)
}
