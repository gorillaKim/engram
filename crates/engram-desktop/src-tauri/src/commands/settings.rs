use crate::settings as crate_settings;
use crate::mcp_supervisor::McpSupervisor;
use std::sync::Arc;
use tauri::State;

#[tauri::command(rename_all = "snake_case")]
pub async fn mcp_set_autostart(
    sup: State<'_, Arc<McpSupervisor>>,
    on: bool,
) -> Result<(), String> {
    sup.set_autostart(on);
    tokio::task::spawn_blocking(move || crate_settings::set_autostart(on))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_activity_settings() -> Result<crate_settings::ActivitySettings, String> {
    Ok(crate_settings::load().unwrap_or_default().activity)
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_activity_settings(
    warn_minutes: i64,
    stall_minutes: i64,
) -> Result<(), String> {
    let mut s = crate_settings::load().unwrap_or_default();
    s.activity.warn_minutes = warn_minutes;
    s.activity.stall_minutes = stall_minutes;
    crate_settings::save(&s).map_err(|e| e.to_string())
}
