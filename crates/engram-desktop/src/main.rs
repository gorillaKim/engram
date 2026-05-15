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
            commands::task_list,
            commands::task_set_status,
            commands::note_list,
            commands::note_get,
            commands::note_add,
            commands::note_resolve,
            commands::blocked_issues_graph,
            commands::mcp_status,
            commands::mcp_start,
            commands::mcp_stop,
            commands::mcp_restart,
            commands::mcp_recent_calls,
            commands::mcp_set_autostart,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let sup = window
                    .app_handle()
                    .state::<Arc<McpSupervisor>>()
                    .inner()
                    .clone();
                tauri::async_runtime::block_on(async move {
                    let _ = sup.stop().await;
                });
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
