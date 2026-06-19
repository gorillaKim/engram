#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod mcp_supervisor;
mod settings;
mod tracing_layer;
mod tray;
mod watcher;

use crate::mcp_supervisor::McpSupervisor;
use crate::tracing_layer::BroadcastLayer;
use engram_core::Db;
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tracing_subscriber::prelude::*;

fn main() {
    let settings = settings::load().unwrap_or_default();

    let db = tauri::async_runtime::block_on(async {
        Arc::new(Db::open_default().await.expect("DB open failed"))
    });
    let supervisor = McpSupervisor::new(Arc::clone(&db), settings.mcp.autostart);

    // Layered tracing: fmt stderr + env-filter + broadcast for McpManager UI
    let log_tx = supervisor.log_sender();
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(BroadcastLayer { tx: log_tx })
        .init();

    // Autostart MCP server
    if settings.mcp.autostart {
        let port = settings.mcp.port;
        let s = Arc::clone(&supervisor);
        tauri::async_runtime::block_on(async move {
            if let Err(e) = s.start(port).await {
                tracing::warn!("MCP autostart failed: {e}");
            }
        });
    }

    let supervisor_for_setup = Arc::clone(&supervisor);

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(move |app| {
            app.manage(db);
            app.manage(supervisor_for_setup);

            // Build tray icon + menu
            tray::build(app.handle())?;

            // Spawn board watcher (emits tray://summary, sends notifications)
            {
                let db_for_watcher = app.state::<Arc<Db>>().inner().clone();
                let ah_watcher = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    watcher::run(ah_watcher, db_for_watcher).await;
                });
            }

            let ah = app.handle().clone();
            let sup = app.state::<Arc<McpSupervisor>>().inner().clone();

            // Heartbeat: emit mcp://status every 10s
            {
                let ah2 = ah.clone();
                let s2 = Arc::clone(&sup);
                tauri::async_runtime::spawn(async move {
                    let mut interval = tokio::time::interval(
                        std::time::Duration::from_secs(10),
                    );
                    loop {
                        interval.tick().await;
                        let snap = s2.status().await;
                        let _ = ah2.emit("mcp://status", &snap);
                    }
                });
            }

            // Log pump: broadcast::Receiver → mcp://log event
            {
                let ah_log = ah.clone();
                let mut log_rx = sup.subscribe_logs();
                tauri::async_runtime::spawn(async move {
                    while let Ok(line) = log_rx.recv().await {
                        let _ = ah_log.emit("mcp://log", &line);
                    }
                });
            }

            // Call pump: broadcast::Receiver<CallRecord> → mcp://call event
            {
                let ah_call = ah.clone();
                let mut call_rx = sup.call_broadcast_sender().subscribe();
                tauri::async_runtime::spawn(async move {
                    while let Ok(rec) = call_rx.recv().await {
                        let _ = ah_call.emit("mcp://call", &rec);
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::session_restore,
            commands::board_status,
            commands::issue_list,
            commands::issue_get,
            commands::issue_set_status,
            commands::issue_set_priority,
            commands::epic_list,
            commands::sprint_current,
            commands::sprint_list,
            commands::sprint_create,
            commands::sprint_update,
            commands::sprint_delete,
            commands::epic_set_sprint,
            commands::task_list,
            commands::task_set_status,
            commands::note_list,
            commands::note_get,
            commands::note_add,
            commands::note_resolve,
            commands::blocked_issues_graph,
            commands::blocking_graph_for_issue,
            commands::epic_create,
            commands::issue_create,
            commands::task_create,
            commands::task_delete,
            commands::issue_link,
            commands::issue_unlink,
            commands::issue_links,
            commands::epic_set_status,
            commands::issue_update,
            commands::epic_update,
            commands::epic_delete,
            commands::issue_delete,
            commands::history_list,
            commands::mcp_status,
            commands::mcp_start,
            commands::mcp_stop,
            commands::mcp_restart,
            commands::mcp_recent_calls,
            commands::mcp_get_tool_definitions,
            commands::mcp_set_autostart,
            commands::get_activity_settings,
            commands::set_activity_settings,
            commands::hide_tray_popover,
            commands::show_main_window,
            commands::get_app_version,
            commands::relaunch_app,
            commands::mission_list,
            commands::mission_create,
            commands::mission_get,
            commands::mission_update,
            commands::mission_delete,
            commands::mission_get_progress,
            commands::mission_get_tree,
        ])
        .on_window_event(|window, event| {
            // Auto-hide tray popover when it loses focus (native popover behaviour).
            // 단, show 직후 grace period 안에 발생한 Focused(false) 는 무시한다 —
            // fullscreen Space 에서 OS 가 우리 팝오버에 focus 를 못 줘서 즉시
            // Focused(false) 가 튀는 시나리오를 막기 위함.
            if window.label() == "tray_popover" {
                if let tauri::WindowEvent::Focused(false) = event {
                    let elapsed = {
                        use std::sync::atomic::Ordering;
                        use std::time::{SystemTime, UNIX_EPOCH};
                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map(|d| d.as_millis() as u64)
                            .unwrap_or(0);
                        let shown = tray::POPOVER_SHOWN_AT_MS.load(Ordering::Relaxed);
                        now.saturating_sub(shown)
                    };
                    tracing::info!("Tray popover lost focus. Elapsed: {}ms, Grace: {}ms", elapsed, tray::POPOVER_AUTO_HIDE_GRACE_MS);
                    if elapsed >= tray::POPOVER_AUTO_HIDE_GRACE_MS {
                        let _ = window.hide();
                    }
                }
                return;
            }
            // Hide main window on close instead of destroying it (menu bar app pattern)
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
